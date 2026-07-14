//! Polling foundation (reference-server spec「健康檢查與 ETag 輪詢地基」).
//! `/sync-state` returns 304 while the scope is unchanged and 200 with a new
//! ETag after any commit; a query against an unreachable store is 503
//! unavailable.

mod common;

use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_remote::client::Client;
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Scope, TeamStore,
};
use speclink_server::state::SharedStore;
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn seeded_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    store
}

fn crashed_store() -> SharedStore {
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
    Arc::new(store)
}

fn sync_url(base: &str) -> String {
    format!("{base}/api/speclink/v1/projects/demo/sync-state")
}

/// GET `/sync-state`, optionally with If-None-Match; returns (status, etag).
fn poll(url: &str, if_none_match: Option<&str>) -> (u16, String) {
    let mut req = ureq::get(url)
        .set("Authorization", "Bearer secret")
        .set("X-Speclink-Api-Version", "1")
        .set("X-Speclink-Repo", "backend");
    if let Some(inm) = if_none_match {
        req = req.set("If-None-Match", inm);
    }
    match req.call() {
        Ok(resp) => (resp.status(), resp.header("etag").unwrap_or_default().to_string()),
        Err(ureq::Error::Status(code, resp)) => {
            (code, resp.header("etag").unwrap_or_default().to_string())
        }
        Err(e) => panic!("transport error: {e}"),
    }
}

fn client(base: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        "secret",
        Some("backend"),
    )
}

#[test]
fn polling_detects_a_change_across_a_commit() {
    let store = seeded_store();
    let base = common::start(common::state_with(store));
    let url = sync_url(&base);

    let (status, e0) = poll(&url, None);
    assert_eq!(status, 200, "the first poll returns the current token");
    assert!(!e0.is_empty(), "an ETag is declared");

    let (status, _) = poll(&url, Some(&e0));
    assert_eq!(status, 304, "re-polling before any write is not modified");

    // Another writer completes a commit.
    client(&base).task_done("demo", "1", &[]).expect("task done");

    let (status, e1) = poll(&url, Some(&e0));
    assert_eq!(status, 200, "re-polling after the write returns 200");
    assert_ne!(e1, e0, "the ETag advanced with the commit");
}

#[test]
fn a_query_against_an_unreachable_store_is_503_unavailable() {
    // Asserted at the wire envelope: the typed client collapses every 5xx to a
    // generic reasonless message, so the raw response carries the registry
    // reason.
    let base = common::start(common::state_with(crashed_store()));
    let url = format!("{base}/api/speclink/v1/projects/demo/changes");
    let (status, reason) = match ureq::get(&url)
        .set("Authorization", "Bearer secret")
        .set("X-Speclink-Api-Version", "1")
        .set("X-Speclink-Repo", "backend")
        .call()
    {
        Ok(resp) => panic!("expected a failure, got HTTP {}", resp.status()),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            let reason = serde_json::from_str::<ErrorResponse>(&body)
                .map(|e| e.reason)
                .unwrap_or_else(|_| panic!("body is an ErrorResponse: {body}"));
            (code, reason)
        }
        Err(e) => panic!("transport error: {e}"),
    };
    assert_eq!(status, 503, "an unreachable store answers 503");
    assert_eq!(reason, ErrorReason::Unavailable, "the reason is unavailable");
}
