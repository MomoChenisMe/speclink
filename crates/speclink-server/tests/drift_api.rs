//! Spec drift endpoint (server-drift-api spec「規格面 drift 端點且工作區面不進
//! wire」). One `GET /changes/{name}/drift` returns the spec-side drift report
//! and the basis digests of the snapshot it was computed at. Workspace facts
//! never appear: the Server runs no git, so the wire gives it no field to claim
//! one in. Plus the shared bearer/binding precondition (401/403), unknown-change
//! (404), store-unavailable (503), and the read-only guarantee (no outbox event).

mod common;

use speclink_protocol::drift::SpecDriftResponse;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, FaultPoint, OutboxCursor, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

const META: &str = "schema: spec-driven\ncreated: 2026-07-13\n";
const CONFIG: &str = "schema: spec-driven\n";
const DESIGN: &str = "## Context\n\nUses `Widget_kind` and `src/app.rs`.\n";
const TASKS: &str = "- [ ] 1.1 wire `src/app.rs`\n";

/// `demo`'s delta touches two capabilities. `payment` MODIFIES a requirement
/// the canonical spec still has (assumption holds); `auth` MODIFIES one the
/// canonical spec no longer has (a stale assumption the Specs dimension scores).
const DELTA_PAYMENT: &str = "## MODIFIED Requirements\n\n### Requirement: Pay\n\nPay SHALL work.\n";
const DELTA_AUTH: &str = "## MODIFIED Requirements\n\n### Requirement: Rotate tokens\n\nIt SHALL rotate.\n";
const SPEC_PAYMENT: &str = "## Purpose\n\nP\n\n## Requirements\n\n### Requirement: Pay\n\nPay SHALL work.\n";
const SPEC_AUTH: &str = "## Purpose\n\nA\n\n## Requirements\n\n### Requirement: Sign in\n\nIt SHALL sign in.\n";

/// Seed a scope with the `demo` change (design, tasks, two delta specs) and the
/// canonical specs its deltas address.
fn seed(store: &MemoryStore) {
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, META);
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "design.md".into() },
        DESIGN,
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        TASKS,
    );
    uow.create(
        DocumentId::ChangeArtifact {
            change: "demo".into(),
            artifact: "specs/payment/spec.md".into(),
        },
        DELTA_PAYMENT,
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "specs/auth/spec.md".into() },
        DELTA_AUTH,
    );
    uow.create(DocumentId::CanonicalSpec { capability: "payment".into() }, SPEC_PAYMENT);
    uow.create(DocumentId::CanonicalSpec { capability: "auth".into() }, SPEC_AUTH);
    uow.create(DocumentId::WorkflowConfig, CONFIG);
    store.commit(uow, Vec::new()).expect("seed commit");
}

fn state_over(store: Arc<MemoryStore>) -> AppState {
    let state = AppState {
        store: store as SharedStore,
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
        events: common::detached_events(),
    };
    common::seed_demo_registry(&*state.identity);
    state
}

fn drift_url(base: &str, change: &str) -> String {
    format!("{base}/api/speclink/v1/projects/demo/changes/{change}/drift")
}

