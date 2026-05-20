//! `status` JSON output 的 insta snapshot 鎖定。

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
fn status_only_proposal_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root);
    let out = cmd(root)
        .args(["status", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "status_only_proposal",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn status_with_design_and_spec_snapshot() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root);
    let _ = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("d\n")
        .output()
        .unwrap();
    let _ = cmd(root)
        .args([
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "auth",
            "--stdin",
            "--json",
        ])
        .write_stdin("a\n")
        .output()
        .unwrap();
    let out = cmd(root)
        .args(["status", "--change", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!(
        "status_with_design_and_spec",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}

#[test]
fn status_change_not_found_snapshot() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["status", "--change", "missing", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    insta::assert_snapshot!(
        "status_change_not_found",
        pretty(&String::from_utf8(out.stdout).unwrap())
    );
}
