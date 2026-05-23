//! Integration tests for `runtime::change_ops::ChangeOperations`.

use std::path::Path;
use std::process::Command;

use speclink_runtime::{Bootstrap, ChangeOperations, RealGitProbe, RuntimeError};
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

#[tokio::test]
async fn create_change_success_returns_row_with_version_one_and_proposing_state() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let row = ops
        .create_change(&working, "billing-system")
        .await
        .expect("create");
    assert_eq!(row.name, "billing-system");
    assert_eq!(row.state, "proposing");
    assert_eq!(row.version, 1);
    assert_eq!(row.schema_id, "spec-driven");
    assert!(working.join(".speclink/changes/billing-system").is_dir());
}

#[tokio::test]
async fn create_change_rejects_invalid_name() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let err = ops
        .create_change(&working, "BillingSystem")
        .await
        .expect_err("uppercase rejected");
    assert!(matches!(err, RuntimeError::ChangeInvalidName { .. }));
}

#[tokio::test]
async fn create_change_rejects_duplicate_name() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "billing-system")
        .await
        .expect("first create");
    let err = ops
        .create_change(&working, "billing-system")
        .await
        .expect_err("dup");
    assert!(matches!(err, RuntimeError::ChangeDuplicateName { .. }));
}

#[tokio::test]
async fn list_changes_returns_in_updated_at_desc() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "alpha").await.expect("a");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    ops.create_change(&working, "beta").await.expect("b");
    let rows = ops.list_changes(&working).await.expect("list");
    let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["beta", "alpha"]);
}

#[tokio::test]
async fn show_change_lists_artifacts_after_seed() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    let change_dir = working.join(".speclink/changes/foo");
    std::fs::write(change_dir.join("proposal.md"), b"x").unwrap();
    std::fs::write(change_dir.join("design.md"), b"x").unwrap();
    std::fs::create_dir_all(change_dir.join("specs/user-auth")).unwrap();
    std::fs::write(change_dir.join("specs/user-auth/spec.md"), b"x").unwrap();

    let data = ops.show_change(&working, "foo").await.expect("show");
    assert_eq!(data.change.name, "foo");
    assert_eq!(data.artifacts.len(), 3);
    assert!(data.artifacts.iter().any(|a| a.kind == "proposal"));
    assert!(data.artifacts.iter().any(|a| a.kind == "design"));
    assert!(
        data.artifacts
            .iter()
            .any(|a| a.kind == "spec" && a.capability.as_deref() == Some("user-auth"))
    );
}

#[tokio::test]
async fn show_change_empty_returns_no_artifacts() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    let data = ops.show_change(&working, "foo").await.expect("show");
    assert!(data.artifacts.is_empty());
}

#[tokio::test]
async fn show_change_unknown_returns_not_found() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let err = ops
        .show_change(&working, "unknown")
        .await
        .expect_err("unknown");
    assert!(matches!(err, RuntimeError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn delete_change_requires_confirm_name() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    let err = ops
        .delete_change(&working, "foo", None)
        .await
        .expect_err("missing confirm");
    assert!(matches!(err, RuntimeError::ChangeInvalidName { .. }));
    // row + dir intact
    assert!(working.join(".speclink/changes/foo").is_dir());
}

#[tokio::test]
async fn delete_change_rejects_mismatched_confirm_name() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    let err = ops
        .delete_change(&working, "foo", Some("bar"))
        .await
        .expect_err("mismatch confirm");
    assert!(matches!(err, RuntimeError::ChangeInvalidName { .. }));
    assert!(working.join(".speclink/changes/foo").is_dir());
}

#[tokio::test]
async fn delete_change_success() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    ops.delete_change(&working, "foo", Some("foo"))
        .await
        .expect("delete");
    assert!(!working.join(".speclink/changes/foo").exists());
}

#[tokio::test]
async fn delete_change_missing_returns_not_found() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    let err = ops
        .delete_change(&working, "missing", Some("missing"))
        .await
        .expect_err("missing");
    assert!(matches!(err, RuntimeError::ChangeNotFound { .. }));
}

