//! The quality stations, the branch and the landing on the wire (change
//! station-status-and-landing; specs client-protocol「線上變更回應的關卡欄位」
//! 「落地回報的請求與回應」, server-verb-api「變更回應攜帶關卡狀態與落地記錄」,
//! server-event-stream「審查與驗證事件歸到變更」). Every scenario goes through
//! the door on the in-memory store port behind the shared recorder.

use serde_json::{json, Value};
use speclink_core::command::Command;
use speclink_host::http::{routes, wire_coverage, Method, WireCoverage};
use speclink_host::kit::{
    AfterCommit, BeforeCommand, Decision, Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, RepoEntry,
};
use speclink_protocol::events::InvalidationScope;
use speclink_store_port::{MemoryStorePort, ScopeId};
use std::sync::{Arc, Mutex};

mod common;
use common::door::*;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
const DONE_TASKS: &str = "## 1. Section\n\n- [x] 1.1 first task\n";
const REVIEW_TICKET: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n";
const VERIFY_TICKET: &str = "# Verify — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n";
/// A review stamp of 3 rounds over 12 tasks and two files, with a commit.
const REVIEW_STAMP: &str = "reviewed_at: 2026-11-02T10:00:00+08:00\nreviewed_by: Rev <r@example.com>\n\
reviewed_with: claude\nreviewed_tasks_total: 12\nreviewed_rounds: 3\nreviewed_commit: \"0123456789ab\"\n\
reviewed_scope:\n  - path: src/a.rs\n    hash: aa\n  - path: src/b.rs\n    hash: bb\n";
const VERIFY_STAMP: &str = "verified_at: 2026-11-03T10:00:00+08:00\nverified_tasks_total: 12\nverified_rounds: 1\n\
verified_scope:\n  - path: src/a.rs\n    hash: aa\n";

fn keys(value: &Value) -> Vec<&str> {
    value.as_object().unwrap_or_else(|| panic!("an object: {value}")).keys().map(String::as_str).collect()
}

fn meta_with(parts: &[&str]) -> String {
    format!("{META}{}", parts.concat())
}

// --- 線上變更回應的關卡欄位 ---

#[test]
fn the_list_reports_the_three_states() {
    // Scenario「清單回填三態」。
    let door = door();
    seed(
        &door,
        &[
            ("changes/a-none/.meta.yaml", META),
            ("changes/b-ticket/.meta.yaml", META),
            ("changes/b-ticket/review.md", REVIEW_TICKET),
            ("changes/c-stamped/.meta.yaml", &meta_with(&[REVIEW_STAMP])),
        ],
    );
    let states: Vec<Value> = get(&door, "changes")["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["review"]["state"].clone())
        .collect();
    assert_eq!(states, [json!("none"), json!("inProgress"), json!("stamped")]);
}

#[test]
fn a_ticketed_item_carries_the_state_only() {
    // Scenario「審查中的清單項」。
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META), ("changes/demo/review.md", REVIEW_TICKET)]);
    let demo = listed(&door, "demo");
    assert_eq!(demo["review"], json!({ "state": "inProgress" }));
    assert_eq!(demo["verify"], json!({ "state": "none" }));
}

#[test]
fn a_stamped_item_carries_the_stamp_without_its_scope() {
    // Scenario「已蓋章的清單項不帶 scope」。
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", &meta_with(&[REVIEW_STAMP]))]);
    let review = listed(&door, "demo")["review"].clone();
    assert_eq!(
        review,
        json!({
            "state": "stamped",
            "stampedAt": "2026-11-02",
            "stampedBy": "Rev <r@example.com>",
            "stampedWith": "claude",
            "rounds": 3,
            "tasksTotal": 12,
            "commit": "0123456789ab",
        })
    );
    assert!(!keys(&review).contains(&"scope"));
    assert!(review["rounds"].is_u64() && review["tasksTotal"].is_u64() && review["stampedAt"].is_string());
}

#[test]
fn the_single_change_response_adds_the_scope() {
    // Scenario「單筆回應帶 scope」。
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", &meta_with(&[REVIEW_STAMP]))]);
    let change = get(&door, "changes/demo");
    assert_eq!(change["review"]["state"], "stamped");
    assert_eq!(change["review"]["commit"], "0123456789ab");
    assert_eq!(
        change["review"]["scope"],
        json!([{ "path": "src/a.rs", "hash": "aa" }, { "path": "src/b.rs", "hash": "bb" }])
    );
    assert_eq!(change["verify"], json!({ "state": "none" }));
}

