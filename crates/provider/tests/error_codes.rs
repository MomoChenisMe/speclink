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

// ----- slice A5 (`add-config-rw`) -----

#[test]
fn slice_a5_codes_are_dot_separated_namespace_strings() {
    assert_eq!(codes::CONFIG_NOT_FOUND, "config.not_found");
    assert_eq!(codes::CONFIG_MALFORMED, "config.malformed");
    assert_eq!(codes::CONFIG_KEY_NOT_FOUND, "config.key_not_found");
    assert_eq!(codes::STATE_ETAG_MISMATCH, "state.etag_mismatch");
}

#[test]
fn provider_error_variants_map_to_slice_a5_codes() {
    assert_eq!(
        ProviderError::ConfigNotFound {
            path: ".speclink/config.yaml".into()
        }
        .code(),
        codes::CONFIG_NOT_FOUND
    );
    assert_eq!(
        ProviderError::ConfigMalformed {
            reason: "parse err".into()
        }
        .code(),
        codes::CONFIG_MALFORMED
    );
    assert_eq!(
        ProviderError::ConfigKeyNotFound {
            key: "rules.unknown".into()
        }
        .code(),
        codes::CONFIG_KEY_NOT_FOUND
    );
    assert_eq!(
        ProviderError::StateEtagMismatch {
            expected: Some("v1.abc".into()),
            actual: "v1.xyz".into()
        }
        .code(),
        codes::STATE_ETAG_MISMATCH
    );
}

#[test]
fn only_state_etag_mismatch_is_retryable_among_slice_a5_codes() {
    // `state.etag_mismatch` 是 CAS 衝突的同類：caller 重讀後可重試。
    assert!(
        ProviderError::StateEtagMismatch {
            expected: None,
            actual: "v1.x".into()
        }
        .retryable()
    );
    assert!(!ProviderError::ConfigNotFound { path: "p".into() }.retryable());
    assert!(!ProviderError::ConfigMalformed { reason: "r".into() }.retryable());
    assert!(!ProviderError::ConfigKeyNotFound { key: "k".into() }.retryable());
}

// ----- slice polish-config-error-messages -----

#[test]
fn state_etag_mismatch_display_does_not_leak_rust_debug_some_wrapper() {
    // polish-config-error-messages spec scenario「`state.etag_mismatch` message does
    // not leak Rust Debug formatting」：Display 輸出 SHALL NOT 包含 `Some(`，並 SHALL
    // 含 expected / actual 兩條 etag 字面。
    let e = ProviderError::StateEtagMismatch {
        expected: Some("v1.aaa".into()),
        actual: "v2.bbb".into(),
    };
    let msg = format!("{e}");
    assert!(
        !msg.contains("Some("),
        "Display SHALL NOT leak `Some(...)` wrapper: {msg}"
    );
    assert!(msg.contains("v1.aaa"), "missing expected etag: {msg}");
    assert!(msg.contains("v2.bbb"), "missing actual etag: {msg}");
}

#[test]
fn state_etag_mismatch_display_renders_none_as_explicit_marker() {
    // `expected: None` 時 Display SHALL NOT 印 Rust `None` 關鍵字、SHALL 用 `<none>`
    // 顯式 marker（avoid leaking Debug 表示）。
    let e = ProviderError::StateEtagMismatch {
        expected: None,
        actual: "v3.ccc".into(),
    };
    let msg = format!("{e}");
    assert!(
        !msg.contains("None"),
        "Display SHALL NOT contain `None`: {msg}"
    );
    assert!(
        msg.contains("<none>"),
        "Display SHALL render absent expected as `<none>`: {msg}"
    );
    assert!(msg.contains("v3.ccc"), "missing actual etag: {msg}");
}

#[test]
fn provider_error_config_edit_mode_required_has_correct_code() {
    // 「New error codes SHALL be registered with stable exit codes」requirement
    // 對新 code `config.edit_mode_required` 成立：code() 字串穩定、retryable=false。
    // exit_code mapping 由 `crates/runtime/src/error.rs` 的 unit test 覆蓋。
    assert_eq!(
        codes::CONFIG_EDIT_MODE_REQUIRED,
        "config.edit_mode_required"
    );
    assert_eq!(
        ProviderError::ConfigEditModeRequired.code(),
        codes::CONFIG_EDIT_MODE_REQUIRED
    );
    assert!(!ProviderError::ConfigEditModeRequired.retryable());
}

#[test]
fn warning_carrier_codes_for_config_are_declared() {
    // A5 兩條 audit-only warning code（不對應 ProviderError variant）。
    assert_eq!(
        codes::CONFIG_EXTERNAL_EDIT_DETECTED,
        "config.external_edit_detected"
    );
    assert_eq!(
        codes::CONFIG_MALFORMED_USING_DEFAULTS,
        "config.malformed_using_defaults"
    );
}
