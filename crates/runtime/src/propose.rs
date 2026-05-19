//! Runtime 編排：`create_proposal` — propose create 工作流的純編排層。
//!
//! 此模組不直接接觸 filesystem 或 SQLite；它接受任意 [`Arc<dyn Provider>`] 並依序
//! 呼叫 `create_change` → `write_artifact`，輸入驗證在最前面完成。

use provider::Provider;
use provider::error::ProviderError;
use provider::model::{ArtifactKind, ChangeId, NewArtifact, NewChange, ProjectId};
use std::sync::Arc;
use thiserror::Error;

/// Summary 上限長度。
pub const MAX_SUMMARY_LEN: usize = 200;

/// Runtime 層錯誤型別。
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// 下層 Provider 失敗。
    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),

    /// 使用者輸入不合法。
    #[error("invalid input: {reason}")]
    InvalidInput {
        /// 失敗原因。
        reason: String,
    },
}

impl RuntimeError {
    /// 對應 CLI 層 error code。
    pub fn error_code(&self) -> &'static str {
        match self {
            RuntimeError::Provider(p) => p.error_code(),
            RuntimeError::InvalidInput { .. } => "input.invalid",
        }
    }
}

/// `create_proposal` 的輸入。
#[derive(Debug, Clone)]
pub struct CreateProposalInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// Change 摘要；上限 [`MAX_SUMMARY_LEN`] 字元，且不可為空字串。
    pub summary: String,
}

/// `create_proposal` 的輸出。
#[derive(Debug, Clone)]
pub struct CreateProposalOutput {
    /// 已建立 change 的識別碼。
    pub change_id: ChangeId,
    /// 寫入的 proposal artifact 路徑（POSIX 風格、相對於 base）。
    pub artifact_path: String,
}

/// 編排：先呼叫 `create_change`，成功後呼叫 `write_artifact`。
///
/// 輸入驗證（空字串 / 超長 summary）在呼叫 Provider 前完成；失敗則 Provider 不會被觸碰。
pub async fn create_proposal(
    provider: Arc<dyn Provider>,
    input: CreateProposalInput,
) -> Result<CreateProposalOutput, RuntimeError> {
    if input.summary.is_empty() {
        return Err(RuntimeError::InvalidInput {
            reason: "summary is empty".to_string(),
        });
    }
    if input.summary.chars().count() > MAX_SUMMARY_LEN {
        return Err(RuntimeError::InvalidInput {
            reason: format!("summary exceeds maximum length of {MAX_SUMMARY_LEN} characters"),
        });
    }

    let new_change = NewChange {
        change_id: input.change_id.clone(),
        summary: input.summary.clone(),
    };
    let _change = provider
        .create_change(&input.project_id, new_change)
        .await?;

    let content = format!("## Why\n\n{}\n", input.summary);
    let new_artifact = NewArtifact {
        kind: ArtifactKind::Proposal,
        content,
    };
    let artifact = provider
        .write_artifact(&input.project_id, &input.change_id, new_artifact)
        .await?;

    Ok(CreateProposalOutput {
        change_id: input.change_id,
        artifact_path: artifact.path,
    })
}

#[cfg(test)]
mod tests {
    use crate::propose::{CreateProposalInput, RuntimeError, create_proposal};
    use async_trait::async_trait;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        Artifact, ArtifactKind, Change, ChangeId, CreatedBy, NewArtifact, NewChange, ProjectId,
        State,
    };
    use std::sync::Arc;
    use std::sync::Mutex;

    /// 紀錄被呼叫的 method，用於驗證順序。
    #[derive(Debug, Default)]
    struct CallLog {
        events: Vec<String>,
    }

    /// Mock provider：可配置每個 method 是 OK 還是 Err。
    struct MockProvider {
        log: Mutex<CallLog>,
        create_err: Option<&'static str>,
        write_err: Option<&'static str>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                log: Mutex::new(CallLog::default()),
                create_err: None,
                write_err: None,
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn create_change(
            &self,
            _project_id: &ProjectId,
            input: NewChange,
        ) -> Result<Change, ProviderError> {
            self.log
                .lock()
                .unwrap()
                .events
                .push("create_change".to_string());
            if let Some(msg) = self.create_err {
                return Err(ProviderError::Internal {
                    message: msg.to_string(),
                });
            }
            Ok(Change {
                change_id: input.change_id.clone(),
                state: State::Proposed,
                created_at: "2026-05-19T12:00:00Z".to_string(),
                created_by: CreatedBy {
                    kind: "agent".to_string(),
                    name: String::new(),
                },
            })
        }

        async fn write_artifact(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _input: NewArtifact,
        ) -> Result<Artifact, ProviderError> {
            self.log
                .lock()
                .unwrap()
                .events
                .push("write_artifact".to_string());
            if let Some(msg) = self.write_err {
                return Err(ProviderError::Internal {
                    message: msg.to_string(),
                });
            }
            Ok(Artifact {
                kind: ArtifactKind::Proposal,
                path: ".speclink/changes/demo/proposal.md".to_string(),
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
    }

    fn input() -> CreateProposalInput {
        CreateProposalInput {
            project_id: ProjectId::from("p"),
            change_id: ChangeId::from("demo"),
            summary: "test summary".to_string(),
        }
    }

    #[tokio::test]
    async fn happy_path_calls_create_then_write() {
        let mock = Arc::new(MockProvider::new());
        let provider: Arc<dyn Provider> = mock.clone();
        let _ = create_proposal(provider, input()).await.expect("ok");
        let events = mock.log.lock().unwrap().events.clone();
        assert_eq!(events, vec!["create_change", "write_artifact"]);
    }

    #[tokio::test]
    async fn create_change_failure_skips_write_artifact() {
        let mut mock = MockProvider::new();
        mock.create_err = Some("boom");
        let mock = Arc::new(mock);
        let provider: Arc<dyn Provider> = mock.clone();
        let err = create_proposal(provider, input()).await.expect_err("err");
        assert!(matches!(err, RuntimeError::Provider(_)));
        let events = mock.log.lock().unwrap().events.clone();
        assert_eq!(events, vec!["create_change"]);
    }

    #[tokio::test]
    async fn write_artifact_failure_returns_provider_error() {
        let mut mock = MockProvider::new();
        mock.write_err = Some("io");
        let mock = Arc::new(mock);
        let provider: Arc<dyn Provider> = mock.clone();
        let err = create_proposal(provider, input()).await.expect_err("err");
        assert!(matches!(err, RuntimeError::Provider(_)));
        let events = mock.log.lock().unwrap().events.clone();
        assert_eq!(events, vec!["create_change", "write_artifact"]);
    }

    #[tokio::test]
    async fn empty_summary_rejected_before_provider_calls() {
        let mut i = input();
        i.summary = String::new();
        let mock = Arc::new(MockProvider::new());
        let provider: Arc<dyn Provider> = mock.clone();
        let err = create_proposal(provider, i).await.expect_err("err");
        assert!(
            matches!(err, RuntimeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
        let events = mock.log.lock().unwrap().events.clone();
        assert!(events.is_empty(), "provider must not be called");
    }

    #[tokio::test]
    async fn over_long_summary_rejected_before_provider_calls() {
        let mut i = input();
        i.summary = "x".repeat(201);
        let mock = Arc::new(MockProvider::new());
        let provider: Arc<dyn Provider> = mock.clone();
        let err = create_proposal(provider, i).await.expect_err("err");
        assert!(
            matches!(err, RuntimeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
        let events = mock.log.lock().unwrap().events.clone();
        assert!(events.is_empty(), "provider must not be called");
    }
}
