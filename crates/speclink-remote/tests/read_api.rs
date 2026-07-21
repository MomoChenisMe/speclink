//! Typed read-client methods against the real in-process server. These assert
//! URL derivation for identity-only `/scopes` and every project-bound response
//! shape without a mock JSON bypass.

use chrono::{Duration, Utc};
use speclink_remote::client::Client;
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

const DATED_NAME: &str = "2026-07-19-old-feature";

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
                command: "seed-read-client".to_string(),
                actor: "test".to_string(),
            },
        )
        .expect("begin seed");
    for (document, content) in [
        (
            DocumentId::WorkflowConfig,
            "schema: spec-driven\nlocale: en\n",
        ),
        (
            DocumentId::CanonicalSpec {
                capability: "payments".to_string(),
            },
            "# payments Specification\n\nCanonical truth.\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: DATED_NAME.to_string(),
                doc: ".openspec.yaml".to_string(),
            },
            "schema: spec-driven\ncreated: 2026-07-18\ncreated_by: Creator\nfrom_discussion: source-talk\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: DATED_NAME.to_string(),
                doc: "proposal.md".to_string(),
            },
            "## Why\n\nArchived truth.\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: DATED_NAME.to_string(),
                doc: "tasks.md".to_string(),
            },
            "- [x] 1.1 Done\n- [ ] 1.2 Pending\n",
        ),
        (
            DocumentId::ArchivedChange {
                change: DATED_NAME.to_string(),
                doc: "specs/payments/spec.md".to_string(),
            },
            "## ADDED Requirements\n\n### Requirement: Pay\n",
        ),
        (
            DocumentId::ChangeMeta {
                change: "searchable".to_string(),
            },
            "schema: spec-driven\n",
        ),
        (
            DocumentId::ChangeArtifact {
                change: "searchable".to_string(),
                artifact: "proposal.md".to_string(),
            },
            "## Why\n\nRemoteNeedle in a change.\n",
        ),
        (
            DocumentId::Discussion {
                slug: "search-talk".to_string(),
                archived: false,
            },
            "---\ntopic: Search\nslug: search-talk\nstatus: open\ncreated: 2026-07-19\n---\n\nRemoteNeedle in a discussion.\n",
        ),
    ] {
        uow.create(document, content);
    }
    store.commit(uow, Vec::new()).expect("seed commit");

    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    identity.create_project("demo", "Demo").expect("project");
    identity
        .create_repo("demo", "backend", "Backend")
        .expect("repo");
    let invitation = identity
        .create_invitation(NewInvitation {
            email: "reader@example.com".to_string(),
            display: "Reader".to_string(),
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
fn scopes_and_documents_are_typed_end_to_end() {
    let fixture = fixture();

    let scopes = fixture.client.list_scopes().expect("scopes");
    assert_eq!(scopes.projects.len(), 1);
    assert_eq!(scopes.projects[0].key, "demo");
    assert_eq!(scopes.projects[0].repos[0].key, "backend");

    let spec = fixture
        .client
        .spec_document("payments")
        .expect("spec document");
    assert_eq!(
        spec.content,
        "# payments Specification\n\nCanonical truth.\n"
    );
}

#[test]
fn archive_and_search_methods_return_desktop_aligned_typed_shapes() {
    let fixture = fixture();

    let archived = fixture.client.archived_list().expect("archived list");
    assert_eq!(archived.archived.len(), 1);
    let item = &archived.archived[0];
    assert_eq!(item.dated_name, DATED_NAME);
    assert_eq!(item.tasks_total, Some(2));
    assert_eq!(item.tasks_done, Some(1));
    assert_eq!(item.spec_count, 1);
    assert_eq!(item.created_by.as_deref(), Some("Creator"));
    assert_eq!(item.from_discussions, ["source-talk"]);

    let artifact = fixture
        .client
        .archived_artifact(DATED_NAME, "proposal.md")
        .expect("archived artifact");
    assert_eq!(artifact.content, "## Why\n\nArchived truth.\n");
    assert_eq!(
        fixture
            .client
            .archived_capabilities(DATED_NAME)
            .expect("archived capabilities"),
        ["payments"]
    );

    let search = fixture.client.search("remoteneedle").expect("search");
    assert_eq!(search.hits.len(), 2);
    assert!(search.hits.iter().any(|hit| hit.kind == "change"));
    assert!(search.hits.iter().any(|hit| hit.kind == "discussion"));
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
        fixture.client.config().expect("winner remains").content.as_deref(),
        Some(updated_source),
        "the stale write has no side effect",
    );
}
