//! `speclink apply pause <change-id>`。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ApplyOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `apply pause`。
///
/// # Errors
/// `ChangeNotFound` / `StateTransitionInvalid` / `StateVersionConflict` / `Internal`。
pub async fn run(
    working_dir: &Path,
    change: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ApplyOperations::new(RealGitProbe);
    let data = ops.pause(working_dir, change).await?;
    let payload = serde_json::to_value(&data).map_err(|e| RuntimeError::Internal(e.to_string()))?;
    Ok((payload, Vec::new()))
}
