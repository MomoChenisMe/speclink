//! Human-mode（無 `--json`）輸出 cross-check for slice A4 archive op。
//!
//! 對應 `cli-human-output` capability 三條 requirement（cross-check）：
//! - Human-mode output 透過 `render_human` pipeline pretty-print
//! - `--json` envelope byte-for-byte 不受 renderer 影響
//! - stderr error / hint output 不被 renderer 修改

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
use rusqlite::Connection;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "git command failed: {cmd:?}");
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--quiet")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "t@e.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "t"])
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

fn write_stdin(working: &Path, kind: &str, change: &str, cap: Option<&str>, body: &[u8]) {
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
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success());
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
    write_stdin(working, "proposal", "demo", None, b"## Why\n");
    write_stdin(
        working,
        "spec",
        "demo",
        Some("ua"),
        b"## ADDED Requirements\n\n### Requirement: x\n\n#### Scenario: y\n\n- **WHEN** z\n- **THEN** w\n",
    );
    write_stdin(working, "tasks", "demo", None, b"- [ ] step1\n");
    speclink(working)
        .args(["--json", "apply", "start", "demo", "--actor", "c"])
        .assert()
        .success();
    speclink(working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();
}

#[test]
fn archive_human_output_is_pretty_printed_not_json_stringified() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    drive_to_in_progress_with_all_tasks_done(&working);
    let out = speclink(&working)
        .args(["archive", "demo"])
        .output()
        .expect("archive (human)");
    assert!(out.status.success(), "exit 0 expected: {out:?}");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    // Must NOT contain JSON-stringified key/value pairs (no `"key"`, no `{"`).
    assert!(
        !stdout.contains("\"changeId\""),
        "human mode SHALL NOT emit JSON-stringified keys: {stdout}"
    );
    // Must contain pretty-printed structure
    assert!(stdout.contains("changeId"), "key present: {stdout}");
    assert!(stdout.contains("archived"), "value present: {stdout}");
    assert!(
        stdout.contains("mergedSpecs"),
        "nested key present: {stdout}"
    );
    assert!(stdout.contains("ua"), "capability name present: {stdout}");
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
fn archive_error_writes_to_stderr_with_hint() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    inject_change(&working, "in_progress", false); // all_tasks_done=0
    let out = speclink(&working)
        .args(["archive", "demo"])
        .output()
        .expect("archive");
    assert_eq!(out.status.code().unwrap_or(0), 2);
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(
        stderr.contains("error[change.tasks_incomplete]"),
        "stderr SHALL contain error code: {stderr}"
    );
    assert!(
        stderr.contains("hint:") && stderr.contains("speclink task done"),
        "stderr SHALL contain hint with `speclink task done`: {stderr}"
    );
    // stdout SHALL be empty (no JSON envelope leaks into human mode error path)
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(
        stdout.is_empty(),
        "human-mode error SHALL NOT write stdout: {stdout:?}"
    );
}
