//! Dual-path consistency of the engine over a store port (host-runtime spec
//! 「engine 動詞經儲存接口執行」的「兩條路徑結果一致」). The same Engine command
//! runs over the local fs seam (`FsStore`) and over `port_exec::execute` on a
//! `MemoryStorePort` with the same content. The typed outcome, the kinds of
//! domain events and the error code of a not_found failure must agree.

use speclink_core::command::{
    execute as engine_execute, Command, CommandOutcome, DomainEvent, ErrorCode, ExecutionContext, ListOutcome,
};
use speclink_core::store::Store;
use speclink_core::workspace::{Workspace, DEFAULT_SPEC_DIR};
use speclink_fs::FsStore;
use speclink_host::port_exec::{self, ExecError, PortExecution};
use speclink_store_port::{keys, DocKey, MemoryStorePort, StorePort};

mod common;
use common::{host_ctx, keys_under, scope, scope_version, seed, ACTOR};

const CHANGE: &str = "demo";
const META: &str = "schema: spec-driven\n";
const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";
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

/// A `MemoryStorePort` with the seed documents, written in one commit.
fn seed_port() -> MemoryStorePort {
    let port = MemoryStorePort::new();
    let docs = seed_docs()
        .into_iter()
        .map(|(key, content)| (key, content.to_string()))
        .collect();
    seed(&port, docs);
    port
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
        detail: false,
        archived: false,
    }
}

#[test]
fn list_changes_matches_and_commits_nothing() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let before = scope_version(&port);

    let (fs_out, fs_events) = on_fs(&fs, list_cmd());
    let ported = on_port(&port, list_cmd()).expect("ported list");

    // Each store dates a change with its own clock (the file times, the
    // commit times), so `modifiedAt` is the one field that differs: both
    // sides must carry it, and everything else must be the same.
    let changes = |o: &CommandOutcome| match o {
        CommandOutcome::List(ListOutcome::Sections { changes, .. }) => {
            let mut items = serde_json::to_value(changes).unwrap();
            for item in items.as_array_mut().expect("a changes array") {
                let at = item.as_object_mut().unwrap().remove("modifiedAt");
                assert!(at.is_some_and(|at| at.is_string()), "every change is dated");
            }
            items
        }
        other => panic!("expected list, got {other:?}"),
    };
    assert_eq!(changes(&fs_out), changes(&ported.outcome));
    assert!(fs_events.is_empty() && ported.events.is_empty());
    // 「查詢不提交」：沒有提交，範圍版本號不變，回傳的是「沒有」。
    assert_eq!(ported.scope_version, None);
    assert_eq!(scope_version(&port), before);
}

