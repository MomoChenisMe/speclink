//! Integration test：`LocalProvider::create_change` + `LocalProvider::write_artifact`
//! 串接後檔案系統與 SQLite state 必須完整。

use provider::Provider;
use provider::model::{ArtifactKind, ChangeId, NewArtifact, NewChange, ProjectId};
use provider_local::LocalProvider;
use tempfile::TempDir;

#[tokio::test]
async fn create_change_then_write_artifact_succeeds() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let provider = LocalProvider::new(base.clone()).await.expect("new");
    let project_id = ProjectId::from("p");
    let change_id = ChangeId::from("demo");
    let summary = "test summary";

    // 1) create_change
    let change = provider
        .create_change(
            &project_id,
            NewChange {
                change_id: change_id.clone(),
                summary: summary.to_string(),
            },
        )
        .await
        .expect("create_change");
    assert_eq!(change.change_id.as_str(), "demo");

    // 2) write_artifact (proposal)
    let proposal_content = format!("## Why\n\n{summary}\n");
    let artifact = provider
        .write_artifact(
            &project_id,
            &change_id,
            NewArtifact {
                kind: ArtifactKind::Proposal,
                content: proposal_content.clone(),
            },
        )
        .await
        .expect("write_artifact");
    assert_eq!(artifact.kind, ArtifactKind::Proposal);
    assert!(
        artifact
            .path
            .ends_with(".speclink/changes/demo/proposal.md")
    );

    // 3) Verify proposal.md content
    let body = std::fs::read_to_string(base.join(".speclink/changes/demo/proposal.md")).unwrap();
    assert_eq!(body, "## Why\n\ntest summary\n");

    // 4) Verify metadata.json
    let meta = std::fs::read_to_string(base.join(".speclink/changes/demo/metadata.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&meta).expect("parse json");
    assert_eq!(v.get("state").and_then(|v| v.as_str()), Some("proposed"));

    // 5) Verify SQLite state db has the row (open a raw connection)
    let conn = rusqlite::Connection::open(base.join(".speclink/state.db")).unwrap();
    let id: String = conn
        .query_row("SELECT change_id FROM in_progress_change", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(id, "demo");
}

#[tokio::test]
async fn duplicate_change_id_yields_change_already_exists() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let provider = LocalProvider::new(base.clone()).await.expect("new");
    let project_id = ProjectId::from("p");

    // 第一次 propose create
    provider
        .create_change(
            &project_id,
            NewChange {
                change_id: ChangeId::from("demo"),
                summary: "first".to_string(),
            },
        )
        .await
        .expect("first create_change");
    provider
        .write_artifact(
            &project_id,
            &ChangeId::from("demo"),
            NewArtifact {
                kind: ArtifactKind::Proposal,
                content: "## Why\n\nfirst\n".to_string(),
            },
        )
        .await
        .expect("first write_artifact");

    // 第二次 with same id：create_change should fail
    let err = provider
        .create_change(
            &project_id,
            NewChange {
                change_id: ChangeId::from("demo"),
                summary: "second".to_string(),
            },
        )
        .await
        .expect_err("duplicate");
    use provider::error::ProviderError;
    assert!(
        matches!(err, ProviderError::ChangeAlreadyExists { .. }),
        "expected ChangeAlreadyExists, got {err:?}"
    );

    // 既有檔案未被覆寫
    let body = std::fs::read_to_string(base.join(".speclink/changes/demo/proposal.md")).unwrap();
    assert_eq!(body, "## Why\n\nfirst\n");
}

#[tokio::test]
async fn invalid_change_id_yields_invalid_change_id_error() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let provider = LocalProvider::new(base).await.expect("new");
    let project_id = ProjectId::from("p");

    let err = provider
        .create_change(
            &project_id,
            NewChange {
                change_id: ChangeId::from("Add-Feature"),
                summary: "x".to_string(),
            },
        )
        .await
        .expect_err("must error");
    use provider::error::ProviderError;
    assert!(
        matches!(err, ProviderError::InvalidChangeId { .. }),
        "expected InvalidChangeId, got {err:?}"
    );
}
