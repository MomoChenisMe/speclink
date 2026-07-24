//! Browser session/login/logout JSON API（server-identity spec「本機密碼登入與
//! session 安全屬性」與 server-web-console「導向遵守伺服器裁決與安全優先序」,
//! 設計決策 D2／D3）。
//!
//! `/api/speclink/v1/web/*` 是獨立的 same-origin session-cookie API：成功回
//! `{data}`、失敗回 `{error:{code,message,fieldErrors?}}`，欄位 camelCase。所有
//! mutation 先做 Origin 同源檢查再解析 session。登入 destination 由 Server 依固定
//! 優先序裁決：有效 device userCode → 通過白名單的 returnTo → 角色 home；外部
//! returnTo 一律忽略，一般成員的 `/admin` destination 回 403 不降級。

mod common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const PASSWORD: &str = "pw-correct-horse";
/// A format-valid device user code (the `XXXX-XXXX` confusable-free alphabet).
const USER_CODE: &str = "ABCD-EFGH";
/// The origin matching `demo_config().public_url`.
const SAME_ORIGIN: &str = "http://127.0.0.1";

/// Seed a user with the given admin flag and a known password; returns its id.
fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("User <{email}>"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    identity.accept_invitation(&token, PASSWORD).expect("accept")
}

/// Start a server with a seeded admin and plain member; returns the base URL and
/// the shared identity store.
fn start() -> (String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    seed_user(&identity, "admin@example.com", true);
    seed_user(&identity, "member@example.com", false);
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

/// POST a JSON body to a web API path with an optional Origin and session cookie.
fn post(
    base: &str,
    path: &str,
    body: Value,
    origin: Option<&str>,
    cookie: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().post(&format!("{base}{path}"));
    if let Some(o) = origin {
        req = req.set("Origin", o);
    }
    if let Some(c) = cookie {
        req = req.set("Cookie", &format!("speclink_session={c}"));
    }
    req.send_json(body)
}

fn get(base: &str, path: &str, cookie: Option<&str>) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().get(&format!("{base}{path}"));
    if let Some(c) = cookie {
        req = req.set("Cookie", &format!("speclink_session={c}"));
    }
    req.call()
}

/// Status code and parsed JSON body of a response (Ok or error status).
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

/// Log in and return the whole response (so a test can inspect Set-Cookie).
fn login_raw(base: &str, email: &str, extra: Value) -> Result<ureq::Response, ureq::Error> {
    let mut body = json!({ "email": email, "password": PASSWORD });
    if let Value::Object(fields) = extra {
        for (k, v) in fields {
            body[k] = v;
        }
    }
    post(base, "/api/speclink/v1/web/login", body, Some(SAME_ORIGIN), None)
}

/// The session id from a login response's Set-Cookie header.
fn cookie_of(resp: &ureq::Response) -> String {
    resp.header("set-cookie")
        .expect("a Set-Cookie header")
        .split(';')
        .next()
        .unwrap()
        .trim()
        .strip_prefix("speclink_session=")
        .expect("the session cookie")
        .to_string()
}

// --- session read ---

#[test]
fn session_is_unauthenticated_without_a_cookie() {
    let (base, _id) = start();
    let (status, body) = json_of(get(&base, "/api/speclink/v1/web/session", None));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["authenticated"], json!(false));
    assert_eq!(body["data"]["user"], Value::Null);
    assert_eq!(body["data"]["home"], json!("/login"));
}

#[test]
fn session_reports_the_user_and_role_home() {
    let (base, _id) = start();
    let admin_cookie = cookie_of(&login_raw(&base, "admin@example.com", json!({})).expect("login"));
    let (status, body) = json_of(get(
        &base,
        "/api/speclink/v1/web/session",
        Some(&admin_cookie),
    ));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["authenticated"], json!(true));
    assert_eq!(body["data"]["user"]["email"], json!("admin@example.com"));
    assert_eq!(body["data"]["user"]["admin"], json!(true));
    assert!(body["data"]["user"]["id"].is_string());
    assert!(body["data"]["user"]["display"].is_string());
    assert_eq!(body["data"]["home"], json!("/admin"), "admin home is /admin");
}

// --- cookie security attributes ---

#[test]
fn login_sets_a_hardened_session_cookie() {
    let (base, _id) = start();
    let resp = login_raw(&base, "member@example.com", json!({})).expect("login");
    let set_cookie = resp.header("set-cookie").expect("Set-Cookie").to_string();
    assert!(set_cookie.contains("speclink_session="));
    assert!(set_cookie.contains("HttpOnly"), "cookie is HttpOnly: {set_cookie}");
    assert!(set_cookie.contains("Secure"), "cookie is Secure: {set_cookie}");
    assert!(
        set_cookie.contains("SameSite=Strict"),
        "cookie is SameSite=Strict: {set_cookie}"
    );
}

// --- same-origin guard ---

#[test]
fn a_cross_origin_login_is_refused_before_authentication() {
    let (base, _id) = start();
    let body = json!({ "email": "admin@example.com", "password": PASSWORD });
    let (status, _b) = json_of(post(
        &base,
        "/api/speclink/v1/web/login",
        body,
        Some("http://evil.example"),
        None,
    ));
    assert_eq!(status, 403, "a foreign-origin mutation is 403");
}