#[test]
fn the_archived_list_matches_on_both_paths() {
    // verb-contract「已封存變更的清單」Scenario「三種執行方式輸出相同」的宿主套件那一邊：
    // 同樣的已封存文件，經本機儲存與經儲存接口，`archived` 深度相等。
    const STAMPED: &str = "schema: spec-driven\ncreated_by: Ann\nfrom_discussion: two\nreviewed_at: 2026-11-20\nreviewed_tasks_total: 1\nreviewed_scope:\n  - path: src/a.rs\n    hash: 0f9c\n";
    let docs: [(&str, &str, &str); 7] = [
        ("2026-11-22-add-two", ".meta.yaml", STAMPED),
        ("2026-11-22-add-two", "tasks.md", "- [x] 1.1 a\n"),
        ("2026-11-22-add-two", "specs/auth/spec.md", "## ADDED Requirements\n"),
        ("2026-11-15-open", ".meta.yaml", "schema: spec-driven\nfields:\n  ticket: \"PROJ-1\"\n"),
        ("2026-11-15-open", "verify.md", "# Verify\n\n## Round 1\n"),
        ("2026-11-01-broken", ".meta.yaml", "schema: [unclosed\n"),
        ("legacy", ".openspec.yaml", "schema: spec-driven\ncreated_by: Old\n"),
    ];
    let (dir, fs) = seed_fs();
    let archive = dir.path().join(DEFAULT_SPEC_DIR).join("changes").join("archive");
    let port = seed_port();
    let mut keyed = Vec::new();
    for (dated, doc, text) in docs {
        let file = archive.join(dated).join(doc.split('/').collect::<std::path::PathBuf>());
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
        // A plain key: the builder takes dated names only, and a store can hold
        // an older archived change without a date.
        keyed.push((DocKey::new(format!("{}{dated}/{doc}", keys::ARCHIVED_CHANGES)).unwrap(), text.to_string()));
    }
    seed(&port, keyed);
    let cmd = || Command::List {
        sort: "name".to_string(),
        specs: false,
        changes: false,
        worktrees: Default::default(),
        detail: false,
        archived: true,
    };
    let archived = |o: &CommandOutcome| match o {
        CommandOutcome::List(ListOutcome::Archived(items)) => serde_json::to_value(items).unwrap(),
        other => panic!("expected list, got {other:?}"),
    };
    let (fs_out, _) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported archived list");
    assert_eq!(archived(&fs_out), archived(&ported.outcome));
    assert_eq!(archived(&fs_out).as_array().map(Vec::len), Some(4), "{}", archived(&fs_out));
    assert_eq!(ported.scope_version, None, "a query commits nothing");
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

fn port_content(port: &MemoryStorePort, key: &str) -> Option<String> {
    port.read(&scope(), &[DocKey::new(key).unwrap()])
        .expect("read")
        .remove(0)
        .map(|doc| doc.content)
}

/// Archive stamps the @trace `updated:` line with the wall-clock second, and
/// the two paths run one after the other, so they can land on either side of
/// a second boundary. Compare the spec with that value masked.
fn without_trace_time(spec: Option<String>) -> Option<String> {
    spec.map(|s| {
        s.lines()
            .map(|line| if line.starts_with("updated: ") { "updated: <time>" } else { line })
            .collect::<Vec<_>>()
            .join("\n")
    })
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
    assert_eq!(keys_under(&port, "changes/"), Vec::<String>::new());
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
    assert_eq!(keys_under(&port, "changes/"), Vec::<String>::new());
    assert_eq!(
        without_trace_time(fs.read_canonical_spec("auth")),
        without_trace_time(port_content(&port, "specs/auth/spec.md"))
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
    assert_eq!(keys_under(&port, "discussions/"), Vec::<String>::new());
    assert_eq!(
        keys_under(&port, "archive/discussions/"),
        ["archive/discussions/2026-11-10-board-search.md"]
    );
    assert_eq!(
        port_content(&port, "archive/discussions/2026-11-10-board-search.md").as_deref(),
        Some(TALK)
    );
}

// --- language replace／discuss restore（language-replace-and-discuss-restore D2、D4） ---

/// The code of a refused command on each path: the fs seam's and the port's.
fn refusal_codes(fs: &FsStore, port: &MemoryStorePort, cmd: impl Fn() -> Command) -> (ErrorCode, ErrorCode) {
    let fs_code = engine_execute(fs, &fs_engine_ctx(), cmd()).expect_err("the fs path refuses").code;
    let port_code = match on_port(port, cmd()) {
        Err(ExecError::Command(e)) => e.code,
        other => panic!("the port path refuses with a command error, got {other:?}"),
    };
    (fs_code, port_code)
}

#[test]
fn replacing_the_language_document_matches_on_both_paths() {
    const NEW: &str = "# Shared Vocabulary\n\n- Change: 一個提出的修改。\n- Cut: 一刀\n";
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let cmd = |content: &str| Command::LanguageReplace { content: content.to_string() };

    let (fs_out, fs_events) = on_fs(&fs, cmd(NEW));
    let ported = on_port(&port, cmd(NEW)).expect("ported language replace");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    assert!(matches!(&ported.outcome, CommandOutcome::LanguageReplace(o) if !o.created), "{:?}", ported.outcome);
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["language-replaced"]);
    assert_eq!(fs.read_language().as_deref(), Some(NEW));
    assert_eq!(port_content(&port, "LANGUAGE.md").as_deref(), Some(NEW));

    // 空內容：兩條路徑同一個錯誤碼，文件都不變，儲存接口上沒有提交。
    let before = scope_version(&port);
    let codes = refusal_codes(&fs, &port, || cmd("  \n"));
    assert_eq!(codes, (ErrorCode::InvalidArgv, ErrorCode::InvalidArgv));
    assert_eq!(fs.read_language().as_deref(), Some(NEW));
    assert_eq!(port_content(&port, "LANGUAGE.md").as_deref(), Some(NEW));
    assert_eq!(scope_version(&port), before, "a refusal commits nothing");
}

#[test]
fn creating_the_language_document_matches_on_both_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fs = FsStore::new(dir.path(), DEFAULT_SPEC_DIR);
    let port = MemoryStorePort::new();
    let cmd = || Command::LanguageReplace { content: "# Language\n".to_string() };

    let (fs_out, _) = on_fs(&fs, cmd());
    let ported = on_port(&port, cmd()).expect("ported language replace");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    assert!(matches!(&ported.outcome, CommandOutcome::LanguageReplace(o) if o.created), "{:?}", ported.outcome);
    assert_eq!(fs.read_language().as_deref(), Some("# Language\n"));
    assert_eq!(port_content(&port, "LANGUAGE.md").as_deref(), Some("# Language\n"));
}

