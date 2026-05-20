//! `LocalProvider::get_artifact_instructions` 整合測試 — 對 4 種 kind 驗證
//! 非空 instruction / template / rules、output_path 為預期 POSIX 路徑、locale 一致。

use provider::Provider;
use provider::error::ProviderError;
use provider::model::{ArtifactKind, ChangeId, NewArtifact, NewChange, ProjectId};
use provider_local::LocalProvider;
use std::path::Path;
use tempfile::TempDir;

const PROJECT: &str = "default";
const LOCALE: &str = "Traditional Chinese (繁體中文)";

async fn bootstrap(base: &Path, change_id: &str) {
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
async fn instructions_proposal_returns_non_empty_content() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let ai = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Proposal,
            None,
        )
        .await
        .expect("ok");
    assert_eq!(ai.artifact_id, "proposal");
    assert_eq!(ai.kind, ArtifactKind::Proposal);
    assert_eq!(ai.output_path, ".speclink/changes/demo/proposal.md");
    assert!(ai.dependencies.is_empty());
    assert_eq!(ai.unlocks, vec!["design", "tasks", "spec"]);
    assert!(!ai.instruction.is_empty());
    assert!(!ai.template.is_empty());
    assert!(!ai.rules.is_empty());
    assert_eq!(ai.locale, LOCALE);
}

#[tokio::test]
async fn instructions_design_returns_proposal_dependency() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let ai = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Design,
            None,
        )
        .await
        .expect("ok");
    assert_eq!(ai.artifact_id, "design");
    assert_eq!(ai.dependencies, vec!["proposal"]);
    assert_eq!(ai.unlocks, vec!["tasks"]);
    assert_eq!(ai.output_path, ".speclink/changes/demo/design.md");
}

#[tokio::test]
async fn instructions_tasks_returns_proposal_and_spec_dependency() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let ai = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Tasks,
            None,
        )
        .await
        .expect("ok");
    assert_eq!(ai.artifact_id, "tasks");
    assert_eq!(ai.dependencies, vec!["proposal", "spec"]);
    assert!(ai.unlocks.is_empty());
}

#[tokio::test]
async fn instructions_spec_with_capability() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let ai = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Spec,
            Some("user-auth"),
        )
        .await
        .expect("ok");
    assert_eq!(ai.artifact_id, "spec:user-auth");
    assert_eq!(
        ai.output_path,
        ".speclink/changes/demo/specs/user-auth/spec.md"
    );
    assert_eq!(ai.dependencies, vec!["proposal"]);
    assert_eq!(ai.unlocks, vec!["tasks"]);
    assert!(!ai.rules.is_empty());
    assert_eq!(ai.locale, LOCALE);
}

#[tokio::test]
async fn instructions_spec_missing_capability_returns_error() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let err = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Spec,
            None,
        )
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::MissingCapability));
}

#[tokio::test]
async fn instructions_design_with_capability_returns_error() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let err = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Design,
            Some("user-auth"),
        )
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::Internal { .. }));
}

#[tokio::test]
async fn instructions_invalid_capability_returns_error() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    bootstrap(base, "demo").await;
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let err = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("demo"),
            ArtifactKind::Spec,
            Some("Bad-Name"),
        )
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::InvalidCapability { .. }));
}

#[tokio::test]
async fn instructions_change_not_found() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();
    // 沒 bootstrap
    let provider = LocalProvider::new(base.to_path_buf()).await.unwrap();
    let err = provider
        .get_artifact_instructions(
            &ProjectId::from(PROJECT),
            &ChangeId::from("missing"),
            ArtifactKind::Design,
            None,
        )
        .await
        .expect_err("err");
    assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
}