// =====================================================================
// show_change envelope 加 all_tasks_done + next_actions（Group 3）
// 對齊 specs/change-store 新 Requirement
// =====================================================================

use speclink_provider::{ChangeState, StateMachineStore, TransitionRequest};
use speclink_provider_local::LocalStateMachineStore;

fn sm_store(working: &Path) -> LocalStateMachineStore {
    let state_root =
        speclink_runtime::paths::resolve_state_root(&RealGitProbe, working).expect("state root");
    LocalStateMachineStore::new(state_root)
}

async fn transition_to(working: &Path, name: &str, to: ChangeState) {
    let sm = sm_store(working);
    let v = sm
        .get_change_state(name)
        .await
        .expect("get_change_state")
        .version;
    // 走最短合法路徑（依 spec state machine）：proposing → ready → in_progress / reviewing / archived
    // 為 test 簡化：先到 ready（如果 from_state 是 proposing），再走到目標。
    let current = sm.get_change_state(name).await.expect("get").state;
    if current == ChangeState::Proposing && to != ChangeState::Reviewing {
        let v2 = sm
            .transition_state(
                name,
                v,
                TransitionRequest {
                    to_state: ChangeState::Ready,
                    actor: None,
                    reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
                },
            )
            .await
            .expect("→ready")
            .version;
        if to == ChangeState::Ready {
            return;
        }
        let reason = match to {
            ChangeState::InProgress => speclink_provider::StateTransitionReason::ApplyStart,
            ChangeState::Archived => speclink_provider::StateTransitionReason::ArchiveRun,
            _ => speclink_provider::StateTransitionReason::ApplyStart,
        };
        sm.transition_state(
            name,
            v2,
            TransitionRequest {
                to_state: to,
                actor: None,
                reason,
            },
        )
        .await
        .expect("→target");
    } else if current == ChangeState::Proposing && to == ChangeState::Reviewing {
        sm.transition_state(
            name,
            v,
            TransitionRequest {
                to_state: ChangeState::Reviewing,
                actor: None,
                reason: speclink_provider::StateTransitionReason::ArtifactDagComplete,
            },
        )
        .await
        .expect("→reviewing");
    }
}

#[tokio::test]
async fn show_change_lists_artifacts_after_seed_has_new_envelope_fields() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "foo").await.expect("create");
    let data = ops.show_change(&working, "foo").await.expect("show");
    // fresh proposing change → all_tasks_done=false
    assert!(!data.all_tasks_done);
    // proposing + 無 artifact → 三個 artifact.write hint
    assert_eq!(
        data.next_actions,
        vec![
            "artifact.write proposal".to_string(),
            "artifact.write design".to_string(),
            "artifact.write tasks".to_string()
        ]
    );
}

#[tokio::test]
async fn show_change_archived_returns_empty_next_actions() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "a").await.expect("create");
    // 走 ready → archived
    transition_to(&working, "a", ChangeState::Archived).await;
    let data = ops.show_change(&working, "a").await.expect("show");
    assert_eq!(data.next_actions, Vec::<String>::new());
}

#[tokio::test]
async fn show_change_in_progress_with_pending_tasks_returns_task_done_with_first_index() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "wip").await.expect("create");
    transition_to(&working, "wip", ChangeState::InProgress).await;
    // 寫 tasks.md：3 個 `- [x]` 在前、第 4 行為第一個 `- [ ]`（label `2.1`）→ INDEX = 4
    // 對齊 spec change-store Scenario「first unchecked line is `- [ ] 2.1 Implement parser`
    // while three earlier `- [x]` lines have already been checked off」
    let change_dir = working.join(".speclink/changes/wip");
    std::fs::write(
        change_dir.join("tasks.md"),
        "# Tasks\n\n- [x] 1.1 done\n- [x] 1.2 done\n- [x] 1.3 done\n- [ ] 2.1 Implement parser\n- [ ] 3.1 next\n",
    )
    .unwrap();
    let data = ops.show_change(&working, "wip").await.expect("show");
    // emit INDEX 4（task.done CLI 收的整數），不是 label "2.1"
    assert_eq!(data.next_actions, vec!["task.done 4".to_string()]);
}

