//! Verify verb routes over the same bridge and commit path (design D4a／D8):
//! add-round / show / stamp / discard ride `Command::Verify*`, so the engine's
//! gates (including D3's「任務全完成才落工單」asymmetry), the
//! delete-ticket-with-stamp atomicity, and outbox events all come from the
//! engine. The wire shape is the review station's — the two stations differ
//! only in which document the engine reads and which meta prefix it stamps.

use crate::common;

use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_remote::client::Client;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn client(base: &str, token: &str) -> Client {
    Client::new(&format!("{base}/api/speclink/v1/projects/demo"), token, Some("backend"))
}

struct Fixture {
    base: String,
    store: Arc<MemoryStore>,
    pat: String,
}

const ROUND_WITH_FINDING: &str =
    "**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — requirement R2 has no implementation\n";
const CLEAN_ROUND: &str = "**Scope**: src/lib.rs\n";

/// Server over a store seeded with change `demo-change`; `tasks` decides
/// whether the D3 add-round gate lets a ticket exist at all.
fn fixture_with_tasks(tasks: &str) -> Fixture {
    let store = Arc::new(MemoryStore::new());
    let state = common::state_with(store.clone());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed-verify-api".into(), actor: "seed".into() },
        )
        .expect("begin seed uow");
    uow.create(
        DocumentId::ChangeMeta { change: "demo-change".into() },
        "schema: spec-driven\ncreated: 2026-08-01\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo-change".into(), artifact: "tasks.md".into() },
        tasks,
    );
    store.commit(uow, Vec::new()).expect("seed documents");
    Fixture { base: common::start(state), store, pat }
}

fn fixture() -> Fixture {
    fixture_with_tasks("- [x] 1 done\n")
}

fn artifact(f: &Fixture, name: &str) -> Option<String> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::ChangeArtifact {
            change: "demo-change".into(),
            artifact: name.into(),
        })
        .expect("read artifact")
        .map(|d| d.content)
}

fn meta_doc(f: &Fixture) -> String {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::ChangeMeta { change: "demo-change".into() })
        .expect("read meta")
        .expect("meta exists")
        .content
}

fn outbox_names(f: &Fixture) -> Vec<String> {
    f.store
        .read_outbox(&scope(), OutboxCursor(0))
        .expect("read outbox")
        .iter()
        .map(|e| e.record.name.clone())
        .collect()
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

fn raw(method: &str, f: &Fixture, tail: &str) -> ureq::Request {
    ureq::request(method, &format!("{}/api/speclink/v1/projects/demo/{tail}", f.base))
        .set("Authorization", &format!("Bearer {}", f.pat))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
}

#[test]
fn add_round_show_and_discard_ride_the_verify_endpoints() {
    // spec「驗證動詞的 remote 模式行為」：工單經 store 文件管道讀寫，wire 欄位
    // 與審查站同構，事件落 outbox。
    let f = fixture();
    let c = client(&f.base, &f.pat);
    let round = c.station_add_round("verify", "demo-change", ROUND_WITH_FINDING).expect("round");
    assert_eq!(round.round, 1);
    assert!(artifact(&f, "verify.md").is_some(), "the ticket document was written");
    assert!(artifact(&f, "review.md").is_none(), "the review document is untouched");

    let ticket = c.station_ticket("verify", "demo-change").expect("ticket");
    assert_eq!(ticket.change, "demo-change");
    assert_eq!(ticket.rounds.len(), 1);
    assert_eq!(ticket.last_round.findings[0].severity, "CRITICAL");
    assert!(ticket.last_round.phase.is_none(), "legacy round emits an explicit null");

    c.station_discard("verify", "demo-change").expect("discard");
    assert!(artifact(&f, "verify.md").is_none(), "the ticket is gone");
    let names = outbox_names(&f);
    assert!(names.contains(&"verify-round-added".to_string()), "{names:?}");
    assert!(names.contains(&"verify-discarded".to_string()), "{names:?}");
}

#[test]
fn add_round_is_refused_until_every_task_is_done() {
    // design D3 的引擎守門在 remote 面同樣成立：盤點輪不得落工單。
    let f = fixture_with_tasks("- [x] 1 done\n- [ ] 2 pending\n");
    let (status, error) = protocol_error(
        raw("POST", &f, "changes/demo-change/verify/rounds")
            .send_json(serde_json::json!({ "content": CLEAN_ROUND })),
    );
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused);
    assert!(error.message.contains("1/2"), "the count rides the wire: {}", error.message);
    assert!(artifact(&f, "verify.md").is_none(), "a refusal writes nothing");
}

