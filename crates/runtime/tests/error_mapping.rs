//! Exit code mapping snapshot test for slice A3 error codes.
//!
//! 對應 spec requirement「New error codes SHALL be registered with stable exit codes」
//! 的 runtime 層 fixture：state.invalid_value→1、state.transition_invalid→7、
//! state.version_conflict→7、state.db.schema_invalid→1、change.dag_incomplete→2、
//! task.no_tasks_file→2、task.index_out_of_range→2。

use speclink_runtime::{RuntimeError, codes, task_codes};

#[test]
fn slice_a3_state_codes_map_to_expected_exit_codes() {
    assert_eq!(
        RuntimeError::StateInvalidValue { value: "x".into() }.code(),
        codes::STATE_INVALID_VALUE
    );
    assert_eq!(
        RuntimeError::StateInvalidValue { value: "x".into() }.exit_code(),
        1
    );

    assert_eq!(
        RuntimeError::StateTransitionInvalid {
            from: "proposing".into(),
            to: "in_progress".into()
        }
        .code(),
        codes::STATE_TRANSITION_INVALID
    );
    assert_eq!(
        RuntimeError::StateTransitionInvalid {
            from: "p".into(),
            to: "i".into()
        }
        .exit_code(),
        7
    );

    assert_eq!(
        RuntimeError::StateVersionConflict { current_version: 5 }.code(),
        codes::STATE_VERSION_CONFLICT
    );
    assert_eq!(
        RuntimeError::StateVersionConflict { current_version: 5 }.exit_code(),
        7
    );

    assert_eq!(
        RuntimeError::StateDbSchemaInvalid {
            found: 3,
            supported: 2
        }
        .code(),
        codes::STATE_DB_SCHEMA_INVALID
    );
    assert_eq!(
        RuntimeError::StateDbSchemaInvalid {
            found: 3,
            supported: 2
        }
        .exit_code(),
        1
    );

    assert_eq!(
        RuntimeError::ChangeDagIncomplete {
            missing: vec!["tasks.md".into()]
        }
        .code(),
        codes::CHANGE_DAG_INCOMPLETE
    );
    assert_eq!(
        RuntimeError::ChangeDagIncomplete {
            missing: vec!["tasks.md".into()]
        }
        .exit_code(),
        2
    );
}

#[test]
fn slice_a3_task_codes_map_to_exit_2() {
    assert_eq!(
        RuntimeError::TaskNoTasksFile {
            change: "demo".into()
        }
        .code(),
        task_codes::TASK_NO_TASKS_FILE
    );
    assert_eq!(
        RuntimeError::TaskNoTasksFile {
            change: "demo".into()
        }
        .exit_code(),
        2
    );

    assert_eq!(
        RuntimeError::TaskIndexOutOfRange {
            index: 99,
            total: 5
        }
        .code(),
        task_codes::TASK_INDEX_OUT_OF_RANGE
    );
    assert_eq!(
        RuntimeError::TaskIndexOutOfRange {
            index: 99,
            total: 5
        }
        .exit_code(),
        2
    );
}

#[test]
fn slice_a3_state_version_conflict_is_retryable() {
    assert!(RuntimeError::StateVersionConflict { current_version: 1 }.retryable());
    assert!(
        !RuntimeError::StateTransitionInvalid {
            from: "p".into(),
            to: "i".into()
        }
        .retryable()
    );
    assert!(!RuntimeError::StateInvalidValue { value: "x".into() }.retryable());
    assert!(!RuntimeError::TaskNoTasksFile { change: "d".into() }.retryable());
}
