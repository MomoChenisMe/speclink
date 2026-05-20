//! `task done` JSON output 的 insta snapshot 鎖定。

use assert_cmd::Command;
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

fn pretty(s: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("json");
    serde_json::to_string_pretty(&v).unwrap()
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
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let out = cmd(cwd)
        .args([
            "artifact", "write", "tasks", "--change", change, "--stdin", "--json",
        ])
        .write_stdin(tasks_body.to_string())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn task_done_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 Write tests\n");
    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "task_done_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn task_done_idempotent_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [x] 1.1 Already\n");
    let out = cmd(root)
        .args(["task", "done", "1.1", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "task_done_idempotent",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn task_done_not_found_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n");
    let out = cmd(root)
        .args(["task", "done", "1.99", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    insta::assert_snapshot!(
        "task_done_not_found",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}
