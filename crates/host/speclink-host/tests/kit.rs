//! The host kit (spec host-kit): one kit for every user, an identity and a
//! scope on every call, a hook that can refuse before anything is written and
//! a hook that only watches after the commit. Every scenario of the six
//! requirements runs here on the in-memory store port, wrapped in the shared
//! recorder that keeps the calls the store receives.

use speclink_core::command::{Command, CommandOutcome, ErrorCode};
use speclink_core::config::WorkflowConfigEdit;
use speclink_core::model::ChangeMeta;
use speclink_host::kit::{AfterCommit, BeforeCommand, Caller, Decision, Hooks, HostKit, Identity};
use speclink_host::port_exec::{self, ExecError};
use speclink_store_port::{DocKey, MemoryStorePort, PortError, ScopeId, StorePort, Version};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

mod common;

const META: &str = "schema: spec-driven\n";
const DONE_TASKS: &str = "## 1. Section\n\n- [x] 1.1 first task\n";
const OPEN_TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n";
const CONFIG: &str = "schema: spec-driven\naudit: false\n";

// --- the store ---

fn scope_of(id: &str) -> ScopeId {
    ScopeId::new(id).unwrap()
}

fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

/// A store with one change `demo` in scope `s1`.
fn demo_store(tasks: &str) -> MemoryStorePort {
    let port = MemoryStorePort::new();
    common::write_in(
        &port,
        &scope_of("s1"),
        &[("changes/demo/.meta.yaml", META), ("changes/demo/tasks.md", tasks), ("config.yaml", CONFIG)],
    );
    port
}

fn store_with_demo(tasks: &str) -> Arc<MemoryStorePort> {
    Arc::new(demo_store(tasks))
}

/// The demo store behind a recorder that counts every call it receives.
fn recorded(port: MemoryStorePort) -> Arc<common::Recorder> {
    Arc::new(common::Recorder::new(port))
}

fn content_of(port: &dyn StorePort, scope: &ScopeId, k: &str) -> Option<String> {
    port.read(scope, &[key(k)]).unwrap().remove(0).map(|doc| doc.content)
}

fn created_by(port: &dyn StorePort, scope: &ScopeId, change: &str) -> Option<String> {
    let meta = content_of(port, scope, &format!("changes/{change}/.meta.yaml")).expect("the change's meta");
    ChangeMeta::from_text(Some(&meta)).expect("meta parses").created_by
}

// --- callers and commands ---

fn caller(id: &str, display_name: &str, scope: &str) -> Caller {
    Caller { identity: Identity::new(id, display_name).unwrap(), scope: scope_of(scope) }
}

fn alice() -> Caller {
    caller("u_1", "Alice", "s1")
}

fn new_change(name: &str) -> Command {
    Command::NewChange {
        name: name.to_string(),
        description: None,
        schema: None,
        agent: None,
        from_discussion: None,
        last: false,
        fields: Default::default(),
    }
}

fn list_changes() -> Command {
    Command::List { sort: "name".to_string(), specs: false, changes: false, worktrees: Default::default() }
}

fn archive(carry_review: bool) -> Command {
    Command::Archive {
        change: Some("demo".to_string()),
        skip_specs: true,
        no_validate: true,
        mark_tasks_complete: false,
        carry_review,
        carry_verify: false,
    }
}

fn task_done(change: &str) -> Command {
    Command::TaskDone {
        task_id: "1".to_string(),
        change: Some(change.to_string()),
        touched_files: Some(vec!["src/app.rs".to_string()]),
        head_commit: None,
    }
}

fn set_policy(key: &str, value: &str) -> Command {
    Command::WorkflowConfigEdit {
        edit: WorkflowConfigEdit::Policy { key: key.to_string(), value: value.to_string() },
        dry_run: false,
    }
}

fn names(outcome: &CommandOutcome) -> Vec<String> {
    match outcome {
        CommandOutcome::List(list) => {
            list.changes.as_ref().map(|c| c.iter().map(|entry| entry.name.clone()).collect()).unwrap_or_default()
        }
        other => panic!("expected a list outcome, got {other:?}"),
    }
}

fn refused(err: ExecError) -> String {
    match err {
        ExecError::Command(e) => {
            assert_eq!(e.code, ErrorCode::Refused, "{}", e.message);
            e.message
        }
        other => panic!("expected a refused command, got {other:?}"),
    }
}

