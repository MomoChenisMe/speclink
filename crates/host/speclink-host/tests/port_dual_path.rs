//! Dual-path consistency of the engine over a store port (host-runtime spec
//! 「engine 動詞經儲存接口執行」的「兩條路徑結果一致」). The same Engine command
//! runs over the local fs seam (`FsStore`) and over `port_exec::execute` on a
//! `MemoryStorePort` with the same content. The typed outcome, the kinds of
//! domain events and the error code of a not_found failure must agree.

use speclink_core::command::{
    execute as engine_execute, Command, CommandOutcome, DomainEvent, ErrorCode, ExecutionContext,
};
use speclink_core::config::ResolvedPolicy;
use speclink_core::store::Store;
use speclink_core::workspace::{Workspace, DEFAULT_SPEC_DIR};
use speclink_fs::FsStore;
use speclink_host::binding::local_default_binding;
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_host::port_exec::{self, ExecError, PortExecution};
use speclink_store_port::{
    keys, Actor as PortActor, Commit, DocKey, Expected, KeyPrefix, MemoryStorePort, Op, ScopeId,
    StorePort, Timestamp,
};

const CHANGE: &str = "demo";
const META: &str = "schema: spec-driven\n";
const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";
const ACTOR: &str = "Alice <alice@example.com>";
const TALK: &str = "---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-10\n---\n\n## Round 1\n\nFirst round.\n";
const LANGUAGE: &str = "# Shared Vocabulary\n\n- Change: a proposed edit.\n";
const DELTA: &str = "## Purpose\n\n本 capability 負責身分驗證的簽發與撤銷，涵蓋權杖生命週期各階段的可觀察行為、失敗處置與稽核紀錄。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

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
                worktree: false,
            },
            "",
        ),
    }
}

/// The documents both paths start from: one change with tasks, one live
/// discussion and the shared vocabulary.
fn seed_docs() -> Vec<(DocKey, &'static str)> {
    vec![
        (keys::change_meta(CHANGE).unwrap(), META),
        (keys::change_artifact(CHANGE, "tasks.md").unwrap(), TASKS),
        (
            keys::change_artifact(CHANGE, "specs/auth/spec.md").unwrap(),
            DELTA,
        ),
        (keys::discussion("board-search").unwrap(), TALK),
        (keys::language(), LANGUAGE),
    ]
}

/// An `FsStore` over a temp spec tree with the seed documents. Returns the
/// guard so the temp dir outlives the store.
fn seed_fs() -> (tempfile::TempDir, FsStore) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = FsStore::new(dir.path(), DEFAULT_SPEC_DIR);
    store.create_change(CHANGE, META).expect("create change");
    store
        .write_artifact(CHANGE, "tasks.md", TASKS)
        .expect("write tasks");
    store
        .write_artifact(CHANGE, "specs/auth/spec.md", DELTA)
        .expect("write delta spec");
    store
        .write_live_discussion("board-search", TALK)
        .expect("write discussion");
    std::fs::write(
        dir.path().join(DEFAULT_SPEC_DIR).join("LANGUAGE.md"),
        LANGUAGE,
    )
    .expect("write LANGUAGE.md");
    (dir, store)
}

fn scope() -> ScopeId {
    ScopeId::new("default/main").unwrap()
}

/// A `MemoryStorePort` with the seed documents, written in one commit.
fn seed_port() -> MemoryStorePort {
    let port = MemoryStorePort::new();
    let ops = seed_docs()
        .into_iter()
        .map(|(key, content)| Op::Put {
            key,
            content: content.to_string(),
            expected: Expected::Absent,
        })
        .collect();
    let commit = Commit::new(
        PortActor::new("seed", "seed").unwrap(),
        "seed",
        Timestamp::new(1).unwrap(),
        ops,
        Vec::new(),
    )
    .unwrap();
    port.commit(&scope(), &commit).expect("seed commit");
    port
}

fn scope_version(port: &MemoryStorePort) -> u64 {
    port.list(&scope(), &KeyPrefix::new("").unwrap())
        .expect("list")
        .version
        .get()
}

fn on_port(port: &MemoryStorePort, cmd: Command) -> Result<PortExecution, ExecError> {
    port_exec::execute(port, &scope(), &host_ctx(), cmd)
}

