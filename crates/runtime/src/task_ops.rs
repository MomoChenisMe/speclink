//! `speclink task list / done / undo` 的 runtime entry points。
//!
//! 對齊 `apply-task-ops` capability 與 design.md `task.done` auto-trigger contract。
//! tasks.md parser 依 1-based 行內順序 index；done/undo 寫檔走 A2 既有 atomic rename。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use speclink_provider::{
    ChangeState, ChangeStateView, ProviderError, StateMachineStore, StateTransitionReason,
    TransitionRequest,
};
use speclink_provider_local::LocalStateMachineStore;
use tempfile::NamedTempFile;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::{artifact_root, resolve_state_root};
use crate::state_machine::{AllTasksDoneOutcome, ReviewPolicy, all_tasks_done_outcome};

/// 單一 task 行的解析結果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskItem {
    /// 1-based 行內順序 index。
    pub index: usize,
    pub done: bool,
    pub text: String,
}

/// `task.list` 成功時的 data payload。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskListData {
    pub tasks: Vec<TaskItem>,
}

/// `task.done` 成功時的 data payload。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDoneData {
    pub index: usize,
    pub done: bool,
    pub all_tasks_done: bool,
    pub state: ChangeState,
    pub auto_transitioned: bool,
}

/// `task.undo` 成功時的 data payload。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskUndoData {
    pub index: usize,
    pub done: bool,
    pub all_tasks_done: bool,
    pub state: ChangeState,
    /// 若從 `code_reviewing` 退回 `in_progress`，填 `"code_reviewing"`；否則 None。
    pub reverted_from: Option<String>,
}

/// Task 相關 runtime entry。
pub struct TaskOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> TaskOperations<G> {
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_state_store(
        &self,
        working_dir: &Path,
    ) -> Result<LocalStateMachineStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalStateMachineStore::new(state_root))
    }

    /// `task list --change <id>`。
    pub fn list(&self, working_dir: &Path, change: &str) -> Result<TaskListData, RuntimeError> {
        let tasks_path = tasks_path(working_dir, change);
        let content = read_tasks_or_err(&tasks_path, change)?;
        Ok(TaskListData {
            tasks: parse_checkbox_lines(&content),
        })
    }

    /// `task done <index> --change <id>`。
    pub async fn done(
        &self,
        working_dir: &Path,
        change: &str,
        index: usize,
    ) -> Result<TaskDoneData, RuntimeError> {
        let store = self.build_state_store(working_dir)?;
        let view = store
            .get_change_state(change)
            .await
            .map_err(map_provider_error)?;
        ensure_task_state_allows_done(view.state)?;

        let tasks_path = tasks_path(working_dir, change);
        let content = read_tasks_or_err(&tasks_path, change)?;
        let tasks = parse_checkbox_lines(&content);
        let total = tasks.len();
        let target = tasks
            .iter()
            .find(|t| t.index == index)
            .ok_or(RuntimeError::TaskIndexOutOfRange { index, total })?;
        let target_done = target.done;

        if target_done {
            // Idempotent no-op: just return current snapshot.
            return Ok(TaskDoneData {
                index,
                done: true,
                all_tasks_done: view.all_tasks_done,
                state: view.state,
                auto_transitioned: false,
            });
        }

        // Rewrite line: mark [ ] → [x].
        let new_content = toggle_task_line(&content, index, true);
        atomic_write(&tasks_path, new_content.as_bytes())?;

        // Re-parse to check if all tasks now done.
        let updated_tasks = parse_checkbox_lines(&new_content);
        let now_all_done = !updated_tasks.is_empty() && updated_tasks.iter().all(|t| t.done);

        if !now_all_done {
            return Ok(TaskDoneData {
                index,
                done: true,
                all_tasks_done: view.all_tasks_done, // unchanged
                state: view.state,
                auto_transitioned: false,
            });
        }

        // All tasks complete: apply walking-skeleton auto-trigger contract.
        let policy = ReviewPolicy::walking_skeleton();
        match all_tasks_done_outcome(policy) {
            AllTasksDoneOutcome::SetAllTasksDoneFlagOnly => {
                let new_view = store
                    .set_all_tasks_done(change, view.version, true)
                    .await
                    .map_err(map_provider_error)?;
                Ok(TaskDoneData {
                    index,
                    done: true,
                    all_tasks_done: new_view.all_tasks_done,
                    state: new_view.state,
                    auto_transitioned: false,
                })
            }
            AllTasksDoneOutcome::TransitionToCodeReviewing => {
                let new_view = store
                    .transition_state(
                        change,
                        view.version,
                        TransitionRequest {
                            to_state: ChangeState::CodeReviewing,
                            actor: None,
                            reason: StateTransitionReason::TaskDoneAuto,
                        },
                    )
                    .await
                    .map_err(map_provider_error)?;
                // Walking-skeleton path won't take this branch; reserved for review slice.
                let final_view = store
                    .set_all_tasks_done(change, new_view.version, true)
                    .await
                    .map_err(map_provider_error)?;
                Ok(TaskDoneData {
                    index,
                    done: true,
                    all_tasks_done: final_view.all_tasks_done,
                    state: final_view.state,
                    auto_transitioned: true,
                })
            }
        }
    }

    /// `task undo <index> --change <id>`。
    pub async fn undo(
        &self,
        working_dir: &Path,
        change: &str,
        index: usize,
    ) -> Result<TaskUndoData, RuntimeError> {
        let store = self.build_state_store(working_dir)?;
        let view = store
            .get_change_state(change)
            .await
            .map_err(map_provider_error)?;
        ensure_task_state_allows_undo(view.state)?;

        let tasks_path = tasks_path(working_dir, change);
        let content = read_tasks_or_err(&tasks_path, change)?;
        let tasks = parse_checkbox_lines(&content);
        let total = tasks.len();
        let target = tasks
            .iter()
            .find(|t| t.index == index)
            .ok_or(RuntimeError::TaskIndexOutOfRange { index, total })?;

        let mut reverted_from: Option<String> = None;
        let mut current_view: ChangeStateView = view.clone();

        // Step 1: if state is code_reviewing, transition back to in_progress FIRST.
        if current_view.state == ChangeState::CodeReviewing {
            current_view = store
                .transition_state(
                    change,
                    current_view.version,
                    TransitionRequest {
                        to_state: ChangeState::InProgress,
                        actor: None,
                        reason: StateTransitionReason::TaskUndoRevert,
                    },
                )
                .await
                .map_err(map_provider_error)?;
            reverted_from = Some("code_reviewing".to_string());
        }

        // Step 2: clear all_tasks_done flag if it was set.
        if current_view.all_tasks_done {
            current_view = store
                .set_all_tasks_done(change, current_view.version, false)
                .await
                .map_err(map_provider_error)?;
        }

        // Step 3: if task is already unmarked, skip filesystem write (idempotent).
        if !target.done {
            return Ok(TaskUndoData {
                index,
                done: false,
                all_tasks_done: current_view.all_tasks_done,
                state: current_view.state,
                reverted_from,
            });
        }

        // Step 4: rewrite tasks.md.
        let new_content = toggle_task_line(&content, index, false);
        atomic_write(&tasks_path, new_content.as_bytes())?;

        Ok(TaskUndoData {
            index,
            done: false,
            all_tasks_done: current_view.all_tasks_done,
            state: current_view.state,
            reverted_from,
        })
    }
}

