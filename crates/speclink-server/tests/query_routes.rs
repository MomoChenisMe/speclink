//! Changes query routes served as protocol DTOs (reference-server spec「動詞經
//! 正路執行且回 DTO」). The typed client reads a seeded server; the response DTOs
//! carry the fields the stub-tested shapes do, and a missing change is the
//! 404 not_found triple.

mod common;

use speclink_remote::client::Client;
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

/// A server seeded with one change (`demo`) and one canonical spec.
fn seeded_base() -> (String, String, String) {
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() },
        "## Why\n\nBecause it is needed.\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n- [ ] 1.2 Second\n",
    );
    uow.create(DocumentId::CanonicalSpec { capability: "user-auth".into() }, "# user-auth\n");
    store.commit(uow, Vec::new()).expect("seed commit");

    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    let (pat, user) = common::seed_pat(&state.identity, &["demo"]);
    (common::start(state), pat, user)
}

fn client(base: &str, token: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        token,
        Some("backend"),
    )
}

#[test]
fn list_status_and_specs_return_typed_dtos() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);

    let list = client.list_changes().expect("list changes");
    assert_eq!(list.changes.len(), 1);
    assert_eq!(list.changes[0].name, "demo");
    assert_eq!(list.changes[0].total_tasks, 2, "the two tasks are counted");

    let status = client.get_change("demo").expect("status");
    assert_eq!(status.change_name, "demo");
    assert_eq!(status.schema_name, "spec-driven");
    assert!(
        status.artifacts.iter().any(|a| a.id == "proposal"),
        "the status lists the proposal artifact"
    );

    let specs = client.list_specs().expect("specs");
    assert!(
        specs.specs.iter().any(|s| s.id == "user-auth"),
        "the canonical spec is listed"
    );
}

#[test]
fn instructions_and_artifact_content_return_typed_dtos() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);

    let apply = client.apply_instructions("demo").expect("apply instructions");
    assert_eq!(apply.change_name, "demo");
    assert_eq!(apply.progress.total, 2, "apply view counts the tasks");
    assert!(apply.context_files.contains_key("tasks"), "apply view names the context files");

    let proposal = client
        .artifact_instructions("demo", "proposal")
        .expect("proposal instructions");
    assert_eq!(proposal.artifact_id, "proposal");
    assert_eq!(proposal.output_path, "proposal.md");

    let content = client.get_artifact("demo", "proposal").expect("artifact content");
    assert_eq!(content.artifact, "proposal");
    assert_eq!(content.content, "## Why\n\nBecause it is needed.\n");
    assert!(content.version > 0, "the artifact version is stamped for If-Match writes");
}

#[test]
fn config_and_whoami_return_typed_dtos() {
    let (base, pat, user_id) = seeded_base();
    let client = client(&base, &pat);

    let config = client.config().expect("config");
    assert_eq!(config.schema, "spec-driven", "the default workflow schema");

    let whoami = client.whoami().expect("whoami");
    assert_eq!(whoami.user.handle, user_id, "whoami reports the PAT owner's id");
    assert_eq!(whoami.user.name, common::SEED_DISPLAY);
    assert!(whoami.repos.iter().any(|r| r.name == "backend"), "whoami lists the repo");
}

#[test]
fn a_missing_change_is_the_404_not_found_triple() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);
    let err = client.get_change("ghost").expect_err("a missing change is an error");
    assert_eq!(
        err.reason.as_deref(),
        Some("not_found"),
        "the error is the not_found triple: {err:?}"
    );
}
