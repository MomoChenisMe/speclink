//! `speclink new change <name>` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ARTIFACT_ROOT, ChangeOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `new change <name>`。
///
/// # Errors
/// `ChangeInvalidName` / `ChangeDuplicateName` / `RequiresGit` / `Internal`。
pub async fn run(
    working_dir: &Path,
    name: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ChangeOperations::new(RealGitProbe);
    let row = ops.create_change(working_dir, name).await?;
    let artifact_dir = format!("{ARTIFACT_ROOT}/changes/{name}");
    let data = serde_json::json!({
        "changeId": row.change_id,
        "name": row.name,
        "state": row.state,
        "version": row.version,
        "schemaId": row.schema_id,
        "artifactDir": artifact_dir,
        "createdAt": row.created_at,
    });
    Ok((data, Vec::new()))
}
