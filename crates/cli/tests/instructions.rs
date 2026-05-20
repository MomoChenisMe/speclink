//! `speclink instructions` end-to-end CLI 整合測試。

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
        .expect("propose");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn instructions_design_happy_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "design", "--change", "demo", "--json"])
        .output()
        .expect("instructions");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["ok"], true);
    assert_eq!(env["data"]["artifactId"], "design");
    assert_eq!(env["data"]["kind"], "design");
    assert_eq!(
        env["data"]["outputPath"],
        ".speclink/changes/demo/design.md"
    );
    assert!(!env["data"]["instruction"].as_str().unwrap().is_empty());
    assert!(!env["data"]["template"].as_str().unwrap().is_empty());
    assert!(!env["data"]["rules"].as_array().unwrap().is_empty());
    assert_eq!(env["data"]["locale"], "Traditional Chinese (繁體中文)");
}

#[test]
fn instructions_tasks_dependencies() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "tasks", "--change", "demo", "--json"])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["data"]["kind"], "tasks");
    let deps: Vec<&str> = env["data"]["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(deps, vec!["proposal", "spec"]);
    assert!(env["data"]["unlocks"].as_array().unwrap().is_empty());
}

#[test]
fn instructions_spec_with_capability_happy_path() {
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
        .expect("instructions");
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["data"]["artifactId"], "spec:user-auth");
    assert_eq!(
        env["data"]["outputPath"],
        ".speclink/changes/demo/specs/user-auth/spec.md"
    );
}

#[test]
fn instructions_proposal_no_dependencies() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "proposal", "--change", "demo", "--json"])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(0));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert!(env["data"]["dependencies"].as_array().unwrap().is_empty());
}

#[test]
fn instructions_change_not_found() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let out = cmd(root)
        .args(["instructions", "design", "--change", "missing", "--json"])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(1));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "change.not_found");
}

#[test]
fn instructions_spec_missing_capability() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    // 缺 --capability：clap 直接擋 → exit 2
    let out = cmd(root)
        .args(["instructions", "spec", "--change", "demo"])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn instructions_design_with_capability_rejected_by_clap() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args([
            "instructions",
            "design",
            "--change",
            "demo",
            "--capability",
            "x",
        ])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn instructions_spec_invalid_capability_rejected_by_clap() {
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
            "Bad-Name",
            "--json",
        ])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn instructions_stdin_flag_rejected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args([
            "instructions",
            "design",
            "--change",
            "demo",
            "--stdin",
            "--json",
        ])
        .output()
        .expect("instructions");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn instructions_output_uses_lf_line_ending() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap(root, "demo");
    let out = cmd(root)
        .args(["instructions", "design", "--change", "demo", "--json"])
        .output()
        .expect("instructions");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.ends_with('\n'));
    assert!(!stdout.contains('\r'), "stdout must not contain CR");
}
