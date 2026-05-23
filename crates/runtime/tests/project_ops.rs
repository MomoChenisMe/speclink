//! Integration tests for `runtime::project_ops::project_status`.
//!
//! 對齊 specs/project-status：
//!   - changes_count group-by + 永遠 6 buckets
//!   - discussions_count 永遠 {active:0, converged:0}
//!   - current_change 只在 in_progress + actor.host_id == 當前 instance_id 時填
//!   - 非 SpecLink 目錄 → RuntimeError 對應 `project.not_initialized`

use std::path::Path;
use std::process::Command;

use speclink_provider::{Actor, ChangeState, StateMachineStore, TransitionRequest};
use speclink_provider_local::LocalStateMachineStore;
use speclink_runtime::{
    Bootstrap, ChangeOperations, RealGitProbe, RuntimeError, project_ops::project_status,
    state_machine::resolve_host_id,
};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

async fn fresh_project() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    (tmp, working)
}

fn resolve_state_root(working: &Path) -> std::path::PathBuf {
    speclink_runtime::paths::resolve_state_root(&RealGitProbe, working).expect("state root")
}

// ----- 2.1 fresh project（zero rows）-----

#[tokio::test]
async fn project_status_fresh_project_has_zero_changes_count_and_discussions_count_and_null_current()
 {
    let (_tmp, working) = fresh_project().await;
    let status = project_status(&working).await.expect("status");
    assert_eq!(status.provider_type, "local");
    assert_eq!(status.changes_count.proposing, 0);
    assert_eq!(status.changes_count.reviewing, 0);
    assert_eq!(status.changes_count.ready, 0);
    assert_eq!(status.changes_count.in_progress, 0);
    assert_eq!(status.changes_count.code_reviewing, 0);
    assert_eq!(status.changes_count.archived, 0);
    assert_eq!(status.discussions_count.active, 0);
    assert_eq!(status.discussions_count.converged, 0);
    assert!(status.current_change.is_none());
}

// ----- 2.2 seed 11 rows (1 in_progress + 2 ready + 8 archived) -----

#[tokio::test]
async fn project_status_aggregates_changes_count_by_state() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let sm = LocalStateMachineStore::new(resolve_state_root(&working));

    // Helper to seed a change in a target state.
    let me = Actor {
        agent_host: "cli".into(),
        os_user: "test".into(),
        host_id: resolve_host_id(),
    };

    // 2 ready
    for name in ["ready-a", "ready-b"] {
        let row = ops.create_change(&working, name).await.expect("create");
        sm.transition_state(
            name,
            row.version as u64,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→ready");
    }
    // 1 in_progress (assigned to me)
    {
        let row = ops
            .create_change(&working, "in-progress-a")
            .await
            .expect("create");
        // ready 後才能 → in_progress
        let v = sm
            .transition_state(
                "in-progress-a",
                row.version as u64,
                TransitionRequest {
                    to_state: ChangeState::Ready,
                    actor: None,
                    reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
                },
            )
            .await
            .expect("→ready")
            .version;
        sm.transition_state(
            "in-progress-a",
            v,
            TransitionRequest {
                to_state: ChangeState::InProgress,
                actor: Some(Some(me.clone())),
                reason: speclink_provider::StateTransitionReason::ApplyStart,
            },
        )
        .await
        .expect("→in_progress");
    }
    // 8 archived
    for i in 0..8 {
        let name = format!("archived-{i}");
        let row = ops.create_change(&working, &name).await.expect("create");
        // 走 ready → archived 直線
        let v = sm
            .transition_state(
                &name,
                row.version as u64,
                TransitionRequest {
                    to_state: ChangeState::Ready,
                    actor: None,
                    reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
                },
            )
            .await
            .expect("→ready")
            .version;
        sm.transition_state(
            &name,
            v,
            TransitionRequest {
                to_state: ChangeState::Archived,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArchiveRun,
            },
        )
        .await
        .expect("→archived");
    }

    let status = project_status(&working).await.expect("status");
    assert_eq!(status.changes_count.proposing, 0, "proposing");
    assert_eq!(status.changes_count.reviewing, 0, "reviewing");
    assert_eq!(status.changes_count.ready, 2, "ready");
    assert_eq!(status.changes_count.in_progress, 1, "in_progress");
    assert_eq!(status.changes_count.code_reviewing, 0, "code_reviewing");
    assert_eq!(status.changes_count.archived, 8, "archived");
}

