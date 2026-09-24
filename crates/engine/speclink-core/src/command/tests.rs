use super::*;
use crate::teststore::TestStore;

/// Minimal spec-driven change meta accepted by ChangeMeta::from_text.
const META: &str = "schema: spec-driven\n";

fn list_cmd() -> Command {
    Command::List {
        sort: "name".to_string(),
        specs: false,
        changes: false,
        worktrees: Default::default(),
    }
}

// --- 查詢群經唯一進入點回 typed outcome、不產生事件 ---

#[test]
fn list_returns_typed_outcome_without_events() {
    let store = TestStore::with_meta("demo", META);
    let (outcome, events) = execute(&store, &ExecutionContext::default(), list_cmd()).expect("list executes");
    match outcome {
        CommandOutcome::List(list) => {
            let changes = list.changes.expect("changes section present");
            assert!(
                changes.iter().any(|c| c.name == "demo"),
                "list outcome carries the change"
            );
            assert!(list.specs.is_none(), "specs section only when requested");
        }
        other => panic!("expected a list outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "queries never produce events");
}

#[test]
fn status_returns_typed_outcome_without_events() {
    let store = TestStore::with_meta("demo", META);
    let (outcome, events) = execute(
        &store,
        &ExecutionContext::default(),
        Command::Status {
            change: Some("demo".to_string()),
            schema: None,
        },
    )
    .expect("status executes");
    match outcome {
        CommandOutcome::Status(report) => {
            assert_eq!(report.change_name, "demo");
            assert_eq!(report.schema_name, "spec-driven");
        }
        other => panic!("expected a status outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "queries never produce events");
}

#[test]
fn plan_returns_typed_outcome_without_events() {
    // change-plan：plan 是查詢，經唯一進入點回 Plan outcome、零事件。
    let store = TestStore::with_meta("demo", META);
    store.metas.borrow_mut().insert("later".to_string(), "schema: spec-driven\ndepends_on: demo\n".to_string());
    let (outcome, events) = execute(&store, &ExecutionContext::default(), Command::Plan { ranks: None }).expect("plan executes");
    match outcome {
        CommandOutcome::Plan(plan) => {
            assert_eq!(plan.next.as_deref(), Some("demo"));
            assert_eq!(plan.changes.len(), 2);
            assert_eq!(plan.changes[1].blocked_by, ["demo"]);
        }
        other => panic!("expected a plan outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "queries never produce events");
}

#[test]
fn plan_with_an_external_rank_table_orders_by_that_table() {
    // add-change-plan-remote D1：ranks 為 Some 時以外部 rank 圖為準（remote 的
    // board resource），meta 的 board_rank 不參與；None 照舊讀 meta。
    let store = TestStore::with_meta("alpha", "schema: spec-driven\nboard_rank: a\n");
    store.metas.borrow_mut().insert("beta".to_string(), "schema: spec-driven\nboard_rank: z\n".to_string());
    let names = |cmd: Command| match execute(&store, &ExecutionContext::default(), cmd).expect("plan executes").0 {
        CommandOutcome::Plan(plan) => plan.changes.into_iter().map(|c| c.name).collect::<Vec<_>>(),
        other => panic!("expected a plan outcome, got {other:?}"),
    };
    let external = std::collections::BTreeMap::from([
        ("beta".to_string(), "b".to_string()),
        ("alpha".to_string(), "f".to_string()),
    ]);
    assert_eq!(names(Command::Plan { ranks: Some(external) }), ["beta", "alpha"]);
    assert_eq!(names(Command::Plan { ranks: None }), ["alpha", "beta"]);
}

#[test]
fn plan_strict_overlap_reaches_the_engine() {
    // `plan --strict-overlap` 是自己的命令：共用 delta capability 的兩者在 strict
    // 下分兩波，預設的 plan 下同波。
    let store = TestStore::with_meta("a", "schema: spec-driven\ncreated: 2026-09-01\n");
    store.metas.borrow_mut().insert("b".to_string(), "schema: spec-driven\ncreated: 2026-09-02\n".to_string());
    store.put_artifact("a", "specs/desktop-app/spec.md", "## ADDED Requirements\n");
    store.put_artifact("b", "specs/desktop-app/spec.md", "## ADDED Requirements\n");
    let waves = |strict_overlap: bool| {
        match execute(&store, &ExecutionContext::default(), if strict_overlap { Command::PlanStrict } else { Command::Plan { ranks: None } }).expect("plan executes").0 {
            CommandOutcome::Plan(plan) => plan.changes.iter().map(|c| c.wave).collect::<Vec<_>>(),
            other => panic!("expected a plan outcome, got {other:?}"),
        }
    };
    assert_eq!(waves(true), [1, 2]);
    assert_eq!(waves(false), [1, 1]);
}

#[test]
fn plan_dependency_cycle_is_an_error_naming_the_cycle() {
    let store = TestStore::with_meta("a", "schema: spec-driven\ndepends_on: b\n");
    store.metas.borrow_mut().insert("b".to_string(), "schema: spec-driven\ndepends_on: a\n".to_string());
    let err = execute(&store, &ExecutionContext::default(), Command::Plan { ranks: None }).expect_err("a cycle cannot plan");
    assert_eq!(err.code, ErrorCode::Error);
    assert_eq!(err.message, "dependency cycle: a -> b -> a");
}

#[test]
fn validate_returns_typed_outcome_without_events() {
    let store = TestStore::with_meta("demo", META);
    let (outcome, events) = execute(
        &store,
        &ExecutionContext::default(),
        Command::Validate {
            item: Some("demo".to_string()),
            all: false,
            changes: false,
            specs: false,
            strict: false,
        },
    )
    .expect("validate executes");
    match outcome {
        CommandOutcome::Validate(v) => {
            assert_eq!(v.results.len(), 1);
            assert_eq!(v.results[0].change, "demo");
        }
        other => panic!("expected a validate outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "queries never produce events");
}

// --- validate 的 --specs／--all 旗標語意（design D4；spec spec-validation
//     「validate --specs 驗證正式規格」）---

/// 一個 change（demo）＋一份 Purpose 缺席的正式規格（auth）的專案。
fn validate_flags_store() -> TestStore {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
    store.canonical.borrow_mut().insert(
        "auth".to_string(),
        "# auth Specification\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n"
            .to_string(),
    );
    store
}

fn validated_names(store: &TestStore, all: bool, changes: bool, specs: bool) -> Vec<String> {
    let (outcome, _) = execute(
        store,
        &ExecutionContext::default(),
        Command::Validate { item: None, all, changes, specs, strict: false },
    )
    .expect("validate executes");
    let CommandOutcome::Validate(v) = outcome else { panic!("expected a validate outcome") };
    v.results.iter().map(|r| r.change.clone()).collect()
}

#[test]
fn validate_specs_flag_alone_validates_only_specs() {
    // spec：`--specs` 單獨傳入時僅驗規格。
    let store = validate_flags_store();
    assert_eq!(validated_names(&store, false, false, true), vec!["auth".to_string()]);
}

#[test]
fn validate_all_validates_changes_and_specs() {
    // spec：`--all` 同時驗 changes 與 specs。
    let store = validate_flags_store();
    assert_eq!(
        validated_names(&store, true, false, false),
        vec!["demo".to_string(), "auth".to_string()]
    );
    // `--specs --changes` 同傳的聯集語意與 `--all` 等效（design 風險項）。
    assert_eq!(
        validated_names(&store, false, true, true),
        vec!["demo".to_string(), "auth".to_string()]
    );
}

#[test]
fn validate_without_flags_still_validates_only_changes() {
    // spec Scenario「預設行為不變」：兩旗標皆缺席時只驗 changes。
    let store = validate_flags_store();
    assert_eq!(validated_names(&store, false, false, false), vec!["demo".to_string()]);
    assert_eq!(validated_names(&store, false, true, false), vec!["demo".to_string()]);
}

#[test]
fn validate_item_with_specs_flag_is_rejected() {
    // spec Scenario「--specs 與 change 名稱同傳被拒」：--specs 驗的是正典
    // 規格、無法指定單一規格，與名稱同傳是參數錯誤——大聲拒絕，不做聯集。
    let store = validate_flags_store();
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::Validate {
            item: Some("demo".to_string()),
            all: false,
            changes: false,
            specs: true,
            strict: false,
        },
    )
    .expect_err("--specs with an item must refuse");
    assert_eq!(err.code, ErrorCode::InvalidArgv);
    assert!(err.message.contains("--specs"), "the flag is named: {}", err.message);
    assert!(err.message.contains("--all"), "the way out is named: {}", err.message);
}

// --- 壞 metadata 的查詢群 fail closed 與 list 診斷（spec command-runtime）---

#[test]
fn single_change_queries_on_corrupt_meta_are_invalid_config_with_path_and_reason() {
    // spec「單一 change 查詢對壞 metadata fail closed」＋「穩定錯誤碼註冊表」
    // Example 新行：.openspec.yaml 存在但解析失敗 → invalid_config，訊息
    // 沿 ConfigError 形式指出 workspace 相對路徑與解析原因。
    let store = TestStore::with_meta("demo", BAD_META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
    let cmds: Vec<(&str, Command)> = vec![
        ("status", Command::Status { change: Some("demo".to_string()), schema: None }),
        (
            "instructions",
            Command::Instructions { artifact: None, change: Some("demo".to_string()), schema: None },
        ),
        (
            "validate",
            Command::Validate {
                item: Some("demo".to_string()),
                all: false,
                changes: false,
                specs: false,
                strict: false,
            },
        ),
        ("analyze", Command::Analyze { change: Some("demo".to_string()) }),
        // drift no longer routes through `execute` — it is CLI-orchestrated
        // (Host-collected facts → compute_* → merge); its corrupt-meta
        // fail-closed is covered at the CLI layer (meta_fail_closed).
        (
            "artifact cat",
            Command::ArtifactCat { artifact: "tasks".to_string(), change: Some("demo".to_string()) },
        ),
    ];
    for (verb, cmd) in cmds {
        let err = execute(&store, &ExecutionContext::default(), cmd)
            .expect_err(&format!("{verb} on corrupt meta must refuse"));
        assert_eq!(
            err.code,
            ErrorCode::InvalidConfig,
            "{verb} must classify invalid_config, got {:?}: {}",
            err.code,
            err.message
        );
        assert!(
            err.message.starts_with("invalid openspec/changes/demo/.openspec.yaml: "),
            "{verb} message must name path then reason: {}",
            err.message
        );
        assert!(
            err.message.len() > "invalid openspec/changes/demo/.openspec.yaml: ".len(),
            "{verb} message must carry the parse reason"
        );
    }
}

#[test]
fn list_on_corrupt_meta_flags_the_item_and_keeps_valid_items_intact() {
    // spec「list 對壞 metadata 標 invalid 而不失效」：壞檔項目帶診斷，
    // 有效項目不帶且內容與無壞檔時一致。
    let store = TestStore::with_meta("good", META);
    store.metas.borrow_mut().insert("broken".to_string(), BAD_META.to_string());
    store.put_artifact("good", "tasks.md", "- [x] 1.1 a\n");
    let (outcome, _) = execute(&store, &ExecutionContext::default(), list_cmd()).expect("list must stay available");
    let CommandOutcome::List(list) = outcome else {
        panic!("expected a list outcome");
    };
    let changes = list.changes.expect("changes section present");
    assert_eq!(changes.len(), 2, "corrupt meta must not drop the change");
    let broken = changes.iter().find(|c| c.name == "broken").expect("broken listed");
    let reason = broken.meta_error.as_deref().expect("corrupt item carries the diagnostic");
    assert!(!reason.is_empty());
    let good = changes.iter().find(|c| c.name == "good").expect("good listed");
    assert!(good.meta_error.is_none(), "valid item carries no diagnostic");
    assert_eq!((good.completed_tasks, good.total_tasks), (1, 1));
    assert_eq!(good.status, "done");
}

// --- 壞 change metadata 的生命週期寫入 fail closed（spec change-lifecycle）---

/// `.openspec.yaml` 存在但 YAML 解析失敗的固定樣本（與 store_fs 測試同款）。
const BAD_META: &str = ": : :\n\t bad yaml [unclosed\n";

#[test]
fn task_flip_on_corrupt_meta_refuses_and_leaves_files_untouched() {
    // spec「task done 因蘊含開工標記而拒絕」：done 與 undone 皆拒，
    // tasks.md 與 .openspec.yaml 逐位元不變。
    const TASKS: &str = "- [ ] 1.1 a\n- [x] 1.2 b\n";
    let store = TestStore::with_meta("demo", BAD_META);
    store.put_artifact("demo", "tasks.md", TASKS);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    )
    .expect_err("task done on corrupt meta must refuse");
    assert!(
        err.message.contains("openspec/changes/demo/.openspec.yaml"),
        "error must name the metadata file: {}",
        err.message
    );
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
    )
    .expect_err("task undone on corrupt meta must refuse");
    assert!(err.message.contains("openspec/changes/demo/.openspec.yaml"));
    assert_eq!(
        store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
        TASKS,
        "tasks.md byte-identical"
    );
    assert_eq!(store.meta("demo"), BAD_META, "meta byte-identical");
    assert_eq!(*store.artifact_writes.borrow(), 0, "no artifact write on refusal");
    assert_eq!(*store.meta_writes.borrow(), 0, "no meta write on refusal");
}

#[test]
fn claim_on_corrupt_meta_refuses_naming_the_file() {
    let store = TestStore::with_meta("demo", BAD_META);
    let err = execute(&store, &ExecutionContext::default(), Command::Claim { name: "demo".to_string() })
        .expect_err("claim on corrupt meta must refuse");
    assert!(
        err.message.contains("openspec/changes/demo/.openspec.yaml"),
        "error must name the metadata file: {}",
        err.message
    );
    assert_eq!(store.meta("demo"), BAD_META);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn archive_on_open_review_ticket_refuses_before_marking_tasks_complete() {
    // 未結工單守門與壞 metadata 守門同級：拒絕路徑不得留下副作用。
    // --mark-tasks-complete 的前置寫入排在守門之前的話，被拒的封存仍會把
    // tasks.md 全勾成完成——任務狀態被污染且不回滾。
    const TASKS: &str = "- [ ] 1.1 open\n";
    const TICKET: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n";
    let store = TestStore::with_meta("demo", "schema: spec-driven\n");
    store.put_artifact("demo", "tasks.md", TASKS);
    store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: true,
            no_validate: true,
            mark_tasks_complete: true,
            carry_review: false,
            carry_verify: false,
        },
    )
    .expect_err("open review ticket must refuse archive");
    assert!(err.message.contains("--carry-review"), "three disposals listed: {}", err.message);
    assert_eq!(
        store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
        TASKS,
        "tasks.md must stay byte-identical when the gate refuses"
    );
    assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
}

#[test]
fn archive_on_corrupt_meta_refuses_without_moving_or_merging() {
    // spec「archive 對壞 metadata 拒絕」：正典未併入、目錄未移動；
    // --mark-tasks-complete 的前置寫入也不得發生。
    const TASKS: &str = "- [ ] 1.1 open\n";
    for mark in [false, true] {
        let store = TestStore::with_meta("demo", BAD_META);
        store.put_artifact("demo", "tasks.md", TASKS);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::Archive {
                change: Some("demo".to_string()),
                skip_specs: false,
                no_validate: false,
                mark_tasks_complete: mark,
                carry_review: false,
            carry_verify: false,
            },
        )
        .expect_err("archive on corrupt meta must refuse");
        assert!(
            err.message.contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file (mark={mark}): {}",
            err.message
        );
        assert!(store.metas.borrow().contains_key("demo"), "change not moved (mark={mark})");
        assert!(store.archived_metas.borrow().is_empty(), "archive untouched (mark={mark})");
        assert!(store.canonical.borrow().is_empty(), "canon untouched (mark={mark})");
        assert_eq!(
            store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
            TASKS,
            "tasks.md byte-identical (mark={mark})"
        );
        assert_eq!(*store.artifact_writes.borrow(), 0, "no write (mark={mark})");
    }
}

#[test]
fn new_artifact_on_corrupt_meta_refuses_without_default_schema_fallback() {
    // spec：壞 metadata 不得被解讀為預設 schema 而照常產出 artifact。
    let store = TestStore::with_meta("demo", BAD_META);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::NewArtifact {
            kind: "design".to_string(),
            capability: None,
            change: Some("demo".to_string()),
            content: None,
            force: false,
            new_capability: false,
        },
    )
    .expect_err("new artifact on corrupt meta must refuse");
    assert!(
        err.message.contains("openspec/changes/demo/.openspec.yaml"),
        "error must name the metadata file: {}",
        err.message
    );
    assert!(store.artifacts.borrow().is_empty(), "no artifact created via default schema");
    assert_eq!(*store.artifact_writes.borrow(), 0);
}

#[test]
fn newcmd_gate_keeps_the_change_not_found_error() {
    // spec Scenario「change 不存在時維持既有錯誤」：主閘不得改變
    // 找不到 change 的錯誤碼與訊息。
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::NewArtifact {
            kind: "spec".to_string(),
            capability: Some("brand-new-cap".to_string()),
            change: Some("no-such-change".to_string()),
            content: Some("## ADDED Requirements\n\n### Requirement: R1\n\nOk.\n".to_string()),
            force: false,
            new_capability: false,
        },
    )
    .expect_err("missing change must fail");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(err.message, "Change 'no-such-change' not found", "frozen CLI text");
}

// --- 不存在的主體：not_found，訊息沿用現行 CLI 文字 ---

#[test]
fn status_of_missing_change_is_not_found() {
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::Status {
            change: Some("ghost".to_string()),
            schema: None,
        },
    )
    .expect_err("missing change must fail");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(err.code.as_str(), "not_found", "stable registry string");
    assert_eq!(err.message, "Change 'ghost' not found.", "frozen CLI text");
}

#[test]
fn validate_of_missing_change_is_not_found() {
    let store = TestStore::default();
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::Validate {
            item: Some("ghost".to_string()),
            all: false,
            changes: false,
            specs: false,
            strict: false,
        },
    )
    .expect_err("missing change must fail");
    assert_eq!(err.code, ErrorCode::NotFound);
}

