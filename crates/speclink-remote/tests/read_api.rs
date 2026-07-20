//! Typed workflow-config client methods against the real in-process server.

use chrono::{Duration, Utc};
use speclink_remote::client::Client;
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

struct Fixture {
    client: Client,
}

fn fixture() -> Fixture {
    let store = Arc::new(MemoryStore::new());
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext {
                command: "seed-workflow-config-client".to_string(),
                actor: "test".to_string(),
            },
        )
        .expect("begin seed");
    uow.create(
        DocumentId::WorkflowConfig,
        "schema: spec-driven\nlocale: en\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");

    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    identity.create_project("demo", "Demo").expect("project");
    identity
        .create_repo("demo", "backend", "Backend")
        .expect("repo");
    let invitation = identity
        .create_invitation(NewInvitation {
            email: "editor@example.com".to_string(),
            display: "Editor".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invitation");
    let user_id = identity
        .accept_invitation(&invitation, "seed-password")
        .expect("accept invitation");
    let (_, pat) = identity.create_pat(&user_id, "test", None).expect("PAT");

    let settings = EventSettings::default();
    let state = AppState {
        store: store.clone(),
        identity,
        events: EventHub::new(store, settings.clone()),
        config: Arc::new(ServerConfig {
            store: StoreConfig::Memory,
            identity: IdentityConfig::Memory,
            public_url: "http://127.0.0.1".to_string(),
            events: settings,
        }),
    };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("local address");
    listener.set_nonblocking(true).expect("nonblocking");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
            axum::serve(listener, speclink_server::app::router(state))
                .await
                .expect("serve");
        });
    });

    let project_url = format!("http://{address}/api/speclink/v1/projects/demo");
    Fixture {
        client: Client::new(&project_url, &pat, Some("backend")),
    }
}

#[test]
fn config_read_and_cas_write_round_trip_through_typed_methods() {
    let fixture = fixture();

    let before = fixture.client.config().expect("config read");
    assert_eq!(before.schema, "spec-driven");
    assert_eq!(
        before.content.as_deref(),
        Some("schema: spec-driven\nlocale: en\n")
    );
    assert!(before.revision > 0, "the scope revision is returned");

    let updated_source = "schema: spec-driven\nlocale: tw\ntdd: true\n";
    let saved = fixture
        .client
        .put_config(updated_source, before.revision)
        .expect("config write");
    assert!(saved.revision > before.revision, "the revision advances");

    let after = fixture.client.config().expect("config re-read");
    assert_eq!(after.content.as_deref(), Some(updated_source));
    assert_eq!(after.revision, saved.revision);

    let stale = fixture
        .client
        .put_config("schema: spec-driven\nlocale: ja\n", before.revision)
        .expect_err("stale expected revision conflicts");
    assert_eq!(stale.reason.as_deref(), Some("revision_conflict"));
    assert_eq!(
        fixture
            .client
            .config()
            .expect("winner remains")
            .content
            .as_deref(),
        Some(updated_source),
        "the stale write has no side effect",
    );
}
