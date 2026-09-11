//! The device authorization flow: initiation and the polling state machine
//! (server-device-auth spec「device 授權發起與輪詢狀態機」). Two codes are minted
//! and only hashed; polling reports pending/slow_down/expired/denied as typed
//! states (决策 1); an unknown device code is a not_found wire error.

use crate::common;

use chrono::{Duration, Utc};
use speclink_protocol::device::{DeviceAuthorizationResponse, DeviceTokenResponse, DeviceTokenStatus};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::identity::{DevicePoll, IdentitySqlite, IdentityStore};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

fn store() -> IdentitySqlite {
    IdentitySqlite::open_memory().expect("in-memory identity store")
}

// --- store-level state machine (deterministic, no HTTP timing) ---

#[test]
fn initiation_mints_two_distinct_codes_and_the_device_code_is_the_poll_key() {
    let s = store();
    let a = s
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(a.device_code.starts_with("spk_dc_"), "device code carries its prefix: {}", a.device_code);
    assert!(a.user_code.contains('-'), "user code is grouped for reading: {}", a.user_code);
    assert_ne!(a.device_code, a.user_code, "the two codes are distinct");
    assert!(a.expires_at > Utc::now(), "expiry is in the future");
    // The device code is the poll key; a fresh authorization is pending.
    assert!(matches!(s.poll_device(&a.device_code).unwrap(), DevicePoll::Pending));
    // The user code is not a device code — it is never accepted for polling.
    assert!(matches!(s.poll_device(&a.user_code).unwrap(), DevicePoll::NotFound));
}

#[test]
fn polling_before_approval_is_pending_then_slow_down_without_invalidating() {
    let s = store();
    // A large interval makes the slow_down deterministic without a real sleep.
    let a = s
        .create_device_authorization(Duration::hours(1), Duration::minutes(15))
        .expect("initiate");
    assert!(matches!(s.poll_device(&a.device_code).unwrap(), DevicePoll::Pending), "first poll pending");
    assert!(matches!(s.poll_device(&a.device_code).unwrap(), DevicePoll::SlowDown), "immediate re-poll slow_down");
    // The slow_down did not consume or invalidate the request: it can still act.
    assert!(s.deny_device(&a.user_code, "usr_approver").expect("deny"), "the request survived slow_down");
}

#[test]
fn an_expired_authorization_polls_expired() {
    let s = store();
    let a = s
        .create_device_authorization(Duration::seconds(5), Duration::seconds(-1))
        .expect("initiate");
    assert!(matches!(s.poll_device(&a.device_code).unwrap(), DevicePoll::Expired));
}

#[test]
fn a_denied_authorization_polls_denied_and_denial_is_terminal() {
    let s = store();
    let a = s
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(s.deny_device(&a.user_code, "usr_x").expect("deny"), "a pending request denies");
    assert!(matches!(s.poll_device(&a.device_code).unwrap(), DevicePoll::Denied));
    // An already-terminal or unknown user code does not deny again — the same
    // false the approval page renders as one invalid response.
    assert!(!s.deny_device(&a.user_code, "usr_x").expect("second deny"), "a denied request is terminal");
    assert!(!s.deny_device("ZZZZ-ZZZZ", "usr_x").expect("unknown deny"), "an unknown user code does not deny");
}

#[test]
fn an_unknown_device_code_polls_not_found() {
    let s = store();
    assert!(matches!(s.poll_device("spk_dc_nope").unwrap(), DevicePoll::NotFound));
}

// --- HTTP endpoints ---

fn server() -> (String, Arc<IdentitySqlite>) {
    let identity = Arc::new(store());
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity)
}

/// Initiate over HTTP and return the parsed response.
fn initiate(base: &str) -> DeviceAuthorizationResponse {
    let resp = ureq::post(&format!("{base}/auth/device")).call().expect("initiate");
    assert_eq!(resp.status(), 200);
    serde_json::from_str(&resp.into_string().unwrap()).expect("init body")
}

#[test]
fn the_initiation_endpoint_returns_codes_a_uri_and_polling_metadata() {
    let (base, _identity) = server();
    let body = initiate(&base);
    assert!(!body.device_code.is_empty() && !body.user_code.is_empty(), "both codes present");
    assert!(
        body.verification_uri.ends_with("/activate"),
        "the URI points at the approval page: {}",
        body.verification_uri
    );
    assert_eq!(body.expires_in, 900, "default 15-minute expiry");
    assert!(body.interval >= 1, "a minimum poll interval is declared");
}

#[test]
fn polling_a_fresh_authorization_is_pending_over_http() {
    let (base, _identity) = server();
    let init = initiate(&base);
    let resp = ureq::post(&format!("{base}/auth/device/token"))
        .send_json(serde_json::json!({ "deviceCode": init.device_code }))
        .expect("poll");
    assert_eq!(resp.status(), 200);
    let body: DeviceTokenResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body.status, DeviceTokenStatus::Pending);
    assert!(body.access_token.is_none(), "a pending poll carries no token");
}

#[test]
fn an_unknown_device_code_is_a_not_found_wire_error() {
    let (base, _identity) = server();
    match ureq::post(&format!("{base}/auth/device/token"))
        .send_json(serde_json::json!({ "deviceCode": "spk_dc_unknown" }))
        .expect_err("unknown device code errors")
    {
        ureq::Error::Status(code, resp) => {
            assert_eq!(code, 404);
            let body: ErrorResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
            assert_eq!(body.reason, ErrorReason::NotFound, "an unknown device code is a not_found wire error");
        }
        e => panic!("transport error: {e}"),
    }
}

#[test]
fn a_blank_device_code_is_an_invalid_argument_wire_error() {
    let (base, _identity) = server();
    match ureq::post(&format!("{base}/auth/device/token"))
        .send_json(serde_json::json!({ "deviceCode": "" }))
        .expect_err("a blank device code errors")
    {
        ureq::Error::Status(code, resp) => {
            assert_eq!(code, 400);
            let body: ErrorResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
            assert_eq!(body.reason, ErrorReason::InvalidArgument);
        }
        e => panic!("transport error: {e}"),
    }
}
