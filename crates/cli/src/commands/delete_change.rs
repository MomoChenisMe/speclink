//! `speclink delete change <name> --confirm-name <name>` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ChangeOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `delete change <name> --confirm-name <name>`（destructive）。
///
/// # Errors
/// `ChangeInvalidName`（缺/錯 confirm）/ `ChangeNotFound` / `RequiresGit` / `Internal`。
pub async fn run(
    working_dir: &Path,
    name: &str,
    confirm_name: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ChangeOperations::new(RealGitProbe);
    ops.delete_change(working_dir, name, confirm_name).await?;
    Ok((
        serde_json::json!({ "name": name, "deleted": true }),
        Vec::new(),
    ))
}
