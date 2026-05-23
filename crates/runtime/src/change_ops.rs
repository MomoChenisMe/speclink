//! `speclink new/list/show/delete change` 的 runtime entry points.
//!
//! 沿用 bootstrap `ops.rs::Operations<G>` 的「struct + GitProbe 泛型 + build_store helper」pattern；
//! 對 `LocalChangeStore` 包一層，加上 name validation 與 destructive confirm-name 校對。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use speclink_provider::{ChangeRow, ChangeStore, ProviderError, validate_kebab_id};
use speclink_provider_local::LocalChangeStore;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::{ARTIFACT_ROOT, resolve_state_root};

/// 預設 schema id（slice A 沒有 schema CLI 之前的 placeholder）。
pub const DEFAULT_SCHEMA_ID: &str = "spec-driven";

/// `change-store` 觀察到的單一 artifact 參照（用於 `speclink show change` 輸出）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRef {
    pub kind: String,
    pub capability: Option<String>,
}

/// `speclink show change` 回傳的完整 data shape。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowChangeData {
    pub change: ChangeRow,
    pub artifacts: Vec<ArtifactRef>,
}

/// Change CRUD 的 entry。
pub struct ChangeOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ChangeOperations<G> {
    /// 建立 handle。不接觸 disk。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalChangeStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalChangeStore::new(working_dir.to_path_buf(), state_root))
    }

    /// 建立新 change。
    ///
    /// # Errors
    /// `ChangeInvalidName` / `ChangeDuplicateName` / `RequiresGit` / `Internal`。
    pub async fn create_change(
        &self,
        working_dir: &Path,
        name: &str,
    ) -> Result<ChangeRow, RuntimeError> {
        if let Err(e) = validate_kebab_id(name) {
            return Err(RuntimeError::ChangeInvalidName {
                name: name.to_string(),
                reason: e.to_string(),
            });
        }
        let store = self.build_store(working_dir)?;
        store
            .create_change(name, DEFAULT_SCHEMA_ID)
            .await
            .map_err(map_provider_error_change)
    }

    /// 列舉所有 change。
    pub async fn list_changes(&self, working_dir: &Path) -> Result<Vec<ChangeRow>, RuntimeError> {
        let store = self.build_store(working_dir)?;
        store
            .list_changes()
            .await
            .map_err(map_provider_error_change)
    }

    /// 顯示單一 change metadata + 該 change 下既有 artifact 清單。
    pub async fn show_change(
        &self,
        working_dir: &Path,
        name: &str,
    ) -> Result<ShowChangeData, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let change = store
            .get_change(name)
            .await
            .map_err(map_provider_error_change)?;
        let dir = change_dir(working_dir, name);
        let artifacts = discover_artifacts(&dir)?;
        Ok(ShowChangeData { change, artifacts })
    }

    /// 刪除 change row + filesystem 目錄。
    ///
    /// `--confirm-name` 必須與 `name` 完全相符；否則回 `ChangeInvalidName`。
    pub async fn delete_change(
        &self,
        working_dir: &Path,
        name: &str,
        confirm_name: Option<&str>,
    ) -> Result<(), RuntimeError> {
        match confirm_name {
            Some(c) if c == name => {}
            Some(c) => {
                return Err(RuntimeError::ChangeInvalidName {
                    name: name.to_string(),
                    reason: format!("`--confirm-name {c}` does not match target name `{name}`"),
                });
            }
            None => {
                return Err(RuntimeError::ChangeInvalidName {
                    name: name.to_string(),
                    reason: "destructive delete requires `--confirm-name <name>`".into(),
                });
            }
        }
        let store = self.build_store(working_dir)?;
        store
            .delete_change(name)
            .await
            .map_err(map_provider_error_change)
    }
}

fn change_dir(working_dir: &Path, name: &str) -> PathBuf {
    working_dir.join(ARTIFACT_ROOT).join("changes").join(name)
}

fn discover_artifacts(change_dir: &Path) -> Result<Vec<ArtifactRef>, RuntimeError> {
    let mut out = Vec::new();
    for (kind, file) in [
        ("proposal", "proposal.md"),
        ("design", "design.md"),
        ("tasks", "tasks.md"),
    ] {
        if change_dir.join(file).is_file() {
            out.push(ArtifactRef {
                kind: kind.to_string(),
                capability: None,
            });
        }
    }
    let specs_dir = change_dir.join("specs");
    if specs_dir.is_dir() {
        let mut caps: Vec<String> = Vec::new();
        let entries = fs::read_dir(&specs_dir).map_err(|e| {
            RuntimeError::Internal(format!("read_dir {}: {e}", specs_dir.display()))
        })?;
        for entry in entries {
            let entry =
                entry.map_err(|e| RuntimeError::Internal(format!("read_dir entry: {e}")))?;
            let ft = entry
                .file_type()
                .map_err(|e| RuntimeError::Internal(format!("entry file_type: {e}")))?;
            if !ft.is_dir() {
                continue;
            }
            if entry.path().join("spec.md").is_file() {
                if let Ok(name) = entry.file_name().into_string() {
                    caps.push(name);
                }
            }
        }
        caps.sort();
        for cap in caps {
            out.push(ArtifactRef {
                kind: "spec".to_string(),
                capability: Some(cap),
            });
        }
    }
    Ok(out)
}

/// `ProviderError → RuntimeError` exhaustive mapping for change ops.
fn map_provider_error_change(err: ProviderError) -> RuntimeError {
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
