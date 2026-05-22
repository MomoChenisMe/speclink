//! `speclink archive <change-id> [--skip-specs] [--yes] [--no-validate]`。
//!
//! 對應 archive-runner spec「`speclink archive` SHALL transition the change from
//! `in_progress` to `archived` when all tasks are done」與「JSON envelope SHALL
//! conform to the bootstrap / A2 / A3 contract」。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ArchiveOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `archive`。
///
/// # Errors
/// `ChangeNotFound` / `StateTransitionInvalid` / `ChangeTasksIncomplete` /
/// `StateVersionConflict` / `Internal`。
pub async fn run(
    working_dir: &Path,
    change_id: &str,
    skip_specs: bool,
    no_validate: bool,
    yes: bool,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ArchiveOperations::new(RealGitProbe);
    let outcome = ops
        .run(working_dir, change_id, skip_specs, no_validate, yes)
        .await?;
    let payload =
        serde_json::to_value(&outcome.data).map_err(|e| RuntimeError::Internal(e.to_string()))?;
    let warnings = outcome
        .warnings
        .into_iter()
        .map(|w| Warning {
            code: w.code,
            message: w.message,
            details: w.details,
        })
        .collect();
    Ok((payload, warnings))
}
