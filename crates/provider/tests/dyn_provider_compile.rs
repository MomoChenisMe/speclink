//! 編譯時測試：`Provider` trait 必須 dyn-compatible，且 `Arc<dyn Provider>` 可跨 thread 傳遞。
//!
//! 任何此處的 type-check 失敗都會以紅燈的方式由 cargo 回報。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use provider::Provider;
use provider::error::ProviderError;
use provider::model::{
    ArchiveOptions, ArchivedChange, Artifact, ArtifactInstructions, ArtifactKind, Change, ChangeId,
    ChangeStatus, InstructionRule, NewArtifact, NewChange, ProjectId, RuleLevel, SpecDeltaSummary,
    State, TaskStatus, TaskUpdate,
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

    async fn get_artifact_instructions(
        &self,
        _project_id: &ProjectId,
        _change_id: &ChangeId,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<ArtifactInstructions, ProviderError> {
        let artifact_id = match (kind, capability) {
            (ArtifactKind::Spec, Some(cap)) => format!("spec:{cap}"),
            (ArtifactKind::Proposal, _) => "proposal".to_string(),
            (ArtifactKind::Design, _) => "design".to_string(),
            (ArtifactKind::Tasks, _) => "tasks".to_string(),
            (ArtifactKind::Spec, None) => return Err(ProviderError::MissingCapability),
        };
        Ok(ArtifactInstructions {
            artifact_id,
            kind,
            output_path: ".speclink/changes/demo/<file>".to_string(),
            dependencies: Vec::new(),
            unlocks: Vec::new(),
            instruction: "stub".to_string(),
            template: "## Heading\n".to_string(),
            rules: vec![InstructionRule {
                id: "stub.rule".to_string(),
                level: RuleLevel::Error,
                description: "stub".to_string(),
            }],
            locale: "Traditional Chinese (繁體中文)".to_string(),
        })
    }

    async fn mark_task_done(
        &self,
        _project_id: &ProjectId,
        _change_id: &ChangeId,
        task_id: &str,
    ) -> Result<TaskUpdate, ProviderError> {
        Ok(TaskUpdate {
            task_id: task_id.to_string(),
            previous_status: TaskStatus::Todo,
            current_status: TaskStatus::Done,
            task_description: "stub".to_string(),
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
