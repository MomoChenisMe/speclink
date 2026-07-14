//! The device approval page `/activate` (server-device-auth spec「核准頁 session
//! 保護且明確確認」). It requires a logged-in session — an unauthenticated visit
//! redirects to login and leaves the request unapproved; entering a valid user
//! code shows an explicit confirm step; unknown, used and expired user codes all
//! get one invalid response; the change-making POST is same-origin.

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "approver@example.com";
const PASSWORD: &str = "correct-horse-battery";

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

fn status_body(result: Result<ureq::Response, ureq::Error>) -> (u16, String) {
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

/// Log in and return the session cookie value.
fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

#[test]
fn an_unauthenticated_visit_redirects_to_login() {
    let (base, _identity) = server_with_user();
    let (status, _) = status_body(agent().get(&format!("{base}/activate")).call());
    assert!((300..400).contains(&status), "an unauthenticated visit redirects to login, got {status}");
}

#[test]
fn an_unauthenticated_post_leaves_the_request_unapproved() {
    let (base, identity) = server_with_user();
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/activate"))
            .send_form(&[("user_code", auth.user_code.as_str()), ("action", "approve")]),
    );
    assert!((300..400).contains(&status), "an unauthenticated POST redirects, got {status}");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "the request stays pending");
}

#[test]
fn a_valid_user_code_shows_the_explicit_confirm_step() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, body) = status_body(
        agent()
            .post(&format!("{base}/activate"))
            .set("Cookie", &format!("speclink_session={session}"))
            .send_form(&[("user_code", auth.user_code.as_str())]),
    );
    assert_eq!(status, 200, "the confirm step renders");
    assert!(body.contains("核准") && body.contains("拒絕"), "the confirm step offers approve and deny: {body}");
    assert!(body.contains(&auth.user_code), "the confirm step echoes the user code");
    // Merely reaching the confirm step is not a decision — nothing is approved.
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "still pending before an explicit decision");
}

#[test]
fn approving_from_the_confirm_step_clears_pending() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/activate"))
            .set("Cookie", &format!("speclink_session={session}"))
            .send_form(&[("user_code", auth.user_code.as_str()), ("action", "approve")]),
    );
    assert_eq!(status, 200, "approve succeeds");
    assert!(!identity.device_is_pending(&auth.user_code).unwrap(), "the request is no longer pending after approval");
}

#[test]
fn unknown_used_and_expired_user_codes_get_one_invalid_response() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");
    let submit = |code: &str| {
        status_body(
            agent()
                .post(&format!("{base}/activate"))
                .set("Cookie", &cookie)
                .send_form(&[("user_code", code)]),
        )
    };

    // Unknown.
    let unknown = submit("ZZZZ-ZZZZ");
    // Already-decided (used): approved by someone else.
    let used_auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .unwrap();
    identity.approve_device(&used_auth.user_code, "usr_other").unwrap();
    let used = submit(&used_auth.user_code);
    // Expired.
    let expired_auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::seconds(-1))
        .unwrap();
    let expired = submit(&expired_auth.user_code);

    assert_eq!(unknown.0, used.0, "same status for unknown and used");
    assert_eq!(unknown.0, expired.0, "same status for unknown and expired");
    assert_eq!(unknown.1, used.1, "byte-identical body for unknown and used — the reason never leaks");
    assert_eq!(unknown.1, expired.1, "byte-identical body for unknown and expired");
}

#[test]
fn a_foreign_origin_decision_is_refused_and_leaves_the_request_pending() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/activate"))
            .set("Cookie", &format!("speclink_session={session}"))
            .set("Origin", "https://evil.example")
            .send_form(&[("user_code", auth.user_code.as_str()), ("action", "approve")]),
    );
    assert_eq!(status, 403, "a foreign-origin POST is refused");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "the request stays pending");
}
