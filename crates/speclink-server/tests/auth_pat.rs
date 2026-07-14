//! Bearer PAT authentication is per-request and its classification is sharp
//! (server-identity spec「bearer 驗證逐請求生效且分類明確」, 決策 5). A token the
//! identity store does not hold, a revoked or expired PAT, and a suspended
//! user's PAT all return 401 permission_denied indistinguishably; a valid PAT
//! whose user is not a member of the URL project returns 403; a valid PAT with
//! membership binds with the right actor and advances last-used.

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::binding::BindingResponse;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// Seed an identity store with a user (member of `demo`, not of `multi`) and a
/// PAT, start a server over it. Returns the base URL, the identity store, the
/// PAT plaintext, and the user id.
fn seed() -> (String, Arc<IdentitySqlite>, String, String) {
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
    let (_, pat) = identity.create_pat(&user_id, "cli", None).expect("pat");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::config_with_dual_repo_project()),
        identity: identity.clone(),
    };
    (common::start(state), identity, pat, user_id)
}

fn get_binding(
    base: &str,
    project: &str,
    token: &str,
    repo: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let url = format!("{base}/api/speclink/v1/projects/{project}/binding");
    let mut req = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", API_VERSION);
    if let Some(repo) = repo {
        req = req.set("X-Speclink-Repo", repo);
    }
    req.call()
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
fn an_unknown_token_is_401_permission_denied() {
    let (base, _identity, _pat, _user) = seed();
    let (status, err) = error_of(get_binding(&base, "demo", "spk_pat_deadbeef", Some("backend")));
    assert_eq!(status, 401);
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_revoked_pat_is_401_indistinguishable_from_unknown() {
    let (base, identity, _pat, user_id) = seed();
    let (_, revoked) = identity.create_pat(&user_id, "temp", None).expect("pat");
    let pat_id = identity.list_pats(&user_id).unwrap().iter().find(|p| p.name == "temp").unwrap().id.clone();
    identity.revoke_pat(&user_id, &pat_id).expect("revoke");

    let (rev_status, rev_err) = error_of(get_binding(&base, "demo", &revoked, Some("backend")));
    let (unk_status, unk_err) = error_of(get_binding(&base, "demo", "spk_pat_nope", Some("backend")));
    assert_eq!(rev_status, 401);
    assert_eq!(rev_status, unk_status, "revoked and unknown share a status");
    assert_eq!(rev_err.reason, unk_err.reason, "and share a reason — the cause is not distinguished");
    assert_eq!(rev_err.message, unk_err.message, "and share a message");
}

#[test]
fn an_expired_pat_is_401() {
    let (base, identity, _pat, user_id) = seed();
    let (_, expired) = identity.create_pat(&user_id, "old", Some(Utc::now() - Duration::hours(1))).expect("pat");
    let (status, err) = error_of(get_binding(&base, "demo", &expired, Some("backend")));
    assert_eq!(status, 401);
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_suspended_users_pat_is_401() {
    let (base, identity, pat, user_id) = seed();
    identity.set_user_active(&user_id, false).expect("suspend");
    let (status, err) = error_of(get_binding(&base, "demo", &pat, Some("backend")));
    assert_eq!(status, 401, "a suspended user's PAT is refused");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_valid_pat_on_a_non_member_project_is_403() {
    let (base, _identity, pat, _user) = seed();
    // The user is a member of `demo`, not of `multi`.
    let (status, err) = error_of(get_binding(&base, "multi", &pat, Some("web")));
    assert_eq!(status, 403, "a non-member is forbidden, distinct from the 401 of an invalid token");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_valid_pat_with_membership_binds_with_the_right_actor() {
    let (base, _identity, pat, user_id) = seed();
    let resp = get_binding(&base, "demo", &pat, Some("backend")).expect("a valid PAT binds");
    assert_eq!(resp.status(), 200);
    let binding: BindingResponse = serde_json::from_str(&resp.into_string().unwrap()).expect("binding");
    assert_eq!(binding.actor.id, user_id, "the actor is the PAT's owner");
    assert_eq!(binding.actor.name, "Dev <dev@example.com>");
    assert_eq!(binding.project.key, "demo");
}

#[test]
fn a_successful_request_advances_last_used() {
    let (base, identity, pat, user_id) = seed();
    assert!(identity.list_pats(&user_id).unwrap()[0].last_used_at.is_none(), "unused before the request");
    let resp = get_binding(&base, "demo", &pat, Some("backend")).expect("bind");
    assert_eq!(resp.status(), 200);
    assert!(
        identity.list_pats(&user_id).unwrap()[0].last_used_at.is_some(),
        "last-used advanced after a successful request"
    );
}
