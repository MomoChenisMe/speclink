//! Free fields through the host kit's two doors (change host-free-fields;
//! specs host-kit「呼叫入口時附上自由欄位」, server-verb-api「自由欄位的寫入端點」,
//! client-protocol「自由欄位的線上欄位與請求」). Every scenario runs on the
//! in-memory store port behind the shared recorder.

use serde_json::{json, Value};
use speclink_core::command::{Command, ErrorCode};
use speclink_core::fields::Fields;
use speclink_host::http::{routes, wire_coverage, HttpRequest, HttpResponse, Method, WireCoverage};
use speclink_host::kit::{BeforeCommand, Caller, Decision, Hooks};
use speclink_host::port_exec::ExecError;
use speclink_store_port::{Commit, Op};
use std::sync::{Arc, Mutex};

mod common;
use common::door::*;

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n";
const RECORD: &str = "---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-01\n---\n\n\
                      # Discussion: Board search\n\n## Context\n\n## Rounds\n\n## Conclusion\n\nShip it.\n";

// --- the door ---

fn caller() -> Caller {
    Caller { identity: alice(), scope: scope_s1() }
}

/// A door whose before hook keeps what it was asked.
fn door_that_records() -> (Door, Arc<Mutex<Vec<BeforeCommand>>>) {
    let asked = Arc::new(Mutex::new(Vec::new()));
    let seen = asked.clone();
    let hooks = Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand| {
            seen.lock().unwrap().push(info.clone());
            Decision::Allow
        })),
        ..Default::default()
    };
    (door_with(true, hooks), asked)
}

fn map(pairs: &[(&str, &str)]) -> Fields {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn answer_with(door: &Door, req: &HttpRequest, attached: &Fields) -> HttpResponse {
    door.kit
        .handle_with_fields(req, Some(&alice()), attached)
        .unwrap_or_else(|| panic!("{} {} is the door's path", req.method.as_str(), req.path))
}

fn put(door: &Door, tail: &str, body: Value) -> HttpResponse {
    answer(door, &request(Method::Put, tail, Some(body)))
}

/// A response that must be a 200, as JSON.
fn ok(response: &HttpResponse) -> Value {
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    body_of(response)
}

fn scope_version(door: &Door) -> u64 {
    common::scope_version_in(&door.store.inner, &scope_s1())
}

/// The content a commit writes to `key`.
fn written<'c>(commit: &'c Commit, key: &str) -> Option<&'c str> {
    commit.ops().iter().find_map(|op| match op {
        Op::Put { key: k, content, .. } if k.as_str() == key => Some(content.as_str()),
        _ => None,
    })
}

fn new_change(name: &str, fields: &[(&str, &str)]) -> Command {
    Command::NewChange {
        name: name.to_string(),
        description: None,
        schema: Some("spec-driven".to_string()),
        agent: None,
        from_discussion: None,
        last: false,
        fields: map(fields),
    }
}

fn list_changes() -> Command {
    Command::List { sort: "name".to_string(), specs: false, changes: false, worktrees: Default::default() }
}

fn fields_of(door: &Door, change: &str) -> Fields {
    let meta = content(door, &format!("changes/{change}/.meta.yaml")).unwrap_or_else(|| panic!("'{change}' exists"));
    speclink_core::model::ChangeMeta::from_text(Some(&meta)).expect("the metadata parses").fields
}

fn command_error(result: Result<speclink_host::port_exec::PortExecution, ExecError>) -> speclink_core::command::CommandError {
    match result {
        Err(ExecError::Command(e)) => e,
        Err(other) => panic!("expected a command error, got {other:?}"),
        Ok(_) => panic!("expected a command error, the call succeeded"),
    }
}

// === host-kit「呼叫入口時附上自由欄位」：Agent 工具的門（HostKit::run_with_fields） ===

