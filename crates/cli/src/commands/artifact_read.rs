//! `speclink artifact read <kind> --change <name> [--capability <cap>]` subcommand.

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_provider::ArtifactKind;
use speclink_runtime::{ArtifactOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `artifact read <kind> --change <name> [--capability <cap>]`。
///
/// # Errors
/// `ArtifactKindInvalid` / `ArtifactNotFound` / `ChangeNotFound` /
/// `ArtifactCapabilityRequired` / `RequiresGit` / `Internal`。
pub async fn run(
    working_dir: &Path,
    kind_str: &str,
    change: &str,
    capability: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let kind = ArtifactKind::parse(kind_str).ok_or_else(|| RuntimeError::ArtifactKindInvalid {
        kind: kind_str.to_string(),
    })?;
    let ops = ArtifactOperations::new(RealGitProbe);
    let v = ops
        .read_artifact(working_dir, change, kind, capability)
        .await?;

    let path = match kind {
        ArtifactKind::Proposal => format!("changes/{change}/proposal.md"),
        ArtifactKind::Design => format!("changes/{change}/design.md"),
        ArtifactKind::Tasks => format!("changes/{change}/tasks.md"),
        ArtifactKind::Spec => format!(
            "changes/{change}/specs/{}/spec.md",
            capability.unwrap_or("")
        ),
    };

    let content = String::from_utf8(v.value)
        .map_err(|e| RuntimeError::Internal(format!("artifact body is not valid UTF-8: {e}")))?;

    let data = serde_json::json!({
        "kind": kind.as_str(),
        "capability": capability,
        "path": path,
        "content": content,
        "etag": v.etag.as_str(),
    });
    Ok((data, Vec::new()))
}
