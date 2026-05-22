//! Integration tests for change CRUD CLI commands.

use std::path::Path;
use std::process::Command;

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

fn fresh_project() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    (tmp, working)
}

// --- new change ---------------------------------------------------------------

#[test]
fn new_change_success_envelope() {
    let (_tmp, working) = fresh_project();
    let out = speclink(&working)
        .args(["--json", "new", "change", "billing-system"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["ok"], true);
    assert_eq!(env["data"]["name"], "billing-system");
    assert_eq!(env["data"]["state"], "proposing");
    assert_eq!(env["data"]["version"], 1);
    assert_eq!(env["data"]["schemaId"], "spec-driven");
    assert_eq!(
        env["data"]["artifactDir"],
        ".speclink/changes/billing-system"
    );
    assert!(env["data"]["changeId"].is_string());
    assert!(env["data"]["createdAt"].is_string());
    assert!(working.join(".speclink/changes/billing-system").is_dir());
}

#[test]
fn new_change_duplicate_name_exit_7() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .code(7)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "change.duplicate_name");
}

#[test]
fn new_change_invalid_name_uppercase_exit_2() {
    let (_tmp, working) = fresh_project();
    let out = speclink(&working)
        .args(["--json", "new", "change", "Foo"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "change.invalid_name");
}

#[test]
fn new_change_invalid_name_underscore_exit_2() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo_bar"])
        .assert()
        .code(2);
}

#[test]
fn new_change_invalid_name_leading_hyphen_exit_2() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "-foo"])
        .assert()
        .code(2);
}

#[test]
fn new_change_invalid_name_65_byte_exit_2() {
    let (_tmp, working) = fresh_project();
    let long = "a".repeat(65);
    speclink(&working)
        .args(["--json", "new", "change", &long])
        .assert()
        .code(2);
}

// --- list changes -------------------------------------------------------------

#[test]
fn list_changes_empty_returns_empty_array() {
    let (_tmp, working) = fresh_project();
    let out = speclink(&working)
        .args(["--json", "list", "--changes"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["data"]["changes"], serde_json::json!([]));
}

#[test]
fn list_changes_sorted_by_updated_at_desc() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "alpha"])
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    speclink(&working)
        .args(["--json", "new", "change", "beta"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "list", "--changes"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    let names: Vec<&str> = env["data"]["changes"]
        .as_array()
        .expect("changes array")
        .iter()
        .map(|c| c["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, vec!["beta", "alpha"]);
}

// --- show change --------------------------------------------------------------

#[test]
fn show_change_with_artifacts() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    // seed artifacts directly on filesystem (artifact write API tested elsewhere)
    let change_dir = working.join(".speclink/changes/foo");
    std::fs::write(change_dir.join("proposal.md"), b"x").unwrap();
    std::fs::write(change_dir.join("design.md"), b"x").unwrap();
    std::fs::create_dir_all(change_dir.join("specs/user-auth")).unwrap();
    std::fs::write(change_dir.join("specs/user-auth/spec.md"), b"x").unwrap();

    let out = speclink(&working)
        .args(["--json", "show", "change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["data"]["change"]["name"], "foo");
    let arts = env["data"]["artifacts"].as_array().expect("artifacts");
    assert_eq!(arts.len(), 3);
    let kinds: Vec<&str> = arts
        .iter()
        .map(|a| a["kind"].as_str().expect("kind"))
        .collect();
    assert!(kinds.contains(&"proposal"));
    assert!(kinds.contains(&"design"));
    assert!(
        arts.iter()
            .any(|a| a["kind"] == "spec" && a["capability"] == "user-auth")
    );
}

#[test]
fn show_change_empty_has_empty_artifacts_array() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "show", "change", "foo"])
        .assert()
        .success()
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["data"]["artifacts"], serde_json::json!([]));
}

#[test]
fn show_change_not_found_exit_2() {
    let (_tmp, working) = fresh_project();
    let out = speclink(&working)
        .args(["--json", "show", "change", "unknown"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "change.not_found");
}

// --- delete change ------------------------------------------------------------

#[test]
fn delete_change_success() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "delete", "change", "foo", "--confirm-name", "foo"])
        .assert()
        .success();
    assert!(!working.join(".speclink/changes/foo").exists());
}

#[test]
fn delete_change_missing_confirm_exit_2() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    let out = speclink(&working)
        .args(["--json", "delete", "change", "foo"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "change.invalid_name");
    assert!(working.join(".speclink/changes/foo").is_dir());
}

#[test]
fn delete_change_mismatched_confirm_exit_2() {
    let (_tmp, working) = fresh_project();
    speclink(&working)
        .args(["--json", "new", "change", "foo"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "delete", "change", "foo", "--confirm-name", "bar"])
        .assert()
        .code(2);
    assert!(working.join(".speclink/changes/foo").is_dir());
}

#[test]
fn delete_change_not_found_exit_2() {
    let (_tmp, working) = fresh_project();
    let out = speclink(&working)
        .args([
            "--json",
            "delete",
            "change",
            "unknown",
            "--confirm-name",
            "unknown",
        ])
        .assert()
        .code(2)
        .get_output()
        .clone();
    let env = parse_json(&out.stdout);
    assert_eq!(env["error"]["code"], "change.not_found");
}