#[test]
fn fields_the_host_attaches_are_in_the_one_commit_that_creates_a_change() {
    // Scenario「建立變更時附上來源」。
    let door = door();
    door.kit.run_with_fields(&caller(), new_change("demo", &[]), &map(&[("source", "portal")])).expect("created");
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1, "exactly one commit");
    let meta = written(&calls.commits[0], "changes/demo/.meta.yaml").expect("the commit writes the metadata");
    assert!(meta.ends_with("fields:\n  source: \"portal\"\n"), "{meta}");
    assert_eq!(fields_of(&door, "demo"), map(&[("source", "portal")]));
    // 建立時的事件不變：沒有另外的欄位事件。
    let events: Vec<&str> = calls.commits[0].events().iter().map(|e| e.name()).collect();
    assert_eq!(events, ["change-created"]);
}

#[test]
fn fields_the_host_attaches_are_in_the_one_commit_that_creates_a_discussion_or_promotes_it() {
    let door = door();
    let new = Command::DiscussNew {
        topic: "Board search".to_string(),
        slug: Some("board-search".to_string()),
        kind: None,
        fields: Fields::new(),
    };
    door.kit.run_with_fields(&caller(), new, &map(&[("owner", "alice")])).expect("created");
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1);
    let record = written(&calls.commits[0], "discussions/board-search.md").expect("the commit writes the record");
    assert!(record.contains("\nfields:\n  owner: \"alice\"\n---\n"), "{record}");

    let promote =
        Command::DiscussPromote { slug: "board-search".to_string(), name: Some("demo".to_string()), last: false, fields: Fields::new() };
    door.kit.run_with_fields(&caller(), promote, &map(&[("source", "portal")])).expect("promoted");
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1);
    let meta = written(&calls.commits[0], "changes/demo/.meta.yaml").expect("the commit writes the metadata");
    assert!(meta.contains("fields:\n  source: \"portal\"\n"), "{meta}");
    assert_eq!(fields_of(&door, "demo"), map(&[("source", "portal")]));
}

#[test]
fn the_hosts_value_wins_over_the_commands_own_and_the_other_keys_stay() {
    let door = door();
    let cmd = new_change("demo", &[("source", "cli"), ("ticket", "PROJ-1")]);
    door.kit.run_with_fields(&caller(), cmd, &map(&[("source", "portal"), ("tenant", "t1")])).expect("created");
    assert_eq!(fields_of(&door, "demo"), map(&[("source", "portal"), ("tenant", "t1"), ("ticket", "PROJ-1")]));
}

#[test]
fn every_other_command_ignores_the_attached_fields() {
    // Scenario「其他命令忽略」：列出變更與勾選任務。
    let door = door();
    seed(
        &door,
        &[("config.yaml", "schema: spec-driven\n"), ("changes/demo/.meta.yaml", META), ("changes/demo/tasks.md", TASKS)],
    );
    let attached = map(&[("source", "portal")]);
    let listed = door.kit.run_with_fields(&caller(), list_changes(), &attached).expect("list");
    assert!(listed.scope_version.is_none(), "a list commits nothing");
    assert!(door.store.take().commits.is_empty());

    let done =
        Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: Some(vec![]), head_commit: None };
    door.kit.run_with_fields(&caller(), done, &attached).expect("task done");
    assert!(fields_of(&door, "demo").is_empty(), "no field was written");
    assert_eq!(content(&door, "changes/demo/.meta.yaml").as_deref().map(|m| m.contains("fields")), Some(false));

    // 每次呼叫都帶同一組：就算這組不合規則，不建立東西的命令也不因此失敗。
    door.kit.run_with_fields(&caller(), list_changes(), &map(&[("Bad", "1")])).expect("still a list");
}

#[test]
fn the_before_hook_sees_the_fields_that_will_be_written() {
    // Scenario「掛鉤看到最後的欄位」。
    let (door, asked) = door_that_records();
    let cmd = new_change("demo", &[("source", "cli"), ("ticket", "PROJ-1")]);
    door.kit.run_with_fields(&caller(), cmd, &map(&[("source", "portal")])).expect("created");
    let asked = asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].command, "new-change");
    assert_eq!(asked[0].params["fields"], json!({ "source": "portal", "ticket": "PROJ-1" }));
}

