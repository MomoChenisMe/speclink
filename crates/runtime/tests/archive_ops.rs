//! `ArchiveOperations::run` integration tests — runtime 層 archive.run 路徑的 e2e。
//!
//! 對應 archive-runner spec「Spec delta merge SHALL atomically overwrite the target
//! capability spec for each capability under the change」「`--skip-specs` SHALL bypass
//! merge while still transitioning state and emit an audit warning」「JSON envelope
//! SHALL conform to the bootstrap / A2 / A3 contract」與 design「Observable behavior」
//! 「JSON envelope shape」決策。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use speclink_provider::ChangeState;
use speclink_provider_local::StateDb;
use speclink_runtime::{ArchiveOperations, RealGitProbe, RuntimeError};
use tempfile::TempDir;

fn init_git_dir(tmp: &TempDir) -> PathBuf {
    let dir = tmp.path().to_path_buf();
    let out = Command::new("git")
        .args(["init", "--quiet", "--initial-branch=main"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    assert!(out.status.success(), "git init: {:?}", out.stderr);
    dir
}

fn seed_change(
    working_dir: &Path,
    name: &str,
    state: &str,
    all_tasks_done: bool,
    capabilities: &[&str],
) -> PathBuf {
    let change_dir = working_dir.join(".speclink").join("changes").join(name);
    fs::create_dir_all(change_dir.join("specs")).expect("create change dir");
    fs::write(change_dir.join("proposal.md"), b"# Proposal\n").expect("proposal");
    fs::write(change_dir.join("tasks.md"), b"- [x] do thing\n").expect("tasks");
    for cap in capabilities {
        let cap_dir = change_dir.join("specs").join(cap);
        fs::create_dir_all(&cap_dir).expect("create cap dir");
        fs::write(
            cap_dir.join("spec.md"),
            format!("## Purpose\nLines for {cap}.\n").as_bytes(),
        )
        .expect("spec");
    }

    let state_root = working_dir.join(".git").join("speclink");
    fs::create_dir_all(&state_root).expect("create state root");
    let db = StateDb::open(&state_root.join("state.db")).expect("open");
    db.migrate(4).expect("v4");
    db.insert_change_row(
        name,
        name,
        state,
        "spec-driven",
        "2026-05-22T10:00:00Z",
        "2026-05-22T10:00:00Z",
    )
    .expect("seed change row");
    if all_tasks_done {
        db.cas_set_all_tasks_done(name, 1, true, "2026-05-22T10:00:00Z")
            .expect("set all_tasks_done");
    }
    change_dir
}

#[tokio::test]
async fn run_happy_path_returns_archive_data_with_expected_shape() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(
        &working_dir,
        "demo",
        "in_progress",
        true,
        &["user-auth", "audit-log"],
    );
    let ops = ArchiveOperations::new(RealGitProbe);
    let out = ops
        .run(&working_dir, "demo", false, false, false)
        .await
        .expect("archive run");
    assert_eq!(out.data.change_id, "demo");
    assert_eq!(out.data.state, ChangeState::Archived);
    assert_eq!(out.data.merged_specs.len(), 2);
    assert!(
        out.data
            .archive_dir
            .starts_with(".speclink/changes/archive/")
    );
    assert!(out.warnings.is_empty(), "happy path has no warnings");
}

#[tokio::test]
async fn run_skip_specs_appends_archive_specs_skipped_warning_with_sorted_capabilities() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(
        &working_dir,
        "demo",
        "in_progress",
        true,
        &["user-auth", "audit-log"],
    );
    let ops = ArchiveOperations::new(RealGitProbe);
    let out = ops
        .run(&working_dir, "demo", true, false, false)
        .await
        .expect("archive run --skip-specs");
    assert!(out.data.merged_specs.is_empty());
    assert_eq!(out.warnings.len(), 1, "exactly one warning");
    let w = &out.warnings[0];
    assert_eq!(w.code, "archive.specs_skipped");
    let caps = w
        .details
        .as_ref()
        .and_then(|d| d.get("capabilities_skipped"))
        .and_then(|v| v.as_array())
        .expect("capabilities_skipped array");
    let names: Vec<&str> = caps.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(names, vec!["audit-log", "user-auth"]);
}

