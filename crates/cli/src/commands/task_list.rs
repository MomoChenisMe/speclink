//! `speclink task list --change <id>`。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{RealGitProbe, RuntimeError, TaskOperations};

use crate::output::Warning;

/// 執行 `task list`。
///
/// # Errors
/// `TaskNoTasksFile` / `Internal`。
pub async fn run(
    working_dir: &Path,
    change: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = TaskOperations::new(RealGitProbe);
    let data = ops.list(working_dir, change)?;
    let payload = serde_json::to_value(&data).map_err(|e| RuntimeError::Internal(e.to_string()))?;
    Ok((payload, Vec::new()))
}
