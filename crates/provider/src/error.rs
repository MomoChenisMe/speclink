//! Provider 錯誤型別與 declared error code 字串常量。

use thiserror::Error;

use crate::types::Etag;

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

    /// `change` 表內找不到名稱對應的 row。
    pub const CHANGE_NOT_FOUND: &str = "change.not_found";
    /// `new change` 名稱已存在。
    pub const CHANGE_DUPLICATE_NAME: &str = "change.duplicate_name";
    /// `new change` 名稱不符 grammar 或長度；或 `delete change` 缺/錯 `--confirm-name`。
    pub const CHANGE_INVALID_NAME: &str = "change.invalid_name";
    /// `--kind` 不在白名單；或 `--capability` 不符 grammar。
    pub const ARTIFACT_KIND_INVALID: &str = "artifact.kind_invalid";
    /// `--kind spec` 缺 `--capability`。
    pub const ARTIFACT_CAPABILITY_REQUIRED: &str = "artifact.capability_required";
    /// `artifact.read` 對應檔案不存在；或新建路徑卻帶了 non-null `--expected-etag`。
    pub const ARTIFACT_NOT_FOUND: &str = "artifact.not_found";
    /// `artifact.write` 並發衝突：覆寫缺 etag、etag 不符、或既檔存在卻被當新建。
    pub const ARTIFACT_VERSION_CONFLICT: &str = "artifact.version_conflict";

    // ----- slice A3 (`add-state-machine-and-apply`) -----

    /// `change.state` 欄位讀到 6 個合法 enum 之外的值；視為資料庫毀損 / 不變式破壞。
    pub const STATE_INVALID_VALUE: &str = "state.invalid_value";
    /// 請求的 transition 不在合法 transition table 內。
    pub const STATE_TRANSITION_INVALID: &str = "state.transition_invalid";
    /// `change.version` compare-and-swap 失敗：他人已 mutate。
    pub const STATE_VERSION_CONFLICT: &str = "state.version_conflict";
    /// state.db `_migrations` 最高 version 超過本 binary 支援的範圍；拒絕讀寫。
    pub const STATE_DB_SCHEMA_INVALID: &str = "state.db.schema_invalid";
    /// Change artifact DAG 未齊全（缺 proposal.md / tasks.md / specs/*）；
    /// 預留給未來 doctor slice manual override，本 slice 不暴露 CLI。
    pub const CHANGE_DAG_INCOMPLETE: &str = "change.dag_incomplete";
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

    /// 對應 `change.not_found`。
    #[error("change `{name}` not found in state.db")]
    ChangeNotFound { name: String },

    /// 對應 `change.duplicate_name`。
    #[error("change name `{name}` already exists")]
    ChangeDuplicateName { name: String },

    /// 對應 `change.invalid_name`。
    #[error("invalid change name `{name}`: {reason}")]
    ChangeInvalidName { name: String, reason: String },

    /// 對應 `artifact.kind_invalid`。
    #[error("invalid artifact kind `{kind}`")]
    ArtifactKindInvalid { kind: String },

    /// 對應 `artifact.capability_required`。
    #[error("artifact kind `spec` requires `--capability`")]
    ArtifactCapabilityRequired,

    /// 對應 `artifact.not_found`。
    #[error("artifact not found at {path}")]
    ArtifactNotFound { path: String },

    /// 對應 `artifact.version_conflict`。
    #[error("artifact version conflict (expected={expected:?}, actual={actual})")]
    ArtifactVersionConflict {
        expected: Option<Etag>,
        actual: Etag,
    },

    /// 對應 `state.invalid_value`。
    #[error("change.state column contains illegal value `{value}`")]
    StateInvalidValue { value: String },

    /// 對應 `state.transition_invalid`。
    #[error("state transition `{from} → {to}` is not permitted")]
    StateTransitionInvalid { from: String, to: String },

    /// 對應 `state.version_conflict`。`current_version` 是 store 端在 CAS 失敗時
    /// 觀察到的真實 version；caller 重讀後可重試。
    #[error("change.version compare-and-swap failed; current_version={current_version}")]
    StateVersionConflict { current_version: u64 },

    /// 對應 `state.db.schema_invalid`。
    #[error("state.db schema version {found} exceeds this binary's supported max {supported}")]
    StateDbSchemaInvalid { found: u32, supported: u32 },

    /// 對應 `change.dag_incomplete`。`missing` 列出缺失的 artifact 路徑（相對 change dir）。
    #[error("change artifact DAG incomplete; missing: {missing:?}")]
    ChangeDagIncomplete { missing: Vec<String> },

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
            ProviderError::ChangeNotFound { .. } => codes::CHANGE_NOT_FOUND,
            ProviderError::ChangeDuplicateName { .. } => codes::CHANGE_DUPLICATE_NAME,
            ProviderError::ChangeInvalidName { .. } => codes::CHANGE_INVALID_NAME,
            ProviderError::ArtifactKindInvalid { .. } => codes::ARTIFACT_KIND_INVALID,
            ProviderError::ArtifactCapabilityRequired => codes::ARTIFACT_CAPABILITY_REQUIRED,
            ProviderError::ArtifactNotFound { .. } => codes::ARTIFACT_NOT_FOUND,
            ProviderError::ArtifactVersionConflict { .. } => codes::ARTIFACT_VERSION_CONFLICT,
            ProviderError::StateInvalidValue { .. } => codes::STATE_INVALID_VALUE,
            ProviderError::StateTransitionInvalid { .. } => codes::STATE_TRANSITION_INVALID,
            ProviderError::StateVersionConflict { .. } => codes::STATE_VERSION_CONFLICT,
            ProviderError::StateDbSchemaInvalid { .. } => codes::STATE_DB_SCHEMA_INVALID,
            ProviderError::ChangeDagIncomplete { .. } => codes::CHANGE_DAG_INCOMPLETE,
            ProviderError::Internal(_) => "internal.error",
        }
    }

    /// 是否屬於使用者可重試的錯誤。
    ///
    /// `ArtifactVersionConflict` / `StateVersionConflict` 兩個 CAS 衝突 retryable：
    /// caller 重讀最新 etag / version 後可重試。
    #[must_use]
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            ProviderError::ArtifactVersionConflict { .. }
                | ProviderError::StateVersionConflict { .. }
        )
    }
}
