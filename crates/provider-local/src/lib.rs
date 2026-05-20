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
    ArchiveOptions, ArchivedChange, Artifact, ArtifactInstructions, ArtifactKind, Change, ChangeId,
    ChangeStatus, CreatedBy, NewArtifact, NewChange, ProjectId, State, TaskUpdate,
};
use std::path::{Path, PathBuf};

pub mod archive;
pub mod error;
pub mod state_db;
pub mod storage;

use crate::error::LocalProviderError;
use crate::state_db::StateDb;
use crate::storage::{
    change_dir, is_valid_change_id, to_posix_string, write_design_atomic,
    write_proposal_content_atomic, write_spec_atomic, write_tasks_atomic,
};

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
            LocalProviderError::ChangeNotFound { change_id } => ProviderError::ChangeNotFound {
                change_id: ChangeId::from(change_id),
            },
            LocalProviderError::ArtifactAlreadyExists { kind, change_id } => {
                ProviderError::ArtifactAlreadyExists {
                    kind,
                    change_id: ChangeId::from(change_id),
                }
            }
            LocalProviderError::MissingCapability => ProviderError::MissingCapability,
            LocalProviderError::InvalidCapability { capability } => {
                ProviderError::InvalidCapability { capability }
            }
            LocalProviderError::ChangeNotArchivable { reason } => {
                ProviderError::ChangeNotArchivable { reason }
            }
            LocalProviderError::SpecDeltaConflict {
                capability,
                requirement,
                operation,
            } => ProviderError::SpecDeltaConflict {
                capability,
                requirement,
                operation,
            },
            LocalProviderError::SpecDeltaParseError {
                capability,
                message,
            } => ProviderError::SpecDeltaParseError {
                capability,
                message,
            },
            LocalProviderError::ArtifactMissing {
                artifact_id,
                change_id,
            } => ProviderError::ArtifactMissing {
                artifact_id,
                change_id: ChangeId::from(change_id),
            },
            LocalProviderError::TaskInvalidId { task_id } => {
                ProviderError::TaskInvalidId { task_id }
            }
            LocalProviderError::TaskNotFound { task_id } => ProviderError::TaskNotFound { task_id },
            LocalProviderError::TasksParseError { message } => {
                ProviderError::TasksParseError { message }
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
        let kind = input.kind;
        let content = input.content;
        let capability = input.capability.clone();
        let base = self.base_path.clone();
        let cid = change_id.clone();

        // spec 缺 capability → MissingCapability；非 spec 帶 capability → Internal（CLI 應已先擋）
        match (kind, capability.as_deref()) {
            (ArtifactKind::Spec, None) => {
                return Err(ProviderError::MissingCapability);
            }
            (ArtifactKind::Proposal | ArtifactKind::Design | ArtifactKind::Tasks, Some(_)) => {
                return Err(ProviderError::Internal {
                    message: "capability must not be set for non-spec artifact".to_string(),
                });
            }
            _ => {}
        }

        let path: PathBuf =
            tokio::task::spawn_blocking(move || -> Result<PathBuf, LocalProviderError> {
                match kind {
                    ArtifactKind::Proposal => write_proposal_content_atomic(&base, &cid, &content),
                    ArtifactKind::Design => write_design_atomic(&base, &cid, &content),
                    ArtifactKind::Tasks => write_tasks_atomic(&base, &cid, &content),
                    ArtifactKind::Spec => {
                        let cap = capability.ok_or(LocalProviderError::MissingCapability)?;
                        write_spec_atomic(&base, &cid, &cap, &content)
                    }
                }
            })
            .await
            .map_err(|e| ProviderError::Internal {
                message: format!("background task failed: {e}"),
            })?
            .map_err(Self::map_local_err)?;

        // 僅 proposal 寫入時更新 state DB（標記為 in-progress）。
        // design / tasks / spec 不更新 metadata.json 或 state.db。
        if kind == ArtifactKind::Proposal {
            self.state_db
                .set_in_progress(change_id)
                .await
                .map_err(|e| ProviderError::Internal {
                    message: format!("state db error: {e}"),
                })?;
        }

        // path 為相對於 base_path 的 POSIX 字串。
        let relative_path = path.strip_prefix(&self.base_path).unwrap_or(&path);
        let relative = to_posix_string(relative_path);
        Ok(Artifact {
            kind,
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

    async fn get_status(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
    ) -> Result<ChangeStatus, ProviderError> {
        let base = self.base_path.clone();
        let cid = change_id.clone();
        tokio::task::spawn_blocking(move || crate::storage::scan_change_status(&base, &cid))
            .await
            .map_err(|e| ProviderError::Internal {
                message: format!("background task failed: {e}"),
            })?
            .map_err(Self::map_local_err)
    }

    async fn archive_change(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
        options: ArchiveOptions,
    ) -> Result<ArchivedChange, ProviderError> {
        let base = self.base_path.clone();
        let cid = change_id.clone();

        // 步驟 1-7 由 spawn_blocking 內的同步邏輯處理；SQLite 步驟 8 在 spawn_blocking 完成
        // 後執行（StateDb 自帶 async + spawn_blocking）。
        let result =
            tokio::task::spawn_blocking(move || crate::archive::run_archive(&base, &cid, options))
                .await
                .map_err(|e| ProviderError::Internal {
                    message: format!("background task failed: {e}"),
                })?
                .map_err(Self::map_local_err)?;

        // dry-run 不動 SQLite；正常路徑：DELETE row（idempotent — 找不到不算失敗）。
        if !options.dry_run {
            self.state_db
                .clear_in_progress(change_id)
                .await
                .map_err(|e| ProviderError::Internal {
                    message: format!("state db error: {e}"),
                })?;
        }
        Ok(result)
    }

    async fn get_artifact_instructions(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<ArtifactInstructions, ProviderError> {
        // change id 與 capability 預檢
        if !crate::storage::is_valid_change_id(change_id.as_str()) {
            return Err(ProviderError::InvalidChangeId {
                change_id: change_id.as_str().to_string(),
            });
        }
        match (kind, capability) {
            (ArtifactKind::Spec, None) => return Err(ProviderError::MissingCapability),
            (ArtifactKind::Spec, Some(cap)) if !crate::storage::is_valid_capability_name(cap) => {
                return Err(ProviderError::InvalidCapability {
                    capability: cap.to_string(),
                });
            }
            (ArtifactKind::Proposal | ArtifactKind::Design | ArtifactKind::Tasks, Some(_)) => {
                return Err(ProviderError::Internal {
                    message: "capability must not be set for non-spec artifact".to_string(),
                });
            }
            _ => {}
        }
        // change 必須存在（metadata.json 為 source of truth）
        let dir = crate::storage::change_dir(&self.base_path, change_id);
        if !dir.exists() || !dir.join("metadata.json").is_file() {
            return Err(ProviderError::ChangeNotFound {
                change_id: change_id.clone(),
            });
        }
        // 委派給 runtime 的 compose helper。
        let cid_str = change_id.as_str().to_string();
        let cap_owned = capability.map(|s| s.to_string());
        runtime::instructions::compose_local_instructions(kind, &cid_str, cap_owned.as_deref())
            .map_err(|e| ProviderError::Internal {
                message: format!("compose_local_instructions failed: {e}"),
            })
    }

    async fn mark_task_done(
        &self,
        _project_id: &ProjectId,
        change_id: &ChangeId,
        task_id: &str,
    ) -> Result<TaskUpdate, ProviderError> {
        // 提早格式校驗：runtime 也會擋，但這裡能避免 spawn_blocking 浪費。
        if !runtime::tasks_parser::is_valid_task_id(task_id) {
            return Err(ProviderError::TaskInvalidId {
                task_id: task_id.to_string(),
            });
        }
        if !crate::storage::is_valid_change_id(change_id.as_str()) {
            return Err(ProviderError::InvalidChangeId {
                change_id: change_id.as_str().to_string(),
            });
        }

        let base = self.base_path.clone();
        let cid = change_id.clone();
        let tid = task_id.to_string();

        tokio::task::spawn_blocking(move || -> Result<TaskUpdate, LocalProviderError> {
            crate::storage::mark_task_done_on_disk(&base, &cid, &tid)
        })
        .await
        .map_err(|e| ProviderError::Internal {
            message: format!("background task failed: {e}"),
        })?
        .map_err(Self::map_local_err)
    }
}
