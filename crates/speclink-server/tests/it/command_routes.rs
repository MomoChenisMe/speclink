//! Change command routes: writes commit atomically through the bridge and a
//! CAS conflict is distinguishable (reference-server spec「寫入原子提交且 CAS
//! 衝突可辨」). The losing competitor gets a 409 revision_conflict with no partial
//! write; a completed task lands a task-completed event in the outbox.

use crate::common;

use speclink_remote::client::Client;
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

const SCOPE_PROJECT: &str = "demo";
const SCOPE_REPO: &str = "backend";

fn scope() -> Scope {
    Scope::new(ProjectId::new(SCOPE_PROJECT), RepoId::new(SCOPE_REPO))
}

/// A store seeded with change `demo` (a proposal and a tasks file), returned so
/// the test can inspect its outbox after driving the server.
fn seeded_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() },
        "## Why\n\noriginal\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n- [ ] 1.2 Second\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    store
}

fn client(base: &str, token: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        token,
        Some("backend"),
    )
}

#[test]
fn competing_writers_on_the_same_version_leave_a_distinguishable_conflict() {
    let store = seeded_store();
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let read = client.get_artifact("demo", "proposal").expect("read proposal");
    let version = read.version;

    let winner = client.put_artifact("demo", "proposal", "## Why\n\nwinner wins\n", version);
    assert!(winner.is_ok(), "the first writer at the read version succeeds: {winner:?}");

    let loser = client
        .put_artifact("demo", "proposal", "## Why\n\nloser loses\n", version)
        .expect_err("the second writer at the stale version conflicts");
    assert_eq!(
        loser.reason.as_deref(),
        Some("revision_conflict"),
        "the conflict is distinguishable: {loser:?}"
    );

    let after = client.get_artifact("demo", "proposal").expect("re-read proposal");
    assert_eq!(
        after.content, "## Why\n\nwinner wins\n",
        "the winner's write stands; no partial loser write"
    );
}

#[test]
fn a_completed_task_lands_a_task_completed_event_in_the_outbox() {
    let store = seeded_store();
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let done = client.task_done("demo", "1", &[]).expect("task done");
    assert!(!done.already_done, "the task flipped for the first time");

    let entries = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox");
    let completed: Vec<_> = entries
        .iter()
        .filter(|e| e.record.name == "task-completed")
        .collect();
    assert_eq!(completed.len(), 1, "exactly one task-completed event landed in the outbox");
}
