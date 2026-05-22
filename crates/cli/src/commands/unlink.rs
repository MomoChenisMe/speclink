//! `speclink unlink` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Operations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 unlink。
///
/// # Errors
/// `Internal`（filesystem 失敗）。
pub async fn run(working_dir: &Path) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = Operations::new(RealGitProbe);
    ops.unlink(working_dir).await?;
    Ok((serde_json::json!({}), Vec::new()))
}