// --- auto-detect 語意：零與多個 active change ---

#[test]
fn status_with_no_changes_is_not_found_with_the_informational_text() {
    let store = TestStore::default();
    let err = execute(&store, &ExecutionContext::default(), Command::Status { change: None, schema: None })
        .expect_err("no changes must fail resolution");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(
        err.message,
        "No active changes. Create one with: speclink new change <name>"
    );
}

#[test]
fn status_with_multiple_changes_is_invalid_argv_naming_candidates() {
    let store = TestStore::with_meta("alpha", META);
    store.metas.borrow_mut().insert("beta".to_string(), META.to_string());
    let err = execute(&store, &ExecutionContext::default(), Command::Status { change: None, schema: None })
        .expect_err("ambiguous auto-detect must fail");
    assert_eq!(err.code, ErrorCode::InvalidArgv);
    assert!(
        err.message.starts_with("Multiple changes found. Use --change to specify one:"),
        "frozen CLI wording: {}",
        err.message
    );
    assert!(err.message.contains("alpha") && err.message.contains("beta"));
}

// --- 穩定錯誤碼註冊表：碼字串值域固定 ---

#[test]
fn error_code_registry_strings_are_stable() {
    assert_eq!(ErrorCode::InvalidArgv.as_str(), "invalid_argv");
    assert_eq!(ErrorCode::NotFound.as_str(), "not_found");
    assert_eq!(ErrorCode::InvalidConfig.as_str(), "invalid_config");
    assert_eq!(ErrorCode::Refused.as_str(), "refused");
    assert_eq!(ErrorCode::Error.as_str(), "error");
}