fn on_fs(fs: &FsStore, cmd: Command) -> (CommandOutcome, Vec<DomainEvent>) {
    engine_execute(fs, &fs_engine_ctx(), cmd).expect("fs command")
}

fn list_cmd() -> Command {
    Command::List {
        sort: "name".to_string(),
        specs: false,
        changes: false,
        worktrees: Default::default(),
    }
}

#[test]
fn list_changes_matches_and_commits_nothing() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let before = scope_version(&port);

    let (fs_out, fs_events) = on_fs(&fs, list_cmd());
    let ported = on_port(&port, list_cmd()).expect("ported list");

    let changes = |o: &CommandOutcome| match o {
        CommandOutcome::List(l) => serde_json::to_value(&l.changes).unwrap(),
        other => panic!("expected list, got {other:?}"),
    };
    assert_eq!(changes(&fs_out), changes(&ported.outcome));
    assert!(fs_events.is_empty() && ported.events.is_empty());
    // 「查詢不提交」：沒有提交，範圍版本號不變，回傳的是「沒有」。
    assert_eq!(ported.scope_version, None);
    assert_eq!(scope_version(&port), before);
}

#[test]
fn status_of_one_change_matches() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let status_cmd = || Command::Status {
        change: Some(CHANGE.to_string()),
        schema: None,
    };
    let mut fs_ctx = fs_engine_ctx();
    fs_ctx.workspace = Some(Workspace {
        root: std::path::PathBuf::new(),
        spec_dir_name: DEFAULT_SPEC_DIR.to_string(),
    });
    let (fs_out, _) = engine_execute(&fs, &fs_ctx, status_cmd()).expect("fs status");
    let ported = on_port(&port, status_cmd()).expect("ported status");

    let report = |o: &CommandOutcome| match o {
        CommandOutcome::Status(r) => serde_json::to_value(r).unwrap(),
        other => panic!("expected status, got {other:?}"),
    };
    assert_eq!(report(&fs_out), report(&ported.outcome));
    assert_eq!(ported.scope_version, None);
}

#[test]
fn discussion_list_matches() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let cmd = || Command::DiscussList { archived: false };
    let (fs_out, _) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported discuss list");

    // `path` is a display location and differs by backend; the rest is content.
    let rows = |o: &CommandOutcome| match o {
        CommandOutcome::DiscussList(infos) => infos
            .iter()
            .map(|i| {
                (
                    i.slug.clone(),
                    i.topic.clone(),
                    i.status.clone(),
                    i.rounds,
                    i.created.clone(),
                    i.archived,
                )
            })
            .collect::<Vec<_>>(),
        other => panic!("expected discuss list, got {other:?}"),
    };
    assert_eq!(rows(&fs_out), rows(&ported.outcome));
    assert_eq!(
        rows(&ported.outcome),
        vec![(
            "board-search".to_string(),
            "Board search".to_string(),
            "open".to_string(),
            1,
            "2026-11-10".to_string(),
            false
        )]
    );
}

#[test]
fn shared_vocabulary_matches() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let (fs_out, _) = on_fs(&fs, Command::LanguageShow);
    let ported = on_port(&port, Command::LanguageShow).expect("ported language show");
    let content = |o: &CommandOutcome| match o {
        CommandOutcome::Language(c) => c.clone(),
        other => panic!("expected a language outcome, got {other:?}"),
    };
    assert_eq!(content(&fs_out), LANGUAGE);
    assert_eq!(content(&ported.outcome), LANGUAGE);
}

#[test]
fn not_found_carries_the_same_error_code() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let missing = || Command::Status {
        change: Some("nonexistent".to_string()),
        schema: None,
    };
    let fs_err = engine_execute(&fs, &fs_engine_ctx(), missing()).expect_err("fs status fails");
    let code = match on_port(&port, missing()).expect_err("ported status fails") {
        ExecError::Command(e) => e.code,
        ExecError::Store(e) => panic!("expected a command error, got store {e:?}"),
    };
    assert_eq!(fs_err.code, ErrorCode::NotFound);
    assert_eq!(code, ErrorCode::NotFound);
}

// --- 寫入、刪除變更、封存變更、封存討論 ---

fn event_kinds(events: &[DomainEvent]) -> Vec<&'static str> {
    events.iter().map(DomainEvent::kind).collect()
}

