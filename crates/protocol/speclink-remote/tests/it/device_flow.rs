//! Device flow typed client (design 決策 1「device flow client 落 speclink-remote」、
//! 決策 3「登入前探測＝直接 POST /auth/device」): initiate／poll／refresh／revoke
//! walk a real in-process speclink-server (memory identity, tempdir sqlite
//! store), statuses come back as the protocol's typed states, and the probe
//! semantics are baked into `initiate` — 404/405 is the explicit Unsupported
//! signal (PAT fallback), a 5xx stays an error and never a fallback.
//!
//! Approval and denial are driven directly against the identity store, the
//! way the /activate page would act — the browser step itself belongs to the
//! desktop orchestration knife.

use chrono::{Duration, Utc};
use speclink_protocol::device::DeviceTokenStatus;
use speclink_remote::device::{self, InitiateOutcome};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use std::sync::Arc;

// --- in-process server harness (memory identity, tempdir sqlite store) ---

struct TestServer {
    base: String,
    identity: Arc<IdentitySqlite>,
    user_id: String,
    _dir: tempfile::TempDir,
}

/// Start a real speclink-server on a loopback port: memory identity seeded
/// with one member of `demo`, a sqlite team store in a tempdir. The server
/// thread lives for the rest of the test process.
fn server() -> TestServer {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = speclink_server::build_store(&StoreConfig::Sqlite {
        path: dir.path().join("store.db"),
    })
    .expect("tempdir sqlite store");

    let identity = Arc::new(IdentitySqlite::open_memory().expect("memory identity store"));
    let invite = identity
        .create_invitation(NewInvitation {
            email: "dev@example.com".to_string(),
            display: "Dev <dev@example.com>".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&invite, "pw-correct-horse").expect("accept");
    identity.create_project("demo", "Demo").expect("seed demo project");

    let events = EventHub::new(store.clone(), EventSettings::default());
    let config = ServerConfig {
        store: StoreConfig::Memory,
        identity: IdentityConfig::Memory,
        public_url: "http://127.0.0.1".to_string(),
        events: EventSettings::default(),
    };
    let state = AppState { store, identity: identity.clone(), config: Arc::new(config), events };

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    listener.set_nonblocking(true).expect("nonblocking");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
            axum::serve(listener, speclink_server::app::router(state)).await.expect("serve");
        });
    });

    TestServer { base: format!("http://{addr}"), identity, user_id, _dir: dir }
}

/// Initiate against the live server, asserting the supported outcome.
fn initiate(t: &TestServer) -> speclink_protocol::device::DeviceAuthorizationResponse {
    match device::initiate(&t.base).expect("initiate") {
        InitiateOutcome::Supported(auth) => auth,
        InitiateOutcome::Unsupported => panic!("a real server supports the device flow"),
    }
}

// --- initiation ---

#[test]
fn initiate_returns_codes_the_verification_uri_and_the_poll_interval() {
    let t = server();
    let auth = initiate(&t);
    assert!(auth.device_code.starts_with("spk_dc_"), "device code carries its prefix: {}", auth.device_code);
    assert!(auth.user_code.contains('-'), "user code is grouped for human entry: {}", auth.user_code);
    assert!(auth.verification_uri.ends_with("/activate"), "the URI points at the approval page: {}", auth.verification_uri);
    assert!(auth.expires_in > 0, "an expiry is declared");
    assert!(auth.interval >= 1, "the minimum poll interval is declared");
}

// --- the polling state machine, typed end to end ---

#[test]
fn poll_before_approval_is_pending_and_an_immediate_repoll_is_slow_down() {
    let t = server();
    let auth = initiate(&t);
    let first = device::poll(&t.base, &auth.device_code).expect("poll");
    assert_eq!(first.status, DeviceTokenStatus::Pending);
    assert!(first.access_token.is_none(), "a pending poll carries no token");
    // Polling again inside the declared interval is the typed slow_down —
    // the signal the orchestrator respects, never a wire error.
    let repoll = device::poll(&t.base, &auth.device_code).expect("re-poll");
    assert_eq!(repoll.status, DeviceTokenStatus::SlowDown);
    assert!(repoll.access_token.is_none());
}

#[test]
fn poll_after_approval_is_approved_with_the_token_pair() {
    let t = server();
    let auth = initiate(&t);
    // Approve directly against the identity store — what /activate does.
    assert!(t.identity.approve_device(&auth.user_code, &t.user_id).expect("approve"));
    let granted = device::poll(&t.base, &auth.device_code).expect("poll");
    assert_eq!(granted.status, DeviceTokenStatus::Approved);
    let access = granted.access_token.expect("access token");
    let refresh = granted.refresh_token.expect("refresh token");
    assert!(access.starts_with("spk_at_"), "access token prefix: {access}");
    assert!(refresh.starts_with("spk_rt_"), "refresh token prefix: {refresh}");
    assert!(granted.expires_in.is_some(), "the access token lifetime travels with approval");
    assert!(
        t.identity.authenticate_access_token(&access).expect("authenticate").is_some(),
        "the granted access token is live and bound to the approver"
    );
}

#[test]
fn a_browser_denial_polls_denied() {
    let t = server();
    let auth = initiate(&t);
    assert!(t.identity.deny_device(&auth.user_code, &t.user_id).expect("deny"));
    let denied = device::poll(&t.base, &auth.device_code).expect("poll");
    assert_eq!(denied.status, DeviceTokenStatus::Denied);
    assert!(denied.access_token.is_none(), "a denial leaves no credential behind");
}

