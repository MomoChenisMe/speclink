//! Integration tests for runtime status / link / unlink.

use std::fs;
use std::path::Path;
use std::process::Command;

use speclink_runtime::{Bootstrap, Operations, RealGitProbe, RuntimeError};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "command failed: {:?}", cmd);
}

fn git_init_with_commit(dir: &Path) {
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
    run(Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn sha256_of(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(p).expect("read");
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

#[tokio::test]
async fn status_returns_metadata() {
    let tmp = TempDir::new().unwrap();
    let working = canonical(tmp.path());
    git_init_with_commit(&working);
    let boot = Bootstrap::new(RealGitProbe);
    let info = boot.init(&working, false).await.expect("init");
    let ops = Operations::new(RealGitProbe);
    let status = ops.status(&working).await.expect("status");
    assert_eq!(status.project_id, info.project_id);
    assert_eq!(status.provider, "local");
    assert_eq!(status.artifact_root, ".speclink");
    assert_eq!(status.state_root, ".git/speclink");
    assert!(
        !status.git_head.is_empty(),
        "git_head must be non-empty after a commit"
    );
    assert!(status.requires_git);
}

#[tokio::test]
async fn status_without_init() {
    let tmp = TempDir::new().unwrap();
    let working = canonical(tmp.path());
    git_init_with_commit(&working);
    let ops = Operations::new(RealGitProbe);
    let err = ops.status(&working).await.expect_err("status must fail");
    assert!(matches!(err, RuntimeError::NotInitialized { .. }));
}

#[tokio::test]
async fn link_known_project() {
    let tmp = TempDir::new().unwrap();
    let working = canonical(tmp.path());
    git_init_with_commit(&working);
    let boot = Bootstrap::new(RealGitProbe);
    let info = boot.init(&working, false).await.expect("init");
    // Remove link.yaml to simulate "re-bind after fresh clone".
    fs::remove_file(working.join(".speclink").join("link.yaml")).unwrap();
    let ops = Operations::new(RealGitProbe);
    let relinked = ops.link(&working, &info.project_id).await.expect("link");
    assert_eq!(relinked.project_id, info.project_id);
    assert!(working.join(".speclink").join("link.yaml").exists());
}

#[tokio::test]
async fn link_unknown_project() {
    let tmp = TempDir::new().unwrap();
    let working = canonical(tmp.path());
    git_init_with_commit(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    fs::remove_file(working.join(".speclink").join("link.yaml")).unwrap();
    let ops = Operations::new(RealGitProbe);
    let err = ops
        .link(&working, "00000000-0000-0000-0000-000000000000")
        .await
        .expect_err("link should fail");
    assert!(matches!(err, RuntimeError::LinkTargetNotFound { .. }));
    assert!(!working.join(".speclink").join("link.yaml").exists());
}

#[tokio::test]
async fn unlink_preserves_state_and_schemas() {
    let tmp = TempDir::new().unwrap();
    let working = canonical(tmp.path());
    git_init_with_commit(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    fs::write(
        working.join(".speclink").join("schemas").join("spec.json"),
        "{}",
    )
    .expect("seed schema");
    let state_db = working.join(".git").join("speclink").join("state.db");
    let sha_before = sha256_of(&state_db);

    let ops = Operations::new(RealGitProbe);
    ops.unlink(&working).await.expect("unlink");

    assert!(!working.join(".speclink").join("link.yaml").exists());
    assert!(state_db.exists());
    assert_eq!(sha_before, sha256_of(&state_db));
    assert!(
        working
            .join(".speclink")
            .join("schemas")
            .join("spec.json")
            .exists()
    );
}
