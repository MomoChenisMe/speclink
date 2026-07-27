//! Browser JSON admin API（server-admin spec, D4）。`/api/speclink/v1/web/admin/*`
//! 是 session-cookie 的管理面：未登入 401、非 admin 403 `permission_denied`；mutation
//! 先驗同源再裁權限。六個 view model 只回頁面所需 metadata（絕不含 hash／plaintext／
//! refresh credential／token），mutation 呼叫與 bearer API／CLI 相同的單點 `admin_*`
//! 函式，audit source `web`。Store 不健康只降級 overview／system／data，identity 管理仍可用。

mod common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

const PASSWORD: &str = "pw-correct-horse";
const SAME_ORIGIN: &str = "http://127.0.0.1";

fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("User <{email}>"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    identity.accept_invitation(&token, PASSWORD).expect("accept")
}

/// Seed an admin + member over a store; returns (base, identity, admin_id, member_id).
fn start_over(store: SharedStore) -> (String, Arc<IdentitySqlite>, String, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin_id = seed_user(&identity, "admin@example.com", true);
    let member_id = seed_user(&identity, "member@example.com", false);
    let state = AppState {
        events: common::detached_events(),
        store,
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, admin_id, member_id)
}

fn start_healthy() -> (String, Arc<IdentitySqlite>, String, String) {
    start_over(Arc::new(MemoryStore::new()))
}

/// A store crashed so `health()` reports an error, while the identity store stays
/// readable — the store-degradation case.
fn start_crashed() -> (String, Arc<IdentitySqlite>, String, String) {
    let store = MemoryStore::new();
    store.crash_at(FaultPoint::AfterDocWrites);
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let ctx = CommandContext { command: "crash".into(), actor: "seed".into() };
    let mut uow = store.begin_unit_of_work(&scope, ctx).expect("uow");
    uow.create(DocumentId::WorkflowConfig, "cfg");
    let _ = store.commit(uow, Vec::new());
    assert!(store.health().is_err(), "the store is unhealthy after the crash");
    start_over(Arc::new(store))
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn login(base: &str, email: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .set("Origin", SAME_ORIGIN)
        .send_json(json!({ "email": email, "password": PASSWORD }))
        .expect("login");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

fn get(base: &str, path: &str, cookie: Option<&str>) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().get(&format!("{base}{path}"));
    if let Some(c) = cookie {
        req = req.set("Cookie", &format!("speclink_session={c}"));
    }
    req.call()
}

fn post(
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

// 六個 view model：原「資料操作」已併入 system（admin-console-redesign），
// 其欄位與斷言改由 admin_system_view.rs 覆蓋。
const VIEWS: [&str; 6] = ["overview", "users", "registry", "credentials", "system", "audit"];

#[test]
fn admin_views_require_an_admin_session() {
    let (base, _id, _a, _m) = start_healthy();
    // Unauthenticated → 401.
    let (status, body) = json_of(get(&base, "/api/speclink/v1/web/admin/users", None));
    assert_eq!(status, 401, "an unauthenticated admin read is 401");
    assert_eq!(body["error"]["code"], json!("unauthenticated"));
    // A logged-in member → 403 permission_denied.
    let member = login(&base, "member@example.com");
    let (status, body) = json_of(get(&base, "/api/speclink/v1/web/admin/users", Some(&member)));
    assert_eq!(status, 403, "a non-admin is 403");
    assert_eq!(body["error"]["code"], json!("permission_denied"));
    // An admin → 200.
    let admin = login(&base, "admin@example.com");
    let (status, _b) = json_of(get(&base, "/api/speclink/v1/web/admin/users", Some(&admin)));
    assert_eq!(status, 200, "an admin reads the view model");
}

#[test]
fn a_cross_origin_admin_mutation_is_refused_before_the_permission_decision() {
    let (base, identity, _a, member_id) = start_healthy();
    let admin = login(&base, "admin@example.com");
    let (status, body) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/admin/users/{member_id}/suspend"),
        json!({}),
        Some("http://evil.example"),
        Some(&admin),
    ));
    assert_eq!(status, 403, "a foreign-origin admin mutation is refused");
    assert_eq!(body["error"]["code"], json!("same_origin_required"));
    assert!(
        identity.get_user(&member_id).unwrap().unwrap().active,
        "the refused mutation changed nothing"
    );
}

#[test]
fn all_six_view_models_load_for_an_admin() {
    let (base, _id, _a, _m) = start_healthy();
    let admin = login(&base, "admin@example.com");
    for view in VIEWS {
        let (status, body) = json_of(get(
            &base,
            &format!("/api/speclink/v1/web/admin/{view}"),
            Some(&admin),
        ));
        assert_eq!(status, 200, "{view} view model loads: {body}");
        assert!(body["data"].is_object() || body["data"].is_array(), "{view} returns data");
    }
    // A few field spot-checks.
    let (_s, overview) = json_of(get(&base, "/api/speclink/v1/web/admin/overview", Some(&admin)));
    assert_eq!(overview["data"]["activeUsers"], json!(2), "admin + member are active");
    assert_eq!(overview["data"]["storeHealthy"], json!(true));
    assert!(overview["data"]["identitySchemaVersion"].is_number());
    let (_s, registry) = json_of(get(&base, "/api/speclink/v1/web/admin/registry", Some(&admin)));
    assert_eq!(registry["data"]["projects"][0]["key"], json!("demo"));
    let (_s, system) = json_of(get(&base, "/api/speclink/v1/web/admin/system", Some(&admin)));
    assert!(system["data"]["storeDriver"].is_string(), "system carries the store driver (camelCase)");
}

