//! `speclink artifact read` / `new artifact` / `list --specs` 的 runtime entry points.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_provider::{ArtifactKind, ArtifactStore, ExpectedEtag, ProviderError, Versioned};
use speclink_provider_local::LocalArtifactStore;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::resolve_state_root;

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

    /// 寫入 artifact。
    pub async fn write_artifact(
        &self,
        working_dir: &Path,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
        bytes: &[u8],
        expected: ExpectedEtag,
    ) -> Result<Versioned<()>, RuntimeError> {
        let store = self.build_store(working_dir)?;
        store
            .write_artifact(change, kind, capability, bytes, expected)
            .await
            .map_err(map_provider_error_artifact)
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

/// `ProviderError → RuntimeError` exhaustive mapping for artifact ops.
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
        ProviderError::Internal(s) => RuntimeError::Internal(s),
    }
}
