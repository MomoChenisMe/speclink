//! The system status page/API aggregates the existing observable surface
//! (server-admin spec「系統資訊唯讀聚合」, 決策 5): engine/API versions, the store
//! manifest and live health, the identity schema version, and per-scope outbox
//! backlog. When the store backend is unavailable the page reports a health
//! failure without failing — the identity-side management面 keeps working
//! (scenario「store 失聯不癱管理面」).

use crate::common;

use chrono::{Duration, Utc};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
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

fn start(identity: Arc<IdentitySqlite>, store: SharedStore) -> String {
    let state = AppState {
        events: common::detached_events(),
        store,
        config: Arc::new(common::demo_config()),
        identity,
    };
    common::start(state)
}

#[test]
fn the_system_view_aggregates_the_observable_surface() {
    let (identity, (_, admin_pat), _) = seed();
    let base = start(identity, Arc::new(MemoryStore::new()));

    // The system aggregation is served over the bearer admin API.
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
    assert_eq!(json["identity_schema_version"], 6, "current identity schema version");
    assert_eq!(json["store_healthy"], true);
    assert!(json["store_driver"].as_str().is_some_and(|v| !v.is_empty()), "store driver present");
    let backlogs = json["outbox_backlogs"].as_array().expect("backlogs");
    assert!(
        backlogs.iter().any(|b| b["project"] == "demo" && b["repo"] == "backend" && b["backlog"] == 0),
        "the demo/backend backlog is zero on a fresh store"
    );
}
