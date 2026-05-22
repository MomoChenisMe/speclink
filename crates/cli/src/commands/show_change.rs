//! `speclink show change <name>` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{ChangeOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `show change <name>`。
///
/// # Errors
/// `ChangeNotFound` / `RequiresGit` / `Internal`。
pub async fn run(
    working_dir: &Path,
    name: &str,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ChangeOperations::new(RealGitProbe);
    let show = ops.show_change(working_dir, name).await?;
    let data = serde_json::json!({
        "change": {
            "changeId": show.change.change_id,
            "name": show.change.name,
            "state": show.change.state,
            "version": show.change.version,
            "schemaId": show.change.schema_id,
            "createdAt": show.change.created_at,
            "updatedAt": show.change.updated_at,
        },
        "artifacts": show.artifacts.iter().map(|a| serde_json::json!({
            "kind": a.kind,
            "capability": a.capability,
        })).collect::<Vec<_>>(),
    });
    Ok((data, Vec::new()))
}
