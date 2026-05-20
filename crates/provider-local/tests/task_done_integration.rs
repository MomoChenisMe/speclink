//! `LocalProvider::mark_task_done` 整合測試 — 涵蓋 happy / idempotent / not_found /
//! artifact_missing 與 atomic rollback。

use provider::Provider;
use provider::error::ProviderError;
use provider::model::{ArtifactKind, ChangeId, NewArtifact, NewChange, ProjectId, TaskStatus};
use provider_local::LocalProvider;
use std::path::Path;
use tempfile::TempDir;

const PROJECT: &str = "default";

async fn bootstrap_with_tasks(base: &Path, change_id: &str, tasks_body: &str) {
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
                kind: ArtifactKind::Tasks,
                content: tasks_body.to_string(),
                capability: None,
            },
        )
        .await
        .expect("tasks");
}

async fn bootstrap_without_tasks(base: &Path, change_id: &str) {
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
}

#[tokio::test]
async fn mark_task_done_happy_path_updates_tasks_md() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(
        base,
        "demo",
        "## 1. Setup\n\n- [ ] 1.1 Write tests\n- [ ] 1.2 Configure\n",
    )
    .await;

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let update = provider
        .mark_task_done(&pid, &cid, "1.1")
        .await
        .expect("mark_task_done");
    assert_eq!(update.task_id, "1.1");
    assert_eq!(update.previous_status, TaskStatus::Todo);
    assert_eq!(update.current_status, TaskStatus::Done);
    assert_eq!(update.task_description, "Write tests");

    let path = base.join(".speclink/changes/demo/tasks.md");
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        content,
        "## 1. Setup\n\n- [x] 1.1 Write tests\n- [ ] 1.2 Configure\n"
    );
    // 無 .tmp 殘留
    assert!(!base.join(".speclink/changes/demo/tasks.md.tmp").exists());
}

#[tokio::test]
async fn mark_task_done_idempotent_on_already_done() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(base, "demo", "## 1. Setup\n\n- [x] 1.1 Done\n").await;

    let path = base.join(".speclink/changes/demo/tasks.md");
    let mtime_before = std::fs::metadata(&path).unwrap().modified().unwrap();
    let original = std::fs::read_to_string(&path).unwrap();

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let update = provider
        .mark_task_done(&pid, &cid, "1.1")
        .await
        .expect("idempotent ok");
    assert_eq!(update.previous_status, TaskStatus::Done);
    assert_eq!(update.current_status, TaskStatus::Done);

    let content_after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content_after, original, "content must not change");
    let mtime_after = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(mtime_after, mtime_before, "mtime must not change");
    assert!(!base.join(".speclink/changes/demo/tasks.md.tmp").exists());
}

#[tokio::test]
async fn mark_task_done_not_found_returns_task_not_found() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(base, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n").await;

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let err = provider
        .mark_task_done(&pid, &cid, "1.99")
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::TaskNotFound { .. }));
}

#[tokio::test]
async fn mark_task_done_invalid_id_returns_task_invalid_id() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(base, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n").await;

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let err = provider
        .mark_task_done(&pid, &cid, "1.1.2")
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::TaskInvalidId { .. }));
}

#[tokio::test]
async fn mark_task_done_missing_tasks_md_returns_artifact_missing() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_without_tasks(base, "demo").await;

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let err = provider
        .mark_task_done(&pid, &cid, "1.1")
        .await
        .expect_err("err");
    assert!(matches!(
        err,
        ProviderError::ArtifactMissing { ref artifact_id, .. } if artifact_id == "tasks"
    ));
}

#[tokio::test]
async fn mark_task_done_change_not_found() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    // 不 bootstrap，直接呼叫
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("missing");
    let err = provider
        .mark_task_done(&pid, &cid, "1.1")
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn mark_task_done_preserves_parallel_marker_on_disk() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(
        base,
        "demo",
        "## 2. Refactor\n\n- [ ] 2.3 [P] Refactor parser\n",
    )
    .await;

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let update = provider
        .mark_task_done(&pid, &cid, "2.3")
        .await
        .expect("ok");
    assert_eq!(update.task_description, "[P] Refactor parser");
    let path = base.join(".speclink/changes/demo/tasks.md");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("- [x] 2.3 [P] Refactor parser"));
}

#[cfg(unix)]
#[tokio::test]
async fn mark_task_done_rollback_keeps_tasks_md_when_rename_fails() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap_with_tasks(base, "demo", "## 1. Setup\n\n- [ ] 1.1 First\n").await;

    let dir = base.join(".speclink/changes/demo");
    let original = std::fs::read_to_string(dir.join("tasks.md")).unwrap();
    let prev_mtime = std::fs::metadata(dir.join("tasks.md"))
        .unwrap()
        .modified()
        .unwrap();

    // 把目錄設為 readonly — 阻止 `.tmp` 寫入與 rename
    let mut perms = std::fs::metadata(&dir).unwrap().permissions();
    perms.set_mode(0o555);
    std::fs::set_permissions(&dir, perms).unwrap();

    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let pid = ProjectId::from(PROJECT);
    let cid = ChangeId::from("demo");
    let err = provider
        .mark_task_done(&pid, &cid, "1.1")
        .await
        .expect_err("rename must fail");

    // 還原權限以便 TempDir 清掉
    let mut restore = std::fs::metadata(&dir).unwrap().permissions();
    restore.set_mode(0o755);
    std::fs::set_permissions(&dir, restore).unwrap();

    // 必須是 internal.error（io）
    assert!(matches!(err, ProviderError::Internal { .. }));

    // tasks.md 內容未變
    let after = std::fs::read_to_string(dir.join("tasks.md")).unwrap();
    assert_eq!(after, original);
    let after_mtime = std::fs::metadata(dir.join("tasks.md"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(after_mtime, prev_mtime);
    // 無 .tmp 殘留
    assert!(!dir.join("tasks.md.tmp").exists());
}
