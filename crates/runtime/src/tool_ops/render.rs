//! `tool.describe` 三種 format renderer。
//!
//! - `json`：陣列 of `{ id, name, description, parameters, outputs_schema }`
//!   （machine 預設；`parameters` = inputs schema、`outputs_schema` 為 sibling）
//! - `text`：markdown table（human debug）
//! - `copilot-sdk`：陣列 of `{ name, description, parameters }`（移除 `id` 與
//!   `outputs_schema`，對應 CopilotKit SDK `defineTool` 的 inputs-only shape）
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

/// JSON format：array of `{ id, name, description, parameters, outputs_schema }`。
///
/// `parameters` 為該 op 的 inputs schema（JSON Schema Draft 2020-12 object）。
/// `outputs_schema` 為 sibling（不 nested 在 parameters 內），對齊
/// `specs/project-status` Requirement「describe-tools json output emits both
/// parameters and outputs_schema for project.status」。
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
                "outputs_schema": (op.outputs_schema)(),
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
    fn render_json_emits_id_name_description_parameters_outputs_keys() {
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
            assert!(obj.contains_key("outputs_schema"));
            assert_eq!(
                obj.len(),
                5,
                "json format must have exactly 5 keys (id, name, description, parameters, outputs_schema)"
            );
        }
    }

    /// B 方案：`outputs_schema` 是 `parameters` 的 sibling（同層 key），
    /// 非 nested 在 `parameters` 內。對齊 spec project-status Requirement
    /// 「describe-tools json output emits both parameters and outputs_schema」。
    #[test]
    fn render_json_includes_outputs_schema_sibling() {
        let project_status = Catalogue::get("project.status").expect("project.status");
        let ops: Vec<&Operation> = vec![project_status];
        let v = render_json(&ops);
        let arr = v.as_array().expect("array");
        let obj = arr[0].as_object().expect("object");
        // sibling，非 nested
        assert!(
            obj.contains_key("parameters"),
            "parameters key missing at top level"
        );
        assert!(
            obj.contains_key("outputs_schema"),
            "outputs_schema must be sibling to parameters, not nested"
        );
        // outputs_schema 不應出現在 parameters 內（confused-developer guard）
        let params = obj["parameters"].as_object().expect("parameters object");
        assert!(
            !params.contains_key("outputs_schema"),
            "outputs_schema must NOT be nested inside parameters"
        );
        // project.status outputs_schema 必須含 required array 且 包含七 field 名
        let outputs = obj["outputs_schema"]
            .as_object()
            .expect("outputs_schema object");
        let required = outputs["required"].as_array().expect("required array");
        let required_names: Vec<&str> = required.iter().filter_map(Value::as_str).collect();
        for name in [
            "provider_type",
            "project_id",
            "working_dir",
            "changes_count",
            "discussions_count",
            "schema_active",
        ] {
            assert!(
                required_names.contains(&name),
                "project.status outputs_schema.required missing {}",
                name
            );
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
