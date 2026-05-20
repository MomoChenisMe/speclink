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

    /// Change 處於不可 archive 狀態（已 archived、目標目錄已存在等）。
    #[error("change cannot be archived: {reason}")]
    ChangeNotArchivable {
        /// 人類可讀的原因。
        reason: String,
    },

    /// Spec delta 套用衝突。
    #[error(
        "spec delta conflict for capability '{capability}': requirement '{requirement}' ({operation})"
    )]
    SpecDeltaConflict {
        /// 觸發衝突的 capability 名稱。
        capability: String,
        /// 衝突的 requirement 名稱。
        requirement: String,
        /// 觸發衝突的 heading 操作。
        operation: &'static str,
    },

    /// Spec delta 格式錯誤。
    #[error("spec delta parse error for capability '{capability}': {message}")]
    SpecDeltaParseError {
        /// 觸發解析錯誤的 capability 名稱。
        capability: String,
        /// 解析失敗描述。
        message: String,
    },

    /// Target artifact 檔案不存在（如對沒寫 tasks.md 的 change 呼叫 `task done`）。
    #[error("artifact '{artifact_id}' is missing for change '{change_id}'")]
    ArtifactMissing {
        /// 缺少的 artifact id（如 `"tasks"`、`"spec:user-auth"`）。
        artifact_id: String,
        /// 目標 change id。
        change_id: String,
    },

    /// Task id 不符合 `^\d+\.\d+$` 格式。
    #[error("invalid task id: '{task_id}'")]
    TaskInvalidId {
        /// 不合法的 task id 原始字串。
        task_id: String,
    },

    /// Task id 在 `tasks.md` 中找不到對應 checkbox。
    #[error("task '{task_id}' not found")]
    TaskNotFound {
        /// 缺少的 task id。
        task_id: String,
    },

    /// `tasks.md` 解析失敗（如缺 section heading、出現三層 task id 等）。
    #[error("tasks.md parse error: {message}")]
    TasksParseError {
        /// 解析失敗描述。
        message: String,
    },

    /// archive 流程步驟 5-7 失敗且 rollback 本身亦失敗的最後手段；
    /// 訊息列出殘留檔案路徑供人工修復。
    #[error(
        "archive rollback failed; manual recovery required (leftover .tmp: {tmp_files:?}; leftover .bak: {backup_files:?}; cause: {source})"
    )]
    RollbackFailed {
        /// 未能清除的 `.tmp` 檔案 POSIX 路徑列表。
        tmp_files: Vec<String>,
        /// 未能還原的 `.bak` 檔案 POSIX 路徑列表。
        backup_files: Vec<String>,
        /// 原始觸發失敗的錯誤。
        source: Box<LocalProviderError>,
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
            LocalProviderError::ChangeNotArchivable { .. } => "archive.change_not_archivable",
            LocalProviderError::SpecDeltaConflict { .. } => "spec.delta_conflict",
            LocalProviderError::SpecDeltaParseError { .. } => "spec.delta_parse_error",
            LocalProviderError::ArtifactMissing { .. } => "artifact.missing",
            LocalProviderError::TaskInvalidId { .. } => "task.invalid_id",
            LocalProviderError::TaskNotFound { .. } => "task.not_found",
            LocalProviderError::TasksParseError { .. } => "tasks.parse_error",
            LocalProviderError::Io(_)
            | LocalProviderError::Json(_)
            | LocalProviderError::StateDb(_)
            | LocalProviderError::RollbackFailed { .. }
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

    #[test]
    fn change_not_archivable_code() {
        let err = LocalProviderError::ChangeNotArchivable {
            reason: "already archived".to_string(),
        };
        assert_eq!(err.error_code(), "archive.change_not_archivable");
    }

    #[test]
    fn spec_delta_conflict_code() {
        let err = LocalProviderError::SpecDeltaConflict {
            capability: "auth".to_string(),
            requirement: "User login".to_string(),
            operation: "ADDED",
        };
        assert_eq!(err.error_code(), "spec.delta_conflict");
    }

    #[test]
    fn spec_delta_parse_error_code() {
        let err = LocalProviderError::SpecDeltaParseError {
            capability: "auth".to_string(),
            message: "unknown heading".to_string(),
        };
        assert_eq!(err.error_code(), "spec.delta_parse_error");
    }

    #[test]
    fn artifact_missing_code() {
        let err = LocalProviderError::ArtifactMissing {
            artifact_id: "tasks".to_string(),
            change_id: "demo".to_string(),
        };
        assert_eq!(err.error_code(), "artifact.missing");
    }

    #[test]
    fn task_invalid_id_code() {
        let err = LocalProviderError::TaskInvalidId {
            task_id: "1.1.2".to_string(),
        };
        assert_eq!(err.error_code(), "task.invalid_id");
    }

    #[test]
    fn task_not_found_code() {
        let err = LocalProviderError::TaskNotFound {
            task_id: "1.99".to_string(),
        };
        assert_eq!(err.error_code(), "task.not_found");
    }

    #[test]
    fn tasks_parse_error_code() {
        let err = LocalProviderError::TasksParseError {
            message: "missing heading".to_string(),
        };
        assert_eq!(err.error_code(), "tasks.parse_error");
    }

    #[test]
    fn rollback_failed_code_is_internal() {
        let err = LocalProviderError::RollbackFailed {
            tmp_files: vec![".speclink/specs/auth/spec.md.tmp".to_string()],
            backup_files: vec![".speclink/specs/auth/spec.md.bak".to_string()],
            source: Box::new(LocalProviderError::Internal {
                message: "boom".to_string(),
            }),
        };
        assert_eq!(err.error_code(), "internal.error");
    }
}