#[tokio::test]
async fn show_change_in_progress_all_tasks_done_returns_archive_run() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "done").await.expect("create");
    transition_to(&working, "done", ChangeState::InProgress).await;
    // 把 change 的 all_tasks_done 設為 true
    let sm = sm_store(&working);
    let v = sm.get_change_state("done").await.expect("v").version;
    sm.set_all_tasks_done("done", v, true)
        .await
        .expect("set done");
    let data = ops.show_change(&working, "done").await.expect("show");
    assert!(data.all_tasks_done);
    assert_eq!(data.next_actions, vec!["archive.run".to_string()]);
}

#[tokio::test]
async fn show_change_in_progress_without_tasks_md_returns_bare_task_done() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "notasks")
        .await
        .expect("create");
    transition_to(&working, "notasks", ChangeState::InProgress).await;
    // 不寫 tasks.md
    let data = ops.show_change(&working, "notasks").await.expect("show");
    assert!(!data.all_tasks_done);
    assert_eq!(data.next_actions, vec!["task.done".to_string()]);
}

#[tokio::test]
async fn show_change_proposing_filters_existing_artifacts() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "p").await.expect("create");
    let change_dir = working.join(".speclink/changes/p");
    std::fs::write(change_dir.join("proposal.md"), b"x").unwrap();
    let data = ops.show_change(&working, "p").await.expect("show");
    // proposal 已存在 → 只剩 design + tasks
    assert_eq!(
        data.next_actions,
        vec![
            "artifact.write design".to_string(),
            "artifact.write tasks".to_string()
        ]
    );
}

#[tokio::test]
async fn show_change_proposing_with_all_three_artifacts_returns_empty() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "p3").await.expect("create");
    let change_dir = working.join(".speclink/changes/p3");
    std::fs::write(change_dir.join("proposal.md"), b"x").unwrap();
    std::fs::write(change_dir.join("design.md"), b"x").unwrap();
    std::fs::write(change_dir.join("tasks.md"), b"# Tasks\n").unwrap();
    let data = ops.show_change(&working, "p3").await.expect("show");
    assert_eq!(data.next_actions, Vec::<String>::new());
}

#[tokio::test]
async fn show_change_ready_returns_apply_start() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "rdy").await.expect("create");
    transition_to(&working, "rdy", ChangeState::Ready).await;
    let data = ops.show_change(&working, "rdy").await.expect("show");
    assert_eq!(data.next_actions, vec!["apply.start".to_string()]);
}

#[tokio::test]
async fn show_change_reviewing_returns_review_pair() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "rev").await.expect("create");
    transition_to(&working, "rev", ChangeState::Reviewing).await;
    let data = ops.show_change(&working, "rev").await.expect("show");
    assert_eq!(
        data.next_actions,
        vec!["review.approve".to_string(), "review.reject".to_string()]
    );
}

#[tokio::test]
async fn show_change_code_reviewing_returns_review_pair() {
    let (_tmp, working) = fresh_project().await;
    let ops = ChangeOperations::new(RealGitProbe);
    ops.create_change(&working, "cr").await.expect("create");
    // proposing → ready → in_progress → code_reviewing 需要 require_code_review=true。
    // 直接用 set_all_tasks_done 後手動 transition_state 強制到 code_reviewing。
    transition_to(&working, "cr", ChangeState::InProgress).await;
    let sm = sm_store(&working);
    let v = sm.get_change_state("cr").await.expect("v").version;
    sm.transition_state(
        "cr",
        v,
        TransitionRequest {
            to_state: ChangeState::CodeReviewing,
            actor: None,
            reason: speclink_provider::StateTransitionReason::TaskDoneAuto,
        },
    )
    .await
    .expect("→code_reviewing");
    let data = ops.show_change(&working, "cr").await.expect("show");
    assert_eq!(
        data.next_actions,
        vec!["review.approve".to_string(), "review.reject".to_string()]
    );
}
