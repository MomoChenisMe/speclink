//! Context snapshot endpoint (server-context-api spec「一致快照端點」/「change 縮小
//! 與 flow 透傳」). One `POST /context` returns a `ContextSnapshot` read from a
//! single consistent store snapshot: every document at the same state, the
//! snapshot id sourced from the scope state token, per-document contract
//! digests, `If-None-Match`/304, change-narrowing, and the shared bearer/binding
//! precondition (401/403) and store-unavailable (503) failures.

mod common;

use speclink_protocol::context::{ContextSnapshot, ContextSnapshotRequest};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{
    content_digest, CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Revision, Scope,
    TeamStore,
};
use std::collections::BTreeSet;
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

const PROPOSAL: &str = "## Why\n\nDemo change.\n";
const DESIGN: &str = "## Context\n\nDemo design.\n";
const TASKS: &str = "- [ ] 1.1 First\n- [ ] 1.2 Second\n";
const DELTA: &str = "## MODIFIED Requirements\n\n### Requirement: Pay\n";
const SPEC_PAYMENT: &str = "### Requirement: Pay\nPay SHALL work.\n";
const SPEC_AUTH: &str = "### Requirement: Auth\nAuth SHALL work.\n";
const SPEC_BILLING: &str = "### Requirement: Bill\nBill SHALL work.\n";
const CONFIG: &str = "schema: spec-driven\n";
const LANGUAGE: &str = "# Shared Vocabulary\n\n- Change: a proposed edit.\n";

/// Seed a scope with two changes (`demo` with a delta spec, `other`), three
/// canonical specs, the workflow config, and the LANGUAGE document — the full
/// read surface a change-narrowed snapshot draws from.
fn seed_full(store: &MemoryStore, with_config: bool) {
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, CONFIG);
    uow.create(DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() }, PROPOSAL);
    uow.create(DocumentId::ChangeArtifact { change: "demo".into(), artifact: "design.md".into() }, DESIGN);
    uow.create(DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() }, TASKS);
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "specs/payment/spec.md".into() },
        DELTA,
    );
    uow.create(DocumentId::ChangeMeta { change: "other".into() }, CONFIG);
    uow.create(DocumentId::ChangeArtifact { change: "other".into(), artifact: "proposal.md".into() }, "## Why\n\nOther.\n");
    uow.create(DocumentId::CanonicalSpec { capability: "payment".into() }, SPEC_PAYMENT);
    uow.create(DocumentId::CanonicalSpec { capability: "auth".into() }, SPEC_AUTH);
    uow.create(DocumentId::CanonicalSpec { capability: "billing".into() }, SPEC_BILLING);
    uow.create(DocumentId::Language, LANGUAGE);
    if with_config {
        uow.create(DocumentId::WorkflowConfig, CONFIG);
    }
    store.commit(uow, Vec::new()).expect("seed commit");
}

/// Build application state over `store` with the demo registry seeded.
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

/// A commit that edits `demo`'s tasks.md, advancing the scope revision.
fn commit_edit(store: &MemoryStore) {
    let doc = DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() };
    let rev = store
        .snapshot(&scope())
        .unwrap()
        .read(&doc)
        .unwrap()
        .map(|d| d.revision)
        .unwrap_or(Revision(0));
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "edit".into(), actor: "edit".into() },
        )
        .expect("begin uow");
    uow.update(doc, "- [x] 1.1 First\n- [ ] 1.2 Second\n", rev);
    store.commit(uow, Vec::new()).expect("edit commit");
}

fn context_url(base: &str) -> String {
    format!("{base}/api/speclink/v1/projects/demo/context")
}

