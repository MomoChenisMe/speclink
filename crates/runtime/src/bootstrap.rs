//! Bootstrap orchestration: git check → prepare → commit → `.gitignore` append.
//!
//! 採 prepare-then-commit pattern：所有 artifact 與 state 先在 working_dir 內的
//! staging tempdir 中拼好，最後一次性 rename 到正式位置。Commit phase 中途
//! 失敗時由 [`CommitGuard`] RAII cleanup 已 rename 出 staging 的檔案。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use speclink_provider::{LinkYaml, ProjectInfo};
use speclink_provider_local::link_yaml;
use speclink_provider_local::state_db::StateDb;
use uuid::Uuid;

use crate::error::RuntimeError;
use crate::git::GitProbe;
use crate::gitignore;
use crate::paths::{ARTIFACT_ROOT, STATE_ROOT_NAMESPACE, display_state_root};

/// Bootstrap orchestration.
pub struct Bootstrap<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> Bootstrap<G> {
    /// 建立 Bootstrap handle。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    /// 對 `working_dir` 執行 `speclink init`。
    ///
    /// # Errors
    /// 見 [`RuntimeError`] variants：`RequiresGit`、`AlreadyInitialized`、`Internal`。
    pub async fn init(&self, working_dir: &Path, force: bool) -> Result<ProjectInfo, RuntimeError> {
        // 1. Git check — RequiresGit if cannot resolve common dir.
        let common_dir = self.git.common_dir(working_dir)?;
        let state_root = common_dir.join(STATE_ROOT_NAMESPACE);

        // 2. Existing link.yaml triggers AlreadyInitialized unless --force.
        let artifact_root = working_dir.join(ARTIFACT_ROOT);
        let link_path = artifact_root.join("link.yaml");
        let target_schemas = artifact_root.join("schemas");

        let existing_link = if link_path.exists() {
            link_yaml::read(working_dir).ok().flatten()
        } else {
            None
        };
        if existing_link.is_some() && !force {
            return Err(RuntimeError::AlreadyInitialized {
                path: link_path.display().to_string(),
            });
        }

        // 3. Prepare phase: build artifact + state in a staging tempdir.
        //
        // tempdir is created **inside working_dir** so subsequent renames stay on
        // the same filesystem (rename(2) requires same fs). Drop of TempDir cleans
        // up any leftover contents automatically if we error before commit.
        let staging = tempfile::tempdir_in(working_dir)
            .map_err(|e| RuntimeError::Internal(format!("create staging tempdir: {e}")))?;
        let staging_path = staging.path();

        // Decide project_id / instance_id / created_at / fingerprint.
        //
        // 規則：
        // - force 模式且既有 link.yaml：沿用既有 project_id。
        // - state.db 已存在於 state root（worktree 場景）：從中讀取唯一 project row 的 id，
        //   寫進 worktree 的 link.yaml，跳過 staging insert（既存 row 保留）。
        // - 否則：生新 UUID v4。
        let target_state_db = state_root.join("state.db");
        let preexisting_state_db = target_state_db.exists();

        let project_id = if let Some(prev) = existing_link.as_ref().filter(|_| force) {
            prev.project_id.clone()
        } else if preexisting_state_db {
            // 從既存 state.db 取既有 project row id。
            let existing = StateDb::open(&target_state_db)
                .map_err(|e| RuntimeError::Internal(format!("open preexisting state.db: {e}")))?;
            let row_id = existing
                .single_project_id()
                .map_err(|e| RuntimeError::Internal(format!("read preexisting project row: {e}")))?
                .ok_or_else(|| {
                    RuntimeError::Internal(
                        "preexisting state.db has no project row; MVP requires exactly one".into(),
                    )
                })?;
            drop(existing);
            row_id
        } else {
            Uuid::new_v4().to_string()
        };
        let instance_id = Uuid::new_v4().to_string();
        let created_at = now_rfc3339();
        let fingerprint = link_yaml::working_dir_fingerprint(working_dir);

        // 3a. Stage state.db only when target absent. Worktree case reuses
        // the main repo's state.db verbatim — no staging row to insert.
        let staging_state_db = staging_path.join("state.db");
        if !preexisting_state_db {
            let db = StateDb::open(&staging_state_db)
                .map_err(|e| RuntimeError::Internal(format!("staging state.db open: {e}")))?;
            db.migrate(1)
                .map_err(|e| RuntimeError::Internal(format!("staging state.db migrate: {e}")))?;
            db.insert_project_row(
                &project_id,
                &instance_id,
                &working_dir.to_string_lossy(),
                &created_at,
            )
            .map_err(|e| RuntimeError::Internal(format!("staging insert project row: {e}")))?;
            drop(db);
        }

        // 3b. Stage link.yaml.
        let link = LinkYaml {
            version: 1,
            project_id: project_id.clone(),
            instance_id,
            provider: "local".to_string(),
            created_at,
            working_dir_fingerprint: fingerprint,
        };
        let staging_link = staging_path.join("link.yaml");
        let link_yaml_str = serde_yaml::to_string(&link)
            .map_err(|e| RuntimeError::Internal(format!("serialize link.yaml: {e}")))?;
        fs::write(&staging_link, link_yaml_str)
            .map_err(|e| RuntimeError::Internal(format!("write staging link.yaml: {e}")))?;

        // 3c. Stage schemas dir (empty for MVP).
        let staging_schemas = staging_path.join("schemas");
        fs::create_dir(&staging_schemas)
            .map_err(|e| RuntimeError::Internal(format!("staging schemas dir: {e}")))?;

        // 4. Commit phase — guarded so failures roll back partial artifacts.
        let mut guard = CommitGuard {
            artifact_root: artifact_root.clone(),
            target_schemas: target_schemas.clone(),
            target_link: link_path.clone(),
            target_state_db: target_state_db.clone(),
            target_locks: state_root.join("locks"),
            preexisting_artifact_root: artifact_root.exists(),
            preexisting_state_db: target_state_db.exists(),
            success: false,
        };

        // 4a. Ensure target parents (only after prepare succeeded).
        fs::create_dir_all(&state_root)
            .map_err(|e| RuntimeError::Internal(format!("create state root: {e}")))?;
        fs::create_dir_all(state_root.join("locks"))
            .map_err(|e| RuntimeError::Internal(format!("create locks dir: {e}")))?;
        fs::create_dir_all(&artifact_root)
            .map_err(|e| RuntimeError::Internal(format!("create artifact root: {e}")))?;

        // 4b. Commit state.db (only when target doesn't already exist —
        //     force mode keeps the existing DB).
        if !guard.preexisting_state_db {
            fs::rename(&staging_state_db, &target_state_db)
                .map_err(|e| RuntimeError::Internal(format!("rename state.db into target: {e}")))?;
        }

        // 4c. Commit schemas dir (skip if target already exists).
        if !target_schemas.exists() {
            fs::rename(&staging_schemas, &target_schemas)
                .map_err(|e| RuntimeError::Internal(format!("rename schemas dir: {e}")))?;
        }

        // 4d. Append .gitignore (idempotent line policy).
        gitignore::append_if_missing(&working_dir.join(".gitignore"), ".speclink/link.yaml")?;

        // 4e. Commit link.yaml last — its presence marks init completion.
        if link_path.exists() {
            fs::remove_file(&link_path)
                .map_err(|e| RuntimeError::Internal(format!("remove old link.yaml: {e}")))?;
        }
        fs::rename(&staging_link, &link_path)
            .map_err(|e| RuntimeError::Internal(format!("rename link.yaml: {e}")))?;

        // Success — disarm the guard.
        guard.success = true;

        Ok(ProjectInfo {
            project_id,
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: display_state_root(working_dir, &state_root),
        })
    }
}

fn now_rfc3339() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

/// Best-effort cleanup of partial artifacts when commit phase fails mid-way.
///
/// 只清掉本次 init 新建的檔案；既有 `.speclink/` 內容（如 force mode 下原有 schemas）
/// 保留不動。
struct CommitGuard {
    artifact_root: PathBuf,
    target_schemas: PathBuf,
    target_link: PathBuf,
    target_state_db: PathBuf,
    target_locks: PathBuf,
    preexisting_artifact_root: bool,
    preexisting_state_db: bool,
    success: bool,
}

impl Drop for CommitGuard {
    fn drop(&mut self) {
        if self.success {
            return;
        }
        // link.yaml is always created fresh, safe to remove on failure.
        let _ = fs::remove_file(&self.target_link);
        // schemas dir: only remove if it didn't exist before init started.
        if !self.preexisting_artifact_root {
            let _ = fs::remove_dir_all(&self.target_schemas);
            let _ = fs::remove_dir(&self.artifact_root);
        }
        // state.db: only remove if not preexisting (force keeps it).
        if !self.preexisting_state_db {
            let _ = fs::remove_file(&self.target_state_db);
        }
        let _ = fs::remove_dir(&self.target_locks);
    }
}
