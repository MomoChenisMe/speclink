//! `archive` JSON output 的 insta snapshot 鎖定。
//!
//! 透過固定 `SPECLINK_TEST_REQUEST_ID` 與 `SPECLINK_TEST_ARCHIVE_DATE` 確保 `requestId`
//! 與 archive 目錄前綴穩定。若需更新 snapshot，請執行 `cargo insta accept`。

use assert_cmd::Command;
use tempfile::TempDir;

const FIXED_REQ: &str = "req_00000000000000000000000000000000";
const FIXED_DATE: &str = "2026-05-19";

fn cmd(cwd: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("speclink").expect("cargo bin");
    c.current_dir(cwd);
    c.env_remove("SPECLINK_PROVIDER");
    c.env("SPECLINK_TEST_REQUEST_ID", FIXED_REQ);
    c.env("SPECLINK_TEST_ARCHIVE_DATE", FIXED_DATE);
    c.env("SPECLINK_CONFIG_HOME", cwd.join("__no_global__"));
    c
}

fn pretty(s: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("json");
    // Mask archivedAt timestamp to a placeholder for stable snapshot.
    let v = mask_archived_at(v);
    serde_json::to_string_pretty(&v).unwrap()
}

fn mask_archived_at(mut v: serde_json::Value) -> serde_json::Value {
    if let Some(data) = v.get_mut("data") {
        if data.is_object() {
            if let Some(obj) = data.as_object_mut() {
                if obj.contains_key("archivedAt") {
                    obj.insert(
                        "archivedAt".to_string(),
                        serde_json::Value::String("<TIMESTAMP>".to_string()),
                    );
                }
            }
        }
    }
    v
}

fn bootstrap(cwd: &std::path::Path, change: &str, capability: &str, delta_body: &str) {
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
            "artifact",
            "write",
            "spec",
            "--change",
            change,
            "--capability",
            capability,
            "--stdin",
            "--json",
        ])
        .write_stdin(delta_body)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn archive_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody\n",
    );
    let out = cmd(root)
        .args(["archive", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "archive_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn archive_dry_run_success_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody\n",
    );
    let out = cmd(root)
        .args(["archive", "demo", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "archive_dry_run_success",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn archive_delta_conflict_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "first",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody1\n",
    );
    let out = cmd(root)
        .args(["archive", "first", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    bootstrap(
        root,
        "second",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody2\n",
    );
    let out = cmd(root)
        .args(["archive", "second", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(7));
    insta::assert_snapshot!(
        "archive_delta_conflict",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}
