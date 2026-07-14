//! Refresh rotation and family revocation (server-device-auth spec「refresh
//! rotation 與 family 撤銷」, 決策 3). A refresh credential is one-time: rotating
//! kills the old value and issues a fresh pair; reusing a spent value tears down
//! the whole family, including the access token minted alongside; the revoke
//! endpoint (logout) tears down the family too.

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::device::RefreshResponse;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{
    DevicePoll, IdentitySqlite, IdentityStore, NewInvitation, RefreshOutcome, TokenPair,
};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

fn seed() -> (String, Arc<IdentitySqlite>, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
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
    // The registry now lives in the identity store; register `demo` (repo backend).
    common::seed_demo_registry(&*identity);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, user_id)
}

/// Mint an initial token pair through the approval flow at the store level.
fn mint_pair(identity: &IdentitySqlite, approver: &str) -> TokenPair {
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(identity.approve_device(&auth.user_code, approver).expect("approve"));
    match identity.poll_device(&auth.device_code).expect("poll") {
        DevicePoll::Approved(pair) => pair,
        other => panic!("expected approved, got {other:?}"),
    }
}

/// Whether an access token binds against the demo project.
fn binds(base: &str, access_token: &str) -> bool {
    ureq::get(&format!("{base}/api/speclink/v1/projects/demo/binding"))
        .set("Authorization", &format!("Bearer {access_token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}

// --- store-level rotation semantics ---

#[test]
fn rotation_kills_the_old_refresh_and_issues_a_fresh_pair() {
    let (_base, identity, user_id) = seed();
    let pair1 = mint_pair(&identity, &user_id);
    let pair2 = match identity.refresh(&pair1.refresh_token).expect("refresh") {
        RefreshOutcome::Rotated(p) => p,
        other => panic!("expected rotated, got {other:?}"),
    };
    assert_ne!(pair1.refresh_token, pair2.refresh_token, "a fresh refresh credential is issued");
    assert_ne!(pair1.access_token, pair2.access_token, "a fresh access token is issued");
    assert!(identity.authenticate_access_token(&pair2.access_token).unwrap().is_some(), "the new access token is live");
    // The old refresh is spent: presenting it again is a reuse signal.
    assert!(matches!(identity.refresh(&pair1.refresh_token).unwrap(), RefreshOutcome::Reused), "the old refresh is dead");
}

#[test]
fn reusing_a_rotated_refresh_tears_down_the_whole_family() {
    let (_base, identity, user_id) = seed();
    let pair1 = mint_pair(&identity, &user_id);
    let pair2 = match identity.refresh(&pair1.refresh_token).unwrap() {
        RefreshOutcome::Rotated(p) => p,
        o => panic!("expected rotated, got {o:?}"),
    };
    // Reuse the spent old refresh → the whole family is revoked.
    assert!(matches!(identity.refresh(&pair1.refresh_token).unwrap(), RefreshOutcome::Reused));
    assert!(
        identity.authenticate_access_token(&pair2.access_token).unwrap().is_none(),
        "the rotated access token is revoked with the family"
    );
    assert!(
        matches!(identity.refresh(&pair2.refresh_token).unwrap(), RefreshOutcome::Reused),
        "the rotated refresh is revoked with the family"
    );
}

#[test]
fn revoke_by_refresh_tears_down_the_family_and_unknown_is_not_recognized() {
    let (_base, identity, user_id) = seed();
    let pair = mint_pair(&identity, &user_id);
    assert!(identity.revoke_family_by_refresh(&pair.refresh_token).unwrap(), "the refresh is recognized");
    assert!(identity.authenticate_access_token(&pair.access_token).unwrap().is_none(), "the access token is revoked");
    assert!(!identity.revoke_family_by_refresh("spk_rt_nope").unwrap(), "an unknown refresh is not recognized");
}

// --- HTTP endpoints ---

#[test]
fn the_refresh_endpoint_rotates_and_reuse_is_401_and_kills_the_new_access_token() {
    let (base, identity, user_id) = seed();
    let pair1 = mint_pair(&identity, &user_id);

    let rotated: RefreshResponse = serde_json::from_str(
        &ureq::post(&format!("{base}/auth/refresh"))
            .send_json(serde_json::json!({ "refreshToken": pair1.refresh_token }))
            .expect("refresh rotates")
            .into_string()
            .unwrap(),
    )
    .unwrap();
    assert!(rotated.access_token.starts_with("spk_at_"), "a new access token is returned");
    assert!(binds(&base, &rotated.access_token), "the rotated access token binds");

    // Reuse the old refresh → 401.
    match ureq::post(&format!("{base}/auth/refresh"))
        .send_json(serde_json::json!({ "refreshToken": pair1.refresh_token }))
        .expect_err("reuse errors")
    {
        ureq::Error::Status(code, resp) => {
            assert_eq!(code, 401);
            let err: ErrorResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
            assert_eq!(err.reason, ErrorReason::PermissionDenied);
        }
        e => panic!("transport error: {e}"),
    }
    // The reuse teardown revoked the whole family, including the new access token.
    assert!(!binds(&base, &rotated.access_token), "the rotated access token is dead after the family teardown");
}

#[test]
fn the_revoke_endpoint_logs_the_device_out() {
    let (base, identity, user_id) = seed();
    let pair = mint_pair(&identity, &user_id);
    assert!(binds(&base, &pair.access_token), "the access token binds before revoke");
    let resp = ureq::post(&format!("{base}/auth/revoke"))
        .send_json(serde_json::json!({ "refreshToken": pair.refresh_token }))
        .expect("revoke");
    assert_eq!(resp.status(), 200);
    assert!(!binds(&base, &pair.access_token), "the access token is dead after revoke");
}

#[test]
fn an_unknown_refresh_or_revoke_is_401() {
    let (base, _identity, _user_id) = seed();
    for path in ["auth/refresh", "auth/revoke"] {
        match ureq::post(&format!("{base}/{path}"))
            .send_json(serde_json::json!({ "refreshToken": "spk_rt_unknown" }))
            .expect_err("an unknown refresh errors")
        {
            ureq::Error::Status(code, _) => assert_eq!(code, 401, "{path} rejects an unknown refresh with 401"),
            e => panic!("transport error: {e}"),
        }
    }
}
