//! `speclink artifact write` end-to-end 命令介面測試。

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

/// 先在 `cwd` 用 `propose create` 建好 `demo` change，方便後續 artifact write 測試。
fn bootstrap_demo(cwd: &std::path::Path) {
    let out = cmd(cwd)
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "bootstrap failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn write_design_happy_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);

    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("design body")
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["ok"], Value::Bool(true));
    assert_eq!(env["data"]["changeId"], "demo");
    assert_eq!(env["data"]["artifactId"], "design");
    assert_eq!(env["data"]["kind"], "design");
    assert_eq!(env["data"]["path"], ".speclink/changes/demo/design.md");
    assert_eq!(env["data"]["mode"], "local");
    // trailing newline 補齊
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/design.md")).unwrap();
    assert_eq!(body, "design body\n");
}

#[test]
fn write_spec_happy_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);

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
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["data"]["artifactId"], "spec:user-auth");
    assert_eq!(env["data"]["kind"], "spec");
    assert_eq!(
        env["data"]["path"],
        ".speclink/changes/demo/specs/user-auth/spec.md"
    );
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/specs/user-auth/spec.md"))
        .unwrap();
    assert_eq!(body, "spec body\n");
}

#[test]
fn trailing_newline_appended_when_missing() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("no newline")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/design.md")).unwrap();
    assert_eq!(body, "no newline\n");
}

#[test]
fn existing_trailing_newline_preserved() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("with newline\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/design.md")).unwrap();
    assert_eq!(body, "with newline\n");
}

#[test]
fn empty_stdin_rejected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn invalid_utf8_stdin_rejected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin(vec![0xffu8, 0xfe, 0xfd])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn change_not_found_exits_1() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "artifact", "write", "design", "--change", "missing", "--stdin", "--json",
        ])
        .write_stdin("x\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "change.not_found");
}

#[test]
fn artifact_already_exists_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let _ = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("OLD\n")
        .output()
        .expect("first");

    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("NEW\n")
        .output()
        .expect("second");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "artifact.already_exists");
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/design.md")).unwrap();
    assert_eq!(body, "OLD\n", "existing must not be overwritten");
}

#[test]
fn design_with_capability_is_input_invalid() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact",
            "write",
            "design",
            "--change",
            "demo",
            "--capability",
            "foo",
            "--stdin",
            "--json",
        ])
        .write_stdin("x\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(env["error"]["code"], "input.invalid");
}

#[test]
fn spec_missing_capability_is_clap_error_exit_2() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args(["artifact", "write", "spec", "--change", "demo", "--stdin"])
        .write_stdin("x\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn spec_invalid_capability_is_clap_error_exit_2() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "Bad-Name",
            "--stdin",
        ])
        .write_stdin("x\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn stdout_uses_lf_line_ending() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    bootstrap_demo(root);
    let out = cmd(root)
        .args([
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .write_stdin("x\n")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.ends_with(b"\n"));
    assert!(!out.stdout.windows(2).any(|w| w == b"\r\n"));
}
