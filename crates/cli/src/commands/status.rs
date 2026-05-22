//! `speclink status` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Operations, RealGitProbe, RuntimeError};

pub async fn run(working_dir: &Path) -> Result<serde_json::Value, RuntimeError> {
    let ops = Operations::new(RealGitProbe);
    let status = ops.status(working_dir).await?;
    serde_json::to_value(status).map_err(|e| RuntimeError::Internal(format!("serialize: {e}")))
}
