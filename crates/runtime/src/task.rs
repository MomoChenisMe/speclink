//! Runtime 編排：`mark_task_done` — `speclink task done` 工作流的純編排層。
//!
//! 純轉發：將 task id 傳給 `provider.mark_task_done`。在 runtime 層補一道 task id
//! 格式防禦（雖然 clap 已擋），確保 trait 直接呼叫端（非 CLI）仍受保護。

use provider::Provider;
use provider::model::{ChangeId, ProjectId, TaskUpdate};
use std::sync::Arc;

use crate::propose::RuntimeError;
use crate::tasks_parser::is_valid_task_id;

/// `mark_task_done` 的輸入。
#[derive(Debug, Clone)]
pub struct MarkTaskDoneInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// 目標 task id（`N.M` 格式）。
    pub task_id: String,
}

/// 純轉發 `provider.mark_task_done`；在前端先做 task id 格式校驗。
pub async fn mark_task_done(
    provider: Arc<dyn Provider>,
    input: MarkTaskDoneInput,
) -> Result<TaskUpdate, RuntimeError> {
    if !is_valid_task_id(&input.task_id) {
        return Err(RuntimeError::Provider(
            provider::error::ProviderError::TaskInvalidId {
                task_id: input.task_id.clone(),
            },
        ));
    }
    provider
        .mark_task_done(&input.project_id, &input.change_id, &input.task_id)
        .await
        .map_err(RuntimeError::Provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        ArchiveOptions, ArchivedChange, Artifact, ArtifactInstructions, ArtifactKind, Change,
        ChangeStatus, NewArtifact, NewChange, TaskStatus,
    };
    use std::sync::Arc;

    #[derive(Default)]
    struct MockProvider {
        not_found: bool,
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
            _change_id: &ChangeId,
            _input: NewArtifact,
        ) -> Result<Artifact, ProviderError> {
            unimplemented!()
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
        ) -> Result<ArtifactInstructions, ProviderError> {
            unimplemented!()
        }
        async fn mark_task_done(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            task_id: &str,
        ) -> Result<TaskUpdate, ProviderError> {
            if self.not_found {
                return Err(ProviderError::TaskNotFound {
                    task_id: task_id.to_string(),
                });
            }
            Ok(TaskUpdate {
                task_id: task_id.to_string(),
                previous_status: TaskStatus::Todo,
                current_status: TaskStatus::Done,
                task_description: "First".to_string(),
            })
        }
    }

    fn input(task_id: &str) -> MarkTaskDoneInput {
        MarkTaskDoneInput {
            project_id: ProjectId::from("p"),
            change_id: ChangeId::from("demo"),
            task_id: task_id.to_string(),
        }
    }

    #[tokio::test]
    async fn happy_path_forwards_task_id() {
        let provider: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let out = mark_task_done(provider, input("1.1")).await.expect("ok");
        assert_eq!(out.task_id, "1.1");
        assert_eq!(out.current_status, TaskStatus::Done);
    }

    #[tokio::test]
    async fn invalid_task_id_short_circuits() {
        let provider: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let err = mark_task_done(provider, input("1.1.2"))
            .await
            .expect_err("err");
        assert!(matches!(
            err,
            RuntimeError::Provider(ProviderError::TaskInvalidId { .. })
        ));
    }

    #[tokio::test]
    async fn provider_not_found_propagates() {
        let provider: Arc<dyn Provider> = Arc::new(MockProvider { not_found: true });
        let err = mark_task_done(provider, input("1.99"))
            .await
            .expect_err("err");
        assert!(matches!(
            err,
            RuntimeError::Provider(ProviderError::TaskNotFound { .. })
        ));
    }
}
