//! `speclink archive <change>` 的 runtime entry point。
//!
//! 對齊 `archive-runner` capability 的 walking-skeleton 路徑：state guard
//! (`in_progress` + `all_tasks_done=1`)、`LocalArchiveStore::archive_change` 委派、
//! warning carrier 組裝（`--skip-specs` 觸發 `archive.specs_skipped` warning）。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use speclink_provider::{
    ArchiveRequest, ArchiveResult, ArchiveStore, ChangeState, MergedSpec, ProviderError, codes,
};
use speclink_provider_local::{LocalArchiveStore, archive_store::collect_capability_names};

use crate::error::{RuntimeError, RuntimeWarning};
use crate::git::GitProbe;
use crate::paths::resolve_state_root;

/// `archive.run` 成功時的 data payload；對應 archive-runner spec「JSON envelope shape」契約。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveData {
    pub change_id: String,
    pub state: ChangeState,
    pub merged_specs: Vec<MergedSpec>,
    pub archived_at: String,
    pub archive_dir: String,
}

/// `ArchiveOperations::run` 回傳：success data + 任意附帶的 warning carrier。
#[derive(Debug, Clone)]
pub struct ArchiveOutcome {
    pub data: ArchiveData,
    pub warnings: Vec<RuntimeWarning>,
}

/// Archive runtime entry。
pub struct ArchiveOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ArchiveOperations<G> {
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalArchiveStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalArchiveStore::new(
            working_dir.to_path_buf(),
            state_root,
        ))
    }

    /// `archive.run <change>`。
    ///
    /// 對 walking-skeleton mode（A3 既有硬編 `require_*_review=false`）：
    /// state guard 接受 `in_progress + all_tasks_done=1`；其他 state reject
    /// `state.transition_invalid`；`in_progress + all_tasks_done=0` reject `change.tasks_incomplete`。
    /// `--skip-specs` 跳過 spec merge 並 append `archive.specs_skipped` warning carrier。
    pub async fn run(
        &self,
        working_dir: &Path,
        change_id: &str,
        skip_specs: bool,
        no_validate: bool,
        yes: bool,
    ) -> Result<ArchiveOutcome, RuntimeError> {
        let store = self.build_store(working_dir)?;

        // For --skip-specs path, collect capability names from CHANGE dir BEFORE archive_change
        // moves it (under archive_dir we can no longer enumerate; alternatively scan after
        // rename — done below for fidelity).
        let req = ArchiveRequest {
            change_id: change_id.to_string(),
            skip_specs,
            no_validate,
            yes,
        };
        let result: ArchiveResult = store
            .archive_change(req)
            .await
            .map_err(map_provider_error)?;

        let mut warnings = Vec::new();
        if skip_specs {
            // After rename: archive_dir 是 working-dir-relative path string
            let abs_archive_dir = working_dir.join(&result.archive_dir);
            let caps = collect_capability_names(&abs_archive_dir).map_err(map_provider_error)?;
            if !caps.is_empty() {
                warnings.push(build_archive_specs_skipped_warning(&caps));
            }
        }

        Ok(ArchiveOutcome {
            data: ArchiveData {
                change_id: result.change_id,
                state: result.state,
                merged_specs: result.merged_specs,
                archived_at: result.archived_at,
                archive_dir: result.archive_dir,
            },
            warnings,
        })
    }
}

/// 組裝 `archive.specs_skipped` warning carrier；對齊 archive-runner spec
/// 「`--skip-specs` SHALL bypass merge while still transitioning state and emit an audit warning」。
fn build_archive_specs_skipped_warning(capabilities_skipped: &[String]) -> RuntimeWarning {
    RuntimeWarning {
        code: codes::ARCHIVE_SPECS_SKIPPED.to_string(),
        message: "Spec delta merge skipped (--skip-specs).".to_string(),
        details: Some(serde_json::json!({
            "capabilities_skipped": capabilities_skipped,
        })),
    }
}

/// `ProviderError → RuntimeError` 對應 archive 路徑。
fn map_provider_error(err: ProviderError) -> RuntimeError {
    match err {
        ProviderError::ChangeNotFound { name } => RuntimeError::ChangeNotFound { name },
        ProviderError::ChangeTasksIncomplete { change_id } => {
            RuntimeError::ChangeTasksIncomplete { change_id }
        }
        ProviderError::ValidationArchiveFailed { reason } => {
            RuntimeError::ValidationArchiveFailed { reason }
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
        ProviderError::ConfigNotFound { path } => RuntimeError::ConfigNotFound { path },
        ProviderError::ConfigMalformed { reason } => RuntimeError::ConfigMalformed { reason },
        ProviderError::ConfigKeyNotFound { key } => RuntimeError::ConfigKeyNotFound {
            key,
            hint: String::new(),
        },
        ProviderError::StateEtagMismatch { expected, actual } => {
            RuntimeError::StateEtagMismatch { expected, actual }
        }
        ProviderError::Internal(s) => RuntimeError::Internal(s),
        other => RuntimeError::Provider(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn archive_specs_skipped_carrier_includes_sorted_capabilities() {
        let w = build_archive_specs_skipped_warning(&[
            "audit-log".to_string(),
            "user-auth".to_string(),
        ]);
        assert_eq!(w.code, "archive.specs_skipped");
        assert_eq!(
            w.details,
            Some(json!({ "capabilities_skipped": ["audit-log", "user-auth"] }))
        );
    }

    #[test]
    fn archive_specs_skipped_carrier_serializes_with_details_key_present() {
        let w = build_archive_specs_skipped_warning(&["a".to_string()]);
        let s = serde_json::to_string(&w).expect("serialize");
        assert!(s.contains("\"details\""));
        assert!(s.contains("\"capabilities_skipped\""));
    }

    #[test]
    fn runtime_warning_without_details_omits_field_in_json() {
        let w = RuntimeWarning {
            code: "x".to_string(),
            message: "y".to_string(),
            details: None,
        };
        let s = serde_json::to_string(&w).expect("serialize");
        assert!(!s.contains("\"details\""), "got: {s}");
    }
}
