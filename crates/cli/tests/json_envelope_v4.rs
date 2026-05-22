//! JSON envelope snapshot tests for archive op (slice A4)。
//!
//! 對應 archive-runner spec「JSON envelope SHALL conform to the bootstrap / A2 / A3
//! contract」與 design「JSON envelope shape」決策。
//!
//! 鎖死 `archive` 的 success / `--skip-specs` success / 3 個典型 error envelope shape：
//! `state.transition_invalid`、`change.tasks_incomplete`、`change.not_found`。
//! `requestId` 與 `archivedAt` 不參與 snapshot（替換成 stable 占位字串）。

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::TempDir;

fn git_init(dir: &Path) {
    Command::new("git")
        .args(["init", "--quiet", "--initial-branch=main"])
        .current_dir(dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "t@e.com"])
        .current_dir(dir)
        .output()
        .ok();
    Command::new("git")
        .args(["config", "user.name", "t"])
        .current_dir(dir)
        .output()
        .ok();
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn speclink(working: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("speclink").expect("binary");
    cmd.current_dir(working);
    cmd
}

fn write_stdin(
    working: &Path,
    kind: &str,
    change: &str,
    cap: Option<&str>,
    body: &[u8],
) -> std::process::Output {
    let bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(bin);
    cmd.current_dir(working)
        .arg("--json")
        .arg("new")
        .arg("artifact")
        .arg(kind)
        .arg("--change")
        .arg(change)
        .arg("--stdin");
    if let Some(c) = cap {
        cmd.arg("--capability").arg(c);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body)
        .expect("write");
    child.wait_with_output().expect("wait")
}

fn drive_to_in_progress_with_all_tasks_done(working: &Path) {
    speclink(working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();
    let _ = write_stdin(working, "proposal", "demo", None, b"## Why\n");
    let _ = write_stdin(
        working,
        "spec",
        "demo",
        Some("ua"),
        b"## ADDED Requirements\n\n### Requirement: x\n\n#### Scenario: y\n\n- **WHEN** z\n- **THEN** w\n",
    );
    let _ = write_stdin(working, "tasks", "demo", None, b"- [ ] step1\n");
    speclink(working)
        .args(["--json", "apply", "start", "demo", "--actor", "c"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();
}

fn redact(env: &mut Value) {
    if let Some(rid) = env.get_mut("requestId") {
        *rid = Value::String("00000000-0000-0000-0000-000000000000".into());
    }
    if let Some(data) = env.get_mut("data") {
        if let Some(at) = data.get_mut("archivedAt") {
            *at = Value::String("2026-05-22T00:00:00Z".into());
        }
        if let Some(ad) = data.get_mut("archiveDir") {
            *ad = Value::String(".speclink/changes/archive/2026-05-22-demo".into());
        }
    }
    if let Some(arr) = env.get_mut("warnings").and_then(|v| v.as_array_mut()) {
        for w in arr.iter_mut() {
            // no time-sensitive payload in archive warnings; nothing to redact
            let _ = w;
        }
    }
}

#[test]
fn archive_success_envelope_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_in_progress_with_all_tasks_done(&working);
    let out = speclink(&working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive");
    assert!(out.status.success());
    let mut env: Value = serde_json::from_slice(&out.stdout).expect("json");
    redact(&mut env);
    insta::assert_json_snapshot!("archive_success_envelope", env);
}

#[test]
fn archive_skip_specs_success_envelope_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_in_progress_with_all_tasks_done(&working);
    let out = speclink(&working)
        .args(["--json", "archive", "demo", "--skip-specs"])
        .output()
        .expect("archive --skip-specs");
    assert!(out.status.success());
    let mut env: Value = serde_json::from_slice(&out.stdout).expect("json");
    redact(&mut env);
    insta::assert_json_snapshot!("archive_skip_specs_envelope", env);
}

fn inject_change(working: &Path, state: &str, all_tasks_done: bool) {
    let db_path = working.join(".git").join("speclink").join("state.db");
    let conn = Connection::open(&db_path).expect("open state.db");
    conn.execute(
        "INSERT OR REPLACE INTO change (change_id, name, state, schema_id, version, all_tasks_done, created_at, updated_at)
         VALUES ('demo', 'demo', ?1, 'spec-driven', 1, ?2, ?3, ?3)",
        rusqlite::params![state, if all_tasks_done { 1 } else { 0 }, "2026-05-22T10:00:00Z"],
    )
    .expect("insert");
    let dir = working.join(".speclink").join("changes").join("demo");
    std::fs::create_dir_all(dir.join("specs")).ok();
    std::fs::write(dir.join("proposal.md"), b"# P\n").ok();
    std::fs::write(dir.join("tasks.md"), b"- [ ] t\n").ok();
}

#[test]
fn archive_state_transition_invalid_error_envelope_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    inject_change(&working, "proposing", true);
    let out = speclink(&working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive");
    assert_eq!(out.status.code().unwrap_or(0), 7);
    let mut env: Value = serde_json::from_slice(&out.stdout).expect("json");
    redact(&mut env);
    insta::assert_json_snapshot!("archive_state_transition_invalid_envelope", env);
}

#[test]
fn archive_change_tasks_incomplete_error_envelope_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    inject_change(&working, "in_progress", false);
    let out = speclink(&working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive");
    assert_eq!(out.status.code().unwrap_or(0), 2);
    let mut env: Value = serde_json::from_slice(&out.stdout).expect("json");
    redact(&mut env);
    insta::assert_json_snapshot!("archive_change_tasks_incomplete_envelope", env);
}

#[test]
fn archive_change_not_found_error_envelope_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "archive", "ghost"])
        .output()
        .expect("archive");
    assert_eq!(out.status.code().unwrap_or(0), 2);
    let mut env: Value = serde_json::from_slice(&out.stdout).expect("json");
    redact(&mut env);
    insta::assert_json_snapshot!("archive_change_not_found_envelope", env);
}
