//! `tool.describe` 三種 format renderer。
//!
//! - `json`：陣列 of `{ id, name, description, parameters }`（machine 預設）
//! - `text`：markdown table（human debug）
//! - `copilot-sdk`：陣列 of `{ name, description, parameters }`（移除 `id`，對應
//!   CopilotKit SDK `defineTool` shape）
//!
//! 未來 5 種 deferred format（`copilotkit` / `openai` / `langchain` / `mcp` /
//! `claude`）接同一介面、留 sub-module 擴充點。

use serde_json::{Value, json};

use crate::catalogue::Operation;

/// 統一 renderer 介面；3 個 MVP renderer 與後續 5 個 deferred format 共用此 trait。
pub trait Render {
    type Output;
    fn render(ops: &[&Operation]) -> Self::Output;
}

/// JSON format renderer marker。
pub struct JsonRenderer;
impl Render for JsonRenderer {
    type Output = Value;
    fn render(ops: &[&Operation]) -> Value {
        render_json(ops)
    }
}

/// Text format renderer marker。
pub struct TextRenderer;
impl Render for TextRenderer {
    type Output = String;
    fn render(ops: &[&Operation]) -> String {
        render_text(ops)
    }
}

/// CopilotKit SDK renderer marker。
pub struct CopilotSdkRenderer;
impl Render for CopilotSdkRenderer {
    type Output = Value;
    fn render(ops: &[&Operation]) -> Value {
        render_copilot_sdk(ops)
    }
}

/// JSON format：array of `{ id, name, description, parameters }`。
#[must_use]
pub fn render_json(ops: &[&Operation]) -> Value {
    let arr: Vec<Value> = ops
        .iter()
        .map(|op| {
            json!({
                "id": op.id,
                "name": op.tool_binding,
                "description": op.description,
                "parameters": (op.inputs_schema)(),
            })
        })
        .collect();
    Value::Array(arr)
}

/// Text format：markdown table（| id | category | cli | tool_binding | description |）。
/// 第一行為 header、第二行為 separator、其後為資料列；總行數 = ops.len() + 2。
#[must_use]
pub fn render_text(ops: &[&Operation]) -> String {
    let mut out = String::new();
    out.push_str("| id | category | cli | tool_binding | description |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for op in ops {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            op.id, op.category, op.cli, op.tool_binding, op.description
        ));
    }
    out
}

/// CopilotKit SDK format：array of `{ name, description, parameters }` — 移除 `id`。
#[must_use]
pub fn render_copilot_sdk(ops: &[&Operation]) -> Value {
    let arr: Vec<Value> = ops
        .iter()
        .map(|op| {
            json!({
                "name": op.tool_binding,
                "description": op.description,
                "parameters": (op.inputs_schema)(),
            })
        })
        .collect();
    Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::Catalogue;

    #[test]
    fn render_json_emits_id_name_description_parameters_keys() {
        let ops: Vec<&Operation> = Catalogue::all().iter().take(3).collect();
        let v = render_json(&ops);
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 3);
        for el in arr {
            let obj = el.as_object().expect("object");
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("description"));
            assert!(obj.contains_key("parameters"));
            assert_eq!(obj.len(), 4, "json format must have exactly 4 keys");
        }
    }

    #[test]
    fn render_text_emits_markdown_table_with_header_and_separator() {
        let ops: Vec<&Operation> = Catalogue::all().iter().take(5).collect();
        let s = render_text(&ops);
        let lines: Vec<&str> = s.lines().collect();
        assert!(lines[0].starts_with('|'), "header must start with |");
        assert!(lines[1].starts_with('|'), "separator must start with |");
        assert!(lines[1].contains("---"), "separator must contain ---");
        let data_rows = lines.len() - 2;
        assert_eq!(data_rows, 5, "5 data rows + header + separator");
    }

    #[test]
    fn render_copilot_sdk_emits_name_description_parameters_only() {
        let ops: Vec<&Operation> = Catalogue::all().iter().take(3).collect();
        let v = render_copilot_sdk(&ops);
        let arr = v.as_array().expect("array");
        for el in arr {
            let obj = el.as_object().expect("object");
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("description"));
            assert!(obj.contains_key("parameters"));
            assert!(!obj.contains_key("id"), "copilot-sdk format must omit id");
            assert_eq!(obj.len(), 3, "copilot-sdk format must have exactly 3 keys");
        }
    }
}
