//! `speclink instructions <kind> [--change <id>] [--role <r>] [--discussion <d>]` subcommand.
//!
//! 對應 `instructions-resolver` capability 與 `doc/protocol/operations.md`
//! §`instructions.get`。CLI thin layer：clap parse → 呼 `InstructionsOperations`
//! → 把 11-field envelope serialize 成 JSON。
//!
//! `--role` / `--discussion` flags 接受但忽略（reserved for Phase 2
//! `add-discuss-ops`）；runtime 端永遠忽略；help text 標示 (reserved for Phase 2)。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::{InstructionsInput, InstructionsOperations, RealGitProbe, RuntimeError};

use crate::output::Warning;

/// 執行 `instructions <kind>`。
///
/// # Errors
/// `InstructionsUnknownKind`（exit 2）/ `ChangeNotFound`（exit 2）/ `RequiresGit` /
/// `Internal`。
pub async fn run(
    working_dir: &Path,
    kind: &str,
    change: Option<&str>,
    role: Option<&str>,
    discussion: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = InstructionsOperations::new(RealGitProbe);
    let input = InstructionsInput {
        kind: kind.to_string(),
        change_id: change.map(str::to_string),
        // 接受但忽略；runtime 端不變動 envelope（spec scenario「--role is accepted but ignored」）。
        role: role.map(str::to_string),
        discussion_id: discussion.map(str::to_string),
    };
    let (output, runtime_warnings) = ops.get_instructions(working_dir, input).await?;
    let payload =
        serde_json::to_value(&output).map_err(|e| RuntimeError::Internal(e.to_string()))?;
    let warnings = runtime_warnings
        .into_iter()
        .map(|w| Warning {
            code: w.code,
            message: w.message,
            details: w.details,
        })
        .collect();
    Ok((payload, warnings))
}
