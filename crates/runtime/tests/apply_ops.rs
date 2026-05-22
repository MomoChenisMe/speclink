//! Integration tests for `ApplyOperations`.
//!
//! 對應 spec requirement「`speclink apply start` SHALL implement the ensure-actor
//! semantics defined by design.md §6.2」與「`speclink apply pause` SHALL implement
//! symmetric idempotency against `apply.start`」。

use std::path::Path;
use std::process::Command;

use speclink_provider::{ChangeState, StateMachineStore, StateTransitionReason, TransitionRequest};
use speclink_provider_local::{LocalChangeStore, LocalStateMachineStore};
use speclink_runtime::{
    ApplyOperations, Bootstrap, ChangeOperations, RealGitProbe, RuntimeError, resolve_state_root,
};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "command failed: {cmd:?}");
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "t@e.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "t"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

async fn fresh_project_with_change(name: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    Bootstrap::new(RealGitProbe)
        .init(&working, false)
        .await
        .expect("init");
    ChangeOperations::new(RealGitProbe)
        .create_change(&working, name)
        .await
        .expect("create");
    (tmp, working)
}

async fn stores(working: &Path) -> (LocalChangeStore, LocalStateMachineStore) {
    let state = resolve_state_root(&RealGitProbe, working).expect("state");
    (
        LocalChangeStore::new(working.to_path_buf(), state.clone()),
        LocalStateMachineStore::new(state),
    )
}

async fn force_state(working: &Path, change: &str, to: ChangeState, reason: StateTransitionReason) {
    let (_cs, sm) = stores(working).await;
    let view = sm.get_change_state(change).await.expect("get");
    sm.transition_state(
        change,
        view.version,
        TransitionRequest {
            to_state: to,
            actor: None,
            reason,
        },
    )
    .await
    .expect("force transition");
}

// ----- apply.start matrix -----

#[tokio::test]
async fn apply_start_on_proposing_returns_transition_invalid() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    let ops = ApplyOperations::new(RealGitProbe);
    let err = ops
        .start(&working, "demo", None)
        .await
        .expect_err("proposing rejects");
    assert!(matches!(err, RuntimeError::StateTransitionInvalid { .. }));
}

#[tokio::test]
async fn apply_start_on_reviewing_returns_transition_invalid() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    // Manually push to reviewing via raw state machine transition.
    force_state(
        &working,
        "demo",
        ChangeState::Reviewing,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    let err = ops.start(&working, "demo", None).await.expect_err("reject");
    assert!(matches!(err, RuntimeError::StateTransitionInvalid { .. }));
}

#[tokio::test]
async fn apply_start_on_ready_transitions_to_in_progress_with_actor() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    force_state(
        &working,
        "demo",
        ChangeState::Ready,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    let data = ops
        .start(&working, "demo", Some("claude-code"))
        .await
        .expect("start");
    assert_eq!(data.state, ChangeState::InProgress);
    let actor = data.actor.expect("actor populated");
    assert_eq!(actor.agent_host, "claude-code");
    assert!(data.message.is_none());
}

#[tokio::test]
async fn apply_start_on_in_progress_is_idempotent_reassign() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    force_state(
        &working,
        "demo",
        ChangeState::Ready,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    let first = ops
        .start(&working, "demo", Some("claude-code"))
        .await
        .expect("first");
    assert_eq!(first.state, ChangeState::InProgress);
    let second = ops
        .start(&working, "demo", Some("cursor"))
        .await
        .expect("second reassign");
    assert_eq!(second.state, ChangeState::InProgress);
    let actor = second.actor.expect("actor");
    assert_eq!(actor.agent_host, "cursor");
}

#[tokio::test]
async fn apply_start_on_code_reviewing_returns_hint_message() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    force_state(
        &working,
        "demo",
        ChangeState::Ready,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    force_state(
        &working,
        "demo",
        ChangeState::InProgress,
        StateTransitionReason::ApplyStart,
    )
    .await;
    force_state(
        &working,
        "demo",
        ChangeState::CodeReviewing,
        StateTransitionReason::TaskDoneAuto,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    let data = ops.start(&working, "demo", None).await.expect("hint");
    assert_eq!(data.state, ChangeState::CodeReviewing);
    assert_eq!(
        data.message.as_deref(),
        Some("Already in code review; nothing to apply.")
    );
}

// ----- apply.pause matrix -----

#[tokio::test]
async fn apply_pause_on_in_progress_transitions_to_ready_and_clears_actor() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    force_state(
        &working,
        "demo",
        ChangeState::Ready,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    ops.start(&working, "demo", Some("cli"))
        .await
        .expect("start");
    let data = ops.pause(&working, "demo").await.expect("pause");
    assert_eq!(data.state, ChangeState::Ready);
    assert!(data.actor.is_none(), "actor SHALL be cleared by pause");
}

#[tokio::test]
async fn apply_pause_on_ready_is_idempotent_with_hint() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    force_state(
        &working,
        "demo",
        ChangeState::Ready,
        StateTransitionReason::ArtifactDagComplete,
    )
    .await;
    let ops = ApplyOperations::new(RealGitProbe);
    let data = ops.pause(&working, "demo").await.expect("pause on ready");
    assert_eq!(data.state, ChangeState::Ready);
    assert_eq!(data.message.as_deref(), Some("Change is already paused."));
}

#[tokio::test]
async fn apply_pause_on_proposing_returns_transition_invalid() {
    let (_tmp, working) = fresh_project_with_change("demo").await;
    let ops = ApplyOperations::new(RealGitProbe);
    let err = ops.pause(&working, "demo").await.expect_err("reject");
    assert!(matches!(err, RuntimeError::StateTransitionInvalid { .. }));
}
