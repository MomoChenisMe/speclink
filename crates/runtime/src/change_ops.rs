//! `speclink new/list/show/delete change` 的 runtime entry points.
//!
//! 沿用 bootstrap `ops.rs::Operations<G>` 的「struct + GitProbe 泛型 + build_store helper」pattern；
//! 對 `LocalChangeStore` 包一層，加上 name validation 與 destructive confirm-name 校對。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use speclink_provider::{
    ChangeRow, ChangeState, ChangeStore, ProviderError, StateMachineStore, validate_kebab_id,
};
use speclink_provider_local::{LocalChangeStore, LocalStateMachineStore};

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::paths::{ARTIFACT_ROOT, resolve_state_root};

/// 預設 schema id（slice A 沒有 schema CLI 之前的 placeholder）。
pub const DEFAULT_SCHEMA_ID: &str = "spec-driven";

/// `change-store` 觀察到的單一 artifact 參照（用於 `speclink show change` 輸出）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRef {
    pub kind: String,
    pub capability: Option<String>,
}

/// `speclink show change` 回傳的完整 data shape。
///
/// `all_tasks_done` 直接讀 `change` row column（task_ops 維護）；
/// `next_actions` 由 [`compute_next_actions`] 依 state + tasks.md 算出。
/// 兩欄位永遠存在、永遠 additive（dogfood UX hint）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ShowChangeData {
    pub change: ChangeRow,
    pub artifacts: Vec<ArtifactRef>,
    pub all_tasks_done: bool,
    pub next_actions: Vec<String>,
}

/// Change CRUD 的 entry。
pub struct ChangeOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ChangeOperations<G> {
    /// 建立 handle。不接觸 disk。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalChangeStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalChangeStore::new(working_dir.to_path_buf(), state_root))
    }

    /// 建立新 change。
    ///
    /// # Errors
    /// `ChangeInvalidName` / `ChangeDuplicateName` / `RequiresGit` / `Internal`。
    pub async fn create_change(
        &self,
        working_dir: &Path,
        name: &str,
    ) -> Result<ChangeRow, RuntimeError> {
        if let Err(e) = validate_kebab_id(name) {
            return Err(RuntimeError::ChangeInvalidName {
                name: name.to_string(),
                reason: e.to_string(),
            });
        }
        let store = self.build_store(working_dir)?;
        store
            .create_change(name, DEFAULT_SCHEMA_ID)
            .await
            .map_err(map_provider_error_change)
    }

    /// 列舉所有 change。
    pub async fn list_changes(&self, working_dir: &Path) -> Result<Vec<ChangeRow>, RuntimeError> {
        let store = self.build_store(working_dir)?;
        store
            .list_changes()
            .await
            .map_err(map_provider_error_change)
    }

    /// 顯示單一 change metadata + 該 change 下既有 artifact 清單。
    ///
    /// envelope 含 `all_tasks_done`（讀 `change` 表 column）與 `next_actions`
    /// （state-driven 查表 + in_progress 掃 tasks.md 第一個 pending task id）。
    pub async fn show_change(
        &self,
        working_dir: &Path,
        name: &str,
    ) -> Result<ShowChangeData, RuntimeError> {
        let store = self.build_store(working_dir)?;
        let change = store
            .get_change(name)
            .await
            .map_err(map_provider_error_change)?;
        let dir = change_dir(working_dir, name);
        let artifacts = discover_artifacts(&dir)?;

        // all_tasks_done：從 state machine view 讀（task_ops 維護的同一 column）。
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        let sm = LocalStateMachineStore::new(state_root);
        let view = sm
            .get_change_state(name)
            .await
            .map_err(map_provider_error_change)?;
        let all_tasks_done = view.all_tasks_done;

        let next_actions = compute_next_actions(view.state, all_tasks_done, &dir);

        Ok(ShowChangeData {
            change,
            artifacts,
            all_tasks_done,
            next_actions,
        })
    }

    /// 刪除 change row + filesystem 目錄。
    ///
    /// `--confirm-name` 必須與 `name` 完全相符；否則回 `ChangeInvalidName`。
    pub async fn delete_change(
        &self,
        working_dir: &Path,
        name: &str,
        confirm_name: Option<&str>,
    ) -> Result<(), RuntimeError> {
        match confirm_name {
            Some(c) if c == name => {}
            Some(c) => {
                return Err(RuntimeError::ChangeInvalidName {
                    name: name.to_string(),
                    reason: format!("`--confirm-name {c}` does not match target name `{name}`"),
                });
            }
            None => {
                return Err(RuntimeError::ChangeInvalidName {
                    name: name.to_string(),
                    reason: "destructive delete requires `--confirm-name <name>`".into(),
                });
            }
        }
        let store = self.build_store(working_dir)?;
        store
            .delete_change(name)
            .await
            .map_err(map_provider_error_change)
    }
}

