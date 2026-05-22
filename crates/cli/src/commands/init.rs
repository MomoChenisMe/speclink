//! `speclink init` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{Bootstrap, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 init；回 JSON 化的 data payload + warnings。
///
/// # Errors
/// 回 [`RuntimeError`] 的任何 variant（含 `RequiresGit` / `AlreadyInitialized` / `Internal`）。
pub async fn run(
    working_dir: &Path,
    force: bool,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let boot = Bootstrap::new(RealGitProbe);
    let info = boot.init(working_dir, force).await?;
    let data = serde_json::json!({
        "project_id": info.project_id,
        "artifact_root": info.artifact_root,
        "state_root": info.state_root,
    });
    Ok((data, Vec::new()))
}
