//! `speclink new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <etag>] --stdin`。

#![allow(clippy::doc_markdown)]

use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use speclink_provider::{ArtifactKind, Etag, ExpectedEtag};
use speclink_runtime::{ArtifactOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <etag>] --stdin`。
///
/// # Errors
/// `ArtifactKindInvalid` / `ArtifactCapabilityRequired` / `ArtifactNotFound` /
/// `ArtifactVersionConflict` / `ChangeNotFound` / `Internal`。
pub async fn run(
    working_dir: &Path,
    kind_str: &str,
    change: &str,
    capability: Option<&str>,
    expected_etag: Option<&str>,
    stdin_flag: bool,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let kind = ArtifactKind::parse(kind_str).ok_or_else(|| RuntimeError::ArtifactKindInvalid {
        kind: kind_str.to_string(),
    })?;

    let expected = match expected_etag {
        None => ExpectedEtag::None,
        Some(s) => ExpectedEtag::Some(Etag::from_str(s).map_err(|e| {
            RuntimeError::ArtifactKindInvalid {
                kind: format!("invalid --expected-etag: {e}"),
            }
        })?),
    };

    let bytes = if stdin_flag {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| RuntimeError::Internal(format!("read stdin: {e}")))?;
        buf
    } else {
        Vec::new()
    };

    let ops = ArtifactOperations::new(RealGitProbe);
    let v = ops
        .write_artifact(working_dir, change, kind, capability, &bytes, expected)
        .await?;

    let path = artifact_envelope_path(kind, change, capability);

    let data = serde_json::json!({
        "kind": kind.as_str(),
        "capability": capability,
        "path": path,
        "etag": v.etag.as_str(),
        "bytesWritten": bytes.len(),
    });

    let mut warnings = Vec::new();
    if capability.is_some() && !kind.requires_capability() {
        warnings.push(Warning {
            code: "artifact.capability_ignored".to_string(),
            message: "`--capability` is only meaningful when kind=spec".to_string(),
        });
    }

    Ok((data, warnings))
}

fn artifact_envelope_path(kind: ArtifactKind, change: &str, capability: Option<&str>) -> String {
    match kind {
        ArtifactKind::Proposal => format!("changes/{change}/proposal.md"),
        ArtifactKind::Design => format!("changes/{change}/design.md"),
        ArtifactKind::Tasks => format!("changes/{change}/tasks.md"),
        ArtifactKind::Spec => format!(
            "changes/{change}/specs/{}/spec.md",
            capability.unwrap_or("")
        ),
    }
}