/// GET the drift endpoint; returns (status, body string).
fn get_drift(url: &str, token: &str) -> (u16, String) {
    let outcome = ureq::get(url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", "1")
        .set("X-Speclink-Repo", "backend")
        .call();
    match outcome {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

fn parse(body: &str) -> SpecDriftResponse {
    serde_json::from_str(body)
        .unwrap_or_else(|e| panic!("body is a SpecDriftResponse ({e}): {body}"))
}

fn started() -> (Arc<MemoryStore>, String, String) {
    let store = Arc::new(MemoryStore::new());
    seed(&store);
    let state = state_over(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    (store, base, pat)
}

// --- Scenario: 規格面報告與 basis ---

#[test]
fn a_change_with_delta_specs_gets_its_spec_dimension_and_basis() {
    let (_store, base, pat) = started();

    let (status, body) = get_drift(&drift_url(&base, "demo"), &pat);
    assert_eq!(status, 200, "a scoped drift request is 200: {body}");
    let response = parse(&body);

    let dim = &response.spec_drift.dimension;
    assert_eq!(dim.kind, "Specs", "the report carries the Specs dimension");
    assert!(dim.contributes_to_total, "the Specs dimension contributes to the total");

    // `auth`'s MODIFIED target is gone from the canonical spec; `payment`'s is
    // still there — exactly one stale assumption, scored 4 (min(4 * 1, 9)).
    let assumptions = &response.spec_drift.spec_assumptions;
    assert_eq!(assumptions.len(), 1, "one stale delta assumption: {assumptions:?}");
    assert_eq!(assumptions[0].capability, "auth");
    assert_eq!(assumptions[0].operation, "MODIFIED");
    assert_eq!(assumptions[0].requirement, "Rotate tokens");
    assert_eq!(dim.status, "1 stale assumptions");
    assert_eq!(dim.score, 4);

    for digest in [&response.basis.spec, &response.basis.tasks, &response.basis.policy] {
        assert!(digest.starts_with("sha256:"), "basis digest form: {digest}");
    }

    // The store-side inputs the client's workspace-side computation reads,
    // fixed by the same snapshot as the report and the basis.
    assert_eq!(response.change.created.as_deref(), Some("2026-07-13"));
    assert_eq!(response.change.design.as_deref(), Some(DESIGN));
    assert_eq!(response.change.tasks.as_deref(), Some(TASKS));
}

/// Absence must survive the trip: an empty string would tell the client's
/// Structure dimension "a design exists with no anchors" instead of "no design".
#[test]
fn a_change_without_a_design_reports_it_absent_not_empty() {
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "bare".into() }, META);
    uow.create(
        DocumentId::ChangeArtifact { change: "bare".into(), artifact: "tasks.md".into() },
        TASKS,
    );
    uow.create(DocumentId::WorkflowConfig, CONFIG);
    store.commit(uow, Vec::new()).expect("seed commit");

    let state = state_over(store);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let (status, body) = get_drift(&drift_url(&base, "bare"), &pat);
    assert_eq!(status, 200, "{body}");
    let response = parse(&body);
    assert_eq!(response.change.design, None, "a missing design is absent, not Some(\"\")");
    assert_eq!(response.change.tasks.as_deref(), Some(TASKS), "the present artifact still arrives");

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        json["change"].get("design").is_none(),
        "an absent input serializes to no key at all: {body}"
    );
}

/// The type layer is the guarantee: a workspace-side key cannot appear because
/// no field carries it. Asserted on the raw body so a future field addition is
/// caught here and not only in review.
#[test]
fn the_response_carries_no_workspace_or_git_field() {
    let (_store, base, pat) = started();
    let (status, body) = get_drift(&drift_url(&base, "demo"), &pat);
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).expect("a JSON body");
    let keys: Vec<String> = collect_keys(&json);
    for absent in [
        "brokenAnchors",
        "commitWindow",
        "trackedDocs",
        "symbolHeadHits",
        "pathStatus",
        "touchedFiles",
        "tasksMaybeResolved",
        "tasksBlockedExternal",
        "commitsSinceCreated",
        "lastCommit",
        "coverage",
        "totalScore",
    ] {
        assert!(
            !keys.iter().any(|k| k == absent),
            "'{absent}' is workspace-side and must not reach the wire: {keys:?}"
        );
    }
    // The wire is exactly what the Server knows from its snapshot: the spec
    // side, the basis, and the change's store-side inputs — nothing more.
    let top: Vec<&str> = json.as_object().expect("an object").keys().map(String::as_str).collect();
    assert_eq!(top, vec!["basis", "change", "specDrift"], "the response is specDrift + basis + change inputs");
    let inputs: Vec<&str> =
        json["change"].as_object().expect("an object").keys().map(String::as_str).collect();
    assert_eq!(
        inputs,
        vec!["created", "design", "tasks"],
        "the store-side inputs are the three the workspace-side computation reads"
    );
}

/// Every key appearing anywhere in the payload.
fn collect_keys(v: &serde_json::Value) -> Vec<String> {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .flat_map(|(k, val)| {
                let mut out = vec![k.clone()];
                out.extend(collect_keys(val));
                out
            })
            .collect(),
        serde_json::Value::Array(items) => items.iter().flat_map(collect_keys).collect(),
        _ => Vec::new(),
    }
}

// --- Scenario: 未知 change 拒絕 ---

#[test]
fn an_unknown_change_is_404_not_found() {
    let (_store, base, pat) = started();
    let (status, body) = get_drift(&drift_url(&base, "nope"), &pat);
    assert_eq!(status, 404, "an unknown change is not found: {body}");
    let err: ErrorResponse = serde_json::from_str(&body).expect("an ErrorResponse envelope");
    assert_eq!(err.reason, ErrorReason::NotFound);
}

// --- shared preconditions and the read-only guarantee ---

#[test]
fn unauthenticated_is_401_and_non_member_is_403() {
    let store = Arc::new(MemoryStore::new());
    seed(&store);
    let state = state_over(store);
    common::seed_multi_project(&*state.identity);
    let (_pat_demo, _) = common::seed_pat(&state.identity, &["demo"]);
    let (pat_other, _) = common::seed_named_pat(
        &state.identity,
        "other@example.com",
        "Other <o@example.com>",
        &["multi"],
    );
    let base = common::start(state);

    let (status, _body) = get_drift(&drift_url(&base, "demo"), "wrong-token");
    assert_eq!(status, 401, "an invalid bearer is unauthorized");

    let (status, body) = get_drift(&drift_url(&base, "demo"), &pat_other);
    assert_eq!(status, 403, "a valid token whose actor is not a member is forbidden");
    let err: ErrorResponse = serde_json::from_str(&body).expect("an ErrorResponse envelope");
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn an_unreachable_store_is_503_unavailable() {
    let store = MemoryStore::new();
    store.crash_at(FaultPoint::AfterDocWrites);
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::WorkflowConfig, "x");
    let _ = store.commit(uow, Vec::new());

    let state = state_over(Arc::new(store));
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let (status, body) = get_drift(&drift_url(&base, "demo"), &pat);
    assert_eq!(status, 503, "an unreachable store answers 503");
    let err: ErrorResponse = serde_json::from_str(&body).expect("an ErrorResponse envelope");
    assert_eq!(err.reason, ErrorReason::Unavailable);
}

/// drift is diagnostic: computing one is a read, so it commits nothing and
/// publishes nothing.
#[test]
fn computing_drift_produces_no_outbox_event() {
    let (store, base, pat) = started();
    let before = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox").len();

    let (status, _body) = get_drift(&drift_url(&base, "demo"), &pat);
    assert_eq!(status, 200);

    let after = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox").len();
    assert_eq!(after, before, "a drift computation is read-only — no event lands in the outbox");
}
