//! 總覽 view model 的新欄位（server-admin「管理 browser API 提供最小且完整的頁面
//! view model」）：既有計數之外，增列待啟用邀請數、待處理事項清單（每則標示類型與
//! 對應目的地）與最近稽核事件清單。資料來源沿用既有 identity／registry／store／audit
//! 查詢，不新增 domain action；回應為 camelCase 且不含任何祕密值。

use crate::common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::audit::{AuditAction, AuditActor, AuditSource};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const PASSWORD: &str = "pw-correct-horse";

fn invite(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> String {
    identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("User <{email}>"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite")
}

fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> String {
    let token = invite(identity, email, admin);
    identity.accept_invitation(&token, PASSWORD).expect("accept")
}

fn start() -> (String, Arc<IdentitySqlite>, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin_id = seed_user(&identity, "admin@example.com", true);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, admin_id)
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

fn overview(base: &str, cookie: &str) -> Value {
    let resp = agent()
        .get(&format!("{base}/api/speclink/v1/web/admin/overview"))
        .set("Cookie", &format!("speclink_session={cookie}"))
        .call()
        .expect("overview");
    assert_eq!(resp.status(), 200);
    resp.into_json::<Value>().expect("json")["data"].clone()
}

/// The kinds of the todo list, in order.
fn todo_kinds(data: &Value) -> Vec<String> {
    data["todos"]
        .as_array()
        .expect("todos array")
        .iter()
        .map(|t| t["kind"].as_str().expect("kind").to_string())
        .collect()
}

#[test]
fn the_overview_counts_outstanding_invitations() {
    let (base, identity, _admin) = start();
    // 兩張未使用的邀請：都算待啟用。
    invite(&identity, "one@example.com", false);
    invite(&identity, "two@example.com", false);
    let cookie = login(&base);
    let data = overview(&base, &cookie);
    assert_eq!(
        data["pendingInvitations"],
        json!(2),
        "both unconsumed invitations count: {data}"
    );
    // 被接受的邀請不再是待啟用（admin 自己就是接受過的那一張）。
    assert_eq!(data["activeUsers"], json!(1), "only the admin is an active user");
}

#[test]
fn an_accepted_invitation_leaves_the_pending_count() {
    let (base, identity, _admin) = start();
    let token = invite(&identity, "later@example.com", false);
    let cookie = login(&base);
    assert_eq!(overview(&base, &cookie)["pendingInvitations"], json!(1));
    identity.accept_invitation(&token, PASSWORD).expect("accept");
    assert_eq!(
        overview(&base, &cookie)["pendingInvitations"],
        json!(0),
        "an accepted invitation is no longer pending"
    );
}

#[test]
fn a_system_with_no_active_credential_reports_that_todo() {
    let (base, _identity, _admin) = start();
    let cookie = login(&base);
    let data = overview(&base, &cookie);
    assert_eq!(data["activeCredentials"], json!(0), "nothing was issued yet");
    let todos = data["todos"].as_array().expect("todos array");
    let credential_todo = todos
        .iter()
        .find(|t| t["kind"] == json!("no-active-credentials"))
        .unwrap_or_else(|| panic!("a no-active-credentials todo is present: {data}"));
    assert_eq!(
        credential_todo["destination"], json!("/account"),
        "the todo names where to act on it"
    );
}

#[test]
fn pending_invitations_are_a_todo_with_the_users_destination() {
    let (base, identity, _admin) = start();
    invite(&identity, "one@example.com", false);
    let cookie = login(&base);
    let data = overview(&base, &cookie);
    let todo = data["todos"]
        .as_array()
        .expect("todos array")
        .iter()
        .find(|t| t["kind"] == json!("pending-invitations"))
        .unwrap_or_else(|| panic!("a pending-invitations todo is present: {data}"));
    assert_eq!(todo["destination"], json!("/admin/users"));
    assert_eq!(todo["count"], json!(1), "the todo carries how many are waiting");
}

#[test]
fn a_healthy_fully_provisioned_system_has_no_todo() {
    let (base, identity, admin_id) = start();
    identity.create_pat(&admin_id, "cli", None).expect("pat");
    let cookie = login(&base);
    let data = overview(&base, &cookie);
    assert_eq!(data["activeCredentials"], json!(1));
    assert_eq!(data["pendingInvitations"], json!(0));
    assert!(
        todo_kinds(&data).is_empty(),
        "nothing needs attention, so the list is empty: {data}"
    );
}

#[test]
fn the_overview_carries_the_most_recent_audit_events() {
    let (base, identity, admin_id) = start();
    let actor = AuditActor::user(admin_id.clone(), AuditSource::Web);
    for subject in ["oldest", "middle", "newest"] {
        identity
            .record_audit(&actor, AuditAction::ProjectCreated, subject)
            .expect("audit");
    }
    let cookie = login(&base);
    let data = overview(&base, &cookie);
    let recent = data["recentAudit"].as_array().expect("recentAudit array");
    assert_eq!(recent[0]["subject"], json!("newest"), "newest first: {data}");
    for field in ["id", "actorId", "action", "subject", "source", "createdAt"] {
        assert!(recent[0][field].is_string(), "{field} is camelCase and present: {recent:?}");
    }
}

#[test]
fn the_overview_carries_no_secret() {
    let (base, identity, admin_id) = start();
    let (_pat, plaintext) = identity.create_pat(&admin_id, "cli", None).expect("pat");
    let invite_token = invite(&identity, "one@example.com", false);
    let cookie = login(&base);
    let raw = overview(&base, &cookie).to_string();
    for forbidden in ["hash", "password", "secret", "token"] {
        assert!(!raw.contains(forbidden), "the overview must not carry `{forbidden}`: {raw}");
    }
    assert!(!raw.contains(&plaintext), "a PAT's plaintext is never in the view");
    assert!(!raw.contains(&invite_token), "an invitation token is never in the view");
}
