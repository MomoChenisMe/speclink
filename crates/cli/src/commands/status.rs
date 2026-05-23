//! `speclink status` subcommand.
//!
//! 對齊 `project.status` op（operations.md §1360）— 印 envelope
//! `{provider_type, project_id, working_dir, current_change?, changes_count,
//! discussions_count, schema_active}` 七欄位。
//!
//! Human mode 走既有 `speclink_cli::human::render_human()` YAML pretty-printer
//! （cli-human-output spec），不做 dashboard table renderer。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_runtime::RuntimeError;
use speclink_runtime::project_ops::project_status;

use crate::output::Warning;

/// 執行 `project.status` op 並回傳 envelope.data。
///
/// # Errors
/// - [`RuntimeError::NotInitialized`]：working_dir 不在 SpecLink 專案內（無 link.yaml）→ exit 2
/// - [`RuntimeError::RequiresGit`]：working_dir 不在 git working tree
/// - [`RuntimeError::Internal`]：state.db 讀取失敗或內部錯誤
pub async fn run(working_dir: &Path) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let status = project_status(working_dir).await?;
    let data = serde_json::to_value(status)
        .map_err(|e| RuntimeError::Internal(format!("serialize: {e}")))?;
    Ok((data, Vec::new()))
}
