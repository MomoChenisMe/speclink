//! Integration tests for runtime path resolution + git probe.
//!
//! TDD red phase: these tests run against the stub `RealGitProbe` and will
//! fail until 5.2 implements the actual git shell-out.

use std::path::Path;
use std::process::Command;

use speclink_runtime::{RealGitProbe, RuntimeError, resolve_state_root};
use tempfile::TempDir;

fn assert_git_available() {
    let ok = Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "test prerequisite: `git` must be on PATH");
}

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn git");
    assert!(
        out.status.success(),
        "git command failed: {:?}\nstdout={}\nstderr={}",
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

fn git_commit_empty(dir: &Path) {
    run(Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir));
}

#[test]
fn state_root_resolves_to_dot_git_speclink_in_fresh_repo() {
    assert_git_available();
    let tmp = TempDir::new().expect("tempdir");
    git_init(tmp.path());

    let probe = RealGitProbe;
    let state_root = resolve_state_root(&probe, tmp.path()).expect("resolve");
    let expected = tmp.path().join(".git").join("speclink");
    let canonical_actual = state_root
        .canonicalize()
        .unwrap_or_else(|_| state_root.clone());
    let canonical_expected = expected.canonicalize().unwrap_or(expected.clone());
    // 直到 .git/speclink 真的存在前 canonicalize 會失敗；只比對 path semantic。
    assert_eq!(
        canonical_actual
            .components()
            .rev()
            .take(2)
            .collect::<Vec<_>>(),
        canonical_expected
            .components()
            .rev()
            .take(2)
            .collect::<Vec<_>>(),
        "state_root tail should be `.git/speclink`, got {state_root:?}"
    );
}

#[test]
fn state_root_in_linked_worktree_points_to_main_git_dir() {
    assert_git_available();
    let tmp_owned = TempDir::new().expect("tempdir");
    // macOS 的 /var -> /private/var symlink 會讓 git 在 main 與 worktree 回不同字面路徑；
    // canonicalize working tree root 後再用相同 base。
    let tmp = tmp_owned.path().canonicalize().expect("canonicalize tmp");
    let main = tmp.join("main");
    std::fs::create_dir(&main).expect("main dir");
    git_init(&main);
    git_commit_empty(&main);
    let wt = tmp.join("wt");
    run(Command::new("git")
        .args(["worktree", "add", "-b", "wt-branch"])
        .arg(&wt)
        .current_dir(&main));

    let probe = RealGitProbe;
    let main_state_root = resolve_state_root(&probe, &main).expect("main");
    let wt_state_root = resolve_state_root(&probe, &wt).expect("worktree");
    assert_eq!(
        main_state_root, wt_state_root,
        "linked worktree state_root must equal main repo's state_root"
    );
}

#[test]
fn resolve_state_root_returns_requires_git_when_not_in_repo() {
    let tmp = TempDir::new().expect("tempdir");
    let probe = RealGitProbe;
    let err = resolve_state_root(&probe, tmp.path()).expect_err("expected error");
    assert!(
        matches!(err, RuntimeError::RequiresGit { .. }),
        "expected RequiresGit, got {err:?}"
    );
}
