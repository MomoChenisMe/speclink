//! Execution-order endpoints (server-verb-api「plan 唯讀衍生查詢端點」「變更依賴
//! 寫入端點」): GET /plan runs the engine plan over the scope with the board
//! resource as its rank source, POST /changes/{name}/depends runs the engine's
//! dependency write — both through the Command gateway, the write editor-only,
//! every refusal a `refused` 409 carrying the engine's line.

use crate::common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

struct Fixture {
    base: String,
    store: Arc<MemoryStore>,
    editor_pat: String,
    reader_pat: String,
}

fn meta(created: &str, depends_on: Option<&str>) -> String {
    let mut text = format!("schema: spec-driven\ncreated: {created}\n");
    if let Some(deps) = depends_on {
        text.push_str(&format!("depends_on: {deps}\n"));
    }
    text
}

/// Write `docs` into the demo scope in one commit (creating or updating).
fn seed(store: &MemoryStore, docs: &[(DocumentId, String)]) {
    let snapshot = store.snapshot(&scope()).expect("snapshot");
    let current: Vec<_> = docs
        .iter()
        .map(|(id, _)| snapshot.read(id).expect("read").map(|d| d.revision))
        .collect();
    drop(snapshot);
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    for ((id, content), revision) in docs.iter().zip(current) {
        match revision {
            Some(rev) => uow.update(id.clone(), content, rev),
            None => uow.create(id.clone(), content),
        }
    }
    store.commit(uow, Vec::new()).expect("seed commit");
}

fn change_meta(name: &str) -> DocumentId {
    DocumentId::ChangeMeta { change: name.into() }
}

/// Server over a store seeded with `changes` (name, meta text); an editor PAT
/// and a reader PAT are both live.
fn fixture(changes: &[(&str, String)]) -> Fixture {
    let store = Arc::new(MemoryStore::new());
    let docs: Vec<_> = changes
        .iter()
        .map(|(name, text)| (change_meta(name), text.clone()))
        .collect();
    seed(&store, &docs);

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
    Fixture { base: common::start(state), store, editor_pat, reader_pat }
}

fn request(method: &str, f: &Fixture, pat: &str, tail: &str) -> ureq::Request {
    ureq::request(method, &format!("{}/api/speclink/v1/projects/demo/{tail}", f.base))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
}

fn get_plan(f: &Fixture, pat: &str) -> (Value, String) {
    let response = request("GET", f, pat, "plan").call().expect("GET /plan succeeds");
    let etag = response.header("ETag").expect("ETag present").to_string();
    (response.into_json::<Value>().expect("JSON body"), etag)
}

fn post_depends(
    f: &Fixture,
    pat: &str,
    change: &str,
    on: &[&str],
    remove: bool,
) -> Result<ureq::Response, ureq::Error> {
    request("POST", f, pat, &format!("changes/{change}/depends"))
        .send_json(json!({ "on": on, "remove": remove }))
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

fn stored(f: &Fixture, id: &DocumentId) -> Option<(String, u64)> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(id)
        .expect("read")
        .map(|d| (d.content, d.revision.0))
}

fn order(plan: &Value) -> Vec<String> {
    plan["changes"]
        .as_array()
        .expect("changes array")
        .iter()
        .map(|c| c["name"].as_str().expect("name").to_string())
        .collect()
}

// --- 規格「plan 唯讀衍生查詢端點」---

#[test]
fn reader_reads_the_plan_with_the_scope_etag_and_nothing_is_written() {
    // Scenario「reader 可讀 plan」。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", None)),
        ("add-b", meta("2026-09-02", Some("add-a"))),
    ]);
    let before = revision(&f);
    let events = outbox_names(&f);

    let (plan, etag) = get_plan(&f, &f.reader_pat);
    assert_eq!(order(&plan), ["add-a", "add-b"]);
    assert_eq!(plan["changes"][1]["blockedBy"], json!(["add-a"]));
    assert_eq!(plan["changes"][1]["dependsOn"], json!(["add-a"]));
    assert_eq!(plan["changes"][1]["wave"], 2);
    assert_eq!(plan["changes"][1]["stage"], "proposed");
    assert_eq!(plan["next"], "add-a");
    assert_eq!(plan["skipped"], json!([]));
    assert_eq!(etag, format!("\"{before}\""), "the ETag is the scope revision");
    assert_eq!(revision(&f), before, "a derived query never advances the revision");
    assert_eq!(outbox_names(&f), events, "a derived query publishes no event");

    let (editor_view, _) = get_plan(&f, &f.editor_pat);
    assert_eq!(editor_view, plan, "editors read the same plan");
}