// === 變更型動詞的領域事件（spec: 變更型動詞的領域事件） ===

/// Ghost workspace: nonexistent root — git probes fail soft, no snapshot
/// or touched-record files are written (same pattern as archive/discard tests).
fn ghost_ws() -> Workspace {
    Workspace {
        root: std::env::temp_dir().join("speclink-command-test-ghost-root"),
        spec_dir_name: "openspec".to_string(),
    }
}

/// Execute expecting success; returns (outcome, events).
fn ok(
    store: &TestStore,
    cmd: Command,
) -> (CommandOutcome, Vec<DomainEvent>) {
    let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
    execute(store, &ctx, cmd).expect("command succeeds")
}

fn kinds(events: &[DomainEvent]) -> Vec<&'static str> {
    events.iter().map(|e| e.kind()).collect()
}

#[test]
fn discuss_search_is_a_query_that_returns_hits_and_no_events() {
    // Spec「動詞覆蓋與跨入口一致性」：discuss search 為唯讀查詢動詞，不發領域事件。
    let store = TestStore::with_live_discussion(
        "drawer-scope",
        "---\ntopic: Drawer scope\nslug: drawer-scope\nstatus: open\ncreated: 2026-07-01\n---\n\n\
             ## Rounds\n\n### Round 1 — interview (2026-07-01)\n\n**Ruled out**: drawer flag\n\n## Conclusion\n",
    );
    let (outcome, events) =
        ok(&store, Command::DiscussSearch { terms: vec!["drawer".to_string()] });
    let hits: Vec<crate::discuss::DiscussionHit> =
        outcome.try_into().expect("search outcome carries hits");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].info.slug, "drawer-scope");
    assert_eq!(hits[0].matches.len(), 3, "topic, slug and the ruled-out line");
    assert!(events.is_empty(), "search is a query and never produces events");

    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::DiscussSearch { terms: vec![" ".to_string()] },
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidArgv, "blank keywords are refused: {err}");
}

#[test]
fn new_change_from_discussion_fails_before_the_change_lands_when_the_record_cannot_take_the_link() {
    // review Round 2：與 discuss::promote 同形——記錄無處插 promoted_to 時，change 目錄不得先落地。
    let unclosed = "---\ntopic: x\nslug: x\nstatus: concluded\n";
    let store = TestStore::with_live_discussion("x", unclosed);
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::NewChange {
            name: "cut-x".to_string(),
            description: None,
            schema: None,
            agent: None,
            from_discussion: Some("x".to_string()),
            last: false,
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("not closed"), "err: {err}");
    assert!(!store.change_exists("cut-x"), "沒有半成品 change");
    assert_eq!(store.discussion("x"), unclosed);
}

#[test]
fn new_change_reports_exactly_one_change_created_event() {
    // Spec scenario 建立變更回報 change-created.
    let store = TestStore::default();
    let (_, events) = ok(
        &store,
        Command::NewChange {
            name: "add-auth".to_string(),
            description: None,
            schema: None,
            agent: None,
            from_discussion: None,
            last: false,
        },
    );
    assert_eq!(events.len(), 1, "exactly one event");
    match &events[0] {
        DomainEvent::ChangeCreated { change, occurred_at } => {
            assert_eq!(change, "add-auth", "subject is the change name");
            assert!(
                *occurred_at <= chrono::Utc::now(),
                "occurredAt is a UTC timestamp of the execution"
            );
        }
        other => panic!("expected change-created, got {other:?}"),
    }
}

#[test]
fn failed_new_change_produces_no_events_and_frozen_message() {
    // Spec scenario 失敗的命令不產生事件 (the Err arm carries no events by type).
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::NewChange {
            name: "demo".to_string(),
            description: None,
            schema: None,
            agent: None,
            from_discussion: None,
            last: false,
        },
    )
    .expect_err("duplicate name must fail");
    assert_eq!(err.code, ErrorCode::Error);
    assert_eq!(err.message, "Change 'demo' already exists.");
}

#[test]
fn new_artifact_reports_artifact_created() {
    let store = TestStore::with_meta("demo", META);
    let (_, events) = ok(
        &store,
        Command::NewArtifact {
            kind: "proposal".to_string(),
            capability: None,
            change: Some("demo".to_string()),
            content: Some("## Why\n\nDemo.\n".to_string()),
            force: false,
            new_capability: false,
        },
    );
    assert_eq!(kinds(&events), ["artifact-created"]);
    match &events[0] {
        DomainEvent::ArtifactCreated { change, artifact, .. } => {
            assert_eq!(change, "demo");
            assert_eq!(artifact, "proposal");
        }
        other => panic!("expected artifact-created, got {other:?}"),
    }
}

#[test]
fn task_done_reports_task_completed() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 Do the thing\n");
    let (_, events) = ok(
        &store,
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files: None,
            head_commit: None,
        },
    );
    assert_eq!(kinds(&events), ["task-completed"]);
    match &events[0] {
        DomainEvent::TaskCompleted { change, task_id, .. } => {
            assert_eq!(change, "demo");
            assert!(task_id.starts_with("tsk_"), "event carries the stamped stable id: {task_id}");
        }
        other => panic!("expected task-completed, got {other:?}"),
    }
}

#[test]
fn already_done_task_produces_no_event() {
    // No state changed → no event; the outcome carries the `already` fact
    // for the entry point's own presentation (CLI error, GUI idempotence).
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 Done already\n");
    let (outcome, events) = ok(
        &store,
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files: None,
            head_commit: None,
        },
    );
    assert!(events.is_empty(), "no event when nothing changed");
    match outcome {
        CommandOutcome::TaskDone(o) => assert!(o.already),
        other => panic!("expected a task-done outcome, got {other:?}"),
    }
}

#[test]
fn task_undone_reports_task_uncompleted() {
    // 覆蓋表: task undone → task-uncompleted.
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 Do the thing\n");
    let (_, events) = ok(
        &store,
        Command::TaskUndone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
        },
    );
    assert_eq!(kinds(&events), ["task-uncompleted"]);
}

// --- 蓋章時機（spec task-identity: 產出全檔蓋章、task done 單行補章）---