fn change_dir(working_dir: &Path, name: &str) -> PathBuf {
    working_dir.join(ARTIFACT_ROOT).join("changes").join(name)
}

fn discover_artifacts(change_dir: &Path) -> Result<Vec<ArtifactRef>, RuntimeError> {
    let mut out = Vec::new();
    for (kind, file) in [
        ("proposal", "proposal.md"),
        ("design", "design.md"),
        ("tasks", "tasks.md"),
    ] {
        if change_dir.join(file).is_file() {
            out.push(ArtifactRef {
                kind: kind.to_string(),
                capability: None,
            });
        }
    }
    let specs_dir = change_dir.join("specs");
    if specs_dir.is_dir() {
        let mut caps: Vec<String> = Vec::new();
        let entries = fs::read_dir(&specs_dir).map_err(|e| {
            RuntimeError::Internal(format!("read_dir {}: {e}", specs_dir.display()))
        })?;
        for entry in entries {
            let entry =
                entry.map_err(|e| RuntimeError::Internal(format!("read_dir entry: {e}")))?;
            let ft = entry
                .file_type()
                .map_err(|e| RuntimeError::Internal(format!("entry file_type: {e}")))?;
            if !ft.is_dir() {
                continue;
            }
            if entry.path().join("spec.md").is_file() {
                if let Ok(name) = entry.file_name().into_string() {
                    caps.push(name);
                }
            }
        }
        caps.sort();
        for cap in caps {
            out.push(ArtifactRef {
                kind: "spec".to_string(),
                capability: Some(cap),
            });
        }
    }
    Ok(out)
}

/// `ProviderError → RuntimeError` exhaustive mapping for change ops.
fn map_provider_error_change(err: ProviderError) -> RuntimeError {
    match err {
        ProviderError::RequiresGit { context } => RuntimeError::RequiresGit { context },
        ProviderError::AlreadyInitialized { path } => RuntimeError::AlreadyInitialized { path },
        ProviderError::NotInitialized { path } => RuntimeError::NotInitialized { path },
        ProviderError::LinkTargetNotFound { project_id } => {
            RuntimeError::LinkTargetNotFound { project_id }
        }
        ProviderError::ChangeNotFound { name } => RuntimeError::ChangeNotFound { name },
        ProviderError::ChangeDuplicateName { name } => RuntimeError::ChangeDuplicateName { name },
        ProviderError::ChangeInvalidName { name, reason } => {
            RuntimeError::ChangeInvalidName { name, reason }
        }
        ProviderError::ArtifactKindInvalid { kind } => RuntimeError::ArtifactKindInvalid { kind },
        ProviderError::ArtifactCapabilityRequired => RuntimeError::ArtifactCapabilityRequired,
        ProviderError::ArtifactNotFound { path } => RuntimeError::ArtifactNotFound { path },
        ProviderError::ArtifactVersionConflict { expected, actual } => {
            RuntimeError::ArtifactVersionConflict { expected, actual }
        }
        e @ (ProviderError::StateInvalidValue { .. }
        | ProviderError::StateTransitionInvalid { .. }
        | ProviderError::StateVersionConflict { .. }
        | ProviderError::StateDbSchemaInvalid { .. }
        | ProviderError::ChangeDagIncomplete { .. }) => RuntimeError::Provider(e),
        ProviderError::ChangeTasksIncomplete { change_id } => {
            RuntimeError::ChangeTasksIncomplete { change_id }
        }
        ProviderError::ValidationArchiveFailed { reason } => {
            RuntimeError::ValidationArchiveFailed { reason }
        }
        ProviderError::ConfigNotFound { path } => RuntimeError::ConfigNotFound { path },
        ProviderError::ConfigMalformed { reason } => RuntimeError::ConfigMalformed { reason },
        ProviderError::ConfigKeyNotFound { key } => RuntimeError::ConfigKeyNotFound {
            key,
            hint: String::new(),
        },
        ProviderError::StateEtagMismatch { expected, actual } => {
            RuntimeError::StateEtagMismatch { expected, actual }
        }
        ProviderError::ConfigEditModeRequired => RuntimeError::ConfigEditModeRequired,
        ProviderError::Internal(s) => RuntimeError::Internal(s),
    }
}

