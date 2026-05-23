//! Integration tests for `TaskOperations` (list / done / undo).
//!
//! 對應 spec requirement「`speclink task list` SHALL enumerate checkbox lines from
//! tasks.md by 1-based index」、「`speclink task done` SHALL mark exactly one checkbox
//! by 1-based index and auto-trigger when all tasks complete」、「`speclink task undo`
//! SHALL unmark exactly one checkbox by 1-based index and revert auto-trigger when
//! needed」、「The transition `code_reviewing → in_progress` triggered by `task.undo`
//! SHALL precede the unmark」與決策「Task id 策略：1-based 行內順序 index，不引入 marker」
//! 與「`task.done` auto-trigger contract」。

use std::fs;
use std::path::Path;
use std::process::Command;

use speclink_provider::{ChangeState, StateMachineStore, StateTransitionReason, TransitionRequest};
use speclink_provider_local::LocalStateMachineStore;
use speclink_runtime::{
    Bootstrap, ChangeOperations, RealGitProbe, RuntimeError, TaskOperations, resolve_state_root,
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

async fn project_with_tasks(name: &str, tasks_body: &str) -> (TempDir, std::path::PathBuf) {
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
    let tasks_path = working
        .join(".speclink")
        .join("changes")
        .join(name)
        .join("tasks.md");
    fs::create_dir_all(tasks_path.parent().unwrap()).expect("dir");
    fs::write(&tasks_path, tasks_body).expect("seed tasks.md");
    (tmp, working)
}

async fn force_state(working: &Path, change: &str, to: ChangeState, reason: StateTransitionReason) {
    let state_root = resolve_state_root(&RealGitProbe, working).expect("state");
    let sm = LocalStateMachineStore::new(state_root);
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
    .expect("force");
}

async fn current_state(working: &Path, change: &str) -> (ChangeState, bool) {
    let state_root = resolve_state_root(&RealGitProbe, working).expect("state");
    let sm = LocalStateMachineStore::new(state_root);
    let view = sm.get_change_state(change).await.expect("get");
    (view.state, view.all_tasks_done)
}

#[tokio::test]
async fn task_list_enumerates_checkboxes_in_document_order() {
    let (_tmp, working) =
        project_with_tasks("demo", "# Tasks\n- [ ] one\n- [x] two\n  - [ ] nested\n").await;
    let ops = TaskOperations::new(RealGitProbe);
    let data = ops.list(&working, "demo").expect("list");
    assert_eq!(data.tasks.len(), 3);
    assert_eq!(data.tasks[0].index, 1);
    assert_eq!(data.tasks[0].text, "one");
    assert!(!data.tasks[0].done);
    assert_eq!(data.tasks[1].index, 2);
    assert!(data.tasks[1].done);
    assert_eq!(data.tasks[2].index, 3);
    assert_eq!(data.tasks[2].text, "nested");
}

#[tokio::test]
async fn task_list_missing_file_returns_no_tasks_file() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    Bootstrap::new(RealGitProbe)
        .init(&working, false)
        .await
        .expect("init");
    ChangeOperations::new(RealGitProbe)
        .create_change(&working, "demo")
        .await
        .expect("create");
    let ops = TaskOperations::new(RealGitProbe);
    let err = ops.list(&working, "demo").expect_err("no tasks.md");
    assert!(matches!(err, RuntimeError::TaskNoTasksFile { .. }));
}

#[tokio::test]
async fn task_done_marks_one_checkbox_and_keeps_state_when_not_all_done() {
    let (_tmp, working) = project_with_tasks("demo", "- [ ] a\n- [ ] b\n").await;
    // need state = in_progress for task.done.
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
    let ops = TaskOperations::new(RealGitProbe);
    let (data, _w) = ops.done(&working, "demo", 1).await.expect("done");
    assert_eq!(data.index, 1);
    assert!(data.done);
    assert!(!data.all_tasks_done);
    assert_eq!(data.state, ChangeState::InProgress);
    assert!(!data.auto_transitioned);
    let body = fs::read_to_string(
        working
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("tasks.md"),
    )
    .unwrap();
    assert_eq!(body, "- [x] a\n- [ ] b\n");
}

