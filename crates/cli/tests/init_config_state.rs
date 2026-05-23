//! Section 9 integration tests：`speclink init` inserts `config_state` row +
//! creates `.speclink/config.yaml` with walking-skeleton defaults。

use assert_cmd::Command as AssertCommand;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "command failed: {cmd:?}");
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

fn db_query_config_state(working: &Path) -> (i64, String, i64, Option<String>) {
    let db_path = working.join(".git").join("speclink").join("state.db");
    let conn = Connection::open(&db_path).expect("open db");
    conn.query_row(
        "SELECT id, content_sha256, version, written_by FROM config_state WHERE id = 1",
        [],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        },
    )
    .expect("query row")
}

// ----- 9.1 fresh init seeds config_state row -----

#[test]
fn fresh_init_inserts_config_state_row_with_walking_skeleton_defaults() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();

    // (a) config.yaml exists with default content.
    let config_path = w.join(".speclink").join("config.yaml");
    assert!(config_path.is_file());
    let body = std::fs::read(&config_path).unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("require_artifact_review: false"));
    assert!(body_str.contains("require_code_review: false"));

    // (b) config_state row: id=1, version=1, content_sha256 matches, written_by=NULL.
    let (id, sha, version, written_by) = db_query_config_state(&w);
    assert_eq!(id, 1);
    assert_eq!(version, 1, "fresh init SHALL seed version=1");
    let expected_sha = hex::encode(Sha256::digest(&body));
    assert_eq!(
        sha, expected_sha,
        "content_sha256 SHALL match config.yaml sha"
    );
    assert!(written_by.is_none(), "written_by SHALL default to NULL");
}

// ----- 9.3 failed init leaves no config_state row -----
//
// Strategy：force = false 對既已存在的 .speclink/link.yaml → init 抛 AlreadyInitialized；
// 此時 config_state 維持原樣（不該被破壞），新 init 也不該寫入新 row。

#[test]
fn second_init_failure_does_not_perturb_existing_config_state() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let (_, sha_before, version_before, _) = db_query_config_state(&w);

    // 第二次 init 沒 --force → fail with AlreadyInitialized (exit 7)。
    speclink(&w).arg("init").assert().failure();

    // config_state row 完全沒動。
    let (_, sha_after, version_after, _) = db_query_config_state(&w);
    assert_eq!(sha_before, sha_after);
    assert_eq!(version_before, version_after);
}

// ----- 9.4 init --force keeps config_state row when config.yaml unchanged -----

#[test]
fn init_force_keeps_config_state_row_aligned_when_config_unchanged() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    git_init(&w);
    speclink(&w).arg("init").assert().success();
    let (_, sha_before, version_before, _) = db_query_config_state(&w);

    // --force re-init：state.db keeps existing config_state；config.yaml not overwritten
    // when target exists（commit phase guard：`!target_config.exists()`）.
    speclink(&w).args(["init", "--force"]).assert().success();

    let (_, sha_after, version_after, _) = db_query_config_state(&w);
    assert_eq!(
        sha_before, sha_after,
        "force re-init SHALL keep sha aligned"
    );
    assert_eq!(
        version_before, version_after,
        "force re-init SHALL keep version unchanged when bytes unchanged"
    );
}
