//! `speclink link` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Operations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `link <project_id>`。
///
/// # Errors
/// `RequiresGit` / `LinkTargetNotFound` / `Internal`。
pub async fn run(
    working_dir: &Path,
    project_id: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = Operations::new(RealGitProbe);
    let info = ops.link(working_dir, project_id).await?;
    let data = serde_json::json!({
        "project_id": info.project_id,
        "artifact_root": info.artifact_root,
        "state_root": info.state_root,
    });
    Ok((data, Vec::new()))
}
