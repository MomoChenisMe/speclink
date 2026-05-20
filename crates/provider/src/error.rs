//! Provider 層錯誤型別與點分隔 error code。
//!
//! 每個 variant 對應 `cli-machine-interface` spec 中 Error code naming convention 規定的
//! 點分隔字串；CLI 層 `classify()` 會依 variant 將 error 對映到 exit code。

use crate::model::ChangeId;
use thiserror::Error;

/// Provider 層的領域錯誤型別。
///
/// 每個 variant 對應一個穩定的 error code，CLI 層會將其映射為 exit code 與 envelope `error.code`。
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Provider 已設定但未提供 auth token。
    #[error("provider '{provider_name}' is not authenticated")]
    NotAuthenticated {
        /// 觸發 fallback 或失敗的 provider 名稱。
        provider_name: String,
    },

    /// Provider 設定但無法觸及（例如 HTTP 端點 down）。
    #[error("provider '{provider_name}' is unavailable")]
    Unavailable {
        /// 不可用的 provider 名稱。
        provider_name: String,
    },

    /// Change id 已存在於 storage。
    #[error("change '{change_id}' already exists")]
    ChangeAlreadyExists {
        /// 重複的 change id。
        change_id: ChangeId,
    },

    /// Change id 不存在。
    #[error("change '{change_id}' not found")]
    ChangeNotFound {
        /// 缺少的 change id。
        change_id: ChangeId,
    },

    /// Change id 不符合 kebab-case 規則。
    #[error("invalid change id: '{change_id}'")]
    InvalidChangeId {
        /// 不合法的 change id 原始字串。
        change_id: String,
    },

    /// Artifact 檔案已存在於目標路徑，本 change 不允許覆寫。
    #[error("artifact '{kind}' already exists for change '{change_id}'")]
    ArtifactAlreadyExists {
        /// Artifact 種類字串（`"design"` / `"tasks"` / `"proposal"` / `"spec:CAP"`；`CAP` 為 capability 名稱）。
        kind: String,
        /// 目標 change id。
        change_id: ChangeId,
    },

    /// `artifact write spec` 缺 `--capability`。
    #[error("--capability is required for spec artifacts")]
    MissingCapability,

    /// Capability 名稱不符合 kebab-case 規則。
    #[error("invalid capability name: '{capability}'")]
    InvalidCapability {
        /// 不合法的 capability 名稱原始字串。
        capability: String,
    },

    /// 兜底錯誤；訊息僅供人類閱讀，不會在 JSON envelope `error.details` 中洩漏。
    #[error("internal provider error: {message}")]
    Internal {
        /// 人類可讀錯誤訊息。
        message: String,
    },
}

impl ProviderError {
    /// 對應 `cli-machine-interface` 規定的點分隔 error code。
    pub fn error_code(&self) -> &'static str {
        match self {
            ProviderError::NotAuthenticated { .. } => "provider.not_authenticated",
            ProviderError::Unavailable { .. } => "provider.unavailable",
            ProviderError::ChangeAlreadyExists { .. } => "change.already_exists",
            ProviderError::ChangeNotFound { .. } => "change.not_found",
            ProviderError::InvalidChangeId { .. } => "change.invalid_id",
            ProviderError::ArtifactAlreadyExists { .. } => "artifact.already_exists",
            ProviderError::MissingCapability => "artifact.missing_capability",
            ProviderError::InvalidCapability { .. } => "artifact.invalid_capability",
            ProviderError::Internal { .. } => "internal.error",
        }
    }
}

/// Provider resolution 過程的錯誤型別。
#[derive(Debug, Error)]
pub enum ResolutionError {
    /// 設定要求 remote provider 但未認證且 `fallback = "disabled"`，無法降級。
    #[error("provider '{provider_name}' requires authentication but local fallback is disabled")]
    AuthRequiredNoFallback {
        /// 觸發此錯誤的 provider 名稱。
        provider_name: String,
    },

    /// 設定檔含不合法 `fallback` 或其他無法解析的欄位。
    #[error("invalid provider configuration: {reason}")]
    InvalidConfig {
        /// 失敗原因。
        reason: String,
    },
}

impl ResolutionError {
    /// 對應點分隔 error code。
    pub fn error_code(&self) -> &'static str {
        match self {
            ResolutionError::AuthRequiredNoFallback { .. } => "provider.not_authenticated",
            ResolutionError::InvalidConfig { .. } => "input.invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ProviderError;
    use crate::model::ChangeId;

    #[test]
    fn not_authenticated_code() {
        let err = ProviderError::NotAuthenticated {
            provider_name: "acme".to_string(),
        };
        assert_eq!(err.error_code(), "provider.not_authenticated");
    }

    #[test]
    fn unavailable_code() {
        let err = ProviderError::Unavailable {
            provider_name: "acme".to_string(),
        };
        assert_eq!(err.error_code(), "provider.unavailable");
    }

    #[test]
    fn change_already_exists_code() {
        let err = ProviderError::ChangeAlreadyExists {
            change_id: ChangeId::from("demo"),
        };
        assert_eq!(err.error_code(), "change.already_exists");
    }

    #[test]
    fn change_not_found_code() {
        let err = ProviderError::ChangeNotFound {
            change_id: ChangeId::from("demo"),
        };
        assert_eq!(err.error_code(), "change.not_found");
    }

    #[test]
    fn internal_code() {
        let err = ProviderError::Internal {
            message: "boom".to_string(),
        };
        assert_eq!(err.error_code(), "internal.error");
    }

    #[test]
    fn artifact_already_exists_code() {
        let err = ProviderError::ArtifactAlreadyExists {
            kind: "design".to_string(),
            change_id: ChangeId::from("demo"),
        };
        assert_eq!(err.error_code(), "artifact.already_exists");
    }

    #[test]
    fn missing_capability_code() {
        let err = ProviderError::MissingCapability;
        assert_eq!(err.error_code(), "artifact.missing_capability");
    }

    #[test]
    fn invalid_capability_code() {
        let err = ProviderError::InvalidCapability {
            capability: "Bad-Name".to_string(),
        };
        assert_eq!(err.error_code(), "artifact.invalid_capability");
    }
}