#[test]
fn new_artifact_tasks_stamps_every_task_line() {
    let store = TestStore::with_meta("demo", META);
    ok(
        &store,
        Command::NewArtifact {
            kind: "tasks".to_string(),
            capability: None,
            change: Some("demo".to_string()),
            content: Some(
                "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [x] 1.3 third\n"
                    .to_string(),
            ),
            force: false,
            new_capability: false,
        },
    );
    let written = store.read_artifact("demo", "tasks.md").unwrap();
    let tasks = crate::tasks::parse(&written);
    assert_eq!(tasks.len(), 3);
    assert!(tasks.iter().all(|t| t.stable_id.is_some()), "every task carries an id: {written}");
    let mut ids: Vec<String> = tasks.iter().filter_map(|t| t.stable_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "ids must be unique");
    assert_eq!(tasks[0].description, "1.1 first");
    assert_eq!(tasks[1].description, "1.2 second");
    assert_eq!(tasks[2].description, "1.3 third");
}

#[test]
fn task_done_on_unstamped_file_touches_only_the_target_line() {
    const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [ ] 1.3 third\n";
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", TASKS);
    ok(&store, Command::TaskDone { task_id: "3".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None });
    let written = store.read_artifact("demo", "tasks.md").unwrap();
    let orig: Vec<&str> = TASKS.lines().collect();
    let new: Vec<&str> = written.lines().collect();
    assert_eq!(orig.len(), new.len());
    for (i, (o, n)) in orig.iter().zip(&new).enumerate() {
        if i == 4 {
            assert!(
                n.starts_with("- [x] 1.3 third <!-- speclink-task:tsk_"),
                "target must be checked and stamped: {n}"
            );
        } else {
            assert_eq!(o, n, "line {i} must stay byte-identical");
        }
    }
    let stamped_id =
        crate::tasks::parse(&written)[2].stable_id.clone().expect("target gained an id");
    assert!(stamped_id.starts_with("tsk_"));
}

#[test]
fn task_undone_never_stamps() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 first\n");
    ok(&store, Command::TaskUndone { task_id: "1".to_string(), change: Some("demo".to_string()) });
    assert_eq!(
        store.read_artifact("demo", "tasks.md").unwrap(),
        "- [ ] 1.1 first\n",
        "undone flips without stamping"
    );
}

// --- 雙值域定址、重複拒絕與事件載荷（spec task-identity）---

const TID_A: &str = "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV";
const TID_B: &str = "tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ";

#[test]
fn stable_id_addressing_hits_the_original_task_after_reorder() {
    let store = TestStore::with_meta("demo", META);
    // 「1.2 beta」原為第 2 任務、帶 ID；重排後移到末位，第 2 位改為 gamma。
    store.put_artifact(
        "demo",
        "tasks.md",
        &format!("- [ ] 1.1 alpha\n- [ ] 1.3 gamma\n- [ ] 1.2 beta <!-- speclink-task:{TID_B} -->\n"),
    );
    let (outcome, _) = ok(
        &store,
        Command::TaskDone { task_id: TID_B.to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    );
    match outcome {
        CommandOutcome::TaskDone(o) => {
            assert_eq!(o.description, "1.2 beta", "stable id must hit the original task");
            assert!(!o.already);
        }
        other => panic!("expected a task-done outcome, got {other:?}"),
    }
    let (outcome, _) = ok(
        &store,
        Command::TaskDone { task_id: "2".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    );
    match outcome {
        CommandOutcome::TaskDone(o) => {
            assert_eq!(o.description, "1.3 gamma", "ordinal must hit the task now in slot 2");
        }
        other => panic!("expected a task-done outcome, got {other:?}"),
    }
}

#[test]
fn unknown_stable_id_errors_symmetric_to_out_of_range() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n- [x] 1.2 b\n");
    let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
    let ordinal_err = execute(
        &store,
        &ctx,
        Command::TaskDone { task_id: "9".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    )
    .expect_err("out-of-range ordinal must fail");
    assert_eq!(ordinal_err.message, "Task 9 not found (total: 2)");
    let id_err = execute(
        &store,
        &ctx,
        Command::TaskDone {
            task_id: "tsk_01NOPE0000000000000000000ial".to_string(),
            change: Some("demo".to_string()),
            touched_files: None,
            head_commit: None,
        },
    )
    .expect_err("unknown stable id must fail");
    assert_eq!(
        id_err.message,
        "Task tsk_01NOPE0000000000000000000ial not found (total: 2)",
        "unknown id error is symmetric to out-of-range"
    );
    assert_eq!(id_err.code, ordinal_err.code, "same error shape as out-of-range");
    let undone_err = execute(
        &store,
        &ctx,
        Command::TaskUndone {
            task_id: "tsk_01NOPE0000000000000000000ial".to_string(),
            change: Some("demo".to_string()),
        },
    )
    .expect_err("unknown stable id must fail for undone too");
    assert_eq!(undone_err.message, "Task tsk_01NOPE0000000000000000000ial not found (total: 2)");
    assert_eq!(*store.artifact_writes.borrow(), 0, "failed addressing writes nothing");
}

#[test]
fn duplicate_stable_ids_refuse_task_verbs_naming_the_value() {
    let dup_md = format!(
        "- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n- [x] 1.2 b <!-- speclink-task:{TID_A} -->\n"
    );
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", &dup_md);
    let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
    let done_err = execute(
        &store,
        &ctx,
        Command::TaskDone { task_id: TID_A.to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    )
    .expect_err("duplicate ids must refuse task done");
    assert!(
        done_err.message.contains(TID_A),
        "error must name the duplicate value: {}",
        done_err.message
    );
    let undone_err = execute(
        &store,
        &ctx,
        Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
    )
    .expect_err("duplicate ids must refuse task undone regardless of addressing");
    assert!(undone_err.message.contains(TID_A));
    assert_eq!(
        store.read_artifact("demo", "tasks.md").unwrap(),
        dup_md,
        "refusal leaves tasks.md byte-identical"
    );
    assert_eq!(*store.artifact_writes.borrow(), 0, "refusal writes nothing");
}

#[test]
fn task_completed_event_carries_the_stable_id() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
    let (_, events) = ok(
        &store,
        Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    );
    match &events[0] {
        DomainEvent::TaskCompleted { task_id, .. } => assert_eq!(task_id, TID_A),
        other => panic!("expected task-completed, got {other:?}"),
    }
}

#[test]
fn task_done_carries_host_injected_touched_files_into_the_event() {
    // spec verify-evidence「遠端 task done 攜檔案後 evidence 可查」的事件面：
    // Host 在邊界解析好的候選一路走到 task-completed payload，且同一份清單
    // 落進 evidence 記錄——事件是流、記錄是狀態，兩條都得有。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
    let (_, events) = ok(
        &store,
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files: Some(vec!["src/wire.rs".to_string()]),
            head_commit: None,
        },
    );
    match &events[0] {
        DomainEvent::TaskCompleted { touched_files, .. } => {
            assert_eq!(touched_files, &vec!["src/wire.rs".to_string()]);
        }
        other => panic!("expected task-completed, got {other:?}"),
    }
    let rec = crate::tasks::TouchedRecord::load(&store, "demo");
    assert_eq!(rec.all_files(), vec!["src/wire.rs".to_string()]);
}

#[test]
fn task_done_without_injected_files_carries_no_touched_files() {
    // 無候選不偽造：payload 未攜帶時事件的 touchedFiles 為空，而非補一份
    // 猜來的清單。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
    let (_, events) = ok(
        &store,
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files: None,
            head_commit: None,
        },
    );
    match &events[0] {
        DomainEvent::TaskCompleted { touched_files, .. } => {
            assert!(touched_files.is_empty(), "nothing to attribute, nothing recorded");
        }
        other => panic!("expected task-completed, got {other:?}"),
    }
    assert_eq!(store.read_evidence("demo"), None);
}

#[test]
fn task_completed_event_on_unstamped_task_carries_the_fresh_id() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
    let (_, events) = ok(
        &store,
        Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
    );
    let written = store.read_artifact("demo", "tasks.md").unwrap();
    let fresh = crate::tasks::parse(&written)[0]
        .stable_id
        .clone()
        .expect("done stamps the target line");
    match &events[0] {
        DomainEvent::TaskCompleted { task_id, .. } => {
            assert_eq!(task_id, &fresh, "event carries the id stamped by this very write");
        }
        other => panic!("expected task-completed, got {other:?}"),
    }
}

#[test]
fn task_uncompleted_event_id_is_stable_id_or_ordinal_string() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "tasks.md",
        &format!("- [x] 1.1 a <!-- speclink-task:{TID_A} -->\n- [x] 1.2 b\n"),
    );
    let (_, events) = ok(
        &store,
        Command::TaskUndone { task_id: "1".to_string(), change: Some("demo".to_string()) },
    );
    match &events[0] {
        DomainEvent::TaskUncompleted { task_id, .. } => assert_eq!(task_id, TID_A),
        other => panic!("expected task-uncompleted, got {other:?}"),
    }
    let (_, events) = ok(
        &store,
        Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
    );
    match &events[0] {
        DomainEvent::TaskUncompleted { task_id, .. } => {
            assert_eq!(task_id, "2", "undone on an unstamped task falls back to the ordinal string");
        }
        other => panic!("expected task-uncompleted, got {other:?}"),
    }
}

