//! `LocalArchiveStore` integration tests — 對齊 archive-runner spec 的 5 條核心
//! requirement scenarios：
//! 1. happy path single-tx + dir rename + spec merge order
//! 2. state guard 矩陣（5 個非法 state + `in_progress + all_tasks_done=0`）
//! 3. same-day collision append suffix
//! 4. `--skip-specs` 路徑
//! 5. rename 失敗 best-effort revert
//!
//! 對應 design 決策「State guard：兩條件 AND vs 完整 6-state check」
//! 「Archive 目錄命名：日期前綴 vs 純 change-id」
//! 「Spec delta merge：整檔覆蓋 vs schema-aware diff」。

use std::fs;
use std::path::{Path, PathBuf};

use speclink_provider::{ArchiveRequest, ArchiveStore, ChangeState, ProviderError, codes};
use speclink_provider_local::{LocalArchiveStore, StateDb};
use tempfile::TempDir;

const STATES: &[&str] = &[
    "proposing",
    "reviewing",
    "ready",
    "code_reviewing",
    "archived",
];

/// 設定 working tree + state.db (v4) + 1 個 change row，state/all_tasks_done 可調。
fn seed_workspace(state: &str, all_tasks_done: bool) -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working_dir = tmp.path().to_path_buf();
    let state_root = working_dir.join(".git").join("speclink");
    fs::create_dir_all(&state_root).expect("create state root");

    // change dir
    let change_dir = working_dir.join(".speclink").join("changes").join("demo");
    fs::create_dir_all(change_dir.join("specs").join("user-auth")).expect("create cap dir");
    fs::create_dir_all(change_dir.join("specs").join("audit-log")).expect("create cap2 dir");
    fs::write(change_dir.join("proposal.md"), "# Proposal\n").expect("proposal");
    fs::write(change_dir.join("tasks.md"), "- [x] do thing\n").expect("tasks");
    fs::write(
        change_dir.join("specs").join("user-auth").join("spec.md"),
        "## Purpose\nauth flow.\n",
    )
    .expect("spec1");
    fs::write(
        change_dir.join("specs").join("audit-log").join("spec.md"),
        "## Purpose\naudit ledger.\n",
    )
    .expect("spec2");

    // state.db v4 with one change row
    let db_path = state_root.join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(4).expect("v4");
    db.insert_change_row(
        "demo",
        "demo",
        state,
        "spec-driven",
        "2026-05-22T10:00:00Z",
        "2026-05-22T10:00:00Z",
    )
    .expect("seed row");
    if all_tasks_done {
        db.cas_set_all_tasks_done("demo", 1, true, "2026-05-22T10:00:00Z")
            .expect("set all_tasks_done");
    }
    (tmp, working_dir, state_root)
}

fn make_request(skip_specs: bool) -> ArchiveRequest {
    ArchiveRequest {
        change_id: "demo".into(),
        skip_specs,
        no_validate: false,
        yes: false,
    }
}

fn assert_dir_contains(p: &Path, names: &[&str]) {
    for n in names {
        assert!(p.join(n).exists(), "expected {n} under {p:?}");
    }
}

