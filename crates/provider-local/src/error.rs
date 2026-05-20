//! Local provider 錯誤型別。
//!
//! 所有 variant 在 CLI 層映射為 `internal.error`（exit code 1），除了
//! `InvalidChangeId` 映射為 `change.invalid_id`（exit code 2）。

use thiserror::Error;

/// Local provider 的領域錯誤型別。
#[derive(Debug, Error)]
pub enum LocalProviderError {
    /// I/O 失敗。
    #[error("filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化失敗。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// State 資料庫操作失敗。
    #[error("state database error: {0}")]
    StateDb(#[from] StateDbError),

    /// Change id 不符合 kebab-case 規則。
    #[error("invalid change id: '{change_id}'")]
    InvalidChangeId {
        /// 不合法的 change id 原始字串。
        change_id: String,
    },

    /// Change 目錄已存在。
    #[error("change '{change_id}' already exists")]
    ChangeAlreadyExists {
        /// 已存在的 change id。
        change_id: String,
    },

    /// 指定 change 找不到 `metadata.json`（即 change 不存在）。
    #[error("change '{change_id}' not found")]
    ChangeNotFound {
        /// 缺少的 change id。
        change_id: String,
    },

    /// 目標 artifact 檔案已存在，本 change 不允許覆寫。
    #[error("artifact '{kind}' already exists for change '{change_id}'")]
    ArtifactAlreadyExists {
        /// Artifact 種類字串（`"design"` / `"tasks"` / `"spec:CAP"` 等；`CAP` 為 capability 名稱）。
        kind: String,
        /// 目標 change id。
        change_id: String,
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

    /// 兜底錯誤。
    #[error("local provider error: {message}")]
    Internal {
        /// 人類可讀錯誤訊息。
        message: String,
    },
}

impl LocalProviderError {
    /// 對應點分隔 error code。
    pub fn error_code(&self) -> &'static str {
        match self {
            LocalProviderError::InvalidChangeId { .. } => "change.invalid_id",
            LocalProviderError::ChangeAlreadyExists { .. } => "change.already_exists",
            LocalProviderError::ChangeNotFound { .. } => "change.not_found",
            LocalProviderError::ArtifactAlreadyExists { .. } => "artifact.already_exists",
            LocalProviderError::MissingCapability => "artifact.missing_capability",
            LocalProviderError::InvalidCapability { .. } => "artifact.invalid_capability",
            LocalProviderError::Io(_)
            | LocalProviderError::Json(_)
            | LocalProviderError::StateDb(_)
            | LocalProviderError::Internal { .. } => "internal.error",
        }
    }
}

/// State 資料庫錯誤型別。所有 variant 對應 `internal.error`。
#[derive(Debug, Error)]
pub enum StateDbError {
    /// `rusqlite` 操作失敗。
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// 資料庫 schema 版本高於 CLI 預期版本。
    #[error("incompatible state database version: found {found}, expected at most {expected}")]
    IncompatibleVersion {
        /// CLI 支援的最高 schema 版本。
        expected: u32,
        /// 實際磁碟上的 `PRAGMA user_version`。
        found: u32,
    },

    /// 背景 `spawn_blocking` task 失敗或被取消。
    #[error("background task failed: {0}")]
    Join(#[from] tokio::task::JoinError),

    /// 兜底錯誤；目前用於互斥鎖中毒。
    #[error("state database internal error: {message}")]
    Internal {
        /// 人類可讀錯誤訊息。
        message: String,
    },
}

impl StateDbError {
    /// 對應點分隔 error code。
    pub fn error_code(&self) -> &'static str {
        "internal.error"
    }
}

#[cfg(test)]
mod tests {
    use crate::error::LocalProviderError;

    #[test]
    fn invalid_change_id_code() {
        let err = LocalProviderError::InvalidChangeId {
            change_id: "Bad".to_string(),
        };
        assert_eq!(err.error_code(), "change.invalid_id");
    }

    #[test]
    fn change_already_exists_code() {
        let err = LocalProviderError::ChangeAlreadyExists {
            change_id: "demo".to_string(),
        };
        assert_eq!(err.error_code(), "change.already_exists");
    }

    #[test]
    fn change_not_found_code() {
        let err = LocalProviderError::ChangeNotFound {
            change_id: "missing".to_string(),
        };
        assert_eq!(err.error_code(), "change.not_found");
    }

    #[test]
    fn artifact_already_exists_code() {
        let err = LocalProviderError::ArtifactAlreadyExists {
            kind: "design".to_string(),
            change_id: "demo".to_string(),
        };
        assert_eq!(err.error_code(), "artifact.already_exists");
    }

    #[test]
    fn missing_capability_code() {
        let err = LocalProviderError::MissingCapability;
        assert_eq!(err.error_code(), "artifact.missing_capability");
    }

    #[test]
    fn invalid_capability_code() {
        let err = LocalProviderError::InvalidCapability {
            capability: "Bad-Name".to_string(),
        };
        assert_eq!(err.error_code(), "artifact.invalid_capability");
    }

    #[test]
    fn io_error_code_is_internal() {
        let err = LocalProviderError::Io(std::io::Error::other("boom"));
        assert_eq!(err.error_code(), "internal.error");
    }
}
