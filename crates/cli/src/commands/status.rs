//! `speclink status` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Operations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 status。
///
/// # Errors
/// `RequiresGit` / `NotInitialized` / `Internal`。
pub async fn run(working_dir: &Path) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = Operations::new(RealGitProbe);
    let status = ops.status(working_dir).await?;
    let data = serde_json::to_value(status)
        .map_err(|e| RuntimeError::Internal(format!("serialize: {e}")))?;
    Ok((data, Vec::new()))
}
