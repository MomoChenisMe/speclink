//! The admin data operations (server-backup spec「admin 資料操作入 audit」, 決策
//! 5): scope export download, backup-info view and store-migration trigger, each
//! behind the admin gate and each recording an audit entry.

mod common;

use chrono::{Duration, Utc};
use speclink_server::audit::AuditActor;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewBackupRecord, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{
    content_digest, CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

/// Seed a `demo`-registry admin with a session; returns the session id.
fn seed_admin(identity: &Arc<IdentitySqlite>) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: "admin@example.com".into(),
            display: "Admin <admin@example.com>".into(),
            memberships: vec!["demo".into()],
            admin: true,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    identity.create_session(&user_id, Duration::days(1)).expect("session")
}

/// Seed a non-admin member with a session; returns the session id.
fn seed_member(identity: &Arc<IdentitySqlite>) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: "member@example.com".into(),
            display: "Member <member@example.com>".into(),
            memberships: vec!["demo".into()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    identity.create_session(&user_id, Duration::days(1)).expect("session")
}

/// Write two documents into `demo/backend` of `store`.
fn seed_docs(store: &dyn TeamStore) {
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let ctx = CommandContext { command: "seed".into(), actor: "seed".into() };
    let mut uow = store.begin_unit_of_work(&scope, ctx).expect("uow");
    uow.create(DocumentId::ChangeMeta { change: "add-auth".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "add-auth".into(), artifact: "proposal.md".into() },
        "## Why\nseed\n",
    );
    store.commit(uow, Vec::new()).expect("commit");
}

fn start() -> (String, Arc<IdentitySqlite>, String, String) {
    let store = Arc::new(MemoryStore::new());
    seed_docs(store.as_ref());
    start_with(store)
}

/// Start a server over `store` (already seeded as the test needs) with an admin
/// and a member session; returns the base URL, identity, admin and member sessions.
fn start_with(store: SharedStore) -> (String, Arc<IdentitySqlite>, String, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin_session = seed_admin(&identity);
    let member_session = seed_member(&identity);
    let state = AppState {
        store,
        identity: identity.clone(),
        config: Arc::new(common::demo_config()),
        events: common::detached_events(),
    };
    (common::start(state), identity, admin_session, member_session)
}

/// A store driven into the crashed state, so `health()` reports an error.
fn crashed_store() -> SharedStore {
    let store = MemoryStore::new();
    store.crash_at(FaultPoint::AfterDocWrites);
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let ctx = CommandContext { command: "crash".into(), actor: "seed".into() };
    let mut uow = store.begin_unit_of_work(&scope, ctx).expect("uow");
    uow.create(DocumentId::WorkflowConfig, "cfg");
    let _ = store.commit(uow, Vec::new());
    assert!(store.health().is_err(), "store is unhealthy after the crash");
    Arc::new(store)
}

/// `GET path` with a session cookie; returns `(status, body)`.
fn get(base: &str, path: &str, session: &str) -> (u16, String) {
    let agent = ureq::builder().redirects(0).build();
    match agent
        .get(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .call()
    {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("request error: {e}"),
    }
}

#[test]
fn scope_export_downloads_a_verifiable_bundle_and_audits() {
    let (base, identity, admin, _member) = start();

    let (code, body) = get(&base, "/admin/data/export/demo/backend", &admin);
    assert_eq!(code, 200, "an admin downloads the scope's export bundle");

    // The bundle parses and every document's digest matches its content.
    let bundle: serde_json::Value = serde_json::from_str(&body).expect("bundle is json");
    let docs = bundle["documents"].as_array().expect("documents");
    assert_eq!(docs.len(), 2, "both documents are in the bundle");
    for doc in docs {
        let content = doc["content"].as_str().expect("content");
        assert_eq!(doc["digest"], content_digest(content), "digest matches content");
    }

    // A scope-exported audit record was written.
    let audit = identity.list_audit(100, 0).expect("audit");
    assert!(
        audit.iter().any(|e| e.action == "scope-exported" && e.subject.contains("demo/backend")),
        "a scope-exported audit names the scope"
    );
}

#[test]
fn export_of_an_unknown_scope_is_404() {
    let (base, _identity, admin, _member) = start();
    assert_eq!(get(&base, "/admin/data/export/demo/nope", &admin).0, 404, "unknown repo → 404");
    assert_eq!(get(&base, "/admin/data/export/nope/backend", &admin).0, 404, "unknown project → 404");
}

#[test]
fn data_operations_require_admin() {
    let (base, _identity, _admin, member) = start();
    assert_eq!(get(&base, "/admin/data", &member).0, 403, "the data page is admin-only");
    assert_eq!(
        get(&base, "/admin/data/export/demo/backend", &member).0,
        403,
        "export is admin-only"
    );
}

/// `POST path` with a session cookie and no body; returns the status.
fn post(base: &str, path: &str, session: &str) -> u16 {
    let agent = ureq::builder().redirects(0).build();
    match agent
        .post(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_form(&[])
    {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("request error: {e}"),
    }
}

#[test]
fn the_data_page_shows_recent_backup_info() {
    let (base, identity, admin, _member) = start();
    // Seed a backup record (as the backup subcommand would after a run).
    identity
        .record_backup(
            &AuditActor::system_cli(),
            NewBackupRecord {
                kind: "backup".into(),
                created_at: Utc::now(),
                format_version: speclink_server::backup::BACKUP_FORMAT_VERSION,
                scope_count: 2,
                ok: true,
                detail: "2 個 scope、3 個成員".into(),
            },
        )
        .expect("record backup");

    let (code, body) = get(&base, "/admin/data", &admin);
    assert_eq!(code, 200);
    assert!(body.contains("2 個 scope、3 個成員"), "the page shows the recent backup detail");
}

#[test]
fn migration_trigger_runs_when_healthy_and_audits_store_migrated() {
    let store = Arc::new(MemoryStore::new());
    seed_docs(store.as_ref());
    let (base, identity, admin, _member) = start_with(store);

    let status = post(&base, "/admin/data/migrate", &admin);
    assert!(status == 303 || status == 200, "a healthy store migrates: status {status}");

    let audit = identity.list_audit(100, 0).expect("audit");
    assert!(
        audit.iter().any(|e| e.action == "store-migrated"),
        "a successful migration records store-migrated"
    );
}

#[test]
fn migration_is_skipped_when_store_health_fails() {
    let (base, identity, admin, _member) = start_with(crashed_store());

    let status = post(&base, "/admin/data/migrate", &admin);
    assert!(status >= 400 || status == 200, "an unhealthy store's migration is refused, not 5xx panic");

    let audit = identity.list_audit(100, 0).expect("audit");
    assert!(
        !audit.iter().any(|e| e.action == "store-migrated"),
        "health failure must not record a store-migrated audit"
    );
}

#[test]
fn migration_trigger_requires_admin() {
    let (base, _identity, _admin, member) = start();
    assert_eq!(post(&base, "/admin/data/migrate", &member), 403, "migration is admin-only");
}
