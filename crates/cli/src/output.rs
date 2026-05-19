//! `--json` envelope 與 `propose create` data schema。

use serde::Serialize;
use uuid::Uuid;

/// 測試用環境變數：固定 `requestId` 以利 snapshot 比對。
pub const ENV_TEST_REQUEST_ID: &str = "SPECLINK_TEST_REQUEST_ID";

/// `--json` envelope。`T` 為 command 特定 data schema。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T: Serialize> {
    /// `true` 成功；`false` 失敗。
    pub ok: bool,
    /// 成功時為 data payload；失敗時為 `null`。
    pub data: Option<T>,
    /// 非致命警告陣列；陣列必須存在（即使為空）。
    pub warnings: Vec<Warning>,
    /// 失敗時為 [`ErrorBody`]；成功時為 `null`。
    pub error: Option<ErrorBody>,
    /// `req_<32-hex>` 唯一 invocation 識別碼。
    pub request_id: String,
}

impl<T: Serialize> Envelope<T> {
    /// 建構成功 envelope。
    pub fn success(data: T, warnings: Vec<Warning>, request_id: String) -> Self {
        Self {
            ok: true,
            data: Some(data),
            warnings,
            error: None,
            request_id,
        }
    }

    /// 建構失敗 envelope。
    pub fn failure(error: ErrorBody, request_id: String) -> Self {
        Self {
            ok: false,
            data: None,
            warnings: Vec::new(),
            error: Some(error),
            request_id,
        }
    }
}

/// Envelope 中的單一 warning。
#[derive(Debug, Clone, Serialize)]
pub struct Warning {
    /// 點分隔 code（例如 `provider.not_authenticated`）。
    pub code: String,
    /// 給人讀的訊息。
    pub message: String,
}

/// Envelope 中的失敗 detail。
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    /// 點分隔 error code。
    pub code: String,
    /// 給人讀的訊息。
    pub message: String,
    /// 結構化細節，可為空 object（`{}`）。
    pub details: serde_json::Value,
}

/// `propose create` 成功時的 `data` payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeCreateData {
    /// 已建立的 change id。
    pub change_id: String,
    /// 當前 lifecycle 狀態（成功時為 `"proposed"`）。
    pub state: String,
    /// proposal.md 相對於專案根目錄的 POSIX 路徑。
    pub artifact_path: String,
    /// 解析後的 provider mode（`"local"`）。
    pub mode: String,
}

/// 取得本次 invocation 的 `requestId`。
///
/// 若 `SPECLINK_TEST_REQUEST_ID` 環境變數已設定且非空，則直接使用該值（測試用途）；
/// 否則生成 UUID v4 並格式化為 `req_<32-hex>`（無連字號）。
pub fn request_id() -> String {
    match std::env::var(ENV_TEST_REQUEST_ID) {
        Ok(v) if !v.is_empty() => v,
        _ => format!("req_{}", Uuid::new_v4().simple()),
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use crate::output::{
        ENV_TEST_REQUEST_ID, Envelope, ErrorBody, ProposeCreateData, Warning, request_id,
    };
    use serde_json::Value;

    fn parse(v: &impl serde::Serialize) -> Value {
        serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
    }

    #[test]
    fn success_envelope_fields() {
        let data = ProposeCreateData {
            change_id: "demo".to_string(),
            state: "proposed".to_string(),
            artifact_path: ".speclink/changes/demo/proposal.md".to_string(),
            mode: "local".to_string(),
        };
        let env: Envelope<ProposeCreateData> = Envelope::success(data, vec![], "req_x".to_string());
        let v = parse(&env);
        assert_eq!(v["ok"], Value::Bool(true));
        assert!(v["error"].is_null());
        assert!(v["data"].is_object());
        assert!(v["warnings"].is_array());
        assert_eq!(v["requestId"], "req_x");
    }

    #[test]
    fn failure_envelope_fields() {
        let env: Envelope<ProposeCreateData> = Envelope::failure(
            ErrorBody {
                code: "change.already_exists".to_string(),
                message: "change 'demo' already exists".to_string(),
                details: serde_json::json!({}),
            },
            "req_x".to_string(),
        );
        let v = parse(&env);
        assert_eq!(v["ok"], Value::Bool(false));
        assert!(v["data"].is_null());
        assert!(v["error"].is_object());
        assert_eq!(v["error"]["code"], "change.already_exists");
        assert!(v["error"]["message"].is_string());
        assert!(v["error"]["details"].is_object());
    }

    #[test]
    fn propose_create_data_uses_camel_case() {
        let data = ProposeCreateData {
            change_id: "demo".to_string(),
            state: "proposed".to_string(),
            artifact_path: ".speclink/changes/demo/proposal.md".to_string(),
            mode: "local".to_string(),
        };
        let v = parse(&data);
        // 必要欄位
        assert_eq!(v["changeId"], "demo");
        assert_eq!(v["state"], "proposed");
        assert_eq!(v["artifactPath"], ".speclink/changes/demo/proposal.md");
        assert_eq!(v["mode"], "local");
        // 不應該有 snake_case 欄位
        assert!(v.get("change_id").is_none());
        assert!(v.get("artifact_path").is_none());
    }

    #[test]
    fn warning_struct_serializes_correctly() {
        let w = Warning {
            code: "provider.not_authenticated".to_string(),
            message: "Provider 'acme' is configured but not authenticated.".to_string(),
        };
        let v = parse(&w);
        assert_eq!(v["code"], "provider.not_authenticated");
        assert!(v["message"].is_string());
    }

    #[test]
    fn request_id_env_override_is_honored() {
        // Use a guard pattern to avoid leaking env var
        let key = ENV_TEST_REQUEST_ID;
        let prev = std::env::var(key).ok();
        // Safety: only this test mutates this env var.
        unsafe {
            std::env::set_var(key, "req_00000000000000000000000000000000");
        }
        let id = request_id();
        assert_eq!(id, "req_00000000000000000000000000000000");
        unsafe {
            match prev {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn request_id_random_matches_regex() {
        let key = ENV_TEST_REQUEST_ID;
        let prev = std::env::var(key).ok();
        unsafe { std::env::remove_var(key) };
        let id = request_id();
        assert!(
            id.starts_with("req_"),
            "request_id should start with req_: {id}"
        );
        let hex = &id[4..];
        assert_eq!(hex.len(), 32, "hex part must be 32 chars: {id}");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hex must be lowercase hex: {id}"
        );
        unsafe {
            if let Some(v) = prev {
                std::env::set_var(key, v);
            }
        }
    }
}
