//! `artifact write` JSON output 的 insta snapshot 鎖定。
//!
//! 透過固定 `SPECLINK_TEST_REQUEST_ID` 確保 `requestId` 穩定；若要更新 snapshot，請執行
//! `cargo insta accept`。

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

fn bootstrap(root: &std::path::Path) {
    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "t",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn design_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("design body\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("artifact_write_design_success", pretty(&stdout));
}

#[test]
fn spec_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root);
    let out = cmd(root)
        .args([
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "user-auth",
            "--stdin",
            "--json",
        ])
        .write_stdin("spec body\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("artifact_write_spec_success", pretty(&stdout));
}

#[test]
fn artifact_already_exists_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root);
    // 第一次成功
    let _ = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("OLD\n")
        .output()
        .unwrap();
    // 第二次失敗 → already_exists
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("NEW\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    // 失敗 envelope 的 message 含 change-specific 描述；snapshot 同時鎖 code 與 message。
    insta::assert_snapshot!("artifact_write_already_exists", pretty(&stdout));
}