/// State-driven `next_actions` 查表 + in_progress 掃 tasks.md 取第一個 pending task id。
///
/// 對齊 specs/change-store Requirement「`change.show` response envelope SHALL include
/// `all_tasks_done` and `next_actions`」表格。
fn compute_next_actions(
    state: ChangeState,
    all_tasks_done: bool,
    change_dir: &Path,
) -> Vec<String> {
    match state {
        ChangeState::Proposing => {
            // 過濾掉已 done 的 artifact kind
            let mut out = Vec::new();
            for (kind, file) in [
                ("proposal", "proposal.md"),
                ("design", "design.md"),
                ("tasks", "tasks.md"),
            ] {
                if !change_dir.join(file).is_file() {
                    out.push(format!("artifact.write {kind}"));
                }
            }
            out
        }
        ChangeState::Reviewing | ChangeState::CodeReviewing => {
            vec!["review.approve".to_string(), "review.reject".to_string()]
        }
        ChangeState::Ready => vec!["apply.start".to_string()],
        ChangeState::InProgress => {
            if all_tasks_done {
                vec!["archive.run".to_string()]
            } else {
                // tasks.md 不存在 → 給 bare hint；存在 → reuse task_ops 既有 parser
                // 取第一個 `!done` 的 1-based sequential index（與 `task.done <INDEX>` CLI 同源）。
                match first_pending_task_index(change_dir) {
                    Some(index) => vec![format!("task.done {index}")],
                    None => vec!["task.done".to_string()],
                }
            }
        }
        ChangeState::Archived => Vec::new(),
    }
}

