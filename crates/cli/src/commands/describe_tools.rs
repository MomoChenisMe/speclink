//! `speclink describe-tools` — read-only catalogue dump in multiple formats.
//!
//! 對應 `tool.describe` op handler（`crates/runtime/src/tool_ops.rs`）。
//! 不讀 `.speclink/`、不讀 state.db、不取 lock；任何工作目錄都能跑。

#![allow(clippy::doc_markdown)]

use serde_json::json;
use speclink_runtime::RuntimeError;
use speclink_runtime::tool_ops::{
    DescribeContent, DescribeFormat, DescribePhase, DescribeToolsRequest, describe_tools,
};

use crate::output::Warning;

/// 執行 `describe-tools`。
///
/// # Errors
/// `ToolFormatNotSupported` / `ToolUnknownOp` / `ToolUnknownCategory` / `Internal`。
pub fn run(
    format: DescribeFormat,
    filter: Vec<String>,
    categories: Vec<String>,
    phases: Vec<DescribePhase>,
    full: bool,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let req = DescribeToolsRequest {
        format,
        filter,
        categories,
        phases,
        full,
    };
    let resp = describe_tools(req)?;
    let payload = json!({
        "format": resp.format.as_str(),
        "content": match resp.content {
            DescribeContent::Json(v) | DescribeContent::CopilotSdk(v) => v,
            DescribeContent::Text(s) => serde_json::Value::String(s),
        },
    });
    Ok((payload, Vec::new()))
}
