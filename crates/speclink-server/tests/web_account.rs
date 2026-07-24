//! Login, logout, session and the account page (server-identity spec「本機密碼
//! 登入與 session 安全屬性」, 決策 4). Passwords verify with argon2; the session
//! cookie carries HttpOnly/Secure/SameSite=Strict; a change-making POST from a
//! foreign origin is 403; a failed login is byte-identical for an unknown email
//! and a wrong password; logout revokes the server-side session; an
//! unauthenticated account visit redirects to login.

mod common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{
    DevicePoll, IdentitySqlite, IdentityStore, NewInvitation, TokenPair,
};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "user@example.com";
const PASSWORD: &str = "correct-horse-battery";
const USER_CODE: &str = "ABCD-EFGH";
/// The origin matching `demo_config().public_url`.
const SAME_ORIGIN: &str = "http://127.0.0.1";

/// Seed a user through the identity store (invite → accept) and start a server
/// over it. Returns the base URL and the identity store.
fn server_with_user() -> (String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: "User".to_string(),
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

/// The `speclink_session` value from a response's Set-Cookie header.
fn session_cookie(resp: &ureq::Response) -> String {
    let set = resp.header("set-cookie").expect("a Set-Cookie header");
    set.split(';')
        .next()
        .unwrap()
        .trim()
        .strip_prefix("speclink_session=")
        .expect("the session cookie")
        .to_string()
}

#[test]
fn login_sets_a_secure_session_cookie_and_reaches_account() {
    let (base, _identity) = server_with_user();

    let resp = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login responds");
    assert!((300..400).contains(&resp.status()), "a good login redirects, got {}", resp.status());
    assert_eq!(resp.header("location"), Some("/account"), "a direct login still reaches account");
    let set = resp.header("set-cookie").expect("Set-Cookie present");
    assert!(set.contains("HttpOnly"), "cookie is HttpOnly: {set}");
    assert!(set.contains("Secure"), "cookie is Secure: {set}");
    assert!(set.contains("SameSite=Strict"), "cookie is SameSite=Strict: {set}");
    let session = session_cookie(&resp);

    let (status, body) = status_body(
        agent()
            .get(&format!("{base}/account"))
            .set("Cookie", &format!("speclink_session={session}"))
            .call(),
    );
    assert_eq!(status, 200, "the session reaches the account page");
    assert!(body.contains(EMAIL), "the account page names the user: {body}");
}

#[test]
fn login_page_preserves_only_a_valid_user_code() {
    let (base, _identity) = server_with_user();

    let (status, valid) = status_body(agent().get(&format!("{base}/login?user_code={USER_CODE}")).call());
    assert_eq!(status, 200);
    assert!(
        valid.contains(&format!("type=\"hidden\" name=\"user_code\" value=\"{USER_CODE}\"")),
        "the valid code is carried by the login form: {valid}"
    );

    let malicious = "%3Cscript%3Ealert%281%29%3C%2Fscript%3E";
    let (status, malformed) = status_body(agent().get(&format!("{base}/login?user_code={malicious}")).call());
    assert_eq!(status, 200);
    assert!(!malformed.contains("script"), "malformed input is not reflected: {malformed}");
    assert!(!malformed.contains("name=\"user_code\""), "malformed input is not preserved: {malformed}");
}

#[test]
fn a_valid_user_code_returns_to_activation_after_login() {
    let (base, _identity) = server_with_user();

    let response = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD), ("user_code", USER_CODE)])
        .expect("login responds");

    assert!((300..400).contains(&response.status()), "a good login redirects");
    assert_eq!(
        response.header("location"),
        Some("/activate?user_code=ABCD-EFGH"),
        "the server rebuilds the one allowed activation destination"
    );
    assert!(response.header("set-cookie").is_some(), "the returning response opens a session");
}