/// 從 tasks.md 內容 parse 出所有 checkbox 行；忽略非 checkbox 行。
///
/// Regex 等價：`^(\s*)- \[( |x)\] (.+)$`。case-sensitive：大寫 `X` 不算 done。
#[must_use]
pub fn parse_checkbox_lines(content: &str) -> Vec<TaskItem> {
    use regex_lite::Regex;

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^(\s*)- \[( |x)\] (.+)$").unwrap());
    let mut out = Vec::new();
    let mut idx: usize = 0;
    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            idx += 1;
            let marker = caps.get(2).map_or(" ", |m| m.as_str());
            let text = caps.get(3).map_or("", |m| m.as_str()).to_string();
            out.push(TaskItem {
                index: idx,
                done: marker == "x",
                text,
            });
        }
    }
    out
}

fn toggle_task_line(content: &str, target_index: usize, mark_done: bool) -> String {
    use regex_lite::Regex;

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^(\s*)- \[( |x)\] (.+)$").unwrap());
    let mut idx: usize = 0;
    let mut out = String::with_capacity(content.len());
    let needs_trailing_newline = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = re.captures(line) {
            idx += 1;
            if idx == target_index {
                let indent = caps.get(1).map_or("", |m| m.as_str());
                let text = caps.get(3).map_or("", |m| m.as_str());
                let marker = if mark_done { 'x' } else { ' ' };
                out.push_str(&format!("{indent}- [{marker}] {text}"));
            } else {
                out.push_str(line);
            }
        } else {
            out.push_str(line);
        }
        if i + 1 < total || needs_trailing_newline {
            out.push('\n');
        }
    }
    out
}

fn tasks_path(working_dir: &Path, change: &str) -> PathBuf {
    artifact_root(working_dir)
        .join("changes")
        .join(change)
        .join("tasks.md")
}

