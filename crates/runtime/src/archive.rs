//! Runtime 編排：`archive` — `speclink archive` 工作流的純編排層。
//!
//! 純轉發：將 `ArchiveOptions` 與 change id 傳給 `provider.archive_change`，policy 與
//! filesystem rollback 邏輯皆在 provider-local 內部處理；runtime 不在此層做額外校驗。

use provider::Provider;
use provider::model::{ArchiveOptions, ArchivedChange, ChangeId, ProjectId};
use std::sync::Arc;

use crate::propose::RuntimeError;

/// `archive` 的輸入。
#[derive(Debug, Clone)]
pub struct ArchiveInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// 目標 change 識別碼。
    pub change_id: ChangeId,
    /// archive 呼叫選項；`archive_date` 由 caller（CLI）注入。
    pub options: ArchiveOptions,
}

/// 編排：直接呼叫 `provider.archive_change` 並回傳結果。
pub async fn archive(
    provider: Arc<dyn Provider>,
    input: ArchiveInput,
) -> Result<ArchivedChange, RuntimeError> {
    provider
        .archive_change(&input.project_id, &input.change_id, input.options)
        .await
        .map_err(RuntimeError::Provider)
}

#[cfg(test)]
mod tests {
    use crate::archive::{ArchiveInput, archive};
    use crate::propose::RuntimeError;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        ArchiveOptions, ArchivedChange, Artifact, Change, ChangeId, ChangeStatus, NewArtifact,
        NewChange, ProjectId, SpecDeltaSummary, State,
    };
    use std::sync::Arc;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockProvider {
        calls: Mutex<Vec<String>>,
        force_err: bool,
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
            change_id: &ChangeId,
            options: ArchiveOptions,
        ) -> Result<ArchivedChange, ProviderError> {
            self.calls.lock().unwrap().push(format!(
                "archive:{}:{}",
                change_id.as_str(),
                options.dry_run
            ));
            if self.force_err {
                return Err(ProviderError::ChangeNotArchivable {
                    reason: "already archived".to_string(),
                });
            }
            Ok(ArchivedChange {
                change_id: change_id.clone(),
                archive_path: format!(
                    ".speclink/changes/archive/{}-{}",
                    options.archive_date.format("%Y-%m-%d"),
                    change_id.as_str()
                ),
                state: State::Archived,
                archived_at: "2026-05-19T12:00:00Z".to_string(),
                spec_sync: SpecDeltaSummary {
                    capabilities_synced: Vec::new(),
                },
                dry_run: options.dry_run,
            })
        }

        async fn get_artifact_instructions(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _kind: provider::model::ArtifactKind,
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

    fn input(dry_run: bool) -> ArchiveInput {
        ArchiveInput {
            project_id: ProjectId::from("p"),
            change_id: ChangeId::from("demo"),
            options: ArchiveOptions {
                dry_run,
                archive_date: NaiveDate::from_ymd_opt(2026, 5, 19).unwrap(),
            },
        }
    }

    #[tokio::test]
    async fn happy_path_forwards_options() {
        let mock = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = mock.clone();
        let out = archive(provider, input(false)).await.expect("ok");
        assert_eq!(out.change_id.as_str(), "demo");
        assert!(!out.dry_run);
        assert_eq!(
            out.archive_path,
            ".speclink/changes/archive/2026-05-19-demo"
        );
        assert_eq!(out.state, State::Archived);
        assert_eq!(
            mock.calls.lock().unwrap().as_slice(),
            &["archive:demo:false"]
        );
    }

    #[tokio::test]
    async fn dry_run_flag_forwarded() {
        let mock = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = mock.clone();
        let out = archive(provider, input(true)).await.expect("ok");
        assert!(out.dry_run);
        assert_eq!(
            mock.calls.lock().unwrap().as_slice(),
            &["archive:demo:true"]
        );
    }

    #[tokio::test]
    async fn provider_error_propagates_as_runtime_error() {
        let mock = Arc::new(MockProvider {
            calls: Mutex::new(Vec::new()),
            force_err: true,
        });
        let provider: Arc<dyn Provider> = mock.clone();
        let err = archive(provider, input(false)).await.expect_err("err");
        assert!(matches!(
            err,
            RuntimeError::Provider(ProviderError::ChangeNotArchivable { .. })
        ));
    }
}
