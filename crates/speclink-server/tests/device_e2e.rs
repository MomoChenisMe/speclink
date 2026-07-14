//! End-to-end device flow against a real server over SQLite store + identity
//! (server-device-auth spec, 決策 1/3/4/5). An HTTP client walks the whole
//! journey: initiate → log in and approve → poll for the token pair → drive the
//! existing query and command routes with the access token → refresh → reuse the
//! old refresh to trigger family teardown → revoke another family on the account
//! page.

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::device::{DeviceAuthorizationResponse, DeviceTokenResponse, DeviceTokenStatus, RefreshResponse};
use speclink_protocol::API_VERSION;
use speclink_server::config::StoreConfig;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use std::sync::Arc;

const EMAIL: &str = "dev@example.com";
const DISPLAY: &str = "Dev <dev@example.com>";
const PASSWORD: &str = "e2e-correct-horse";

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn initiate(base: &str) -> DeviceAuthorizationResponse {
    let resp = ureq::post(&format!("{base}/auth/device")).call().expect("initiate");
    serde_json::from_str(&resp.into_string().unwrap()).expect("init body")
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

/// Walk the approval page two-step: enter the user code to reach the confirm
/// step, then approve.
fn approve(base: &str, cookie: &str, user_code: &str) {
    let confirm = agent()
        .post(&format!("{base}/activate"))
        .set("Cookie", cookie)
        .send_form(&[("user_code", user_code)])
        .expect("confirm step");
    assert_eq!(confirm.status(), 200, "the confirm step renders");
    let done = agent()
        .post(&format!("{base}/activate"))
        .set("Cookie", cookie)
        .send_form(&[("user_code", user_code), ("action", "approve")])
        .expect("approve");
    assert_eq!(done.status(), 200, "approval succeeds");
}

fn poll(base: &str, device_code: &str) -> DeviceTokenResponse {
    let resp = ureq::post(&format!("{base}/auth/device/token"))
        .send_json(serde_json::json!({ "deviceCode": device_code }))
        .expect("poll");
    serde_json::from_str(&resp.into_string().unwrap()).expect("poll body")
}

fn binds(base: &str, access_token: &str) -> bool {
    ureq::get(&format!("{base}/api/speclink/v1/projects/demo/binding"))
        .set("Authorization", &format!("Bearer {access_token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}

#[test]
fn device_flow_end_to_end_over_a_real_sqlite_server() {
    let dir = tempfile::tempdir().expect("workdir");
    let store_path = dir.path().join("store.db");
    let id_path = dir.path().join("identity.db");

    // A real SQLite identity store, seeded with a member of `demo`.
    let identity = Arc::new(IdentitySqlite::open(&id_path).expect("identity store"));
    let invite = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: DISPLAY.to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&invite, PASSWORD).expect("accept");

    // A real SQLite team store, behind the real router.
    let store = speclink_server::build_store(&StoreConfig::Sqlite { path: store_path }).expect("store");
    let state = AppState { store, identity: identity.clone(), config: Arc::new(common::demo_config()), events: common::detached_events() };
    let base = common::start(state);

    // 1. initiate → two codes.
    let init = initiate(&base);
    assert!(init.device_code.starts_with("spk_dc_"));

    // 2. log in and approve on the approval page. (The pending/slow_down poll
    //    cadence is covered in device_flow; a real client respects the declared
    //    interval, so here the client polls once, after approval.)
    let cookie = format!("speclink_session={}", login(&base));
    approve(&base, &cookie, &init.user_code);

    // 3. poll → the token pair, bound to the approver.
    let approved = poll(&base, &init.device_code);
    assert_eq!(approved.status, DeviceTokenStatus::Approved);
    let access = approved.access_token.expect("access token");
    let refresh = approved.refresh_token.expect("refresh token");

    // 4. drive the existing query and command routes with the access token.
    let create = ureq::post(&format!("{base}/api/speclink/v1/projects/demo/changes"))
        .set("Authorization", &format!("Bearer {access}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .send_json(serde_json::json!({ "name": "e2e-change", "schema": "spec-driven" }))
        .expect("create change with the access token");
    assert_eq!(create.status(), 200, "the command route runs under the access token");
    let list = ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes"))
        .set("Authorization", &format!("Bearer {access}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("query with the access token")
        .into_string()
        .unwrap();
    assert!(list.contains("e2e-change"), "the query route lists the change created via the access token: {list}");

    // 5. refresh → a fresh pair; the new access token works.
    let rotated: RefreshResponse = serde_json::from_str(
        &ureq::post(&format!("{base}/auth/refresh"))
            .send_json(serde_json::json!({ "refreshToken": refresh }))
            .expect("refresh")
            .into_string()
            .unwrap(),
    )
    .unwrap();
    assert!(binds(&base, &rotated.access_token), "the rotated access token binds");

    // 6. reuse the old refresh → 401 and the whole family is torn down.
    match ureq::post(&format!("{base}/auth/refresh"))
        .send_json(serde_json::json!({ "refreshToken": refresh }))
        .expect_err("reuse errors")
    {
        ureq::Error::Status(code, _) => assert_eq!(code, 401, "reuse of a rotated refresh is 401"),
        e => panic!("transport error: {e}"),
    }
    assert!(!binds(&base, &rotated.access_token), "reuse revoked the family, killing the rotated access token");

    // 7. a second family, revoked from the account page.
    let init_b = initiate(&base);
    approve(&base, &cookie, &init_b.user_code);
    let access_b = poll(&base, &init_b.device_code).access_token.expect("family B access token");
    assert!(binds(&base, &access_b), "family B binds");
    let fam_b = identity
        .list_device_families(&user_id)
        .unwrap()
        .into_iter()
        .find(|f| f.revoked_at.is_none())
        .expect("an active family B");
    let revoke = agent()
        .post(&format!("{base}/account/device/{}/revoke", fam_b.id))
        .set("Cookie", &cookie)
        .send_form(&[])
        .expect("revoke family B on the account page");
    assert!((200..400).contains(&revoke.status()), "the account-page revoke succeeds");
    assert!(!binds(&base, &access_b), "the account-page revoke kills family B");
}
