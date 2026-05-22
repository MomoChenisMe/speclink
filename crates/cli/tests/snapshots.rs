//! Slice-A 7 ops 各取一個 success envelope + 一個 error envelope 的 insta snapshot。
//!
//! 為了讓 snapshot 穩定，會 redact `requestId`、`createdAt`、`updatedAt`、`changeId`
//! 為固定字串。

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

fn write_stdin(
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

/// 把 envelope 內的 volatile 欄位（requestId, changeId, timestamps）替換為固定 placeholder。
fn redact(mut env: serde_json::Value) -> serde_json::Value {
    if let Some(map) = env.as_object_mut() {
        if map.contains_key("requestId") {
            map["requestId"] =
                serde_json::Value::String("00000000-0000-4000-8000-000000000000".into());
        }
    }
    redact_recursive(&mut env);
    env
}

fn redact_recursive(v: &mut serde_json::Value) {
    if let Some(map) = v.as_object_mut() {
        for key in ["changeId", "createdAt", "updatedAt"] {
            if map.contains_key(key) {
                map[key] = serde_json::Value::String(format!("<{key}>"));
            }
        }
        for (_, val) in map.iter_mut() {
            redact_recursive(val);
        }
    } else if let Some(arr) = v.as_array_mut() {
        for val in arr.iter_mut() {
            redact_recursive(val);
        }
    }
}

#[test]
fn snapshot_new_change_success() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "new", "change", "billing-system"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "new_change_success",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_new_change_duplicate_name_error() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .code(7)
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "new_change_duplicate_error",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_list_changes_after_one_create() {
    let (_tmp, working) = fresh_project_with_change("billing-system");
    let out = speclink(&working)
        .args(["--json", "list", "--changes"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "list_changes_one",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_show_change_empty() {
    let (_tmp, working) = fresh_project_with_change("billing-system");
    let out = speclink(&working)
        .args(["--json", "show", "change", "billing-system"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "show_change_empty",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_show_change_not_found_error() {
    let (_tmp, working) = fresh_project_with_change("dummy");
    let out = speclink(&working)
        .args(["--json", "show", "change", "missing"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "show_change_not_found_error",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_delete_change_success() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = speclink(&working)
        .args(["--json", "delete", "change", "foo", "--confirm-name", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "delete_change_success",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_new_artifact_success_hello_newline() {
    let (_tmp, working) = fresh_project_with_change("foo");
    let out = write_stdin(&working, "proposal", "foo", None, None, b"hello\n");
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "new_artifact_proposal_hello",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_new_artifact_version_conflict_error() {
    let (_tmp, working) = fresh_project_with_change("foo");
    write_stdin(&working, "proposal", "foo", None, None, b"B0");
    let out = write_stdin(&working, "proposal", "foo", None, None, b"B1");
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "new_artifact_version_conflict_error",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_artifact_read_success_hello_newline() {
    let (_tmp, working) = fresh_project_with_change("foo");
    write_stdin(&working, "proposal", "foo", None, None, b"hello\n");
    let out = speclink(&working)
        .args(["--json", "artifact", "read", "proposal", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "artifact_read_proposal_hello",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_list_specs_two_caps() {
    let (_tmp, working) = fresh_project_with_change("foo");
    write_stdin(&working, "spec", "foo", Some("user-auth"), None, b"x");
    write_stdin(&working, "spec", "foo", Some("rate-limiting"), None, b"x");
    let out = speclink(&working)
        .args(["--json", "list", "--specs", "--change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = redact(parse_json(&out.stdout));
    insta::assert_snapshot!(
        "list_specs_two_caps",
        serde_json::to_string_pretty(&env).unwrap()
    );
}
