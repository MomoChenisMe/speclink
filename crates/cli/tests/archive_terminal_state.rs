//! 對 archive 完的 change 跑 A3 既有 op，斷言 A3 spec 內 unreachable scenario 全部可達且
//! 行為對齊：
//! (a) `apply.start` on archived → exit 0 + `data.message='Change is archived.'`
//! (b) `apply.pause` on archived → exit 7 `state.transition_invalid`
//! (c) `task.done 1` on archived → exit 7 `state.transition_invalid`
//! (d) `task.undo 1` on archived → exit 7 `state.transition_invalid`
//!
//! 對應 state-machine spec requirement「`archived` state SHALL be terminal — all
//! subsequent `apply.*` and `task.*` operations SHALL be rejected or returned as
//! hints without mutation」。

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

fn git_init(dir: &Path) {
    Command::new("git")
        .args(["init", "--quiet", "--initial-branch=main"])
        .current_dir(dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "t@e.com"])
        .current_dir(dir)
        .output()
        .ok();
    Command::new("git")
        .args(["config", "user.name", "t"])
        .current_dir(dir)
        .output()
        .ok();
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn speclink(working: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("speclink").expect("binary");
    cmd.current_dir(working);
    cmd
}

fn write_stdin(
    working: &Path,
    kind: &str,
    change: &str,
    cap: Option<&str>,
    body: &[u8],
) -> std::process::Output {
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
    child.wait_with_output().expect("wait")
}

fn drive_to_archived(working: &Path, name: &str) {
    speclink(working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "new", "change", name])
        .assert()
        .success();
    let _ = write_stdin(working, "proposal", name, None, b"## Why\n");
    let _ = write_stdin(
        working,
        "spec",
        name,
        Some("ua"),
        b"## ADDED Requirements\n\n### Requirement: x\n\n#### Scenario: y\n\n- **WHEN** z\n- **THEN** w\n",
    );
    let _ = write_stdin(working, "tasks", name, None, b"- [ ] step1\n");
    speclink(working)
        .args(["--json", "apply", "start", name, "--actor", "c"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "task", "done", "1", "--change", name])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "archive", name])
        .assert()
        .success();
}

#[test]
fn apply_start_on_archived_returns_hint_message_with_exit_0() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_archived(&working, "demo");
    let out = speclink(&working)
        .args(["--json", "apply", "start", "demo"])
        .output()
        .expect("apply start on archived");
    assert!(out.status.success(), "exit 0 expected: {out:?}");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["ok"], true);
    let msg = env["data"]["message"].as_str().unwrap_or("");
    assert_eq!(msg, "Change is archived.");
}

#[test]
fn apply_pause_on_archived_rejects_with_state_transition_invalid_exit_7() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_archived(&working, "demo");
    let out = speclink(&working)
        .args(["--json", "apply", "pause", "demo"])
        .output()
        .expect("apply pause on archived");
    assert_eq!(out.status.code().unwrap_or(0), 7);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "state.transition_invalid");
}

#[test]
fn task_done_on_archived_rejects_with_state_transition_invalid_exit_7() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_archived(&working, "demo");
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .output()
        .expect("task done on archived");
    assert_eq!(out.status.code().unwrap_or(0), 7);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["error"]["code"], "state.transition_invalid");
}

#[test]
fn task_undo_on_archived_rejects_with_state_transition_invalid_exit_7() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_archived(&working, "demo");
    let out = speclink(&working)
        .args(["--json", "task", "undo", "1", "--change", "demo"])
        .output()
        .expect("task undo on archived");
    assert_eq!(out.status.code().unwrap_or(0), 7);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["error"]["code"], "state.transition_invalid");
}
