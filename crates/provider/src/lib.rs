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
use crate::model::{Artifact, Change, ChangeId, NewArtifact, NewChange, ProjectId};

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
}