#[test]
fn the_two_stations_are_judged_apart() {
    // Scenario「兩個關卡各自獨立」。
    let door = door();
    seed(
        &door,
        &[("changes/demo/.meta.yaml", &meta_with(&[REVIEW_STAMP])), ("changes/demo/verify.md", VERIFY_TICKET)],
    );
    let demo = listed(&door, "demo");
    assert_eq!(demo["review"]["state"], "stamped");
    assert_eq!(demo["verify"], json!({ "state": "inProgress" }));
    let change = get(&door, "changes/demo");
    assert_eq!((&change["review"]["state"], &change["verify"]["state"]), (&json!("stamped"), &json!("inProgress")));
}

#[test]
fn a_change_with_a_broken_data_file_has_no_station_objects() {
    // Scenario「壞 metadata 的變更」。
    let door = door();
    seed(
        &door,
        &[
            ("changes/broken/.meta.yaml", ": : :\n\t bad yaml [unclosed\n"),
            ("changes/broken/review.md", REVIEW_TICKET),
            ("changes/fine/.meta.yaml", META),
        ],
    );
    let broken = listed(&door, "broken");
    assert!(broken["metaError"].is_string(), "{broken}");
    for key in ["review", "verify", "branch", "landedCommit"] {
        assert!(broken.get(key).is_none(), "{key}: {broken}");
    }
    assert_eq!(listed(&door, "fine")["review"], json!({ "state": "none" }));
}

#[test]
fn the_archived_list_reports_each_stations_outcome() {
    // Scenario「已封存的結局」與「蓋章後重開工單再封存」：有工單 → notPassed（章還在也
    // 一樣）；否則有章 → stamped；否則 none。
    let door = door();
    seed(
        &door,
        &[
            ("archive/changes/2026-11-01-both/.meta.yaml", &meta_with(&[REVIEW_STAMP, VERIFY_STAMP])),
            ("archive/changes/2026-11-02-carried/.meta.yaml", &meta_with(&[REVIEW_STAMP])),
            ("archive/changes/2026-11-02-carried/verify.md", VERIFY_TICKET),
            ("archive/changes/2026-11-03-plain/.meta.yaml", META),
            ("archive/changes/2026-11-04-reopened/.meta.yaml", &meta_with(&[REVIEW_STAMP])),
            ("archive/changes/2026-11-04-reopened/review.md", REVIEW_TICKET),
        ],
    );
    let archived = get(&door, "archived");
    let outcome = |name: &str| {
        let item = archived["archived"].as_array().unwrap().iter().find(|i| i["name"] == name).unwrap();
        (item["reviewOutcome"].clone(), item["verifyOutcome"].clone())
    };
    assert_eq!(outcome("both"), (json!("stamped"), json!("stamped")));
    assert_eq!(outcome("carried"), (json!("stamped"), json!("notPassed")));
    assert_eq!(outcome("plain"), (json!("none"), json!("none")));
    assert_eq!(outcome("reopened"), (json!("notPassed"), json!("none")));
}

#[test]
fn the_station_fields_cost_no_store_call() {
    // Scenario「回填不多發儲存呼叫」：次數是加上這些欄位之前量到的值。
    let door = door();
    seed(
        &door,
        &[
            ("config.yaml", "schema: spec-driven\n"),
            ("changes/a/.meta.yaml", &meta_with(&[REVIEW_STAMP])),
            ("changes/a/tasks.md", DONE_TASKS),
            ("changes/b/.meta.yaml", META),
            ("changes/b/review.md", REVIEW_TICKET),
            ("changes/b/verify.md", VERIFY_TICKET),
            ("archive/changes/2026-11-02-z/.meta.yaml", &meta_with(&[REVIEW_STAMP])),
            ("archive/changes/2026-11-02-z/verify.md", VERIFY_TICKET),
        ],
    );
    for (tail, lists, reads) in [("changes", 2, 3), ("changes/b", 3, 3), ("archived", 1, 1)] {
        get(&door, tail);
        let calls = door.store.take();
        assert_eq!(calls.lists.len(), lists, "{tail}: lists {:?}", calls.lists);
        assert_eq!(calls.reads.len(), reads, "{tail}: reads {:?}", calls.read_keys());
        assert!(calls.commits.is_empty(), "{tail}");
    }
}

