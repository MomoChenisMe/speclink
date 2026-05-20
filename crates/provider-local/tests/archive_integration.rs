//! `LocalProvider::archive_change` 整合測試 — 模擬 propose create → write artifacts → archive
//! 的完整 happy / failure / rollback 流程，使用 `tempfile::TempDir` 模擬隔離環境。

use chrono::NaiveDate;
use provider::Provider;
use provider::model::{
    ArchiveOptions, ArtifactKind, ChangeId, NewArtifact, NewChange, ProjectId, State,
};
use provider_local::LocalProvider;
use std::path::Path;
use tempfile::TempDir;

const PROJECT: &str = "default";

fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
}

async fn bootstrap_change_with_spec(
    base: &Path,
    change_id: &str,
    capability: &str,
    delta_body: &str,
) {
    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from(change_id);
    provider
        .create_change(
            &pid,
            NewChange {
                change_id: cid.clone(),
                summary: "test".to_string(),
            },
        )
        .await
        .expect("create_change");
    provider
        .write_artifact(
            &pid,
            &cid,
            NewArtifact {
                kind: ArtifactKind::Proposal,
                content: "## Why\n\ntest\n".to_string(),
                capability: None,
            },
        )
        .await
        .expect("proposal");
    provider
        .write_artifact(
            &pid,
            &cid,
            NewArtifact {
                kind: ArtifactKind::Spec,
                content: delta_body.to_string(),
                capability: Some(capability.to_string()),
            },
        )
        .await
        .expect("spec");
}

#[tokio::test]
async fn archive_happy_path_moves_dir_and_creates_main_spec() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\n#### Scenario: Email\n\nbody\n",
    )
    .await;

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    let result = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect("archive ok");

    assert_eq!(result.change_id.as_str(), "demo");
    assert_eq!(result.state, State::Archived);
    assert!(!result.dry_run);
    assert!(result.archive_path.contains("2026-05-19-demo"));
    assert_eq!(result.spec_sync.capabilities_synced.len(), 1);
    let cs = &result.spec_sync.capabilities_synced[0];
    assert_eq!(cs.capability, "auth");
    assert_eq!(cs.added_count, 1);
    assert!(cs.created_main_spec);

    // active dir gone, archive dir exists
    assert!(!base.join(".speclink/changes/demo").exists());
    let archive_dir = base.join(".speclink/changes/archive/2026-05-19-demo");
    assert!(archive_dir.is_dir());
    // main spec written
    let main_spec = base.join(".speclink/specs/auth/spec.md");
    assert!(main_spec.is_file());
    let body = std::fs::read_to_string(&main_spec).unwrap();
    assert!(body.contains("### Requirement: User login"));
    assert!(body.contains("#### Scenario: Email"));
    // metadata.json updated in archive dir
    let meta: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(archive_dir.join("metadata.json")).unwrap())
            .unwrap();
    assert_eq!(meta["state"], "archived");
    assert!(meta["archivedAt"].is_string());
}

#[tokio::test]
async fn archive_dry_run_leaves_filesystem_unchanged() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody\n",
    )
    .await;

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    let result = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: true,
                archive_date: date(),
            },
        )
        .await
        .expect("dry-run ok");

    assert!(result.dry_run);
    assert_eq!(result.spec_sync.capabilities_synced.len(), 1);
    assert!(result.spec_sync.capabilities_synced[0].created_main_spec);
    // 仍然存在
    assert!(base.join(".speclink/changes/demo").is_dir());
    // archive 目錄不存在（沒有先前 archive）
    assert!(!base.join(".speclink/changes/archive").exists());
    // 主 spec 目錄不存在
    assert!(!base.join(".speclink/specs").exists());
}

#[tokio::test]
async fn archive_already_archived_returns_change_not_archivable() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    )
    .await;

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect("first archive ok");

    // Re-archive — but archive subdir already exists; metadata is in archive subdir, so
    // 重複呼叫 archive_change 對同 id 將回 ChangeNotFound（active dir 不存在），這對應
    // spec scenario：already-archived 應回 change_not_archivable。CLI 層把 ChangeNotFound
    // 映射為 change.not_found，但本測試模擬的場景是「metadata 已 archived」。
    // 為驗證 already-archived 場景，將 archive dir 內 metadata 還原 active 位置：
    let archive_dir = base.join(".speclink/changes/archive/2026-05-19-demo");
    let active_dir = base.join(".speclink/changes/demo");
    std::fs::rename(&archive_dir, &active_dir).expect("manual restore");

    let result = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect_err("must err");
    assert_eq!(result.error_code(), "archive.change_not_archivable");
}

#[tokio::test]
async fn archive_same_day_target_dir_rejected() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    )
    .await;

    // 預先建立同日同名目錄
    std::fs::create_dir_all(base.join(".speclink/changes/archive/2026-05-19-demo")).unwrap();

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    let err = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect_err("must err");
    assert_eq!(err.error_code(), "archive.change_not_archivable");

    // active dir 未動
    assert!(base.join(".speclink/changes/demo").is_dir());
}

