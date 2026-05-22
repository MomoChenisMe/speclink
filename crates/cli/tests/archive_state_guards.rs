//! Archive state guard 6 條 reject 路徑：
//! (a) proposing  (b) reviewing  (c) ready  (d) code_reviewing  (e) archived
//! (f) in_progress + all_tasks_done=0 → change.tasks_incomplete
//!
//! 對應 archive-runner spec「`speclink archive` SHALL transition the change from
//! `in_progress` to `archived` when all tasks are done」與 state guard matrix scenario。

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use rusqlite::Connection;
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

/// 直接 inject 一個 change row 到 state.db，state/all_tasks_done 由 caller 指定。
fn inject_change(working: &Path, name: &str, state: &str, all_tasks_done: bool) {
    let db_path = working.join(".git").join("speclink").join("state.db");
    let conn = Connection::open(&db_path).expect("open state.db");
    conn.pragma_update(None, "journal_mode", "wal").ok();
    conn.execute("DELETE FROM change WHERE name = ?1", [name])
        .ok();
    conn.execute(
        "INSERT INTO change (change_id, name, state, schema_id, version, all_tasks_done, created_at, updated_at)
         VALUES (?1, ?1, ?2, 'spec-driven', 1, ?3, ?4, ?4)",
        rusqlite::params![
            name,
            state,
            if all_tasks_done { 1 } else { 0 },
            "2026-05-22T10:00:00Z",
        ],
    )
    .expect("insert change row");
    // also create change dir on disk so archive doesn't trip on missing source
    let dir = working.join(".speclink").join("changes").join(name);
    std::fs::create_dir_all(dir.join("specs")).expect("mkdir");
    std::fs::write(dir.join("proposal.md"), b"# P\n").ok();
    std::fs::write(dir.join("tasks.md"), b"- [ ] t\n").ok();
}

fn setup(state: &str, all_tasks_done: bool) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    inject_change(&working, "demo", state, all_tasks_done);
    (tmp, working)
}

fn run_archive(working: &Path) -> std::process::Output {
    speclink(working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive")
}

fn expect_error(out: &std::process::Output, expected_code: &str, expected_exit: i32) {
    assert!(!out.status.success(), "expected failure");
    let code = out.status.code().unwrap_or(0);
    assert_eq!(code, expected_exit, "exit code mismatch");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], expected_code);
}

#[test]
fn archive_rejects_proposing_with_state_transition_invalid_exit_7() {
    let (_tmp, working) = setup("proposing", true);
    let out = run_archive(&working);
    expect_error(&out, "state.transition_invalid", 7);
}

#[test]
fn archive_rejects_reviewing_with_state_transition_invalid_exit_7() {
    let (_tmp, working) = setup("reviewing", true);
    let out = run_archive(&working);
    expect_error(&out, "state.transition_invalid", 7);
}

#[test]
fn archive_rejects_ready_with_state_transition_invalid_exit_7() {
    let (_tmp, working) = setup("ready", true);
    let out = run_archive(&working);
    expect_error(&out, "state.transition_invalid", 7);
}

#[test]
fn archive_rejects_code_reviewing_with_state_transition_invalid_exit_7() {
    let (_tmp, working) = setup("code_reviewing", true);
    let out = run_archive(&working);
    expect_error(&out, "state.transition_invalid", 7);
}

#[test]
fn archive_rejects_archived_repeat_call_with_state_transition_invalid_exit_7() {
    let (_tmp, working) = setup("archived", true);
    let out = run_archive(&working);
    expect_error(&out, "state.transition_invalid", 7);
}

#[test]
fn archive_rejects_in_progress_with_all_tasks_done_false_with_change_tasks_incomplete_exit_2() {
    let (_tmp, working) = setup("in_progress", false);
    let out = run_archive(&working);
    expect_error(&out, "change.tasks_incomplete", 2);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let hint = env["error"]["hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("speclink task done"),
        "hint should mention speclink task done, got: {hint}"
    );
}

#[test]
fn archive_rejects_nonexistent_change_with_change_not_found_exit_2() {
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
    expect_error(&out, "change.not_found", 2);
}