#[test]
fn the_station_and_landing_fields_come_from_the_callers_scope_only() {
    // sharp-edges 稽核：同名的變更在兩個範圍，回填只讀請求所在範圍的文件。
    let repo = |key: &str, scope: &str| RepoEntry {
        key: key.to_string(),
        name: key.to_uppercase(),
        scope: ScopeId::new(scope).unwrap(),
        can_write: true,
        can_write_policy: true,
    };
    let projects =
        vec![ProjectEntry { key: "alpha".to_string(), name: "Project alpha".to_string(), repos: vec![repo("web", "s1"), repo("api", "s2")] }];
    let options = HostKitOptions {
        scopes: Some(Box::new(move |_identity: &Identity| Ok(projects.clone()))),
        ..Default::default()
    };
    let store = Arc::new(common::Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::with_options(store.clone(), Hooks::default(), options).expect("the options name no unknown skill");
    let door = Door { kit, store };
    let landed = "landed_commit: \"89abcdef0123\"\nlanded_at: 2026-11-04T09:00:00+08:00\nbranch: feature/x\n";
    seed(&door, &[("changes/demo/.meta.yaml", &meta_with(&[REVIEW_STAMP, landed])), ("changes/demo/verify.md", VERIFY_TICKET)]);
    common::write_in(&door.store.inner, &ScopeId::new("s2").unwrap(), &[("changes/demo/.meta.yaml", META)]);

    let in_repo = |tail: &str, repo: &str| {
        let mut req = request(Method::Get, tail, None);
        req.headers.push(("x-speclink-repo".to_string(), repo.to_string()));
        let response = answer(&door, &req);
        assert_eq!(response.status, 200, "{repo} {tail}: {:?}", String::from_utf8_lossy(&response.body));
        body_of(&response)
    };
    for other in [in_repo("changes", "api")["changes"][0].clone(), in_repo("changes/demo", "api")] {
        assert_eq!(other["review"], json!({ "state": "none" }), "{other}");
        assert_eq!(other["verify"], json!({ "state": "none" }), "{other}");
        for key in ["branch", "landedCommit", "landedAt"] {
            assert!(other.get(key).is_none(), "{key}: {other}");
        }
    }
    let own = in_repo("changes/demo", "web");
    assert_eq!((&own["review"]["state"], &own["verify"]["state"]), (&json!("stamped"), &json!("inProgress")));
    assert_eq!(own["landedCommit"], "89abcdef0123");
}

// --- 落地回報 ---

#[test]
fn a_landing_report_is_recorded_and_read_back() {
    // Scenarios「回報成功」「落地回報端點」。
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    let before = etag_of(&answer(&door, &request(Method::Get, "changes", None)));

    let response = post(&door, "changes/demo/landed", json!({ "commit": SHA, "branch": "speclink/demo" }));
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert_eq!(body_of(&response), json!({ "change": "demo", "commit": SHA, "branch": "speclink/demo" }));
    assert!(etag_of(&response) > before, "a new scope version");

    let change = get(&door, "changes/demo");
    assert_eq!(change["landedCommit"], SHA);
    assert_eq!(change["landedBy"], "Alice");
    assert_eq!(change["branch"], "speclink/demo");
    assert!(change["landedAt"].as_str().is_some_and(|at| at.contains('T')), "RFC 3339: {change}");

    let item = listed(&door, "demo");
    assert_eq!((&item["branch"], &item["landedCommit"]), (&json!("speclink/demo"), &json!(SHA)));
    assert!(item.get("landedAt").is_none() && item.get("landedBy").is_none(), "the list keeps to two fields");
}

#[test]
fn a_landing_report_without_a_branch_answers_without_one() {
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    let response = post(&door, "changes/demo/landed", json!({ "commit": "0123456" }));
    assert_eq!(body_of(&response), json!({ "change": "demo", "commit": "0123456" }));
    let change = get(&door, "changes/demo");
    assert!(change.get("branch").is_none(), "{change}");
}

#[test]
fn a_change_without_records_has_no_branch_or_landing_key() {
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    for value in [listed(&door, "demo"), get(&door, "changes/demo")] {
        for key in ["branch", "landedCommit", "landedAt", "landedBy"] {
            assert!(value.get(key).is_none(), "{key}: {value}");
        }
    }
}

#[test]
fn a_landing_report_is_one_commit_with_one_event_and_the_hooks_see_it() {
    // Scenario「經儲存接口是一次提交並帶事件」。
    let asked: Arc<Mutex<Vec<(String, Value)>>> = Arc::default();
    let watched: Arc<Mutex<Vec<(String, Vec<String>)>>> = Arc::default();
    let (before, after) = (asked.clone(), watched.clone());
    let hooks = Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand| {
            before.lock().unwrap().push((info.command.to_string(), info.params.clone()));
            Decision::Allow
        })),
        after_commit: Some(Box::new(move |info: &AfterCommit| {
            let events = info.events.iter().map(|e| e.name().to_string()).collect();
            after.lock().unwrap().push((info.command.to_string(), events));
        })),
    };
    let door = door_with(true, hooks);
    seed(&door, &[("changes/demo/.meta.yaml", META)]);

    assert_eq!(post(&door, "changes/demo/landed", json!({ "commit": SHA })).status, 200);

    let calls = door.store.take();
    let [commit] = &calls.commits[..] else { panic!("exactly one commit, got {}", calls.commits.len()) };
    let events: Vec<(&str, &Value)> = commit.events().iter().map(|e| (e.name(), e.payload())).collect();
    assert_eq!(events, [("change-landed", &json!({ "change": "demo", "commit": SHA }))]);
    assert_eq!(
        *asked.lock().unwrap(),
        [("change-landed".to_string(), json!({ "name": "demo", "commit": SHA, "branch": null }))]
    );
    assert_eq!(*watched.lock().unwrap(), [("change-landed".to_string(), vec!["change-landed".to_string()])]);
}