#[test]
fn plan_ranks_come_from_the_board_resource() {
    // Scenario「rank 來自 board resource」：無 board 時依 created，有 rank 圖
    // 時依 rank 字典序。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", None)),
        ("add-b", meta("2026-09-02", None)),
    ]);
    let (unranked, _) = get_plan(&f, &f.reader_pat);
    assert_eq!(order(&unranked), ["add-a", "add-b"], "no board resource: created order");

    seed(
        &f.store,
        &[(
            DocumentId::BoardOrder,
            r#"{"changes":{"add-b":"b","add-a":"f"},"discussions":{}}"#.to_string(),
        )],
    );
    let (ranked, _) = get_plan(&f, &f.reader_pat);
    assert_eq!(order(&ranked), ["add-b", "add-a"]);
    assert_eq!(ranked["waves"], json!([{ "index": 1, "changes": ["add-b", "add-a"] }]));
}

#[test]
fn unusable_board_content_reads_as_no_ranks_and_is_never_rewritten() {
    // Scenario「壞 board 內容視為空圖」與 remote-board-order「plan 端點寬鬆讀取」。
    let f = fixture(&[
        ("add-a", meta("2026-09-02", None)),
        ("add-b", meta("2026-09-01", None)),
    ]);
    seed(
        &f.store,
        &[(DocumentId::BoardOrder, r#"{"changes":{"add-a":"n"},"discussions":{}}"#.to_string())],
    );
    let board = stored(&f, &DocumentId::BoardOrder);
    let (ranked, _) = get_plan(&f, &f.reader_pat);
    assert_eq!(order(&ranked), ["add-a", "add-b"], "add-a's rank n places it first");
    assert_eq!(stored(&f, &DocumentId::BoardOrder), board, "the plan never writes the board");

    seed(&f.store, &[(DocumentId::BoardOrder, "not json".to_string())]);
    let board = stored(&f, &DocumentId::BoardOrder);
    let before = revision(&f);
    let (unranked, _) = get_plan(&f, &f.reader_pat);
    assert_eq!(order(&unranked), ["add-b", "add-a"], "bad content: every change unranked");
    assert_eq!(stored(&f, &DocumentId::BoardOrder), board, "bad content is left as it is");
    assert_eq!(revision(&f), before);
}

#[test]
fn a_dependency_cycle_is_a_refused_409_with_the_engine_line() {
    // Scenario「成環回 409」。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", Some("add-b"))),
        ("add-b", meta("2026-09-01", Some("add-a"))),
    ]);
    let before = revision(&f);
    let (status, error) = protocol_error(request("GET", &f, &f.reader_pat, "plan").call());
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused);
    assert_eq!(error.message, "dependency cycle: add-a -> add-b -> add-a");
    assert_eq!(revision(&f), before);
}

// --- 規格「變更依賴寫入端點」---

#[test]
fn an_editor_declares_a_prerequisite_and_the_commit_publishes_an_event() {
    // Scenario「editor 寫入前置」。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", None)),
        ("add-b", meta("2026-09-02", None)),
    ]);
    let before = revision(&f);
    let response = post_depends(&f, &f.editor_pat, "add-b", &["add-a"], false)
        .expect("editor POST succeeds");
    assert_eq!(response.status(), 200);
    let etag = response.header("ETag").expect("ETag present").to_string();
    let body = response.into_json::<Value>().expect("JSON body");
    assert_eq!(body, json!({ "change": "add-b", "dependsOn": ["add-a"] }));

    let (text, _) = stored(&f, &change_meta("add-b")).expect("meta present");
    assert_eq!(text, format!("{}depends_on: add-a\n", meta("2026-09-02", None)));
    let after = revision(&f);
    assert!(after > before, "the write advances the scope revision");
    assert_eq!(etag, format!("\"{after}\""), "the response carries the post-commit ETag");
    assert_eq!(outbox_names(&f).last().map(String::as_str), Some("change-depends-changed"));

    let (plan, _) = get_plan(&f, &f.reader_pat);
    assert_eq!(plan["changes"][1]["blockedBy"], json!(["add-a"]), "plan sees the new edge");
}