fn port_keys(port: &MemoryStorePort, prefix: &str) -> Vec<String> {
    port.list(&scope(), &KeyPrefix::new(prefix).unwrap())
        .expect("list")
        .entries
        .into_iter()
        .map(|entry| entry.key.as_str().to_string())
        .collect()
}

fn port_content(port: &MemoryStorePort, key: &str) -> Option<String> {
    port.read(&scope(), &[DocKey::new(key).unwrap()])
        .expect("read")
        .remove(0)
        .map(|doc| doc.content)
}

#[test]
fn task_done_matches_and_records_the_same_evidence() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let before = scope_version(&port);
    let done_cmd = || Command::TaskDone {
        task_id: "1".to_string(),
        change: Some(CHANGE.to_string()),
        touched_files: Some(vec!["src/app.rs".to_string()]),
        head_commit: None,
    };
    let (fs_out, fs_events) = on_fs(&fs, done_cmd());
    let ported = on_port(&port, done_cmd()).expect("ported task done");

    // The stable id is stamped fresh on each path; the rest is the same.
    let shape = |o: &CommandOutcome| match o {
        CommandOutcome::TaskDone(o) => (
            o.change.clone(),
            o.task_id,
            o.task_id_arg.clone(),
            o.description.clone(),
            o.already,
            o.touched_files.clone(),
        ),
        other => panic!("expected a task-done outcome, got {other:?}"),
    };
    assert_eq!(shape(&fs_out), shape(&ported.outcome));
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["task-completed"]);
    assert_eq!(ported.scope_version.map(|v| v.get()), Some(scope_version(&port)));
    assert!(scope_version(&port) > before);

    // 寫流程會讀寫的 evidence：兩條路徑記下同樣的檔案。
    let touched = |text: Option<String>| {
        let record: serde_json::Value = serde_json::from_str(&text.expect("evidence")).unwrap();
        record["entries"][0]["touchedFiles"].clone()
    };
    assert_eq!(
        touched(fs.read_evidence(CHANGE)),
        touched(port_content(&port, "changes/demo/.evidence.json"))
    );
}

#[test]
fn discarding_a_change_matches_and_removes_every_document() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let cmd = || Command::Discard {
        change: CHANGE.to_string(),
        force: false,
    };
    let (fs_out, fs_events) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported discard");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["change-discarded"]);
    assert!(!fs.change_exists(CHANGE));
    assert_eq!(port_keys(&port, "changes/"), Vec::<String>::new());
}

#[test]
fn archiving_a_change_matches_and_moves_every_document() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let cmd = || Command::Archive {
        change: Some(CHANGE.to_string()),
        skip_specs: false,
        no_validate: true,
        mark_tasks_complete: true,
        carry_review: false,
        carry_verify: false,
    };
    let (fs_out, fs_events) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported archive");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["change-archived"]);
    let dated = match &ported.outcome {
        CommandOutcome::Archive(o) => o.dated_name.clone(),
        other => panic!("expected an archive outcome, got {other:?}"),
    };

    // The same documents at the same places, with the same content.
    assert!(!fs.change_exists(CHANGE));
    assert_eq!(port_keys(&port, "changes/"), Vec::<String>::new());
    assert_eq!(
        fs.read_canonical_spec("auth"),
        port_content(&port, "specs/auth/spec.md")
    );
    for doc in ["tasks.md", "specs/auth/spec.md"] {
        assert_eq!(
            fs.read_archived_artifact(&dated, doc),
            port_content(&port, &format!("archive/changes/{dated}/{doc}")),
            "{doc}"
        );
    }
    assert!(port_content(&port, &format!("archive/changes/{dated}/.meta.yaml")).is_some());
}

#[test]
fn archiving_a_discussion_matches_and_moves_the_record() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let cmd = || Command::DiscussArchive {
        slug: "board-search".to_string(),
    };
    let (fs_out, fs_events) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported discuss archive");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["discussion-archived"]);
    assert!(!fs.live_discussion_exists("board-search"));
    assert!(fs.archived_discussion_exists("board-search"));
    assert_eq!(port_keys(&port, "discussions/"), Vec::<String>::new());
    assert_eq!(
        port_keys(&port, "archive/discussions/"),
        ["archive/discussions/2026-11-10-board-search.md"]
    );
    assert_eq!(
        port_content(&port, "archive/discussions/2026-11-10-board-search.md").as_deref(),
        Some(TALK)
    );
}