#[test]
fn admin_view_models_never_carry_secrets() {
    let (base, identity, _a, member_id) = start_healthy();
    let (_, plaintext) = identity.create_pat(&member_id, "cli", None).expect("pat");
    let admin = login(&base, "admin@example.com");
    let (_s, creds) = json_of(get(&base, "/api/speclink/v1/web/admin/credentials", Some(&admin)));
    let raw = creds.to_string();
    for forbidden in ["hash", "refresh", "password", "secret"] {
        assert!(!raw.contains(forbidden), "credentials view must not carry `{forbidden}`: {raw}");
    }
    assert!(!raw.contains(&plaintext), "a PAT's plaintext is never in the view");
    // The PAT is listed by prefix.
    assert!(creds["data"]["pats"].as_array().unwrap().iter().any(|p| p["name"] == json!("cli")));
}

#[test]
fn the_users_view_marks_the_last_active_admin_ineligible() {
    let (base, _id, admin_id, member_id) = start_healthy();
    let admin = login(&base, "admin@example.com");
    let (_s, users) = json_of(get(&base, "/api/speclink/v1/web/admin/users", Some(&admin)));
    let list = users["data"]["users"].as_array().unwrap();
    let admin_row = list.iter().find(|u| u["id"] == json!(admin_id)).unwrap();
    assert_eq!(admin_row["canSuspend"], json!(false), "the last active admin cannot be suspended");
    assert_eq!(admin_row["canRemoveAdmin"], json!(false), "nor lose the admin flag");
    let member_row = list.iter().find(|u| u["id"] == json!(member_id)).unwrap();
    assert_eq!(member_row["canSuspend"], json!(true), "a member can be suspended");
}

#[test]
fn suspending_the_last_active_admin_is_refused() {
    let (base, identity, admin_id, _m) = start_healthy();
    let admin = login(&base, "admin@example.com");
    let (status, _b) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/admin/users/{admin_id}/suspend"),
        json!({}),
        Some(SAME_ORIGIN),
        Some(&admin),
    ));
    assert_eq!(status, 409, "suspending the last active admin is refused");
    assert!(identity.get_user(&admin_id).unwrap().unwrap().active, "the admin stays active");
}

#[test]
fn suspending_a_member_via_the_browser_api_records_the_web_source() {
    let (base, identity, _a, member_id) = start_healthy();
    let admin = login(&base, "admin@example.com");
    let (status, _b) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/admin/users/{member_id}/suspend"),
        json!({}),
        Some(SAME_ORIGIN),
        Some(&admin),
    ));
    assert_eq!(status, 200, "the single-point suspend runs");
    assert!(!identity.get_user(&member_id).unwrap().unwrap().active, "the member is suspended");
    // The action recorded an audit whose source is `web`.
    let recent = identity.list_audit(10, 0).unwrap();
    assert!(
        recent
            .iter()
            .any(|e| e.action == "user-suspended" && e.source == "web"),
        "the browser-API suspend records source web"
    );
}

#[test]
fn registry_key_is_immutable_rename_changes_only_the_name() {
    let (base, _id, _a, _m) = start_healthy();
    let admin = login(&base, "admin@example.com");
    let (status, _b) = json_of(post(
        &base,
        "/api/speclink/v1/web/admin/registry/projects/demo/rename",
        json!({ "name": "示範專案" }),
        Some(SAME_ORIGIN),
        Some(&admin),
    ));
    assert_eq!(status, 200, "the display name changes");
    let (_s, registry) = json_of(get(&base, "/api/speclink/v1/web/admin/registry", Some(&admin)));
    let project = &registry["data"]["projects"][0];
    assert_eq!(project["key"], json!("demo"), "the key is stable");
    assert_eq!(project["name"], json!("示範專案"), "only the name changed");
}

#[test]
fn store_degradation_keeps_identity_management_up() {
    let (base, _id, _a, _m) = start_crashed();
    let admin = login(&base, "admin@example.com");
    // Overview degrades the store block but still reports identity counts.
    let (status, overview) = json_of(get(&base, "/api/speclink/v1/web/admin/overview", Some(&admin)));
    assert_eq!(status, 200);
    assert_eq!(overview["data"]["storeHealthy"], json!(false), "the store is reported unhealthy");
    assert!(overview["data"]["storeHealthError"].is_string(), "with a public error");
    assert_eq!(overview["data"]["activeUsers"], json!(2), "identity counts are still available");
    // Users and credentials management stays fully available.
    let (users_status, _b) = json_of(get(&base, "/api/speclink/v1/web/admin/users", Some(&admin)));
    assert_eq!(users_status, 200, "users management stays up under store failure");
    let (creds_status, _b2) = json_of(get(&base, "/api/speclink/v1/web/admin/credentials", Some(&admin)));
    assert_eq!(creds_status, 200, "credentials management stays up");
    // System reflects the unhealthy store.
    let (_s, system) = json_of(get(&base, "/api/speclink/v1/web/admin/system", Some(&admin)));
    assert_eq!(system["data"]["storeHealthy"], json!(false));
}