/// POST `/context` with `request` and optional `If-None-Match`; returns
/// (status, etag header, body string).
fn post_context(
    url: &str,
    token: &str,
    request: &ContextSnapshotRequest,
    if_none_match: Option<&str>,
) -> (u16, String, String) {
    let mut req = ureq::post(url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", "1")
        .set("X-Speclink-Repo", "backend");
    if let Some(inm) = if_none_match {
        req = req.set("If-None-Match", inm);
    }
    let outcome = req.send_json(serde_json::to_value(request).unwrap());
    match outcome {
        Ok(resp) => {
            let status = resp.status();
            let etag = resp.header("etag").unwrap_or_default().to_string();
            (status, etag, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(code, resp)) => {
            let etag = resp.header("etag").unwrap_or_default().to_string();
            (code, etag, resp.into_string().unwrap_or_default())
        }
        Err(e) => panic!("transport error: {e}"),
    }
}

fn for_change(change: &str) -> ContextSnapshotRequest {
    ContextSnapshotRequest { change: Some(change.into()), flow: None }
}

fn parse(body: &str) -> ContextSnapshot {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("body is a ContextSnapshot ({e}): {body}"))
}

fn paths(snap: &ContextSnapshot) -> BTreeSet<String> {
    snap.documents.iter().map(|d| d.path.clone()).collect()
}

fn started(with_config: bool) -> (Arc<MemoryStore>, String, String) {
    let store = Arc::new(MemoryStore::new());
    seed_full(&store, with_config);
    let state = state_over(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    (store, base, pat)
}

#[test]
fn snapshot_is_consistent_and_identified_by_the_scope_token() {
    let (store, base, pat) = started(true);
    let url = context_url(&base);

    let (status, etag1, body1) = post_context(&url, &pat, &for_change("demo"), None);
    assert_eq!(status, 200, "a scoped snapshot is 200");
    let snap1 = parse(&body1);
    assert!(!etag1.is_empty(), "the response declares the scope state token as ETag");
    assert_eq!(snap1.snapshot_id, etag1, "the snapshot id is the scope state token (same source)");
    let tasks_before = snap1
        .documents
        .iter()
        .find(|d| d.path == "openspec/changes/demo/tasks.md")
        .expect("tasks.md is in the snapshot")
        .content
        .clone();
    assert_eq!(tasks_before, TASKS, "the first snapshot reflects the pre-edit state");

    // A commit lands after the first snapshot was taken.
    commit_edit(&store);

    let (status, etag2, body2) = post_context(&url, &pat, &for_change("demo"), None);
    assert_eq!(status, 200);
    let snap2 = parse(&body2);
    assert_ne!(etag2, etag1, "any commit advances the scope token");
    assert_ne!(snap2.snapshot_id, snap1.snapshot_id, "the two snapshot ids differ");
    let tasks_after = snap2
        .documents
        .iter()
        .find(|d| d.path == "openspec/changes/demo/tasks.md")
        .expect("tasks.md present")
        .content
        .clone();
    assert_eq!(tasks_after, "- [x] 1.1 First\n- [ ] 1.2 Second\n", "the second snapshot has the write");
}

#[test]
fn if_none_match_is_304_until_a_commit() {
    let (store, base, pat) = started(true);
    let url = context_url(&base);

    let (status, etag0, _) = post_context(&url, &pat, &for_change("demo"), None);
    assert_eq!(status, 200);

    let (status, _, body) = post_context(&url, &pat, &for_change("demo"), Some(&etag0));
    assert_eq!(status, 304, "an unchanged scope with matching If-None-Match is 304");
    assert!(body.is_empty(), "a 304 carries no body");

    commit_edit(&store);

    let (status, etag1, _) = post_context(&url, &pat, &for_change("demo"), Some(&etag0));
    assert_eq!(status, 200, "after a commit the stale If-None-Match gets a fresh 200");
    assert_ne!(etag1, etag0, "the ETag advanced");
}

#[test]
fn policy_revision_is_the_config_revision_and_absent_without_config() {
    let (_store, base, pat) = started(true);
    let (_status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("demo"), None);
    let snap = parse(&body);
    let config_rev = snap
        .documents
        .iter()
        .find(|d| d.path == "openspec/config.yaml")
        .expect("config document present")
        .revision;
    assert!(config_rev.is_some(), "the config document carries a revision");
    assert_eq!(snap.policy_revision, config_rev, "policy revision is the config document's revision");

    // A scope with no workflow config: policy revision is absent.
    let (_store, base, pat) = started(false);
    let (_status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("demo"), None);
    let snap = parse(&body);
    assert!(
        snap.documents.iter().all(|d| d.path != "openspec/config.yaml"),
        "no config document without config"
    );
    assert_eq!(snap.policy_revision, None, "no config → policy revision absent");
}

#[test]
fn every_document_carries_its_contract_digest() {
    let (_store, base, pat) = started(true);
    let (_status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("demo"), None);
    let snap = parse(&body);
    assert!(!snap.documents.is_empty(), "the snapshot has documents");
    for d in &snap.documents {
        assert_eq!(
            d.digest,
            content_digest(&d.content),
            "{} carries the contract content digest",
            d.path
        );
    }
}

#[test]
fn change_narrowed_document_set_is_complete() {
    let (_store, base, pat) = started(true);
    let (_status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("demo"), None);
    let snap = parse(&body);
    let got = paths(&snap);

    let expected: BTreeSet<String> = [
        "openspec/changes/demo/proposal.md",
        "openspec/changes/demo/design.md",
        "openspec/changes/demo/tasks.md",
        "openspec/changes/demo/specs/payment/spec.md",
        "openspec/specs/payment/spec.md",
        "openspec/specs/auth/spec.md",
        "openspec/specs/billing/spec.md",
        "openspec/config.yaml",
        "openspec/LANGUAGE.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(got, expected, "the change-narrowed set is A's docs + all canonical specs + config + LANGUAGE");

    assert!(
        got.iter().all(|p| !p.contains("changes/other")),
        "the other change's documents are excluded: {got:?}"
    );
    assert!(
        got.iter().all(|p| !p.ends_with(".openspec.yaml")),
        "internal change metadata is not projected: {got:?}"
    );
}

#[test]
fn absent_change_returns_the_full_content() {
    let (_store, base, pat) = started(true);
    let request = ContextSnapshotRequest { change: None, flow: None };
    let (status, _etag, body) = post_context(&context_url(&base), &pat, &request, None);
    assert_eq!(status, 200);
    let got = paths(&parse(&body));
    assert!(got.contains("openspec/changes/demo/proposal.md"), "demo carried: {got:?}");
    assert!(got.contains("openspec/changes/other/proposal.md"), "every change carried: {got:?}");
    assert!(got.contains("openspec/specs/auth/spec.md"), "all canonical specs carried");
    assert!(got.contains("openspec/config.yaml") && got.contains("openspec/LANGUAGE.md"));
}

#[test]
fn unknown_change_is_404_not_found() {
    let (_store, base, pat) = started(true);
    let (status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("ghost"), None);
    assert_eq!(status, 404, "an unknown change is not found");
    let err: ErrorResponse = serde_json::from_str(&body).expect("an ErrorResponse envelope");
    assert_eq!(err.reason, ErrorReason::NotFound);
}

#[test]
fn flow_is_passed_through_without_changing_the_document_set() {
    let (_store, base, pat) = started(true);
    let url = context_url(&base);
    let with_flow = ContextSnapshotRequest { change: Some("demo".into()), flow: Some("apply".into()) };
    let (_s1, _e1, b1) = post_context(&url, &pat, &with_flow, None);
    let (_s2, _e2, b2) = post_context(&url, &pat, &for_change("demo"), None);
    assert_eq!(
        paths(&parse(&b1)),
        paths(&parse(&b2)),
        "the flow field does not narrow the server's document set (materializer's job)"
    );
}

#[test]
fn unauthenticated_is_401_and_non_member_is_403() {
    let store = Arc::new(MemoryStore::new());
    seed_full(&store, true);
    let state = state_over(store);
    common::seed_multi_project(&*state.identity);
    let (_pat_demo, _) = common::seed_pat(&state.identity, &["demo"]);
    let (pat_other, _) =
        common::seed_named_pat(&state.identity, "other@example.com", "Other <o@example.com>", &["multi"]);
    let base = common::start(state);
    let url = context_url(&base);

    let (status, _e, _b) = post_context(&url, "wrong-token", &for_change("demo"), None);
    assert_eq!(status, 401, "an invalid bearer is unauthorized");

    let (status, _e, body) = post_context(&url, &pat_other, &for_change("demo"), None);
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

    let (status, _etag, body) = post_context(&context_url(&base), &pat, &for_change("demo"), None);
    assert_eq!(status, 503, "an unreachable store answers 503");
    let err: ErrorResponse = serde_json::from_str(&body).expect("an ErrorResponse envelope");
    assert_eq!(err.reason, ErrorReason::Unavailable);
}
