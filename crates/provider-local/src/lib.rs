//! Local filesystem provider — `.speclink/` 目錄結構 + SQLite state DB。
//!
//! 公開：
//! - [`LocalProvider`]：實作 [`provider::Provider`] trait 的本地端 storage 變體
//! - [`state_db::StateDb`]：對 `.speclink/state.db` 的 async 封裝
//! - [`error::LocalProviderError`]、[`error::StateDbError`]：對應 CLI error code

use async_trait::async_trait;
use provider::Provider;
use provider::error::ProviderError;
use provider::model::{
    Artifact, ArtifactKind, Change, ChangeId, CreatedBy, NewArtifact, NewChange, ProjectId, State,
};
use std::path::{Path, PathBuf};

pub mod error;
pub mod state_db;
pub mod storage;

use crate::error::LocalProviderError;
use crate::state_db::StateDb;
use crate::storage::{change_dir, is_valid_change_id, write_proposal_content_atomic};

/// 本地端 provider 實作，將 change 與 artifact 持久化於 `<base>/.speclink/`。
#[derive(Debug)]
pub struct LocalProvider {
    state_db: StateDb,
    base_path: PathBuf,
}

impl LocalProvider {
    /// 建立 provider 實體：在 `<base>/.speclink/` 下開啟（或建立）`state.db`。
    pub async fn new(base_path: PathBuf) -> Result<Self, LocalProviderError> {
        let speclink_dir = base_path.join(".speclink");
        std::fs::create_dir_all(&speclink_dir)?;
        let db_path = speclink_dir.join("state.db");
        let state_db = StateDb::open(&db_path).await?;
        Ok(Self {
            state_db,
            base_path,
        })
    }

    /// Provider base path（專案根目錄）。
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    fn map_local_err(e: LocalProviderError) -> ProviderError {
        match e {
            LocalProviderError::InvalidChangeId { change_id } => {
                ProviderError::InvalidChangeId { change_id }
            }
            LocalProviderError::ChangeAlreadyExists { change_id } => {
                ProviderError::ChangeAlreadyExists {
                    change_id: ChangeId::from(change_id),
                }
            }
            other => ProviderError::Internal {
                message: other.to_string(),
            },
        }
    }
}

#[async_trait]
impl Provider for LocalProvider {
    async fn create_change(
        &self,
        _project_id: &ProjectId,
        input: NewChange,
    ) -> Result<Change, ProviderError> {
        // 1) 驗證 change id
        if !is_valid_change_id(input.change_id.as_str()) {
            return Err(ProviderError::InvalidChangeId {
                change_id: input.change_id.as_str().to_string(),
            });
        }
        // 2) 檢查 change 目錄不存在
        let dir = change_dir(&self.base_path, &input.change_id);
        if dir.exists() {
            return Err(ProviderError::ChangeAlreadyExists {
                change_id: input.change_id.clone(),
            });
        }
        // create_change 本身不寫入檔案；實際側效應在 write_artifact 完成。
        Ok(Change {
            change_id: input.change_id.clone(),
            state: State::Proposed,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            created_by: CreatedBy {
                kind: "agent".to_string(),
                name: String::new(),
            },
        })
    }

    async fn write_artifact(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
        input: NewArtifact,
    ) -> Result<Artifact, ProviderError> {
        if input.kind != ArtifactKind::Proposal {
            return Err(ProviderError::Internal {
                message: format!(
                    "artifact kind {:?} not supported in this change",
                    input.kind
                ),
            });
        }
        let base = self.base_path.clone();
        let cid = change_id.clone();
        let content = input.content;
        let path: PathBuf =
            tokio::task::spawn_blocking(move || -> Result<PathBuf, LocalProviderError> {
                write_proposal_content_atomic(&base, &cid, &content)
            })
            .await
            .map_err(|e| ProviderError::Internal {
                message: format!("background task failed: {e}"),
            })?
            .map_err(Self::map_local_err)?;

        // 更新 state DB（spec 步驟 6-7）。
        self.state_db
            .set_in_progress(change_id)
            .await
            .map_err(|e| ProviderError::Internal {
                message: format!("state db error: {e}"),
            })?;

        // 組合 Artifact 回傳值；path 為相對於 base_path 的 POSIX 字串。
        let relative = path
            .strip_prefix(&self.base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned()
            .replace('\\', "/");
        Ok(Artifact {
            kind: ArtifactKind::Proposal,
            path: relative,
            content_hash: String::new(),
        })
    }

    async fn get_change(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
    ) -> Result<Change, ProviderError> {
        let meta_path = change_dir(&self.base_path, change_id).join("metadata.json");
        if !meta_path.exists() {
            return Err(ProviderError::ChangeNotFound {
                change_id: change_id.clone(),
            });
        }
        let content = std::fs::read_to_string(&meta_path).map_err(|e| ProviderError::Internal {
            message: format!("failed to read metadata: {e}"),
        })?;
        let change: Change =
            serde_json::from_str(&content).map_err(|e| ProviderError::Internal {
                message: format!("failed to parse metadata: {e}"),
            })?;
        Ok(change)
    }
}
