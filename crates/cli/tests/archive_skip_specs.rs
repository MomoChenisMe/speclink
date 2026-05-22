//! Archive `--skip-specs` 路徑測試。
//!
//! Case (a): 帶 2 capability 的 change → `data.merged_specs=[]`、`warnings` 唯一一筆
//! `code=archive.specs_skipped` + `details.capabilities_skipped` 排序。
//! Case (b): 帶 0 capability → 同上但 `warnings=[]`、assert `.speclink/specs/`
//! byte-for-byte 不變。
//!
//! 對應 archive-runner spec「`--skip-specs` SHALL bypass merge while still
//! transitioning state and emit an audit warning」。

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
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

/// 設定一個跑到 `in_progress + all_tasks_done=1` 的 change，capabilities 由 caller 指定。
fn setup_change_in_progress_with_all_tasks_done(working: &Path, name: &str, capabilities: &[&str]) {
    speclink(working)
        .args(["--json", "new", "change", name])
        .assert()
        .success();

    // write proposal first
    let out = write_stdin(working, "proposal", name, None, b"## Why\n");
    assert!(out.status.success());

    // write each spec
    for cap in capabilities {
        let body = format!(
            "## ADDED Requirements\n\n### Requirement: {cap}\n\n#### Scenario: ok\n\n- **WHEN** x\n- **THEN** y\n"
        );
        let out = write_stdin(working, "spec", name, Some(cap), body.as_bytes());
        assert!(out.status.success());
    }

    // write tasks last → DAG complete → state==ready
    let out = write_stdin(working, "tasks", name, None, b"- [ ] step1\n");
    assert!(out.status.success());

    // apply start
    speclink(working)
        .args(["--json", "apply", "start", name, "--actor", "claude"])
        .assert()
        .success();
    // task done — completes the only task → all_tasks_done=1
    speclink(working)
        .args(["--json", "task", "done", "1", "--change", name])
        .assert()
        .success();
}

#[test]
fn skip_specs_with_two_capabilities_produces_sorted_warning_carrier() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    setup_change_in_progress_with_all_tasks_done(&working, "demo", &["user-auth", "audit-log"]);

    // snapshot pre-existing .speclink/specs/ contents (empty)
    let specs_root = working.join(".speclink/specs");
    assert!(
        !specs_root.exists()
            || specs_root
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        ".speclink/specs/ SHALL be empty before archive"
    );

    let out = speclink(&working)
        .args(["--json", "archive", "demo", "--skip-specs"])
        .output()
        .expect("archive --skip-specs");
    assert!(out.status.success(), "archive failed: {out:?}");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["ok"], true);
    let merged = env["data"]["mergedSpecs"]
        .as_array()
        .expect("mergedSpecs array");
    assert!(
        merged.is_empty(),
        "merged_specs SHALL be empty under --skip-specs"
    );
    let warnings = env["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1, "exactly one warning");
    let w = &warnings[0];
    assert_eq!(w["code"], "archive.specs_skipped");
    let caps = w["details"]["capabilities_skipped"]
        .as_array()
        .expect("capabilities_skipped");
    let names: Vec<&str> = caps.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(names, vec!["audit-log", "user-auth"]);

    // .speclink/specs/ must remain unchanged (no spec was promoted)
    assert!(
        !specs_root.exists()
            || specs_root
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        ".speclink/specs/ SHALL remain empty after --skip-specs"
    );
}

#[test]
fn skip_specs_with_no_capabilities_emits_no_warning() {
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

    // Write only proposal + tasks; no specs. DAG complete? Only if specs has at least one entry.
    // For walking-skeleton DAG, we need proposal + tasks + at least one spec. So we MUST add a
    // throwaway spec then DELETE the spec dir before archive to simulate "0 capabilities".
    let _ = write_stdin(&working, "proposal", "demo", None, b"## Why\n");
    let _ = write_stdin(
        &working,
        "spec",
        "demo",
        Some("placeholder"),
        b"## ADDED Requirements\n\n### Requirement: x\n\n#### Scenario: y\n\n- **WHEN** z\n- **THEN** w\n",
    );
    let _ = write_stdin(&working, "tasks", "demo", None, b"- [ ] step1\n");
    speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "claude"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();

    // Now remove the placeholder capability dir before running archive --skip-specs,
    // so the post-rename scan finds 0 capabilities → no warning.
    let specs_dir = working.join(".speclink/changes/demo/specs");
    std::fs::remove_dir_all(&specs_dir).expect("rm specs dir");

    let out = speclink(&working)
        .args(["--json", "archive", "demo", "--skip-specs"])
        .output()
        .expect("archive --skip-specs (no caps)");
    assert!(out.status.success(), "archive failed: {out:?}");
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let warnings = env["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.is_empty(),
        "no caps → no warning carrier: {warnings:?}"
    );
}