#[tokio::test]
async fn happy_path_archives_in_progress_change_with_all_tasks_done() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    let store = LocalArchiveStore::new(working_dir.clone(), state_root.clone());
    let result = store
        .archive_change(make_request(false))
        .await
        .expect("archive");
    assert_eq!(result.change_id, "demo");
    assert_eq!(result.state, ChangeState::Archived);
    assert_eq!(result.merged_specs.len(), 2);
    let caps: Vec<_> = result
        .merged_specs
        .iter()
        .map(|m| m.capability.as_str())
        .collect();
    assert!(caps.contains(&"user-auth"));
    assert!(caps.contains(&"audit-log"));
    for m in &result.merged_specs {
        assert!(m.lines_added > 0, "lines_added > 0 for {}", m.capability);
        assert_eq!(m.lines_removed, 0, "new capability dir => lines_removed=0");
    }
    // archive dir present & old change dir gone
    let archive_root = working_dir
        .join(".speclink")
        .join("changes")
        .join("archive");
    let entries: Vec<_> = fs::read_dir(&archive_root)
        .expect("read archive root")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1, "exactly one archive entry");
    let archive_entry = entries[0].path();
    assert!(
        archive_entry
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("-demo")
    );
    assert_dir_contains(&archive_entry, &["proposal.md", "tasks.md", "specs"]);
    let old_change_dir = working_dir.join(".speclink").join("changes").join("demo");
    assert!(!old_change_dir.exists(), "old change dir SHALL be removed");
    // specs promoted
    for cap in ["user-auth", "audit-log"] {
        let target = working_dir
            .join(".speclink")
            .join("specs")
            .join(cap)
            .join("spec.md");
        assert!(target.is_file(), "spec.md promoted for {cap}");
    }
    // state.db: state=archived, archived_at non-NULL, state_transition row
    let db = StateDb::open(&state_root.join("state.db")).expect("open db");
    let row = db.read_change_state_row("demo").expect("read");
    assert_eq!(row.state, "archived");
    let archived_at = db.read_archived_at("demo").expect("read archived_at");
    assert!(archived_at.is_some(), "archived_at SHALL be set");
    let conn = rusqlite::Connection::open(state_root.join("state.db")).expect("reopen");
    let cnt: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM state_transition WHERE change_id='demo' AND reason='archive_run'",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(cnt, 1, "exactly one archive_run audit row");
}

#[tokio::test]
async fn state_guard_rejects_each_non_in_progress_state() {
    for &state in STATES {
        let (_tmp, working_dir, state_root) = seed_workspace(state, true);
        let store = LocalArchiveStore::new(working_dir, state_root);
        let err = store
            .archive_change(make_request(false))
            .await
            .expect_err(&format!("state={state} SHALL be rejected"));
        match err {
            ProviderError::StateTransitionInvalid { from, to } => {
                assert_eq!(from, state);
                assert_eq!(to, "archived");
            }
            other => panic!("state={state}: expected StateTransitionInvalid, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn state_guard_rejects_in_progress_when_all_tasks_done_is_false() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", false);
    let store = LocalArchiveStore::new(working_dir, state_root);
    let err = store
        .archive_change(make_request(false))
        .await
        .expect_err("all_tasks_done=0 SHALL be rejected");
    assert_eq!(err_code(&err), codes::CHANGE_TASKS_INCOMPLETE);
    match &err {
        ProviderError::ChangeTasksIncomplete { change_id } => {
            assert_eq!(change_id, "demo");
        }
        other => panic!("expected ChangeTasksIncomplete, got {other:?}"),
    }
}

fn err_code(e: &ProviderError) -> &'static str {
    e.code()
}

#[tokio::test]
async fn change_not_found_when_change_id_absent_from_state_db() {
    let tmp = TempDir::new().expect("tempdir");
    let working_dir = tmp.path().to_path_buf();
    let state_root = working_dir.join(".git").join("speclink");
    fs::create_dir_all(&state_root).expect("state root");
    let db = StateDb::open(&state_root.join("state.db")).expect("open");
    db.migrate(4).expect("v4");
    let store = LocalArchiveStore::new(working_dir, state_root);
    let err = store
        .archive_change(make_request(false))
        .await
        .expect_err("missing change_id SHALL fail");
    assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn same_day_collision_appends_dash_n_suffix() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    // Pre-create same-day archive dir to trigger collision
    let archive_root = working_dir
        .join(".speclink")
        .join("changes")
        .join("archive");
    fs::create_dir_all(&archive_root).expect("archive root");
    let occupied = first_day_dir_name(&archive_root);
    fs::create_dir_all(archive_root.join(&occupied)).expect("pre-occupy");
    let store = LocalArchiveStore::new(working_dir.clone(), state_root);
    let result = store
        .archive_change(make_request(false))
        .await
        .expect("archive should still succeed with suffix");
    assert!(
        result.archive_dir.ends_with("-demo-2") || result.archive_dir.contains("-demo-2/"),
        "expected -2 suffix, got: {}",
        result.archive_dir
    );
    let mut entries: Vec<String> = fs::read_dir(&archive_root)
        .expect("read")
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
        .collect();
    entries.sort();
    assert_eq!(entries.len(), 2);
    assert!(entries[1].ends_with("-demo-2"));
}

/// Helper：用 archive root 中沒檔的「今日」格式 name；採 RFC3339 prefix 切 10 char。
fn first_day_dir_name(_archive_root: &Path) -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;
    let today = OffsetDateTime::now_utc();
    let rfc = today.format(&Rfc3339).expect("fmt");
    let date = &rfc[..10]; // YYYY-MM-DD prefix
    format!("{date}-demo")
}

