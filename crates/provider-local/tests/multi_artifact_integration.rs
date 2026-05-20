//! Integration test：跨 `propose create` 與多 artifact `write_artifact` 的 LocalProvider 行為。
//!
//! 覆蓋 spec：
//! - `Multi-artifact atomic write`（design/tasks/spec 不更新 metadata.json）
//! - `Change status filesystem scan`（get_status 排序與 missing 處理）
//! - `Local provider directory layout`（檔案只在被寫入時才出現）

use provider::Provider;
use provider::error::ProviderError;
use provider::model::{ArtifactKind, ArtifactState, ChangeId, NewArtifact, NewChange, ProjectId};
use provider_local::LocalProvider;
use tempfile::TempDir;

async fn bootstrap_proposal(base: &std::path::Path) -> LocalProvider {
    let provider = LocalProvider::new(base.to_path_buf()).await.expect("new");
    let project_id = ProjectId::from("p");
    provider
        .create_change(
            &project_id,
            NewChange {
                change_id: ChangeId::from("demo"),
                summary: "test".to_string(),
            },
        )
        .await
        .expect("create_change");
    provider
        .write_artifact(
            &project_id,
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Proposal,
                content: "## Why\n\ntest\n".to_string(),
                capability: None,
            },
        )
        .await
        .expect("write proposal");
    provider
}

#[tokio::test]
async fn write_design_tasks_spec_succeeds() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;

    for (kind, content, capability) in [
        (ArtifactKind::Design, "design body\n", None),
        (ArtifactKind::Tasks, "tasks body\n", None),
        (
            ArtifactKind::Spec,
            "spec body\n",
            Some("user-auth".to_string()),
        ),
    ] {
        provider
            .write_artifact(
                &ProjectId::from("p"),
                &ChangeId::from("demo"),
                NewArtifact {
                    kind,
                    content: content.to_string(),
                    capability,
                },
            )
            .await
            .expect("write");
    }

    let dir = base.join(".speclink/changes/demo");
    assert!(dir.join("design.md").is_file());
    assert!(dir.join("tasks.md").is_file());
    assert!(dir.join("specs/user-auth/spec.md").is_file());
}

#[tokio::test]
async fn metadata_json_unchanged_after_artifact_writes() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;
    let meta_path = base.join(".speclink/changes/demo/metadata.json");
    let before = std::fs::read_to_string(&meta_path).unwrap();

    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Design,
                content: "d\n".to_string(),
                capability: None,
            },
        )
        .await
        .unwrap();
    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Tasks,
                content: "t\n".to_string(),
                capability: None,
            },
        )
        .await
        .unwrap();
    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Spec,
                content: "s\n".to_string(),
                capability: Some("auth".to_string()),
            },
        )
        .await
        .unwrap();

    let after = std::fs::read_to_string(&meta_path).unwrap();
    assert_eq!(
        before, after,
        "metadata.json must not be rewritten by non-proposal artifact writes"
    );
}

#[tokio::test]
async fn directory_layout_only_grows_when_written() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;
    let dir = base.join(".speclink/changes/demo");
    // 初始只應有 proposal.md + metadata.json
    assert!(dir.join("proposal.md").is_file());
    assert!(dir.join("metadata.json").is_file());
    assert!(!dir.join("design.md").exists());
    assert!(!dir.join("tasks.md").exists());
    assert!(!dir.join("specs").exists());

    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Design,
                content: "d\n".to_string(),
                capability: None,
            },
        )
        .await
        .unwrap();
    assert!(dir.join("design.md").is_file());
    assert!(!dir.join("tasks.md").exists());
    assert!(!dir.join("specs").exists());

    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Tasks,
                content: "t\n".to_string(),
                capability: None,
            },
        )
        .await
        .unwrap();
    assert!(dir.join("tasks.md").is_file());
    assert!(!dir.join("specs").exists());

    provider
        .write_artifact(
            &ProjectId::from("p"),
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Spec,
                content: "s\n".to_string(),
                capability: Some("auth".to_string()),
            },
        )
        .await
        .unwrap();
    assert!(dir.join("specs/auth/spec.md").is_file());
}

#[tokio::test]
async fn get_status_proposal_only_returns_three_entries() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;

    let status = provider
        .get_status(&ProjectId::from("p"), &ChangeId::from("demo"))
        .await
        .expect("status");
    assert_eq!(status.artifacts.len(), 3);
    assert_eq!(status.artifacts[0].id, "proposal");
    assert_eq!(status.artifacts[0].status, ArtifactState::Done);
    assert_eq!(status.artifacts[1].status, ArtifactState::Missing);
}

#[tokio::test]
async fn get_status_with_two_specs_returns_sorted() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;
    for cap in ["billing", "auth"] {
        provider
            .write_artifact(
                &ProjectId::from("p"),
                &ChangeId::from("demo"),
                NewArtifact {
                    kind: ArtifactKind::Spec,
                    content: format!("{cap}\n"),
                    capability: Some(cap.to_string()),
                },
            )
            .await
            .unwrap();
    }

    let status = provider
        .get_status(&ProjectId::from("p"), &ChangeId::from("demo"))
        .await
        .expect("status");
    let ids: Vec<&str> = status.artifacts.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["proposal", "design", "tasks", "spec:auth", "spec:billing"],
    );
}

#[tokio::test]
async fn get_status_malformed_metadata_is_internal_error() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    let provider = bootstrap_proposal(base).await;
    // corrupt metadata.json
    let meta = base.join(".speclink/changes/demo/metadata.json");
    std::fs::write(&meta, "{not json").unwrap();
    let err = provider
        .get_status(&ProjectId::from("p"), &ChangeId::from("demo"))
        .await
        .expect_err("err");
    assert!(
        matches!(err, ProviderError::Internal { .. }),
        "expected Internal, got {err:?}"
    );
}

#[tokio::test]
async fn get_status_change_not_found() {
    let tmp = TempDir::new().unwrap();
    let provider = LocalProvider::new(tmp.path().to_path_buf()).await.unwrap();
    let err = provider
        .get_status(&ProjectId::from("p"), &ChangeId::from("missing"))
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
}