#[test]
fn stamp_writes_the_verified_fields_and_deletes_the_ticket_atomically() {
    // spec「驗證蓋章守門與蓋章效果」：五欄位寫入與工單刪除在同一次提交裡，
    // 不得出現「章已寫而工單仍在」的中間狀態。
    let f = fixture();
    let c = client(&f.base, &f.pat);
    c.station_add_round("verify", "demo-change", CLEAN_ROUND).expect("round");
    c.station_stamp(
        "verify",
        "demo-change",
        false,
        Some("claude"),
        &[speclink_protocol::command::ReviewScopeEntryDto {
            path: "src/lib.rs".into(),
            hash: "deadbeef".into(),
        }],
        &[],
    )
    .expect("stamp");
    let meta = meta_doc(&f);
    for key in ["verified_at:", "verified_with:", "verified_tasks_total:", "verified_scope:"] {
        assert!(meta.contains(key), "missing {key}: {meta}");
    }
    assert!(!meta.contains("reviewed_at:"), "the review station is untouched: {meta}");
    assert!(artifact(&f, "verify.md").is_none(), "the ticket left with the stamp");
    assert!(outbox_names(&f).contains(&"verify-stamped".to_string()));
}

#[test]
fn stamp_refuses_unresolved_findings_without_accept() {
    let f = fixture();
    let c = client(&f.base, &f.pat);
    c.station_add_round("verify", "demo-change", ROUND_WITH_FINDING).expect("round");
    let (status, error) = protocol_error(
        raw("POST", &f, "changes/demo-change/verify/stamp")
            .send_json(serde_json::json!({ "accept": false, "scope": [] })),
    );
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused);
    assert!(artifact(&f, "verify.md").is_some(), "the ticket survives the refusal");
}

#[test]
fn show_without_a_ticket_is_a_typed_not_found() {
    let f = fixture();
    let (status, error) = protocol_error(raw("GET", &f, "changes/demo-change/verify").call());
    assert_eq!(status, 404);
    assert_eq!(error.reason, ErrorReason::NotFound);
    assert!(error.message.contains("no verify ticket"), "{}", error.message);
}

#[test]
fn carry_verify_rides_the_wire_so_remote_keeps_all_three_disposals() {
    // 拒絕訊息叫人加 `--carry-verify`——remote 模式下該旗標必須真的到得了引擎，
    // 否則三處置在遠端只剩兩條、帶未結驗證工單的 change 永遠封存不了。
    let f = fixture();
    let c = client(&f.base, &f.pat);
    c.station_add_round("verify", "demo-change", ROUND_WITH_FINDING).expect("round");

    let (status, error) = protocol_error(
        raw("POST", &f, "changes/demo-change/archive").send_json(serde_json::json!({})),
    );
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused);
    assert!(error.message.contains("verify stamp"), "lists the disposals: {}", error.message);

    c.archive("demo-change", false, true).expect("carry-verify archives over the wire");
    assert!(artifact(&f, "verify.md").is_none(), "the change (with its ticket) left the active area");
}

#[test]
fn structured_verify_rounds_survive_the_wire() {
    // spec Scenario「local 與 remote payload 同構」的驗證面：rounds[] 與
    // lastRound 帶 phase／patchHash，欄位集合與 local CLI 完全一致。
    let f = fixture();
    let c = client(&f.base, &f.pat);
    let hex = "a".repeat(64);
    let structured = format!(
        "**Phase**: discovery\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n\n- [WARNING] src/lib.rs — scenario 3 untested\n"
    );
    c.station_add_round("verify", "demo-change", &structured).expect("structured round");
    let body = raw("GET", &f, "changes/demo-change/verify")
        .call()
        .expect("get ticket")
        .into_string()
        .expect("body");
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["rounds"][0]["phase"], "discovery");
    assert_eq!(v["lastRound"]["patchHash"], format!("sha256:{hex}"));
    assert_eq!(
        v["lastRound"].as_object().map(|o| o.len()),
        Some(5),
        "index/phase/patchHash/scope/findings only: {v}"
    );
}
