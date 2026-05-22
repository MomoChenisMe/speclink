//! SpecLink CLI JSON envelope.
//!
//! Stable contract for all SpecLink CLI commands invoked with `--json`. 兩種 shape：
//! - 成功：`{ ok: true, data, warnings, requestId }`
//! - 失敗：`{ ok: false, error: { code, message, hint, retryable, retry_after_ms }, requestId }`

#![allow(clippy::doc_markdown)]

use serde::Serialize;
use uuid::Uuid;

/// Envelope（兩種 shape）。
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Envelope<T: Serialize> {
    Ok(SuccessBody<T>),
    Err(ErrorBody),
}

/// 成功 shape：欄位順序固定為 `ok` / `data` / `warnings` / `requestId`，以利 snapshot 比對。
#[derive(Debug, Serialize)]
pub struct SuccessBody<T: Serialize> {
    pub ok: bool,
    pub data: T,
    pub warnings: Vec<Warning>,
    #[serde(rename = "requestId")]
    pub request_id: String,
}

/// 失敗 shape：欄位順序固定為 `ok` / `error` / `requestId`。
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub ok: bool,
    pub error: ErrorDetail,
    #[serde(rename = "requestId")]
    pub request_id: String,
}

/// `warnings` 內單筆。
#[derive(Debug, Serialize, Clone)]
pub struct Warning {
    pub code: String,
    pub message: String,
}

/// `error.*` 內容。順序鎖：`code` / `message` / `hint` / `retryable` / `retry_after_ms`。
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub retryable: bool,
    pub retry_after_ms: Option<u32>,
}

/// 產生新的 request id（UUID v4）。
#[must_use]
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// 構造一個 success envelope。
pub fn success<T: Serialize>(data: T, warnings: Vec<Warning>) -> Envelope<T> {
    Envelope::Ok(SuccessBody {
        ok: true,
        data,
        warnings,
        request_id: new_request_id(),
    })
}

/// 構造一個 error envelope。
#[must_use]
pub fn error(code: &str, message: &str, hint: Option<&str>, retryable: bool) -> Envelope<()> {
    Envelope::Err(ErrorBody {
        ok: false,
        error: ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
            hint: hint.map(str::to_string),
            retryable,
            retry_after_ms: None,
        },
        request_id: new_request_id(),
    })
}

/// Declared error code → process exit code 對照表。
///
/// 與 spec「SpecLink CLI exit codes follow a fixed mapping」嚴格對齊。
///
/// 注意：此處與 `RuntimeError::exit_code()` 兩處對照表並存（bootstrap slice 既有
/// duplication）；如需更新，兩處務必同步。
#[must_use]
pub fn error_code_to_exit(code: &str) -> i32 {
    match code {
        // exit 2 — user-input errors（bootstrap project.* + slice-A change.*/artifact.* + slice-A3 task.* / change.dag_incomplete）
        "project.requires_git"
        | "project.not_initialized"
        | "project.link_target_not_found"
        | "change.not_found"
        | "change.invalid_name"
        | "artifact.kind_invalid"
        | "artifact.capability_required"
        | "artifact.not_found"
        | "change.dag_incomplete"
        | "task.no_tasks_file"
        | "task.index_out_of_range" => 2,
        // exit 7 — conflicts / already-exists（含 etag mismatch + slice-A3 state transition / CAS）
        "project.already_initialized"
        | "change.duplicate_name"
        | "artifact.version_conflict"
        | "state.transition_invalid"
        | "state.version_conflict" => 7,
        // exit 1 — fallthrough：state.invalid_value、state.db.schema_invalid、internal.error
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_shape() {
        let env = success(serde_json::json!({"foo": "bar"}), vec![]);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["ok"], true);
        assert!(json["data"].is_object());
        assert!(json["warnings"].is_array());
        let req = json["requestId"].as_str().expect("requestId");
        assert!(
            uuid::Uuid::parse_str(req).is_ok(),
            "requestId must be UUID v4: got {req}"
        );
    }

    #[test]
    fn error_envelope_shape() {
        let env = error(
            "project.requires_git",
            "not a git repo",
            Some("run `git init` first"),
            false,
        );
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["code"], "project.requires_git");
        assert_eq!(json["error"]["retryable"], false);
        assert!(json["error"]["hint"].is_string());
        assert!(json["error"]["retry_after_ms"].is_null());
        let req = json["requestId"].as_str().expect("requestId");
        assert!(uuid::Uuid::parse_str(req).is_ok());
    }

    #[test]
    fn error_envelope_json_snapshot_with_fixed_request_id() {
        // 鎖 requestId 以便 snapshot 比對。
        let body = ErrorBody {
            ok: false,
            error: ErrorDetail {
                code: "project.requires_git".to_string(),
                message: "not a git repo".to_string(),
                hint: Some("run `git init` first".to_string()),
                retryable: false,
                retry_after_ms: None,
            },
            request_id: "00000000-0000-4000-8000-000000000000".to_string(),
        };
        let env: Envelope<()> = Envelope::Err(body);
        let json = serde_json::to_string_pretty(&env).unwrap();
        insta::assert_snapshot!("error_envelope_pretty", json);
    }

    #[test]
    fn exit_code_mapping_matches_spec_table() {
        // bootstrap-slice
        assert_eq!(error_code_to_exit("project.requires_git"), 2);
        assert_eq!(error_code_to_exit("project.not_initialized"), 2);
        assert_eq!(error_code_to_exit("project.link_target_not_found"), 2);
        assert_eq!(error_code_to_exit("project.already_initialized"), 7);
        assert_eq!(error_code_to_exit("internal.error"), 1);
        // slice-A
        assert_eq!(error_code_to_exit("change.not_found"), 2);
        assert_eq!(error_code_to_exit("change.duplicate_name"), 7);
        assert_eq!(error_code_to_exit("change.invalid_name"), 2);
        assert_eq!(error_code_to_exit("artifact.kind_invalid"), 2);
        assert_eq!(error_code_to_exit("artifact.capability_required"), 2);
        assert_eq!(error_code_to_exit("artifact.not_found"), 2);
        assert_eq!(error_code_to_exit("artifact.version_conflict"), 7);
        // slice-A3
        assert_eq!(error_code_to_exit("state.invalid_value"), 1);
        assert_eq!(error_code_to_exit("state.transition_invalid"), 7);
        assert_eq!(error_code_to_exit("state.version_conflict"), 7);
        assert_eq!(error_code_to_exit("state.db.schema_invalid"), 1);
        assert_eq!(error_code_to_exit("change.dag_incomplete"), 2);
        assert_eq!(error_code_to_exit("task.no_tasks_file"), 2);
        assert_eq!(error_code_to_exit("task.index_out_of_range"), 2);
    }
}
