//! `speclink list --specs --change <name>` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ArtifactOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `list --specs --change <name>`。
///
/// # Errors
/// `ChangeNotFound` / `RequiresGit` / `Internal`。
pub async fn run(
    working_dir: &Path,
    change: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ArtifactOperations::new(RealGitProbe);
    let caps = ops.list_spec_capabilities(working_dir, change).await?;
    Ok((
        serde_json::json!({ "change": change, "capabilities": caps }),
        Vec::new(),
    ))
}
