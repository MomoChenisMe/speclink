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

/// A store seeded with an archive-ready change: tasks all done, one delta spec
/// so the archive reports capability counts, and a source discussion linked
/// both ways so it co-travels into the archive.
fn archive_ready_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed-archive".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta { change: "demo-archive".into() },
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: scope-talk\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo-archive".into(), artifact: "tasks.md".into() },
        "- [x] 1.1 done\n",
    );
    uow.create(
        DocumentId::ChangeArtifact {
            change: "demo-archive".into(),
            artifact: "specs/user-auth/spec.md".into(),
        },
        "## Purpose\n\n本 capability 是測試用的示範能力，涵蓋一個可觀察行為與其成功路徑，專供伺服端的測試取用。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
    );
    uow.create(
        DocumentId::Discussion { slug: "scope-talk".into(), archived: false },
        "---\ntopic: Scope talk\nslug: scope-talk\nstatus: promoted\npromoted_to: demo-archive\n---\n\n## Conclusion\n\nGo.\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    store
}

#[test]
fn conclude_reports_the_restale_flagged_changes_over_the_wire() {
    // spec server-verb-api「結論端點回填被打回的變更」：對已轉出且進行中的
    // 討論 re-conclude，回應點名被打回重收的變更。
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed-conclude".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta { change: "add-auth".into() },
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: auth-scope\n",
    );
    uow.create(
        DocumentId::Discussion { slug: "auth-scope".into(), archived: false },
        "---\ntopic: Auth scope\nslug: auth-scope\nstatus: promoted\npromoted_to: add-auth\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\nold\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let resp = client
        .discussion_conclude("auth-scope", "**Decision**: revised\n", false)
        .expect("conclude over the wire");
    assert_eq!(
        resp.restale_flagged,
        ["add-auth"],
        "the re-conclude names the in-flight derived change"
    );
    assert!(!resp.auto_archived, "an in-flight derived change blocks the closing step");
}

#[test]
fn conclude_backfills_auto_archived_over_the_wire() {
    // spec server-verb-api「討論結論端點回填順手封存事實」：promoted_to 非空且
    // 無在途變更引用 → 回應含 autoArchived: true，討論自 live 清單消失。
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed-auto-archive".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::Discussion { slug: "done-talk".into(), archived: false },
        "---\ntopic: Done talk\nslug: done-talk\nstatus: promoted\npromoted_to: shipped-change\ncreated: 2026-07-01\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let resp = client
        .discussion_conclude("done-talk", "**Decision**: done\n", false)
        .expect("conclude over the wire");
    assert!(resp.auto_archived, "the closing step is reported over the wire");

    let listed = client.list_discussions(false).expect("list live discussions");
    assert!(
        listed.discussions.iter().all(|d| d.slug != "done-talk"),
        "the record left the live list"
    );
    let archived = client.list_discussions(true).expect("list archived discussions");
    assert!(
        archived.discussions.iter().any(|d| d.slug == "done-talk"),
        "the record landed in the archived list"
    );
}

#[test]
fn conclude_with_hold_keeps_the_record_live_over_the_wire() {
    // spec server-verb-api「討論結論端點轉傳保留旗標並回填 held」：閉環條件原本
    // 成立的討論以 hold: true 結論 → 回應 held: true 無 autoArchived，記錄留 live。
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed-hold".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::Discussion { slug: "staged-talk".into(), archived: false },
        "---\ntopic: Staged talk\nslug: staged-talk\nstatus: promoted\npromoted_to: shipped-change\ncreated: 2026-07-01\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let before = store.snapshot(&scope()).expect("snapshot").revision().0;
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let resp = client
        .discussion_conclude("staged-talk", "**Decision**: cut-b later\n", true)
        .expect("conclude with hold over the wire");
    assert!(resp.held, "held 回填");
    assert!(!resp.auto_archived, "帶 hold 不觸發閉環");
    let after = store.snapshot(&scope()).expect("snapshot").revision().0;
    assert!(after > before, "the conclude commit advances the scope revision");

    let listed = client.list_discussions(false).expect("list live discussions");
    assert!(
        listed.discussions.iter().any(|d| d.slug == "staged-talk"),
        "the record stays in the live list"
    );
    let archived = client.list_discussions(true).expect("list archived discussions");
    assert!(
        archived.discussions.iter().all(|d| d.slug != "staged-talk"),
        "the record never reaches the archive"
    );
}

#[test]
fn archive_reports_the_full_engine_outcome_over_the_wire() {
    let store = archive_ready_store();
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let resp = client.archive("demo-archive", false, false).expect("archive over the wire");

    let dated = resp.dated_name.as_deref().expect("datedName is the sentinel for a new server");
    assert!(
        dated.ends_with("demo-archive"),
        "datedName carries the archive destination: {dated}"
    );
    let spec = resp.specs.iter().find(|s| s.capability == "user-auth").expect("capability listed");
    assert_eq!(spec.added, 1, "the delta's one ADDED requirement is counted: {spec:?}");
    assert_eq!((spec.modified, spec.removed, spec.renamed), (0, 0, 0));
    let discussion = resp
        .archived_discussions
        .iter()
        .find(|d| d.slug == "scope-talk")
        .expect("the source discussion co-travels");
    assert!(
        discussion.file.ends_with("scope-talk.md"),
        "the archived file name travels: {}",
        discussion.file
    );
    assert_eq!(
        resp.evidence_recorded,
        Some(false),
        "a change with no per-task evidence reports the fact"
    );
}

#[test]
fn a_completed_task_lands_a_task_completed_event_in_the_outbox() {
    let store = seeded_store();
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let done = client.task_done("demo", "1", &[], None).expect("task done");
    assert!(!done.already_done, "the task flipped for the first time");

    let entries = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox");
    let completed: Vec<_> = entries
        .iter()
        .filter(|e| e.record.name == "task-completed")
        .collect();
    assert_eq!(completed.len(), 1, "exactly one task-completed event landed in the outbox");
}
