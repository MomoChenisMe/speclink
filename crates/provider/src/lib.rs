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
    ArchiveOptions, ArchivedChange, Artifact, Change, ChangeId, ChangeStatus, NewArtifact,
    NewChange, ProjectId,
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
}
