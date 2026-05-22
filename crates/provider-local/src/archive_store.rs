//! `LocalArchiveStore` — `ArchiveStore` trait 的 SQLite-backed + filesystem 實作。
//!
//! 對齊 design.md「Observable behavior」「Provider trait surface」決策；單一
//! `archive_change` method 完成 state guard、SQLite tx（state transition + audit +
//! `archived_at`）、change dir atomic rename、spec delta merge、與失敗時的
//! best-effort revert。
//!
//! 與 `LocalStateMachineStore` 不同：本 store 跨 SQLite 與 filesystem 兩個邊界、
//! 在 commit 後做不可逆的 rename。順序契約對應 archive-runner spec：
//! 「Filesystem rename SHALL happen after SQLite transaction commit」、
//! 「Spec merge SHALL happen after the directory rename」。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use speclink_provider::{
    ArchiveRequest, ArchiveResult, ArchiveStore, ChangeState, MergedSpec, ProviderError,
    StateTransitionReason,
};
use uuid::Uuid;

use crate::state_db::{ActorUpdate, StateDb, StateDbError, StateTransitionRow};

/// LocalProvider 的 `ArchiveStore` 實作。
pub struct LocalArchiveStore {
    working_dir: PathBuf,
    state_root: PathBuf,
}

impl LocalArchiveStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(working_dir: PathBuf, state_root: PathBuf) -> Self {
        Self {
            working_dir,
            state_root,
        }
    }

    /// 工作樹根（含 `.speclink/changes/<id>/` artifact tree）。
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// State root 路徑（含 `state.db`）。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn open_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let path = self.state_root.join("state.db");
        let db = StateDb::open(&path).map_err(|e| map_state_db_error(e, "open state.db"))?;
        db.migrate(4)
            .map_err(|e| map_state_db_error(e, "migrate state.db"))?;
        Ok(db)
    }
}

