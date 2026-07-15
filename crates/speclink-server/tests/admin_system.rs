//! The system status page/API aggregates the existing observable surface
//! (server-admin spec「系統資訊唯讀聚合」, 決策 5): engine/API versions, the store
//! manifest and live health, the identity schema version, and per-scope outbox
//! backlog. When the store backend is unavailable the page reports a health
//! failure without failing — the identity-side management面 keeps working
//! (scenario「store 失聯不癱管理面」).

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

/// Seed an admin (session + pat) and a plain member; returns the identity handle,
/// the admin `(session, pat)`, and the member's `(session, id)`.
#[allow(clippy::type_complexity)]
fn seed() -> (Arc<IdentitySqlite>, (String, String), (String, String)) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin = seed_user(&identity, "admin@example.com", true);
    let member = seed_user(&identity, "member@example.com", false);
    (identity, (admin.0, admin.1), (member.0, member.2))
}

/// Seed a user; returns `(session, pat, id)`.
fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> (String, String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("U <{email}>"),
            memberships: vec![],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    let (_, pat) = identity.create_pat(&user_id, "tok", None).expect("pat");
    let session = identity.create_session(&user_id, Duration::days(1)).expect("session");
    (session, pat, user_id)
}

/// A store driven into the crashed state, so `health()` reports Unavailable.
fn crashed_store() -> SharedStore {
    let store = MemoryStore::new();
    store.crash_at(FaultPoint::AfterDocWrites);
    let scope = Scope::new(ProjectId::new("default"), RepoId::new("main"));
    let mut uow = store
        .begin_unit_of_work(&scope, CommandContext { command: "seed".into(), actor: "seed".into() })
        .expect("begin uow");
    uow.create(DocumentId::WorkflowConfig, "x");
    let _ = store.commit(uow, Vec::new());
    assert!(store.health().is_err(), "the store is unhealthy after the crash");
    Arc::new(store)
}

fn start(identity: Arc<IdentitySqlite>, store: SharedStore) -> String {
    let state = AppState {
        events: common::detached_events(),
        store,
        config: Arc::new(common::demo_config()),
        identity,
    };
    common::start(state)
}

fn get_page(base: &str, path: &str, session: &str) -> (u16, String) {
    let agent = ureq::builder().redirects(0).build();
    match agent.get(&format!("{base}{path}")).set("Cookie", &format!("speclink_session={session}")).call() {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn the_system_view_aggregates_the_observable_surface() {
    let (identity, (admin_session, admin_pat), _member) = seed();
    let base = start(identity, Arc::new(MemoryStore::new()));

    // The page renders every field.
    let (status, body) = get_page(&base, "/admin/system", &admin_session);
    assert_eq!(status, 200);
    assert!(body.contains("Engine 版本"), "engine version shown");
    assert!(body.contains("API 版本"), "api version shown");
    assert!(body.contains("Identity schema 版本"), "identity schema shown");
    assert!(body.contains("正常"), "store health shown as healthy");
    assert!(body.contains("demo") && body.contains("backend"), "the demo/backend scope backlog is listed");

    // The JSON API returns the same aggregation.
    let json: serde_json::Value = ureq::builder()
        .redirects(0)
        .build()
        .get(&format!("{base}/api/speclink/v1/admin/system"))
        .set("Authorization", &format!("Bearer {admin_pat}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .call()
        .expect("system api")
        .into_json()
        .expect("json");
    assert_eq!(json["api_version"], API_VERSION);
    assert!(json["engine_version"].as_str().is_some_and(|v| !v.is_empty()), "engine version present");
    assert_eq!(json["identity_schema_version"], 4, "current identity schema version");
    assert_eq!(json["store_healthy"], true);
    assert!(json["store_driver"].as_str().is_some_and(|v| !v.is_empty()), "store driver present");
    let backlogs = json["outbox_backlogs"].as_array().expect("backlogs");
    assert!(
        backlogs.iter().any(|b| b["project"] == "demo" && b["repo"] == "backend" && b["backlog"] == 0),
        "the demo/backend backlog is zero on a fresh store"
    );
}

#[test]
fn a_store_outage_does_not_disable_the_management_ui() {
    let (identity, (admin_session, _admin_pat), (_member_session, member_id)) = seed();
    let base = start(identity.clone(), crashed_store());

    // The system page renders and reports the store health failure — not a 500.
    let (status, body) = get_page(&base, "/admin/system", &admin_session);
    assert_eq!(status, 200, "the system page renders despite the store outage");
    assert!(body.contains("異常"), "the store health failure is shown");

    // Identity-side management is unaffected: suspending the member still works.
    let suspend = ureq::builder()
        .redirects(0)
        .build()
        .post(&format!("{base}/admin/users/{member_id}/suspend"))
        .set("Cookie", &format!("speclink_session={admin_session}"))
        .call()
        .expect("suspend");
    assert!((300..400).contains(&suspend.status()), "the suspend action succeeds while the store is down");
    assert!(!identity.get_user(&member_id).unwrap().unwrap().active, "the member is suspended");
    let audit = identity.list_audit(10, 0).unwrap();
    assert!(audit.iter().any(|e| e.action == "user-suspended"), "the action was audited");
}
