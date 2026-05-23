//! Integration tests for `speclink status` CLI.
//!
//! 對齊 specs/project-status 的 Requirements 與 Implementation Contract「Command output」第一範例。

use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use serde_json::Value;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init_with_commit(dir: &Path) {
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
    fs::write(dir.join("README.md"), b"seed\n").unwrap();
    run(Command::new("git").args(["add", "."]).current_dir(dir));
    run(Command::new("git")
        .args(["commit", "-m", "seed"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn speclink(dir: &Path) -> AssertCommand {
    let mut c = AssertCommand::cargo_bin("speclink").expect("speclink bin");
    c.current_dir(dir);
    c
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("valid JSON envelope")
}

// ===== 4.1 status in empty dir → exit 2 + project.not_initialized =====

#[test]
fn status_in_empty_dir_exits_2_with_project_not_initialized() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    // 不跑 speclink init — .speclink/ 不存在
    let out = speclink(&w)
        .args(["status", "--json"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let env = parse_json(&out.stdout);
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "project.not_initialized");
}

// ===== 4.2 status in SpecLink project → envelope.data 含七 required field =====

#[test]
fn status_in_speclink_project_emits_envelope_with_seven_fields() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();

    let out = speclink(&w)
        .args(["status", "--json"])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit not 0: {:?}", out.status);
    let env = parse_json(&out.stdout);
    assert_eq!(env["ok"], true);

    let data = &env["data"];
    // 七個 required field（spec Requirement: SHALL include exactly these fields）
    for key in [
        "provider_type",
        "project_id",
        "working_dir",
        "current_change",
        "changes_count",
        "discussions_count",
        "schema_active",
    ] {
        assert!(
            data.get(key).is_some(),
            "envelope.data missing required key `{key}`; data={data}"
        );
    }
    assert_eq!(data["provider_type"], "local");
    assert!(data["project_id"].is_string());
    assert_eq!(data["current_change"], Value::Null);
    // changes_count 六 bucket 全 0（fresh project）
    let cc = &data["changes_count"];
    for s in [
        "proposing",
        "reviewing",
        "ready",
        "in_progress",
        "code_reviewing",
        "archived",
    ] {
        assert_eq!(cc[s], 0, "changes_count.{s} != 0");
    }
    // discussions_count {active:0, converged:0}
    assert_eq!(data["discussions_count"]["active"], 0);
    assert_eq!(data["discussions_count"]["converged"], 0);
    // schema_active hardcode (P1 walking-skeleton) — 對齊 DEFAULT_SCHEMA_ID
    assert_eq!(data["schema_active"], "spec-driven");
}

// ===== 4.3 read-only：兩次 status 不動 state.db =====

#[test]
fn status_read_only_does_not_mutate_state() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();

    let state_db = w.join(".git/speclink/state.db");
    assert!(state_db.exists(), "state.db must exist after init");
    let mtime_before = fs::metadata(&state_db).unwrap().modified().unwrap();

    // 跑兩次
    speclink(&w).args(["status", "--json"]).assert().success();
    std::thread::sleep(std::time::Duration::from_millis(50));
    speclink(&w).args(["status", "--json"]).assert().success();

    let mtime_after = fs::metadata(&state_db).unwrap().modified().unwrap();
    assert_eq!(
        mtime_before, mtime_after,
        "state.db mtime changed — status is not read-only"
    );
}

// ===== 6.1 insta snapshot：status envelope =====

fn redact(mut v: Value) -> Value {
    fn walk(v: &mut Value) {
        match v {
            Value::Object(map) => {
                for (k, val) in map.iter_mut() {
                    if matches!(
                        k.as_str(),
                        "requestId" | "project_id" | "working_dir" | "created_at" | "updated_at"
                    ) {
                        *val = Value::String(format!("<{k}>"));
                    } else {
                        walk(val);
                    }
                }
            }
            Value::Array(arr) => arr.iter_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut v);
    v
}

#[test]
fn snapshot_status_envelope() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();
    let out = speclink(&w).args(["status", "--json"]).output().unwrap();
    let env = redact(parse_json(&out.stdout));
    insta::assert_json_snapshot!("status_envelope", env);
}

// ===== 4.4 human mode → YAML，無 ANSI / 無 box-drawing =====

#[test]
fn status_human_mode_emits_yaml_not_ansi() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();

    // 無 --json flag → human mode
    let out = speclink(&w).arg("status").output().expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");

    // 無 ANSI escape
    assert!(
        !stdout.contains('\x1b'),
        "stdout contains ANSI escape sequence; got:\n{stdout}"
    );
    // 無 box-drawing chars
    for bad in ['┌', '┐', '└', '┘', '─', '│', '├', '┤', '┬', '┴', '┼'] {
        assert!(
            !stdout.contains(bad),
            "stdout contains box-drawing char `{bad}`; got:\n{stdout}"
        );
    }
    // YAML 樣式 — provider_type 與 schema_active 必須以 `key:` 形式出現
    assert!(
        stdout.contains("provider_type:") || stdout.contains("providerType:"),
        "stdout missing provider_type YAML key; got:\n{stdout}"
    );
}
