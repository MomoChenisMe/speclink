//! Human-mode（無 `--json`）輸出 cross-check for slice A3 5 ops。
//!
//! 對應 `cli-human-output` capability 三條 requirement：
//! - Human-mode output 透過 `render_human` pipeline pretty-print（無 JSON-stringified 字串）
//! - `--json` envelope byte-for-byte 不受 renderer 影響
//! - stderr error / hint output 不被 renderer 修改

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

fn setup(working: &Path) {
    git_init(working);
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
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    write_stdin(working, "tasks", "demo", None, b"- [ ] one\n- [ ] two\n");
    speclink(working)
        .args(["--json", "apply", "start", "demo", "--actor", "cli"])
        .assert()
        .success();
}

fn looks_like_json_envelope(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.starts_with('{') && trimmed.contains("\"ok\":") && trimmed.contains("\"requestId\":")
}

#[test]
fn apply_start_human_mode_does_not_print_json_envelope_string() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    speclink(&working)
        .args(["--json", "apply", "pause", "demo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["apply", "start", "demo", "--actor", "cli"])
        .output()
        .expect("start");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !looks_like_json_envelope(&stdout),
        "human mode SHALL NOT emit raw JSON envelope; got:\n{stdout}"
    );
    // human-mode SHALL still surface the state / actor identity (pretty-print).
    assert!(
        stdout.contains("in_progress"),
        "human output mentions state"
    );
}

#[test]
fn task_list_human_mode_does_not_print_json_envelope_string() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["task", "list", "--change", "demo"])
        .output()
        .expect("list");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!looks_like_json_envelope(&stdout));
    assert!(stdout.contains("one") && stdout.contains("two"));
}

#[test]
fn task_done_human_mode_pretty_prints_payload() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["task", "done", "1", "--change", "demo"])
        .output()
        .expect("done");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!looks_like_json_envelope(&stdout));
}

#[test]
fn error_path_writes_to_stderr_with_hint_in_human_mode() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["apply", "start", "demo"])
        .output()
        .expect("start");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("error[state.transition_invalid]"),
        "stderr SHALL carry error code header; got:\n{stderr}"
    );
    assert!(stderr.contains("hint:"), "stderr SHALL include hint line");
    // stdout SHALL NOT contain JSON envelope in human mode.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.trim().is_empty() || !looks_like_json_envelope(&stdout));
}

#[test]
fn json_mode_envelope_unchanged_by_renderer() {
    // 確認帶 `--json` 時 stdout 仍是 envelope，renderer 沒介入。
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    setup(&working);
    let out = speclink(&working)
        .args(["--json", "task", "list", "--change", "demo"])
        .output()
        .expect("list json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        looks_like_json_envelope(&stdout),
        "--json mode SHALL emit envelope verbatim; got:\n{stdout}"
    );
}
