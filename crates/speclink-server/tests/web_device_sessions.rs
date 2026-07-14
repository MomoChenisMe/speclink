//! Device credential families on the account page (server-device-auth spec
//! 「帳號頁撤銷 device session」, 決策 5). The account page lists device families
//! alongside PATs; revoking one family kills its access token and refresh
//! credential at once and leaves other families and PATs untouched.

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{DevicePoll, IdentitySqlite, IdentityStore, NewInvitation, TokenPair};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "owner@example.com";
const PASSWORD: &str = "correct-horse-battery";

fn seed() -> (String, Arc<IdentitySqlite>, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let token = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: "Owner <owner@example.com>".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, PASSWORD).expect("accept");
    let state = AppState {
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, user_id)
}

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

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

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

fn binds(base: &str, token: &str) -> bool {
    ureq::get(&format!("{base}/api/speclink/v1/projects/demo/binding"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}

#[test]
fn revoking_one_device_family_on_the_account_page_spares_the_others_and_pats() {
    let (base, identity, user_id) = seed();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    // Two device families and a PAT, all live.
    let family_a = mint_pair(&identity, &user_id);
    let fam_a_id = {
        let list = identity.list_device_families(&user_id).unwrap();
        assert_eq!(list.len(), 1, "the first family is listed");
        assert_eq!(list[0].source, "device 授權", "the approval source is recorded");
        list[0].id.clone()
    };
    let family_b = mint_pair(&identity, &user_id);
    let (_, pat) = identity.create_pat(&user_id, "cli", None).expect("pat");

    assert!(binds(&base, &family_a.access_token), "family A binds before revoke");
    assert!(binds(&base, &family_b.access_token), "family B binds before revoke");
    assert!(binds(&base, &pat), "the PAT binds before revoke");

    // The account page lists the family with a per-family revoke control.
    let account = agent()
        .get(&format!("{base}/account"))
        .set("Cookie", &cookie)
        .call()
        .expect("account page")
        .into_string()
        .unwrap();
    assert!(account.contains("裝置登入 Sessions"), "the account page has a device sessions section");
    assert!(
        account.contains(&format!("/account/device/{fam_a_id}/revoke")),
        "the account page offers a revoke control for the family"
    );

    // Revoke family A via the account page.
    let resp = agent()
        .post(&format!("{base}/account/device/{fam_a_id}/revoke"))
        .set("Cookie", &cookie)
        .send_form(&[])
        .expect("revoke device family");
    assert!((200..400).contains(&resp.status()), "revoke succeeds, got {}", resp.status());

    // Family A's access token and refresh credential are dead; B and the PAT live.
    assert!(!binds(&base, &family_a.access_token), "family A's access token is revoked");
    match ureq::post(&format!("{base}/auth/refresh"))
        .send_json(serde_json::json!({ "refreshToken": family_a.refresh_token }))
        .expect_err("family A's refresh is dead")
    {
        ureq::Error::Status(code, _) => assert_eq!(code, 401, "family A's refresh credential is 401"),
        e => panic!("transport error: {e}"),
    }
    assert!(binds(&base, &family_b.access_token), "family B is untouched");
    assert!(binds(&base, &pat), "the PAT is untouched");
}
