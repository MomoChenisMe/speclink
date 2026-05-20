//! `speclink status` end-to-end 命令介面測試。

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

fn bootstrap_demo(cwd: &std::path::Path) {
    let out = cmd(cwd)
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
        .expect("propose");
    assert_eq!(out.status.code(), Some(0));
}

fn write_artifact(cwd: &std::path::Path, args: &[&str], stdin: &str) {
    let out = cmd(cwd).args(args).write_stdin(stdin).output().expect("aw");
    assert_eq!(
        out.status.code(),
        Some(0),
        "artifact write failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn fresh_change_status_three_entries() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);

    let out = cmd(root)
        .args(["status", "--change", "demo", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["data"]["changeId"], "demo");
    assert_eq!(env["data"]["state"], "proposed");
    let artifacts = env["data"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 3);
    assert_eq!(artifacts[0]["id"], "proposal");
    assert_eq!(artifacts[0]["status"], "done");
    assert_eq!(artifacts[0]["required"], true);
    assert_eq!(artifacts[0]["dependencies"].as_array().unwrap().len(), 0);
    assert_eq!(artifacts[1]["id"], "design");
    assert_eq!(artifacts[1]["status"], "missing");
    assert_eq!(artifacts[1]["required"], false);
    assert_eq!(artifacts[2]["id"], "tasks");
    assert_eq!(artifacts[2]["status"], "missing");
}

#[test]
fn status_with_design_and_spec() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    write_artifact(
        root,
        &[
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ],
        "d\n",
    );
    write_artifact(
        root,
        &[
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "auth",
            "--stdin",
            "--json",
        ],
        "a\n",
    );

    let out = cmd(root)
        .args(["status", "--change", "demo", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    let artifacts = env["data"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 4);
    let ids: Vec<&str> = artifacts
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["proposal", "design", "tasks", "spec:auth"]);
    assert_eq!(artifacts[3]["required"], true);
    assert_eq!(
        artifacts[3]["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<&str>>(),
        vec!["proposal"]
    );
}

#[test]
fn change_not_found_exit_1() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["status", "--change", "missing", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "change.not_found");
}

#[test]
fn malformed_metadata_is_internal_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let dir = root.join(".speclink/changes/broken");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("metadata.json"), "{bad json").unwrap();

    let out = cmd(root)
        .args(["status", "--change", "broken", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "internal.error");
}

#[test]
fn status_invalid_change_id_clap_error_exit_2() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["status", "--change", "Add-Feature", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn status_does_not_create_files() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["status", "--change", "missing", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    // 不應該為查詢失敗而建立任何 change 目錄
    assert!(!tmp.path().join(".speclink/changes/missing").exists());
}

#[test]
fn status_rejects_stdin_flag() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args(["status", "--change", "demo", "--stdin", "--json"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}
