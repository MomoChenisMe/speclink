//! `project.status` op — project-level read aggregator.
//!
//! 對齊 specs/project-status：
//!   - `provider_type` / `project_id` / `working_dir` 來自 link.yaml
//!   - `changes_count` runtime in-memory group-by `ChangeStore::list_changes()`
//!   - `current_change` 過濾 in_progress + actor.host_id == 當前 instance_id
//!   - `discussions_count` 永遠 `{active:0, converged:0}`（P2 #1 `add-discuss-ops` 才實作）
//!   - `schema_active` 目前 hardcode `DEFAULT_SCHEMA_ID`（待 `add-schema-ops` slice 接通 config.schema.active）
//!
//! Decision: project.status 走 runtime-side aggregation，不新增 provider trait method —
//! 直接呼叫 `LocalChangeStore::list_changes()` 與 `LocalStateMachineStore::get_change_state()`，
//! 避免「provider 加 `count_changes_by_state` thin-wrapper」反模式。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use speclink_provider::{Actor, ChangeStore, StateMachineStore};
use speclink_provider_local::{LocalChangeStore, LocalStateMachineStore, link_yaml};

use crate::change_ops::DEFAULT_SCHEMA_ID;
use crate::error::RuntimeError;
use crate::git::RealGitProbe;
use crate::paths::resolve_state_root;
use crate::state_machine::resolve_host_id;

/// `project.status` 對齊 operations.md §1389 output schema 的 runtime struct。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectStatusData {
    pub provider_type: String,
    pub project_id: String,
    pub working_dir: String,
    pub current_change: Option<CurrentChangeRef>,
    pub changes_count: ChangesCountByState,
    pub discussions_count: DiscussionsCountByState,
    pub schema_active: String,
}

/// `current_change` 子物件；只在 in_progress + host 匹配時填。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrentChangeRef {
    pub change_id: String,
    pub state: String,
    pub actor: Actor,
}

/// 六個 state bucket 的 row count。Schema 永遠完整、不省欄位。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChangesCountByState {
    pub proposing: u64,
    pub reviewing: u64,
    pub ready: u64,
    pub in_progress: u64,
    pub code_reviewing: u64,
    pub archived: u64,
}

/// 兩個 discussion bucket。P1 slice 永遠回 0/0。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiscussionsCountByState {
    pub active: u64,
    pub converged: u64,
}

/// 取得 `project.status` 完整 envelope data。
///
/// Read-only，不取 lock，不寫 audit event。
///
/// # Errors
/// - [`RuntimeError::NotInitialized`]：working_dir 不在 SpecLink 專案內（link.yaml 缺）
/// - [`RuntimeError::RequiresGit`]：working_dir 不在 git working tree
/// - [`RuntimeError::Internal`]：state.db 讀取或 list_changes 失敗
pub async fn project_status(working_dir: &Path) -> Result<ProjectStatusData, RuntimeError> {
    let probe = RealGitProbe;
    let state_root = resolve_state_root(&probe, working_dir)?;
    let link = link_yaml::read(working_dir)
        .map_err(map_provider_error)?
        .ok_or_else(|| RuntimeError::NotInitialized {
            path: working_dir.display().to_string(),
        })?;

    let change_store = LocalChangeStore::new(working_dir.to_path_buf(), state_root.clone());
    let sm_store = LocalStateMachineStore::new(state_root);

    let rows = change_store
        .list_changes()
        .await
        .map_err(map_provider_error)?;

    // Group-by state count（六 bucket 永遠完整）
    let mut counts = ChangesCountByState::default();
    for row in &rows {
        match row.state.as_str() {
            "proposing" => counts.proposing += 1,
            "reviewing" => counts.reviewing += 1,
            "ready" => counts.ready += 1,
            "in_progress" => counts.in_progress += 1,
            "code_reviewing" => counts.code_reviewing += 1,
            "archived" => counts.archived += 1,
            other => {
                // 不合法 state — 不阻斷整個 status；忽略並繼續（state-machine spec 已會在
                // 寫入路徑 reject 不合法值，這裡防禦性 skip 即可）。
                let _ = other;
            }
        }
    }

    // current_change：過濾 in_progress + actor.host_id == 當前 host hostname；
    // 多個匹配時取 updated_at 最新（list_changes 已依 updated_at desc 排序，第一個即為最新）。
    //
    // 比對端用 `state_machine::resolve_host_id()` — 與 `apply.start` 寫入
    // `actor_json.host_id` 走同一條 resolution chain。NOT 比對 `link.instance_id`
    // （那是 project-scoped UUID、不是 host-scoped）— 對齊 spec project-status
    // Requirement「current_change ... actor.host_id field equals the current host's
    // hostname identifier」。
    let current_host_id = resolve_host_id();
    let mut current_change: Option<CurrentChangeRef> = None;
    for row in rows.iter().filter(|r| r.state == "in_progress") {
        // 讀 actor.host_id；errors → skip，不 fail 整個 status。
        let Ok(view) = sm_store.get_change_state(&row.name).await else {
            continue;
        };
        let Some(actor) = view.actor else {
            continue;
        };
        if actor.host_id == current_host_id {
            current_change = Some(CurrentChangeRef {
                change_id: row.change_id.clone(),
                state: "in_progress".to_string(),
                actor,
            });
            break; // list_changes 已 desc 排序，第一個 match 即 latest
        }
    }

    // working_dir → display string（不 canonicalize，避免 Windows / macOS 行為差異）
    let working_str = working_dir.display().to_string();

    Ok(ProjectStatusData {
        provider_type: link.provider.clone(),
        project_id: link.project_id.clone(),
        working_dir: working_str,
        current_change,
        changes_count: counts,
        discussions_count: DiscussionsCountByState::default(),
        // 待 add-schema-ops slice 接 config.schema.active；MVP 期間 hardcode。
        schema_active: DEFAULT_SCHEMA_ID.to_string(),
    })
}

/// 集中收 `ProviderError` → `RuntimeError`，只覆蓋本 op 會碰到的 variant。
fn map_provider_error(err: speclink_provider::ProviderError) -> RuntimeError {
    use speclink_provider::ProviderError as P;
    match err {
        P::RequiresGit { context } => RuntimeError::RequiresGit { context },
        P::NotInitialized { path } => RuntimeError::NotInitialized { path },
        // 其他狀況 bubble 成 Internal — read-only op 不該碰到 state machine / artifact 變更錯。
        other => RuntimeError::Internal(format!("project.status: {other}")),
    }
}

