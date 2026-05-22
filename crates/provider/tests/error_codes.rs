//! 5 個 slice A3 新 error code 常量 + `ProviderError` variant → code 對應的 integration 測試。
//!
//! 對應 spec requirement「New error codes SHALL be registered with stable exit codes」
//! 的 provider 層 fixture。CLI 層 exit code 映射由 `crates/cli` 自己的測試覆蓋。

use speclink_provider::{ProviderError, codes};

#[test]
fn slice_a3_codes_are_dot_separated_namespace_strings() {
    assert_eq!(codes::STATE_INVALID_VALUE, "state.invalid_value");
    assert_eq!(codes::STATE_TRANSITION_INVALID, "state.transition_invalid");
    assert_eq!(codes::STATE_VERSION_CONFLICT, "state.version_conflict");
    assert_eq!(codes::STATE_DB_SCHEMA_INVALID, "state.db.schema_invalid");
    assert_eq!(codes::CHANGE_DAG_INCOMPLETE, "change.dag_incomplete");
}

#[test]
fn provider_error_variants_map_to_slice_a3_codes() {
    assert_eq!(
        ProviderError::StateInvalidValue {
            value: "garbage".into()
        }
        .code(),
        codes::STATE_INVALID_VALUE
    );
    assert_eq!(
        ProviderError::StateTransitionInvalid {
            from: "proposing".into(),
            to: "in_progress".into()
        }
        .code(),
        codes::STATE_TRANSITION_INVALID
    );
    assert_eq!(
        ProviderError::StateVersionConflict { current_version: 3 }.code(),
        codes::STATE_VERSION_CONFLICT
    );
    assert_eq!(
        ProviderError::StateDbSchemaInvalid {
            found: 3,
            supported: 2
        }
        .code(),
        codes::STATE_DB_SCHEMA_INVALID
    );
    assert_eq!(
        ProviderError::ChangeDagIncomplete {
            missing: vec!["proposal.md".into(), "tasks.md".into()],
        }
        .code(),
        codes::CHANGE_DAG_INCOMPLETE
    );
}

#[test]
fn only_state_version_conflict_is_retryable_among_slice_a3_codes() {
    assert!(ProviderError::StateVersionConflict { current_version: 1 }.retryable());
    assert!(!ProviderError::StateInvalidValue { value: "x".into() }.retryable());
    assert!(
        !ProviderError::StateTransitionInvalid {
            from: "proposing".into(),
            to: "in_progress".into()
        }
        .retryable()
    );
    assert!(
        !ProviderError::StateDbSchemaInvalid {
            found: 3,
            supported: 2
        }
        .retryable()
    );
    assert!(
        !ProviderError::ChangeDagIncomplete {
            missing: vec!["p".into()]
        }
        .retryable()
    );
}

// ----- slice A4 (`add-archive`) -----

#[test]
fn slice_a4_codes_are_dot_separated_namespace_strings() {
    assert_eq!(codes::CHANGE_TASKS_INCOMPLETE, "change.tasks_incomplete");
    assert_eq!(
        codes::VALIDATION_ARCHIVE_FAILED,
        "validation.archive_failed"
    );
    assert_eq!(codes::ARCHIVE_SPECS_SKIPPED, "archive.specs_skipped");
}

#[test]
fn provider_error_variants_map_to_slice_a4_codes() {
    assert_eq!(
        ProviderError::ChangeTasksIncomplete {
            change_id: "demo".into(),
        }
        .code(),
        codes::CHANGE_TASKS_INCOMPLETE
    );
    assert_eq!(
        ProviderError::ValidationArchiveFailed {
            reason: "stub".into(),
        }
        .code(),
        codes::VALIDATION_ARCHIVE_FAILED
    );
}

#[test]
fn slice_a4_error_variants_are_not_retryable() {
    // 兩個 A4 error 都是「修正後重試」非「立即重試」，retryable=false。
    assert!(
        !ProviderError::ChangeTasksIncomplete {
            change_id: "demo".into()
        }
        .retryable()
    );
    assert!(!ProviderError::ValidationArchiveFailed { reason: "x".into() }.retryable());
}

#[test]
fn archive_specs_skipped_is_warning_carrier_not_provider_error() {
    // ARCHIVE_SPECS_SKIPPED 是 warning carrier code、不對應任何 ProviderError variant；
    // CLI envelope 在 success path 的 warnings array 內帶這個 code、不走 error path。
    // 此測試僅斷言常量字串本身存在、不期待 ProviderError 表面有對應 variant。
    let _ = codes::ARCHIVE_SPECS_SKIPPED;
}