#[tokio::test]
async fn run_skip_specs_with_no_capabilities_does_not_emit_warning() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(&working_dir, "demo", "in_progress", true, &[]);
    let ops = ArchiveOperations::new(RealGitProbe);
    let out = ops
        .run(&working_dir, "demo", true, false, false)
        .await
        .expect("archive run --skip-specs (empty specs)");
    assert!(out.data.merged_specs.is_empty());
    assert!(out.warnings.is_empty());
}

#[tokio::test]
async fn run_rejects_proposing_with_state_transition_invalid() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(&working_dir, "demo", "proposing", true, &["user-auth"]);
    let ops = ArchiveOperations::new(RealGitProbe);
    let err = ops
        .run(&working_dir, "demo", false, false, false)
        .await
        .expect_err("proposing SHALL be rejected");
    match err {
        RuntimeError::StateTransitionInvalid { from, to } => {
            assert_eq!(from, "proposing");
            assert_eq!(to, "archived");
        }
        other => panic!("expected StateTransitionInvalid, got {other:?}"),
    }
}

#[tokio::test]
async fn run_rejects_in_progress_with_all_tasks_done_false() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(&working_dir, "demo", "in_progress", false, &["user-auth"]);
    let ops = ArchiveOperations::new(RealGitProbe);
    let err = ops
        .run(&working_dir, "demo", false, false, false)
        .await
        .expect_err("all_tasks_done=0 SHALL be rejected");
    match err {
        RuntimeError::ChangeTasksIncomplete { change_id } => {
            assert_eq!(change_id, "demo");
        }
        other => panic!("expected ChangeTasksIncomplete, got {other:?}"),
    }
}

#[tokio::test]
async fn run_rejects_missing_change_with_change_not_found() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    // open state.db so migrate runs but do not seed any change row
    let state_root = working_dir.join(".git").join("speclink");
    fs::create_dir_all(&state_root).expect("state root");
    let db = StateDb::open(&state_root.join("state.db")).expect("open");
    db.migrate(4).expect("v4");
    let ops = ArchiveOperations::new(RealGitProbe);
    let err = ops
        .run(&working_dir, "ghost", false, false, false)
        .await
        .expect_err("missing change SHALL fail");
    assert!(matches!(err, RuntimeError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn run_spec_merge_records_old_line_count_when_target_exists() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(&working_dir, "demo", "in_progress", true, &["user-auth"]);
    // pre-existing 3-line target
    let target = working_dir
        .join(".speclink")
        .join("specs")
        .join("user-auth")
        .join("spec.md");
    fs::create_dir_all(target.parent().unwrap()).expect("mkdir");
    fs::write(&target, b"L1\nL2\nL3\n").expect("seed");
    let ops = ArchiveOperations::new(RealGitProbe);
    let out = ops
        .run(&working_dir, "demo", false, false, false)
        .await
        .expect("archive");
    let row = out
        .data
        .merged_specs
        .iter()
        .find(|m| m.capability == "user-auth")
        .expect("user-auth row");
    assert_eq!(row.lines_removed, 3, "old file had 3 lines");
    assert!(row.lines_added > 0);
}

#[tokio::test]
async fn run_no_validate_and_yes_flags_are_no_ops() {
    let tmp = TempDir::new().expect("tmp");
    let working_dir = init_git_dir(&tmp);
    seed_change(&working_dir, "demo", "in_progress", true, &["user-auth"]);
    let ops = ArchiveOperations::new(RealGitProbe);
    let out = ops
        .run(&working_dir, "demo", false, true, true)
        .await
        .expect("archive with no_validate=true and yes=true");
    assert_eq!(out.data.state, ChangeState::Archived);
    assert!(out.warnings.is_empty(), "no-op flags emit no warnings");
}
