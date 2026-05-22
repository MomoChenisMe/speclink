//! 6-state lifecycle 的 transition table、walking-skeleton `ReviewPolicy`、
//! actor 推導 fallback chain，對應 design §6.2 / §6.3 / §16.7。
//!
//! 本模組為純邏輯層（不接 disk / network）；CAS 與 audit insert 由
//! `LocalStateMachineStore` 負責。

#![allow(clippy::doc_markdown)]

use speclink_provider::{Actor, ChangeState, StateTransitionReason};

/// review optionality flag（design §6.3）。walking-skeleton MVP 硬編 false / false。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewPolicy {
    /// `proposing → reviewing` 是否強制走 review；false 直接走 `proposing → ready`。
    pub require_artifact_review: bool,
    /// `in_progress → code_reviewing` 是否強制走 review；false 直接設 `all_tasks_done=1`。
    pub require_code_review: bool,
}

impl ReviewPolicy {
    /// walking-skeleton MVP 政策：兩個 flag 都硬編 false。
    ///
    /// 對應決策「Walking-skeleton review-flag 預設值：硬編 `false`，不讀 config」。
    /// 未來 `add-config-rw` slice 接通後改為 `from_config(&config)`，transition table 不變。
    #[must_use]
    pub const fn walking_skeleton() -> Self {
        Self {
            require_artifact_review: false,
            require_code_review: false,
        }
    }
}

/// 透過 `policy` 計算 `proposing` 在 DAG 齊全後應推進到的下一 state。
#[must_use]
pub const fn proposing_target(policy: ReviewPolicy) -> ChangeState {
    if policy.require_artifact_review {
        ChangeState::Reviewing
    } else {
        ChangeState::Ready
    }
}

/// 透過 `policy` 計算 `in_progress` 在所有 task done 後應做的動作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllTasksDoneOutcome {
    /// `require_code_review=true`：transition 到 `code_reviewing`。
    TransitionToCodeReviewing,
    /// `require_code_review=false`：state 不變，僅設 `all_tasks_done=1`。
    SetAllTasksDoneFlagOnly,
}

#[must_use]
pub const fn all_tasks_done_outcome(policy: ReviewPolicy) -> AllTasksDoneOutcome {
    if policy.require_code_review {
        AllTasksDoneOutcome::TransitionToCodeReviewing
    } else {
        AllTasksDoneOutcome::SetAllTasksDoneFlagOnly
    }
}

/// 6-state lifecycle 的合法 transition table（design §6.2）。
///
/// `(from, to, reason)` 三元組；任何不在此表的 transition 都 SHALL 被
/// `RuntimeError::StateTransitionInvalid` 拒絕。
#[must_use]
pub fn legal_transitions() -> &'static [(ChangeState, ChangeState, StateTransitionReason)] {
    &[
        // proposing → reviewing/ready：DAG 齊全後 artifact.write hook trigger
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
        // reviewing → ready：future review slice
        (
            ChangeState::Reviewing,
            ChangeState::Ready,
            StateTransitionReason::ReviewApprovedArtifact,
        ),
        // ready ⇌ in_progress：apply.start / apply.pause
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
        // in_progress → code_reviewing：task.done auto-trigger (require_code_review=true)
        (
            ChangeState::InProgress,
            ChangeState::CodeReviewing,
            StateTransitionReason::TaskDoneAuto,
        ),
        // code_reviewing → in_progress：task.undo OR review.reject (future)
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
        // code_reviewing / in_progress → archived：archive.run（A4 落實）
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
        // archived → in_progress：archive.run 在 post-commit rename 失敗時的 best-effort revert
        // 路徑。對應 design 「Filesystem rename SHALL happen after SQLite transaction commit」
        // 與「best-effort revert」契約。僅由 `LocalArchiveStore::archive_change` 寫入；其他
        // caller 試圖以此 reason 進來都會被 archive_change 之外的呼叫鏈拒絕。
        (
            ChangeState::Archived,
            ChangeState::InProgress,
            StateTransitionReason::ArchiveRunRevert,
        ),
    ]
}

/// 檢查 `(from, to, reason)` 是否在合法 transition table 內。
#[must_use]
pub fn is_legal_transition(
    from: ChangeState,
    to: ChangeState,
    reason: StateTransitionReason,
) -> bool {
    legal_transitions()
        .iter()
        .any(|(f, t, r)| *f == from && *t == to && *r == reason)
}