// --- email non-enumerability ---

#[test]
fn login_failure_does_not_reveal_whether_the_email_exists() {
    let (base, _id) = start();
    // Unknown email vs a real email with the wrong password: identical outcome.
    let (unknown_status, unknown_body) = json_of(post(
        &base,
        "/api/speclink/v1/web/login",
        json!({ "email": "nobody@example.com", "password": PASSWORD }),
        Some(SAME_ORIGIN),
        None,
    ));
    let (wrong_status, wrong_body) = json_of(post(
        &base,
        "/api/speclink/v1/web/login",
        json!({ "email": "admin@example.com", "password": "wrong-password" }),
        Some(SAME_ORIGIN),
        None,
    ));
    assert_eq!(unknown_status, 401);
    assert_eq!(wrong_status, 401);
    assert_eq!(
        unknown_body["error"]["message"], wrong_body["error"]["message"],
        "the message must not distinguish an unknown email from a wrong password"
    );
    assert_eq!(unknown_body["error"]["code"], wrong_body["error"]["code"]);
}

// --- server-computed destination priority (D3) ---

#[test]
fn role_home_is_returned_when_there_is_no_code_or_return_to() {
    let (base, _id) = start();
    let (_s, admin_body) = json_of(login_raw(&base, "admin@example.com", json!({})));
    let (_s, member_body) = json_of(login_raw(&base, "member@example.com", json!({})));
    assert_eq!(admin_body["data"]["destination"], json!("/admin"));
    assert_eq!(member_body["data"]["destination"], json!("/account"));
}

#[test]
fn a_safe_return_to_takes_priority_over_role_home() {
    let (base, _id) = start();
    let (status, body) = json_of(login_raw(
        &base,
        "admin@example.com",
        json!({ "returnTo": "/account" }),
    ));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["destination"], json!("/account"));
}

#[test]
fn a_valid_device_code_takes_priority_over_return_to() {
    let (base, _id) = start();
    let (status, body) = json_of(login_raw(
        &base,
        "admin@example.com",
        json!({ "userCode": USER_CODE, "returnTo": "/account" }),
    ));
    assert_eq!(status, 200);
    assert_eq!(
        body["data"]["destination"],
        json!(format!("/activate?user_code={USER_CODE}")),
        "device activation outranks the return path"
    );
}

#[test]
fn an_external_return_to_is_ignored() {
    let (base, _id) = start();
    for evil in ["https://evil.example/path", "//evil.example/path"] {
        let (status, body) = json_of(login_raw(
            &base,
            "admin@example.com",
            json!({ "returnTo": evil }),
        ));
        assert_eq!(status, 200, "login still succeeds for {evil}");
        assert_eq!(
            body["data"]["destination"], json!("/admin"),
            "an external returnTo ({evil}) is ignored in favour of role home"
        );
    }
}

#[test]
fn a_return_to_outside_the_whitelisted_prefixes_is_ignored() {
    let (base, _id) = start();
    // First segment not in {account, activate, admin}.
    let (status, body) = json_of(login_raw(
        &base,
        "member@example.com",
        json!({ "returnTo": "/setup" }),
    ));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["destination"], json!("/account"));
}

#[test]
fn a_dot_dot_traversal_return_to_cannot_walk_past_the_whitelist() {
    let (base, _id) = start();
    // A first segment of `account` must not be walkable to `/admin` via `..` — the
    // whitelist rejects any traversal, so a member falls to their role home.
    for evil in ["/account/../admin", "/account/..%2fadmin", "/admin/../account/../admin"] {
        let (status, body) = json_of(login_raw(
            &base,
            "member@example.com",
            json!({ "returnTo": evil }),
        ));
        assert_eq!(status, 200, "login still succeeds for {evil}");
        assert_eq!(
            body["data"]["destination"], json!("/account"),
            "a `..` traversal returnTo ({evil}) is ignored in favour of role home"
        );
    }
}

#[test]
fn a_member_cannot_use_return_to_to_reach_admin() {
    let (base, _id) = start();
    let (status, body) = json_of(login_raw(
        &base,
        "member@example.com",
        json!({ "returnTo": "/admin" }),
    ));
    assert_eq!(status, 403, "a member's /admin destination is refused, not downgraded");
    assert_eq!(body["data"], Value::Null, "no destination is returned");
}

// --- logout invalidates the session ---

#[test]
fn logout_revokes_the_session_server_side() {
    let (base, _id) = start();
    let cookie = cookie_of(&login_raw(&base, "member@example.com", json!({})).expect("login"));
    // The session is live before logout.
    let (before, _b) = json_of(get(&base, "/api/speclink/v1/web/session", Some(&cookie)));
    assert_eq!(before, 200);
    let (logout_status, _lb) = json_of(post(
        &base,
        "/api/speclink/v1/web/logout",
        json!({}),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert_eq!(logout_status, 200);
    // After logout the same cookie is no longer authenticated.
    let (_after, after_body) = json_of(get(&base, "/api/speclink/v1/web/session", Some(&cookie)));
    assert_eq!(
        after_body["data"]["authenticated"], json!(false),
        "the revoked session is no longer authenticated"
    );
}