#[tokio::test]
async fn task_done_idempotent_on_already_done() {
    let (_tmp, working) = project_with_tasks("demo", "- [x] a\n- [ ] b\n").await;
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
    let ops = TaskOperations::new(RealGitProbe);
    let (data, _w) = ops.done(&working, "demo", 1).await.expect("done");
    assert!(data.done);
    assert_eq!(data.state, ChangeState::InProgress);
    let body = fs::read_to_string(
        working
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("tasks.md"),
    )
    .unwrap();
    assert_eq!(body, "- [x] a\n- [ ] b\n", "no rewrite on idempotent done");
}

#[tokio::test]
async fn task_done_out_of_range_rejected() {
    let (_tmp, working) = project_with_tasks("demo", "- [ ] only\n").await;
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
    let ops = TaskOperations::new(RealGitProbe);
    let err = ops.done(&working, "demo", 99).await.expect_err("oor");
    assert!(matches!(err, RuntimeError::TaskIndexOutOfRange { .. }));
}

#[tokio::test]
async fn task_done_last_task_sets_all_tasks_done_under_walking_skeleton() {
    let (_tmp, working) = project_with_tasks("demo", "- [x] a\n- [ ] b\n").await;
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
    let ops = TaskOperations::new(RealGitProbe);
    let (data, _w) = ops.done(&working, "demo", 2).await.expect("last");
    assert!(data.all_tasks_done);
    assert_eq!(data.state, ChangeState::InProgress);
    assert!(!data.auto_transitioned);
    let (state, flag) = current_state(&working, "demo").await;
    assert_eq!(state, ChangeState::InProgress);
    assert!(flag);
}

#[tokio::test]
async fn task_undo_clears_flag_and_unmarks_line() {
    let (_tmp, working) = project_with_tasks("demo", "- [x] a\n- [x] b\n").await;
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
    // Manually set all_tasks_done true via setter to simulate done-then-undo path.
    let state_root = resolve_state_root(&RealGitProbe, &working).expect("state");
    let sm = LocalStateMachineStore::new(state_root);
    let view = sm.get_change_state("demo").await.expect("get");
    sm.set_all_tasks_done("demo", view.version, true)
        .await
        .expect("set");

    let ops = TaskOperations::new(RealGitProbe);
    let data = ops.undo(&working, "demo", 1).await.expect("undo");
    assert!(!data.done);
    assert!(!data.all_tasks_done);
    assert_eq!(data.state, ChangeState::InProgress);
    assert!(data.reverted_from.is_none());

    let body = fs::read_to_string(
        working
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("tasks.md"),
    )
    .unwrap();
    assert_eq!(body, "- [ ] a\n- [x] b\n");
}

#[tokio::test]
async fn task_undo_from_code_reviewing_transitions_to_in_progress_first() {
    // Synthesize a code_reviewing change via raw state machine.
    let (_tmp, working) = project_with_tasks("demo", "- [x] a\n- [x] b\n").await;
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
    let state_root = resolve_state_root(&RealGitProbe, &working).expect("state");
    let sm = LocalStateMachineStore::new(state_root);
    let view = sm.get_change_state("demo").await.expect("get");
    sm.set_all_tasks_done("demo", view.version, true)
        .await
        .expect("set");

    let ops = TaskOperations::new(RealGitProbe);
    let data = ops.undo(&working, "demo", 1).await.expect("undo");
    assert_eq!(data.state, ChangeState::InProgress);
    assert!(!data.all_tasks_done);
    assert_eq!(data.reverted_from.as_deref(), Some("code_reviewing"));
}

#[tokio::test]
async fn task_undo_idempotent_on_already_unmarked() {
    let (_tmp, working) = project_with_tasks("demo", "- [ ] a\n- [x] b\n").await;
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
    let ops = TaskOperations::new(RealGitProbe);
    let data = ops.undo(&working, "demo", 1).await.expect("undo");
    assert!(!data.done);
    let body = fs::read_to_string(
        working
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("tasks.md"),
    )
    .unwrap();
    assert_eq!(
        body, "- [ ] a\n- [x] b\n",
        "idempotent undo SHALL NOT rewrite tasks.md"
    );
}
