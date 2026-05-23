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

// =====================================================================
// Outputs schemas (B 方案 — Operation.outputs_schema 欄位)
// =====================================================================
//
// 37 個 op 對應的 outputs schema 函式。命名規則：<inputs fn name>_outputs。
// 未實作 op 一律給 `empty_object_outputs_schema()` stub；對應 SDD slice 真做時
// 補完整 schema（與 inputs 同迭代節奏）。
//
// 兩個本 slice 補完整 schema 的 op：
//   - `project.status` → 對齊 `doc/protocol/operations.md` §1389
//   - `change.show` → envelope shape（change / artifacts / all_tasks_done / next_actions）

fn empty_object_outputs_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {},
        "additionalProperties": true
    })
}

// ----- project outputs -----

pub fn project_init_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn project_link_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn project_unlink_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn project_status_outputs() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": [
            "provider_type",
            "project_id",
            "working_dir",
            "changes_count",
            "discussions_count",
            "schema_active"
        ],
        "additionalProperties": false,
        "properties": {
            "provider_type": { "enum": ["local", "http"] },
            "project_id": { "type": "string" },
            "working_dir": { "type": "string" },
            "current_change": {
                "type": ["object", "null"],
                "description": "若有 change 處於 in_progress + actor 為當前 host；否則 null。",
                "properties": {
                    "change_id": { "type": "string" },
                    "state": { "type": "string" },
                    "actor": { "type": "object" }
                }
            },
            "changes_count": {
                "type": "object",
                "required": [
                    "proposing", "reviewing", "ready",
                    "in_progress", "code_reviewing", "archived"
                ],
                "properties": {
                    "proposing": { "type": "integer", "minimum": 0 },
                    "reviewing": { "type": "integer", "minimum": 0 },
                    "ready": { "type": "integer", "minimum": 0 },
                    "in_progress": { "type": "integer", "minimum": 0 },
                    "code_reviewing": { "type": "integer", "minimum": 0 },
                    "archived": { "type": "integer", "minimum": 0 }
                }
            },
            "discussions_count": {
                "type": "object",
                "required": ["active", "converged"],
                "properties": {
                    "active": { "type": "integer", "minimum": 0 },
                    "converged": { "type": "integer", "minimum": 0 }
                }
            },
            "schema_active": { "type": "string" }
        }
    })
}

// ----- config outputs -----

pub fn config_read_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn config_write_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- schema outputs -----

pub fn schema_list_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn schema_show_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn schema_fork_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn schema_delete_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- discuss outputs -----

pub fn discuss_new_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn discuss_list_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn discuss_show_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn discuss_patch_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn discuss_conclude_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn discuss_delete_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- change outputs -----

pub fn change_create_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn change_list_outputs() -> Value {
    empty_object_outputs_schema()
}

pub fn change_show_outputs() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["change", "artifacts", "all_tasks_done", "next_actions"],
        "additionalProperties": false,
        "properties": {
            "change": {
                "type": "object",
                "description": "Change row metadata from state.db.",
                "required": [
                    "change_id", "name", "state", "schema_id",
                    "version", "created_at", "updated_at"
                ],
                "properties": {
                    "change_id": { "type": "string" },
                    "name": { "type": "string" },
                    "state": { "type": "string" },
                    "schema_id": { "type": "string" },
                    "version": { "type": "integer", "minimum": 1 },
                    "created_at": { "type": "string" },
                    "updated_at": { "type": "string" }
                }
            },
            "artifacts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["kind"],
                    "properties": {
                        "kind": { "type": "string" },
                        "capability": { "type": ["string", "null"] }
                    }
                }
            },
            "all_tasks_done": {
                "type": "boolean",
                "description": "Mirrors the change row's all_tasks_done column maintained by task_ops."
            },
            "next_actions": {
                "type": "array",
                "description": "State-driven next-action hints for dogfood UX (see specs/change-store).",
                "items": { "type": "string" }
            }
        }
    })
}

pub fn change_delete_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- artifact outputs -----

pub fn artifact_write_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn artifact_read_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- apply outputs -----

pub fn apply_start_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn apply_pause_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn task_done_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- review outputs -----

pub fn review_approve_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn review_reject_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn review_history_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- archive outputs -----

pub fn archive_run_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- spec outputs -----

pub fn spec_list_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn spec_show_outputs() -> Value {
    empty_object_outputs_schema()
}

// ----- instructions / analyze / doctor / tool outputs -----

pub fn instructions_get_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn analyze_run_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn validate_run_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn drift_run_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn doctor_run_outputs() -> Value {
    empty_object_outputs_schema()
}
pub fn tool_describe_outputs() -> Value {
    empty_object_outputs_schema()
}
