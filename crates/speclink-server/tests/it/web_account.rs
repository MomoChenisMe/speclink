//! Login, logout, session and the account page (server-identity spec「本機密碼
//! 登入與 session 安全屬性」, 決策 4). Passwords verify with argon2; the session
//! cookie carries HttpOnly/Secure/SameSite=Strict; a change-making POST from a
//! foreign origin is 403; a failed login is byte-identical for an unknown email
//! and a wrong password; logout revokes the server-side session; an
//! unauthenticated account visit redirects to login.

use crate::common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::audit::{AuditActor, AuditSource};
use speclink_server::identity::{
    DevicePoll, IdentitySqlite, IdentityStore, MembershipRole, NewInvitation, TokenPair,
};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "user@example.com";
const PASSWORD: &str = "correct-horse-battery";
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

/// Start a server whose seeded user holds exactly `memberships` (project key,
/// display name, role), with every one of those projects registered so the
/// summary can resolve a display name. Returns the base URL, the identity store
/// and the seeded user's id.
fn server_with_memberships(
    memberships: &[(&str, &str, MembershipRole)],
    admin: bool,
) -> (String, Arc<IdentitySqlite>, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    for (key, name, _) in memberships {
        identity.create_project(key, name).expect("register project");
    }
    let token = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: "User".to_string(),
            memberships: memberships.iter().map(|(key, _, _)| (*key).to_string()).collect(),
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, PASSWORD).expect("accept");
    // An invitation grants editor; set each membership's intended role.
    let actor = AuditActor::user(user_id.clone(), AuditSource::Web);
    for (key, _, role) in memberships {
        identity
            .admin_set_membership(&actor, &user_id, key, *role, true)
            .expect("set role");
    }
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, user_id)
}

/// The summary's memberships as `(projectKey, projectName, role)` triples.
fn membership_rows(summary: &Value) -> Vec<(String, String, String)> {
    summary["data"]["memberships"]
        .as_array()
        .unwrap_or_else(|| panic!("a memberships array: {summary}"))
        .iter()
        .map(|m| {
            (
                m["projectKey"].as_str().unwrap_or_default().to_string(),
                m["projectName"].as_str().unwrap_or_default().to_string(),
                m["role"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
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

// --- own project memberships in the summary (server-identity「帳號 browser API
// 保持憑證祕密邊界」, 決策一) ---

#[test]
fn the_summary_lists_the_callers_own_memberships() {
    let (base, _identity, _user_id) = server_with_memberships(
        &[
            ("alpha", "Alpha 專案", MembershipRole::Editor),
            ("beta", "Beta 專案", MembershipRole::Reader),
        ],
        false,
    );
    let cookie = json_login(&base);
    let (status, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    assert_eq!(status, 200);
    let mut rows = membership_rows(&summary);
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("alpha".to_string(), "Alpha 專案".to_string(), "editor".to_string()),
            ("beta".to_string(), "Beta 專案".to_string(), "reader".to_string()),
        ],
        "each membership carries the project key, display name and role (camelCase)"
    );
    // The added section keeps the read payload's secret boundary.
    let raw = summary.to_string();
    for forbidden in ["hash", "refresh", "password", "secret"] {
        assert!(!raw.contains(forbidden), "read payload must not carry `{forbidden}`: {raw}");
    }
}

#[test]
fn the_summary_lists_no_memberships_as_an_empty_array() {
    let (base, _identity, _user_id) = server_with_memberships(&[], false);
    let cookie = json_login(&base);
    let (_s, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    assert_eq!(
        membership_rows(&summary),
        Vec::new(),
        "a user with no membership gets an empty array, not a missing section"
    );
}

#[test]
fn an_admins_summary_lists_own_memberships_not_every_project() {
    let (base, identity, _user_id) =
        server_with_memberships(&[("alpha", "Alpha 專案", MembershipRole::Editor)], true);
    // A project the admin governs but is not a member of.
    identity.create_project("gamma", "Gamma 專案").expect("register project");

    let cookie = json_login(&base);
    let (_s, summary) = json_of(get_json(&base, "/api/speclink/v1/web/account", Some(&cookie)));
    assert_eq!(summary["data"]["user"]["admin"], json!(true), "the caller is an admin");
    assert_eq!(
        membership_rows(&summary),
        vec![("alpha".to_string(), "Alpha 專案".to_string(), "editor".to_string())],
        "an admin sees their own membership, not the whole registry"
    );
}

#[test]
fn the_account_summary_requires_a_session() {
    let (base, _identity) = server_with_user();
    let (status, _b) = json_of(get_json(&base, "/api/speclink/v1/web/account", None));
    assert_eq!(status, 401, "an unauthenticated account read is 401");
}
