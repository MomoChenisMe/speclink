//! Transition-table matrix tests for the 6-state lifecycle.
//!
//! 對應 spec requirement「State machine SHALL enforce the legal transition table」
//! 「Change lifecycle SHALL define exactly six legal states」「Walking-skeleton mode
//! SHALL hard-code both review flags to `false`」與決策「Walking-skeleton review-flag
//! 預設值：硬編 `false`，不讀 config」。

use speclink_provider::{ChangeState, StateTransitionReason};
use speclink_runtime::{
    AllTasksDoneOutcome, ReviewPolicy, all_tasks_done_outcome, is_legal_transition,
    legal_transitions, proposing_target,
};

const ALL_STATES: [ChangeState; 6] = [
    ChangeState::Proposing,
    ChangeState::Reviewing,
    ChangeState::Ready,
    ChangeState::InProgress,
    ChangeState::CodeReviewing,
    ChangeState::Archived,
];

const ALL_REASONS: [StateTransitionReason; 8] = [
    StateTransitionReason::ApplyStart,
    StateTransitionReason::ApplyPause,
    StateTransitionReason::TaskDoneAuto,
    StateTransitionReason::TaskUndoRevert,
    StateTransitionReason::ArtifactDagComplete,
    StateTransitionReason::ReviewApprovedArtifact,
    StateTransitionReason::ReviewRejectedCode,
    StateTransitionReason::ArchiveRun,
];

#[test]
fn transition_matrix_covers_six_source_states_with_legal_and_illegal_outcomes() {
    // Confirm 6 source states × 6 target states × 8 reasons = 288 combos.
    // For each combo, is_legal_transition is true iff present in legal_transitions().
    let table = legal_transitions();
    for &from in &ALL_STATES {
        for &to in &ALL_STATES {
            for &reason in &ALL_REASONS {
                let in_table = table
                    .iter()
                    .any(|(f, t, r)| *f == from && *t == to && *r == reason);
                let result = is_legal_transition(from, to, reason);
                assert_eq!(
                    result, in_table,
                    "mismatch for ({from:?}, {to:?}, {reason:?})"
                );
            }
        }
    }
}

#[test]
fn transition_table_includes_all_design_required_pairs() {
    // The minimum pairs from design.md §6.2 transition table.
    let must_include = [
        (
            ChangeState::Proposing,
            ChangeState::Reviewing,
            StateTransitionReason::ArtifactDagComplete,
        ),
        (
            ChangeState::Proposing,
            ChangeState::Ready,
            StateTransitionReason::ArtifactDagComplete,
        ),
        (
            ChangeState::Reviewing,
            ChangeState::Ready,
            StateTransitionReason::ReviewApprovedArtifact,
        ),
        (
            ChangeState::Ready,
            ChangeState::InProgress,
            StateTransitionReason::ApplyStart,
        ),
        (
            ChangeState::InProgress,
            ChangeState::Ready,
            StateTransitionReason::ApplyPause,
        ),
        (
            ChangeState::InProgress,
            ChangeState::CodeReviewing,
            StateTransitionReason::TaskDoneAuto,
        ),
        (
            ChangeState::CodeReviewing,
            ChangeState::InProgress,
            StateTransitionReason::TaskUndoRevert,
        ),
        (
            ChangeState::CodeReviewing,
            ChangeState::InProgress,
            StateTransitionReason::ReviewRejectedCode,
        ),
        (
            ChangeState::CodeReviewing,
            ChangeState::Archived,
            StateTransitionReason::ArchiveRun,
        ),
        (
            ChangeState::InProgress,
            ChangeState::Archived,
            StateTransitionReason::ArchiveRun,
        ),
    ];
    for (f, t, r) in must_include {
        assert!(
            is_legal_transition(f, t, r),
            "design.md transition ({f:?}, {t:?}, {r:?}) SHALL be legal"
        );
    }
}

#[test]
fn transition_table_rejects_proposing_skipping_states() {
    // proposing cannot skip to in_progress / code_reviewing / archived directly.
    for to in [
        ChangeState::InProgress,
        ChangeState::CodeReviewing,
        ChangeState::Archived,
    ] {
        for &reason in &ALL_REASONS {
            assert!(
                !is_legal_transition(ChangeState::Proposing, to, reason),
                "proposing → {to:?} via {reason:?} SHALL be illegal"
            );
        }
    }
}

#[test]
fn transition_table_rejects_archived_outbound() {
    // archived is terminal; no legal transition leaves it.
    for &to in &ALL_STATES {
        for &reason in &ALL_REASONS {
            assert!(
                !is_legal_transition(ChangeState::Archived, to, reason),
                "archived → {to:?} via {reason:?} SHALL be illegal"
            );
        }
    }
}

#[test]
fn walking_skeleton_proposing_dag_complete_walks_to_ready() {
    let p = ReviewPolicy::walking_skeleton();
    assert_eq!(proposing_target(p), ChangeState::Ready);
}

#[test]
fn walking_skeleton_all_tasks_done_keeps_in_progress_and_sets_flag_only() {
    let p = ReviewPolicy::walking_skeleton();
    assert_eq!(
        all_tasks_done_outcome(p),
        AllTasksDoneOutcome::SetAllTasksDoneFlagOnly
    );
}

#[test]
fn review_flags_enabled_diverts_to_review_states() {
    let p = ReviewPolicy {
        require_artifact_review: true,
        require_code_review: true,
    };
    assert_eq!(proposing_target(p), ChangeState::Reviewing);
    assert_eq!(
        all_tasks_done_outcome(p),
        AllTasksDoneOutcome::TransitionToCodeReviewing
    );
}