/// 用 `task_ops::parse_checkbox_lines` 算 tasks.md 第一個 `!done` checkbox 的 1-based index。
///
/// 找不到 tasks.md、或檔內無 unchecked 行 → `None`。emit 的 index 與
/// `speclink task done <INDEX>` CLI 用的 index 完全等價（同一 parser）。
fn first_pending_task_index(change_dir: &Path) -> Option<usize> {
    let text = fs::read_to_string(change_dir.join("tasks.md")).ok()?;
    crate::task_ops::parse_checkbox_lines(&text)
        .into_iter()
        .find(|item| !item.done)
        .map(|item| item.index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_tasks(content: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("tasks.md"), content).unwrap();
        tmp
    }

    #[test]
    fn first_pending_index_returns_first_unchecked_position() {
        // 3 個 x + 1 個 unchecked → 第 4 個 checkbox 是 first pending
        let tmp = write_tasks(
            "# Tasks\n\n- [x] 1.1 done\n- [x] 1.2 done\n- [x] 1.3 done\n- [ ] 2.1 wip\n- [ ] 3.1 next\n",
        );
        assert_eq!(first_pending_task_index(tmp.path()), Some(4));
    }

    #[test]
    fn first_pending_index_returns_one_when_first_line_is_unchecked() {
        let tmp = write_tasks("- [ ] 1 first\n- [ ] 2 next\n");
        assert_eq!(first_pending_task_index(tmp.path()), Some(1));
    }

    #[test]
    fn first_pending_index_returns_none_when_all_checked() {
        let tmp = write_tasks("- [x] 1 done\n- [x] 2 done\n");
        assert_eq!(first_pending_task_index(tmp.path()), None);
    }

    #[test]
    fn first_pending_index_returns_none_for_missing_file() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(first_pending_task_index(tmp.path()), None);
    }

    #[test]
    fn first_pending_index_returns_none_for_empty_file() {
        let tmp = write_tasks("");
        assert_eq!(first_pending_task_index(tmp.path()), None);
    }

    #[test]
    fn first_pending_index_counts_all_checkbox_lines_regardless_of_label() {
        // label 可任意（dotted / numeric / alpha / 缺）— index 純看 checkbox 出現順序
        let tmp = write_tasks(
            "- [x] alpha done\n- [x] 12.34 done\n- [ ] anything pending\n- [x] later done\n",
        );
        assert_eq!(first_pending_task_index(tmp.path()), Some(3));
    }

    #[test]
    fn compute_next_actions_proposing_filters_existing_artifacts() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("proposal.md"), b"x").unwrap();
        let actions = compute_next_actions(ChangeState::Proposing, false, tmp.path());
        assert_eq!(
            actions,
            vec![
                "artifact.write design".to_string(),
                "artifact.write tasks".to_string()
            ]
        );
    }

    #[test]
    fn compute_next_actions_all_three_present_returns_empty_for_proposing() {
        let tmp = TempDir::new().unwrap();
        for f in ["proposal.md", "design.md", "tasks.md"] {
            std::fs::write(tmp.path().join(f), b"x").unwrap();
        }
        let actions = compute_next_actions(ChangeState::Proposing, false, tmp.path());
        assert!(actions.is_empty());
    }

    #[test]
    fn compute_next_actions_state_lookup_matrix() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            compute_next_actions(ChangeState::Ready, false, tmp.path()),
            vec!["apply.start".to_string()]
        );
        assert_eq!(
            compute_next_actions(ChangeState::Reviewing, false, tmp.path()),
            vec!["review.approve".to_string(), "review.reject".to_string()]
        );
        assert_eq!(
            compute_next_actions(ChangeState::CodeReviewing, true, tmp.path()),
            vec!["review.approve".to_string(), "review.reject".to_string()]
        );
        assert_eq!(
            compute_next_actions(ChangeState::Archived, true, tmp.path()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn compute_next_actions_in_progress_with_done_flag_suggests_archive() {
        let tmp = TempDir::new().unwrap();
        let actions = compute_next_actions(ChangeState::InProgress, true, tmp.path());
        assert_eq!(actions, vec!["archive.run".to_string()]);
    }

    #[test]
    fn compute_next_actions_in_progress_no_tasks_md_returns_bare_task_done() {
        let tmp = TempDir::new().unwrap();
        let actions = compute_next_actions(ChangeState::InProgress, false, tmp.path());
        assert_eq!(actions, vec!["task.done".to_string()]);
    }

    #[test]
    fn compute_next_actions_in_progress_with_pending_id_suggests_task_done_with_index() {
        // fixture：1 個 done + 1 個 unchecked（label `2.5` — 證明 emit 的是 INDEX 不是 label）。
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("tasks.md"), "- [x] 1 a\n- [ ] 2.5 next\n").unwrap();
        let actions = compute_next_actions(ChangeState::InProgress, false, tmp.path());
        // 第 2 個 checkbox 行（1-based）= INDEX 2，與 `speclink task done 2` CLI 對齊。
        assert_eq!(actions, vec!["task.done 2".to_string()]);
    }
}