// --- hooks that record what they saw ---

/// What one `before_command` call received.
#[derive(Debug, Clone)]
struct Asked {
    id: String,
    scope: String,
    command: String,
    params: serde_json::Value,
    changed_keys: Option<Vec<String>>,
}

type AskLog = Arc<Mutex<Vec<Asked>>>;

/// A `before_command` hook that records each call and answers with `decide`.
fn before(log: &AskLog, decide: impl Fn(&BeforeCommand) -> Decision + Send + Sync + 'static) -> Hooks {
    let log = log.clone();
    Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand| {
            log.lock().unwrap().push(Asked {
                id: info.caller.identity.id().to_string(),
                scope: info.caller.scope.as_str().to_string(),
                command: info.command.to_string(),
                params: info.params.clone(),
                changed_keys: info.changed_keys.clone(),
            });
            decide(info)
        })),
        after_commit: None,
    }
}

/// What one `after_commit` call received.
#[derive(Debug, Clone)]
struct Watched {
    command: String,
    scope_version: u64,
    events: Vec<(String, serde_json::Value)>,
}

type WatchLog = Arc<Mutex<Vec<Watched>>>;

fn after(log: &WatchLog) -> Hooks {
    let log = log.clone();
    Hooks {
        before_command: None,
        after_commit: Some(Box::new(move |info: &AfterCommit| {
            log.lock().unwrap().push(Watched {
                command: info.command.to_string(),
                scope_version: info.scope_version.get(),
                events: info.events.iter().map(|e| (e.name().to_string(), e.payload().clone())).collect(),
            });
        })),
    }
}

// --- 宿主套件以儲存接口與選填的掛鉤建立 ---

#[test]
fn without_hooks_the_kit_gives_what_a_direct_execution_gives() {
    let port = Arc::new(MemoryStorePort::new());
    let kit = HostKit::new(port.clone(), Hooks::default());
    let direct = scope_of("direct");
    let ctx = {
        let mut ctx = common::host_ctx();
        ctx.actor = speclink_host::context::Actor::Identified {
            display: "Alice".to_string(),
            source: speclink_host::context::ActorSource::Explicit,
        };
        ctx
    };

    for cmd in [new_change("demo"), list_changes()] {
        let through_kit = kit.run(&alice(), cmd.clone()).expect("kit run");
        let straight = port_exec::execute(port.as_ref(), &direct, &ctx, cmd).expect("direct run");
        assert_eq!(format!("{:?}", through_kit.outcome), format!("{:?}", straight.outcome));
        let kinds = |events: &[speclink_core::command::DomainEvent]| events.iter().map(|e| e.kind()).collect::<Vec<_>>();
        assert_eq!(kinds(&through_kit.events), kinds(&straight.events));
        assert_eq!(through_kit.scope_version, straight.scope_version);
    }
}

#[test]
fn two_users_in_two_scopes_do_not_mix() {
    let port = Arc::new(MemoryStorePort::new());
    let kit = HostKit::new(port.clone(), Hooks::default());
    let ko = caller("u_ko", "甲", "s1");
    let otsu = caller("u_otsu", "乙", "s2");

    kit.run(&ko, new_change("alpha")).expect("alpha");
    kit.run(&otsu, new_change("beta")).expect("beta");

    assert_eq!(names(&kit.run(&ko, list_changes()).unwrap().outcome), ["alpha"]);
    assert_eq!(names(&kit.run(&otsu, list_changes()).unwrap().outcome), ["beta"]);
    assert_eq!(created_by(port.as_ref(), &scope_of("s1"), "alpha").as_deref(), Some("甲"));
    assert_eq!(created_by(port.as_ref(), &scope_of("s2"), "beta").as_deref(), Some("乙"));
}

// --- 每次呼叫由宿主給身分與範圍 ---

#[test]
fn the_id_and_the_display_name_each_go_their_way() {
    let recorder = recorded(MemoryStorePort::new());
    let kit = HostKit::new(recorder.clone(), Hooks::default());

    kit.run(&caller("u_42", "Alice <alice@example.com>", "s1"), new_change("demo")).expect("new change");

    let commits = recorder.calls().commits;
    assert_eq!(commits.len(), 1);
    let actor = commits[0].actor();
    assert_eq!((actor.id(), actor.display_name()), ("u_42", "Alice <alice@example.com>"));
    assert_eq!(commits[0].command(), "new-change");
    assert_eq!(
        created_by(&recorder.inner, &scope_of("s1"), "demo").as_deref(),
        Some("Alice <alice@example.com>")
    );
}

