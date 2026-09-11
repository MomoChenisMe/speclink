//! The device approval page `/activate` (server-device-auth spec「核准頁 session
//! 保護且明確確認」). It requires a logged-in session — an unauthenticated visit
//! redirects to login and leaves the request unapproved; entering a valid user
//! code shows an explicit confirm step; unknown, used and expired user codes all
//! get one invalid response; the change-making POST is same-origin.

use crate::common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "approver@example.com";
const PASSWORD: &str = "correct-horse-battery";
/// The origin matching `demo_config().public_url`.
const SAME_ORIGIN: &str = "http://127.0.0.1";

fn server_with_user() -> (String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: "Approver".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    identity.accept_invitation(&token, PASSWORD).expect("accept");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity)
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

/// Log in via the browser JSON API and return the session cookie value.
fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .set("Origin", SAME_ORIGIN)
        .send_json(json!({ "email": EMAIL, "password": PASSWORD }))
        .expect("login");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

// --- browser JSON activation API (server-device-auth「核准頁 session 保護且明確確認」,
// D2／D3) ---
//
// `POST /api/speclink/v1/web/activate` 需已登入 session 與同源。無 action 為明確確認的
// 「檢查」步驟（pending → 顯示核准／拒絕）；action approve／deny 記錄操作者身分。未知／
// 已用／逾期的 user code 一律不可區分。GET 不查狀態，故無 GET 端點。

fn json_of(result: Result<ureq::Response, ureq::Error>) -> (u16, Value) {
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_json().unwrap_or(Value::Null))
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_json().unwrap_or(Value::Null)),
        Err(e) => panic!("transport error: {e}"),
    }
}

fn activate(
    base: &str,
    body: Value,
    origin: Option<&str>,
    cookie: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().post(&format!("{base}/api/speclink/v1/web/activate"));
    if let Some(o) = origin {
        req = req.set("Origin", o);
    }
    if let Some(c) = cookie {
        req = req.set("Cookie", &format!("speclink_session={c}"));
    }
    req.send_json(body)
}

#[test]
fn a_json_activation_requires_a_session() {
    let (base, identity) = server_with_user();
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, _b) = json_of(activate(
        &base,
        json!({ "userCode": auth.user_code, "action": "approve" }),
        Some(SAME_ORIGIN),
        None,
    ));
    assert_eq!(status, 401, "an unauthenticated activation is refused");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "the request stays pending");
}

#[test]
fn a_json_confirm_then_approve_records_the_decision() {
    let (base, identity) = server_with_user();
    let cookie = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");

    // The confirm step (no action) checks the code without deciding.
    let (status, body) = json_of(activate(
        &base,
        json!({ "userCode": auth.user_code }),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert_eq!(status, 200, "a pending code confirms: {body}");
    assert_eq!(body["data"]["status"], json!("pending"), "the confirm step reports pending");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "the confirm step does not decide");

    // Approving records the decision.
    let (status, body) = json_of(activate(
        &base,
        json!({ "userCode": auth.user_code, "action": "approve" }),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["status"], json!("approved"));
    assert!(!identity.device_is_pending(&auth.user_code).unwrap(), "the device is no longer pending");
}

#[test]
fn a_json_deny_stops_the_device() {
    let (base, identity) = server_with_user();
    let cookie = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, body) = json_of(activate(
        &base,
        json!({ "userCode": auth.user_code, "action": "deny" }),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["status"], json!("denied"));
    assert!(!identity.device_is_pending(&auth.user_code).unwrap(), "the denied device is not pending");
}

#[test]
fn an_invalid_json_user_code_is_indistinguishable() {
    let (base, identity) = server_with_user();
    let cookie = login(&base);
    // Used (approved by another) and expired codes, plus an unknown one.
    let used = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device");
    identity.approve_device(&used.user_code, "usr_other").unwrap();
    let expired = identity
        .create_device_authorization(Duration::seconds(5), Duration::seconds(-1))
        .expect("device");

    let mut codes = ["ZZZZ-ZZZZ".to_string(), used.user_code.clone(), expired.user_code.clone()];
    codes.sort();
    let mut statuses = Vec::new();
    for code in &codes {
        let (status, _b) = json_of(activate(
            &base,
            json!({ "userCode": code }),
            Some(SAME_ORIGIN),
            Some(&cookie),
        ));
        statuses.push(status);
    }
    assert!(
        statuses.iter().all(|&s| s == statuses[0]),
        "unknown, used and expired user codes are indistinguishable, got {statuses:?}"
    );
    assert_eq!(statuses[0], 404, "an invalid user code is the invalid result");
}

#[test]
fn a_cross_origin_json_activation_is_refused() {
    let (base, identity) = server_with_user();
    let cookie = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, _b) = json_of(activate(
        &base,
        json!({ "userCode": auth.user_code, "action": "approve" }),
        Some("http://evil.example"),
        Some(&cookie),
    ));
    assert_eq!(status, 403, "a foreign-origin activation is refused");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "the request stays pending");
}
