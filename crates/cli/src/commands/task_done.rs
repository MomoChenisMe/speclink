//! `speclink task done <index> --change <id>`。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{RealGitProbe, RuntimeError, TaskOperations};

use crate::output::Warning;

/// 執行 `task done`。
///
/// # Errors
/// `ChangeNotFound` / `TaskNoTasksFile` / `TaskIndexOutOfRange` /
/// `StateTransitionInvalid` / `StateVersionConflict` / `Internal`。
pub async fn run(
    working_dir: &Path,
    change: &str,
    index: usize,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = TaskOperations::new(RealGitProbe);
    let (data, runtime_warnings) = ops.done(working_dir, change, index).await?;
    let payload = serde_json::to_value(&data).map_err(|e| RuntimeError::Internal(e.to_string()))?;
    let warnings = runtime_warnings
        .into_iter()
        .map(|w| Warning {
            code: w.code,
            message: w.message,
            details: w.details,
        })
        .collect();
    Ok((payload, warnings))
}