#[test]
fn an_expired_authorization_polls_expired() {
    let t = server();
    // The HTTP endpoint mints a fixed TTL; expiry is driven at the store
    // level with an already-past deadline, then observed over the wire.
    let auth = t
        .identity
        .create_device_authorization(Duration::seconds(5), Duration::seconds(-1))
        .expect("initiate at the store level");
    let expired = device::poll(&t.base, &auth.device_code).expect("poll");
    assert_eq!(expired.status, DeviceTokenStatus::Expired);
}

// --- refresh rotation and revoke ---

#[test]
fn refresh_rotates_the_pair_and_reusing_the_old_credential_is_refused() {
    let t = server();
    let auth = initiate(&t);
    assert!(t.identity.approve_device(&auth.user_code, &t.user_id).expect("approve"));
    let granted = device::poll(&t.base, &auth.device_code).expect("poll");
    let old_refresh = granted.refresh_token.expect("refresh token");

    let rotated = device::refresh(&t.base, &old_refresh).expect("rotation");
    assert_ne!(rotated.refresh_token, old_refresh, "a fresh refresh credential is issued");
    assert!(rotated.access_token.starts_with("spk_at_"));

    // Reusing the spent credential is refused — and the reuse signal tears
    // down the whole family, killing the rotated access token with it.
    let err = device::refresh(&t.base, &old_refresh).expect_err("reuse is refused");
    assert_eq!(err.reason.as_deref(), Some("permission_denied"));
    assert!(
        t.identity.authenticate_access_token(&rotated.access_token).expect("authenticate").is_none(),
        "family revocation killed the rotated access token"
    );
}

#[test]
fn revoke_tears_down_the_family() {
    let t = server();
    let auth = initiate(&t);
    assert!(t.identity.approve_device(&auth.user_code, &t.user_id).expect("approve"));
    let granted = device::poll(&t.base, &auth.device_code).expect("poll");
    let access = granted.access_token.expect("access token");
    let refresh = granted.refresh_token.expect("refresh token");

    device::revoke(&t.base, &refresh).expect("revoke");
    assert!(
        t.identity.authenticate_access_token(&access).expect("authenticate").is_none(),
        "logout revoked the family, killing the access token"
    );
    // Revoking again with the same known credential is idempotent — logging
    // out twice never errors. Only an unknown credential is a refusal.
    device::revoke(&t.base, &refresh).expect("revoke is idempotent for a known credential");
    let err = device::revoke(&t.base, "spk_rt_unknown").expect_err("an unknown credential is refused");
    assert_eq!(err.reason.as_deref(), Some("permission_denied"));
}

// --- root-level identity (決策 8): the display name behind a bearer ---

#[test]
fn whoami_reads_the_approvers_display_with_the_access_token() {
    let t = server();
    let auth = initiate(&t);
    assert!(t.identity.approve_device(&auth.user_code, &t.user_id).expect("approve"));
    let granted = device::poll(&t.base, &auth.device_code).expect("poll");
    let access = granted.access_token.expect("access token");

    let who = device::whoami(&t.base, &access).expect("whoami");
    assert_eq!(who.user.name, "Dev <dev@example.com>", "the identity display travels back");
    assert_eq!(who.user.handle, t.user_id);

    // An invalid bearer is the registry's refusal, typed by reason.
    let err = device::whoami(&t.base, "spk_at_nope").expect_err("an unknown bearer is refused");
    assert_eq!(err.reason.as_deref(), Some("permission_denied"));
}

// --- the probe semantics (決策 3): 404/405 is Unsupported, a 5xx stays an error ---

/// Serve every request with one fixed status/body on a loopback port — the
/// probe cases need a server *without* the device endpoints.
fn fixed_server(status: u16, body: &'static str) -> (Arc<tiny_http::Server>, String) {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip addr").port();
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let _ = req.respond(tiny_http::Response::from_string(body).with_status_code(status));
        }
    });
    (server, format!("http://127.0.0.1:{port}"))
}

#[test]
fn initiate_maps_404_and_405_to_the_unsupported_signal() {
    for status in [404u16, 405] {
        let (server, base) = fixed_server(status, "");
        let outcome = device::initiate(&base).expect("a probe miss is an outcome, not an error");
        assert!(
            matches!(outcome, InitiateOutcome::Unsupported),
            "HTTP {status} at the initiation endpoint is the explicit PAT-fallback signal"
        );
        server.unblock();
    }
}

#[test]
fn initiate_5xx_is_a_connection_error_and_never_a_fallback_signal() {
    let (server, base) = fixed_server(503, r#"{"status":503,"reason":"unavailable","message":"maintenance"}"#);
    let err = device::initiate(&base).expect_err("a 5xx is an error");
    assert!(
        err.message.contains("unavailable") || err.message.contains("unreachable"),
        "the error reads as a connection problem: {}",
        err.message
    );
    server.unblock();
}

#[test]
fn initiate_transport_failure_is_a_connection_error() {
    // A port nothing listens on: the transport itself fails.
    let err = device::initiate("http://127.0.0.1:1").expect_err("refused connection is an error");
    assert!(err.reason.is_none(), "a transport failure carries no wire reason");
}