// ----- 2.3 current_change happy path：in_progress + me -----

#[tokio::test]
async fn project_status_current_change_matches_in_progress_owned_by_current_host() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let sm = LocalStateMachineStore::new(resolve_state_root(&working));
    let me = Actor {
        agent_host: "cli".into(),
        os_user: "test".into(),
        host_id: resolve_host_id(),
    };
    let row = ops.create_change(&working, "abc").await.expect("create");
    let v = sm
        .transition_state(
            "abc",
            row.version as u64,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→ready")
        .version;
    sm.transition_state(
        "abc",
        v,
        TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: Some(Some(me.clone())),
            reason: speclink_provider::StateTransitionReason::ApplyStart,
        },
    )
    .await
    .expect("→in_progress");

    let status = project_status(&working).await.expect("status");
    let cc = status.current_change.expect("current_change present");
    assert_eq!(cc.change_id, row.change_id);
    assert_eq!(cc.state, "in_progress");
    assert_eq!(cc.actor.host_id, me.host_id);
}

// ----- 2.4 current_change reject：in_progress 但 host_id 不對 -----

#[tokio::test]
async fn project_status_current_change_null_when_actor_host_id_does_not_match() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let sm = LocalStateMachineStore::new(resolve_state_root(&working));
    let other = Actor {
        agent_host: "cli".into(),
        os_user: "test".into(),
        host_id: "some-other-host-uuid".into(),
    };
    let row = ops.create_change(&working, "abc").await.expect("create");
    let v = sm
        .transition_state(
            "abc",
            row.version as u64,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→ready")
        .version;
    sm.transition_state(
        "abc",
        v,
        TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: Some(Some(other)),
            reason: speclink_provider::StateTransitionReason::ApplyStart,
        },
    )
    .await
    .expect("→in_progress");

    let status = project_status(&working).await.expect("status");
    assert!(status.current_change.is_none());
}

// ----- 2.5 兩個 in_progress 都屬我，取 updated_at 最新 -----

#[tokio::test]
async fn project_status_current_change_picks_latest_updated_at_when_multiple_match() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let sm = LocalStateMachineStore::new(resolve_state_root(&working));
    let me = Actor {
        agent_host: "cli".into(),
        os_user: "test".into(),
        host_id: resolve_host_id(),
    };

    // older
    let row1 = ops.create_change(&working, "older").await.expect("create");
    let v = sm
        .transition_state(
            "older",
            row1.version as u64,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→ready")
        .version;
    sm.transition_state(
        "older",
        v,
        TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: Some(Some(me.clone())),
            reason: speclink_provider::StateTransitionReason::ApplyStart,
        },
    )
    .await
    .expect("→in_progress");

    // ensure updated_at differs (確保 monotonic time)；test-only blocking sleep
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // newer
    let row2 = ops.create_change(&working, "newer").await.expect("create");
    let v = sm
        .transition_state(
            "newer",
            row2.version as u64,
            TransitionRequest {
                to_state: ChangeState::Ready,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→ready")
        .version;
    sm.transition_state(
        "newer",
        v,
        TransitionRequest {
            to_state: ChangeState::InProgress,
            actor: Some(Some(me.clone())),
            reason: speclink_provider::StateTransitionReason::ApplyStart,
        },
    )
    .await
    .expect("→in_progress");

    let status = project_status(&working).await.expect("status");
    let cc = status.current_change.expect("current_change present");
    assert_eq!(cc.change_id, row2.change_id, "newest in_progress wins");
}

// ----- 2.6 working_dir 不在 SpecLink 專案 → NotInitialized -----

#[tokio::test]
async fn project_status_returns_project_not_initialized_outside_project() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    // 不跑 bootstrap.init() — .speclink/ 不存在
    let err = project_status(&working).await.expect_err("must error");
    assert!(
        matches!(err, RuntimeError::NotInitialized { .. }),
        "expected NotInitialized, got {err:?}"
    );
}

// ----- 2.7 任何 seed → discussions_count == {active:0, converged:0} -----

#[tokio::test]
async fn project_status_discussions_count_is_always_zero_zero_in_p1() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "any-change")
        .await
        .expect("seed");

    let status = project_status(&working).await.expect("status");
    assert_eq!(status.discussions_count.active, 0);
    assert_eq!(status.discussions_count.converged, 0);
}
