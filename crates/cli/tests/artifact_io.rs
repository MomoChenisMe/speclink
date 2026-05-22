//! Integration tests for artifact I/O CLI commands.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "git command failed: {cmd:?}");
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn speclink(working: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("speclink").expect("binary");
    cmd.current_dir(working);
    cmd
}

fn parse_json(output: &[u8]) -> serde_json::Value {
    serde_json::from_slice(output).unwrap_or_else(|e| {
        panic!(
            "stdout was not JSON: {e}\n{}",
            String::from_utf8_lossy(output)
        )
    })
}

fn fresh_project_with_change(name: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", name])
        .assert()
        .success();
    (tmp, working)
}

/// 用 stdin 餵入 bytes，跑 `speclink --json new artifact ... --stdin`。
fn write_artifact_stdin(
    working: &Path,
    kind: &str,
    change: &str,
    capability: Option<&str>,
    expected_etag: Option<&str>,
    body: &[u8],
) -> std::process::Output {
    let bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(bin);
    cmd.current_dir(working);
    cmd.arg("--json")
        .arg("new")
        .arg("artifact")
        .arg(kind)
        .arg("--change")
        .arg(change)
        .arg("--stdin");
    if let Some(c) = capability {
        cmd.arg("--capability").arg(c);
    }
    if let Some(e) = expected_etag {
        cmd.arg("--expected-etag").arg(e);
    }
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body)
        .expect("write stdin");
    child.wait_with_output().expect("wait")
}

// --- §7.6 kind / capability validation ----------------------------------------

#[test]
fn new_artifact_invalid_kind_exit_2() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = write_artifact_stdin(&working, "summary", "foo", None, None, b"x");
    assert_eq!(out.status.code(), Some(2));
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.kind_invalid");
}

#[test]
fn new_artifact_spec_without_capability_exit_2() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = write_artifact_stdin(&working, "spec", "foo", None, None, b"x");
    assert_eq!(out.status.code(), Some(2));
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.capability_required");
}

#[test]
fn new_artifact_proposal_with_capability_emits_warning() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = write_artifact_stdin(&working, "proposal", "foo", Some("ignored"), None, b"x");
    assert!(out.status.success());
    let env = parse_json(&out.stdout);
    let warnings = env["warnings"].as_array().expect("warnings");
    assert!(
        warnings
            .iter()
            .any(|w| w["code"] == "artifact.capability_ignored"),
        "expected artifact.capability_ignored warning, got: {warnings:?}"
    );
    assert_eq!(env["data"]["path"], "changes/foo/proposal.md");
}

// --- §7.7 artifact read -------------------------------------------------------

#[test]
fn artifact_read_success_returns_content_and_etag() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let body = b"## Why\n\nfoo\n";
    write_artifact_stdin(&working, "proposal", "foo", None, None, body);
    let out = speclink(&working)
        .args(["--json", "artifact", "read", "proposal", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(
        env["data"]["content"],
        String::from_utf8(body.to_vec()).unwrap()
    );
    let etag = env["data"]["etag"].as_str().expect("etag");
    assert!(etag.starts_with("sha256:"));
    assert_eq!(etag.len(), "sha256:".len() + 64);
}

#[test]
fn artifact_read_missing_file_exit_2() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = speclink(&working)
        .args(["--json", "artifact", "read", "proposal", "--change", "foo"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.not_found");
}

#[test]
fn artifact_read_unknown_change_exit_2() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = speclink(&working)
        .args([
            "--json", "artifact", "read", "proposal", "--change", "missing",
        ])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "change.not_found");
}

// --- §7.9 list --specs --------------------------------------------------------

#[test]
fn list_specs_empty() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = speclink(&working)
        .args(["--json", "list", "--specs", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["data"]["capabilities"], serde_json::json!([]));
}

#[test]
fn list_specs_sorted_after_writes() {
    let (_tmp, working) = fresh_project_with_change("foo");
    write_artifact_stdin(&working, "spec", "foo", Some("user-auth"), None, b"x");
    write_artifact_stdin(&working, "spec", "foo", Some("rate-limiting"), None, b"x");
    let out = speclink(&working)
        .args(["--json", "list", "--specs", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(
        env["data"]["capabilities"],
        serde_json::json!(["rate-limiting", "user-auth"])
    );
}

#[test]
fn list_specs_ignores_subdir_without_spec_md() {
    let (_tmp, working) = fresh_project_with_change("foo");
    write_artifact_stdin(&working, "spec", "foo", Some("user-auth"), None, b"x");
    std::fs::create_dir_all(working.join(".speclink/changes/foo/specs/incomplete")).unwrap();
    let out = speclink(&working)
        .args(["--json", "list", "--specs", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(
        env["data"]["capabilities"],
        serde_json::json!(["user-auth"])
    );
}