#[tokio::test]
async fn skip_specs_path_leaves_specs_untouched_and_reports_no_merged_specs() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    // Pre-existing spec content under .speclink/specs/user-auth/spec.md
    let preexisting_target = working_dir
        .join(".speclink")
        .join("specs")
        .join("user-auth")
        .join("spec.md");
    fs::create_dir_all(preexisting_target.parent().unwrap()).expect("mkdir");
    let original_content = b"## Original content; SHALL remain byte-for-byte\n";
    fs::write(&preexisting_target, original_content).expect("seed pre-existing");
    let store = LocalArchiveStore::new(working_dir.clone(), state_root.clone());
    let result = store
        .archive_change(make_request(true))
        .await
        .expect("archive skip-specs");
    assert!(
        result.merged_specs.is_empty(),
        "skip_specs SHALL produce empty merged_specs"
    );
    // Pre-existing spec content unchanged
    let after = fs::read(&preexisting_target).expect("read after");
    assert_eq!(after, original_content);
    // audit-log spec NOT promoted (path absent)
    assert!(
        !working_dir
            .join(".speclink")
            .join("specs")
            .join("audit-log")
            .join("spec.md")
            .exists(),
        "audit-log spec.md SHALL NOT be promoted under --skip-specs"
    );
    // state still transitioned, dir still renamed
    let db = StateDb::open(&state_root.join("state.db")).expect("open");
    let row = db.read_change_state_row("demo").expect("read");
    assert_eq!(row.state, "archived");
}

#[tokio::test]
async fn empty_specs_path_reports_no_merged_specs_without_warning_carrier() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    // remove pre-seeded specs dir to simulate no capabilities
    let specs_dir = working_dir
        .join(".speclink")
        .join("changes")
        .join("demo")
        .join("specs");
    fs::remove_dir_all(&specs_dir).expect("rm specs dir");
    let store = LocalArchiveStore::new(working_dir, state_root);
    let result = store
        .archive_change(make_request(false))
        .await
        .expect("archive");
    assert!(result.merged_specs.is_empty());
    assert_eq!(result.state, ChangeState::Archived);
}

#[tokio::test]
async fn spec_merge_overwrites_existing_target_with_old_lines_count() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    // Pre-existing 5-line target
    let target = working_dir
        .join(".speclink")
        .join("specs")
        .join("user-auth")
        .join("spec.md");
    fs::create_dir_all(target.parent().unwrap()).expect("mkdir");
    fs::write(&target, "L1\nL2\nL3\nL4\nL5\n").expect("seed");
    let store = LocalArchiveStore::new(working_dir.clone(), state_root);
    let result = store
        .archive_change(make_request(false))
        .await
        .expect("archive");
    let user_auth = result
        .merged_specs
        .iter()
        .find(|m| m.capability == "user-auth")
        .expect("user-auth row");
    assert_eq!(user_auth.lines_removed, 5, "old file had 5 lines");
    // new file written from change spec
    let after = fs::read_to_string(&target).expect("read");
    assert!(after.contains("auth flow"), "got: {after}");
}

