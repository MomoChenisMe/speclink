//! Provider 錯誤型別與 declared error code 字串常量。

use thiserror::Error;

/// SpecLink declared error codes (dot-separated namespace).
///
/// 這些常量是 stable identifier，跨 change 不可隨意重命名。
pub mod codes {
    /// Working directory 不在 git working tree 內。
    pub const REQUIRES_GIT: &str = "project.requires_git";
    /// `.speclink/link.yaml` 已存在，且未提供 `--force`。
    pub const ALREADY_INITIALIZED: &str = "project.already_initialized";
    /// `.speclink/link.yaml` 不存在，無法執行 status/link/unlink。
    pub const NOT_INITIALIZED: &str = "project.not_initialized";
    /// `link <id>` 時 state.db 內無對應 project row。
    pub const LINK_TARGET_NOT_FOUND: &str = "project.link_target_not_found";
}

/// Provider 層的錯誤型別。
///
/// `code()` 對應 SpecLink JSON envelope 中的 `error.code`；CLI 層會把這個
/// code 對應到 process exit code。
#[derive(Debug, Error)]
pub enum ProviderError {
    /// 對應 `project.requires_git`。
    #[error("not inside a git working tree: {context}")]
    RequiresGit { context: String },

    /// 對應 `project.already_initialized`。
    #[error("project already initialized at {path}")]
    AlreadyInitialized { path: String },

    /// 對應 `project.not_initialized`。
    #[error("project is not initialized: {path}")]
    NotInitialized { path: String },

    /// 對應 `project.link_target_not_found`。
    #[error("link target project_id `{project_id}` not found in state.db")]
    LinkTargetNotFound { project_id: String },

    /// 內部 I/O / SQLite / YAML / 其他底層錯誤；CLI 層映射為通用 exit code 1。
    #[error("provider internal error: {0}")]
    Internal(String),
}

impl ProviderError {
    /// 對應的 declared error code 字串常量。
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            ProviderError::RequiresGit { .. } => codes::REQUIRES_GIT,
            ProviderError::AlreadyInitialized { .. } => codes::ALREADY_INITIALIZED,
            ProviderError::NotInitialized { .. } => codes::NOT_INITIALIZED,
            ProviderError::LinkTargetNotFound { .. } => codes::LINK_TARGET_NOT_FOUND,
            ProviderError::Internal(_) => "internal.error",
        }
    }

    /// 是否屬於使用者可重試的錯誤；目前所有 declared error 皆 non-retryable。
    #[must_use]
    pub fn retryable(&self) -> bool {
        false
    }
}