#[test]
fn claim_on_the_fs_store_is_an_error_without_events() {
    // claim is remote-store-only; the fs path refuses with the frozen text.
    // Its event mapping (change-claimed) is asserted in the kind table below.
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::Claim { name: "demo".to_string() },
    )
    .expect_err("claim must refuse on a plain store");
    assert_eq!(err.code, ErrorCode::Error);
    assert_eq!(
        err.message,
        "claim requires a remote store — this project uses the local fs store"
    );
}

// --- claim:團隊模式 store 的認領語意(change-lifecycle「認領標記欄位」)---

/// 認領測試共用的既有 meta:含建立欄位與 board_rank,用來釘「既有欄位
/// 逐字元保留」。
const CLAIM_META: &str =
    "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\nboard_rank: n\n";

fn claim_ctx(actor: &str) -> ExecutionContext {
    ExecutionContext {
        actor: Some(actor.to_string()),
        workspace: Some(ghost_ws()),
        ..Default::default()
    }
}

fn claim(store: &TestStore, actor: &str) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
    execute(store, &claim_ctx(actor), Command::Claim { name: "demo".to_string() })
}

#[test]
fn first_claim_on_a_team_store_stamps_the_owner_and_reports_change_claimed() {
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    let (outcome, events) = claim(&store, "Alice <a@example.com>").expect("first claim succeeds");

    match &outcome {
        CommandOutcome::Claim(o) => {
            assert_eq!(o.name, "demo");
            assert_eq!(o.claimed_by.as_deref(), Some("Alice <a@example.com>"));
            assert!(o.claimed, "the first claim is the one that stamps");
        }
        other => panic!("expected a claim outcome, got {other:?}"),
    }
    assert_eq!(kinds(&events), ["change-claimed"]);

    let meta = store.meta("demo");
    assert!(
        meta.starts_with(CLAIM_META),
        "existing meta fields survive byte-for-byte: {meta}"
    );
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).expect("stamped meta parses");
    assert_eq!(parsed.claimed_by.as_deref(), Some("Alice <a@example.com>"));
    assert_eq!(parsed.claimed_at.as_deref(), Some(crate::util::today().as_str()));
    assert_eq!(*store.meta_writes.borrow(), 1, "exactly one meta write");
}

#[test]
fn repeat_claim_by_the_same_actor_is_idempotent_with_no_write_and_no_event() {
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    claim(&store, "Alice <a@example.com>").expect("first claim succeeds");
    let first_stamp = store.meta("demo");

    let (outcome, events) = claim(&store, "Alice <a@example.com>").expect("repeat claim succeeds");
    match &outcome {
        CommandOutcome::Claim(o) => {
            assert_eq!(o.claimed_by.as_deref(), Some("Alice <a@example.com>"));
            assert!(!o.claimed, "the repeat claim reports no new stamp");
        }
        other => panic!("expected a claim outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "an idempotent pass states no mutation");
    assert_eq!(store.meta("demo"), first_stamp, "the first stamp survives verbatim");
    assert_eq!(*store.meta_writes.borrow(), 1, "no second meta write");
}

#[test]
fn repeat_claim_by_an_actor_whose_identity_needs_quoting_is_still_idempotent() {
    // `yaml_scalar` 把含 `:` 的身分包上引號寫入；讀回比對必須解引號，否則本人會被
    // 當成他人而遭拒。拒絕訊息指名的持有人也是解引號後的原字串。
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    let actor = "Alice: dev <a@example.com>";
    claim(&store, actor).expect("first claim succeeds");
    assert!(
        store.meta("demo").contains("claimed_by: \"Alice: dev <a@example.com>\"\n"),
        "the identity is written quoted: {}",
        store.meta("demo")
    );
    let (outcome, events) = claim(&store, actor).expect("repeat claim by the holder succeeds");
    match &outcome {
        CommandOutcome::Claim(o) => {
            assert_eq!(o.claimed_by.as_deref(), Some(actor));
            assert!(!o.claimed, "the repeat claim reports no new stamp");
        }
        other => panic!("expected a claim outcome, got {other:?}"),
    }
    assert!(events.is_empty());
    assert_eq!(*store.meta_writes.borrow(), 1, "no second meta write");
    let err = claim(&store, "Bob <b@example.com>").expect_err("a held change refuses another claimant");
    assert!(
        err.message.contains("already claimed by Alice: dev <a@example.com> —"),
        "the refusal names the unquoted holder: {}",
        err.message
    );
}

#[test]
fn claim_by_another_actor_is_refused_naming_the_holder_and_writes_nothing() {
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    claim(&store, "Alice <a@example.com>").expect("first claim succeeds");
    let held = store.meta("demo");

    let err = claim(&store, "Bob <b@example.com>").expect_err("a held change refuses another claimant");
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(
        err.message.contains("Alice <a@example.com>"),
        "the refusal must name the current holder: {}",
        err.message
    );
    assert_eq!(store.meta("demo"), held, "the holder's stamp is untouched");
    assert_eq!(*store.meta_writes.borrow(), 1, "the refused claim writes nothing");
}

/// 只剩 claimed_at 的 meta:目前沒有 release 動詞,人工「釋出」最直覺
/// 就是手刪 claimed_by 那一行。
const HALF_CLAIMED_META: &str =
    "schema: spec-driven\ncreated: 2026-07-01\nclaimed_at: 2026-07-02\n";

#[test]
fn claim_on_a_half_stamped_meta_is_refused_instead_of_duplicating_the_key() {
    // 認領章的兩個欄位一起寫,也必須一起判。半章 meta 若再追加一次,
    // claimed_at 會成為重複鍵而讓這個 change 永久無法解析——寧可拒絕。
    let store = TestStore::team_with_meta("demo", HALF_CLAIMED_META);
    let err = claim(&store, "Alice <a@example.com>").expect_err("a half stamp refuses");
    assert_eq!(err.code, ErrorCode::Refused);
    assert_eq!(store.meta("demo"), HALF_CLAIMED_META, "meta byte-identical");
    assert_eq!(*store.meta_writes.borrow(), 0, "no write on a refusal");
    crate::model::ChangeMeta::from_text(Some(&store.meta("demo")))
        .expect("the refused meta stays parseable");
}

#[test]
fn claim_on_a_team_store_with_corrupt_meta_fails_closed_without_writing() {
    let store = TestStore::team_with_meta("demo", BAD_META);
    let err = claim(&store, "Alice <a@example.com>").expect_err("corrupt meta refuses");
    assert_eq!(err.code, ErrorCode::InvalidConfig);
    assert!(
        err.message.contains("openspec/changes/demo/.openspec.yaml"),
        "the error must name the metadata file: {}",
        err.message
    );
    assert_eq!(store.meta("demo"), BAD_META, "meta byte-identical");
    assert_eq!(*store.meta_writes.borrow(), 0, "no write on a fail-closed refusal");
}

#[test]
fn claim_of_an_unknown_change_on_a_team_store_is_not_found() {
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    let err = execute(
        &store,
        &claim_ctx("Alice <a@example.com>"),
        Command::Claim { name: "ghost".to_string() },
    )
    .expect_err("an unknown change cannot be claimed");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn claim_without_an_actor_is_refused_rather_than_stamping_an_anonymous_owner() {
    // 認領是「誰在做」的宣告——無身分可歸屬時寧可拒絕,也不留一筆
    // 沒有持有人的認領章(那會讓別人永遠撞衝突卻不知該找誰)。
    let store = TestStore::team_with_meta("demo", CLAIM_META);
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::Claim { name: "demo".to_string() },
    )
    .expect_err("an anonymous caller cannot claim");
    assert_eq!(err.code, ErrorCode::Refused);
    assert_eq!(store.meta("demo"), CLAIM_META);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn in_progress_add_reports_change_marked_in_progress() {
    let store = TestStore::with_meta("demo", META);
    let (_, events) = ok(&store, Command::InProgressAdd { name: "demo".to_string() });
    assert_eq!(kinds(&events), ["change-marked-in-progress"]);
}

// --- in-progress remove:退回動詞的 outcome、事件與守門錯誤分類 ---

/// 帶開工戳記、無任何工作痕跡的 meta。
const STARTED_META: &str = "schema: spec-driven\nstarted_at: 2026-07-10\nstarted_by: T <t@example.com>\n";

#[test]
fn in_progress_remove_reports_change_in_progress_removed() {
    let store = TestStore::with_meta("demo", STARTED_META);
    let (outcome, events) = ok(&store, Command::InProgressRemove { name: "demo".to_string() });
    match outcome {
        CommandOutcome::InProgressRemove(o) => {
            assert_eq!(o.name, "demo");
            assert!(o.removed, "the marker must actually be removed");
        }
        other => panic!("expected an in-progress-remove outcome, got {other:?}"),
    }
    assert_eq!(kinds(&events), ["change-in-progress-removed"]);
    assert!(!store.meta("demo").contains("started_"), "started_* lines stripped");
}

#[test]
fn in_progress_remove_idempotent_pass_reports_no_event() {
    let store = TestStore::with_meta("demo", META);
    let (outcome, events) = ok(&store, Command::InProgressRemove { name: "demo".to_string() });
    match outcome {
        CommandOutcome::InProgressRemove(o) => {
            assert!(!o.removed, "nothing to remove must report removed: false");
        }
        other => panic!("expected an in-progress-remove outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "an idempotent pass mutates nothing — no event");
}

#[test]
fn in_progress_remove_gate_refusal_is_refused_with_evidence_on_the_error() {
    let store = TestStore::with_meta("demo", STARTED_META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 a\n- [x] 1.2 b\n- [ ] 1.3 c\n");
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::InProgressRemove { name: "demo".to_string() },
    )
    .expect_err("work traces must refuse the removal");
    assert_eq!(err.code, ErrorCode::Refused);
    let evidence = err
        .source
        .as_ref()
        .expect("the refusal keeps the flow error as source")
        .downcast_ref::<crate::inprogress::RevertBlocked>()
        .expect("structured evidence rides the error");
    assert_eq!(evidence.checked_tasks, 2);
    assert!(evidence.touched_files.is_empty());
    assert!(store.meta("demo").contains("started_at:"), "refusal must not strip the marker");
}

#[test]
fn in_progress_remove_unknown_change_is_not_found() {
    // 與 add 的 parity 靜默刻意不對稱:修正動作打錯名字必須明確報錯。
    let store = TestStore::default();
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::InProgressRemove { name: "ghost".to_string() },
    )
    .expect_err("unknown change must error");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert!(err.message.contains("ghost"), "error names the change: {}", err.message);
}

// --- change depends：宣告依賴的 outcome、事件與守門分類（change-plan design D4/D6）---

#[test]
fn change_depends_reports_change_depends_changed_with_the_full_list() {
    let store = TestStore::with_meta("demo", META);
    store.metas.borrow_mut().insert("a".to_string(), META.to_string());
    let (outcome, events) = ok(
        &store,
        Command::ChangeDepends { name: "demo".to_string(), on: vec!["a".to_string()], remove: false },
    );
    match outcome {
        CommandOutcome::ChangeDepends(o) => {
            assert_eq!(o.change, "demo");
            assert_eq!(o.depends_on, ["a"]);
            assert!(o.changed);
        }
        other => panic!("expected a change-depends outcome, got {other:?}"),
    }
    assert_eq!(kinds(&events), ["change-depends-changed"]);
    match &events[0] {
        DomainEvent::ChangeDependsChanged { change, depends_on, .. } => {
            assert_eq!(change, "demo");
            assert_eq!(depends_on, &["a".to_string()]);
        }
        other => panic!("unexpected event {other:?}"),
    }
    assert_eq!(store.meta("demo"), format!("{META}depends_on: a\n"));
}

#[test]
fn change_depends_idempotent_pass_reports_no_event() {
    let store = TestStore::with_meta("demo", &format!("{META}depends_on: a\n"));
    store.metas.borrow_mut().insert("a".to_string(), META.to_string());
    let (outcome, events) = ok(
        &store,
        Command::ChangeDepends { name: "demo".to_string(), on: vec!["a".to_string()], remove: false },
    );
    match outcome {
        CommandOutcome::ChangeDepends(o) => assert!(!o.changed, "nothing changed"),
        other => panic!("expected a change-depends outcome, got {other:?}"),
    }
    assert!(events.is_empty(), "an idempotent pass mutates nothing — no event");
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn change_depends_guards_are_refused_and_unknown_target_is_not_found() {
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::ChangeDepends { name: "demo".to_string(), on: vec!["ghost".to_string()], remove: false },
    )
    .expect_err("an unknown prerequisite must refuse");
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(err.message.contains("ghost"), "{}", err.message);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::ChangeDepends { name: "ghost".to_string(), on: vec!["demo".to_string()], remove: false },
    )
    .expect_err("an unknown target must error");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn archive_reports_change_archived() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    let (_, events) = ok(
        &store,
        Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
    );
    assert_eq!(kinds(&events), ["change-archived"]);
    match &events[0] {
        DomainEvent::ChangeArchived { change, .. } => assert_eq!(change, "demo"),
        other => panic!("expected change-archived, got {other:?}"),
    }
}

#[test]
fn archive_never_consults_evidence_wherever_it_runs() {
    // 討論 evidence-gate-false-blocks：封存不再讀證據判生死,所以有沒有 host
    // workspace 都一樣通過——server bridge／Node SDK 的合成 workspace 曾是
    // 「一律判缺席」的地雷,現在連判的動作都不存在。
    let cmd = Command::Archive {
        change: Some("demo".to_string()),
        skip_specs: false,
        no_validate: false,
        mark_tasks_complete: false,
        carry_review: false,
            carry_verify: false,
    };

    for workspace in [None, Some(ghost_ws())] {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        execute(&store, &ExecutionContext { workspace, ..Default::default() }, cmd.clone())
            .expect("an evidence-less change archives with or without a host workspace");
        assert!(!store.change_exists("demo"), "the change moved into the archive");
    }
}

#[test]
fn archive_of_incomplete_change_is_refused() {
    // spec change-lifecycle「單筆封存的任務完成度守門」：任務未完成的單筆
    // Archive 經 runtime 分類為 refused，change 原地不動、無事件。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
    )
    .expect_err("incomplete change must refuse archive");
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(err.message.contains("1/3"), "evidence rides the message: {}", err.message);
    assert!(store.change_exists("demo"), "nothing moved on refusal");
}

#[test]
fn archive_of_incomplete_change_refuses_before_the_merge_gate() {
    // 守門序只在 archive() 一份（design D2）：CLI 單筆與 server 都經此 Command，
    // 任務未完成＋delta 過期時任務守門先拒，merge 拒絕字串不出現。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
    );
    store.canonical.borrow_mut().insert(
        "auth".to_string(),
        "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n".to_string(),
    );
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
    )
    .expect_err("incomplete change must refuse archive");
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(err.message.contains("0/1 tasks complete"), "task gate first: {}", err.message);
    assert!(!err.message.contains("cannot be archived"), "merge refusal never reached: {}", err.message);
    assert!(store.change_exists("demo"), "nothing moved on refusal");
}