#[async_trait]
impl ArchiveStore for LocalArchiveStore {
    async fn archive_change(&self, req: ArchiveRequest) -> Result<ArchiveResult, ProviderError> {
        // 1. Acquire stub locks (no-op；真實 lock impl 由 add-locking-and-concurrency slice 接通)
        let _change_lock = lock_stubs::acquire_change_exclusive(&req.change_id);
        let _global_lock = lock_stubs::acquire_global_short();

        // 2. Open DB + read current state row
        let db = self.open_db()?;
        let row = db
            .read_change_state_row(&req.change_id)
            .map_err(|e| map_state_db_error(e, "read change state"))?;

        // 3. State guard (design「State guard：兩條件 AND」決策)
        if row.state != ChangeState::InProgress.as_str() {
            return Err(ProviderError::StateTransitionInvalid {
                from: row.state.clone(),
                to: ChangeState::Archived.as_str().to_string(),
            });
        }
        if !row.all_tasks_done {
            return Err(ProviderError::ChangeTasksIncomplete {
                change_id: req.change_id.clone(),
            });
        }

        // 4. Time + transition id; archive dir name resolved lazily AFTER tx commit.
        let now = now_rfc3339();
        let transition_id = Uuid::new_v4().to_string();
        let audit = StateTransitionRow {
            transition_id: &transition_id,
            from_state: ChangeState::InProgress.as_str(),
            to_state: ChangeState::Archived.as_str(),
            actor_json: None,
            transitioned_at: &now,
            reason: StateTransitionReason::ArchiveRun.as_str(),
        };

        // 5. SQLite tx: state→archived + archived_at=now + audit insert (single tx)
        let new_version = db
            .update_change_state_with_archived_at_cas(
                &row.change_id,
                row.version,
                ChangeState::Archived.as_str(),
                ActorUpdate::Keep,
                Some(&now),
                &audit,
                &now,
            )
            .map_err(|e| map_state_db_error(e, "archive transition"))?;

        // 6. POST-COMMIT: resolve archive dir, ensure parent, atomic rename.
        //    Failure on this leg triggers best-effort revert.
        let source_dir = changes_dir(&self.working_dir).join(&req.change_id);
        let archive_root = changes_dir(&self.working_dir).join("archive");
        let date = utc_date_prefix(&now);
        let post_commit = || -> Result<(PathBuf, String), ProviderError> {
            fs::create_dir_all(&archive_root).map_err(|e| {
                ProviderError::Internal(format!(
                    "create archive root {}: {e}",
                    archive_root.display()
                ))
            })?;
            let target_dir = resolve_archive_dir(&archive_root, &req.change_id, &date)?;
            fs::rename(&source_dir, &target_dir).map_err(|e| {
                ProviderError::Internal(format!(
                    "rename {} -> {}: {e}",
                    source_dir.display(),
                    target_dir.display()
                ))
            })?;
            let rel = format!(
                ".speclink/changes/archive/{}",
                target_dir
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            );
            Ok((target_dir, rel))
        };
        let (target_dir, archive_dir_rel) = match post_commit() {
            Ok(v) => v,
            Err(rename_err) => {
                // Best-effort revert: state→in_progress, archived_at=NULL, +archive_run_revert audit.
                let revert_transition_id = Uuid::new_v4().to_string();
                let revert_audit = StateTransitionRow {
                    transition_id: &revert_transition_id,
                    from_state: ChangeState::Archived.as_str(),
                    to_state: ChangeState::InProgress.as_str(),
                    actor_json: None,
                    transitioned_at: &now,
                    reason: StateTransitionReason::ArchiveRunRevert.as_str(),
                };
                let _ = db.update_change_state_with_archived_at_cas(
                    &row.change_id,
                    new_version,
                    ChangeState::InProgress.as_str(),
                    ActorUpdate::Keep,
                    None,
                    &revert_audit,
                    &now,
                );
                return Err(rename_err);
            }
        };

        // 7. Spec merge (skipped under --skip-specs)
        let merged_specs = if req.skip_specs {
            Vec::new()
        } else {
            merge_spec_files(&target_dir, &self.working_dir)?
        };

        // 8. Build result
        Ok(ArchiveResult {
            change_id: req.change_id,
            state: ChangeState::Archived,
            merged_specs,
            archived_at: now,
            archive_dir: archive_dir_rel,
        })
    }
}

/// Resolve the target archive dir name with same-day collision suffix `-2`/`-3`/...
///
/// 100-attempt cap aligns with design decision「Archive 目錄命名：日期前綴 vs 純
/// change-id」under the same-day collision rule.
pub(crate) fn resolve_archive_dir(
    archive_root: &Path,
    change_id: &str,
    date: &str,
) -> Result<PathBuf, ProviderError> {
    let base = format!("{date}-{change_id}");
    for n in 1u32..=100 {
        let name = if n == 1 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        let candidate = archive_root.join(&name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ProviderError::Internal(format!(
        "archive dir collision: exhausted 100 retries for {base}"
    )))
}

/// 遍歷 `<archive_dir>/specs/<capability>/spec.md` 每份檔，對 `.speclink/specs/<capability>/spec.md`
/// 做整檔覆蓋寫入；對應 design 決策「Spec delta merge：整檔覆蓋 vs schema-aware diff」。
pub(crate) fn merge_spec_files(
    archive_dir: &Path,
    working_dir: &Path,
) -> Result<Vec<MergedSpec>, ProviderError> {
    let specs_root = archive_dir.join("specs");
    if !specs_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(&specs_root)
        .map_err(|e| ProviderError::Internal(format!("read specs root: {e}")))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    let mut out = Vec::new();
    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let cap_name = match entry.file_name().to_string_lossy().into_owned() {
            s if s.is_empty() => continue,
            s => s,
        };
        let source = path.join("spec.md");
        if !source.is_file() {
            continue;
        }
        let new_bytes = fs::read(&source)
            .map_err(|e| ProviderError::Internal(format!("read source spec.md: {e}")))?;
        let lines_added = count_lines(&new_bytes);

        let target_cap_dir = working_dir.join(".speclink").join("specs").join(&cap_name);
        let target = target_cap_dir.join("spec.md");
        let lines_removed = if target.is_file() {
            match fs::read(&target) {
                Ok(bs) => count_lines(&bs),
                Err(_) => 0,
            }
        } else {
            0
        };
        fs::create_dir_all(&target_cap_dir)
            .map_err(|e| ProviderError::Internal(format!("create capability dir: {e}")))?;
        atomic_write(&target, &new_bytes)?;
        out.push(MergedSpec {
            capability: cap_name,
            lines_added,
            lines_removed,
        });
    }
    Ok(out)
}

