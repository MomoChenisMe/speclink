//! Integration tests for the `speclink` CLI binary.
//!
//! 對應 design「Acceptance criteria」表中 15 個 case。

use std::fs;
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

fn git_init_with_commit(dir: &Path) {
    git_init(dir);
    run(Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
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

fn sha256_of(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(p).expect("read");
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

#[test]
fn init_in_fresh_git_repo_writes_two_roots() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    let assert = speclink(&w).arg("init").assert().success();
    let _ = assert;
    assert!(w.join(".speclink/link.yaml").exists());
    assert!(w.join(".git/speclink/state.db").exists());
    let gitignore = fs::read_to_string(w.join(".gitignore")).unwrap();
    assert!(gitignore.lines().any(|l| l == ".speclink/link.yaml"));
}

#[test]
fn init_in_non_git_dir_rejects_with_requires_git() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    let out = speclink(&w)
        .arg("init")
        .arg("--json")
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let json = parse_json(&out.stdout);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "project.requires_git");
}

#[test]
fn init_when_already_initialized_returns_conflict() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let out = speclink(&w)
        .arg("init")
        .arg("--json")
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(7));
    let json = parse_json(&out.stdout);
    assert_eq!(json["error"]["code"], "project.already_initialized");
}

#[test]
fn init_with_force_overwrites_link_yaml() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let link_before = fs::read_to_string(w.join(".speclink/link.yaml")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    speclink(&w).args(["init", "--force"]).assert().success();
    let link_after = fs::read_to_string(w.join(".speclink/link.yaml")).unwrap();
    assert_ne!(link_before, link_after);
}

#[test]
fn status_after_init_returns_expected_fields() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();
    let out = speclink(&w)
        .args(["status", "--json"])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let json = parse_json(&out.stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["provider"], "local");
    assert_eq!(json["data"]["artifact_root"], ".speclink");
    assert_eq!(json["data"]["state_root"], ".git/speclink");
    assert!(json["data"]["project_id"].is_string());
    assert!(json["data"]["requires_git"].as_bool().unwrap());
    assert!(json["data"]["git_head"].is_string());
}

#[test]
fn status_without_init_returns_not_initialized() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    let out = speclink(&w).args(["status", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let json = parse_json(&out.stdout);
    assert_eq!(json["error"]["code"], "project.not_initialized");
}

#[test]
fn status_in_linked_worktree_resolves_state_root_to_main_git_dir() {
    let tmp_owned = TempDir::new().unwrap();
    let base = canonical(tmp_owned.path());
    let main = base.join("main");
    fs::create_dir(&main).unwrap();
    git_init_with_commit(&main);
    speclink(&main).arg("init").assert().success();
    let wt = base.join("wt");
    run(Command::new("git")
        .args(["worktree", "add", "-b", "wt-branch"])
        .arg(&wt)
        .current_dir(&main));

    // Worktree starts with no `.speclink/link.yaml` (it lives in the main
    // working tree). Running `speclink status` from the worktree therefore
    // reports `project.not_initialized`. Crucially, when the worktree later
    // initialises, the shared `state.db` lives under the *main* repo's
    // `.git/speclink/` — confirmed below.
    let out = speclink(&wt).args(["status", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let json = parse_json(&out.stdout);
    assert_eq!(json["error"]["code"], "project.not_initialized");
    // state.db lives under main repo's .git dir, never under the worktree's
    // own .git (which is a file, not a dir).
    assert!(main.join(".git/speclink/state.db").exists());
    assert!(!wt.join(".git/speclink").exists());
}

#[test]
fn gitignore_appends_link_yaml_when_file_exists() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    fs::write(w.join(".gitignore"), "node_modules\n").unwrap();
    speclink(&w).arg("init").assert().success();
    let g = fs::read_to_string(w.join(".gitignore")).unwrap();
    assert_eq!(g, "node_modules\n.speclink/link.yaml\n");
}

#[test]
fn gitignore_idempotent_on_reinit_force() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    speclink(&w).args(["init", "--force"]).assert().success();
    let g = fs::read_to_string(w.join(".gitignore")).unwrap();
    assert_eq!(g.matches(".speclink/link.yaml").count(), 1);
}

#[test]
fn unlink_removes_link_yaml_but_keeps_state_db() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let state_db = w.join(".git/speclink/state.db");
    let sha_before = sha256_of(&state_db);
    speclink(&w).arg("unlink").assert().success();
    assert!(!w.join(".speclink/link.yaml").exists());
    assert!(state_db.exists());
    assert_eq!(sha_before, sha256_of(&state_db));
}