#[test]
fn a_reader_cannot_declare_prerequisites() {
    // Scenario「reader 被拒」。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", None)),
        ("add-b", meta("2026-09-02", None)),
    ]);
    let meta_before = stored(&f, &change_meta("add-b"));
    let before = revision(&f);
    let (status, error) =
        protocol_error(post_depends(&f, &f.reader_pat, "add-b", &["add-a"], false));
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
    assert_eq!(stored(&f, &change_meta("add-b")), meta_before);
    assert_eq!(revision(&f), before);
}

#[test]
fn an_unknown_change_is_a_404() {
    // Scenario「不存在的 change」。
    let f = fixture(&[("add-a", meta("2026-09-01", None))]);
    let before = revision(&f);
    let (status, error) =
        protocol_error(post_depends(&f, &f.editor_pat, "ghost", &["add-a"], false));
    assert_eq!(status, 404);
    assert_eq!(error.reason, ErrorReason::NotFound);
    assert_eq!(revision(&f), before);
}

#[test]
fn guard_refusals_are_refused_409s_with_the_engine_line_and_write_nothing() {
    // Scenario「守門失敗 409」：自依賴、已封存、成環；另補需求內文列出的目標不存在。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", Some("add-b"))),
        ("add-b", meta("2026-09-02", None)),
    ]);
    seed(
        &f.store,
        &[(
            DocumentId::ArchivedChange {
                change: "2026-08-01-old-change".into(),
                doc: ".openspec.yaml".into(),
            },
            "schema: spec-driven\ncreated: 2026-07-01\n".to_string(),
        )],
    );
    let meta_before = stored(&f, &change_meta("add-b"));
    let before = revision(&f);
    let events = outbox_names(&f);

    for (on, message) in [
        ("add-b", "'add-b' cannot depend on itself"),
        ("ghost", "cannot depend on 'ghost': no active change with that name"),
        ("old-change", "cannot depend on 'old-change': it is already archived"),
        ("add-a", "dependency cycle: add-b -> add-a -> add-b"),
    ] {
        let (status, error) =
            protocol_error(post_depends(&f, &f.editor_pat, "add-b", &[on], false));
        assert_eq!(status, 409, "--on {on}");
        assert_eq!(error.reason, ErrorReason::Refused, "--on {on}");
        assert_eq!(error.message, message, "--on {on}: the engine's line verbatim");
    }
    assert_eq!(stored(&f, &change_meta("add-b")), meta_before, "a refusal writes nothing");
    assert_eq!(revision(&f), before);
    assert_eq!(outbox_names(&f), events, "a refusal publishes no event");
}

#[test]
fn re_adding_a_declared_prerequisite_is_an_idempotent_200() {
    // Scenario「冪等重加」。
    let f = fixture(&[
        ("add-a", meta("2026-09-01", None)),
        ("add-b", meta("2026-09-02", Some("add-a"))),
    ]);
    let before = revision(&f);
    let events = outbox_names(&f);
    let response = post_depends(&f, &f.editor_pat, "add-b", &["add-a"], false)
        .expect("idempotent POST succeeds");
    assert_eq!(response.status(), 200);
    let body = response.into_json::<Value>().expect("JSON body");
    assert_eq!(body["dependsOn"], json!(["add-a"]));
    assert_eq!(revision(&f), before, "an idempotent pass commits nothing");
    assert_eq!(outbox_names(&f), events, "an idempotent pass publishes no event");
}
