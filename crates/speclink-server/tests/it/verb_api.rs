//! Verb-parity endpoints (server-verb-api spec): validate/analyze as read-only
//! derived queries, DELETE change with full discard semantics, and the task
//! move endpoint — all through the Command gateway, write verbs editor-only.

use crate::common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_remote::client::Client;
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

/// 兩群組任務檔（spec「跨群組搬移重編號」的 GIVEN）。
const TASKS_TWO_GROUPS: &str =
    "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n";

struct Fixture {
    base: String,
    store: Arc<MemoryStore>,
    /// Kept live so a test can seed a third identity after the server started.
    identity: speclink_server::state::SharedIdentity,
    editor_pat: String,
    reader_pat: String,
}

/// Server over a store seeded with change `demo`; `tasks` picks its tasks.md
/// (None seeds no tasks file). An editor PAT and a reader PAT are both live.
fn fixture(tasks: Option<&str>) -> Fixture {
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
        "## Why\n\nseed\n",
    );
    if let Some(tasks) = tasks {
        uow.create(
            DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
            tasks,
        );
    }
    store.commit(uow, Vec::new()).expect("seed commit");

    let state = common::state_with(store.clone());
    let (editor_pat, _) =
        common::seed_named_pat(&state.identity, "editor@example.com", "Editor", &["demo"]);
    let (reader_pat, reader_id) =
        common::seed_named_pat(&state.identity, "reader@example.com", "Reader", &["demo"]);
    state
        .identity
        .admin_set_membership(
            &AuditActor::system_cli(),
            &reader_id,
            "demo",
            MembershipRole::Reader,
            true,
        )
        .expect("set reader role");
    let identity = state.identity.clone();
    Fixture { base: common::start(state), store, identity, editor_pat, reader_pat }
}

fn client(f: &Fixture, pat: &str) -> Client {
    Client::new(&format!("{}/api/speclink/v1/projects/demo", f.base), pat, Some("backend"))
}

fn request(method: &str, f: &Fixture, pat: &str, tail: &str) -> ureq::Request {
    ureq::request(method, &format!("{}/api/speclink/v1/projects/demo/{tail}", f.base))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
}

fn get_json(f: &Fixture, pat: &str, tail: &str) -> (Value, Option<String>) {
    let response = request("GET", f, pat, tail).call().expect("GET succeeds");
    let etag = response.header("ETag").map(str::to_string);
    (response.into_json::<Value>().expect("JSON body"), etag)
}

fn protocol_error(result: Result<ureq::Response, ureq::Error>) -> (u16, ErrorResponse) {
    match result {
        Ok(response) => panic!("expected protocol error, got {}", response.status()),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            let error = serde_json::from_str(&body)
                .unwrap_or_else(|_| panic!("expected ErrorResponse, got {body:?}"));
            (status, error)
        }
        Err(error) => panic!("transport error: {error}"),
    }
}

fn revision(f: &Fixture) -> u64 {
    f.store.snapshot(&scope()).expect("snapshot").revision().0
}

fn outbox_names(f: &Fixture) -> Vec<String> {
    f.store
        .read_outbox(&scope(), OutboxCursor(0))
        .expect("read outbox")
        .iter()
        .map(|e| e.record.name.clone())
        .collect()
}

fn artifact_content(f: &Fixture, artifact: &str) -> Option<String> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::ChangeArtifact { change: "demo".into(), artifact: artifact.into() })
        .expect("read artifact")
        .map(|d| d.content)
}

// --- 規格「validate 與 analyze 為唯讀衍生查詢端點」---

