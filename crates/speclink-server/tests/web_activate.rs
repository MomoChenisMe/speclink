//! The device approval page `/activate` (server-device-auth spec「核准頁 session
//! 保護且明確確認」). It requires a logged-in session — an unauthenticated visit
//! redirects to login and leaves the request unapproved; entering a valid user
//! code shows an explicit confirm step; unknown, used and expired user codes all
//! get one invalid response; the change-making POST is same-origin.

mod common;

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
fn an_unauthenticated_activation_query_preserves_a_valid_user_code() {
    let (base, identity) = server_with_user();
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");

    let response = agent()
        .get(&format!("{base}/activate?user_code={}", auth.user_code))
        .call()
        .expect("activation redirect");

    assert!((300..400).contains(&response.status()), "an unauthenticated visit redirects");
    assert_eq!(
        response.header("location"),
        Some(format!("/login?user_code={}", auth.user_code).as_str()),
        "the login redirect preserves only the valid device code"
    );
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "GET leaves the request pending");
}

#[test]
fn an_authenticated_activation_query_prefills_without_confirming() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");

    let (status, body) = status_body(
        agent()
            .get(&format!("{base}/activate?user_code={}", auth.user_code))
            .set("Cookie", &format!("speclink_session={session}"))
            .call(),
    );

    assert_eq!(status, 200, "the activation entry page renders");
    assert!(
        body.contains(&format!("name=\"user_code\" value=\"{}\"", auth.user_code)),
        "the device code is prefilled: {body}"
    );
    assert!(!body.contains("name=\"action\""), "GET does not skip to the confirm step: {body}");
    assert!(identity.device_is_pending(&auth.user_code).unwrap(), "GET leaves the request pending");
}

#[test]
fn a_missing_or_malformed_activation_query_is_not_reflected() {
    let (base, _identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let (status, blank) = status_body(
        agent()
            .get(&format!("{base}/activate"))
            .set("Cookie", &cookie)
            .call(),
    );
    assert_eq!(status, 200);
    assert!(
        blank.contains("<input type=\"text\" name=\"user_code\" required>"),
        "a direct visit keeps the field blank: {blank}"
    );

    let malicious = "%3Cscript%3Ealert%281%29%3C%2Fscript%3E";
    let (status, malformed) = status_body(
        agent()
            .get(&format!("{base}/activate?user_code={malicious}"))
            .set("Cookie", &cookie)
            .call(),
    );
    assert_eq!(status, 200);
    assert!(!malformed.contains("script"), "malformed input is not reflected: {malformed}");
    assert!(
        malformed.contains("<input type=\"text\" name=\"user_code\" required>"),
        "malformed input falls back to a blank field: {malformed}"
    );

    let response = agent()
        .get(&format!("{base}/activate?user_code={malicious}"))
        .call()
        .expect("activation redirect");
    assert_eq!(response.header("location"), Some("/login"), "malformed input is not carried to login");
}

#[test]
fn activation_get_does_not_disclose_user_code_state() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let used = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("used authorization");
    identity.approve_device(&used.user_code, "usr_other").unwrap();
    let expired = identity
        .create_device_authorization(Duration::seconds(5), Duration::seconds(-1))
        .expect("expired authorization");

    for code in ["ZZZZ-ZZZZ", used.user_code.as_str(), expired.user_code.as_str()] {
        let (status, body) = status_body(
            agent()
                .get(&format!("{base}/activate?user_code={code}"))
                .set("Cookie", &cookie)
                .call(),
        );
        assert_eq!(status, 200, "GET renders the same entry-page status for {code}");
        assert!(
            body.contains(&format!("name=\"user_code\" value=\"{code}\"")),
            "GET pre-fills without disclosing the authorization state: {body}"
        );
        assert!(!body.contains("name=\"action\""), "GET does not expose the confirm step: {body}");
    }
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
