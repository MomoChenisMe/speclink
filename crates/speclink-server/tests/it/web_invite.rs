//! The invite acceptance page (server-identity spec「邀請一次性且到期失效」,
//! 決策 4). A valid token shows a set-password form; submitting it atomically
//! creates an active user with the invited memberships and consumes the
//! invitation. A used, expired or unknown token yields the same "邀請無效" page
//! (404), without distinguishing the reason.

use crate::common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// The origin matching `demo_config().public_url`.
const SAME_ORIGIN: &str = "http://127.0.0.1";

/// An identity store seeded with one invitation carrying `admin`, plus a running
/// server over it. Returns the server base URL, the invitation token, and the
/// identity store (so the test can inspect the outcome).
fn seeded_flag(days: i64, admin: bool) -> (String, String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: "invitee@example.com".to_string(),
            display: "Invitee".to_string(),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(days),
        })
        .expect("seed invitation");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    let base = common::start(state);
    (base, token, identity)
}

/// A ureq agent that does not follow redirects, so we observe the immediate
/// response.
fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

// --- browser JSON invite API (server-identity「邀請一次性且到期失效」, D2／D3) ---
//
// `GET /api/speclink/v1/web/invite/{token}` 回非祕密邀請摘要供設定密碼表單；`POST`
// 同源提交後原子建立 active user（含 memberships）並耗用邀請，接著建立該 user 的 Web
// session，回 Server 裁決的 destination（admin→`/admin`，一般→`/account`）。已用／過期
// ／未知 token 一律不可區分的 404，且不建 session。

fn get_json(base: &str, path: &str) -> Result<ureq::Response, ureq::Error> {
    agent().get(&format!("{base}{path}")).call()
}

fn get_json_cookie(base: &str, path: &str, session: &str) -> Result<ureq::Response, ureq::Error> {
    agent()
        .get(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .call()
}

fn post_json(
    base: &str,
    path: &str,
    body: Value,
    origin: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().post(&format!("{base}{path}"));
    if let Some(o) = origin {
        req = req.set("Origin", o);
    }
    req.send_json(body)
}

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

/// The session id from a response's Set-Cookie header, if present.
fn session_cookie(resp: &ureq::Response) -> Option<String> {
    resp.header("set-cookie")?
        .split(';')
        .next()?
        .trim()
        .strip_prefix("speclink_session=")
        .map(str::to_string)
}

#[test]
fn json_invite_summary_then_accept_creates_the_user_and_logs_in() {
    let (base, token, identity) = seeded_flag(7, false);

    // GET the non-secret summary for the set-password form.
    let (status, body) = json_of(get_json(&base, &format!("/api/speclink/v1/web/invite/{token}")));
    assert_eq!(status, 200, "a valid invitation yields its summary");
    assert_eq!(body["data"]["email"], json!("invitee@example.com"));
    assert_eq!(body["data"]["admin"], json!(false));

    // POST accepts the invitation: same-origin, creates the user and opens a session.
    let resp = post_json(
        &base,
        &format!("/api/speclink/v1/web/invite/{token}"),
        json!({ "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    )
    .expect("accept");
    let session = session_cookie(&resp).expect("acceptance sets a session cookie");
    let body: Value = resp.into_json().unwrap();
    assert_eq!(
        body["data"]["destination"], json!("/account"),
        "a member invitation lands on /account"
    );

    // The user exists, is active, and carries the invited membership.
    let user = identity
        .find_user_by_email("invitee@example.com")
        .unwrap()
        .expect("the account was created");
    assert!(user.active, "the created user is active");
    assert!(identity.is_member(&user.id, "demo").unwrap(), "the invited membership is granted");

    // The session cookie authenticates as that user.
    let (s, session_body) = json_of(get_json_cookie(
        &base,
        "/api/speclink/v1/web/session",
        &session,
    ));
    assert_eq!(s, 200);
    assert_eq!(session_body["data"]["authenticated"], json!(true), "acceptance logs the user in");
    assert_eq!(session_body["data"]["user"]["email"], json!("invitee@example.com"));

    // The invitation is consumed: the summary is now the invalid result.
    let (again, _b) = json_of(get_json(&base, &format!("/api/speclink/v1/web/invite/{token}")));
    assert_eq!(again, 404, "a consumed invitation is indistinguishable from an invalid one");
}

#[test]
fn a_json_admin_invitation_lands_on_admin() {
    let (base, token, _id) = seeded_flag(7, true);
    let resp = post_json(
        &base,
        &format!("/api/speclink/v1/web/invite/{token}"),
        json!({ "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    )
    .expect("accept");
    let body: Value = resp.into_json().unwrap();
    assert_eq!(
        body["data"]["destination"], json!("/admin"),
        "an admin invitation lands on /admin"
    );
}

#[test]
fn a_json_invite_that_is_expired_is_indistinguishable_and_creates_nothing() {
    let (base, token, identity) = seeded_flag(-1, false);

    // The expired token's summary and an unknown token share one code and status.
    let (expired_status, expired_body) =
        json_of(get_json(&base, &format!("/api/speclink/v1/web/invite/{token}")));
    assert_eq!(expired_status, 404);
    let (unknown_status, unknown_body) =
        json_of(get_json(&base, "/api/speclink/v1/web/invite/does-not-exist"));
    assert_eq!(unknown_status, 404);
    assert_eq!(
        expired_body["error"]["code"], unknown_body["error"]["code"],
        "expired and unknown invitations are indistinguishable"
    );

    // Posting to the expired token creates neither a user nor a session.
    let (post_status, _b) = json_of(post_json(
        &base,
        &format!("/api/speclink/v1/web/invite/{token}"),
        json!({ "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    ));
    assert_eq!(post_status, 404);
    assert!(
        identity
            .find_user_by_email("invitee@example.com")
            .unwrap()
            .is_none(),
        "no user is created from an expired invitation"
    );
}

#[test]
fn a_cross_origin_json_invite_acceptance_is_refused() {
    let (base, token, identity) = seeded_flag(7, false);
    let (status, _b) = json_of(post_json(
        &base,
        &format!("/api/speclink/v1/web/invite/{token}"),
        json!({ "password": "hunter2password" }),
        Some("http://evil.example"),
    ));
    assert_eq!(status, 403, "a foreign-origin acceptance is refused");
    assert!(
        identity
            .find_user_by_email("invitee@example.com")
            .unwrap()
            .is_none(),
        "nothing is created by a cross-origin acceptance"
    );
}