/// 收集 `<archive_dir>/specs/<capability>/spec.md` 存在的 capability 名（排序）；
/// 供 `--skip-specs` warning carrier 的 `details.capabilities_skipped` 使用。
pub fn collect_capability_names(archive_dir: &Path) -> Result<Vec<String>, ProviderError> {
    let specs_root = archive_dir.join("specs");
    if !specs_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = fs::read_dir(&specs_root)
        .map_err(|e| ProviderError::Internal(format!("read specs root: {e}")))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            if !path.is_dir() {
                return None;
            }
            if !path.join("spec.md").is_file() {
                return None;
            }
            Some(e.file_name().to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    Ok(names)
}

fn count_lines(bytes: &[u8]) -> u64 {
    // Count newline-terminated lines; tolerate file with no trailing newline by
    // adding 1 if content present and last byte != b'\n'.
    if bytes.is_empty() {
        return 0;
    }
    let nl = bytes.iter().filter(|b| **b == b'\n').count() as u64;
    if *bytes.last().unwrap() == b'\n' {
        nl
    } else {
        nl + 1
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ProviderError> {
    let parent = path
        .parent()
        .ok_or_else(|| ProviderError::Internal("spec target has no parent dir".into()))?;
    let tmp = parent.join(format!(
        ".speclink-archive-spec-{}.tmp",
        Uuid::new_v4().simple()
    ));
    fs::write(&tmp, bytes).map_err(|e| ProviderError::Internal(format!("write temp spec: {e}")))?;
    fs::rename(&tmp, path)
        .map_err(|e| ProviderError::Internal(format!("atomic rename spec: {e}")))?;
    Ok(())
}

fn changes_dir(working_dir: &Path) -> PathBuf {
    working_dir.join(".speclink").join("changes")
}

fn utc_date_prefix(rfc3339_utc: &str) -> String {
    // RFC3339 UTC「YYYY-MM-DDTHH:MM:SSZ」prefix 10 char = YYYY-MM-DD。
    rfc3339_utc.get(..10).unwrap_or("1970-01-01").to_string()
}

fn now_rfc3339() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

fn map_state_db_error(e: StateDbError, ctx: &str) -> ProviderError {
    match e {
        StateDbError::CasConflict { current_version } => {
            ProviderError::StateVersionConflict { current_version }
        }
        StateDbError::ChangeRowNotFound { change_id } => {
            ProviderError::ChangeNotFound { name: change_id }
        }
        StateDbError::SchemaVersion { expected, found } => ProviderError::StateDbSchemaInvalid {
            found,
            supported: expected,
        },
        other => ProviderError::Internal(format!("{ctx}: {other}")),
    }
}

/// Stub locks — no-op placeholders aligned with design「Lock acquisition：stub no-op vs 真 lock」決策。
/// 真實實作（per-change exclusive + global short）由 `add-locking-and-concurrency` slice 接通；
/// 本 slice 走 SQLite tx atomicity 加 filesystem rename 自身原子性即足以保證 walking-skeleton
/// 單機單行為的一致性。
pub(crate) mod lock_stubs {
    /// 持有 lock 的 RAII guard；drop 為 no-op。
    pub struct NoopGuard;

    /// 取得 `per-change-exclusive` lock 的 stub helper。
    #[must_use]
    pub fn acquire_change_exclusive(_change_id: &str) -> NoopGuard {
        NoopGuard
    }

    /// 取得 `global-short` lock 的 stub helper。
    #[must_use]
    pub fn acquire_global_short() -> NoopGuard {
        NoopGuard
    }
}
