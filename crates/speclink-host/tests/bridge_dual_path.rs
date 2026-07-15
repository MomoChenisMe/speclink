//! Dual-path consistency of the engine-over-TeamStore bridge (host-runtime
//! spec「雙路徑 outcome 一致」). The same Engine command run over the local fs
//! seam (`FsStore`) and over the bridge (a `MemoryStore` scope) with the same
//! content must yield the same typed outcome structure, the same kind of
//! domain events, and the same error code on a not_found failure.

use speclink_core::command::{
    execute as engine_execute, Command, CommandError, CommandOutcome, DomainEvent, ErrorCode,
    ExecutionContext,
};
use speclink_core::config::ResolvedPolicy;
use speclink_core::store::Store;
use speclink_core::workspace::Workspace;
use speclink_fs::FsStore;
use speclink_host::binding::local_default_binding;
use speclink_host::bridge::{self, BridgeError};
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, OutboxCursor, Scope, TeamStore};

const CHANGE: &str = "demo";
const META: &str = "schema: spec-driven\n";
const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";
const ACTOR: &str = "Alice <alice@example.com>";

/// The Engine context the fs seam runs under: same identity the bridge injects,
/// no host workspace (built-in `spec-driven` resolves without one) so the only
/// difference between the two paths is the storage backend.
fn fs_engine_ctx() -> ExecutionContext {
    ExecutionContext {
        actor: Some(ACTOR.to_string()),
        repo: Some("main".to_string()),
        ..Default::default()
    }
}

fn host_ctx() -> SpeclinkExecutionContext {
    let binding = local_default_binding();
    SpeclinkExecutionContext {
        actor: Actor::Identified {
            display: ACTOR.to_string(),
            source: ActorSource::Explicit,
        },
        project: binding.project,
        repo: binding.repo,
        mode: ExecutionMode::SharedStore,
        policy: EffectiveWorkflowPolicy::new(
            ResolvedPolicy {
                locale: "English".to_string(),
                spec_locale: None,
                tdd: false,
                audit: false,
            },
            "",
        ),
    }
}

/// An `FsStore` over a temp `openspec/` tree seeded with the change content.
/// Returns the guard so the temp dir outlives the store.
fn seed_fs() -> (tempfile::TempDir, FsStore) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = FsStore::new(dir.path(), "openspec");
    store.create_change(CHANGE, META).expect("create change");
    store
        .write_artifact(CHANGE, "tasks.md", TASKS)
        .expect("write tasks");
    (dir, store)
}

/// A `MemoryStore` with the same change content committed into the default
/// scope through one unit of work.
fn seed_teamstore() -> (MemoryStore, Scope) {
    let store = MemoryStore::new();
    let binding = local_default_binding();
    let scope = Scope::new(binding.project, binding.repo);
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext {
                command: "seed".to_string(),
                actor: "seed".to_string(),
            },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: CHANGE.to_string() }, META);
    uow.create(
        DocumentId::ChangeArtifact {
            change: CHANGE.to_string(),
            artifact: "tasks.md".to_string(),
        },
        TASKS,
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    (store, scope)
}

fn event_kinds(events: &[DomainEvent]) -> Vec<&'static str> {
    events.iter().map(DomainEvent::kind).collect()
}

/// The task-flip outcome fields that are storage-independent (the freshly
/// stamped stable id and the written file content are not).
fn task_done_shape(outcome: &CommandOutcome) -> (String, usize, String, String, bool) {
    match outcome {
        CommandOutcome::TaskDone(o) => (
            o.change.clone(),
            o.task_id,
            o.task_id_arg.clone(),
            o.description.clone(),
            o.already,
        ),
        other => panic!("expected a task-done outcome, got {other:?}"),
    }
}

#[test]
fn query_verbs_yield_the_same_outcome_on_both_paths() {
    let ws = Workspace {
        root: std::path::PathBuf::new(),
        spec_dir_name: "openspec".to_string(),
    };
    let (_dir, fs) = seed_fs();
    let (mem, _scope) = seed_teamstore();

    // --- list --sort name (query) ---
    let list_cmd = || Command::List {
        sort: "name".to_string(),
        specs: false,
        changes: false,
    };
    let (fs_out, fs_events) =
        engine_execute(&fs, &fs_engine_ctx(), list_cmd()).expect("fs list");
    let bridged = bridge::execute(&mem, &host_ctx(), list_cmd()).expect("bridged list");
    let fs_changes = match &fs_out {
        CommandOutcome::List(l) => serde_json::to_value(&l.changes).unwrap(),
        other => panic!("expected list, got {other:?}"),
    };
    let br_changes = match &bridged.outcome {
        CommandOutcome::List(l) => serde_json::to_value(&l.changes).unwrap(),
        other => panic!("expected list, got {other:?}"),
    };
    assert_eq!(fs_changes, br_changes, "list changes section matches");
    assert!(fs_events.is_empty() && bridged.events.is_empty(), "queries emit no events");
    assert!(bridged.revision.is_none(), "a query opens no unit of work");

    // --- status --change demo (query) ---
    let status_cmd = || Command::Status {
        change: Some(CHANGE.to_string()),
        schema: None,
    };
    let mut fs_ctx = fs_engine_ctx();
    fs_ctx.workspace = Some(ws.clone());
    let (fs_out, _) = engine_execute(&fs, &fs_ctx, status_cmd()).expect("fs status");
    let bridged = bridge::execute(&mem, &host_ctx(), status_cmd()).expect("bridged status");
    let fs_report = match &fs_out {
        CommandOutcome::Status(r) => serde_json::to_value(r).unwrap(),
        other => panic!("expected status, got {other:?}"),
    };
    let br_report = match &bridged.outcome {
        CommandOutcome::Status(r) => serde_json::to_value(r).unwrap(),
        other => panic!("expected status, got {other:?}"),
    };
    assert_eq!(fs_report, br_report, "status report matches structurally");
}