fn read_tasks_or_err(path: &Path, change: &str) -> Result<String, RuntimeError> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(RuntimeError::TaskNoTasksFile {
            change: change.to_string(),
        }),
        Err(e) => Err(RuntimeError::Internal(format!(
            "read tasks.md ({}): {e}",
            path.display()
        ))),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    let dir = path
        .parent()
        .ok_or_else(|| RuntimeError::Internal("tasks.md has no parent dir".into()))?;
    fs::create_dir_all(dir)
        .map_err(|e| RuntimeError::Internal(format!("ensure tasks dir: {e}")))?;
    let mut tmp =
        NamedTempFile::new_in(dir).map_err(|e| RuntimeError::Internal(format!("tempfile: {e}")))?;
    use std::io::Write;
    tmp.write_all(bytes)
        .map_err(|e| RuntimeError::Internal(format!("write tempfile: {e}")))?;
    tmp.persist(path)
        .map_err(|e| RuntimeError::Internal(format!("persist tempfile: {e}")))?;
    Ok(())
}

fn ensure_task_state_allows_done(state: ChangeState) -> Result<(), RuntimeError> {
    match state {
        ChangeState::InProgress | ChangeState::CodeReviewing => Ok(()),
        other => Err(RuntimeError::TaskOpStateInvalid {
            op: "task.done".to_string(),
            current_state: other.as_str().to_string(),
            allowed: "in_progress, code_reviewing".to_string(),
        }),
    }
}

fn ensure_task_state_allows_undo(state: ChangeState) -> Result<(), RuntimeError> {
    match state {
        ChangeState::InProgress | ChangeState::CodeReviewing | ChangeState::Ready => Ok(()),
        other => Err(RuntimeError::TaskOpStateInvalid {
            op: "task.undo".to_string(),
            current_state: other.as_str().to_string(),
            allowed: "in_progress, code_reviewing, ready".to_string(),
        }),
    }
}

fn map_provider_error(err: ProviderError) -> RuntimeError {
    match err {
        ProviderError::ChangeNotFound { name } => RuntimeError::ChangeNotFound { name },
        ProviderError::StateInvalidValue { value } => RuntimeError::StateInvalidValue { value },
        ProviderError::StateTransitionInvalid { from, to } => {
            RuntimeError::StateTransitionInvalid { from, to }
        }
        ProviderError::StateVersionConflict { current_version } => {
            RuntimeError::StateVersionConflict { current_version }
        }
        ProviderError::StateDbSchemaInvalid { found, supported } => {
            RuntimeError::StateDbSchemaInvalid { found, supported }
        }
        other => RuntimeError::Provider(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_matches_top_level_checkboxes() {
        let content = "# Tasks\n- [ ] one\n- [x] two\n";
        let tasks = parse_checkbox_lines(content);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].index, 1);
        assert!(!tasks[0].done);
        assert_eq!(tasks[0].text, "one");
        assert_eq!(tasks[1].index, 2);
        assert!(tasks[1].done);
        assert_eq!(tasks[1].text, "two");
    }

    #[test]
    fn parser_matches_indented_checkboxes_with_1_based_index() {
        let content = "- [ ] outer\n  - [x] nested\n- [ ] another\n";
        let tasks = parse_checkbox_lines(content);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[1].text, "nested");
        assert!(tasks[1].done);
    }

    #[test]
    fn parser_rejects_uppercase_x_marker() {
        let content = "- [X] uppercase\n- [ ] normal\n";
        let tasks = parse_checkbox_lines(content);
        assert_eq!(tasks.len(), 1, "uppercase X SHALL be skipped");
        assert_eq!(tasks[0].text, "normal");
    }

    #[test]
    fn parser_ignores_prose_and_headings() {
        let content = "# Heading\nSome prose.\n- [ ] real task\n  not a task\n";
        let tasks = parse_checkbox_lines(content);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].text, "real task");
    }

    #[test]
    fn toggle_marks_specific_index_done_and_preserves_others() {
        let content = "- [ ] one\n- [ ] two\n- [ ] three\n";
        let updated = toggle_task_line(content, 2, true);
        assert_eq!(updated, "- [ ] one\n- [x] two\n- [ ] three\n");
    }

    #[test]
    fn toggle_unmarks_specific_index_and_preserves_others() {
        let content = "- [x] one\n- [x] two\n- [x] three\n";
        let updated = toggle_task_line(content, 1, false);
        assert_eq!(updated, "- [ ] one\n- [x] two\n- [x] three\n");
    }

    #[test]
    fn toggle_preserves_indent_and_trailing_newline() {
        let content = "  - [ ] indented\n";
        let updated = toggle_task_line(content, 1, true);
        assert_eq!(updated, "  - [x] indented\n");
    }
}