#[test]
fn discard_reports_change_discarded() {
    let store = TestStore::with_meta("demo", META);
    let (_, events) = ok(
        &store,
        Command::Discard { change: "demo".to_string(), force: false },
    );
    assert_eq!(kinds(&events), ["change-discarded"]);
}

#[test]
fn discard_of_started_change_is_refused_without_force() {
    // Spec scenario 需 --force 的拒絕: started work refuses without --force,
    // classified refused, no files deleted, no events.
    let store = TestStore::with_meta("demo", "schema: spec-driven\nstarted_at: 2026-07-10\n");
    let err = execute(
        &store,
        &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
        Command::Discard { change: "demo".to_string(), force: false },
    )
    .expect_err("started change must refuse discard");
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(store.change_exists("demo"), "nothing deleted on refusal");
}

#[test]
fn review_verbs_execute_over_commands_with_events_and_actor() {
    // design D4a：review 動詞家族經 Command 分派（server 承載的動詞契約）——
    // 一條真實生命週期；stamp 取 ctx.actor 落 reviewed_by、scope 由提交端預算；
    // 查無映 NotFound（server 404）。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1 a\n");
    let (outcome, ev) = ok(
        &store,
        Command::ReviewAddRound {
            change: "demo".to_string(),
            content: "**Scope**: src/lib.rs\n".to_string(),
        },
    );
    match outcome {
        CommandOutcome::ReviewAddRound(o) => {
            assert_eq!((o.change.as_str(), o.round), ("demo", 1));
        }
        other => panic!("expected review add-round outcome, got {other:?}"),
    }
    assert_eq!(kinds(&ev), ["review-round-added"]);

    let (outcome, ev) = ok(&store, Command::ReviewShow { change: "demo".to_string() });
    match outcome {
        CommandOutcome::ReviewShow(o) => {
            assert_eq!(o.ticket.last_round().scope, ["src/lib.rs"]);
        }
        other => panic!("expected review show outcome, got {other:?}"),
    }
    assert!(ev.is_empty(), "show is a query and never produces events");

    let ctx = ExecutionContext {
        actor: Some("Rev <r@example.com>".to_string()),
        ..Default::default()
    };
    let hash = crate::station::content_fingerprint("fn lib() {}\n");
    let (_, ev) = execute(
        &store,
        &ctx,
        Command::ReviewStamp {
            change: "demo".to_string(),
            accept: false,
            tool: Some("claude".to_string()),
            scope: vec![crate::model::ReviewedScopeEntry {
                path: "src/lib.rs".to_string(),
                hash,
            }],
            missing: vec![],
        },
    )
    .expect("stamp executes");
    assert_eq!(kinds(&ev), ["review-stamped"]);
    let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo")))
        .expect("meta parses");
    assert_eq!(meta.reviewed_by.as_deref(), Some("Rev <r@example.com>"));
    assert_eq!(meta.reviewed_with.as_deref(), Some("claude"));

    let (_, ev) = ok(
        &store,
        Command::ReviewAddRound {
            change: "demo".to_string(),
            content: "**Scope**: src/lib.rs\n".to_string(),
        },
    );
    assert_eq!(kinds(&ev), ["review-round-added"]);
    let (_, ev) = ok(&store, Command::ReviewDiscard { change: "demo".to_string() });
    assert_eq!(kinds(&ev), ["review-discarded"]);

    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::ReviewShow { change: "demo".to_string() },
    )
    .expect_err("no ticket left after discard");
    assert_eq!(err.code, ErrorCode::NotFound, "missing ticket maps to 404: {}", err.message);
}

