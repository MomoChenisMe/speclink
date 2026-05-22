//! `LocalStateMachineStore` integration tests.
//!
//! 對應 `state-machine` capability requirement「State mutation SHALL be atomic with
//! audit insert via a single SQLite transaction」、「`change.version` SHALL serve as
//! the compare-and-swap token for all state-machine mutations」。

use speclink_provider::{
    Actor, ChangeState, ChangeStore, ProviderError, StateMachineStore, StateTransitionReason,
    TransitionRequest,
};
use speclink_provider_local::{LocalChangeStore, LocalStateMachineStore};
use tempfile::TempDir;

fn make_stores(tmp: &TempDir) -> (LocalChangeStore, LocalStateMachineStore) {
    let working = tmp.path().to_path_buf();
    let state = working.join(".git").join("speclink");
    std::fs::create_dir_all(&state).expect("state dir");
    (
        LocalChangeStore::new(working.clone(), state.clone()),
        LocalStateMachineStore::new(state),
    )
}

async fn seed_change(change_store: &LocalChangeStore, name: &str) {
    change_store
        .create_change(name, "spec-driven")
        .await
        .expect("seed");
}

#[tokio::test]
async fn get_change_state_returns_proposing_view_after_create() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    let view = sm.get_change_state("demo").await.expect("get");
    assert_eq!(view.state, ChangeState::Proposing);
    assert_eq!(view.version, 1);
    assert!(view.actor.is_none());
    assert!(!view.all_tasks_done);
}

#[tokio::test]
async fn transition_state_proposing_to_ready_writes_state_and_audit_atomically() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    let view = sm
        .transition_state(
            "demo",
            1,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("transition");
    assert_eq!(view.state, ChangeState::Ready);
    assert_eq!(view.version, 2);
    assert!(view.actor.is_none());
}

#[tokio::test]
async fn transition_state_assigns_actor_when_request_carries_actor() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    // First proposing → ready (no actor).
    sm.transition_state(
        "demo",
        1,
        TransitionRequest {
            to_state: ChangeState::Ready,
            actor: None,
            reason: StateTransitionReason::ArtifactDagComplete,
        },
    )
    .await
    .expect("→ready");
    // Then ready → in_progress with actor.
    let view = sm
        .transition_state(
            "demo",
            2,
            TransitionRequest {
                to_state: ChangeState::InProgress,
                actor: Some(Some(Actor {
                    agent_host: "claude-code".into(),
                    os_user: "alice".into(),
                    host_id: "macbook".into(),
                })),
                reason: StateTransitionReason::ApplyStart,
            },
        )
        .await
        .expect("→in_progress");
    assert_eq!(view.state, ChangeState::InProgress);
    assert_eq!(view.version, 3);
    let actor = view.actor.expect("actor populated");
    assert_eq!(actor.agent_host, "claude-code");
    assert_eq!(actor.os_user, "alice");
    assert_eq!(actor.host_id, "macbook");
}

#[tokio::test]
async fn transition_state_returns_version_conflict_on_stale_version() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    let err = sm
        .transition_state(
            "demo",
            99,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect_err("stale version");
    match err {
        ProviderError::StateVersionConflict { current_version } => {
            assert_eq!(current_version, 1);
        }
        other => panic!("expected StateVersionConflict, got {other:?}"),
    }
    // State unchanged
    let view = sm.get_change_state("demo").await.expect("get");
    assert_eq!(view.state, ChangeState::Proposing);
    assert_eq!(view.version, 1);
}

#[tokio::test]
async fn transition_state_returns_change_not_found_for_unknown_name() {
    let tmp = TempDir::new().expect("tempdir");
    let (_change_store, sm) = make_stores(&tmp);
    let err = sm
        .transition_state(
            "missing",
            1,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect_err("missing");
    assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn set_actor_bumps_version_without_state_change() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    let view = sm
        .set_actor(
            "demo",
            1,
            Some(Actor {
                agent_host: "cli".into(),
                os_user: "bob".into(),
                host_id: "h".into(),
            }),
        )
        .await
        .expect("set");
    assert_eq!(view.state, ChangeState::Proposing); // unchanged
    assert_eq!(view.version, 2); // bumped
    assert!(view.actor.is_some());
    // Clearing path
    let view = sm.set_actor("demo", 2, None).await.expect("clear");
    assert_eq!(view.version, 3);
    assert!(view.actor.is_none());
}

#[tokio::test]
async fn set_all_tasks_done_bumps_version_without_state_change() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    seed_change(&change_store, "demo").await;
    let view = sm.set_all_tasks_done("demo", 1, true).await.expect("set");
    assert_eq!(view.state, ChangeState::Proposing);
    assert_eq!(view.version, 2);
    assert!(view.all_tasks_done);
    let view = sm
        .set_all_tasks_done("demo", 2, false)
        .await
        .expect("clear");
    assert_eq!(view.version, 3);
    assert!(!view.all_tasks_done);
}

#[tokio::test]
async fn new_change_starts_at_version_one_per_change_store_baseline() {
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, _sm) = make_stores(&tmp);
    let row = change_store
        .create_change("demo", "spec-driven")
        .await
        .expect("create");
    assert_eq!(row.version, 1);
    assert_eq!(row.state, "proposing");
}

#[tokio::test]
async fn state_machine_mutation_correctly_increments_change_version() {
    // 對應 baseline「Change row Etag (the `version` column) SHALL start at 1 on creation」
    // 之 A3 演進「`version` SHALL serve as the compare-and-swap token for all
    // state-machine mutations」；本測 cross-checks A2 baseline + A3 CAS 行為共存。
    let tmp = TempDir::new().expect("tempdir");
    let (change_store, sm) = make_stores(&tmp);
    let row = change_store
        .create_change("demo", "spec-driven")
        .await
        .expect("create");
    assert_eq!(row.version, 1);
    sm.transition_state(
        "demo",
        1,
        TransitionRequest {
            to_state: ChangeState::Ready,
            actor: None,
            reason: StateTransitionReason::ArtifactDagComplete,
        },
    )
    .await
    .expect("→2");
    sm.set_actor("demo", 2, None).await.expect("→3");
    sm.set_all_tasks_done("demo", 3, true).await.expect("→4");
    let view = sm.get_change_state("demo").await.expect("get");
    assert_eq!(view.version, 4, "version SHALL increment monotonically");
}