#[test]
fn a_hook_can_refuse_by_the_fields_and_nothing_is_written() {
    let hooks = Hooks {
        before_command: Some(Box::new(|info: &BeforeCommand| match info.params["fields"].get("source") {
            Some(_) => Decision::Allow,
            None => Decision::Deny { reason: "a change needs a source".to_string() },
        })),
        ..Default::default()
    };
    let door = door_with(true, hooks);
    let refused = command_error(door.kit.run(&caller(), new_change("demo", &[])));
    assert_eq!(refused.code, ErrorCode::Refused);
    assert!(door.store.take().commits.is_empty());
    door.kit.run_with_fields(&caller(), new_change("demo", &[]), &map(&[("source", "portal")])).expect("allowed");
}

#[test]
fn attached_fields_outside_the_rules_fail_the_call_as_invalid_argv_with_nothing_written() {
    // Scenario「附上的欄位不合規則」。
    let (door, asked) = door_that_records();
    let too_long = "x".repeat(1025);
    for attached in [map(&[("Bad", "1")]), map(&[("note", "a\nb")]), map(&[("note", too_long.as_str())])] {
        let err = command_error(door.kit.run_with_fields(&caller(), new_change("demo", &[]), &attached));
        assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        assert!(!err.message.contains(too_long.as_str()), "the message never carries a value");
    }
    // 合起來超過 32 個也是。
    let many: Vec<(String, String)> = (0..20).map(|i| (format!("a{i:02}"), "v".to_string())).collect();
    let own: Vec<(&str, &str)> = many.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let more: Fields = (0..20).map(|i| (format!("b{i:02}"), "v".to_string())).collect();
    let err = command_error(door.kit.run_with_fields(&caller(), new_change("demo", &own), &more));
    assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);

    assert!(content(&door, "changes/demo/.meta.yaml").is_none(), "the change was not created");
    assert!(door.store.take().commits.is_empty());
    // 是宿主的程式錯誤：在問掛鉤之前就失敗。
    assert!(asked.lock().unwrap().is_empty());
}

#[test]
fn a_run_without_attached_fields_is_a_plain_run() {
    let door = door();
    door.kit.run(&caller(), new_change("plain", &[("ticket", "PROJ-1")])).expect("created");
    door.kit.run_with_fields(&caller(), new_change("same", &[("ticket", "PROJ-1")]), &Fields::new()).expect("created");
    assert_eq!(fields_of(&door, "plain"), fields_of(&door, "same"));
}

// === host-kit：HTTP 入口（HostKit::handle_with_fields） ===

#[test]
fn the_http_door_attaches_the_hosts_fields_over_the_requests_own() {
    // Scenario「宿主的值蓋過請求的值」。
    let door = door();
    let req = request(Method::Post, "changes", Some(json!({ "name": "demo", "fields": { "source": "cli", "ticket": "PROJ-1" } })));
    ok(&answer_with(&door, &req, &map(&[("source", "portal")])));
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1, "the fields are in the commit that creates the change");
    assert_eq!(fields_of(&door, "demo"), map(&[("source", "portal"), ("ticket", "PROJ-1")]));
    assert_eq!(get(&door, "changes/demo")["fields"], json!({ "source": "portal", "ticket": "PROJ-1" }));
}

#[test]
fn the_http_door_attaches_fields_to_a_new_discussion_and_to_a_promoted_change() {
    let door = door();
    let new = request(Method::Post, "discussions", Some(json!({ "topic": "Board search", "slug": "board-search" })));
    ok(&answer_with(&door, &new, &map(&[("owner", "alice")])));
    assert_eq!(get(&door, "discussions/board-search")["info"]["fields"], json!({ "owner": "alice" }));

    let promote = request(Method::Post, "discussions/board-search/promote", Some(json!({ "name": "demo" })));
    ok(&answer_with(&door, &promote, &map(&[("source", "portal")])));
    assert_eq!(fields_of(&door, "demo"), map(&[("source", "portal")]));
}