#[test]
fn a_hook_can_refuse_a_landing_report() {
    // Scenario「掛鉤可以拒絕回報」。
    let hooks = Hooks {
        before_command: Some(Box::new(|info: &BeforeCommand| match info.command == "change-landed" {
            true => Decision::Deny { reason: "the commit is not on main".to_string() },
            false => Decision::Allow,
        })),
        after_commit: None,
    };
    let door = door_with(true, hooks);
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    let response = post(&door, "changes/demo/landed", json!({ "commit": SHA }));
    assert_eq!(error(&response), (409, "refused".to_string()));
    assert_eq!(content(&door, "changes/demo/.meta.yaml").as_deref(), Some(META));
}

#[test]
fn a_caller_who_cannot_write_cannot_report_a_landing() {
    // Scenarios「沒有寫入權限」「reader 不能回報」。
    let door = door_with(false, Hooks::default());
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    let response = post(&door, "changes/demo/landed", json!({ "commit": SHA }));
    assert_eq!(error(&response), (403, "permission_denied".to_string()));
    assert_eq!(content(&door, "changes/demo/.meta.yaml").as_deref(), Some(META));
    assert!(door.store.calls().commits.is_empty());
}

#[test]
fn a_malformed_report_is_400_and_an_unknown_change_is_404() {
    // Scenario「commit 形狀不合」。
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META)]);
    for body in [json!({ "commit": "zzz" }), json!({ "commit": SHA, "branch": "has space" }), json!({ "branch": "x" })] {
        let response = post(&door, "changes/demo/landed", body.clone());
        assert_eq!(error(&response), (400, "invalid_argument".to_string()), "{body}");
    }
    assert_eq!(error(&post(&door, "changes/ghost/landed", json!({ "commit": SHA }))), (404, "not_found".to_string()));
    assert_eq!(content(&door, "changes/demo/.meta.yaml").as_deref(), Some(META));
    assert!(door.store.calls().commits.is_empty());
}

// --- 蓋章的 headCommit 與開工的 branch ---

/// Change demo with every task done and one clean round on `station`.
fn ready_to_stamp(door: &Door, station: &str) {
    seed(door, &[("changes/demo/.meta.yaml", META), ("changes/demo/tasks.md", DONE_TASKS)]);
    let round = post(door, &format!("changes/demo/{station}/rounds"), json!({ "content": "**Scope**: src/a.rs\n" }));
    assert_eq!(round.status, 200, "{:?}", String::from_utf8_lossy(&round.body));
}

#[test]
fn a_stamp_request_with_a_head_commit_records_it() {
    // Scenario「蓋章請求帶 headCommit」，兩個關卡。
    for station in ["review", "verify"] {
        let door = door();
        ready_to_stamp(&door, station);
        assert_eq!(get(&door, "changes/demo")[station], json!({ "state": "inProgress" }), "{station}");
        let stamp = json!({ "scope": [{ "path": "src/a.rs", "hash": "aa" }], "headCommit": "0123456789ab" });
        let response = post(&door, &format!("changes/demo/{station}/stamp"), stamp);
        assert_eq!(response.status, 200, "{station}: {:?}", String::from_utf8_lossy(&response.body));
        let change = get(&door, "changes/demo");
        assert_eq!(change[station]["state"], "stamped", "{station}");
        assert_eq!(change[station]["commit"], "0123456789ab", "{station}");
        assert_eq!(change[station]["stampedBy"], "Alice", "{station}");
        assert_eq!(change[station]["scope"], json!([{ "path": "src/a.rs", "hash": "aa" }]), "{station}");
    }
}

