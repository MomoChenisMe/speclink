//! 使用者 view model 的待啟用邀請（server-web-console「總覽提供可行動入口與待辦」的
//! 落點）：受邀者在接受邀請前沒有 user row，卻是管理員需要看見的對象——總覽的「待啟用」
//! 指標正是連往這一頁。view model 因此在使用者清單之外另附一份待啟用邀請清單。
//!
//! 祕密邊界不變：回應帶邀請的 email、顯示名稱、角色、成員資格與時間，絕不帶 token 或
//! 其 hash。

mod common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{AuditFilter, IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const PASSWORD: &str = "pw-correct-horse";

fn invite(identity: &Arc<IdentitySqlite>, email: &str, admin: bool, days: i64) -> String {
    identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("Invited {email}"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(days),
        })
        .expect("invite")
}

fn start() -> (String, Arc<IdentitySqlite>, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let token = invite(&identity, "admin@example.com", true, 1);
    identity.accept_invitation(&token, PASSWORD).expect("accept");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    let base = common::start(state);
    let cookie = login(&base);
    (base, identity, cookie)
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .set("Origin", "http://127.0.0.1")
        .send_json(json!({ "email": "admin@example.com", "password": PASSWORD }))
        .expect("login");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

fn revoke_invitation(base: &str, cookie: &str, id: &str) -> u16 {
    agent()
        .post(&format!(
            "{base}/api/speclink/v1/web/admin/users/invitations/{id}/revoke"
        ))
        .set("Origin", "http://127.0.0.1")
        .set("Cookie", &format!("speclink_session={cookie}"))
        .send_json(json!({}))
        .map(|r| r.status())
        .unwrap_or_else(|e| match e {
            ureq::Error::Status(code, _) => code,
            other => panic!("revoke: {other}"),
        })
}

/// The users view model as raw text, so secret-exclusion assertions see the wire bytes.
fn users_text(base: &str, cookie: &str) -> String {
    agent()
        .get(&format!("{base}/api/speclink/v1/web/admin/users"))
        .set("Cookie", &format!("speclink_session={cookie}"))
        .call()
        .expect("users")
        .into_string()
        .expect("body")
}

fn users(base: &str, cookie: &str) -> Value {
    serde_json::from_str(&users_text(base, cookie)).expect("json")
}

#[test]
fn pending_invitations_appear_in_the_users_view() {
    let (base, identity, cookie) = start();
    invite(&identity, "invited@example.com", false, 1);

    let body = users(&base, &cookie);
    let pending = body["data"]["pending"]
        .as_array()
        .expect("pending is an array");
    assert_eq!(pending.len(), 1, "剛建立的邀請要出現在待啟用清單");

    let row = &pending[0];
    assert_eq!(row["email"], "invited@example.com");
    assert_eq!(row["display"], "Invited invited@example.com");
    assert_eq!(row["admin"], false);
    assert_eq!(row["memberships"], serde_json::json!(["demo"]));
    assert!(row["createdAt"].as_str().is_some(), "createdAt 為 RFC3339 字串");
    assert!(row["expiresAt"].as_str().is_some(), "expiresAt 為 RFC3339 字串");
}

#[test]
fn accepted_invitations_leave_the_pending_list_and_become_users() {
    let (base, identity, cookie) = start();
    let token = invite(&identity, "invited@example.com", false, 1);
    identity.accept_invitation(&token, PASSWORD).expect("accept");

    let body = users(&base, &cookie);
    assert!(
        body["data"]["pending"].as_array().expect("pending").is_empty(),
        "已接受的邀請不再是待啟用"
    );
    let users = body["data"]["users"].as_array().expect("users");
    assert!(
        users.iter().any(|u| u["email"] == "invited@example.com"),
        "接受後成為正式使用者"
    );
}

#[test]
fn expired_invitations_are_not_pending() {
    let (base, identity, cookie) = start();
    invite(&identity, "stale@example.com", false, -1);

    let body = users(&base, &cookie);
    assert!(
        body["data"]["pending"].as_array().expect("pending").is_empty(),
        "過期的邀請沒有任何動作可清除，不列為待啟用"
    );
}

#[test]
fn pending_invitations_never_expose_the_token() {
    let (base, identity, cookie) = start();
    let token = invite(&identity, "invited@example.com", false, 1);

    let raw = users_text(&base, &cookie);
    assert!(!raw.contains(&token), "回應不得帶邀請 token");
    assert!(!raw.contains("tokenHash"), "回應不得帶 token hash");
}

#[test]
fn revoking_a_pending_invitation_removes_it_and_kills_the_token() {
    let (base, identity, cookie) = start();
    let token = invite(&identity, "invited@example.com", false, 1);
    let id = users(&base, &cookie)["data"]["pending"][0]["id"]
        .as_str()
        .expect("pending id")
        .to_string();

    assert_eq!(revoke_invitation(&base, &cookie, &id), 200);

    let body = users(&base, &cookie);
    assert!(
        body["data"]["pending"].as_array().expect("pending").is_empty(),
        "取消後不再是待啟用"
    );
    // 取消的重點是連結立刻失效——受邀者手上那份連結不能還能建帳號。
    assert!(
        identity.find_valid_invitation(&token).expect("lookup").is_none(),
        "取消後 token 不再有效"
    );
    assert!(
        identity.accept_invitation(&token, PASSWORD).is_err(),
        "取消後不得再以該 token 建立帳號"
    );
}

#[test]
fn revoking_writes_an_audit_event_naming_the_invitee() {
    let (base, identity, cookie) = start();
    invite(&identity, "invited@example.com", false, 1);
    let id = users(&base, &cookie)["data"]["pending"][0]["id"]
        .as_str()
        .expect("pending id")
        .to_string();

    revoke_invitation(&base, &cookie, &id);

    let entries = identity
        .query_audit(&AuditFilter {
            keyword: None,
            action: Some("invitation-revoked".to_string()),
            source: None,
            from: None,
            to: None,
            page: 1,
            per_page: 10,
        })
        .expect("audit");
    assert_eq!(entries.entries.len(), 1, "取消邀請要留下稽核事件");
    assert_eq!(entries.entries[0].subject, "invited@example.com");
}

#[test]
fn revoking_an_unknown_or_accepted_invitation_is_not_found() {
    let (base, identity, cookie) = start();
    assert_eq!(revoke_invitation(&base, &cookie, "no-such-id"), 404);

    let token = invite(&identity, "invited@example.com", false, 1);
    let id = users(&base, &cookie)["data"]["pending"][0]["id"]
        .as_str()
        .expect("pending id")
        .to_string();
    identity.accept_invitation(&token, PASSWORD).expect("accept");
    // 已接受的邀請不能「取消」：帳號已經存在，該走停權而不是回收邀請。
    assert_eq!(revoke_invitation(&base, &cookie, &id), 404);
}