#[test]
fn the_http_door_ignores_the_attached_fields_on_every_other_route() {
    let door = door();
    seed(
        &door,
        &[("config.yaml", "schema: spec-driven\n"), ("changes/demo/.meta.yaml", META), ("changes/demo/tasks.md", TASKS)],
    );
    let attached = map(&[("Bad Key", "1")]);
    ok(&answer_with(&door, &request(Method::Get, "changes", None), &attached));
    ok(&answer_with(&door, &request(Method::Post, "changes/demo/tasks/1/done", Some(json!({}))), &attached));
    ok(&answer_with(&door, &request(Method::Put, "changes/demo/fields", Some(json!({ "set": { "a": "1" } }))), &attached));
    // 宿主附上的欄位只在建立時生效：欄位的寫入端點寫的是請求自己的內容。
    assert_eq!(fields_of(&door, "demo"), map(&[("a", "1")]));
}

#[test]
fn attached_fields_outside_the_rules_fail_a_create_request_with_400() {
    let door = door();
    let req = request(Method::Post, "changes", Some(json!({ "name": "demo" })));
    let response = answer_with(&door, &req, &map(&[("Bad", "1")]));
    assert_eq!(error(&response), (400, "invalid_argument".to_string()));
    assert!(door.store.take().commits.is_empty());
}

#[test]
fn handle_is_handle_with_no_fields() {
    let door = door();
    ok(&post(&door, "changes", json!({ "name": "demo", "fields": { "ticket": "PROJ-1" } })));
    assert_eq!(fields_of(&door, "demo"), map(&[("ticket", "PROJ-1")]));
}

// === 稽核（sharp edges）：這次呼叫裡沒有東西能移除或改寫宿主附上的欄位 ===

#[test]
fn nothing_in_the_request_removes_or_rewrites_what_the_host_attached() {
    let door = door();
    let attached = map(&[("source", "portal")]);
    let bodies = [
        json!({ "name": "a", "fields": { "source": "" } }),
        json!({ "name": "b", "fields": {} }),
        json!({ "name": "c" }),
        // 請求裡沒有「移除」的寫法：不認得的鍵被忽略。
        json!({ "name": "d", "unset": ["source"], "extras": { "fields": {} }, "fields": { "source": "client" } }),
    ];
    for body in bodies {
        let name = body["name"].as_str().unwrap().to_string();
        ok(&answer_with(&door, &request(Method::Post, "changes", Some(body)), &attached));
        assert_eq!(fields_of(&door, &name), map(&[("source", "portal")]), "{name}");
    }
    // `fields` 不是物件、或值不是字串：整個請求不成立，什麼都沒建立。
    for body in [json!({ "name": "e", "fields": null }), json!({ "name": "e", "fields": { "source": null } }), json!({ "name": "e", "fields": ["source"] })] {
        let response = answer_with(&door, &request(Method::Post, "changes", Some(body.clone())), &attached);
        assert_eq!(error(&response), (400, "invalid_argument".to_string()), "{body}");
    }
    assert!(content(&door, "changes/e/.meta.yaml").is_none());
}

#[test]
fn the_attached_fields_change_no_other_part_of_the_command() {
    let (door, asked) = door_that_records();
    let cmd = || Command::NewChange {
        name: "demo".to_string(),
        description: Some("A demo".to_string()),
        schema: Some("spec-driven".to_string()),
        agent: Some("claude".to_string()),
        from_discussion: None,
        last: false,
        fields: map(&[("ticket", "PROJ-1")]),
    };
    let plain = cmd().params();
    // 名稱像命令其他參數的鍵，也只是欄位。
    door.kit
        .run_with_fields(&caller(), cmd(), &map(&[("name", "other"), ("schema", "evil"), ("agent", "x")]))
        .expect("created");
    let asked = asked.lock().unwrap();
    let mut seen = asked[0].params.clone();
    assert_eq!(seen["fields"], json!({ "agent": "x", "name": "other", "schema": "evil", "ticket": "PROJ-1" }));
    seen["fields"] = plain["fields"].clone();
    assert_eq!(seen, plain, "only `fields` differs");
    assert!(content(&door, "changes/demo/.meta.yaml").unwrap().starts_with("schema: spec-driven\n"));
    assert!(content(&door, "changes/other/.meta.yaml").is_none());
}