#[test]
fn an_empty_or_blank_identity_is_refused_before_anything_runs() {
    let recorder = recorded(MemoryStorePort::new());
    let _kit = HostKit::new(recorder.clone(), Hooks::default());

    for (id, display_name, field) in
        [("", "Alice", "id"), ("  ", "Alice", "id"), ("u_1", "", "display name"), ("u_1", " \t", "display name")]
    {
        let err = Identity::new(id, display_name).expect_err("an invalid identity");
        assert_eq!(err.field(), field, "{id:?} / {display_name:?}");
        assert!(err.to_string().contains(field), "{err}");
    }
    assert!(recorder.calls().is_empty(), "the store received no call");
}

#[test]
fn two_people_with_one_name_are_not_one_owner() {
    // 「同名的兩個人不是同一個擁有者」
    let recorder = recorded(demo_store(OPEN_TASKS));
    let kit = HostKit::new(recorder.clone(), Hooks::default());
    let claim = || Command::Claim { name: "demo".to_string() };

    kit.run(&caller("u_1", "Alice", "s1"), claim()).expect("the first claim");
    let meta = content_of(&recorder.inner, &scope_of("s1"), "changes/demo/.meta.yaml").unwrap();
    assert!(meta.contains("claimed_by: Alice\n") && meta.contains("claimed_by_id: u_1\n"), "{meta}");
    let commits = recorder.calls().commits.len();

    match kit.run(&caller("u_2", "Alice", "s1"), claim()) {
        Err(ExecError::Command(e)) => {
            assert_eq!(e.code, ErrorCode::Refused);
            assert!(e.message.contains("Alice"), "{}", e.message);
        }
        other => panic!("expected a refused claim, got {other:?}"),
    }
    kit.run(&caller("u_1", "Alice Chen", "s1"), claim()).expect("the holder under a new name");

    assert_eq!(recorder.calls().commits.len(), commits, "neither later call committed");
    let after = content_of(&recorder.inner, &scope_of("s1"), "changes/demo/.meta.yaml").unwrap();
    assert_eq!(after, meta, "claimed_by stays Alice");
}

#[test]
fn the_process_git_identity_does_not_stamp() {
    // The test process runs inside this repository, where git may have a
    // user.name; the kit reads no checkout, so the stamp is the caller's.
    let port = Arc::new(MemoryStorePort::new());
    let kit = HostKit::new(port.clone(), Hooks::default());
    kit.run(&caller("u_bob", "Bob", "s1"), new_change("demo")).expect("new change");
    assert_eq!(created_by(port.as_ref(), &scope_of("s1"), "demo").as_deref(), Some("Bob"));
}

// --- 執行前掛鉤可以拒絕命令 ---

#[test]
fn refusing_an_archive_gives_the_reason_and_touches_nothing() {
    let recorder = recorded(demo_store(DONE_TASKS));
    let log = AskLog::default();
    let hooks = before(&log, |info| match info.command {
        "archive" => Decision::Deny { reason: "程式碼還沒合併".to_string() },
        _ => Decision::Allow,
    });
    let kit = HostKit::new(recorder.clone(), hooks);

    let err = kit.run(&alice(), archive(false)).expect_err("refused");

    assert_eq!(refused(err), "程式碼還沒合併");
    assert!(recorder.calls().is_empty(), "no list, no read, no commit");
    assert_eq!(names(&kit.run(&alice(), list_changes()).unwrap().outcome), ["demo"], "still active");
}

#[test]
fn an_empty_reason_gets_the_fixed_text() {
    let log = AskLog::default();
    let kit = HostKit::new(
        Arc::new(MemoryStorePort::new()),
        before(&log, |_| Decision::Deny { reason: String::new() }),
    );
    assert_eq!(refused(kit.run(&alice(), list_changes()).unwrap_err()), "refused by host");
}

