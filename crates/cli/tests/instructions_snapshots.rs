//! `instructions` JSON output 的 insta snapshot 鎖定。
//!
//! 透過固定 `SPECLINK_TEST_REQUEST_ID` 確保 `requestId` 穩定。Snapshot 內容含
//! 完整 `instruction` / `template` 文字 — 任何 instructions markdown 內容變動
//! 都會觸發 snapshot diff，請以 `cargo insta accept` 更新。

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

fn bootstrap(cwd: &std::path::Path, change: &str) {
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
}

#[test]
fn instructions_design_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "design", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "instructions_design_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn instructions_tasks_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "tasks", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "instructions_tasks_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn instructions_spec_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args([
            "instructions",
            "spec",
            "--change",
            "demo",
            "--capability",
            "user-auth",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "instructions_spec_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn instructions_change_not_found_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let out = cmd(root)
        .args(["instructions", "design", "--change", "missing", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    insta::assert_snapshot!(
        "instructions_change_not_found",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}
