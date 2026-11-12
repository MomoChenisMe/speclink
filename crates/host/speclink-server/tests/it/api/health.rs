//! Health and readiness endpoints (reference-server spec「健康檢查」). `/healthz`
//! answers while the process lives; `/readyz` reflects store health and turns
//! non-2xx when the store cannot serve.

use crate::common;

use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, FaultPoint, ProjectId, RepoId, Scope, TeamStore,
};
use speclink_server::state::SharedStore;
use std::sync::Arc;

/// The HTTP status of a GET, whether the response was 2xx or an error status.
fn status_of(url: &str) -> u16 {
    match ureq::get(url).call() {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error hitting {url}: {e}"),
    }
}

/// A store driven into the crashed state (a fault injected at a commit stage),
/// so `health()` reports Unavailable.
fn crashed_store() -> SharedStore {
    let store = MemoryStore::new();
    store.crash_at(FaultPoint::AfterDocWrites);
    let scope = Scope::new(ProjectId::new("default"), RepoId::new("main"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::WorkflowConfig, "x");
    let _ = store.commit(uow, Vec::new()); // hits the armed fault; store is now crashed
    assert!(store.health().is_err(), "the store is unhealthy after the crash");
    Arc::new(store)
}

#[test]
fn healthz_and_readyz_are_2xx_on_a_healthy_store() {
    let store: SharedStore = Arc::new(MemoryStore::new());
    let base = common::start(common::state_with(store));
    assert_eq!(status_of(&format!("{base}/healthz")), 200, "healthz is 2xx");
    assert_eq!(status_of(&format!("{base}/readyz")), 200, "readyz is 2xx on a live store");
}

#[test]
fn readyz_turns_non_2xx_when_the_store_is_unavailable() {
    let base = common::start(common::state_with(crashed_store()));
    // healthz stays green — the process is alive.
    assert_eq!(status_of(&format!("{base}/healthz")), 200, "healthz is process-level");
    assert_eq!(
        status_of(&format!("{base}/readyz")),
        503,
        "readyz reflects the unavailable store"
    );
}

/// spec host-kit「示範用 Server 的宣告」: asked with no credential and no
/// version header, the demo server declares device login and access keys,
/// and no browser login.
#[test]
fn the_login_methods_are_device_and_token_without_a_credential() {
    let store: SharedStore = Arc::new(MemoryStore::new());
    let base = common::start(common::state_with(store));

    let response = ureq::get(&format!("{base}/api/speclink/v1/auth-methods"))
        .call()
        .expect("the declaration needs no credential");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.into_json().expect("JSON body");
    assert_eq!(body, serde_json::json!({ "methods": ["device", "token"] }), "no browser object");
}

/// host-kit-client-door D1: a method the door does not take is answered 405
/// by the host, with the methods it does take.
#[test]
fn a_method_the_door_does_not_take_is_405() {
    let store: SharedStore = Arc::new(MemoryStore::new());
    let base = common::start(common::state_with(store));
    for path in ["/api/speclink/v1/scopes", "/api/speclink/v1/projects/demo/changes"] {
        match ureq::request("PATCH", &format!("{base}{path}")).call() {
            Err(ureq::Error::Status(405, response)) => {
                assert_eq!(response.header("allow"), Some("GET, HEAD, POST, PUT, DELETE"), "{path}");
            }
            other => panic!("{path}: expected 405, got {other:?}"),
        }
    }
}