#[tokio::test]
async fn rename_failure_triggers_best_effort_revert() {
    // Trigger rename failure by pre-creating target dir AS A FILE (causes rename to fail with EEXIST/ENOTDIR).
    // Actually simpler: pre-occupy collisions all the way up to 100 → resolve_archive_dir returns Internal error
    // BEFORE we even tx-commit, so no revert path triggers. To force POST-COMMIT rename failure, we'll
    // simulate by making the source dir read-only OR by another method.
    //
    // Simpler shim: pre-occupy 100 collision suffixes will hit Internal on resolve_archive_dir, but that's
    // a pre-commit failure path (no revert needed; tx not started). To force the post-commit revert path,
    // we make the target archive ROOT a FILE so fs::create_dir_all fails AT rename time but only after commit.
    //
    // Easiest: create archive root as a regular file before archive.run; the impl will fail on
    // `fs::create_dir_all(archive_root)` (after the tx commit) → triggers revert.
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    let changes_dir = working_dir.join(".speclink").join("changes");
    let archive_path = changes_dir.join("archive");
    // ensure "archive" is a regular file, not a directory
    fs::create_dir_all(&changes_dir).expect("mkdir");
    fs::write(&archive_path, b"not a dir").expect("seed file");
    let store = LocalArchiveStore::new(working_dir.clone(), state_root.clone());
    let err = store
        .archive_change(make_request(false))
        .await
        .expect_err("rename SHALL fail with archive root being a file");
    assert!(matches!(err, ProviderError::Internal(_)), "got: {err:?}");
    // State reverted: read state.db, expect state=in_progress, archived_at=NULL,
    // and one revert row in state_transition.
    let db = StateDb::open(&state_root.join("state.db")).expect("open db");
    let row = db.read_change_state_row("demo").expect("read row");
    assert_eq!(row.state, "in_progress", "state SHALL be reverted");
    let archived_at = db.read_archived_at("demo").expect("read archived_at");
    assert!(
        archived_at.is_none(),
        "archived_at SHALL be cleared by revert"
    );
    let conn = rusqlite::Connection::open(state_root.join("state.db")).expect("reopen");
    let revert_cnt: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM state_transition WHERE reason='archive_run_revert'",
            [],
            |r| r.get(0),
        )
        .expect("query");
    assert_eq!(revert_cnt, 1, "exactly one archive_run_revert row");
    // Original change dir SHALL still exist (rename never completed)
    assert!(
        working_dir
            .join(".speclink")
            .join("changes")
            .join("demo")
            .exists(),
        "original change dir SHALL remain after revert"
    );
}

#[tokio::test]
async fn cross_platform_rename_within_same_mount_succeeds() {
    // 對齊 design「Cross-device rename」row：本 slice 只支援同 mount fs::rename。
    // 此測試確認在 Linux/macOS/Windows 同 mount 上 archive 的 fs::rename 對含 nested
    // subdirs 的 change dir 一致成功；不模擬 cross-device EXDEV（留 doctor slice）。
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    let store = LocalArchiveStore::new(working_dir.clone(), state_root.clone());
    let result = store
        .archive_change(make_request(false))
        .await
        .expect("rename within same mount");
    assert_eq!(result.state, ChangeState::Archived);
    // verify nested subdirs preserved
    let archive_dir = working_dir.join(&result.archive_dir);
    assert!(archive_dir.join("specs/user-auth/spec.md").exists());
    assert!(archive_dir.join("specs/audit-log/spec.md").exists());
}

#[tokio::test]
async fn resolve_archive_dir_fails_after_exhausting_100_collisions() {
    let (_tmp, working_dir, state_root) = seed_workspace("in_progress", true);
    let archive_root = working_dir
        .join(".speclink")
        .join("changes")
        .join("archive");
    fs::create_dir_all(&archive_root).expect("archive root");
    let base = first_day_dir_name(&archive_root);
    fs::create_dir_all(archive_root.join(&base)).expect("base");
    for n in 2..=100 {
        fs::create_dir_all(archive_root.join(format!("{base}-{n}"))).expect("collision dir");
    }
    let store = LocalArchiveStore::new(working_dir, state_root);
    let err = store
        .archive_change(make_request(false))
        .await
        .expect_err("100 collisions SHALL fail");
    assert!(matches!(err, ProviderError::Internal(_)));
}