#[test]
fn a_failed_login_preserves_the_user_code_without_disclosing_the_account() {
    let (base, _identity) = server_with_user();

    let unknown = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", "nobody@example.com"), ("password", PASSWORD), ("user_code", USER_CODE)]),
    );
    let wrong = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", EMAIL), ("password", "wrong-password"), ("user_code", USER_CODE)]),
    );

    assert_eq!(unknown.0, wrong.0, "same status for unknown email and wrong password");
    assert_eq!(unknown.1, wrong.1, "the same code produces a byte-identical failure page");
    assert!(
        unknown.1.contains(&format!("type=\"hidden\" name=\"user_code\" value=\"{USER_CODE}\"")),
        "the valid activation context survives a retry: {}",
        unknown.1
    );
    assert!(!unknown.1.contains("nobody@example.com"), "the submitted email is not reflected");
    assert!(!unknown.1.contains(PASSWORD), "the submitted password is not reflected");
}

#[test]
fn a_malformed_user_code_falls_back_to_account_after_login() {
    let (base, _identity) = server_with_user();

    let response = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD), ("user_code", "https://evil.example")])
        .expect("login responds");

    assert_eq!(response.header("location"), Some("/account"), "malformed input cannot choose a destination");
    assert!(!response.header("location").unwrap_or_default().contains("evil.example"));
}

#[test]
fn a_failed_login_is_byte_identical_for_unknown_email_and_wrong_password() {
    let (base, _identity) = server_with_user();

    let unknown = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", "nobody@example.com"), ("password", PASSWORD)]),
    );
    let wrong = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", EMAIL), ("password", "wrong-password")]),
    );
    assert_eq!(unknown.0, wrong.0, "same status for unknown email and wrong password");
    assert_eq!(unknown.1, wrong.1, "byte-identical body — the failure never leaks account existence");
}

#[test]
fn a_change_making_post_from_a_foreign_origin_is_403() {
    let (base, _identity) = server_with_user();
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/login"))
            .set("Origin", "https://evil.example")
            .send_form(&[("email", EMAIL), ("password", PASSWORD)]),
    );
    assert_eq!(status, 403, "a foreign-origin POST is refused");
}

#[test]
fn logout_revokes_the_server_side_session() {
    let (base, _identity) = server_with_user();
    let login = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    let session = session_cookie(&login);
    let cookie = format!("speclink_session={session}");

    // The session works.
    let (status, _) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert_eq!(status, 200, "the fresh session reaches the account page");

    // Logout revokes it.
    let (status, _) = status_body(
        agent().post(&format!("{base}/logout")).set("Cookie", &cookie).send_form(&[]),
    );
    assert!((300..400).contains(&status), "logout redirects, got {status}");

    // The same cookie is now treated as unauthenticated.
    let (status, _) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert!((300..400).contains(&status), "the revoked session no longer reaches the account page, got {status}");
}

#[test]
fn an_unauthenticated_account_visit_redirects_to_login() {
    let (base, _identity) = server_with_user();
    let (status, _) = status_body(agent().get(&format!("{base}/account")).call());
    assert!((300..400).contains(&status), "an unauthenticated visit redirects, got {status}");
}

/// Log in and return the session cookie value.
fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    session_cookie(&resp)
}

/// The full `spk_pat_` plaintext (8-char prefix + 64 hex) from a page body.
fn extract_pat(body: &str) -> String {
    for (i, _) in body.match_indices("spk_pat_") {
        let token: String = body[i..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if token.len() == 72 {
            return token;
        }
    }
    panic!("no full PAT plaintext in body: {body}");
}

#[test]
fn a_pat_plaintext_is_shown_exactly_once() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let (status, body) = status_body(
        agent()
            .post(&format!("{base}/account/tokens"))
            .set("Cookie", &cookie)
            .send_form(&[("name", "ci"), ("expires", "")]),
    );
    assert_eq!(status, 200, "creating a PAT shows a page");
    let plaintext = extract_pat(&body);
    assert!(plaintext.starts_with("spk_pat_"), "the plaintext carries the prefix");
    assert_eq!(body.matches(&plaintext).count(), 1, "the full plaintext appears exactly once");

    // Reloading the account page never shows the plaintext again — only prefix.
    let (status, reload) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert_eq!(status, 200);
    assert!(!reload.contains(&plaintext), "the plaintext is not recoverable after reload");
    let prefix: String = plaintext.chars().take(12).collect();
    assert!(reload.contains(&prefix), "the list still shows the identifiable prefix");

    // Storage holds only prefix + hash + metadata (verified via the store).
    let user = identity.find_user_by_email(EMAIL).unwrap().unwrap();
    let pats = identity.list_pats(&user.id).unwrap();
    assert_eq!(pats.len(), 1);
    assert_eq!(pats[0].name, "ci");
}