// === server-verb-api「自由欄位的寫入端點」 ===

fn door_with_demo() -> Door {
    let door = door();
    seed(
        &door,
        &[
            ("config.yaml", "schema: spec-driven\n"),
            ("changes/demo/.meta.yaml", META),
            ("changes/demo/tasks.md", TASKS),
            ("discussions/board-search.md", RECORD),
        ],
    );
    door
}

#[test]
fn an_editor_sets_fields_and_the_scope_version_moves_once() {
    // Scenario「editor 設定欄位」。
    let door = door_with_demo();
    let before = scope_version(&door);
    let response = put(&door, "changes/demo/fields", json!({ "set": { "ticket": "PROJ-1" } }));
    let body = ok(&response);
    assert_eq!(body["fields"], json!({ "ticket": "PROJ-1" }));
    assert_eq!(body["changed"], json!(["ticket"]));
    assert_eq!(etag_of(&response), before + 1);
    assert_eq!(scope_version(&door), before + 1);
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1);
    assert_eq!(calls.commits[0].command(), "change-fields-edit");
}

#[test]
fn a_write_request_sets_and_unsets_in_one_commit() {
    // client-protocol Scenario「寫入請求」：{"set":{"a":"1"},"unset":["b"]}。
    let door = door_with_demo();
    ok(&put(&door, "changes/demo/fields", json!({ "set": { "b": "2", "keep": "k" } })));
    door.store.take();
    let before = scope_version(&door);
    let response = put(&door, "changes/demo/fields", json!({ "set": { "a": "1" }, "unset": ["b"] }));
    let body = ok(&response);
    assert_eq!(body["fields"], json!({ "a": "1", "keep": "k" }));
    assert_eq!(body["changed"], json!(["a", "b"]));
    assert_eq!(etag_of(&response), before + 1, "a new scope version");
    assert_eq!(door.store.take().commits.len(), 1);
}

#[test]
fn the_commit_of_a_field_write_carries_one_event_with_keys_and_no_value() {
    // free-fields Scenario「事件只帶鍵」：secret=abc。
    let door = door_with_demo();
    ok(&put(&door, "changes/demo/fields", json!({ "set": { "secret": "abc" } })));
    ok(&put(&door, "discussions/board-search/fields", json!({ "set": { "owner": "alice" } })));
    let calls = door.store.take();
    let events: Vec<(&str, &Value)> =
        calls.commits.iter().flat_map(|c| c.events()).map(|e| (e.name(), e.payload())).collect();
    assert_eq!(
        events,
        [
            ("change-fields-changed", &json!({ "change": "demo", "keys": ["secret"] })),
            ("discussion-fields-changed", &json!({ "slug": "board-search", "keys": ["owner"] })),
        ]
    );
}

#[test]
fn a_reader_is_refused_and_the_store_stays() {
    // Scenario「reader 被拒」。
    let door = door_with(false, Hooks::default());
    seed(&door, &[("changes/demo/.meta.yaml", META), ("discussions/board-search.md", RECORD)]);
    for tail in ["changes/demo/fields", "discussions/board-search/fields"] {
        let response = put(&door, tail, json!({ "set": { "ticket": "PROJ-1" } }));
        assert_eq!(response.status, 403, "{tail}");
    }
    assert!(door.store.take().commits.is_empty());
    assert_eq!(content(&door, "changes/demo/.meta.yaml").as_deref(), Some(META));
}