#[test]
fn the_hook_sees_the_flags_in_the_params() {
    let log = AskLog::default();
    let kit = HostKit::new(store_with_demo(DONE_TASKS), before(&log, |_| Decision::Deny { reason: "no".into() }));

    let _ = kit.run(&alice(), archive(true));

    let asked = log.lock().unwrap().clone();
    assert_eq!(asked.len(), 1);
    assert_eq!((asked[0].id.as_str(), asked[0].scope.as_str()), ("u_1", "s1"));
    assert_eq!(asked[0].command, "archive");
    assert_eq!(asked[0].params["change"], "demo");
    assert_eq!(asked[0].params["carryReview"], true);
    assert_eq!(asked[0].changed_keys, None, "only workflow-config writes carry changed keys");
}

#[test]
fn allowing_runs_as_usual_and_asks_exactly_once() {
    let log = AskLog::default();
    let with_hook = HostKit::new(Arc::new(MemoryStorePort::new()), before(&log, |_| Decision::Allow));
    let without = HostKit::new(Arc::new(MemoryStorePort::new()), Hooks::default());

    let a = with_hook.run(&alice(), new_change("demo")).expect("allowed");
    let b = without.run(&alice(), new_change("demo")).expect("no hook");

    assert_eq!(format!("{:?}", a.outcome), format!("{:?}", b.outcome));
    assert_eq!(a.scope_version, b.scope_version);
    assert_eq!(log.lock().unwrap().len(), 1);
}

#[test]
fn a_query_is_asked_too() {
    let log = AskLog::default();
    let recorder = recorded(demo_store(OPEN_TASKS));
    let kit = HostKit::new(
        recorder.clone(),
        before(&log, |info| {
            if info.command == "list" { Decision::Deny { reason: "no listing".into() } } else { Decision::Allow }
        }),
    );
    assert_eq!(refused(kit.run(&alice(), list_changes()).unwrap_err()), "no listing");
    assert!(recorder.calls().is_empty(), "no data was read");
}

// --- 流程設定的寫入在提交前帶著變動的鍵詢問掛鉤 ---

/// "A change of tdd is allowed for u_admin only."
fn tdd_admins_only(log: &AskLog) -> Hooks {
    before(log, |info| {
        let touches_tdd = info.changed_keys.as_ref().is_some_and(|keys| keys.iter().any(|k| k == "tdd"));
        if touches_tdd && info.caller.identity.id() != "u_admin" {
            Decision::Deny { reason: "only admins change tdd".into() }
        } else {
            Decision::Allow
        }
    })
}

#[test]
fn only_the_named_user_changes_a_policy_key() {
    let port = store_with_demo(OPEN_TASKS);
    let log = AskLog::default();
    let kit = HostKit::new(port.clone(), tdd_admins_only(&log));

    let err = kit.run(&caller("u_42", "User", "s1"), set_policy("tdd", "true")).expect_err("refused");

    assert_eq!(refused(err), "only admins change tdd");
    let asked = log.lock().unwrap().clone();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].command, "workflow-config-edit");
    assert_eq!(asked[0].changed_keys.as_deref(), Some(&["tdd".to_string()][..]));
    assert_eq!(content_of(port.as_ref(), &scope_of("s1"), "config.yaml").as_deref(), Some(CONFIG));

    kit.run(&caller("u_admin", "Admin", "s1"), set_policy("tdd", "true")).expect("the admin may");
}

#[test]
fn a_context_edit_is_not_held_by_the_policy_rule() {
    let port = store_with_demo(OPEN_TASKS);
    let log = AskLog::default();
    let kit = HostKit::new(port.clone(), tdd_admins_only(&log));

    let edit = Command::WorkflowConfigEdit {
        edit: WorkflowConfigEdit::Context("A new project context.".to_string()),
        dry_run: false,
    };
    kit.run(&caller("u_42", "User", "s1"), edit).expect("written");

    assert_eq!(log.lock().unwrap()[0].changed_keys.as_deref(), Some(&["context".to_string()][..]));
    let config = content_of(port.as_ref(), &scope_of("s1"), "config.yaml").unwrap();
    assert!(config.contains("A new project context."), "{config}");
}

#[test]
fn a_whole_replace_lists_its_changed_keys() {
    let log = AskLog::default();
    let kit = HostKit::new(store_with_demo(OPEN_TASKS), before(&log, |_| Decision::Allow));

    kit.run(&alice(), Command::WorkflowConfigReplace { content: "schema: spec-driven\naudit: true\n".to_string() })
        .expect("replaced");

    let asked = log.lock().unwrap().clone();
    assert_eq!(asked[0].command, "workflow-config-replace");
    assert_eq!(asked[0].changed_keys.as_deref(), Some(&["audit".to_string()][..]));
}