#[test]
fn validate_reports_the_engines_frozen_errors_without_advancing_revision() {
    let f = fixture(None);
    // 追加一個零操作 delta spec：fs 模式 speclink validate 對同內容的凍結錯誤項。
    let mut uow = f
        .store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed2".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "specs/auth/spec.md".into() },
        "## Notes\n\nno delta operation here\n",
    );
    f.store.commit(uow, Vec::new()).expect("seed spec");

    let before = revision(&f);
    // reader 與 editor 皆可用（唯讀衍生查詢）。
    for pat in [&f.editor_pat, &f.reader_pat] {
        let (body, etag) = get_json(&f, pat, "changes/demo/validate");
        assert_eq!(body["change"], "demo");
        assert_eq!(body["valid"], false);
        let errors: Vec<&str> =
            body["errors"].as_array().expect("errors array").iter().filter_map(|e| e.as_str()).collect();
        assert_eq!(
            errors.first().copied(),
            Some("openspec/changes/demo/specs/auth/spec.md: Parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)"),
            "the endpoint relays the engine's fs-mode frozen error verbatim"
        );
        // 同一份 delta 也是「正典尚無的 capability 缺 Purpose」——引擎的第二條
        // error 一併原樣轉載（spec spec-validation「新開 capability 的 change
        // 驗證早期檢查」）。
        assert_eq!(errors.len(), 2, "兩條 error 都到端點: {errors:?}");
        assert!(
            errors[1].contains("## Purpose") && errors[1].contains("auth"),
            "Purpose 早期檢查的訊息原樣轉載: {}",
            errors[1]
        );
        assert!(body["warnings"].as_array().expect("warnings array").is_empty());
        assert_eq!(etag.as_deref(), Some(format!("\"{before}\"").as_str()), "scope ETag attached");
    }
    assert_eq!(revision(&f), before, "a derived query never advances the scope revision");
    assert!(outbox_names(&f).is_empty(), "a derived query publishes no event");
}

#[test]
fn analyze_is_available_to_reader_and_reports_the_engines_findings() {
    let f = fixture(None);
    // 情境缺具體 Example → 引擎的 Ambiguity finding（與本地 fs 模式同 outcome）。
    let mut uow = f
        .store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed2".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "specs/auth/spec.md".into() },
        "## ADDED Requirements\n\n### Requirement: 登入保護\n\n系統 SHALL 保護登入。\n\n#### Scenario: 密碼錯誤被拒\n\n- **WHEN** 密碼錯誤\n- **THEN** 拒絕登入\n",
    );
    f.store.commit(uow, Vec::new()).expect("seed spec");

    let before = revision(&f);
    let (body, _) = get_json(&f, &f.reader_pat, "changes/demo/analyze");
    assert_eq!(body["changeId"], "demo", "the full AnalyzeReport shape, camelCase fields");
    assert_eq!(body["dimensions"].as_array().expect("dimensions").len(), 4);
    let findings = body["findings"].as_array().expect("findings");
    let amb = findings
        .iter()
        .find(|x| x["id"].as_str().unwrap_or_default().starts_with("AMB"))
        .expect("the engine's ambiguity finding is relayed");
    assert_eq!(amb["severity"], "Suggestion");
    assert_eq!(amb["location"], "specs/auth/spec.md");
    assert!(amb["summaryMsg"]["key"].is_string(), "typed msg key relayed");
    assert_eq!(revision(&f), before, "analyze never advances the scope revision");
    assert!(outbox_names(&f).is_empty(), "analyze publishes no event");
}

#[test]
fn validate_and_analyze_on_a_missing_change_are_404() {
    let f = fixture(None);
    for tail in ["changes/no-such/validate", "changes/no-such/analyze"] {
        let (status, error) = protocol_error(request("GET", &f, &f.editor_pat, tail).call());
        assert_eq!(status, 404, "{tail}");
        assert_eq!(error.reason, ErrorReason::NotFound);
        assert!(
            error.message.contains("no-such"),
            "{tail} names the missing change: {}",
            error.message
        );
    }
}

// --- 規格「DELETE change 為 discard 全語意」---

#[test]
fn delete_unstarted_change_removes_it_and_publishes_the_invalidation_event() {
    let f = fixture(Some("- [ ] 1.1 甲\n"));
    let response = request("DELETE", &f, &f.editor_pat, "changes/demo")
        .call()
        .expect("delete an unstarted change succeeds");
    assert_eq!(response.status(), 200);

    let (list, _) = get_json(&f, &f.editor_pat, "changes");
    assert!(
        list["changes"].as_array().expect("changes").iter().all(|c| c["name"] != "demo"),
        "the deleted change leaves the list"
    );
    assert_eq!(artifact_content(&f, "tasks.md"), None, "artifacts are gone");
    assert!(
        outbox_names(&f).contains(&"change-discarded".to_string()),
        "the commit publishes change-discarded so SSE subscribers get an invalidate: {:?}",
        outbox_names(&f)
    );
}