#[test]
fn a_request_outside_the_rules_is_invalid_argument_and_the_store_stays() {
    // Scenario「規則不符」與 client-protocol「set 與 unset 都是空的」。
    let door = door_with_demo();
    ok(&put(&door, "changes/demo/fields", json!({ "set": { "a": "1" } })));
    door.store.take();
    let too_long = "x".repeat(1025);
    let too_many: serde_json::Map<String, Value> = (0..33).map(|i| (format!("k{i:02}"), json!("v"))).collect();
    let bodies = [
        json!({ "set": { "Bad": "1" } }),
        json!({ "set": { "note": "a\nb" } }),
        json!({ "set": { "note": too_long } }),
        json!({ "set": too_many }),
        json!({ "set": { "a": "2" }, "unset": ["a"] }),
        json!({}),
        json!({ "set": {}, "unset": [] }),
        // 值不是字串：線上的型別就不收。
        json!({ "set": { "n": 1 } }),
    ];
    for tail in ["changes/demo/fields", "discussions/board-search/fields"] {
        for body in &bodies {
            let response = put(&door, tail, body.clone());
            assert_eq!(error(&response), (400, "invalid_argument".to_string()), "{tail}: {body}");
        }
    }
    assert!(door.store.take().commits.is_empty());
    assert_eq!(fields_of(&door, "demo"), map(&[("a", "1")]));
    assert_eq!(content(&door, "discussions/board-search.md").as_deref(), Some(RECORD));
}

#[test]
fn a_change_or_a_discussion_that_is_not_there_is_not_found() {
    let door = door_with_demo();
    seed(&door, &[("archive/discussions/2026-10-01-old-talk.md", RECORD)]);
    for tail in ["changes/ghost/fields", "discussions/ghost/fields", "discussions/old-talk/fields"] {
        let response = put(&door, tail, json!({ "set": { "a": "1" } }));
        assert_eq!(error(&response), (404, "not_found".to_string()), "{tail}");
    }
    assert!(door.store.take().commits.is_empty());
}

#[test]
fn a_write_that_changes_nothing_answers_the_fields_and_keeps_the_scope_version() {
    // Scenario「沒有變動不前進版本」：{"unset":["ghost"]}。
    let door = door_with_demo();
    ok(&put(&door, "changes/demo/fields", json!({ "set": { "a": "1" } })));
    door.store.take();
    let before = scope_version(&door);
    for body in [json!({ "unset": ["ghost"] }), json!({ "set": { "a": "1" } })] {
        let response = put(&door, "changes/demo/fields", body);
        let answered = ok(&response);
        assert_eq!(answered["fields"], json!({ "a": "1" }));
        assert_eq!(answered["changed"], json!([]));
        assert_eq!(etag_of(&response), before);
    }
    assert_eq!(scope_version(&door), before);
    assert!(door.store.take().commits.is_empty());
}

#[test]
fn a_discussion_takes_the_same_write() {
    let door = door_with_demo();
    let response = put(&door, "discussions/board-search/fields", json!({ "set": { "owner": "alice", "n": "123" } }));
    assert_eq!(ok(&response)["fields"], json!({ "n": "123", "owner": "alice" }));
    let response = put(&door, "discussions/board-search/fields", json!({ "unset": ["n"] }));
    let body = ok(&response);
    assert_eq!(body["fields"], json!({ "owner": "alice" }));
    assert_eq!(body["changed"], json!(["n"]));
    let record = content(&door, "discussions/board-search.md").unwrap();
    let (head, rest) = RECORD.split_once("---\n\n").unwrap();
    assert_eq!(record, format!("{head}fields:\n  owner: \"alice\"\n---\n\n{rest}"), "only the frontmatter changed");
}

