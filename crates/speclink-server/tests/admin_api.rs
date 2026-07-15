//! The admin gate is a flag check layered on the existing authentication
//! (server-admin spec「admin 門禁前置且非 admin 一律 403」, 決策 1). The admin API
//! (bearer) and the /admin pages (session) both authenticate first, then require
//! the user's admin flag: a non-admin is 403 permission_denied and no action
//! runs; an admin passes through either entry; a suspended admin loses access on
//! the very next request; and the admin API runs the same API version check as
//! every other API route.

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// Seed a user with `email`/`display`, the given `admin` flag and a membership of
/// `demo`, then mint both a PAT and a session for it. Returns `(pat, session,
/// user_id)`.
fn seed_user(
    identity: &Arc<IdentitySqlite>,
    email: &str,
    display: &str,
    admin: bool,
) -> (String, String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: display.to_string(),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    let (_, pat) = identity.create_pat(&user_id, "cli", None).expect("pat");
    let session = identity.create_session(&user_id, Duration::days(1)).expect("session");
    (pat, session, user_id)
}

/// Start a server with an admin user and a plain member seeded; returns the base
/// URL, the shared identity store (to mutate state mid-test), and both users'
/// `(pat, session, user_id)` triples.
#[allow(clippy::type_complexity)]
fn start() -> (
    String,
    Arc<IdentitySqlite>,
    (String, String, String),
    (String, String, String),
) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin = seed_user(&identity, "admin@example.com", "Admin <admin@example.com>", true);
    let member = seed_user(&identity, "member@example.com", "Member <member@example.com>", false);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, admin, member)
}

// --- request helpers ---

/// `GET` an admin API path with an optional bearer token and API version header.
fn get_api(base: &str, path: &str, bearer: Option<&str>, version: Option<&str>) -> Result<ureq::Response, ureq::Error> {
    let agent = ureq::builder().redirects(0).build();
    let mut req = agent.get(&format!("{base}{path}"));
    if let Some(b) = bearer {
        req = req.set("Authorization", &format!("Bearer {b}"));
    }
    if let Some(v) = version {
        req = req.set("X-Speclink-Api-Version", v);
    }
    req.call()
}

/// `GET` a /admin page with an optional session cookie; returns the HTTP status,
/// following no redirects (a login redirect is meaningful here).
fn get_page_status(base: &str, path: &str, session: Option<&str>) -> u16 {
    let agent = ureq::builder().redirects(0).build();
    let mut req = agent.get(&format!("{base}{path}"));
    if let Some(s) = session {
        req = req.set("Cookie", &format!("speclink_session={s}"));
    }
    match req.call() {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error: {e}"),
    }
}

fn error_of(result: Result<ureq::Response, ureq::Error>) -> (u16, ErrorResponse) {
    match result {
        Ok(resp) => panic!("expected a protocol error, got HTTP {}", resp.status()),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            let err = serde_json::from_str::<ErrorResponse>(&body)
                .unwrap_or_else(|_| panic!("body is an ErrorResponse envelope, got: {body}"));
            (code, err)
        }
        Err(e) => panic!("transport error: {e}"),
    }
}

// --- the gate ---

#[test]
fn a_non_admin_pat_is_403_from_the_admin_api() {
    let (base, _identity, _admin, member) = start();
    let (member_pat, _, _) = member;
    let (status, err) = error_of(get_api(&base, "/api/speclink/v1/admin/audit", Some(&member_pat), Some(API_VERSION)));
    assert_eq!(status, 403, "a valid-but-non-admin token is forbidden");
    assert_eq!(err.reason, ErrorReason::PermissionDenied, "the reason is the reused permission_denied");
}

#[test]
fn a_missing_bearer_is_401_from_the_admin_api() {
    let (base, _identity, _admin, _member) = start();
    let (status, err) = error_of(get_api(&base, "/api/speclink/v1/admin/audit", None, Some(API_VERSION)));
    assert_eq!(status, 401, "no token authenticates to nothing");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_non_admin_session_is_403_from_the_admin_pages() {
    let (base, _identity, _admin, member) = start();
    let (_, member_session, _) = member;
    let status = get_page_status(&base, "/admin", Some(&member_session));
    assert_eq!(status, 403, "a logged-in non-admin may not enter the management pages");
}

#[test]
fn an_admin_passes_the_gate_via_both_bearer_and_session() {
    let (base, _identity, admin, _member) = start();
    let (admin_pat, admin_session, _) = admin;
    let resp = get_api(&base, "/api/speclink/v1/admin/audit", Some(&admin_pat), Some(API_VERSION))
        .expect("an admin bearer passes the API gate");
    assert_eq!(resp.status(), 200, "the admin API answers an admin bearer");
    let page = get_page_status(&base, "/admin", Some(&admin_session));
    assert_eq!(page, 200, "the /admin page answers an admin session");
}

#[test]
fn a_suspended_admin_loses_access_on_the_next_request() {
    let (base, identity, admin, _member) = start();
    let (admin_pat, admin_session, admin_id) = admin;
    // The admin could enter before suspension.
    assert_eq!(get_page_status(&base, "/admin", Some(&admin_session)), 200, "admin enters before suspension");
    // Suspend the admin; the very next request must lose the management面.
    identity.set_user_active(&admin_id, false).expect("suspend");
    assert_ne!(
        get_page_status(&base, "/admin", Some(&admin_session)),
        200,
        "a suspended admin's session no longer reaches /admin"
    );
    let (status, _err) = error_of(get_api(&base, "/api/speclink/v1/admin/audit", Some(&admin_pat), Some(API_VERSION)));
    assert_eq!(status, 401, "a suspended admin's bearer no longer authenticates");
}

#[test]
fn the_admin_api_enforces_the_api_version_check() {
    let (base, _identity, admin, _member) = start();
    let (admin_pat, _, _) = admin;
    // No version header: refused, like every other API route.
    let (status, err) = error_of(get_api(&base, "/api/speclink/v1/admin/audit", Some(&admin_pat), None));
    assert_eq!(status, 409, "a missing API version is refused");
    assert_eq!(err.reason, ErrorReason::Refused);
    // A wrong version: refused too.
    let (bad_status, _) = error_of(get_api(&base, "/api/speclink/v1/admin/audit", Some(&admin_pat), Some("v0-bogus")));
    assert_eq!(bad_status, 409, "an incompatible API version is refused");
}
