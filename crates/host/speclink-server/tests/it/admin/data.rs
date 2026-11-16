//! The admin data operations (server-backup spec「admin 資料操作入 audit」, 決策
//! 5): scope export download, backup-info view and store-migration trigger, each
//! behind the admin gate and each recording an audit entry.

use crate::common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use std::sync::Arc;

/// The digest a bundle carries for `content`: SHA-256, lowercase hex, with
/// the algorithm as a prefix.
fn content_digest(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

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
fn seed_docs(store: &common::TestStore) {
    store.put(
        "demo",
        "backend",
        &[
            ("changes/add-auth/.meta.yaml", "schema: spec-driven\n"),
            ("changes/add-auth/proposal.md", "## Why\nseed\n"),
        ],
    );
}

fn start() -> (String, Arc<IdentitySqlite>, String, String) {
    let store = common::TestStore::memory();
    seed_docs(&store);
    start_with(store.shared())
}

/// Start a server over `store` (already seeded as the test needs) with an admin
/// and a member session; returns the base URL, identity, admin and member sessions.
fn start_with(store: SharedStore) -> (String, Arc<IdentitySqlite>, String, String) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin_session = seed_admin(&identity);
    let member_session = seed_member(&identity);
    let state = AppState::new(
        store,
        identity.clone(),
        Arc::new(common::demo_config()),
        common::detached_events(),
    );
    (common::start(state), identity, admin_session, member_session)
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
    let keys: Vec<&str> = docs.iter().map(|doc| doc["key"].as_str().expect("a document is named by its key")).collect();
    assert_eq!(keys, ["changes/add-auth/.meta.yaml", "changes/add-auth/proposal.md"], "both documents are in the bundle");
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
    assert_eq!(
        get(&base, "/admin/data/export/demo/backend", &member).0,
        403,
        "export is admin-only"
    );
}
