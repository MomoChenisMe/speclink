//! `speclink list --changes` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ChangeOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `list --changes`。
///
/// # Errors
/// `RequiresGit` / `Internal`。
pub async fn run(working_dir: &Path) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ChangeOperations::new(RealGitProbe);
    let rows = ops.list_changes(working_dir).await?;
    let changes: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "changeId": r.change_id,
                "name": r.name,
                "state": r.state,
                "version": r.version,
                "schemaId": r.schema_id,
                "createdAt": r.created_at,
                "updatedAt": r.updated_at,
            })
        })
        .collect();
    Ok((serde_json::json!({ "changes": changes }), Vec::new()))
}
