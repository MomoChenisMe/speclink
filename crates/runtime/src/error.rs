//! Runtime 錯誤型別與 doctor finding code 字串常量。

use serde::{Deserialize, Serialize};
use thiserror::Error;

use speclink_provider::Etag;

/// Runtime 層回給 caller 的 warning entry。CLI 層會把它包到 JSON envelope 的 `warnings` 陣列。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
pub use speclink_provider::codes;

/// slice A3 task-layer 專屬 error codes（不屬於 provider 的 ProviderError 範圍）。
pub mod task_codes {
    pub const TASK_NO_TASKS_FILE: &str = "task.no_tasks_file";
    pub const TASK_INDEX_OUT_OF_RANGE: &str = "task.index_out_of_range";
}

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
    /// 對應 `project.requires_git`。
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

    /// 對應 `state.transition_invalid`，但是「task / lifecycle operation」在錯誤 state 下被呼叫
    /// 的特殊變體。避免把 operation 名（`task.done` / `task.undo`）誤塞進 `to` 欄位產生
    /// 「`proposing → task.done` is not permitted」這種把 op 名洩漏成虛構 state 的訊息。
    ///
    /// 對外 error code 與 exit code 與 `StateTransitionInvalid` 完全一致，僅訊息正名。
    #[error(
        "operation `{op}` not allowed in state `{current_state}`; requires state in {{{allowed}}}"
    )]
    TaskOpStateInvalid {
        op: String,
        current_state: String,
        allowed: String,
    },

    /// 對應 `state.version_conflict`。
    #[error("change.version compare-and-swap failed; current_version={current_version}")]
    StateVersionConflict { current_version: u64 },

    /// 對應 `state.db.schema_invalid`。
    #[error("state.db schema version {found} exceeds this binary's supported max {supported}")]
    StateDbSchemaInvalid { found: u32, supported: u32 },

    /// 對應 `change.dag_incomplete`。
    #[error("change artifact DAG incomplete; missing: {missing:?}")]
    ChangeDagIncomplete { missing: Vec<String> },

    /// 對應 `task.no_tasks_file`：對應 change 內 tasks.md 不存在。
    #[error("tasks.md not found for change `{change}`")]
    TaskNoTasksFile { change: String },

    /// 對應 `task.index_out_of_range`。
    #[error("task index {index} out of range (only {total} task lines)")]
    TaskIndexOutOfRange { index: usize, total: usize },

    /// 對應 `change.tasks_incomplete`（A4）：archive.run 對 `in_progress` 但 tasks 未全完成。
    #[error("change `{change_id}` has incomplete tasks; cannot archive")]
    ChangeTasksIncomplete { change_id: String },

    /// 對應 `validation.archive_failed`（A4 reserved）：本 slice 不會 emit；
    /// 預留給後續 `add-analyze` slice 接通 validation hook。
    #[error("archive-time validation failed: {reason}")]
    ValidationArchiveFailed { reason: String },

    /// 對應 `config.not_found`（A5）。
    #[error("config.yaml not found at {path}")]
    ConfigNotFound { path: String },

    /// 對應 `config.malformed`（A5）。
    #[error("config content malformed: {reason}")]
    ConfigMalformed { reason: String },

    /// 對應 `config.key_not_found`（A5；polish-config-error-messages 加 `hint`）。
    ///
    /// `key` SHALL 保留 user 原始輸入字面字串（不可被診斷理由覆蓋）；`hint` 為
    /// 「: wildcards / filters not supported」之類診斷後綴（無 hint 時為空字串）。
    /// Display format「`config key `{key}` not found{hint}`」。
    #[error("config key `{key}` not found{hint}")]
    ConfigKeyNotFound { key: String, hint: String },

    /// 對應 `state.etag_mismatch`（A5；polish-config-error-messages Display polish）。
    /// Display SHALL 把 `expected` 走純字串、無值時印 `<none>`。
    #[error("config etag mismatch (expected={}, actual={actual})", expected.as_deref().unwrap_or("<none>"))]
    StateEtagMismatch {
        expected: Option<String>,
        actual: String,
    },

    /// 對應 `config.edit_mode_required`（polish-config-error-messages）：CLI `config edit`
    /// 三條輸入路徑皆缺；envelope `error.message` 含 `--stdin` 與 `$EDITOR` 字面字串。
    #[error("`speclink config edit` requires --stdin, --editor <cmd>, or $EDITOR to be set")]
    ConfigEditModeRequired,

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
            RuntimeError::ChangeNotFound { .. } => codes::CHANGE_NOT_FOUND,
            RuntimeError::ChangeDuplicateName { .. } => codes::CHANGE_DUPLICATE_NAME,
            RuntimeError::ChangeInvalidName { .. } => codes::CHANGE_INVALID_NAME,
            RuntimeError::ArtifactKindInvalid { .. } => codes::ARTIFACT_KIND_INVALID,
            RuntimeError::ArtifactCapabilityRequired => codes::ARTIFACT_CAPABILITY_REQUIRED,
            RuntimeError::ArtifactNotFound { .. } => codes::ARTIFACT_NOT_FOUND,
            RuntimeError::ArtifactVersionConflict { .. } => codes::ARTIFACT_VERSION_CONFLICT,
            RuntimeError::StateInvalidValue { .. } => codes::STATE_INVALID_VALUE,
            RuntimeError::StateTransitionInvalid { .. }
            | RuntimeError::TaskOpStateInvalid { .. } => codes::STATE_TRANSITION_INVALID,
            RuntimeError::StateVersionConflict { .. } => codes::STATE_VERSION_CONFLICT,
            RuntimeError::StateDbSchemaInvalid { .. } => codes::STATE_DB_SCHEMA_INVALID,
            RuntimeError::ChangeDagIncomplete { .. } => codes::CHANGE_DAG_INCOMPLETE,
            RuntimeError::TaskNoTasksFile { .. } => task_codes::TASK_NO_TASKS_FILE,
            RuntimeError::TaskIndexOutOfRange { .. } => task_codes::TASK_INDEX_OUT_OF_RANGE,
            RuntimeError::ChangeTasksIncomplete { .. } => codes::CHANGE_TASKS_INCOMPLETE,
            RuntimeError::ValidationArchiveFailed { .. } => codes::VALIDATION_ARCHIVE_FAILED,
            RuntimeError::ConfigNotFound { .. } => codes::CONFIG_NOT_FOUND,
            RuntimeError::ConfigMalformed { .. } => codes::CONFIG_MALFORMED,
            RuntimeError::ConfigKeyNotFound { .. } => codes::CONFIG_KEY_NOT_FOUND,
            RuntimeError::StateEtagMismatch { .. } => codes::STATE_ETAG_MISMATCH,
            RuntimeError::ConfigEditModeRequired => codes::CONFIG_EDIT_MODE_REQUIRED,
            RuntimeError::Provider(p) => p.code(),
            RuntimeError::Internal(_) => "internal.error",
        }
    }

    /// 是否屬於使用者可重試的錯誤。
    ///
    /// 與 `ProviderError::retryable` 對齊：CAS 衝突類錯誤 retryable。
    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            RuntimeError::ArtifactVersionConflict { .. }
            | RuntimeError::StateVersionConflict { .. }
            | RuntimeError::StateEtagMismatch { .. } => true,
            RuntimeError::Provider(p) => p.retryable(),
            _ => false,
        }
    }

    /// 對應的 process exit code（與 spec「SpecLink CLI exit codes follow a fixed mapping」對齊）。
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self.code() {
            c if c == codes::REQUIRES_GIT
                || c == codes::NOT_INITIALIZED
                || c == codes::LINK_TARGET_NOT_FOUND
                || c == codes::CHANGE_NOT_FOUND
                || c == codes::CHANGE_INVALID_NAME
                || c == codes::ARTIFACT_KIND_INVALID
                || c == codes::ARTIFACT_CAPABILITY_REQUIRED
                || c == codes::ARTIFACT_NOT_FOUND
                || c == codes::CHANGE_DAG_INCOMPLETE
                || c == codes::CHANGE_TASKS_INCOMPLETE
                || c == codes::CONFIG_NOT_FOUND
                || c == codes::CONFIG_KEY_NOT_FOUND
                || c == codes::CONFIG_EDIT_MODE_REQUIRED
                || c == task_codes::TASK_NO_TASKS_FILE
                || c == task_codes::TASK_INDEX_OUT_OF_RANGE =>
            {
                2
            }
            c if c == codes::VALIDATION_ARCHIVE_FAILED || c == codes::CONFIG_MALFORMED => 3,
            c if c == codes::ALREADY_INITIALIZED
                || c == codes::CHANGE_DUPLICATE_NAME
                || c == codes::ARTIFACT_VERSION_CONFLICT
                || c == codes::STATE_TRANSITION_INVALID
                || c == codes::STATE_VERSION_CONFLICT
                || c == codes::STATE_ETAG_MISMATCH =>
            {
                7
            }
            // STATE_INVALID_VALUE / STATE_DB_SCHEMA_INVALID / internal.error → 1
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_provider::Etag;

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
        // bootstrap-slice
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

        // slice-A
        assert_eq!(
            RuntimeError::ChangeNotFound { name: "x".into() }.exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::ChangeDuplicateName { name: "x".into() }.exit_code(),
            7
        );
        assert_eq!(
            RuntimeError::ChangeInvalidName {
                name: "x".into(),
                reason: "y".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::ArtifactKindInvalid {
                kind: "summary".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(RuntimeError::ArtifactCapabilityRequired.exit_code(), 2);
        assert_eq!(
            RuntimeError::ArtifactNotFound { path: "p".into() }.exit_code(),
            2
        );
        assert_eq!(
            RuntimeError::ArtifactVersionConflict {
                expected: None,
                actual: Etag::from_bytes(b""),
            }
            .exit_code(),
            7
        );
    }

    #[test]
    fn config_edit_mode_required_maps_to_exit_2_and_not_retryable() {
        // polish-config-error-messages slice：新 variant code 字串穩定、exit code 2、
        // retryable=false。對齊 spec scenario「`config edit` without input mode emits
        // `config.edit_mode_required`」。
        let e = RuntimeError::ConfigEditModeRequired;
        assert_eq!(e.code(), codes::CONFIG_EDIT_MODE_REQUIRED);
        assert_eq!(e.code(), "config.edit_mode_required");
        assert_eq!(e.exit_code(), 2);
        assert!(!e.retryable());
        let msg = e.to_string();
        assert!(msg.contains("--stdin"), "message missing --stdin: {msg}");
        assert!(msg.contains("$EDITOR"), "message missing $EDITOR: {msg}");
    }

    #[test]
    fn task_op_state_invalid_maps_to_same_code_and_exit_as_state_transition_invalid() {
        // B2 regression：新 variant 與 StateTransitionInvalid 共用 envelope 表面契約，
        // 但訊息正名為「operation X not allowed in state Y」，不再洩漏 op 名到 to 欄位。
        let e = RuntimeError::TaskOpStateInvalid {
            op: "task.done".into(),
            current_state: "proposing".into(),
            allowed: "in_progress, code_reviewing".into(),
        };
        assert_eq!(e.code(), codes::STATE_TRANSITION_INVALID);
        assert_eq!(e.exit_code(), 7);
        assert!(!e.retryable());
        let msg = e.to_string();
        assert!(msg.contains("operation `task.done`"), "got: {msg}");
        assert!(msg.contains("state `proposing`"), "got: {msg}");
        assert!(msg.contains("in_progress, code_reviewing"), "got: {msg}");
        assert!(
            !msg.contains("→"),
            "must not use transition-arrow shape, got: {msg}"
        );
    }

    #[test]
    fn code_method_covers_slice_a_variants() {
        assert_eq!(
            RuntimeError::ChangeNotFound { name: "x".into() }.code(),
            codes::CHANGE_NOT_FOUND
        );
        assert_eq!(
            RuntimeError::ArtifactCapabilityRequired.code(),
            codes::ARTIFACT_CAPABILITY_REQUIRED
        );
        assert_eq!(
            RuntimeError::ArtifactVersionConflict {
                expected: None,
                actual: Etag::from_bytes(b""),
            }
            .code(),
            codes::ARTIFACT_VERSION_CONFLICT
        );
    }
}
