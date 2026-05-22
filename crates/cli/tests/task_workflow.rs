//! CLI integration test for `task list / done / undo` workflow + Windows-friendly
//! atomic rename regression sanity check.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "git command failed: {cmd:?}");
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

fn speclink(working: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("speclink").expect("binary");
    cmd.current_dir(working);
    cmd
}

fn write_artifact_stdin(working: &Path, kind: &str, change: &str, cap: Option<&str>, body: &[u8]) {
    let bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(bin);
    cmd.current_dir(working)
        .arg("--json")
        .arg("new")
        .arg("artifact")
        .arg(kind)
        .arg("--change")
        .arg(change)
        .arg("--stdin");
    if let Some(c) = cap {
        cmd.arg("--capability").arg(c);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body)
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    assert!(
        out.status.success(),
        "write artifact failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn parse_data(stdout: &[u8]) -> serde_json::Value {
    let env: serde_json::Value = serde_json::from_slice(stdout).expect("json");
    env["data"].clone()
}

fn setup_change_in_progress(working: &Path, name: &str) {
    speclink(working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "new", "change", name])
        .assert()
        .success();
    write_artifact_stdin(working, "proposal", name, None, b"## Why\n");
    write_artifact_stdin(
        working,
        "spec",
        name,
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    write_artifact_stdin(working, "tasks", name, None, b"- [ ] one\n- [ ] two\n");
    // After tasks write, state should be ready (DAG complete via auto-transition).
    speclink(working)
        .args(["--json", "apply", "start", name, "--actor", "cli"])
        .assert()
        .success();
}

#[test]
fn task_list_then_done_progression_under_walking_skeleton() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    setup_change_in_progress(&working, "demo");

    // list
    let out = speclink(&working)
        .args(["--json", "task", "list", "--change", "demo"])
        .output()
        .expect("list");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    let tasks = data["tasks"].as_array().expect("tasks array");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["index"], 1);
    assert_eq!(tasks[0]["text"], "one");

    // done 1 → not all done
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .output()
        .expect("done 1");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["index"], 1);
    assert_eq!(data["done"], true);
    assert_eq!(data["all_tasks_done"], false);
    assert_eq!(data["state"], "in_progress");

    // idempotent re-done 1 → no rewrite, still done
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .output()
        .expect("done 1 again");
    assert!(out.status.success());

    // done 2 → all_tasks_done=true under walking-skeleton, state stays in_progress
    let out = speclink(&working)
        .args(["--json", "task", "done", "2", "--change", "demo"])
        .output()
        .expect("done 2");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["all_tasks_done"], true);
    assert_eq!(data["state"], "in_progress");
    assert_eq!(data["auto_transitioned"], false);

    // undo 2 → all_tasks_done cleared, state stays in_progress, reverted_from=null
    let out = speclink(&working)
        .args(["--json", "task", "undo", "2", "--change", "demo"])
        .output()
        .expect("undo 2");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["done"], false);
    assert_eq!(data["all_tasks_done"], false);
    assert!(data["reverted_from"].is_null());
}

#[test]
fn task_done_out_of_range_returns_exit_2() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    setup_change_in_progress(&working, "demo");
    let out = speclink(&working)
        .args(["--json", "task", "done", "99", "--change", "demo"])
        .output()
        .expect("done oor");
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap_or(-1), 2);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["error"]["code"], "task.index_out_of_range");
}

#[test]
fn task_list_returns_no_tasks_file_when_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "task", "list", "--change", "demo"])
        .output()
        .expect("list");
    assert!(!out.status.success());
    assert_eq!(out.status.code().unwrap_or(-1), 2);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["error"]["code"], "task.no_tasks_file");
}

/// Cross-platform atomic rename regression: tasks.md rewrite uses tempfile-then-rename;
/// on Windows the rename path must avoid sharing violation. This test exercises the
/// path on whatever platform CI runs (single test SHALL pass on Linux/macOS/Windows).
#[test]
fn task_done_atomic_rewrite_does_not_lock_or_orphan_tempfile() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    setup_change_in_progress(&working, "demo");
    speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();
    // After rewrite, dir SHALL contain only tasks.md (no orphan tempfile).
    let dir = working.join(".speclink").join("changes").join("demo");
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let has_orphan_tmp = entries
        .iter()
        .any(|n| n.starts_with(".tmp") || n.ends_with(".tmp"));
    assert!(
        !has_orphan_tmp,
        "no orphan tempfile after rename; got entries: {entries:?}"
    );
    // tasks.md content reflects mark.
    let body = std::fs::read_to_string(dir.join("tasks.md")).expect("read");
    assert!(body.contains("- [x] one"));
}
