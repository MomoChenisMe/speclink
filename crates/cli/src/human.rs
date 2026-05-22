//! Human-mode CLI 輸出格式化（不帶 `--json` 時用）。
//!
//! 規則見 `cli-human-output` capability spec：
//! - 空 object 整體渲染為 `OK`
//! - 空 array 作為 value 時渲染為 `(empty)`
//! - Object 的非 scalar value 換行 + 2 空白縮排展開
//! - Array element 以 `- ` 前綴，nested 內容 + 2 空白縮排
//! - String 不加 JSON 引號；含 `\n` 在每換行後補當前縮排

#![allow(clippy::doc_markdown)]

use serde_json::{Map, Value};

const INDENT_STEP: usize = 2;

/// 把 `serde_json::Value` 轉成 human-mode 字串。
///
/// 純 function：無 I/O、無 panic、無 ANSI、無終端寬度感知。
#[must_use]
pub fn render_human(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) if map.is_empty() => out.push_str("OK"),
        Value::Object(map) => render_object_body(map, 0, &mut out),
        Value::Array(arr) if arr.is_empty() => out.push_str("(empty)"),
        Value::Array(arr) => render_array_body(arr, 0, &mut out),
        scalar => render_scalar(scalar, 0, &mut out),
    }
    out
}

/// 渲染 object body：每個 key 印 `key: <value>` 或 `key:\n<indented>`，
/// 以 `\n` 分隔；第一筆不前置 `\n`，由 caller 處理 prefix。
fn render_object_body(map: &Map<String, Value>, indent: usize, out: &mut String) {
    let prefix = " ".repeat(indent);
    let mut first = true;
    for (k, v) in map {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(&prefix);
        out.push_str(k);
        out.push(':');
        render_value_after_colon(v, indent, out);
    }
}

fn render_array_body(arr: &[Value], indent: usize, out: &mut String) {
    let prefix = " ".repeat(indent);
    let mut first = true;
    for item in arr {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(&prefix);
        out.push_str("- ");
        // dash 與後面的空白共佔 2 字元；nested 內容對齊到 dash 後 +2
        render_value_after_dash(item, indent + INDENT_STEP, out);
    }
}

/// 處理 object key 的 `:` 之後的渲染（含先導空白或換行）。
fn render_value_after_colon(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if map.is_empty() => {
            out.push(' ');
            out.push_str("OK");
        }
        Value::Array(arr) if arr.is_empty() => {
            out.push(' ');
            out.push_str("(empty)");
        }
        Value::Object(map) => {
            out.push('\n');
            render_object_body(map, indent + INDENT_STEP, out);
        }
        Value::Array(arr) => {
            out.push('\n');
            render_array_body(arr, indent + INDENT_STEP, out);
        }
        scalar => {
            out.push(' ');
            render_scalar(scalar, indent + INDENT_STEP, out);
        }
    }
}

/// 處理 array element 在 `- ` 之後的渲染。
fn render_value_after_dash(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if map.is_empty() => out.push_str("OK"),
        Value::Array(arr) if arr.is_empty() => out.push_str("(empty)"),
        Value::Object(map) => render_object_inline_first(map, indent, out),
        Value::Array(arr) => {
            out.push('\n');
            render_array_body(arr, indent, out);
        }
        scalar => render_scalar(scalar, indent, out),
    }
}

/// Object 緊跟在 `- ` 之後：第一個 key 直接接著印，後續 key 換行 + indent。
fn render_object_inline_first(map: &Map<String, Value>, indent: usize, out: &mut String) {
    let prefix = " ".repeat(indent);
    let mut first = true;
    for (k, v) in map {
        if first {
            out.push_str(k);
        } else {
            out.push('\n');
            out.push_str(&prefix);
            out.push_str(k);
        }
        first = false;
        out.push(':');
        render_value_after_colon(v, indent, out);
    }
}

/// Scalar 渲染；含 `\n` 的 string 在每換行後補當前 indent。
fn render_scalar(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            let pad = " ".repeat(indent);
            for (i, line) in s.split('\n').enumerate() {
                if i > 0 {
                    out.push('\n');
                    out.push_str(&pad);
                }
                out.push_str(line);
            }
        }
        // Object / Array reach here only via mis-routed call; defensive fallback.
        _ => out.push_str(&value.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_renders_as_ok() {
        let v = serde_json::json!({});
        insta::assert_snapshot!("empty_object", render_human(&v));
    }

    #[test]
    fn flat_object_renders_key_value_per_line() {
        let v = serde_json::json!({
            "name": "demo-billing",
            "version": 1,
            "active": true,
        });
        // Default serde_json::Map is alphabetical (BTreeMap)；與 slice-A `--json`
        // 既有 snapshot 對齊。
        insta::assert_snapshot!("flat_object", render_human(&v));
    }

    #[test]
    fn nested_object_renders_with_two_space_indent_on_next_line() {
        let v = serde_json::json!({
            "change": {
                "name": "demo-billing",
                "version": 1,
            }
        });
        insta::assert_snapshot!("nested_object", render_human(&v));
    }

    #[test]
    fn array_of_objects_renders_dash_bullet_with_indented_fields() {
        let v = serde_json::json!({
            "artifacts": [
                { "kind": "proposal", "capability": null },
                { "kind": "spec", "capability": "user-auth" },
            ]
        });
        // Inner object fields 走 lexicographic：capability < kind
        insta::assert_snapshot!("array_of_objects", render_human(&v));
    }

    #[test]
    fn empty_array_renders_as_empty_marker() {
        let v = serde_json::json!({ "changes": [] });
        insta::assert_snapshot!("empty_array", render_human(&v));
    }

    #[test]
    fn array_of_scalars_renders_each_with_dash_bullet() {
        let v = serde_json::json!({ "capabilities": ["rate-limiting", "user-auth"] });
        insta::assert_snapshot!("array_of_scalars", render_human(&v));
    }

    #[test]
    fn string_with_newlines_preserves_with_continuation_indent() {
        let v = serde_json::json!({ "content": "line one\nline two" });
        insta::assert_snapshot!("string_with_newlines", render_human(&v));
    }
}
