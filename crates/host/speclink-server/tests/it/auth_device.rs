//! Device access tokens in the bearer precondition (server-device-auth spec
//! 「access token 短效且併入 bearer 前置」, 決策 4). An access token minted by the
//! approval flow binds as the approver and drives the same routes as an
//! equal-permission PAT; an expired one is refused before any verb; a suspended
//! user's access token stops authenticating at once.

use crate::common;

use chrono::{Duration, SecondsFormat, Utc};
use speclink_protocol::binding::BindingResponse;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{DevicePoll, IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::path::PathBuf;
use std::sync::Arc;

/// Seed a file-backed identity store with a user who is a member of `demo`, and
/// start a server over it. Returns the base URL, the store, its file path (for
/// raw expiry manipulation), the tempdir (kept alive), and the user id.
fn seed() -> (String, Arc<IdentitySqlite>, PathBuf, tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("identity.db");
    let identity = Arc::new(IdentitySqlite::open(&path).expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: "dev@example.com".to_string(),
            display: "Dev <dev@example.com>".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    // The registry now lives in the identity store: seed `demo` (the user's
    // membership) and `multi` (registered but not a membership, for the 403 path).
    common::seed_demo_registry(&*identity);
    common::seed_multi_project(&*identity);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, path, dir, user_id)
}

/// Run the full approval flow at the store level and return the minted access
/// token's plaintext, bound to `approver`.
fn mint_access_token(identity: &IdentitySqlite, approver: &str) -> String {
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(identity.approve_device(&auth.user_code, approver).expect("approve"), "the pending request approves");
    match identity.poll_device(&auth.device_code).expect("poll") {
        DevicePoll::Approved(pair) => {
            assert!(pair.access_token.starts_with("spk_at_"), "the access token carries its prefix");
            assert!(pair.refresh_token.starts_with("spk_rt_"), "the refresh credential carries its prefix");
            pair.access_token
        }
        other => panic!("approved poll must mint a pair, got {other:?}"),
    }
}

fn get_binding(base: &str, token: &str) -> Result<ureq::Response, ureq::Error> {
    ureq::get(&format!("{base}/api/speclink/v1/projects/demo/binding"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
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

#[test]
fn an_access_token_binds_as_the_approver_and_queries_like_a_pat() {
    let (base, identity, _path, _dir, user_id) = seed();
    let token = mint_access_token(&identity, &user_id);

    // binding: the actor is the approver's identity.
    let resp = get_binding(&base, &token).expect("an access token binds");
    assert_eq!(resp.status(), 200);
    let binding: BindingResponse = serde_json::from_str(&resp.into_string().unwrap()).expect("binding");
    assert_eq!(binding.actor.id, user_id, "the actor is the approver");
    assert_eq!(binding.actor.name, "Dev <dev@example.com>");

    // a query route answers normally, exactly like an equal-permission PAT.
    let changes = ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("a query with an access token succeeds");
    assert_eq!(changes.status(), 200, "the query route answers normally");
}

#[test]
fn a_valid_access_token_on_a_non_member_project_is_403() {
    let (base, identity, _path, _dir, user_id) = seed();
    let token = mint_access_token(&identity, &user_id);
    // The approver is a member of `demo`, not of `multi` — a non-member is 403,
    // distinct from the 401 of an invalid token (决策 4, shared with PATs).
    let (status, err) = error_of(
        ureq::get(&format!("{base}/api/speclink/v1/projects/multi/binding"))
            .set("Authorization", &format!("Bearer {token}"))
            .set("X-Speclink-Api-Version", API_VERSION)
            .set("X-Speclink-Repo", "web")
            .call(),
    );
    assert_eq!(status, 403, "a valid access token whose user is not a member is forbidden");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_suspended_users_access_token_is_401_at_once() {
    let (base, identity, _path, _dir, user_id) = seed();
    let token = mint_access_token(&identity, &user_id);
    assert_eq!(get_binding(&base, &token).unwrap().status(), 200, "valid before suspension");

    identity.set_user_active(&user_id, false).expect("suspend");
    let (status, err) = error_of(get_binding(&base, &token));
    assert_eq!(status, 401, "a suspended user's device credential fails immediately");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn an_expired_access_token_is_refused_before_any_verb() {
    let (base, identity, path, _dir, user_id) = seed();
    let token = mint_access_token(&identity, &user_id);
    assert_eq!(get_binding(&base, &token).unwrap().status(), 200, "valid before expiry");

    // Force the short-lived access token past its expiry via a second connection
    // to the same file — the auth precondition must then refuse it.
    let conn = rusqlite::Connection::open(&path).expect("raw connection");
    conn.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
    let past = (Utc::now() - Duration::hours(2)).to_rfc3339_opts(SecondsFormat::Micros, true);
    conn.execute("UPDATE access_tokens SET expires_at = ?1", rusqlite::params![past]).expect("expire the token");
    drop(conn);

    let (status, err) = error_of(get_binding(&base, &token));
    assert_eq!(status, 401, "an expired access token is refused (no verb runs behind the precondition)");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}