#[test]
fn delete_started_change_requires_force_and_refuses_with_zero_side_effects() {
    let f = fixture(Some("- [x] 1.1 已開工\n- [ ] 1.2 乙\n"));
    let before = revision(&f);

    let (status, error) = protocol_error(request("DELETE", &f, &f.editor_pat, "changes/demo").call());
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused, "machine-readable needs-force refusal");
    assert_eq!(revision(&f), before, "a refusal writes nothing");
    assert!(artifact_content(&f, "tasks.md").is_some(), "the change is fully preserved");

    let response = request("DELETE", &f, &f.editor_pat, "changes/demo?force=true")
        .call()
        .expect("force=true deletes the started change");
    assert_eq!(response.status(), 200);
    assert_eq!(artifact_content(&f, "tasks.md"), None, "force removes the change");
}

#[test]
fn delete_unlinks_the_promoted_source_discussion() {
    let f = fixture(None);
    let c = client(&f, &f.editor_pat);
    let slug = c.new_discussion("Auth flow", None, None).expect("new discussion").slug;
    c.discussion_conclude(&slug, "結論：做。").expect("conclude");
    let change = c.discussion_promote(&slug, Some("add-auth-change")).expect("promote").change;
    assert_eq!(change, "add-auth-change");
    let shown = c.show_discussion(&slug).expect("show discussion");
    assert!(shown.content.contains("add-auth-change"), "promote links the change");

    let response = request("DELETE", &f, &f.editor_pat, "changes/add-auth-change?force=false")
        .call()
        .expect("delete the promoted change");
    assert_eq!(response.status(), 200);

    let shown = c.show_discussion(&slug).expect("show discussion");
    assert!(
        !shown.content.contains("add-auth-change"),
        "promoted_to no longer lists the deleted change: {}",
        shown.content
    );
    assert_eq!(
        shown.info.status, "concluded",
        "an emptied promoted_to list reverts the discussion status"
    );
}

// --- 規格「任務搬移端點與重編號效果」---

#[test]
fn move_across_groups_renumbers_and_publishes_the_invalidation_event() {
    let f = fixture(Some(TASKS_TWO_GROUPS));
    let response = request("POST", &f, &f.editor_pat, "changes/demo/tasks/move")
        .send_json(json!({ "from": 1, "to": 3 }))
        .expect("move succeeds");
    assert_eq!(response.status(), 200);
    let body = response.into_json::<Value>().expect("JSON body");
    assert_eq!(body["change"], "demo");
    assert_eq!(body["description"], "2.2 甲", "the moved task's post-move description");

    assert_eq!(
        artifact_content(&f, "tasks.md").expect("tasks.md"),
        "## 1. 前段\n\n- [ ] 1.1 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 甲\n",
        "the checkbox line lands after the anchor and both groups renumber; other lines byte-identical"
    );
    assert!(
        outbox_names(&f).contains(&"task-moved".to_string()),
        "the commit publishes task-moved so SSE subscribers get an invalidate: {:?}",
        outbox_names(&f)
    );
}

#[test]
fn move_out_of_range_refuses_with_zero_side_effects() {
    let f = fixture(Some(TASKS_TWO_GROUPS));
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("POST", &f, &f.editor_pat, "changes/demo/tasks/move")
            .send_json(json!({ "from": 5, "to": 1 })),
    );
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused);
    assert!(error.message.contains("out of range"), "names the refusal: {}", error.message);
    assert_eq!(artifact_content(&f, "tasks.md").as_deref(), Some(TASKS_TWO_GROUPS));
    assert_eq!(revision(&f), before, "a refused move writes nothing");
    assert!(outbox_names(&f).is_empty(), "a refused move publishes no event");
}

// --- 規格「變更開工標記端點」---

