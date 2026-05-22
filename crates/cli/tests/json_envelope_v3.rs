//! JSON envelope shape pinning for slice A3 5 new ops + state_transitioned warning。
//!
//! 對應 spec requirement「All five CLI commands SHALL emit JSON envelopes compatible
//! with the bootstrap and A2 contract」。本 test 採 shape assertion 而非 byte-for-byte
//! snapshot（actor_json 內 os_user / host_id 跨平台不穩；改鎖 keys + 類型）。

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

fn write_stdin(working: &Path, kind: &str, change: &str, cap: Option<&str>, body: &[u8]) {
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
    assert!(out.status.success());
}

fn setup(working: &Path) {
    git_init(working);
    speclink(working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();
    write_stdin(working, "proposal", "demo", None, b"## Why\n");
    write_stdin(
        working,
        "spec",
        "demo",
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    write_stdin(working, "tasks", "demo", None, b"- [ ] one\n- [ ] two\n");
    speclink(working)
        .args(["--json", "apply", "start", "demo", "--actor", "cli"])
        .assert()
        .success();
}

fn assert_envelope_success_keys(env: &serde_json::Value) {
    assert_eq!(env["ok"], true);
    assert!(env["data"].is_object());
    assert!(env["warnings"].is_array());
    assert!(env["requestId"].is_string());
    let req = env["requestId"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(req).is_ok(), "requestId is UUID v4");
}

fn assert_envelope_error_keys(env: &serde_json::Value) {
    assert_eq!(env["ok"], false);
    let err = &env["error"];
    assert!(err["code"].is_string());
    assert!(err["message"].is_string());
    assert!(err["hint"].is_string() || err["hint"].is_null());
    assert!(err["retryable"].is_boolean());
    assert!(err["retry_after_ms"].is_null() || err["retry_after_ms"].is_number());
    assert!(env["requestId"].is_string());
}

#[test]
fn apply_start_success_envelope_has_expected_data_shape() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    speclink(&working)
        .args(["--json", "apply", "pause", "demo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "claude-code"])
        .output()
        .expect("start");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_success_keys(&env);
    let d = &env["data"];
    for key in ["change_id", "state", "actor", "message"] {
        assert!(d.get(key).is_some(), "data missing {key}");
    }
    assert_eq!(d["state"], "in_progress");
    assert_eq!(d["actor"]["agent_host"], "claude-code");
}

#[test]
fn apply_pause_success_envelope_clears_actor() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["--json", "apply", "pause", "demo"])
        .output()
        .expect("pause");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_success_keys(&env);
    assert_eq!(env["data"]["state"], "ready");
    assert!(env["data"]["actor"].is_null());
}

#[test]
fn task_list_success_envelope_returns_tasks_array() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["--json", "task", "list", "--change", "demo"])
        .output()
        .expect("list");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_success_keys(&env);
    let tasks = env["data"]["tasks"].as_array().expect("tasks array");
    assert_eq!(tasks.len(), 2);
    for t in tasks {
        for key in ["index", "done", "text"] {
            assert!(t.get(key).is_some());
        }
    }
}

#[test]
fn task_done_success_envelope_has_full_shape() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .output()
        .expect("done");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_success_keys(&env);
    let d = &env["data"];
    for key in [
        "index",
        "done",
        "all_tasks_done",
        "state",
        "auto_transitioned",
    ] {
        assert!(d.get(key).is_some(), "task.done data missing {key}");
    }
    assert_eq!(d["done"], true);
}

#[test]
fn task_undo_success_envelope_has_full_shape() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "task", "undo", "1", "--change", "demo"])
        .output()
        .expect("undo");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_success_keys(&env);
    let d = &env["data"];
    for key in ["index", "done", "all_tasks_done", "state", "reverted_from"] {
        assert!(d.get(key).is_some(), "task.undo data missing {key}");
    }
    assert_eq!(d["done"], false);
}

#[test]
fn error_envelopes_match_shape_for_state_transition_invalid_and_task_index_oor() {
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
    // apply start on proposing → state.transition_invalid (non-retryable)
    let out = speclink(&working)
        .args(["--json", "apply", "start", "demo"])
        .output()
        .expect("start");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_envelope_error_keys(&env);
    assert_eq!(env["error"]["code"], "state.transition_invalid");
    assert_eq!(env["error"]["retryable"], false);
}
