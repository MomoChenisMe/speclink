//! `speclink archive` end-to-end CLI 整合測試。

use assert_cmd::Command;
use serde_json::Value;
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
        .expect("propose");
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
        .expect("spec");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn archive_happy_path_moves_dir_and_creates_main_spec() {
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
        .expect("archive");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["ok"], true);
    assert_eq!(env["data"]["changeId"], "demo");
    assert_eq!(env["data"]["state"], "archived");
    assert_eq!(env["data"]["dryRun"], false);
    assert_eq!(
        env["data"]["archivePath"],
        ".speclink/changes/archive/2026-05-19-demo"
    );
    assert_eq!(
        env["data"]["specSync"]["capabilitiesSynced"][0]["capability"],
        "auth"
    );
    assert_eq!(
        env["data"]["specSync"]["capabilitiesSynced"][0]["addedCount"],
        1
    );
    assert_eq!(
        env["data"]["specSync"]["capabilitiesSynced"][0]["createdMainSpec"],
        true
    );

    // filesystem 驗證
    assert!(!root.join(".speclink/changes/demo").exists());
    assert!(
        root.join(".speclink/changes/archive/2026-05-19-demo")
            .is_dir()
    );
    assert!(root.join(".speclink/specs/auth/spec.md").is_file());
}

#[test]
fn archive_dry_run_no_side_effect() {
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
        .expect("archive");
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["data"]["dryRun"], true);
    assert_eq!(
        env["data"]["archivePath"],
        ".speclink/changes/archive/2026-05-19-demo"
    );
    assert_eq!(
        env["data"]["specSync"]["capabilitiesSynced"][0]["addedCount"],
        1
    );

    // filesystem 不變
    assert!(root.join(".speclink/changes/demo").is_dir());
    assert!(!root.join(".speclink/changes/archive").exists());
    assert!(!root.join(".speclink/specs").exists());
}

#[test]
fn archive_dry_run_reports_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // 先 archive 第一次建立主 spec
    bootstrap(
        root,
        "first",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody\n",
    );
    let out = cmd(root)
        .args(["archive", "first", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    // 第二個 change：delta ADDED 同名 → conflict
    bootstrap(
        root,
        "second",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody2\n",
    );
    // dry-run 應仍報 conflict（exit 7）
    let out = cmd(root)
        .args(["archive", "second", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(7));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "spec.delta_conflict");
}

#[test]
fn archive_change_not_found_exits_1() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["archive", "missing", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "change.not_found");
}

#[test]
fn archive_invalid_change_id_rejected_by_clap() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["archive", "Bad-Name", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn archive_stdin_rejected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    );
    let out = cmd(root)
        .args(["archive", "demo", "--stdin", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn archive_already_archived_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    );
    // First archive succeeds
    let out = cmd(root)
        .args(["archive", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    // Manually restore archive dir to active to simulate "already archived" state
    let archive_dir = root.join(".speclink/changes/archive/2026-05-19-demo");
    let active_dir = root.join(".speclink/changes/demo");
    std::fs::rename(&archive_dir, &active_dir).expect("manual restore");
    // Second archive should be rejected — metadata says archived
    let out = cmd(root)
        .args(["archive", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "archive.change_not_archivable");
}

#[test]
fn archive_test_date_env_var_controls_dir_prefix() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    );
    // Override the date
    let mut c = Command::cargo_bin("speclink").expect("cargo bin");
    c.current_dir(root);
    c.env_remove("SPECLINK_PROVIDER");
    c.env("SPECLINK_TEST_REQUEST_ID", FIXED_REQ);
    c.env("SPECLINK_TEST_ARCHIVE_DATE", "2030-12-31");
    c.env("SPECLINK_CONFIG_HOME", root.join("__no_global__"));
    let out = c.args(["archive", "demo", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(
        env["data"]["archivePath"],
        ".speclink/changes/archive/2030-12-31-demo"
    );
}

#[test]
fn archive_stdout_uses_lf_line_ending() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(
        root,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    );
    let out = cmd(root)
        .args(["archive", "demo", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.ends_with(b"\n"));
    assert!(!out.stdout.windows(2).any(|w| w == b"\r\n"));
}