#[test]
fn mutating_verb_yields_the_same_outcome_and_event_kind_on_both_paths() {
    let (_dir, fs) = seed_fs();
    let (mem, _scope) = seed_teamstore();
    let done_cmd = || Command::TaskDone {
        task_id: "1".to_string(),
        change: Some(CHANGE.to_string()),
    };
    let (fs_out, fs_events) =
        engine_execute(&fs, &fs_engine_ctx(), done_cmd()).expect("fs task done");
    let bridged = bridge::execute(&mem, &host_ctx(), done_cmd()).expect("bridged task done");

    assert_eq!(
        task_done_shape(&fs_out),
        task_done_shape(&bridged.outcome),
        "task-done outcome structure matches"
    );
    assert_eq!(
        event_kinds(&fs_events),
        event_kinds(&bridged.events),
        "same kind of domain event on both paths"
    );
    assert_eq!(
        event_kinds(&bridged.events),
        vec!["task-completed"],
        "task done reports a task-completed event"
    );
    assert!(
        bridged.revision.is_some(),
        "the bridged mutation committed a new revision"
    );
}

#[test]
fn bridged_task_done_lands_document_revision_and_event_in_one_commit() {
    // 橋接寫入原子落店: the flipped document, its advanced revision, and the
    // task-completed event are all visible at one and the same commit.
    let (mem, scope) = seed_teamstore();
    let base = mem.snapshot(&scope).expect("snapshot").revision();

    let done = bridge::execute(
        &mem,
        &host_ctx(),
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some(CHANGE.to_string()),
        },
    )
    .expect("bridged task done");

    let new_rev = done.revision.expect("a mutation commits a revision");
    assert!(new_rev.0 > base.0, "the commit advanced the project revision");

    let snapshot = mem.snapshot(&scope).expect("post-commit snapshot");
    let doc = snapshot
        .read(&DocumentId::ChangeArtifact {
            change: CHANGE.to_string(),
            artifact: "tasks.md".to_string(),
        })
        .expect("read tasks.md")
        .expect("tasks.md is visible");
    assert!(
        doc.content.contains("- [x] 1.1"),
        "the task checkbox is flipped: {}",
        doc.content
    );
    assert_eq!(doc.revision, new_rev, "the document rides the same commit revision");

    let entries = mem.read_outbox(&scope, OutboxCursor(0)).expect("read outbox");
    let completed: Vec<_> = entries
        .iter()
        .filter(|e| e.record.name == "task-completed")
        .collect();
    assert_eq!(completed.len(), 1, "exactly one task-completed event landed");
    assert_eq!(
        completed[0].revision, new_rev,
        "the event rides the same commit as the document"
    );
}

#[test]
fn not_found_failure_carries_the_same_error_code_on_both_paths() {
    let (_dir, fs) = seed_fs();
    let (mem, _scope) = seed_teamstore();
    let missing = || Command::Status {
        change: Some("nonexistent".to_string()),
        schema: None,
    };
    let fs_err: CommandError = engine_execute(&fs, &fs_engine_ctx(), missing())
        .expect_err("fs status on missing change fails");
    let br_err = bridge::execute(&mem, &host_ctx(), missing())
        .expect_err("bridged status on missing change fails");
    let br_code = match br_err {
        BridgeError::Command(e) => e.code,
        BridgeError::Store(e) => panic!("expected a command error, got store {e:?}"),
    };
    assert_eq!(fs_err.code, ErrorCode::NotFound, "fs path classifies not_found");
    assert_eq!(br_code, ErrorCode::NotFound, "bridge path classifies not_found");
}

/// The shared-vocabulary document: fs reads `LANGUAGE.md`, the bridge reads the
/// `DocumentId::Language` document — the same content on both seams (the store
/// contract gained the Language kind so server mode is no longer LANGUAGE-blind).
#[test]
fn language_show_reads_the_shared_vocabulary_on_both_paths() {
    const LANGUAGE: &str = "# Shared Vocabulary\n\n- Change: a proposed edit.\n";

    // fs seam: LANGUAGE.md written into the openspec tree.
    let (dir, fs) = seed_fs();
    std::fs::write(dir.path().join("openspec").join("LANGUAGE.md"), LANGUAGE)
        .expect("write LANGUAGE.md");

    // bridge seam: the Language document committed into the scope.
    let (mem, scope) = seed_teamstore();
    let mut uow = mem
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".to_string(), actor: "seed".to_string() },
        )
        .expect("begin uow");
    uow.create(DocumentId::Language, LANGUAGE);
    mem.commit(uow, Vec::new()).expect("seed language commit");

    let (fs_out, _) =
        engine_execute(&fs, &fs_engine_ctx(), Command::LanguageShow).expect("fs language show");
    let bridged =
        bridge::execute(&mem, &host_ctx(), Command::LanguageShow).expect("bridged language show");

    let content = |o: &CommandOutcome| match o {
        CommandOutcome::Language(c) => c.clone(),
        other => panic!("expected a language outcome, got {other:?}"),
    };
    assert_eq!(content(&fs_out), LANGUAGE, "fs reads the shared vocabulary");
    assert_eq!(
        content(&bridged.outcome),
        LANGUAGE,
        "the bridge reads the seeded Language document identically",
    );
}