#[test]
fn discuss_verbs_report_their_events() {
    // 覆蓋表 discuss 全系列（new/context/add-round/conclude/link/seal/
    // archive）：一條真實生命週期，每步斷言恰一筆對應事件。
    let store = TestStore::default();
    let (_, ev) = ok(
        &store,
        Command::DiscussNew {
            topic: "API auth".to_string(),
            slug: Some("api-auth".to_string()),
            kind: None,
        },
    );
    assert_eq!(kinds(&ev), ["discussion-created"]);
    match &ev[0] {
        DomainEvent::DiscussionCreated { slug, .. } => assert_eq!(slug, "api-auth"),
        other => panic!("expected discussion-created, got {other:?}"),
    }

    let (_, ev) = ok(
        &store,
        Command::DiscussContext {
            slug: "api-auth".to_string(),
            content: "框架：假設訪談模式。\n".to_string(),
        },
    );
    assert_eq!(kinds(&ev), ["discussion-context-set"]);

    let (_, ev) = ok(
        &store,
        Command::DiscussAddRound {
            slug: "api-auth".to_string(),
            mode: "assumptions".to_string(),
            content: "第一輪內容。\n".to_string(),
        },
    );
    assert_eq!(kinds(&ev), ["discussion-round-added"]);

    let (_, ev) = ok(
        &store,
        Command::DiscussConclude {
            slug: "api-auth".to_string(),
            content: "結論：做。\n".to_string(),
            hold: false,
        },
    );
    assert_eq!(kinds(&ev), ["discussion-concluded"]);

    // link + seal need a change to bind to.
    ok(
        &store,
        Command::NewChange {
            name: "auth-change".to_string(),
            description: None,
            schema: None,
            agent: None,
            from_discussion: None,
            last: false,
        },
    );
    let (_, ev) = ok(
        &store,
        Command::DiscussLink {
            slug: "api-auth".to_string(),
            change: "auth-change".to_string(),
        },
    );
    assert_eq!(kinds(&ev), ["discussion-linked"]);

    let (_, ev) = ok(
        &store,
        Command::DiscussSeal {
            slug: "api-auth".to_string(),
            change: "auth-change".to_string(),
            last: false,
        },
    );
    assert_eq!(kinds(&ev), ["discussion-sealed"]);

    let (_, ev) = ok(&store, Command::DiscussArchive { slug: "api-auth".to_string() });
    assert_eq!(kinds(&ev), ["discussion-archived"]);
}

#[test]
fn discuss_archive_ignores_the_hold_flag() {
    // 手動封存是「放棄後續刀」的明示出口：帶 hold 的記錄照常封存。
    let store = TestStore::with_live_discussion(
        "staged",
        "---\nslug: staged\nstatus: promoted\npromoted_to: cut-a\ncreated: 2026-07-10\nhold: true\n---\n\n# Discussion: staged\n\n## Conclusion\n\n**Decision**: cut-b later\n",
    );

    let (_, events) = ok(&store, Command::DiscussArchive { slug: "staged".to_string() });

    assert_eq!(kinds(&events), ["discussion-archived"]);
    assert!(store.archived_discussion_exists("staged"));
    assert!(!store.live_discussion_exists("staged"));
}

#[test]
fn discuss_discard_reports_discussion_discarded() {
    let store = TestStore::with_live_discussion(
        "scrap-idea",
        "---\nslug: scrap-idea\nstatus: open\ncreated: 2026-07-10\n---\n\n# Discussion: scrap\n",
    );
    let (_, events) = ok(
        &store,
        Command::DiscussDiscard { slug: "scrap-idea".to_string(), force: false },
    );
    assert_eq!(kinds(&events), ["discussion-discarded"]);
    assert!(!store.live_discussion_exists("scrap-idea"));
}

#[test]
fn promote_reports_promoted_and_change_created() {
    // Spec scenario 複合動詞回報多筆事件.
    let store = TestStore::default();
    ok(
        &store,
        Command::DiscussNew {
            topic: "API auth".to_string(),
            slug: Some("api-auth".to_string()),
            kind: None,
        },
    );
    ok(
        &store,
        Command::DiscussConclude {
            slug: "api-auth".to_string(),
            content: "結論：轉正。\n".to_string(),
            hold: false,
        },
    );
    let (_, events) = ok(
        &store,
        Command::DiscussPromote { slug: "api-auth".to_string(), name: None, last: false },
    );
    assert_eq!(kinds(&events), ["discussion-promoted", "change-created"]);
    match (&events[0], &events[1]) {
        (
            DomainEvent::DiscussionPromoted { slug, change, .. },
            DomainEvent::ChangeCreated { change: created, .. },
        ) => {
            assert_eq!(slug, "api-auth");
            assert_eq!(change, "api-auth");
            assert_eq!(created, "api-auth");
        }
        other => panic!("expected promoted+created, got {other:?}"),
    }
}

// --- TaskMove 經 gateway（spec「任務搬移端點與重編號效果」；design 決策 4）---

#[test]
fn task_move_rewrites_tasks_and_reports_task_moved_event() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "tasks.md",
        "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n",
    );
    let (outcome, events) = ok(
        &store,
        Command::TaskMove { change: "demo".to_string(), from: 1, to: 3, before: None },
    );
    match outcome {
        CommandOutcome::TaskMove(o) => {
            assert_eq!(o.change, "demo");
            assert_eq!(o.description, "2.2 甲", "outcome carries the post-move description");
        }
        other => panic!("expected a task-move outcome, got {other:?}"),
    }
    assert_eq!(kinds(&events), ["task-moved"]);
    match &events[0] {
        DomainEvent::TaskMoved { change, .. } => assert_eq!(change, "demo"),
        other => panic!("expected task-moved, got {other:?}"),
    }
    let text = store
        .artifacts
        .borrow()
        .get(&("demo".to_string(), "tasks.md".to_string()))
        .unwrap()
        .clone();
    assert!(text.contains("- [ ] 2.2 甲"), "moved line renumbered into group 2: {text}");
}

#[test]
fn task_move_out_of_range_refuses_without_writes_or_events() {
    const TASKS: &str = "- [ ] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n";
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", TASKS);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::TaskMove { change: "demo".to_string(), from: 5, to: 1, before: None },
    )
    .expect_err("out-of-range move must refuse");
    assert!(err.message.contains("out of range"), "must name the refusal: {}", err.message);
    assert_eq!(
        store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
        TASKS,
        "tasks.md byte-identical"
    );
    assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
}

#[test]
fn task_move_missing_tasks_md_is_not_found() {
    let store = TestStore::with_meta("demo", META);
    let err = execute(
        &store,
        &ExecutionContext::default(),
        Command::TaskMove { change: "demo".to_string(), from: 1, to: 2, before: None },
    )
    .expect_err("move without tasks.md must refuse");
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(err.message, "tasks.md not found for change 'demo'");
}

