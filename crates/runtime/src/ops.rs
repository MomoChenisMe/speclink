//! `speclink status` / `link` / `unlink` 的 runtime entry points.
//!
//! 對 LocalProjectStore 包一層，把 git_head 與 state root path 補進 ProjectStatus。

#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};

use speclink_provider::{ProjectInfo, ProjectStatus, ProjectStore, ProviderError};
use speclink_provider_local::LocalProjectStore;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::{ARTIFACT_ROOT, STATE_ROOT_NAMESPACE, display_state_root};

/// Status / link / unlink 的 entry。
pub struct Operations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> Operations<G> {
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalProjectStore, RuntimeError> {
        let state_root = self.state_root(working_dir)?;
        Ok(LocalProjectStore::new(
            working_dir.to_path_buf(),
            state_root,
        ))
    }

    fn state_root(&self, working_dir: &Path) -> Result<PathBuf, RuntimeError> {
        let common = self.git.common_dir(working_dir)?;
        Ok(common.join(STATE_ROOT_NAMESPACE))
    }

    /// 回傳已 init project 的 status；未 init 回 `NotInitialized`。
    ///
    /// # Errors
    /// `RequiresGit` / `NotInitialized` / `Internal`。
    pub async fn status(&self, working_dir: &Path) -> Result<ProjectStatus, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let link = store
            .get_link()
            .await?
            .ok_or_else(|| RuntimeError::NotInitialized {
                path: working_dir
                    .join(ARTIFACT_ROOT)
                    .join("link.yaml")
                    .display()
                    .to_string(),
            })?;
        let git_head = self.git.head_sha(working_dir)?;
        let state_root = self.state_root(working_dir)?;
        Ok(ProjectStatus {
            project_id: link.project_id,
            provider: link.provider,
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: display_state_root(working_dir, &state_root),
            git_head,
            requires_git: true,
        })
    }

    /// 把當前 working dir 綁定到既存 project_id。
    ///
    /// # Errors
    /// `RequiresGit` / `LinkTargetNotFound` / `Internal`。
    pub async fn link(
        &self,
        working_dir: &Path,
        project_id: &str,
    ) -> Result<ProjectInfo, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let info = store.link(project_id).await.map_err(map_provider_error)?;
        let state_root = self.state_root(working_dir)?;
        Ok(ProjectInfo {
            project_id: info.project_id,
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: display_state_root(working_dir, &state_root),
        })
    }

    /// 移除 link.yaml；不刪 state.db、不刪 schemas。
    ///
    /// # Errors
    /// `Internal` (filesystem)。
    pub async fn unlink(&self, working_dir: &Path) -> Result<(), RuntimeError> {
        let store = self.build_store(working_dir)?;
        store.unlink().await?;
        Ok(())
    }
}

/// 把 provider 層的 declared error 折回 runtime 層對應 variant，保留具體 code。
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
        ProviderError::Internal(s) => RuntimeError::Internal(s),
    }
}
