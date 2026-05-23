//! 37 個 operation 的 JSON Schema 函式。每個函式回 `serde_json::Value`，
//! 為 JSON Schema Draft 2020-12 物件；deterministic（同函式連跑兩次相等）。
//!
//! MVP 階段對 4 個 slice-A 已實作 ops（`change.create` / `change.list` / `change.show` /
//! `change.delete`）填完整 schema 對齊 `doc/protocol/operations.md`；其他 33 個 op 給
//! `{ "type": "object", "properties": {}, "additionalProperties": false }` stub，
//! 等對應 SDD slice 真正實作時補完整 schema。

use serde_json::{Value, json};

fn empty_object_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {},
        "additionalProperties": false
    })
}

// ----- project -----

pub fn project_init() -> Value {
    empty_object_schema()
}
pub fn project_link() -> Value {
    empty_object_schema()
}
pub fn project_unlink() -> Value {
    empty_object_schema()
}
pub fn project_status() -> Value {
    empty_object_schema()
}

// ----- config -----

pub fn config_read() -> Value {
    empty_object_schema()
}
pub fn config_write() -> Value {
    empty_object_schema()
}

// ----- schema -----

pub fn schema_list() -> Value {
    empty_object_schema()
}
pub fn schema_show() -> Value {
    empty_object_schema()
}
pub fn schema_fork() -> Value {
    empty_object_schema()
}
pub fn schema_delete() -> Value {
    empty_object_schema()
}

// ----- discuss -----

pub fn discuss_new() -> Value {
    empty_object_schema()
}
pub fn discuss_list() -> Value {
    empty_object_schema()
}
pub fn discuss_show() -> Value {
    empty_object_schema()
}
pub fn discuss_patch() -> Value {
    empty_object_schema()
}
pub fn discuss_conclude() -> Value {
    empty_object_schema()
}
pub fn discuss_delete() -> Value {
    empty_object_schema()
}

// ----- change（4 個 slice-A 已實作 ops，給完整 schema）-----

pub fn change_create() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["name"],
        "properties": {
            "name": {
                "type": "string",
                "pattern": "^[a-z][a-z0-9]*(-[a-z0-9]+)*$",
                "minLength": 1,
                "maxLength": 64,
                "description": "Change name (kebab-case, 1-64 bytes)"
            },
            "description": {
                "type": "string",
                "description": "Optional one-line summary"
            }
        }
    })
}

pub fn change_list() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

pub fn change_show() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["name"],
        "properties": {
            "name": {
                "type": "string",
                "description": "Change name to read"
            }
        }
    })
}

pub fn change_delete() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["name", "confirm_name"],
        "properties": {
            "name": {
                "type": "string",
                "description": "Change name to delete"
            },
            "confirm_name": {
                "type": "string",
                "description": "Must equal `name` for destructive confirmation"
            }
        }
    })
}

// ----- artifact -----

pub fn artifact_write() -> Value {
    empty_object_schema()
}
pub fn artifact_read() -> Value {
    empty_object_schema()
}

// ----- apply -----

pub fn apply_start() -> Value {
    empty_object_schema()
}
pub fn apply_pause() -> Value {
    empty_object_schema()
}
pub fn task_done() -> Value {
    empty_object_schema()
}

// ----- review -----

pub fn review_approve() -> Value {
    empty_object_schema()
}
pub fn review_reject() -> Value {
    empty_object_schema()
}
pub fn review_history() -> Value {
    empty_object_schema()
}

// ----- archive -----

pub fn archive_run() -> Value {
    empty_object_schema()
}

// ----- spec -----

pub fn spec_list() -> Value {
    empty_object_schema()
}
pub fn spec_show() -> Value {
    empty_object_schema()
}

// ----- instructions / analyze / doctor / tool -----

pub fn instructions_get() -> Value {
    empty_object_schema()
}
pub fn analyze_run() -> Value {
    empty_object_schema()
}
pub fn validate_run() -> Value {
    empty_object_schema()
}
pub fn drift_run() -> Value {
    empty_object_schema()
}
pub fn doctor_run() -> Value {
    empty_object_schema()
}
pub fn tool_describe() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "format": {
                "enum": ["json", "text", "copilot-sdk", "copilotkit", "openai", "langchain", "mcp", "claude"],
                "default": "json"
            },
            "filter": { "type": "array", "items": { "type": "string" } },
            "categories": { "type": "array", "items": { "type": "string" } },
            "phases": {
                "type": "array",
                "items": { "enum": ["discuss", "propose", "apply", "archive", "ingest"] }
            },
            "full": { "type": "boolean", "default": false }
        }
    })
}