#[test]
fn a_create_request_with_fields_is_one_commit_that_already_holds_them() {
    // Scenario「建立與欄位同一次提交」與 client-protocol「建立請求帶欄位」。
    let door = door();
    ok(&post(&door, "changes", json!({ "name": "demo", "fields": { "ticket": "PROJ-1" } })));
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1);
    let meta = written(&calls.commits[0], "changes/demo/.meta.yaml").expect("the commit writes the metadata");
    assert!(meta.contains("fields:\n  ticket: \"PROJ-1\"\n"), "{meta}");
    assert_eq!(get(&door, "changes/demo")["fields"], json!({ "ticket": "PROJ-1" }));

    ok(&post(&door, "discussions", json!({ "topic": "Board search", "slug": "board-search", "fields": { "owner": "alice" } })));
    let calls = door.store.take();
    assert_eq!(calls.commits.len(), 1);
    assert!(written(&calls.commits[0], "discussions/board-search.md").unwrap().contains("  owner: \"alice\"\n"));

    ok(&post(&door, "discussions/board-search/promote", json!({ "name": "cut", "fields": { "source": "portal" } })));
    assert_eq!(fields_of(&door, "cut"), map(&[("source", "portal")]));
}

#[test]
fn a_create_request_with_a_field_outside_the_rules_creates_nothing() {
    let door = door();
    let change = post(&door, "changes", json!({ "name": "demo", "fields": { "Bad": "1" } }));
    assert_eq!(error(&change), (400, "invalid_argument".to_string()));
    let discussion = post(&door, "discussions", json!({ "topic": "T", "slug": "t", "fields": { "Bad": "1" } }));
    assert_eq!(error(&discussion), (400, "invalid_argument".to_string()));
    assert!(door.store.take().commits.is_empty());
}

// === client-protocol「自由欄位的線上欄位與請求」：回應的欄位 ===

fn door_with_fields_everywhere() -> Door {
    let door = door();
    let fields = "fields:\n  source: \"portal\"\n  ticket: \"PROJ-1\"\n";
    let record = RECORD.replacen("created: 2026-11-01\n", &format!("created: 2026-11-01\n{fields}"), 1);
    seed(
        &door,
        &[
            ("config.yaml", "schema: spec-driven\n"),
            ("changes/demo/.meta.yaml", &format!("{META}{fields}")),
            ("changes/demo/tasks.md", TASKS),
            ("changes/other/.meta.yaml", META),
            ("archive/changes/2026-11-02-done/.meta.yaml", &format!("{META}{fields}")),
            ("archive/changes/2026-11-02-plain/.meta.yaml", META),
            ("discussions/board-search.md", &record),
            ("discussions/plain-talk.md", &RECORD.replace("board-search", "plain-talk")),
        ],
    );
    door
}

#[test]
fn the_list_the_single_change_the_archive_and_the_discussions_carry_the_fields() {
    // Scenario「回應帶出欄位」。
    let door = door_with_fields_everywhere();
    let fields = json!({ "source": "portal", "ticket": "PROJ-1" });
    assert_eq!(listed(&door, "demo")["fields"], fields);
    assert_eq!(get(&door, "changes/demo")["fields"], fields);
    let archived = get(&door, "archived");
    let archived = archived["archived"].as_array().expect("the archived items");
    let done = archived.iter().find(|a| a["name"] == "done").expect("the archived change");
    assert_eq!(done["fields"], fields);
    let discussions = get(&door, "discussions");
    let talk = discussions["discussions"].as_array().unwrap().iter().find(|d| d["slug"] == "board-search").unwrap().clone();
    assert_eq!(talk["fields"], fields);
    assert_eq!(get(&door, "discussions/board-search")["info"]["fields"], fields);
    // 值一律是字串。
    assert!(fields.as_object().unwrap().values().all(Value::is_string));
}

#[test]
fn a_document_without_fields_has_no_fields_key_on_the_wire() {
    // Scenario「空的不序列化」。
    let door = door_with_fields_everywhere();
    assert!(listed(&door, "other").get("fields").is_none());
    assert!(get(&door, "changes/other").get("fields").is_none());
    let archived = get(&door, "archived");
    let plain = archived["archived"].as_array().unwrap().iter().find(|a| a["name"] == "plain").unwrap().clone();
    assert!(plain.get("fields").is_none(), "{plain}");
    let discussions = get(&door, "discussions");
    let plain = discussions["discussions"].as_array().unwrap().iter().find(|d| d["slug"] == "plain-talk").unwrap().clone();
    assert!(plain.get("fields").is_none(), "{plain}");
    assert!(get(&door, "discussions/plain-talk")["info"].get("fields").is_none());
}