/// 推導 actor：`agent_host` 由 `--actor` flag 帶入時直接用；省略時 env `SPECLINK_AGENT_HOST`，
/// 仍缺則 fallback `cli`。`os_user` 來自 `whoami` username lookup；fallback `unknown`。
/// `host_id` 來自 `whoami` hostname lookup；fallback `unknown`。
///
/// 對應 spec requirement「Actor SHALL be resolved by fallback chain when `--actor` flag is omitted」。
#[must_use]
pub fn resolve_actor(explicit_agent_host: Option<&str>) -> Actor {
    let agent_host = match explicit_agent_host {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => std::env::var("SPECLINK_AGENT_HOST")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "cli".to_string()),
    };
    Actor {
        agent_host,
        os_user: resolve_os_user(),
        host_id: resolve_host_id(),
    }
}

fn resolve_os_user() -> String {
    let raw = whoami::username();
    if raw.trim().is_empty() {
        "unknown".to_string()
    } else {
        raw
    }
}

fn resolve_host_id() -> String {
    let raw = whoami::fallible::hostname().unwrap_or_default();
    if raw.trim().is_empty() {
        "unknown".to_string()
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_skeleton_policy_hard_codes_false_false() {
        let p = ReviewPolicy::walking_skeleton();
        assert!(!p.require_artifact_review);
        assert!(!p.require_code_review);
    }

    #[test]
    fn proposing_target_walks_to_ready_under_skeleton() {
        let p = ReviewPolicy::walking_skeleton();
        assert_eq!(proposing_target(p), ChangeState::Ready);
    }

    #[test]
    fn proposing_target_walks_to_reviewing_when_artifact_review_required() {
        let p = ReviewPolicy {
            require_artifact_review: true,
            require_code_review: false,
        };
        assert_eq!(proposing_target(p), ChangeState::Reviewing);
    }

    #[test]
    fn all_tasks_done_outcome_under_skeleton_keeps_in_progress() {
        let p = ReviewPolicy::walking_skeleton();
        assert_eq!(
            all_tasks_done_outcome(p),
            AllTasksDoneOutcome::SetAllTasksDoneFlagOnly
        );
    }

    #[test]
    fn all_tasks_done_outcome_with_code_review_transitions_to_code_reviewing() {
        let p = ReviewPolicy {
            require_artifact_review: false,
            require_code_review: true,
        };
        assert_eq!(
            all_tasks_done_outcome(p),
            AllTasksDoneOutcome::TransitionToCodeReviewing
        );
    }

    #[test]
    fn legal_transitions_table_has_expected_entries() {
        let table = legal_transitions();
        assert!(
            table.len() >= 10,
            "table SHALL list at least 10 transitions"
        );
        // sanity: each entry's reason is unique-enough per pair
        for (f, t, _) in table {
            assert_ne!(f, t, "no self-loops in legal table");
        }
    }

    #[test]
    fn is_legal_transition_accepts_apply_start() {
        assert!(is_legal_transition(
            ChangeState::Ready,
            ChangeState::InProgress,
            StateTransitionReason::ApplyStart,
        ));
    }

    #[test]
    fn is_legal_transition_accepts_apply_pause() {
        assert!(is_legal_transition(
            ChangeState::InProgress,
            ChangeState::Ready,
            StateTransitionReason::ApplyPause,
        ));
    }

    #[test]
    fn is_legal_transition_accepts_dag_complete_to_ready_and_reviewing() {
        assert!(is_legal_transition(
            ChangeState::Proposing,
            ChangeState::Ready,
            StateTransitionReason::ArtifactDagComplete,
        ));
        assert!(is_legal_transition(
            ChangeState::Proposing,
            ChangeState::Reviewing,
            StateTransitionReason::ArtifactDagComplete,
        ));
    }

    #[test]
    fn is_legal_transition_rejects_proposing_direct_to_in_progress() {
        assert!(!is_legal_transition(
            ChangeState::Proposing,
            ChangeState::InProgress,
            StateTransitionReason::ApplyStart,
        ));
    }

    #[test]
    fn is_legal_transition_rejects_wrong_reason_for_legal_pair() {
        // ready → in_progress is legal only with reason=apply_start
        assert!(!is_legal_transition(
            ChangeState::Ready,
            ChangeState::InProgress,
            StateTransitionReason::TaskDoneAuto,
        ));
    }

    #[test]
    fn resolve_actor_uses_explicit_agent_host_when_supplied() {
        let actor = resolve_actor(Some("claude-code"));
        assert_eq!(actor.agent_host, "claude-code");
    }

    #[test]
    fn resolve_actor_falls_back_to_cli_when_no_env_and_no_arg() {
        // Snapshot then clear env around the call.
        let prev = std::env::var("SPECLINK_AGENT_HOST").ok();
        // SAFETY (post-2024 edition): test runs single-threaded for this assertion;
        // we restore the env before returning to avoid pollution.
        unsafe {
            std::env::remove_var("SPECLINK_AGENT_HOST");
        }
        let actor = resolve_actor(None);
        if let Some(v) = prev {
            unsafe {
                std::env::set_var("SPECLINK_AGENT_HOST", v);
            }
        }
        assert_eq!(actor.agent_host, "cli");
    }

    #[test]
    fn resolve_actor_treats_empty_agent_host_arg_as_omitted() {
        let prev = std::env::var("SPECLINK_AGENT_HOST").ok();
        unsafe {
            std::env::remove_var("SPECLINK_AGENT_HOST");
        }
        let actor = resolve_actor(Some(""));
        if let Some(v) = prev {
            unsafe {
                std::env::set_var("SPECLINK_AGENT_HOST", v);
            }
        }
        assert_eq!(actor.agent_host, "cli");
    }

    #[test]
    fn resolve_actor_os_user_and_host_id_are_never_empty() {
        let actor = resolve_actor(Some("any"));
        assert!(
            !actor.os_user.is_empty(),
            "os_user SHALL fall back to literal"
        );
        assert!(
            !actor.host_id.is_empty(),
            "host_id SHALL fall back to literal"
        );
    }

    // ----- slice A4 (`add-archive`) transition table coverage -----

    #[test]
    fn is_legal_transition_accepts_in_progress_to_archived_with_archive_run() {
        assert!(is_legal_transition(
            ChangeState::InProgress,
            ChangeState::Archived,
            StateTransitionReason::ArchiveRun,
        ));
    }

    #[test]
    fn is_legal_transition_accepts_code_reviewing_to_archived_with_archive_run() {
        // A4 spec：code_reviewing → archived 在 table 內存在（review slice 後接通），
        // 但 archive_change 本 slice 的 state guard 仍會拒絕 code_reviewing 進來。
        assert!(is_legal_transition(
            ChangeState::CodeReviewing,
            ChangeState::Archived,
            StateTransitionReason::ArchiveRun,
        ));
    }

    #[test]
    fn is_legal_transition_rejects_archived_back_with_wrong_reason() {
        // archived → in_progress 只能由 archive_run_revert 觸發，apply_pause / 任何其他
        // reason 都應被拒絕。
        for r in [
            StateTransitionReason::ApplyStart,
            StateTransitionReason::ApplyPause,
            StateTransitionReason::TaskUndoRevert,
            StateTransitionReason::ArchiveRun,
        ] {
            assert!(
                !is_legal_transition(ChangeState::Archived, ChangeState::InProgress, r,),
                "archived → in_progress SHALL only allow archive_run_revert, got reason {r:?}"
            );
        }
    }

    #[test]
    fn is_legal_transition_accepts_archive_run_revert_path() {
        assert!(is_legal_transition(
            ChangeState::Archived,
            ChangeState::InProgress,
            StateTransitionReason::ArchiveRunRevert,
        ));
    }

    #[test]
    fn archive_run_is_only_legal_writer_of_archived() {
        // 任何 reason 不是 ArchiveRun 試圖把 state 寫成 archived 都不在合法表內。
        for f in [
            ChangeState::Proposing,
            ChangeState::Reviewing,
            ChangeState::Ready,
            ChangeState::InProgress,
            ChangeState::CodeReviewing,
        ] {
            for r in [
                StateTransitionReason::ApplyStart,
                StateTransitionReason::ApplyPause,
                StateTransitionReason::TaskDoneAuto,
                StateTransitionReason::TaskUndoRevert,
                StateTransitionReason::ArtifactDagComplete,
                StateTransitionReason::ReviewApprovedArtifact,
                StateTransitionReason::ReviewRejectedCode,
                StateTransitionReason::ArchiveRunRevert,
            ] {
                assert!(
                    !is_legal_transition(f, ChangeState::Archived, r),
                    "({f:?}, archived, {r:?}) SHALL NOT be legal"
                );
            }
        }
    }
}
