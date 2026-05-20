//! Provider crate — `Provider` trait、共用資料模型、設定載入與 provider resolution。
//!
//! `Provider` 是跨 crate 的核心抽象；CLI 在啟動時依 resolution 結果建構 `Arc<dyn Provider>`，
//! 並傳遞給 runtime 的編排函式（例如 `runtime::create_proposal`）。

use async_trait::async_trait;

pub mod config;
pub mod config_discovery;
pub mod error;
pub mod model;
pub mod resolution;

use crate::error::ProviderError;
use crate::model::{
    ArchiveOptions, ArchivedChange, Artifact, ArtifactInstructions, ArtifactKind, Change, ChangeId,
    ChangeStatus, NewArtifact, NewChange, ProjectId, TaskUpdate,
};

/// SpecLink 對外可替換的 storage 抽象。
///
/// 所有 method 為 async；trait 要求 `Send + Sync` 以支援 `Arc<dyn Provider>` 跨 thread 共用。
/// 本 change 僅實作 `LocalProvider`；HTTP / 其他 provider 由後續 change 加入。
#[async_trait]
pub trait Provider: Send + Sync {
    /// 建立 change，回傳具完整 lifecycle metadata 的 [`Change`]。
    ///
    /// 失敗條件：
    /// - change id 已存在 → [`ProviderError::ChangeAlreadyExists`]
    /// - filesystem / storage 失敗 → [`ProviderError::Internal`]
    async fn create_change(
        &self,
        project_id: &ProjectId,
        input: NewChange,
    ) -> Result<Change, ProviderError>;

    /// 寫入 artifact（例如 `proposal.md`），並更新對應 `metadata.json`。
    ///
    /// 寫入應為原子操作：失敗時 provider 必須清除半成品。
    async fn write_artifact(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
        input: NewArtifact,
    ) -> Result<Artifact, ProviderError>;

    /// 讀取既有 change 的 metadata。
    async fn get_change(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
    ) -> Result<Change, ProviderError>;

    /// 讀取 change 的 artifact 狀態快照（純讀；不修改 filesystem 或 state.db）。
    ///
    /// 失敗條件：
    /// - change 不存在 → [`ProviderError::ChangeNotFound`]
    /// - `metadata.json` 解析失敗或其他 storage 錯誤 → [`ProviderError::Internal`]
    async fn get_status(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
    ) -> Result<ChangeStatus, ProviderError>;

    /// archive 一個 change：套用 spec deltas、搬移目錄、更新 metadata 與 state.db。
    ///
    /// Provider 採 best-effort + explicit rollback 流程：詳見 spec `Archive rollback safeguards`。
    /// 當 `options.dry_run = true` 時，僅計算 delta merge 並返回 summary，不寫檔。
    ///
    /// 失敗條件：
    /// - change 不存在 → [`ProviderError::ChangeNotFound`]
    /// - 已 archived 或同名目標目錄存在 → [`ProviderError::ChangeNotArchivable`]
    /// - delta 衝突（ADDED 已存在 / MODIFIED 找不到等） → [`ProviderError::SpecDeltaConflict`]
    /// - delta 格式錯誤 → [`ProviderError::SpecDeltaParseError`]
    /// - filesystem / SQLite 失敗 → [`ProviderError::Internal`]
    async fn archive_change(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
        options: ArchiveOptions,
    ) -> Result<ArchivedChange, ProviderError>;

    /// 取得指定 artifact kind 的 instructions（template / rules / dependencies / unlocks）。
    ///
    /// 本 method side-effect-free：不寫任何檔案。`capability` 僅在 `kind == Spec` 時使用，
    /// 其他 kind 應傳 `None`；違反時 provider 可回 [`ProviderError::Internal`]
    /// 或 [`ProviderError::InvalidCapability`]，CLI 層通常先擋。
    ///
    /// 失敗條件：
    /// - change 不存在 → [`ProviderError::ChangeNotFound`]
    /// - spec kind 缺 capability → [`ProviderError::MissingCapability`]
    /// - capability 名稱非法 → [`ProviderError::InvalidCapability`]
    async fn get_artifact_instructions(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<ArtifactInstructions, ProviderError>;

    /// 將 `tasks.md` 中對應 `task_id` 的 checkbox 由 `[ ]` 翻為 `[x]`。
    ///
    /// idempotent：已完成的 task 再次呼叫不視為錯誤，`previous_status` 為
    /// [`crate::model::TaskStatus::Done`]、`current_status` 仍為 [`crate::model::TaskStatus::Done`]。
    ///
    /// 失敗條件：
    /// - task id 格式不符 → [`ProviderError::TaskInvalidId`]
    /// - task id 在 tasks.md 中找不到 → [`ProviderError::TaskNotFound`]
    /// - tasks.md 不存在 → [`ProviderError::ArtifactMissing`]
    /// - tasks.md 解析失敗 → [`ProviderError::TasksParseError`]
    /// - filesystem 失敗 → [`ProviderError::Internal`]
    async fn mark_task_done(
        &self,
        project_id: &ProjectId,
        change_id: &ChangeId,
        task_id: &str,
    ) -> Result<TaskUpdate, ProviderError>;
}