#[test]
fn a_dry_run_and_a_failed_write_are_asked_once_too() {
    let log = AskLog::default();
    let kit = HostKit::new(store_with_demo(OPEN_TASKS), before(&log, |_| Decision::Allow));

    let dry = Command::WorkflowConfigEdit {
        edit: WorkflowConfigEdit::Policy { key: "tdd".into(), value: "true".into() },
        dry_run: true,
    };
    kit.run(&alice(), dry).expect("dry run");
    let _ = kit.run(&alice(), set_policy("no-such-key", "true")).expect_err("an unknown key");

    let asked = log.lock().unwrap().clone();
    assert_eq!(asked.len(), 2, "{asked:?}");
    assert_eq!(asked[0].changed_keys.as_deref(), Some(&["tdd".to_string()][..]));
    assert_eq!(asked[1].changed_keys.as_deref(), Some(&[][..]), "a write that fails changes no key");
}

#[test]
fn a_write_whose_ground_moved_after_the_approval_is_a_conflict() {
    let port = store_with_demo(OPEN_TASKS);
    let other = port.clone();
    let log = AskLog::default();
    // The hook runs after the command read the config and before the commit:
    // another caller writes the config in that gap.
    let kit = HostKit::new(
        port.clone(),
        before(&log, move |_| {
            common::write_in(other.as_ref(), &scope_of("s1"), &[("config.yaml", "schema: spec-driven\nlocale: ja\n")]);
            Decision::Allow
        }),
    );

    let err = kit.run(&alice(), set_policy("tdd", "true")).expect_err("conflict");

    assert!(matches!(err, ExecError::Store(PortError::Conflict { .. })), "{err:?}");
    assert_eq!(
        content_of(port.as_ref(), &scope_of("s1"), "config.yaml").as_deref(),
        Some("schema: spec-driven\nlocale: ja\n")
    );
}

// --- 提交後掛鉤只能旁觀 ---

#[test]
fn after_a_commit_the_hook_gets_the_events() {
    let port = store_with_demo(OPEN_TASKS);
    let before_version = common::scope_version_in(port.as_ref(), &scope_of("s1"));
    let log = WatchLog::default();
    let kit = HostKit::new(port.clone(), after(&log));

    let done = kit.run(&alice(), task_done("demo")).expect("task done");

    let watched = log.lock().unwrap().clone();
    assert_eq!(watched.len(), 1);
    assert_eq!(watched[0].command, "task-done");
    assert!(watched[0].scope_version > before_version);
    assert_eq!(Some(watched[0].scope_version), done.scope_version.map(Version::get));
    assert_eq!(watched[0].events.len(), 1);
    assert_eq!(watched[0].events[0].0, "task-completed");
    assert_eq!(watched[0].events[0].1["change"], "demo");
}