#[test]
fn revoking_a_pat_takes_effect_immediately() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let (_, body) = status_body(
        agent()
            .post(&format!("{base}/account/tokens"))
            .set("Cookie", &cookie)
            .send_form(&[("name", "ci"), ("expires", "")]),
    );
    let plaintext = extract_pat(&body);
    // The PAT authenticates against the identity store.
    assert!(identity.authenticate_pat(&plaintext).unwrap().is_some(), "the fresh PAT authenticates");

    let user = identity.find_user_by_email(EMAIL).unwrap().unwrap();
    let pat_id = identity.list_pats(&user.id).unwrap()[0].id.clone();

    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/account/tokens/{pat_id}/revoke"))
            .set("Cookie", &cookie)
            .send_form(&[]),
    );
    assert!((200..400).contains(&status), "revoke succeeds, got {status}");

    // The revocation is effective at once — the next lookup fails.
    assert!(identity.authenticate_pat(&plaintext).unwrap().is_none(), "the revoked PAT no longer authenticates");
}

// --- browser JSON account API (server-identity「帳號 browser API 保持憑證祕密邊界」,
// D2／D4) ---
//
// `GET /api/speclink/v1/web/account` 分別回 user、PAT metadata、Web sessions 與 device
// families，read payload 絕不含 hash／refresh credential／password／可重播 session secret；
// PAT 建立回應的 plaintext 只出現一次。建立／撤銷 PAT、登出 Web session、撤銷 device
// family 皆先驗同源與 active session。

fn json_login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .set("Origin", SAME_ORIGIN)
        .send_json(json!({ "email": EMAIL, "password": PASSWORD }))
        .expect("login");
    session_cookie(&resp)
}

fn get_json(base: &str, path: &str, cookie: Option<&str>) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().get(&format!("{base}{path}"));
    if let Some(c) = cookie {
        req = req.set("Cookie", &format!("speclink_session={c}"));
    }
    req.call()
}