#[tokio::test]
async fn archive_delta_conflict_aborts_no_side_effect() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    // 先 archive 第一次建立主 spec
    bootstrap_change_with_spec(
        base,
        "first",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody1\n",
    )
    .await;
    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("first"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect("first archive ok");

    // 第二個 change，delta ADDED 同名 → conflict
    bootstrap_change_with_spec(
        base,
        "second",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nbody2\n",
    )
    .await;

    let err = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("second"),
            ArchiveOptions {
                dry_run: false,
                archive_date: NaiveDate::from_ymd_opt(2026, 5, 20).unwrap(),
            },
        )
        .await
        .expect_err("conflict");
    assert_eq!(err.error_code(), "spec.delta_conflict");
    // active dir 未動
    assert!(base.join(".speclink/changes/second").is_dir());
    // main spec 內容仍是 body1
    let main = std::fs::read_to_string(base.join(".speclink/specs/auth/spec.md")).unwrap();
    assert!(main.contains("body1"));
    assert!(!main.contains("body2"));
}

#[tokio::test]
async fn archive_idempotent_sqlite_cleanup_succeeds_when_row_absent() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    )
    .await;

    // 手動把 in_progress_change 表清空（模擬「SQLite 已無 row」）
    let conn = rusqlite::Connection::open(base.join(".speclink/state.db")).unwrap();
    conn.execute("DELETE FROM in_progress_change", []).unwrap();
    drop(conn);

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    let result = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await;
    assert!(
        result.is_ok(),
        "archive must succeed even when SQLite row absent"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn archive_failed_step_preserves_main_spec_from_bak() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();

    // 先 archive 一次建立既有主 spec，供本測試的 .bak 路徑使用
    bootstrap_change_with_spec(
        base,
        "first",
        "auth",
        "## ADDED Requirements\n\n### Requirement: User login\n\nORIGINAL\n",
    )
    .await;
    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("first"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect("first archive ok");
    let main_spec = base.join(".speclink/specs/auth/spec.md");
    assert!(main_spec.is_file());
    let original_main = std::fs::read_to_string(&main_spec).unwrap();
    assert!(original_main.contains("ORIGINAL"));

    // 第二個 change，delta MODIFIED 現有 requirement → 套用會成功，但故意觸發 step 6 失敗：
    // 把主 spec 所在 cap 目錄設為 readonly，rename .tmp → spec.md 會失敗（EACCES）。
    bootstrap_change_with_spec(
        base,
        "second",
        "auth",
        "## MODIFIED Requirements\n\n### Requirement: User login\n\nNEW\n",
    )
    .await;
    let cap_dir = base.join(".speclink/specs/auth");
    let prev_mode = std::fs::metadata(&cap_dir).unwrap().permissions().mode();
    std::fs::set_permissions(&cap_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("second"),
            ArchiveOptions {
                dry_run: false,
                archive_date: NaiveDate::from_ymd_opt(2026, 5, 20).unwrap(),
            },
        )
        .await;

    // 恢復權限以便讀取主 spec
    std::fs::set_permissions(&cap_dir, std::fs::Permissions::from_mode(prev_mode)).unwrap();

    assert!(
        result.is_err(),
        "archive must fail when step 3/6 cannot write"
    );
    // active dir 應保留（rollback 不應搬走）
    assert!(base.join(".speclink/changes/second").is_dir());
    // 主 spec 內容（從 .bak 還原或從未動到）應仍是原內容
    let after = std::fs::read_to_string(&main_spec).unwrap();
    assert!(
        after.contains("ORIGINAL"),
        "main spec must retain ORIGINAL content after rollback; got: {after}"
    );
    assert!(
        !after.contains("NEW"),
        "main spec must not contain new content after rollback"
    );
}

#[tokio::test]
async fn metadata_after_archive_contains_archivedat() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_change_with_spec(
        base,
        "demo",
        "auth",
        "## ADDED Requirements\n\n### Requirement: A\n\nbody\n",
    )
    .await;
    // capture pre-archive metadata
    let active_meta_path = base.join(".speclink/changes/demo/metadata.json");
    let pre: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&active_meta_path).unwrap()).unwrap();
    let created_at_pre = pre["createdAt"].as_str().unwrap().to_string();

    let provider = LocalProvider::new(base.to_path_buf())
        .await
        .expect("provider");
    provider
        .archive_change(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArchiveOptions {
                dry_run: false,
                archive_date: date(),
            },
        )
        .await
        .expect("ok");

    let archived_meta_path = base.join(".speclink/changes/archive/2026-05-19-demo/metadata.json");
    let m: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&archived_meta_path).unwrap()).unwrap();
    assert_eq!(m["state"], "archived");
    assert_eq!(m["changeId"], "demo");
    assert_eq!(m["createdAt"], created_at_pre);
    let archived_at = m["archivedAt"].as_str().expect("archivedAt string");
    // 寬鬆 ISO 8601 檢查
    assert!(archived_at.ends_with('Z'));
    assert_eq!(archived_at.len(), "2026-05-19T12:34:56Z".len());
}