#[test]
fn a_query_does_not_trigger_it() {
    let log = WatchLog::default();
    let kit = HostKit::new(store_with_demo(OPEN_TASKS), after(&log));
    kit.run(&alice(), list_changes()).expect("list");
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn a_failed_commit_does_not_trigger_it() {
    // Another writer changes the tasks after the command read them.
    let recorder = Arc::new(common::Recorder::new(demo_store(OPEN_TASKS)).before_first_commit(|store| {
        common::write_in(store, &scope_of("s1"), &[("changes/demo/tasks.md", "## 1. Section\n\n- [ ] 1.1 changed\n")]);
    }));
    let log = WatchLog::default();
    let kit = HostKit::new(recorder.clone(), after(&log));

    let err = kit.run(&alice(), task_done("demo")).expect_err("conflict");

    assert!(matches!(err, ExecError::Store(PortError::Conflict { .. })), "{err:?}");
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn a_refused_command_does_not_trigger_it() {
    let watched = WatchLog::default();
    let mut hooks = after(&watched);
    hooks.before_command = Some(Box::new(|_: &BeforeCommand| Decision::Deny { reason: "no".into() }));
    let kit = HostKit::new(store_with_demo(OPEN_TASKS), hooks);
    let _ = kit.run(&alice(), task_done("demo")).expect_err("refused");
    assert!(watched.lock().unwrap().is_empty());
}

#[test]
fn a_failing_hook_changes_nothing() {
    let port = Arc::new(MemoryStorePort::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let kit = HostKit::new(
        port.clone(),
        Hooks {
            before_command: None,
            after_commit: Some(Box::new(move |_: &AfterCommit| {
                seen.fetch_add(1, Ordering::SeqCst);
                panic!("the notifier is down");
            })),
        },
    );
    let calm = HostKit::new(Arc::new(MemoryStorePort::new()), Hooks::default());

    let done = kit.run(&alice(), new_change("demo")).expect("the call succeeds");
    let same = calm.run(&alice(), new_change("demo")).expect("no hook");

    assert_eq!(calls.load(Ordering::SeqCst), 1, "the hook was called");
    assert_eq!(format!("{:?}", done.outcome), format!("{:?}", same.outcome));
    assert_eq!(done.scope_version, same.scope_version);
    assert!(content_of(port.as_ref(), &scope_of("s1"), "changes/demo/.meta.yaml").is_some());
}

// --- 宿主套件的失敗分成命令層與儲存層 ---

#[test]
fn an_unknown_change_is_a_command_failure() {
    let recorder = recorded(demo_store(OPEN_TASKS));
    let kit = HostKit::new(recorder.clone(), Hooks::default());
    match kit.run(&alice(), task_done("nonexistent")) {
        Err(ExecError::Command(e)) => assert_eq!(e.code, ErrorCode::NotFound),
        other => panic!("expected a command not_found, got {other:?}"),
    }
    assert!(recorder.calls().commits.is_empty());
}

// --- design 的開放問題 ---

#[test]
fn task_done_without_touched_files_is_a_clear_result() {
    // No workspace and no "files changed" list: the Engine's probe of a local
    // workspace has nothing to probe. The call must end as a clear success or
    // a clear error, never a panic.
    let port = store_with_demo(OPEN_TASKS);
    let log = AskLog::default();
    let kit = HostKit::new(port.clone(), before(&log, |_| Decision::Allow));
    let cmd = Command::TaskDone {
        task_id: "1".to_string(),
        change: Some("demo".to_string()),
        touched_files: None,
        head_commit: None,
    };

    let done = kit.run(&alice(), cmd).expect("task done succeeds");

    assert_eq!(log.lock().unwrap()[0].params["touchedFiles"], serde_json::Value::Null, "the params as sent");

    match &done.outcome {
        CommandOutcome::TaskDone(flip) => assert!(flip.touched_files.is_empty(), "{:?}", flip.touched_files),
        other => panic!("expected a task-done outcome, got {other:?}"),
    }
    let tasks = content_of(port.as_ref(), &scope_of("s1"), "changes/demo/tasks.md").unwrap();
    assert!(tasks.contains("- [x] 1.1"), "{tasks}");
    assert_eq!(
        content_of(port.as_ref(), &scope_of("s1"), "changes/demo/.evidence.json"),
        None,
        "nothing attributable, so no evidence entry"
    );
}

#[test]
fn the_evidence_entry_of_the_kit_has_no_repo() {
    // `repo` stays empty on the kit's path: the entry leaves it out, and its
    // readers take only the touched files.
    let port = store_with_demo(OPEN_TASKS);
    let kit = HostKit::new(port.clone(), Hooks::default());
    kit.run(&alice(), task_done("demo")).expect("task done");

    let evidence = content_of(port.as_ref(), &scope_of("s1"), "changes/demo/.evidence.json")
        .expect("the evidence record");
    let evidence: serde_json::Value = serde_json::from_str(&evidence).unwrap();
    let entry = &evidence["entries"][0];
    assert_eq!(entry["touchedFiles"], serde_json::json!(["src/app.rs"]));
    assert_eq!(entry["actor"], "Alice");
    assert!(entry.get("repo").is_none(), "the kit leaves the repo out: {entry}");
    assert!(entry.get("headCommit").is_none(), "no checkout, no head commit: {entry}");
}

#[test]
fn an_unavailable_store_is_a_store_failure() {
    let recorder = Arc::new(common::Recorder::unavailable());
    let kit = HostKit::new(recorder.clone(), Hooks::default());
    match kit.run(&alice(), task_done("demo")) {
        Err(ExecError::Store(PortError::Unavailable { .. })) => {}
        other => panic!("expected a store unavailable, got {other:?}"),
    }
    assert!(recorder.calls().commits.is_empty());
}
