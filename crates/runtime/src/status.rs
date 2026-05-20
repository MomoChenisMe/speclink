//! Runtime 編排：`get_status` — `speclink status` 工作流的純編排層。
//!
//! 純轉發：side-effect-free read，呼叫 `provider.get_status` 後直接回傳結果。

use provider::Provider;
use provider::model::{ChangeId, ChangeStatus, ProjectId};
use std::sync::Arc;

use crate::propose::RuntimeError;

/// `get_status` 的輸入。
#[derive(Debug, Clone)]
pub struct GetStatusInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// Change 識別碼。
    pub change_id: ChangeId,
}

/// 編排：純轉發 `provider.get_status`。
pub async fn get_status(
    provider: Arc<dyn Provider>,
    input: GetStatusInput,
) -> Result<ChangeStatus, RuntimeError> {
    provider
        .get_status(&input.project_id, &input.change_id)
        .await
        .map_err(RuntimeError::Provider)
}

#[cfg(test)]
mod tests {
    use crate::propose::RuntimeError;
    use crate::status::{GetStatusInput, get_status};
    use async_trait::async_trait;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        Artifact, ArtifactKind, ArtifactState, ArtifactStatus, Change, ChangeId, ChangeStatus,
        NewArtifact, NewChange, ProjectId, State,
    };
    use std::sync::Arc;

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
            change_id: &ChangeId,
        ) -> Result<ChangeStatus, ProviderError> {
            if self.not_found {
                return Err(ProviderError::ChangeNotFound {
                    change_id: change_id.clone(),
                });
            }
            Ok(ChangeStatus {
                change_id: change_id.clone(),
                state: State::Proposed,
                artifacts: vec![ArtifactStatus {
                    id: "proposal".to_string(),
                    kind: ArtifactKind::Proposal,
                    path: format!(".speclink/changes/{}/proposal.md", change_id.as_str()),
                    status: ArtifactState::Done,
                    required: false,
                    dependencies: Vec::new(),
                }],
            })
        }
    }

    #[tokio::test]
    async fn happy_path_returns_change_status() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider { not_found: false });
        let out = get_status(
            mock,
            GetStatusInput {
                project_id: ProjectId::from("p"),
                change_id: ChangeId::from("demo"),
            },
        )
        .await
        .expect("ok");
        assert_eq!(out.change_id.as_str(), "demo");
        assert_eq!(out.artifacts.len(), 1);
    }

    #[tokio::test]
    async fn change_not_found_propagates_provider_error() {
        let mock: Arc<dyn Provider> = Arc::new(MockProvider { not_found: true });
        let err = get_status(
            mock,
            GetStatusInput {
                project_id: ProjectId::from("p"),
                change_id: ChangeId::from("missing"),
            },
        )
        .await
        .expect_err("err");
        assert!(matches!(
            err,
            RuntimeError::Provider(ProviderError::ChangeNotFound { .. })
        ));
    }
}
