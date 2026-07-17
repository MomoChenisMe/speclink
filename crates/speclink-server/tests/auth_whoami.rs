//! Root-level bearer identity (server-device-auth spec「root 層 bearer 身分
//! 查詢」, connection-registry-keychain 決策 8): GET /auth/whoami resolves a
//! bearer exactly like the Binding extractor's first step — access token or
//! PAT, every failure the same 401 — without requiring a project scope, an
//! API version header, or a repo header. A PAT hit advances its last-used.

mod common;

use chrono::Duration;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::query::AuthWhoamiResponse;
use speclink_server::identity::{DevicePoll, IdentityStore, TokenPair};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

fn server() -> (String, speclink_server::state::SharedIdentity, String, String) {
    let identity = common::empty_identity();
    let (pat, user_id) = common::seed_pat(&identity, &[]);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, pat, user_id)
}

/// Mint a token pair through the store-level approval flow.
fn mint_pair(identity: &dyn IdentityStore, approver: &str) -> TokenPair {
    let auth = identity
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("initiate");
    assert!(identity.approve_device(&auth.user_code, approver).expect("approve"));
    match identity.poll_device(&auth.device_code).expect("poll") {
        DevicePoll::Approved(pair) => pair,
        other => panic!("expected approved, got {other:?}"),
    }
}

/// GET /auth/whoami with `bearer` (None = no Authorization header at all).
/// Deliberately sends no API version and no repo header — the endpoint must
/// not require a project-scoped contract.
fn whoami(base: &str, bearer: Option<&str>) -> Result<ureq::Response, ureq::Error> {
    let mut req = ureq::get(&format!("{base}/auth/whoami"));
    if let Some(token) = bearer {
        req = req.set("Authorization", &format!("Bearer {token}"));
    }
    req.call()
}

#[test]
fn an_access_token_reads_the_approvers_identity() {
    let (base, identity, _pat, user_id) = server();
    let pair = mint_pair(&*identity, &user_id);
    let resp = whoami(&base, Some(&pair.access_token)).expect("whoami");
    assert_eq!(resp.status(), 200);
    let body: AuthWhoamiResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body.user.name, common::SEED_DISPLAY, "the display name is the approver's");
    assert_eq!(body.user.handle, user_id, "the handle is the owning user id");
}

#[test]
fn a_valid_pat_reads_identity_and_advances_its_last_used() {
    let (base, identity, pat, user_id) = server();
    let before = identity.list_pats(&user_id).expect("list pats");
    assert!(before[0].last_used_at.is_none(), "a fresh PAT has no last-used");

    let resp = whoami(&base, Some(&pat)).expect("whoami");
    assert_eq!(resp.status(), 200);
    let body: AuthWhoamiResponse = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body.user.name, common::SEED_DISPLAY);

    let after = identity.list_pats(&user_id).expect("list pats");
    assert!(after[0].last_used_at.is_some(), "the whoami hit advanced the PAT's last-used");
}

#[test]
fn a_missing_or_invalid_bearer_is_the_same_401() {
    let (base, identity, _pat, user_id) = server();
    // A revoked family's access token joins the missing/garbage cases.
    let pair = mint_pair(&*identity, &user_id);
    assert!(identity.revoke_family_by_refresh(&pair.refresh_token).expect("revoke"));

    let cases: Vec<(Option<String>, &str)> = vec![
        (None, "no Authorization header"),
        (Some("spk_at_nope".to_string()), "an unknown access token"),
        (Some("spk_pat_nope".to_string()), "an unknown PAT"),
        (Some(pair.access_token.clone()), "a revoked family's access token"),
    ];
    for (bearer, label) in cases {
        match whoami(&base, bearer.as_deref()).expect_err(label) {
            ureq::Error::Status(code, resp) => {
                assert_eq!(code, 401, "{label} is 401");
                let err: ErrorResponse =
                    serde_json::from_str(&resp.into_string().unwrap()).unwrap();
                assert_eq!(
                    err.reason,
                    ErrorReason::PermissionDenied,
                    "{label} is the same permission_denied — the cause is never probed"
                );
            }
            e => panic!("transport error for {label}: {e}"),
        }
    }
}