#[test]
fn a_stamp_request_without_a_head_commit_stamps_without_one() {
    // Scenario「蓋章請求沒有 headCommit」：舊版用戶端的請求照常成功。
    let door = door();
    ready_to_stamp(&door, "review");
    let response = post(&door, "changes/demo/review/stamp", json!({ "scope": [{ "path": "src/a.rs", "hash": "aa" }] }));
    assert_eq!(response.status, 200);
    let review = get(&door, "changes/demo")["review"].clone();
    assert_eq!(review["state"], "stamped");
    assert!(review.get("commit").is_none(), "{review}");
}

#[test]
fn a_stamp_request_with_a_malformed_head_commit_is_400_and_stamps_nothing() {
    let door = door();
    ready_to_stamp(&door, "verify");
    let stamp = json!({ "scope": [{ "path": "src/a.rs", "hash": "aa" }], "headCommit": "not-a-sha" });
    assert_eq!(error(&post(&door, "changes/demo/verify/stamp", stamp)), (400, "invalid_argument".to_string()));
    assert_eq!(get(&door, "changes/demo")["verify"], json!({ "state": "inProgress" }));
}

#[test]
fn an_in_progress_request_can_carry_the_branch() {
    let door = door();
    seed(&door, &[("changes/demo/.meta.yaml", META), ("changes/old/.meta.yaml", META)]);
    assert_eq!(post(&door, "changes/demo/in-progress", json!({ "branch": "功能/登入" })).status, 200);
    assert_eq!(get(&door, "changes/demo")["branch"], "功能/登入");
    assert_eq!(listed(&door, "demo")["branch"], "功能/登入");

    // An older client sends no body at all: the mark lands, no branch.
    assert_eq!(answer(&door, &request(Method::Post, "changes/old/in-progress", None)).status, 200);
    let old = get(&door, "changes/old");
    assert!(old["startedAt"].is_string() && old.get("branch").is_none(), "{old}");

    let bad = post(&door, "changes/demo/in-progress", json!({ "branch": "has space" }));
    assert_eq!(error(&bad), (400, "invalid_argument".to_string()));
}

// --- 線上歸屬與事件分類 ---

#[test]
fn the_landing_command_has_its_route() {
    let landed = Command::ChangeLanded { name: "demo".to_string(), commit: SHA.to_string(), branch: None };
    assert_eq!(wire_coverage(&landed), WireCoverage::Route { method: Method::Post, pattern: "/changes/{name}/landed" });
    assert!(routes().any(|(method, pattern)| method == Method::Post && pattern == "/changes/{name}/landed"));
}

#[test]
fn the_station_and_landing_events_point_at_their_change() {
    // Scenarios「蓋章事件歸到該變更」「新增一輪與放棄同樣歸類」「落地事件歸到該變更」。
    use speclink_host::commit::event_resource;
    for name in [
        "review-round-added",
        "review-stamped",
        "review-discarded",
        "verify-round-added",
        "verify-stamped",
        "verify-discarded",
        "change-landed",
    ] {
        let payload = json!({ "change": "demo", "round": 1, "commit": SHA });
        assert_eq!(event_resource(name, &payload), (InvalidationScope::Change, "demo".to_string()), "{name}");
    }
}

#[test]
fn the_other_events_keep_their_resource() {
    // Scenario「其他未知事件維持 unknown」，以及既有的分類不變。
    use speclink_host::commit::event_resource;
    let unknown = (InvalidationScope::Unknown("unknown".to_string()), String::new());
    assert_eq!(event_resource("board-order-updated", &json!({})), unknown);
    assert_eq!(event_resource("widget-frobbed", &json!({ "change": "demo" })), unknown);
    assert_eq!(
        event_resource("task-completed", &json!({ "change": "demo" })),
        (InvalidationScope::Change, "demo".to_string())
    );
    assert_eq!(
        event_resource("change-archived", &json!({ "change": "demo" })),
        (InvalidationScope::Spec, "demo".to_string())
    );
    assert_eq!(
        event_resource("discussion-sealed", &json!({ "slug": "topic", "change": "demo" })),
        (InvalidationScope::Discussion, "topic".to_string())
    );
}

