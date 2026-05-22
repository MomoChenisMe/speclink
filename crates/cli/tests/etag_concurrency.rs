//! Integration tests for sha256 etag concurrency matrix (CLI level).
//!
//! Covers the 5 rows of `specs/artifact-io/spec.md` example "concurrency matrix",
//! plus a cross-platform fixture asserting sha256(b"hello\n") matches a known digest.

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

fn fresh() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    (tmp, working)
}

fn write_stdin(working: &Path, expected_etag: Option<&str>, body: &[u8]) -> std::process::Output {
    let bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(bin);
    cmd.current_dir(working);
    cmd.arg("--json")
        .arg("new")
        .arg("artifact")
        .arg("proposal")
        .arg("--change")
        .arg("foo")
        .arg("--stdin");
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

#[test]
fn write_new_without_etag_succeeds() {
    let (_tmp, working) = fresh();
    let out = write_stdin(&working, None, b"B0");
    assert!(out.status.success());
    let env = parse_json(&out.stdout);
    let etag = env["data"]["etag"].as_str().expect("etag");
    assert!(etag.starts_with("sha256:"));
    assert_eq!(env["data"]["bytesWritten"], 2);
}

#[test]
fn write_new_with_etag_rejected_not_found() {
    let (_tmp, working) = fresh();
    let phantom = "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
    let out = write_stdin(&working, Some(phantom), b"B0");
    assert_eq!(out.status.code(), Some(2));
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.not_found");
    // file MUST NOT be created
    assert!(!working.join(".speclink/changes/foo/proposal.md").exists());
}

#[test]
fn write_existing_without_etag_conflict() {
    let (_tmp, working) = fresh();
    write_stdin(&working, None, b"B0");
    let out = write_stdin(&working, None, b"B1");
    assert_eq!(out.status.code(), Some(7));
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.version_conflict");
    // file must still be B0
    let body = std::fs::read(working.join(".speclink/changes/foo/proposal.md")).unwrap();
    assert_eq!(body, b"B0");
}

#[test]
fn write_existing_matching_etag_overwrites() {
    let (_tmp, working) = fresh();
    let r0 = write_stdin(&working, None, b"B0");
    let env0 = parse_json(&r0.stdout);
    let etag0 = env0["data"]["etag"].as_str().expect("etag").to_string();

    let r1 = write_stdin(&working, Some(&etag0), b"B1");
    assert!(r1.status.success());
    let env1 = parse_json(&r1.stdout);
    let etag1 = env1["data"]["etag"].as_str().expect("etag");
    assert_ne!(etag0, etag1);
    assert!(etag1.starts_with("sha256:"));

    // verify file is now B1
    let body = std::fs::read(working.join(".speclink/changes/foo/proposal.md")).unwrap();
    assert_eq!(body, b"B1");
}

#[test]
fn write_existing_mismatching_etag_conflict() {
    let (_tmp, working) = fresh();
    write_stdin(&working, None, b"B0");
    let wrong = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    let out = write_stdin(&working, Some(wrong), b"B1");
    assert_eq!(out.status.code(), Some(7));
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "artifact.version_conflict");
    // file must still be B0
    let body = std::fs::read(working.join(".speclink/changes/foo/proposal.md")).unwrap();
    assert_eq!(body, b"B0");
}

#[test]
fn hello_newline_etag_cross_platform_fixed_digest() {
    // sha256(b"hello\n") = 5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03
    let (_tmp, working) = fresh();
    let out = write_stdin(&working, None, b"hello\n");
    assert!(out.status.success());
    let env = parse_json(&out.stdout);
    assert_eq!(
        env["data"]["etag"],
        "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
    );
}
