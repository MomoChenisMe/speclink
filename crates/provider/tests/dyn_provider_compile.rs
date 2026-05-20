//! 編譯時測試：`Provider` trait 必須 dyn-compatible，且 `Arc<dyn Provider>` 可跨 thread 傳遞。
//!
//! 任何此處的 type-check 失敗都會以紅燈的方式由 cargo 回報。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use provider::Provider;
use provider::error::ProviderError;
use provider::model::{
    ArchiveOptions, ArchivedChange, Artifact, Change, ChangeId, ChangeStatus, NewArtifact,
    NewChange, ProjectId, SpecDeltaSummary, State,
};

/// In-memory mock provider，內含 `Mutex<HashMap>`：以 `Send + Sync` 包裝可變狀態。
struct MockProvider {
    changes: Mutex<HashMap<String, Change>>,
}

#[async_trait]
impl Provider for MockProvider {
    async fn create_change(
        &self,
        _project_id: &ProjectId,
        _input: NewChange,
    ) -> Result<Change, ProviderError> {
        drop(self.changes.lock());
        unimplemented!("mock — only used to confirm compilation")
    }

    async fn write_artifact(
        &self,
        _project_id: &ProjectId,
        _change_id: &ChangeId,
        _input: NewArtifact,
    ) -> Result<Artifact, ProviderError> {
        unimplemented!("mock — only used to confirm compilation")
    }

    async fn get_change(
        &self,
        _project_id: &ProjectId,
        _change_id: &ChangeId,
    ) -> Result<Change, ProviderError> {
        unimplemented!("mock — only used to confirm compilation")
    }

    async fn get_status(
        &self,
        _project_id: &ProjectId,
        _change_id: &ChangeId,
    ) -> Result<ChangeStatus, ProviderError> {
        unimplemented!("mock — only used to confirm compilation")
    }

    async fn archive_change(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
        options: ArchiveOptions,
    ) -> Result<ArchivedChange, ProviderError> {
        Ok(ArchivedChange {
            change_id: change_id.clone(),
            archive_path: format!(
                ".speclink/changes/archive/{}-{}",
                options.archive_date.format("%Y-%m-%d"),
                change_id.as_str()
            ),
            state: State::Archived,
            archived_at: "2026-05-19T00:00:00Z".to_string(),
            spec_sync: SpecDeltaSummary {
                capabilities_synced: Vec::new(),
            },
            dry_run: options.dry_run,
        })
    }
}

fn accept(_p: Arc<dyn Provider>) {}

#[test]
fn arc_dyn_provider_is_constructible_and_send_sync() {
    let mock: Box<MockProvider> = Box::new(MockProvider {
        changes: Mutex::new(HashMap::new()),
    });
    let dynamic: Arc<dyn Provider> = Arc::from(mock as Box<dyn Provider>);
    accept(dynamic);
}
