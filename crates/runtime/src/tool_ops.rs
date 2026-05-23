//! `tool.describe` operation handler。
//!
//! Read-only catalogue lookup + AND-intersection filter + 多 format render。
//! 對應 `doc/protocol/operations.md` `tool.describe`、`doc/speclink-design.md` §16.2 / §22.2。

use serde_json::Value;

use crate::catalogue::{Catalogue, Operation, Phase};
use crate::error::RuntimeError;

pub mod render;

/// `describe-tools --format` 的合法 enum。MVP 只支援前 3 個；其餘 5 個為 [deferred]，
/// runtime 收到時 early return `ToolFormatNotSupported`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribeFormat {
    Json,
    Text,
    CopilotSdk,
    /// [deferred] — `tool.format_not_supported`
    Copilotkit,
    /// [deferred] — `tool.format_not_supported`
    Openai,
    /// [deferred] — `tool.format_not_supported`
    Langchain,
    /// [deferred] — `tool.format_not_supported`
    Mcp,
    /// [deferred] — `tool.format_not_supported`
    Claude,
}

impl DescribeFormat {
    /// 對應 `--format` literal 字串（machine 表示）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DescribeFormat::Json => "json",
            DescribeFormat::Text => "text",
            DescribeFormat::CopilotSdk => "copilot-sdk",
            DescribeFormat::Copilotkit => "copilotkit",
            DescribeFormat::Openai => "openai",
            DescribeFormat::Langchain => "langchain",
            DescribeFormat::Mcp => "mcp",
            DescribeFormat::Claude => "claude",
        }
    }

    /// MVP 支援的 3 個 format。
    #[must_use]
    pub fn is_mvp_supported(self) -> bool {
        matches!(
            self,
            DescribeFormat::Json | DescribeFormat::Text | DescribeFormat::CopilotSdk
        )
    }
}

/// Skill phase 篩選值（與 `catalogue::Phase` 同型，獨立列舉避免 catalogue
/// 內部表示外洩到 CLI 表面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribePhase {
    Discuss,
    Propose,
    Apply,
    Archive,
    Ingest,
}

impl DescribePhase {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DescribePhase::Discuss => "discuss",
            DescribePhase::Propose => "propose",
            DescribePhase::Apply => "apply",
            DescribePhase::Archive => "archive",
            DescribePhase::Ingest => "ingest",
        }
    }

    fn matches(self, p: Phase) -> bool {
        matches!(
            (self, p),
            (DescribePhase::Discuss, Phase::Discuss)
                | (DescribePhase::Propose, Phase::Propose)
                | (DescribePhase::Apply, Phase::Apply)
                | (DescribePhase::Archive, Phase::Archive)
                | (DescribePhase::Ingest, Phase::Ingest)
        )
    }
}

/// `tool.describe` 請求。
#[derive(Debug, Clone)]
pub struct DescribeToolsRequest {
    pub format: DescribeFormat,
    /// 只輸出 catalogue 內這些 id；空集合視為不套用 filter。
    pub filter: Vec<String>,
    /// 只輸出這些 category；空集合視為不套用 filter。
    pub categories: Vec<String>,
    /// 只輸出涉及這些 phase 的 op；空集合視為不套用 filter。
    pub phases: Vec<DescribePhase>,
    /// false（預設）只輸出 curated subset；true 切全集。
    pub full: bool,
}

/// `tool.describe` 回應。`content` 依 `format` 為不同 shape。
#[derive(Debug, Clone)]
pub struct DescribeToolsResponse {
    pub format: DescribeFormat,
    pub content: DescribeContent,
}

/// Render 後的內容，依 format 分支。
#[derive(Debug, Clone)]
pub enum DescribeContent {
    /// JSON array of tool descriptors。
    Json(Value),
    /// Markdown table。
    Text(String),
    /// CopilotKit SDK `defineTool` shape 的 JSON array。
    CopilotSdk(Value),
}

impl DescribeContent {
    /// 轉成 JSON envelope `data.content`。
    #[must_use]
    pub fn into_value(self) -> Value {
        match self {
            DescribeContent::Json(v) | DescribeContent::CopilotSdk(v) => v,
            DescribeContent::Text(s) => Value::String(s),
        }
    }
}