fn meta_content(f: &Fixture) -> Option<String> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::ChangeMeta { change: "demo".into() })
        .expect("read meta")
        .map(|d| d.content)
}

#[test]
fn in_progress_first_stamp_writes_identity_publishes_event_and_advances_revision() {
    let f = fixture(Some("- [ ] 1.1 甲\n"));
    let before = revision(&f);
    let response = request("POST", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("first stamp succeeds");
    assert_eq!(response.status(), 200);
    let meta = meta_content(&f).expect("meta exists");
    assert!(
        meta.starts_with("schema: spec-driven\n"),
        "existing fields stay byte-identical: {meta}"
    );
    assert!(meta.contains("started_at: "), "started_at stamped: {meta}");
    assert!(
        meta.contains("started_by: Editor <editor@example.com>\n"),
        "started_by is the caller's authenticated identity: {meta}"
    );
    assert!(revision(&f) > before, "the stamp commit advances the scope revision");
    assert!(
        outbox_names(&f).contains(&"change-marked-in-progress".to_string()),
        "the stamp publishes change-marked-in-progress: {:?}",
        outbox_names(&f)
    );
}

#[test]
fn in_progress_repeat_and_unknown_are_http_200_with_zero_side_effects() {
    let f = fixture(None);
    let response = request("POST", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("first stamp succeeds");
    assert_eq!(response.status(), 200);
    let stamped_meta = meta_content(&f).expect("meta exists");
    let rev = revision(&f);
    let events = outbox_names(&f);

    // 重複執行：HTTP 200、首章逐字元保留、零寫入、零事件、revision 不前進。
    let response = request("POST", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("a repeat is a silent success");
    assert_eq!(response.status(), 200);
    assert_eq!(
        meta_content(&f).as_deref(),
        Some(stamped_meta.as_str()),
        "the first stamp is preserved verbatim"
    );
    assert_eq!(revision(&f), rev, "a repeat writes nothing");
    assert_eq!(outbox_names(&f), events, "a repeat publishes no event");

    // 未知 change：同樣 HTTP 200 靜默成功、零副作用。
    let response = request("POST", &f, &f.editor_pat, "changes/no-such/in-progress")
        .call()
        .expect("an unknown name is a silent success");
    assert_eq!(response.status(), 200);
    assert_eq!(revision(&f), rev, "an unknown name writes nothing");
    assert_eq!(outbox_names(&f), events, "an unknown name publishes no event");
}

// --- 規格「in-progress 標記移除端點與加入端點成鏡像」---

#[test]
fn in_progress_remove_zero_trace_removes_marker_and_publishes_event() {
    let f = fixture(None);
    request("POST", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("stamp first");
    let before = revision(&f);
    let response = request("DELETE", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("zero-trace removal succeeds");
    assert_eq!(response.status(), 200);
    assert_eq!(
        meta_content(&f).as_deref(),
        Some("schema: spec-driven\n"),
        "started_* removed, every other line byte-identical"
    );
    assert!(revision(&f) > before, "the removal commit advances the scope revision");
    assert!(
        outbox_names(&f).contains(&"change-in-progress-removed".to_string()),
        "the removal publishes change-in-progress-removed: {:?}",
        outbox_names(&f)
    );
}

#[test]
fn in_progress_remove_not_started_is_http_200_with_zero_side_effects() {
    let f = fixture(None);
    let before = revision(&f);
    let events = outbox_names(&f);
    let response = request("DELETE", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("a not-started change is an idempotent success");
    assert_eq!(response.status(), 200);
    assert_eq!(meta_content(&f).as_deref(), Some("schema: spec-driven\n"));
    assert_eq!(revision(&f), before, "an idempotent pass commits nothing");
    assert_eq!(outbox_names(&f), events, "an idempotent pass publishes no event");
}

#[test]
fn in_progress_remove_with_work_traces_is_409_with_camelcase_evidence() {
    let f = fixture(Some("- [x] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n"));
    request("POST", &f, &f.editor_pat, "changes/demo/in-progress")
        .call()
        .expect("stamp first");
    let stamped = meta_content(&f);
    let before = revision(&f);
    let events = outbox_names(&f);

    // 生 JSON 斷言 wire 形狀:證據欄位 camelCase、型別正確(D4 對外契約)。
    let (status, body) = match request("DELETE", &f, &f.editor_pat, "changes/demo/in-progress").call() {
        Err(ureq::Error::Status(status, response)) => {
            (status, response.into_string().unwrap_or_default())
        }
        other => panic!("expected a 409 status error, got {other:?}"),
    };
    assert_eq!(status, 409);
    let v: Value = serde_json::from_str(&body).expect("JSON error payload");
    assert_eq!(v["reason"], "refused");
    assert!(v["checkedTasks"].is_number(), "checkedTasks is a number: {v}");
    assert_eq!(v["checkedTasks"], 2);
    assert!(v["touchedFiles"].is_array(), "touchedFiles is an array: {v}");
    assert!(
        v["touchedFiles"].as_array().unwrap().iter().all(Value::is_string),
        "touchedFiles elements are strings: {v}"
    );

    assert_eq!(meta_content(&f), stamped, "a refusal must not touch the meta");
    assert_eq!(revision(&f), before, "a refusal writes nothing");
    assert_eq!(outbox_names(&f), events, "a refusal publishes no event");
}

#[test]
fn in_progress_remove_unknown_change_is_404() {
    let f = fixture(None);
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("DELETE", &f, &f.editor_pat, "changes/no-such/in-progress").call(),
    );
    assert_eq!(status, 404);
    assert_eq!(error.reason, ErrorReason::NotFound);
    assert_eq!(revision(&f), before, "an unknown change writes nothing");
}

// --- 規格「寫入動詞 editor 限定」---

#[test]
fn reader_delete_and_move_are_forbidden_with_intact_scope() {
    let f = fixture(Some(TASKS_TWO_GROUPS));
    let before = revision(&f);

    let (status, error) = protocol_error(request("DELETE", &f, &f.reader_pat, "changes/demo").call());
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied, "machine-readable role refusal");

    let (status, error) = protocol_error(
        request("POST", &f, &f.reader_pat, "changes/demo/tasks/move")
            .send_json(json!({ "from": 1, "to": 3 })),
    );
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);

    assert_eq!(revision(&f), before, "reader refusals write nothing");
    assert_eq!(artifact_content(&f, "tasks.md").as_deref(), Some(TASKS_TWO_GROUPS));
}

#[test]
fn handshake_capabilities_follow_the_membership_role() {
    let f = fixture(None);
    let (editor, _) = get_json(&f, &f.editor_pat, "binding");
    let (reader, _) = get_json(&f, &f.reader_pat, "binding");
    for key in ["validate", "analyze", "deleteChange", "moveTask"] {
        assert_eq!(editor["capabilities"][key], true, "editor {key}");
    }
    assert_eq!(reader["capabilities"]["validate"], true);
    assert_eq!(reader["capabilities"]["analyze"], true);
    assert_eq!(reader["capabilities"]["deleteChange"], false, "reader write verbs stay disabled");
    assert_eq!(reader["capabilities"]["moveTask"], false);
}

// --- 規格「claim 端點持久化與 ownership 衝突語意」---

/// editor fixture 的引擎身分字串:server 依 Actor 契約以「顯示名 <email>」
/// 組成,email 是 identity 唯一鍵,同名帳號因此不會被讀成同一人。
const EDITOR_IDENTITY: &str = "Editor <editor@example.com>";

/// 一台「重開機」的 server：同一個 store、全新的 AppState 與 identity。認領若
/// 只活在程序記憶體，這裡就讀不回來。
fn restart_over(f: &Fixture) -> (String, String) {
    let state = common::state_with(f.store.clone() as Arc<dyn TeamStore + Send + Sync>);
    let (pat, _) =
        common::seed_named_pat(&state.identity, "editor@example.com", "Editor", &["demo"]);
    (common::start(state), pat)
}

#[test]
fn claim_persists_the_owner_into_both_read_paths_and_survives_a_restart() {
    let f = fixture(None);
    let editor = client(&f, &f.editor_pat);

    let claimed = editor.claim("demo").expect("editor claims an unclaimed change");
    assert_eq!(claimed.claimed_by.as_deref(), Some(EDITOR_IDENTITY));

    let listed = editor.list_changes().expect("list changes");
    let demo = listed.changes.iter().find(|c| c.name == "demo").expect("demo listed");
    assert_eq!(
        demo.claimed_by.as_deref(),
        Some(EDITOR_IDENTITY),
        "the list reads the owner from meta"
    );
    let status = editor.get_change("demo").expect("single change read");
    assert_eq!(status.claimed_by.as_deref(), Some(EDITOR_IDENTITY));

    let (base, pat) = restart_over(&f);
    let after = Client::new(&format!("{base}/api/speclink/v1/projects/demo"), &pat, Some("backend"));
    let listed = after.list_changes().expect("list after restart");
    let demo = listed.changes.iter().find(|c| c.name == "demo").expect("demo listed");
    assert_eq!(demo.claimed_by.as_deref(), Some(EDITOR_IDENTITY), "the claim outlives the process");
}

#[test]
fn an_unclaimed_change_reports_no_owner() {
    let f = fixture(None);
    let editor = client(&f, &f.editor_pat);
    let listed = editor.list_changes().expect("list changes");
    let demo = listed.changes.iter().find(|c| c.name == "demo").expect("demo listed");
    assert_eq!(demo.claimed_by, None, "an unclaimed change omits the field");
    assert_eq!(editor.get_change("demo").expect("status").claimed_by, None);
}

#[test]
fn a_second_claimant_is_refused_with_the_holder_named_and_nothing_written() {
    let f = fixture(None);
    client(&f, &f.editor_pat).claim("demo").expect("first claim");
    let after_first = revision(&f);

    let (other_pat, _) =
        common::seed_named_pat(&f.identity, "other@example.com", "Other", &["demo"]);
    let error = client(&f, &other_pat)
        .claim("demo")
        .expect_err("a held change refuses the second claimant");
    assert_eq!(error.status, Some(409));
    assert_eq!(error.reason.as_deref(), Some("refused"), "reason stays inside the closed registry");
    assert!(error.message.contains("Editor"), "the refusal names the holder: {}", error.message);
    assert_eq!(revision(&f), after_first, "a refused claim writes nothing");
}

#[test]
fn a_namesake_with_a_different_account_is_refused_rather_than_read_as_the_holder() {
    let f = fixture(None);
    client(&f, &f.editor_pat).claim("demo").expect("first claim");
    let after_first = revision(&f);

    // display 不唯一——identity 只對 email 做 UNIQUE。同名的另一個帳號若被
    // 讀成「同一人重複認領」,這個動詞要防的撞工正好從這裡漏過去。
    let (namesake_pat, _) =
        common::seed_named_pat(&f.identity, "editor2@example.com", "Editor", &["demo"]);
    let error = client(&f, &namesake_pat)
        .claim("demo")
        .expect_err("a namesake is a different person, not the holder");
    assert_eq!(error.status, Some(409));
    assert_eq!(revision(&f), after_first, "the namesake's refused claim writes nothing");
}

#[test]
fn repeat_claim_by_the_same_actor_succeeds_without_writing() {
    let f = fixture(None);
    let editor = client(&f, &f.editor_pat);
    editor.claim("demo").expect("first claim");
    let after_first = revision(&f);

    let again = editor.claim("demo").expect("the same actor may re-claim");
    assert_eq!(again.claimed_by.as_deref(), Some(EDITOR_IDENTITY));
    assert_eq!(revision(&f), after_first, "an idempotent pass writes nothing");
}

#[test]
fn reader_cannot_claim_and_an_unknown_change_is_not_found() {
    let f = fixture(None);
    let before = revision(&f);

    let (status, error) =
        protocol_error(request("POST", &f, &f.reader_pat, "changes/demo/claim").call());
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);

    let (status, error) =
        protocol_error(request("POST", &f, &f.editor_pat, "changes/ghost/claim").call());
    assert_eq!(status, 404);
    assert_eq!(error.reason, ErrorReason::NotFound);

    assert_eq!(revision(&f), before, "neither refusal writes");
}
