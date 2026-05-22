//! CLI integration test for `apply start` / `apply pause` lifecycle。
//!
//! 對應 spec apply scenarios + design.md Observable behavior。實機 spawn `speclink`
//! binary 而非呼叫 runtime fn 直接驗證 CLI surface。

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

fn write_artifact(working: &Path, kind: &str, change: &str, cap: Option<&str>, body: &[u8]) {
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
    assert!(out.status.success(), "write artifact failed: {out:?}");
}

fn parse_data(stdout: &[u8]) -> serde_json::Value {
    let env: serde_json::Value = serde_json::from_slice(stdout).expect("json");
    env["data"].clone()
}

#[test]
fn apply_lifecycle_full_e2e_ready_to_in_progress_to_ready_to_in_progress() {
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

    // Write 3 artifacts → DAG complete → auto-transition proposing → ready.
    write_artifact(&working, "proposal", "demo", None, b"## Why\n");
    write_artifact(
        &working,
        "spec",
        "demo",
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    let tasks_out_bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(tasks_out_bin);
    cmd.current_dir(&working)
        .args([
            "--json", "new", "artifact", "tasks", "--change", "demo", "--stdin",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"- [ ] one\n- [ ] two\n")
        .expect("write");
    let tasks_out = child.wait_with_output().expect("wait");
    assert!(tasks_out.status.success());
    let tasks_env: serde_json::Value = serde_json::from_slice(&tasks_out.stdout).expect("json");
    let warnings = tasks_env["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w["code"] == "state_transitioned"),
        "DAG complete write SHALL emit state_transitioned warning"
    );

    // apply start --actor cli → in_progress + actor populated
    let out = speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "cli"])
        .output()
        .expect("apply start");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["state"], "in_progress");
    assert_eq!(data["actor"]["agent_host"], "cli");

    // apply pause → ready + actor cleared
    let out = speclink(&working)
        .args(["--json", "apply", "pause", "demo"])
        .output()
        .expect("apply pause");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["state"], "ready");
    assert!(data["actor"].is_null(), "actor SHALL be cleared by pause");

    // apply start again → idempotent reassign with new actor
    let out = speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "claude-code"])
        .output()
        .expect("apply start #2");
    assert!(out.status.success());
    let data = parse_data(&out.stdout);
    assert_eq!(data["state"], "in_progress");
    assert_eq!(data["actor"]["agent_host"], "claude-code");
}

#[test]
fn apply_start_on_proposing_returns_state_transition_invalid_exit_7() {
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
        .args(["--json", "apply", "start", "demo"])
        .output()
        .expect("start");
    assert!(!out.status.success(), "proposing rejects start");
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 7, "state.transition_invalid → exit 7");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "state.transition_invalid");
    assert_eq!(env["error"]["retryable"], false);
}
