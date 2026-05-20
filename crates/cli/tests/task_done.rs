//! `speclink task done` end-to-end CLI 整合測試。

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const FIXED_REQ: &str = "req_00000000000000000000000000000000";

fn cmd(cwd: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("speclink").expect("cargo bin");
    c.current_dir(cwd);
    c.env_remove("SPECLINK_PROVIDER");
    c.env("SPECLINK_TEST_REQUEST_ID", FIXED_REQ);
    c.env("SPECLINK_CONFIG_HOME", cwd.join("__no_global__"));
    c
}

fn bootstrap(cwd: &std::path::Path, change: &str, tasks_body: &str) {
    let out = cmd(cwd)
        .args([
            "propose",
            "create",
            "--change",
            change,
            "--summary",
            "test",
            "--json",
        ])
        .output()
        .expect("propose");
    assert_eq!(out.status.code(), Some(0));
    let out = cmd(cwd)
        .args([
            "artifact", "write", "tasks", "--change", change, "--stdin", "--json",
        ])
        .write_stdin(tasks_body.to_string())
        .output()
        .expect("tasks");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn task_done_happy_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 Write tests\n");

    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["ok"], true);
    assert_eq!(env["data"]["changeId"], "demo");
    assert_eq!(env["data"]["taskId"], "1.1");
    assert_eq!(env["data"]["previousStatus"], "todo");
    assert_eq!(env["data"]["currentStatus"], "done");
    assert_eq!(env["data"]["taskDescription"], "Write tests");
    // 確認 tasks.md 已更新
    let tasks_md = root.join(".speclink/changes/demo/tasks.md");
    let content = std::fs::read_to_string(&tasks_md).unwrap();
    assert!(content.contains("- [x] 1.1 Write tests"));
}

#[test]
fn task_done_idempotent() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [x] 1.1 Already\n");
    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["data"]["previousStatus"], "done");
    assert_eq!(env["data"]["currentStatus"], "done");
}

#[test]
fn task_done_not_found_returns_exit_2() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n");
    let out = cmd(root)
        .args(["task", "done", "1.99", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "task.not_found");
}

#[test]
fn task_done_invalid_id_returns_exit_2() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n");
    let out = cmd(root)
        .args(["task", "done", "1.1.2", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "task.invalid_id");
}

#[test]
fn task_done_missing_tasks_md_returns_exit_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // propose 但不寫 tasks
    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "test",
            "--json",
        ])
        .output()
        .expect("propose");
    assert_eq!(out.status.code(), Some(0));

    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(1));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "artifact.missing");
}

#[test]
fn task_done_change_not_found_returns_exit_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "missing", "--json"])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(1));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "change.not_found");
}

#[test]
fn task_done_stdin_flag_rejected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n");
    let out = cmd(root)
        .args([
            "task", "done", "1.1", "--change", "demo", "--stdin", "--json",
        ])
        .output()
        .expect("task done");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn task_done_output_uses_lf_line_ending() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n");
    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .expect("task done");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.ends_with('\n'));
    assert!(!stdout.contains('\r'), "stdout must not contain CR");
}
