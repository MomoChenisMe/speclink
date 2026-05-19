//! `propose create` JSON output 的 insta snapshot 鎖定。
//!
//! 每個 snapshot 透過固定 `SPECLINK_TEST_REQUEST_ID` 環境變數確保 `requestId` 在 CI / 多次
//! 執行間穩定。若要更新 snapshot，請執行 `cargo insta accept`。

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

#[test]
fn success_envelope_snapshot() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "test summary",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("propose_create_success", pretty(&stdout));
}

#[test]
fn change_already_exists_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".speclink/changes/demo")).unwrap();
    std::fs::write(root.join(".speclink/changes/demo/proposal.md"), "EXISTING").unwrap();

    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("propose_create_change_already_exists", pretty(&stdout));
}

#[test]
fn fallback_warning_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join(".speclink")).unwrap();
    std::fs::write(
        root.join(".speclink").join("config.toml"),
        "provider = \"acme\"\nfallback = \"local\"\n",
    )
    .unwrap();

    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("propose_create_fallback_warning", pretty(&stdout));
}
