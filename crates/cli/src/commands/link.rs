//! `speclink link` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Operations, RealGitProbe, RuntimeError};

pub async fn run(working_dir: &Path, project_id: &str) -> Result<serde_json::Value, RuntimeError> {
    let ops = Operations::new(RealGitProbe);
    let info = ops.link(working_dir, project_id).await?;
    Ok(serde_json::json!({
        "project_id": info.project_id,
        "artifact_root": info.artifact_root,
        "state_root": info.state_root,
    }))
}