#[test]
fn restoring_a_discussion_matches_and_is_one_commit_on_the_port() {
    let (_dir, fs) = seed_fs();
    let port = seed_port();
    let archive = || Command::DiscussArchive { slug: "board-search".to_string() };
    let restore = |slug: &str| Command::DiscussRestore { slug: slug.to_string() };
    on_fs(&fs, archive());
    on_port(&port, archive()).expect("ported discuss archive");
    let archived_at = scope_version(&port);

    let (fs_out, fs_events) = on_fs(&fs, restore("board-search"));
    let ported = on_port(&port, restore("board-search")).expect("ported discuss restore");

    assert_eq!(format!("{fs_out:?}"), format!("{:?}", ported.outcome));
    match &ported.outcome {
        CommandOutcome::DiscussRestore(o) => {
            assert_eq!((o.slug.as_str(), o.restored_from.as_str()), ("board-search", "2026-11-10-board-search.md"));
        }
        other => panic!("expected a restore outcome, got {other:?}"),
    }
    assert_eq!(event_kinds(&fs_events), event_kinds(&ported.events));
    assert_eq!(event_kinds(&ported.events), ["discussion-restored"]);
    // 同樣的文件在同樣的位置，內容逐字元不變。
    assert_eq!(fs.read_live_discussion("board-search").as_deref(), Some(TALK));
    assert!(!fs.archived_discussion_exists("board-search"));
    assert_eq!(keys_under(&port, "discussions/"), ["discussions/board-search.md"]);
    assert_eq!(keys_under(&port, "archive/discussions/"), Vec::<String>::new());
    assert_eq!(port_content(&port, "discussions/board-search.md").as_deref(), Some(TALK));
    // 搬回在儲存接口上是一次提交：範圍版本只前進一步。
    assert_eq!(ported.scope_version.map(|v| v.get()), Some(scope_version(&port)));
    assert_eq!(scope_version(&port), archived_at + 1, "one commit");

    // 沒有封存的記錄、已有進行中的記錄：兩條路徑同一個錯誤碼，零寫入。
    assert_eq!(refusal_codes(&fs, &port, || restore("ghost")), (ErrorCode::NotFound, ErrorCode::NotFound));
    on_fs(&fs, archive());
    on_port(&port, archive()).expect("ported discuss archive");
    fs.write_live_discussion("board-search", "a new live record\n").expect("write discussion");
    seed(&port, vec![(keys::discussion("board-search").unwrap(), "a new live record\n".to_string())]);
    let before = scope_version(&port);
    assert_eq!(refusal_codes(&fs, &port, || restore("board-search")), (ErrorCode::Refused, ErrorCode::Refused));
    assert_eq!(scope_version(&port), before, "a refusal commits nothing");
    assert_eq!(fs.read_live_discussion("board-search").as_deref(), Some("a new live record\n"));
    assert_eq!(port_content(&port, "discussions/board-search.md").as_deref(), Some("a new live record\n"));
    assert!(fs.archived_discussion_exists("board-search"));
    assert_eq!(
        port_content(&port, "archive/discussions/2026-11-10-board-search.md").as_deref(),
        Some(TALK),
        "the archived record is untouched"
    );
}