#[test]
fn the_fields_cost_no_store_call() {
    // server-verb-api：讀取端點回填 fields，不為此多發儲存呼叫。同一批文件，有欄位與沒有
    // 欄位各量一次；已封存清單也在內（design 的 Open Question：它用的是已經讀過的封存
    // metadata）。
    let with = door_with_fields_everywhere();
    let without = door();
    seed(
        &without,
        &[
            ("config.yaml", "schema: spec-driven\n"),
            ("changes/demo/.meta.yaml", META),
            ("changes/demo/tasks.md", TASKS),
            ("changes/other/.meta.yaml", META),
            ("archive/changes/2026-11-02-done/.meta.yaml", META),
            ("archive/changes/2026-11-02-plain/.meta.yaml", META),
            ("discussions/board-search.md", RECORD),
            ("discussions/plain-talk.md", &RECORD.replace("board-search", "plain-talk")),
        ],
    );
    for tail in ["changes", "changes/demo", "archived", "discussions", "discussions/board-search"] {
        get(&with, tail);
        get(&without, tail);
        let (a, b) = (with.store.take(), without.store.take());
        assert_eq!(a.lists, b.lists, "{tail}: lists");
        assert_eq!(a.read_keys(), b.read_keys(), "{tail}: reads");
        assert!(a.commits.is_empty(), "{tail}");
    }
    // 變更清單與單筆的次數，是加上這些欄位之前量到的值（station_wire 的同一個量測）。
    for (tail, lists, reads) in [("changes", 2, 3), ("archived", 1, 1)] {
        get(&with, tail);
        let calls = with.store.take();
        assert_eq!((calls.lists.len(), calls.reads.len()), (lists, reads), "{tail}");
    }
}

#[test]
fn the_handshake_declares_fields() {
    // host-kit Scenario「握手宣告支援」。
    let door = door();
    assert_eq!(get(&door, "binding")["capabilities"]["fields"], json!(true));
}

#[test]
fn a_client_type_that_does_not_know_fields_still_reads_the_responses() {
    // Scenario「舊版用戶端解析新回應」。
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OldSummary {
        name: String,
        status: String,
        total_tasks: usize,
    }
    #[derive(serde::Deserialize)]
    struct OldList {
        changes: Vec<OldSummary>,
    }
    #[derive(serde::Deserialize)]
    struct OldCapabilities {
        validate: bool,
    }
    #[derive(serde::Deserialize)]
    struct OldBinding {
        capabilities: OldCapabilities,
    }
    let door = door_with_fields_everywhere();
    let list: OldList = serde_json::from_value(get(&door, "changes")).expect("the old list type reads it");
    let demo = list.changes.iter().find(|c| c.name == "demo").expect("demo");
    assert_eq!((demo.status.as_str(), demo.total_tasks), ("in-progress", 1));
    let binding: OldBinding = serde_json::from_value(get(&door, "binding")).expect("the old handshake type reads it");
    assert!(binding.capabilities.validate);
}

// === 線上歸屬表 ===

#[test]
fn the_two_edit_commands_are_on_their_put_routes() {
    let table: Vec<(Method, &str)> = routes().collect();
    let rows = [
        (
            Command::ChangeFieldsEdit { name: "demo".to_string(), set: map(&[("a", "1")]), unset: vec![] },
            "/changes/{name}/fields",
        ),
        (
            Command::DiscussFieldsEdit { slug: "talk".to_string(), set: map(&[("a", "1")]), unset: vec![] },
            "/discussions/{slug}/fields",
        ),
    ];
    for (cmd, pattern) in rows {
        assert_eq!(wire_coverage(&cmd), WireCoverage::Route { method: Method::Put, pattern }, "{}", cmd.name());
        assert!(table.contains(&(Method::Put, pattern)), "{pattern} is a route");
    }
}