#[test]
fn the_door_events_of_a_stamp_and_a_landing_are_classified_as_the_change() {
    // The events the store received, through the one classification.
    use speclink_host::commit::event_resource;
    let door = door();
    ready_to_stamp(&door, "review");
    post(&door, "changes/demo/review/stamp", json!({ "scope": [{ "path": "src/a.rs", "hash": "aa" }] }));
    post(&door, "changes/demo/landed", json!({ "commit": SHA }));
    let resources: Vec<(String, InvalidationScope, String)> = door
        .store
        .take()
        .commits
        .iter()
        .flat_map(|commit| commit.events().to_vec())
        .map(|event| {
            let (scope, id) = event_resource(event.name(), event.payload());
            (event.name().to_string(), scope, id)
        })
        .collect();
    let change = |name: &str| (name.to_string(), InvalidationScope::Change, "demo".to_string());
    assert_eq!(resources, [change("review-round-added"), change("review-stamped"), change("change-landed")]);
}

// --- 舊版用戶端 ---

/// The list item, the single change and the archived item as a client built
/// before these fields declared them: the fields that client reads, with the
/// same serde rules (camelCase, unknown fields not denied).
mod before {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChangeSummary {
        pub name: String,
        #[serde(default)]
        pub status: String,
        #[serde(default)]
        pub total_tasks: usize,
        #[serde(default)]
        pub started_at: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ListChangesResponse {
        pub changes: Vec<ChangeSummary>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChangeStatus {
        pub change_name: String,
        pub schema_name: String,
        pub is_complete: bool,
        #[serde(default)]
        pub started_at: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ArchivedItem {
        pub dated_name: String,
        pub name: String,
        #[serde(default)]
        pub spec_count: usize,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ArchivedListResponse {
        pub archived: Vec<ArchivedItem>,
    }
}

#[test]
fn a_client_from_before_these_fields_reads_the_three_responses() {
    // Scenario「舊版用戶端解析新回應」：每一個新欄位都在回應裡。
    let door = door();
    let full = meta_with(&[
        "started_at: 2026-11-01T09:00:00+08:00\nbranch: feature/x\n",
        REVIEW_STAMP,
        VERIFY_STAMP,
        "landed_commit: \"89abcdef0123\"\nlanded_at: 2026-11-04T09:00:00+08:00\nlanded_by: Ann\n",
    ]);
    seed(
        &door,
        &[
            ("changes/demo/.meta.yaml", &full),
            ("changes/demo/tasks.md", DONE_TASKS),
            ("archive/changes/2026-11-01-old/.meta.yaml", &full),
            ("archive/changes/2026-11-01-old/specs/auth/spec.md", "## Purpose\n"),
        ],
    );
    let list = get(&door, "changes");
    for key in ["review", "verify", "branch", "landedCommit"] {
        assert!(list["changes"][0].get(key).is_some(), "the new response has {key}");
    }
    let parsed: before::ListChangesResponse = serde_json::from_value(list).expect("the list parses");
    let demo = &parsed.changes[0];
    assert_eq!((demo.name.as_str(), demo.status.as_str(), demo.total_tasks), ("demo", "done", 1));
    assert_eq!(demo.started_at.as_deref(), Some("2026-11-01"));

    let single = get(&door, "changes/demo");
    for key in ["review", "verify", "branch", "landedCommit", "landedAt", "landedBy"] {
        assert!(single.get(key).is_some(), "the new response has {key}");
    }
    assert!(single["review"].get("scope").is_some());
    let parsed: before::ChangeStatus = serde_json::from_value(single).expect("the single change parses");
    assert_eq!((parsed.change_name.as_str(), parsed.schema_name.as_str()), ("demo", "spec-driven"));
    assert!(!parsed.is_complete);
    assert_eq!(parsed.started_at.as_deref(), Some("2026-11-01"));

    let archived = get(&door, "archived");
    for key in ["reviewOutcome", "verifyOutcome"] {
        assert!(archived["archived"][0].get(key).is_some(), "the new response has {key}");
    }
    let parsed: before::ArchivedListResponse = serde_json::from_value(archived).expect("the archived list parses");
    let old = &parsed.archived[0];
    assert_eq!((old.dated_name.as_str(), old.name.as_str(), old.spec_count), ("2026-11-01-old", "old", 1));
}
