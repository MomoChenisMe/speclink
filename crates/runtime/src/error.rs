//! Runtime 錯誤型別與 doctor finding code 字串常量。

use thiserror::Error;

pub use speclink_provider::codes;

/// Doctor finding codes (reserved by this change; check logic lives in
/// the future `add-doctor` change).
pub mod finding_codes {
    /// 對應 `project.requires_git` 的 doctor 檢查 finding（未實作邏輯）。
    pub const PROJECT_REQUIRES_GIT: &str = "doctor.project.requires_git";
    /// state.db 不存在；後續 `add-state-recovery` 會把這個 finding 標為 `auto_fixable=true`。
    pub const STATE_DB_MISSING: &str = "doctor.state.db_missing";
    /// state.db 檔案損毀（無法 open / SQLite 報 malformed）。
    pub const STATE_DB_CORRUPTED: &str = "doctor.state.db_corrupted";
    /// state.db schema version 與 SpecLink binary 預期不符。
    pub const STATE_DB_SCHEMA_INVALID: &str = "doctor.state.db_schema_invalid";
}

/// Runtime 層的錯誤型別。
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// 對應 `project.requires_git`：working dir 不是 git working tree、或 git CLI 不可用。
    #[error("not inside a git working tree: {context}")]
    RequiresGit { context: String },

    /// 對應 `project.already_initialized`。
    #[error("project already initialized at {path}")]
    AlreadyInitialized { path: String },

    /// 對應 `project.not_initialized`。
    #[error("project is not initialized at {path}")]
    NotInitialized { path: String },

    /// 對應 `project.link_target_not_found`。
    #[error("link target project_id `{project_id}` not found in state.db")]
    LinkTargetNotFound { project_id: String },

    /// 透過 provider 傳上來的內部錯誤。
    #[error("provider error: {0}")]
    Provider(#[from] speclink_provider::ProviderError),

    /// 其他內部 I/O / process 錯誤。
    #[error("runtime internal error: {0}")]
    Internal(String),
}

impl RuntimeError {
    /// 對應的 declared error code 字串。
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::RequiresGit { .. } => codes::REQUIRES_GIT,
            RuntimeError::AlreadyInitialized { .. } => codes::ALREADY_INITIALIZED,
            RuntimeError::NotInitialized { .. } => codes::NOT_INITIALIZED,
            RuntimeError::LinkTargetNotFound { .. } => codes::LINK_TARGET_NOT_FOUND,
            RuntimeError::Provider(p) => p.code(),
            RuntimeError::Internal(_) => "internal.error",
        }
    }

    /// 對應的 process exit code（與 spec「SpecLink CLI exit codes follow a fixed mapping」對齊）。
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self.code() {
            c if c == codes::REQUIRES_GIT
                || c == codes::NOT_INITIALIZED
                || c == codes::LINK_TARGET_NOT_FOUND =>
            {
                2
            }
            c if c == codes::ALREADY_INITIALIZED => 7,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_codes_match_declared_namespace() {
        assert_eq!(
            finding_codes::PROJECT_REQUIRES_GIT,
            "doctor.project.requires_git"
        );
        assert_eq!(finding_codes::STATE_DB_MISSING, "doctor.state.db_missing");
        assert_eq!(
            finding_codes::STATE_DB_CORRUPTED,
            "doctor.state.db_corrupted"
        );
        assert_eq!(
            finding_codes::STATE_DB_SCHEMA_INVALID,
            "doctor.state.db_schema_invalid"
        );
    }

    #[test]
    fn exit_code_mapping_matches_spec_table() {
        assert_eq!(
            RuntimeError::RequiresGit {
                context: "x".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::NotInitialized { path: "p".into() }.exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::LinkTargetNotFound {
                project_id: "u".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::AlreadyInitialized { path: "p".into() }.exit_code(),
            7
        );
    }
}
