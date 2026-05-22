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