#[test]
fn event_kind_table_matches_the_spec_coverage_table() {
    // spec Example 變更型動詞與事件種類對應——18 列逐一斷言（含 execute 中
    // 無成功路徑的 change-claimed：對應表本身是契約）。
    let at = chrono::Utc::now();
    let s = |v: &str| v.to_string();
    let table: Vec<(DomainEvent, &str)> = vec![
        (DomainEvent::ChangeCreated { change: s("c"), occurred_at: at }, "change-created"),
        (
            DomainEvent::ArtifactCreated { change: s("c"), artifact: s("proposal"), occurred_at: at },
            "artifact-created",
        ),
        (
            DomainEvent::TaskCompleted {
                change: s("c"),
                task_id: s("tsk_1"),
                touched_files: Vec::new(),
                occurred_at: at,
            },
            "task-completed",
        ),
        (
            DomainEvent::TaskUncompleted { change: s("c"), task_id: s("tsk_1"), occurred_at: at },
            "task-uncompleted",
        ),
        (DomainEvent::TaskMoved { change: s("c"), occurred_at: at }, "task-moved"),
        (DomainEvent::ChangeClaimed { change: s("c"), occurred_at: at }, "change-claimed"),
        (
            DomainEvent::ChangeMarkedInProgress { change: s("c"), occurred_at: at },
            "change-marked-in-progress",
        ),
        (
            DomainEvent::ChangeDependsChanged { change: s("c"), depends_on: vec![s("a")], occurred_at: at },
            "change-depends-changed",
        ),
        (
            DomainEvent::ChangeArchived { change: s("c"), dated_name: s("2026-07-12-c"), occurred_at: at },
            "change-archived",
        ),
        (DomainEvent::ChangeDiscarded { change: s("c"), occurred_at: at }, "change-discarded"),
        (DomainEvent::DiscussionCreated { slug: s("d"), occurred_at: at }, "discussion-created"),
        (
            DomainEvent::DiscussionContextSet { slug: s("d"), occurred_at: at },
            "discussion-context-set",
        ),
        (
            DomainEvent::DiscussionRoundAdded { slug: s("d"), round: 1, occurred_at: at },
            "discussion-round-added",
        ),
        (
            DomainEvent::DiscussionConcluded { slug: s("d"), occurred_at: at },
            "discussion-concluded",
        ),
        (
            DomainEvent::DiscussionPromoted { slug: s("d"), change: s("c"), occurred_at: at },
            "discussion-promoted",
        ),
        (
            DomainEvent::DiscussionLinked { slug: s("d"), change: s("c"), occurred_at: at },
            "discussion-linked",
        ),
        (
            DomainEvent::DiscussionSealed { slug: s("d"), change: s("c"), occurred_at: at },
            "discussion-sealed",
        ),
        (DomainEvent::DiscussionArchived { slug: s("d"), occurred_at: at }, "discussion-archived"),
        (
            DomainEvent::DiscussionDiscarded { slug: s("d"), occurred_at: at },
            "discussion-discarded",
        ),
        (
            DomainEvent::ReviewRoundAdded { change: s("c"), round: 1, occurred_at: at },
            "review-round-added",
        ),
        (DomainEvent::ReviewStamped { change: s("c"), occurred_at: at }, "review-stamped"),
        (DomainEvent::ReviewDiscarded { change: s("c"), occurred_at: at }, "review-discarded"),
        (
            DomainEvent::VerifyRoundAdded { change: s("c"), round: 1, occurred_at: at },
            "verify-round-added",
        ),
        (DomainEvent::VerifyStamped { change: s("c"), occurred_at: at }, "verify-stamped"),
        (DomainEvent::VerifyDiscarded { change: s("c"), occurred_at: at }, "verify-discarded"),
    ];
    assert_eq!(table.len(), 25, "the coverage table has 25 mutating verbs");
    for (event, kind) in &table {
        assert_eq!(event.kind(), *kind, "for {event:?}");
    }
}

// === ExecutionContext 由 Host 解析且不可覆寫（spec: command 無從攜帶 identity／本地 actor 語意不變） ===

#[test]
fn command_inputs_carry_no_actor_or_policy_fields() {
    // Command 封閉 enum 的完整欄位解構：任何 variant 若新增 actor 或
    // policy 欄位，這裡缺欄位的解構就編譯失敗——蓋章身分與政策只能
    // 來自 ExecutionContext，呼叫端與模型無從經 command 參數覆寫。
    let probe = list_cmd();
    match probe {
        Command::List { sort: _, specs: _, changes: _, worktrees: _ } => {}
        Command::Show { item: _, item_type: _ } => {}
        Command::Status { change: _, schema: _ } => {}
        Command::Instructions { artifact: _, change: _, schema: _ } => {}
        Command::Validate { item: _, all: _, changes: _, specs: _, strict: _ } => {}
        Command::Analyze { change: _ } => {}
        Command::Trace { capability: _ } => {}
        Command::Plan { ranks: _ } => {}
        Command::PlanStrict => {}
        Command::ArtifactCat { artifact: _, change: _ } => {}
        Command::LanguageShow => {}
        Command::DiscussList { archived: _ } => {}
        Command::DiscussShow { slug: _ } => {}
        Command::DiscussSearch { terms: _ } => {}
        Command::NewChange {
            name: _,
            description: _,
            schema: _,
            agent: _,
            from_discussion: _,
            last: _,
        } => {}
        Command::NewArtifact { kind: _, capability: _, change: _, content: _, force: _, new_capability: _ } => {}
        Command::TaskDone { task_id: _, change: _, touched_files: _, head_commit: _ } => {}
        Command::TaskUndone { task_id: _, change: _ } => {}
        Command::TaskMove { change: _, from: _, to: _, before: _ } => {}
        Command::Claim { name: _ } => {}
        Command::InProgressAdd { name: _ } => {}
        Command::InProgressRemove { name: _ } => {}
        Command::ChangeDepends { name: _, on: _, remove: _ } => {}
        Command::Archive { change: _, skip_specs: _, no_validate: _, mark_tasks_complete: _, carry_review: _, carry_verify: _ } => {}
        Command::Discard { change: _, force: _ } => {}
        Command::DiscussNew { topic: _, slug: _, kind: _ } => {}
        Command::DiscussContext { slug: _, content: _ } => {}
        Command::DiscussAddRound { slug: _, mode: _, content: _ } => {}
        Command::DiscussConclude { slug: _, content: _, hold: _ } => {}
        Command::DiscussPromote { slug: _, name: _, last: _ } => {}
        Command::DiscussLink { slug: _, change: _ } => {}
        Command::DiscussSeal { slug: _, change: _, last: _ } => {}
        Command::DiscussArchive { slug: _ } => {}
        Command::DiscussDiscard { slug: _, force: _ } => {}
        Command::ReviewAddRound { change: _, content: _ } => {}
        Command::ReviewShow { change: _ } => {}
        // `tool` 是工具名（CLI `--agent`，同 NewChange 的 agent），非身分；
        // 蓋章者身分仍只來自 ExecutionContext.actor。
        Command::ReviewStamp { change: _, accept: _, tool: _, scope: _, missing: _ } => {}
        Command::ReviewDiscard { change: _ } => {}
        Command::VerifyAddRound { change: _, content: _ } => {}
        Command::VerifyShow { change: _ } => {}
        Command::VerifyStamp {
            change: _,
            accept: _,
            tool: _,
            scope: _,
            missing: _,
        } => {}
        Command::VerifyDiscard { change: _ } => {}
    }
}

fn actor_ctx(actor: Option<&str>) -> ExecutionContext {
    ExecutionContext {
        actor: actor.map(str::to_string),
        workspace: Some(ghost_ws()),
        ..Default::default()
    }
}

fn new_change_cmd(name: &str) -> Command {
    Command::NewChange {
        name: name.to_string(),
        description: None,
        schema: None,
        agent: None,
        from_discussion: None,
        last: false,
    }
}

#[test]
fn created_by_stamp_follows_context_actor_only() {
    // new change 的 created_by 章只隨 context actor 改變；無身分時
    // 沿用現行無章行為。
    let store = TestStore::default();
    execute(
        &store,
        &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
        new_change_cmd("stamped"),
    )
    .expect("new change succeeds");
    assert!(
        store.meta("stamped").contains("created_by: Ctx Actor <ctx@example.com>\n"),
        "created_by follows the context actor, meta: {}",
        store.meta("stamped")
    );

    execute(&store, &actor_ctx(None), new_change_cmd("anon")).expect("new change succeeds");
    assert!(
        !store.meta("anon").contains("created_by:"),
        "anonymous context keeps the current no-stamp behavior, meta: {}",
        store.meta("anon")
    );
}

#[test]
fn started_by_stamp_follows_context_actor_only() {
    let store = TestStore::with_meta("demo", META);
    execute(
        &store,
        &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
        Command::InProgressAdd { name: "demo".to_string() },
    )
    .expect("in-progress add succeeds");
    assert!(
        store.meta("demo").contains("started_by: Ctx Actor <ctx@example.com>\n"),
        "started_by follows the context actor, meta: {}",
        store.meta("demo")
    );
}

#[test]
fn discussion_created_by_follows_context_actor_only() {
    let store = TestStore::default();
    let (outcome, _) = execute(
        &store,
        &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
        Command::DiscussNew { topic: "Identity probe".to_string(), slug: None, kind: None },
    )
    .expect("discuss new succeeds");
    match outcome {
        CommandOutcome::DiscussNew(info) => assert_eq!(
            info.created_by.as_deref(),
            Some("Ctx Actor <ctx@example.com>"),
            "the discussion creator stamp follows the context actor"
        ),
        other => panic!("expected a discuss-new outcome, got {other:?}"),
    }
}