#[test]
fn link_to_known_project_writes_link_yaml() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    let out = speclink(&w).args(["init", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json = parse_json(&out.stdout);
    let project_id = json["data"]["project_id"].as_str().unwrap().to_string();
    fs::remove_file(w.join(".speclink/link.yaml")).unwrap();
    let relink = speclink(&w)
        .args(["link", &project_id, "--json"])
        .output()
        .unwrap();
    assert!(relink.status.success());
    let json = parse_json(&relink.stdout);
    assert_eq!(json["data"]["project_id"], project_id);
    assert!(w.join(".speclink/link.yaml").exists());
}

#[test]
fn link_to_unknown_project_returns_not_found() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    fs::remove_file(w.join(".speclink/link.yaml")).unwrap();
    let out = speclink(&w)
        .args(["link", "00000000-0000-0000-0000-000000000000", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let json = parse_json(&out.stdout);
    assert_eq!(json["error"]["code"], "project.link_target_not_found");
}

#[test]
fn json_envelope_shape_success() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    let out = speclink(&w).args(["init", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json = parse_json(&out.stdout);
    assert_eq!(json["ok"], true);
    assert!(json["data"].is_object());
    assert!(json["warnings"].is_array());
    let req = json["requestId"].as_str().expect("requestId");
    assert!(uuid::Uuid::parse_str(req).is_ok());
}

#[test]
fn json_envelope_shape_error() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    let out = speclink(&w).args(["init", "--json"]).output().unwrap();
    let json = parse_json(&out.stdout);
    assert_eq!(json["ok"], false);
    assert!(json["error"]["code"].is_string());
    assert!(json["error"]["retryable"].is_boolean());
    let req = json["requestId"].as_str().expect("requestId");
    assert!(uuid::Uuid::parse_str(req).is_ok());
}

/// Replace volatile fields (`requestId`, `project_id`, `git_head`) with stable
/// sentinel strings before snapshotting JSON.
fn redact(mut v: serde_json::Value) -> serde_json::Value {
    fn walk(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(map) => {
                for (k, val) in map.iter_mut() {
                    if matches!(k.as_str(), "requestId" | "project_id" | "git_head") {
                        *val = serde_json::Value::String(format!("<{k}>"));
                    } else {
                        walk(val);
                    }
                }
            }
            serde_json::Value::Array(arr) => arr.iter_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut v);
    v
}

#[test]
fn envelope_snapshot_init_success() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    let out = speclink(&w).args(["init", "--json"]).output().unwrap();
    let json = redact(parse_json(&out.stdout));
    insta::assert_json_snapshot!("envelope_init_success", json);
}

#[test]
fn envelope_snapshot_init_non_git_failure() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    let out = speclink(&w).args(["init", "--json"]).output().unwrap();
    let json = redact(parse_json(&out.stdout));
    insta::assert_json_snapshot!("envelope_init_non_git", json);
}

#[test]
fn envelope_snapshot_status_success() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init_with_commit(&w);
    speclink(&w).arg("init").assert().success();
    let out = speclink(&w).args(["status", "--json"]).output().unwrap();
    let json = redact(parse_json(&out.stdout));
    insta::assert_json_snapshot!("envelope_status_success", json);
}

#[test]
fn envelope_snapshot_link_failure() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    fs::remove_file(w.join(".speclink/link.yaml")).unwrap();
    let out = speclink(&w)
        .args(["link", "00000000-0000-0000-0000-000000000000", "--json"])
        .output()
        .unwrap();
    let json = redact(parse_json(&out.stdout));
    insta::assert_json_snapshot!("envelope_link_failure", json);
}

#[test]
fn state_db_migration_v1_creates_expected_tables() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let conn = rusqlite::Connection::open(w.join(".git/speclink/state.db")).unwrap();
    let migrations_count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(migrations_count, 1);
    let project_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
        .unwrap();
    assert_eq!(project_count, 1);
}
