//! `speclink artifact read` / `new artifact` / `list --specs` 的 runtime entry points.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_provider::{
    ArtifactKind, ArtifactStore, ChangeState, ExpectedEtag, ProviderError, StateMachineStore,
    StateTransitionReason, TransitionRequest, Versioned,
};
use speclink_provider_local::{LocalArtifactStore, LocalStateMachineStore};

use crate::error::{RuntimeError, RuntimeWarning};
use crate::git::GitProbe;
use crate::paths::{artifact_root, resolve_state_root};
use crate::state_machine::{ReviewPolicy, proposing_target};

/// Artifact I/O 的 entry。
pub struct ArtifactOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ArtifactOperations<G> {
    /// 建立 handle。不接觸 disk。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalArtifactStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalArtifactStore::new(
            working_dir.to_path_buf(),
            state_root,
        ))
    }

    fn build_state_store(
        &self,
        working_dir: &Path,
    ) -> Result<LocalStateMachineStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalStateMachineStore::new(state_root))
    }

    /// 讀取 artifact。
    pub async fn read_artifact(
        &self,
        working_dir: &Path,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<Versioned<Vec<u8>>, RuntimeError> {
        let store = self.build_store(working_dir)?;
        store
            .read_artifact(change, kind, capability)
            .await
            .map_err(map_provider_error_artifact)
    }

    /// 寫入 artifact，回傳 etag + 可能的 `state_transitioned` warning。
    ///
    /// Hook 行為對齊 `state-machine` capability「Forward state transitions from
    /// `proposing` SHALL be triggered automatically by the `artifact.write` DAG
    /// evaluator」：成功 atomic rename 之後檢查 DAG completeness；齊全且 state 為
    /// `proposing` 時透過 walking-skeleton path 推進到 `ready`，並追加 warning。
    pub async fn write_artifact(
        &self,
        working_dir: &Path,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
        bytes: &[u8],
        expected: ExpectedEtag,
    ) -> Result<(Versioned<()>, Vec<RuntimeWarning>), RuntimeError> {
        let store = self.build_store(working_dir)?;
        let versioned = store
            .write_artifact(change, kind, capability, bytes, expected)
            .await
            .map_err(map_provider_error_artifact)?;

        let mut warnings = Vec::new();
        // Best-effort post-write DAG evaluator hook. Failure here SHALL NOT roll back
        // the artifact write — bubble warning if transition succeeds.
        if let Some(w) = self.dag_evaluator(working_dir, change).await? {
            warnings.push(w);
        }
        Ok((versioned, warnings))
    }

    /// DAG evaluator: 若 change state == `proposing` 且 DAG (proposal.md + tasks.md
    /// + 至少一份 specs/*) 齊全，呼叫 `transition_state` 推進並回 warning。
    async fn dag_evaluator(
        &self,
        working_dir: &Path,
        change: &str,
    ) -> Result<Option<RuntimeWarning>, RuntimeError> {
        let sm = self.build_state_store(working_dir)?;
        let view = sm
            .get_change_state(change)
            .await
            .map_err(map_provider_error_artifact)?;
        if view.state != ChangeState::Proposing {
            return Ok(None);
        }
        if !dag_complete(working_dir, change) {
            return Ok(None);
        }
        let policy = ReviewPolicy::walking_skeleton();
        let target = proposing_target(policy);
        let from = view.state;
        let new_view = sm
            .transition_state(
                change,
                view.version,
                TransitionRequest {
                    to_state: target,
                    actor: None,
                    reason: StateTransitionReason::ArtifactDagComplete,
                },
            )
            .await
            .map_err(map_provider_error_artifact)?;
        Ok(Some(RuntimeWarning {
            code: "state_transitioned".to_string(),
            message: format!("Change state advanced to {}", new_view.state.as_str()),
            details: Some(serde_json::json!({
                "from": from.as_str(),
                "to": new_view.state.as_str(),
                "reason": StateTransitionReason::ArtifactDagComplete.as_str(),
            })),
        }))
    }

    /// 列舉某 change 下所有 spec capability id。
    pub async fn list_spec_capabilities(
        &self,
        working_dir: &Path,
        change: &str,
    ) -> Result<Vec<String>, RuntimeError> {
        let store = self.build_store(working_dir)?;
        store
            .list_spec_capabilities(change)
            .await
            .map_err(map_provider_error_artifact)
    }
}

/// DAG 完整性檢查：proposal.md + tasks.md + 至少一份 specs/*/spec.md。
fn dag_complete(working_dir: &Path, change: &str) -> bool {
    let dir = artifact_root(working_dir).join("changes").join(change);
    let proposal = dir.join("proposal.md").is_file();
    let tasks = dir.join("tasks.md").is_file();
    let specs_dir = dir.join("specs");
    let has_spec = match std::fs::read_dir(&specs_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .any(|e| e.path().join("spec.md").is_file()),
        Err(_) => false,
    };
    proposal && tasks && has_spec
}

/// `ProviderError → RuntimeError` exhaustive mapping for artifact ops.
///
/// Slice A3 引入的 `State*` / `Change*` variant 從 artifact 路徑視為 internal error
/// （artifact write 觸發 auto-transition 才會經過這些 code，apply_ops 與 task_ops 自行處理）。
fn map_provider_error_artifact(err: ProviderError) -> RuntimeError {
    match err {
        ProviderError::RequiresGit { context } => RuntimeError::RequiresGit { context },
        ProviderError::AlreadyInitialized { path } => RuntimeError::AlreadyInitialized { path },
        ProviderError::NotInitialized { path } => RuntimeError::NotInitialized { path },
        ProviderError::LinkTargetNotFound { project_id } => {
            RuntimeError::LinkTargetNotFound { project_id }
        }
        ProviderError::ChangeNotFound { name } => RuntimeError::ChangeNotFound { name },
        ProviderError::ChangeDuplicateName { name } => RuntimeError::ChangeDuplicateName { name },
        ProviderError::ChangeInvalidName { name, reason } => {
            RuntimeError::ChangeInvalidName { name, reason }
        }
        ProviderError::ArtifactKindInvalid { kind } => RuntimeError::ArtifactKindInvalid { kind },
        ProviderError::ArtifactCapabilityRequired => RuntimeError::ArtifactCapabilityRequired,
        ProviderError::ArtifactNotFound { path } => RuntimeError::ArtifactNotFound { path },
        ProviderError::ArtifactVersionConflict { expected, actual } => {
            RuntimeError::ArtifactVersionConflict { expected, actual }
        }
        e @ (ProviderError::StateInvalidValue { .. }
        | ProviderError::StateTransitionInvalid { .. }
        | ProviderError::StateVersionConflict { .. }
        | ProviderError::StateDbSchemaInvalid { .. }
        | ProviderError::ChangeDagIncomplete { .. }) => RuntimeError::Provider(e),
        ProviderError::Internal(s) => RuntimeError::Internal(s),
    }
}
