//! `speclink apply start` / `speclink apply pause` 的 runtime entry points。
//!
//! 對齊 `apply-task-ops` capability 的 ensure-actor semantics 與 symmetric idempotency
//! 契約。所有 transition 透過 `LocalStateMachineStore` 的 CAS 路徑寫入。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use speclink_provider::{
    Actor, ChangeState, ChangeStateView, ProviderError, StateMachineStore, StateTransitionReason,
    TransitionRequest,
};
use speclink_provider_local::LocalStateMachineStore;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::resolve_state_root;
use crate::state_machine::resolve_actor;

/// `apply.start` 成功時的 data payload。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyStartData {
    pub change_id: String,
    pub state: ChangeState,
    pub actor: Option<Actor>,
    /// 對 `code_reviewing` / `archived` 等不轉移的 state 帶 hint 訊息；其餘為 None。
    pub message: Option<String>,
}

/// `apply.pause` 成功時的 data payload。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyPauseData {
    pub change_id: String,
    pub state: ChangeState,
    pub actor: Option<Actor>,
    pub message: Option<String>,
}

/// Apply / pause runtime entry。
pub struct ApplyOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ApplyOperations<G> {
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalStateMachineStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalStateMachineStore::new(state_root))
    }

    /// `apply.start change [--actor agent_host]`。
    ///
    /// 對 `ready` transition `in_progress` 並 assign actor；對 `in_progress` 只 reassign
    /// actor（不寫 audit）；對 `code_reviewing` / `archived` 回 hint 不轉移；對 `proposing`
    /// / `reviewing` 回 `state.transition_invalid`。
    pub async fn start(
        &self,
        working_dir: &Path,
        change: &str,
        explicit_agent_host: Option<&str>,
    ) -> Result<ApplyStartData, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let view = store
            .get_change_state(change)
            .await
            .map_err(map_provider_error)?;
        let new_actor = resolve_actor(explicit_agent_host);

        match view.state {
            ChangeState::Ready => {
                let new_view = store
                    .transition_state(
                        change,
                        view.version,
                        TransitionRequest {
                            to_state: ChangeState::InProgress,
                            actor: Some(Some(new_actor.clone())),
                            reason: StateTransitionReason::ApplyStart,
                        },
                    )
                    .await
                    .map_err(map_provider_error)?;
                Ok(start_payload(new_view, None))
            }
            ChangeState::InProgress => {
                // ensure-actor：reassign actor without audit insert.
                let new_view = store
                    .set_actor(change, view.version, Some(new_actor.clone()))
                    .await
                    .map_err(map_provider_error)?;
                Ok(start_payload(new_view, None))
            }
            ChangeState::CodeReviewing => Ok(start_payload(
                view,
                Some("Already in code review; nothing to apply.".to_string()),
            )),
            ChangeState::Archived => {
                Ok(start_payload(view, Some("Change is archived.".to_string())))
            }
            other @ (ChangeState::Proposing | ChangeState::Reviewing) => {
                Err(RuntimeError::StateTransitionInvalid {
                    from: other.as_str().to_string(),
                    to: ChangeState::InProgress.as_str().to_string(),
                })
            }
        }
    }

    /// `apply.pause change`。對 `in_progress` transition `ready` + clear actor；對 `ready`
    /// 為 idempotent no-op；其他 state 回 `state.transition_invalid`。
    pub async fn pause(
        &self,
        working_dir: &Path,
        change: &str,
    ) -> Result<ApplyPauseData, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let view = store
            .get_change_state(change)
            .await
            .map_err(map_provider_error)?;
        match view.state {
            ChangeState::InProgress => {
                let new_view = store
                    .transition_state(
                        change,
                        view.version,
                        TransitionRequest {
                            to_state: ChangeState::Ready,
                            actor: Some(None),
                            reason: StateTransitionReason::ApplyPause,
                        },
                    )
                    .await
                    .map_err(map_provider_error)?;
                Ok(pause_payload(new_view, None))
            }
            ChangeState::Ready => Ok(pause_payload(
                view,
                Some("Change is already paused.".to_string()),
            )),
            other => Err(RuntimeError::StateTransitionInvalid {
                from: other.as_str().to_string(),
                to: ChangeState::Ready.as_str().to_string(),
            }),
        }
    }
}

fn start_payload(view: ChangeStateView, message: Option<String>) -> ApplyStartData {
    ApplyStartData {
        change_id: view.change_id,
        state: view.state,
        actor: view.actor,
        message,
    }
}

fn pause_payload(view: ChangeStateView, message: Option<String>) -> ApplyPauseData {
    ApplyPauseData {
        change_id: view.change_id,
        state: view.state,
        actor: view.actor,
        message,
    }
}

/// `ProviderError → RuntimeError` exhaustive mapping for apply ops.
fn map_provider_error(err: ProviderError) -> RuntimeError {
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
        ProviderError::StateInvalidValue { value } => RuntimeError::StateInvalidValue { value },
        ProviderError::StateTransitionInvalid { from, to } => {
            RuntimeError::StateTransitionInvalid { from, to }
        }
        ProviderError::StateVersionConflict { current_version } => {
            RuntimeError::StateVersionConflict { current_version }
        }
        ProviderError::StateDbSchemaInvalid { found, supported } => {
            RuntimeError::StateDbSchemaInvalid { found, supported }
        }
        ProviderError::ChangeDagIncomplete { missing } => {
            RuntimeError::ChangeDagIncomplete { missing }
        }
        ProviderError::ChangeTasksIncomplete { change_id } => {
            RuntimeError::ChangeTasksIncomplete { change_id }
        }
        ProviderError::ValidationArchiveFailed { reason } => {
            RuntimeError::ValidationArchiveFailed { reason }
        }
        ProviderError::ConfigNotFound { path } => RuntimeError::ConfigNotFound { path },
        ProviderError::ConfigMalformed { reason } => RuntimeError::ConfigMalformed { reason },
        ProviderError::ConfigKeyNotFound { key } => RuntimeError::ConfigKeyNotFound {
            key,
            hint: String::new(),
        },
        ProviderError::StateEtagMismatch { expected, actual } => {
            RuntimeError::StateEtagMismatch { expected, actual }
        }
        ProviderError::ConfigEditModeRequired => RuntimeError::ConfigEditModeRequired,
        ProviderError::Internal(s) => RuntimeError::Internal(s),
    }
}
