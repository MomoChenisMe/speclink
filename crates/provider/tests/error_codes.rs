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