/// `tool.describe` op handler。
///
/// 行為：read-only catalogue lookup + AND-intersection filter + 依 format render。
///
/// # Errors
/// - `RuntimeError::ToolFormatNotSupported`：format 在 enum 內但屬 [deferred] 5 種之一
/// - `RuntimeError::ToolUnknownOp`：filter 含 catalogue 內沒有的 id
/// - `RuntimeError::ToolUnknownCategory`：categories 含 catalogue 內沒有的 category
pub fn describe_tools(req: DescribeToolsRequest) -> Result<DescribeToolsResponse, RuntimeError> {
    if !req.format.is_mvp_supported() {
        return Err(RuntimeError::ToolFormatNotSupported {
            format: req.format.as_str().to_string(),
        });
    }

    for id in &req.filter {
        if Catalogue::get(id).is_none() {
            return Err(RuntimeError::ToolUnknownOp { id: id.clone() });
        }
    }

    let known_categories: std::collections::HashSet<&str> =
        Catalogue::all().iter().map(|op| op.category).collect();
    for cat in &req.categories {
        if !known_categories.contains(cat.as_str()) {
            return Err(RuntimeError::ToolUnknownCategory {
                category: cat.clone(),
            });
        }
    }

    let initial: Vec<&'static Operation> = if req.full {
        Catalogue::all().iter().collect()
    } else {
        Catalogue::all().iter().filter(|op| op.curated).collect()
    };

    let filter_set: std::collections::HashSet<&str> =
        req.filter.iter().map(String::as_str).collect();
    let cat_set: std::collections::HashSet<&str> =
        req.categories.iter().map(String::as_str).collect();

    let selected: Vec<&'static Operation> = initial
        .into_iter()
        .filter(|op| filter_set.is_empty() || filter_set.contains(op.id))
        .filter(|op| cat_set.is_empty() || cat_set.contains(op.category))
        .filter(|op| {
            req.phases.is_empty()
                || req
                    .phases
                    .iter()
                    .any(|p| op.phases.iter().any(|q| p.matches(*q)))
        })
        .collect();

    let content = match req.format {
        DescribeFormat::Json => DescribeContent::Json(render::render_json(&selected)),
        DescribeFormat::Text => DescribeContent::Text(render::render_text(&selected)),
        DescribeFormat::CopilotSdk => {
            DescribeContent::CopilotSdk(render::render_copilot_sdk(&selected))
        }
        _ => unreachable!("filtered out by is_mvp_supported check above"),
    };

    Ok(DescribeToolsResponse {
        format: req.format,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req_defaults() -> DescribeToolsRequest {
        DescribeToolsRequest {
            format: DescribeFormat::Json,
            filter: vec![],
            categories: vec![],
            phases: vec![],
            full: false,
        }
    }

    fn json_ids(resp: &DescribeToolsResponse) -> Vec<String> {
        let v = match &resp.content {
            DescribeContent::Json(v) => v,
            other => panic!("expected Json content, got {other:?}"),
        };
        v.as_array()
            .expect("array")
            .iter()
            .map(|el| {
                el.get("id")
                    .and_then(Value::as_str)
                    .expect("id")
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn describe_tools_default_returns_curated_12() {
        let resp = describe_tools(req_defaults()).expect("ok");
        assert_eq!(resp.format, DescribeFormat::Json);
        let ids = json_ids(&resp);
        assert_eq!(ids.len(), 12, "default = curated 12; got {ids:?}");
    }

    #[test]
    fn describe_tools_full_returns_37() {
        let req = DescribeToolsRequest {
            full: true,
            ..req_defaults()
        };
        let resp = describe_tools(req).expect("ok");
        let ids = json_ids(&resp);
        assert_eq!(ids.len(), 37);
    }

    #[test]
    fn describe_tools_categories_and_filter_intersection() {
        // Filter flags SHALL apply as AND intersection — `--categories change` +
        // `--filter change.delete` returns only `change.delete`.
        let req = DescribeToolsRequest {
            full: true,
            categories: vec!["change".into()],
            filter: vec!["change.delete".into()],
            ..req_defaults()
        };
        let resp = describe_tools(req).expect("ok");
        let ids = json_ids(&resp);
        assert_eq!(ids, vec!["change.delete".to_string()]);
    }

    #[test]
    fn describe_tools_empty_intersection_returns_empty() {
        let req = DescribeToolsRequest {
            full: true,
            categories: vec!["change".into()],
            filter: vec!["discuss.new".into()],
            ..req_defaults()
        };
        let resp = describe_tools(req).expect("ok");
        let ids = json_ids(&resp);
        assert!(ids.is_empty());
    }

    #[test]
    fn describe_tools_phases_discuss_returns_only_discuss_ids() {
        let req = DescribeToolsRequest {
            full: true,
            phases: vec![DescribePhase::Discuss],
            ..req_defaults()
        };
        let resp = describe_tools(req).expect("ok");
        let ids = json_ids(&resp);
        assert!(!ids.is_empty());
        for id in &ids {
            assert!(
                id.starts_with("discuss.") || id == "instructions.get",
                "discuss phase contains non-discuss id: {id}"
            );
        }
    }

    #[test]
    fn describe_tools_unknown_op_returns_unknown_op_error() {
        let req = DescribeToolsRequest {
            filter: vec!["no.such.op".into()],
            ..req_defaults()
        };
        let err = describe_tools(req).expect_err("must error");
        match err {
            RuntimeError::ToolUnknownOp { id } => assert_eq!(id, "no.such.op"),
            other => panic!("expected ToolUnknownOp, got {other:?}"),
        }
    }

    #[test]
    fn describe_tools_unknown_category_returns_unknown_category_error() {
        let req = DescribeToolsRequest {
            categories: vec!["bogus".into()],
            ..req_defaults()
        };
        let err = describe_tools(req).expect_err("must error");
        match err {
            RuntimeError::ToolUnknownCategory { category } => assert_eq!(category, "bogus"),
            other => panic!("expected ToolUnknownCategory, got {other:?}"),
        }
    }

    #[test]
    fn describe_tools_unsupported_format_returns_format_not_supported() {
        for fmt in [
            DescribeFormat::Copilotkit,
            DescribeFormat::Openai,
            DescribeFormat::Langchain,
            DescribeFormat::Mcp,
            DescribeFormat::Claude,
        ] {
            let req = DescribeToolsRequest {
                format: fmt,
                ..req_defaults()
            };
            let err = describe_tools(req).expect_err("must error");
            match err {
                RuntimeError::ToolFormatNotSupported { format } => {
                    assert_eq!(format, fmt.as_str());
                }
                other => panic!("expected ToolFormatNotSupported, got {other:?}"),
            }
        }
    }
}
