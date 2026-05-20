//! Runtime 編排：`write_artifact` — artifact write 工作流的純編排層。
//!
//! 雙重校驗：clap layer 已先擋 kind / capability 組合錯誤；本層作為防禦深度的第二道
//! 把關，避免直接以 library 呼叫時誤用 trait。

use provider::Provider;
use provider::model::{ArtifactKind, ChangeId, NewArtifact, ProjectId};
use std::sync::Arc;

use crate::propose::RuntimeError;

/// `write_artifact` 的輸入。
#[derive(Debug, Clone)]
pub struct WriteArtifactInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// Artifact 種類。
    pub kind: ArtifactKind,
    /// 文字內容（已含 trailing newline）。
    pub content: String,
    /// Capability 名稱；僅當 `kind == Spec` 時提供。
    pub capability: Option<String>,
}

/// `write_artifact` 的輸出。
#[derive(Debug, Clone)]
pub struct WriteArtifactOutput {
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// Artifact 識別碼：`"proposal"` / `"design"` / `"tasks"` 或 `"spec:<capability>"`。
    pub artifact_id: String,
    /// Artifact 種類。
    pub kind: ArtifactKind,
    /// 寫入的 POSIX 相對路徑。
    pub path: String,
}

/// 編排：驗證 kind / capability 組合 → 呼叫 `provider.write_artifact`。
///
/// 失敗條件：
/// - `kind == Spec` 但 `capability == None`、或非 spec kind 帶 capability、或 content 為空 →
///   [`RuntimeError::InvalidInput`]
pub async fn write_artifact(
    provider: Arc<dyn Provider>,
    input: WriteArtifactInput,
) -> Result<WriteArtifactOutput, RuntimeError> {
    match (input.kind, input.capability.as_deref()) {
        (ArtifactKind::Spec, None) => {
            return Err(RuntimeError::InvalidInput {
                reason: "spec artifact requires --capability".to_string(),
            });
        }
        (ArtifactKind::Proposal | ArtifactKind::Design | ArtifactKind::Tasks, Some(_)) => {
            return Err(RuntimeError::InvalidInput {
                reason: "--capability only valid for spec artifacts".to_string(),
            });
        }
        _ => {}
    }
    if input.content.is_empty() {
        return Err(RuntimeError::InvalidInput {
            reason: "stdin content must not be empty".to_string(),
        });
    }

    let new_artifact = NewArtifact {
        kind: input.kind,
        content: input.content.clone(),
        capability: input.capability.clone(),
    };
    let artifact = provider
        .write_artifact(&input.project_id, &input.change_id, new_artifact)
        .await
        .map_err(RuntimeError::Provider)?;

    let artifact_id = match (input.kind, input.capability.as_deref()) {
        (ArtifactKind::Spec, Some(cap)) => format!("spec:{cap}"),
        (ArtifactKind::Proposal, _) => "proposal".to_string(),
        (ArtifactKind::Design, _) => "design".to_string(),
        (ArtifactKind::Tasks, _) => "tasks".to_string(),
        (ArtifactKind::Spec, None) => unreachable!("checked above"),
    };

    Ok(WriteArtifactOutput {
        change_id: input.change_id,
        artifact_id,
        kind: input.kind,
        path: artifact.path,
    })
}

#[cfg(test)]
mod tests {
    use crate::artifact::{WriteArtifactInput, write_artifact};
    use crate::propose::RuntimeError;
    use async_trait::async_trait;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        ArchiveOptions, ArchivedChange, Artifact, ArtifactKind, Change, ChangeId, ChangeStatus,
        NewArtifact, NewChange, ProjectId,
    };
    use std::sync::Arc;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockProvider {
        calls: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn create_change(
            &self,
            _project_id: &ProjectId,
            _input: NewChange,
        ) -> Result<Change, ProviderError> {
            unimplemented!()
        }

        async fn write_artifact(
            &self,
            _project_id: &ProjectId,
            change_id: &ChangeId,
            input: NewArtifact,
        ) -> Result<Artifact, ProviderError> {
            self.calls.lock().unwrap().push(format!(
                "{:?}:{}",
                input.kind,
                input.capability.unwrap_or_default()
            ));
            let path = match input.kind {
                ArtifactKind::Proposal => {
                    format!(".speclink/changes/{}/proposal.md", change_id.as_str())
                }
                ArtifactKind::Design => {
                    format!(".speclink/changes/{}/design.md", change_id.as_str())
                }
                ArtifactKind::Tasks => {
                    format!(".speclink/changes/{}/tasks.md", change_id.as_str())
                }
                ArtifactKind::Spec => {
                    format!(".speclink/changes/{}/specs/X/spec.md", change_id.as_str())
                }
            };
            Ok(Artifact {
                kind: input.kind,
                path,
                content_hash: String::new(),
            })
        }

        async fn get_change(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
        ) -> Result<Change, ProviderError> {
            unimplemented!()
        }

        async fn get_status(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
        ) -> Result<ChangeStatus, ProviderError> {
            unimplemented!()
        }

        async fn archive_change(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _options: ArchiveOptions,
        ) -> Result<ArchivedChange, ProviderError> {
            unimplemented!()
        }

        async fn get_artifact_instructions(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _kind: ArtifactKind,
            _capability: Option<&str>,
        ) -> Result<provider::model::ArtifactInstructions, ProviderError> {
            unimplemented!()
        }

        async fn mark_task_done(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _task_id: &str,
        ) -> Result<provider::model::TaskUpdate, ProviderError> {
            unimplemented!()
        }
    }

    fn input(kind: ArtifactKind, capability: Option<&str>) -> WriteArtifactInput {
        WriteArtifactInput {
            project_id: ProjectId::from("p"),
            change_id: ChangeId::from("demo"),
            kind,
            content: "body\n".to_string(),
            capability: capability.map(|s| s.to_string()),
        }
    }

    #[tokio::test]
    async fn design_happy_path() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let out = write_artifact(mock, input(ArtifactKind::Design, None))
            .await
            .unwrap();
        assert_eq!(out.artifact_id, "design");
        assert_eq!(out.kind, ArtifactKind::Design);
    }

    #[tokio::test]
    async fn tasks_happy_path() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let out = write_artifact(mock, input(ArtifactKind::Tasks, None))
            .await
            .unwrap();
        assert_eq!(out.artifact_id, "tasks");
    }

    #[tokio::test]
    async fn spec_happy_path_produces_namespaced_id() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let out = write_artifact(mock, input(ArtifactKind::Spec, Some("user-auth")))
            .await
            .unwrap();
        assert_eq!(out.artifact_id, "spec:user-auth");
    }

    #[tokio::test]
    async fn spec_without_capability_is_invalid_input() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let err = write_artifact(mock, input(ArtifactKind::Spec, None))
            .await
            .expect_err("err");
        assert!(matches!(err, RuntimeError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn design_with_capability_is_invalid_input() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let err = write_artifact(mock, input(ArtifactKind::Design, Some("auth")))
            .await
            .expect_err("err");
        assert!(matches!(err, RuntimeError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn empty_content_is_invalid_input() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let mut i = input(ArtifactKind::Design, None);
        i.content = String::new();
        let err = write_artifact(mock, i).await.expect_err("err");
        assert!(matches!(err, RuntimeError::InvalidInput { .. }));
    }
}