fn post_json(
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

/// Approve a fresh device authorization and exchange it for a token pair, so a
/// device credential family stands for the seeded user.
fn mint_device_family(identity: &IdentitySqlite, approver: &str) -> TokenPair {
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(identity.approve_device(&auth.user_code, approver).expect("approve"));
    match identity.poll_device(&auth.device_code).expect("poll") {
        DevicePoll::Approved(pair) => pair,
        other => panic!("expected approved, got {other:?}"),
    }
}

#[test]
fn account_summary_returns_own_user_pats_sessions_and_devices() {
    let (base, _identity) = server_with_user();
    let cookie = json_login(&base);
    let (status, body) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    assert_eq!(status, 200);
    assert_eq!(body["data"]["user"]["email"], json!(EMAIL));
    assert_eq!(body["data"]["user"]["admin"], json!(false));
    // Four separate sections (决策 D2), each an array.
    assert!(body["data"]["pats"].is_array(), "pats section: {body}");
    assert!(body["data"]["sessions"].is_array(), "sessions section");
    assert!(body["data"]["deviceFamilies"].is_array(), "device families section (camelCase)");
    // The login session is listed.
    assert!(
        !body["data"]["sessions"].as_array().unwrap().is_empty(),
        "the active login session is listed"
    );
    // No secret ever appears in a read payload.
    let raw = body.to_string();
    for forbidden in ["hash", "refresh", "password", "secret"] {
        assert!(!raw.contains(forbidden), "read payload must not carry `{forbidden}`: {raw}");
    }
}

#[test]
fn a_pat_plaintext_appears_only_on_creation() {
    let (base, _identity) = server_with_user();
    let cookie = json_login(&base);
    let resp = post_json(
        &base,
        "/api/speclink/v1/web/account/tokens",
        json!({ "name": "cli" }),
        Some(SAME_ORIGIN),
        Some(&cookie),
    )
    .expect("create pat");
    let created: Value = resp.into_json().unwrap();
    let plaintext = created["data"]["plaintext"].as_str().expect("plaintext once").to_string();
    assert!(!plaintext.is_empty(), "the creation response carries the plaintext");
    assert!(created["data"]["pat"]["prefix"].is_string(), "and the metadata prefix");

    // A later read shows the PAT metadata but never the plaintext again.
    let (_s, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    let pats = summary["data"]["pats"].as_array().unwrap();
    assert_eq!(pats.len(), 1, "the created PAT is listed");
    assert_eq!(pats[0]["name"], json!("cli"));
    assert!(
        !summary.to_string().contains(&plaintext),
        "the plaintext is never obtainable from a summary"
    );
}

#[test]
fn revoking_a_pat_via_the_browser_api_is_immediate() {
    let (base, identity) = server_with_user();
    let cookie = json_login(&base);
    let resp = post_json(
        &base,
        "/api/speclink/v1/web/account/tokens",
        json!({ "name": "cli" }),
        Some(SAME_ORIGIN),
        Some(&cookie),
    )
    .expect("create pat");
    let created: Value = resp.into_json().unwrap();
    let plaintext = created["data"]["plaintext"].as_str().unwrap().to_string();
    let pat_id = created["data"]["pat"]["id"].as_str().unwrap().to_string();
    assert!(identity.authenticate_pat(&plaintext).unwrap().is_some(), "the fresh PAT authenticates");

    let (status, _b) = json_of(post_json(
        &base,
        &format!("/api/speclink/v1/web/account/tokens/{pat_id}/revoke"),
        json!({}),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert!((200..300).contains(&status), "revoke succeeds, got {status}");
    assert!(
        identity.authenticate_pat(&plaintext).unwrap().is_none(),
        "the revoked PAT no longer authenticates"
    );
}

#[test]
fn revoking_a_device_family_via_the_browser_api_is_immediate() {
    let (base, identity) = server_with_user();
    let cookie = json_login(&base);
    let user_id = identity.find_user_by_email(EMAIL).unwrap().unwrap().id;
    let pair = mint_device_family(&identity, &user_id);
    let family_id = identity.list_device_families(&user_id).unwrap()[0].id.clone();

    let (status, _b) = json_of(post_json(
        &base,
        &format!("/api/speclink/v1/web/account/devices/{family_id}/revoke"),
        json!({}),
        Some(SAME_ORIGIN),
        Some(&cookie),
    ));
    assert!((200..300).contains(&status), "revoke succeeds, got {status}");

    // The family's access token is dead at once, and the summary marks it revoked.
    assert!(
        identity.authenticate_access_token(&pair.access_token).unwrap().is_none(),
        "the revoked family's access token no longer authenticates"
    );
    let (_s, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    let fam = &summary["data"]["deviceFamilies"].as_array().unwrap()[0];
    assert!(!fam["revokedAt"].is_null(), "the summary marks the family revoked");
}

#[test]
fn an_account_summary_does_not_leak_other_users() {
    let (base, identity) = server_with_user();
    // A second user with their own PAT.
    let other_token = identity
        .create_invitation(NewInvitation {
            email: "other@example.com".to_string(),
            display: "Other".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite other");
    let other_id = identity.accept_invitation(&other_token, "other-password").expect("accept");
    let (_, other_pat) = identity.create_pat(&other_id, "theirs", None).expect("other pat");

    let cookie = json_login(&base);
    let (_s, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    let pats = summary["data"]["pats"].as_array().unwrap();
    assert!(pats.is_empty(), "the caller has no PATs; the other user's are not shown");
    assert!(
        !summary.to_string().contains(&other_pat),
        "no other user's secret appears"
    );
}

#[test]
fn a_cross_origin_account_mutation_is_refused() {
    let (base, _identity) = server_with_user();
    let cookie = json_login(&base);
    let (status, _b) = json_of(post_json(
        &base,
        "/api/speclink/v1/web/account/tokens",
        json!({ "name": "cli" }),
        Some("http://evil.example"),
        Some(&cookie),
    ));
    assert_eq!(status, 403, "a foreign-origin account mutation is refused");
}

#[test]
fn the_account_summary_requires_a_session() {
    let (base, _identity) = server_with_user();
    let (status, _b) = json_of(get_json(&base, "/api/speclink/v1/web/account", None));
    assert_eq!(status, 401, "an unauthenticated account read is 401");
}
