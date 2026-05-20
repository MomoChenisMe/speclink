//! 同步 archive 編排（在 `tokio::task::spawn_blocking` 內被呼叫）。
//!
//! 對應 spec `Archive rollback safeguards` 的 9 步驟流程；
//! 失敗 rollback 行為遵循 spec 的「步驟 5-7 失敗 → 還原 .bak、刪 .tmp、保留原 active 目錄」。

use crate::error::LocalProviderError;
use crate::storage::{
    archive_change_dir, archive_root_dir, change_dir, main_spec_dir, main_spec_path,
    to_posix_string,
};
use provider::model::{
    ArchiveOptions, ArchivedChange, CapabilitySyncResult, ChangeId, SpecDeltaSummary, State,
};
use runtime::spec_delta::{ApplySummary, ParsedDelta, SpecDeltaError, apply_delta, parse_delta};
use std::path::{Path, PathBuf};

/// 同步執行 archive 主流程；caller 必須在 `spawn_blocking` 內呼叫。
pub fn run_archive(
    base: &Path,
    change_id: &ChangeId,
    options: ArchiveOptions,
) -> Result<ArchivedChange, LocalProviderError> {
    // 步驟 0：基本前置檢查 — change 是否存在 + 是否已 archived + 目標 archive 目錄不可預存在
    let active_dir = change_dir(base, change_id);
    let meta_path = active_dir.join("metadata.json");
    if !meta_path.exists() {
        return Err(LocalProviderError::ChangeNotFound {
            change_id: change_id.as_str().to_string(),
        });
    }
    let mut metadata: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&meta_path)?)?;
    if metadata
        .get("state")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "archived")
    {
        return Err(LocalProviderError::ChangeNotArchivable {
            reason: format!("change '{}' is already archived", change_id.as_str()),
        });
    }

    let date_prefix = options.archive_date.format("%Y-%m-%d").to_string();
    let target_archive_dir = archive_change_dir(base, change_id, &date_prefix);
    if target_archive_dir.exists() {
        return Err(LocalProviderError::ChangeNotArchivable {
            reason: format!(
                "archive directory already exists: {}",
                to_posix_string(&target_archive_dir)
            ),
        });
    }

    // 步驟 1-2：解析所有 capability spec 的 delta，並計算 merge 結果（in-memory，不寫檔）
    let merges = collect_delta_merges(base, change_id, &active_dir)?;

    // 計算 archive 時間（每次 archive 取一次，與 metadata 一致）
    let archived_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 製作 spec_sync 摘要（順序：capability 字典序）
    let mut sorted = merges;
    sorted.sort_by(|a, b| a.capability.cmp(&b.capability));
    let spec_sync = SpecDeltaSummary {
        capabilities_synced: sorted
            .iter()
            .map(|m| {
                let abs = main_spec_path(base, &m.capability);
                let rel = abs.strip_prefix(base).unwrap_or(&abs).to_path_buf();
                CapabilitySyncResult {
                    capability: m.capability.clone(),
                    main_spec_path: to_posix_string(&rel),
                    added_count: m.summary.added_count,
                    modified_count: m.summary.modified_count,
                    removed_count: m.summary.removed_count,
                    renamed_count: m.summary.renamed_count,
                    created_main_spec: m.summary.created_main_spec,
                }
            })
            .collect(),
    };

    if options.dry_run {
        // dry-run：在計算結果後返回，不寫檔、不動 SQLite
        let archive_rel = relativize_to_base(base, &target_archive_dir);
        return Ok(ArchivedChange {
            change_id: change_id.clone(),
            archive_path: to_posix_string(&archive_rel),
            state: State::Archived,
            archived_at,
            spec_sync,
            dry_run: true,
        });
    }

    // 步驟 3-7：實際寫入。為 rollback 紀錄：已建立的 .tmp / .bak / 已 rename 主 spec / 是否
    // 已 rename change_dir。
    let mut rollback = RollbackState::default();

    let result = (|| -> Result<(), LocalProviderError> {
        // 確保 archive root 存在
        std::fs::create_dir_all(archive_root_dir(base))?;
        std::fs::create_dir_all(main_spec_dir(base))?;

        // 步驟 2-prep：為每個既有主 spec 建立 .bak（後續用於 rollback）
        for merge in &sorted {
            let target = main_spec_path(base, &merge.capability);
            if target.exists() {
                let bak = sibling_with_suffix(&target, ".bak");
                std::fs::copy(&target, &bak)?;
                rollback.bak_files.push(bak);
            }
        }

        // 步驟 3：寫 main spec .tmp
        for merge in &sorted {
            let target = main_spec_path(base, &merge.capability);
            std::fs::create_dir_all(target.parent().expect("main spec must have parent"))?;
            let tmp = sibling_with_suffix(&target, ".tmp");
            std::fs::write(&tmp, &merge.new_main_content)?;
            rollback.tmp_main_specs.push((tmp, target));
        }

        // 步驟 4：更新 metadata.json.tmp（含 archived state 與 archivedAt）
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert(
                "state".to_string(),
                serde_json::Value::String("archived".to_string()),
            );
            obj.insert(
                "archivedAt".to_string(),
                serde_json::Value::String(archived_at.clone()),
            );
        }
        let meta_tmp = sibling_with_suffix(&meta_path, ".tmp");
        std::fs::write(&meta_tmp, serde_json::to_string_pretty(&metadata)?)?;
        rollback.meta_tmp = Some((meta_tmp, meta_path.clone()));

        // 步驟 5：rename metadata.tmp → metadata.json
        if let Some((tmp, final_path)) = rollback.meta_tmp.take() {
            std::fs::rename(&tmp, &final_path)?;
            rollback.meta_renamed = true;
        }

        // 步驟 6：rename 每個 main spec .tmp → 正式名
        // 紀錄已 rename 的對應 main spec path（rollback 用 .bak 還原）。
        let pending: Vec<(PathBuf, PathBuf)> = rollback.tmp_main_specs.drain(..).collect();
        for (tmp, target) in pending {
            std::fs::rename(&tmp, &target)?;
            rollback.renamed_main_specs.push(target);
        }

        // 步驟 7：rename change_dir → archive target
        std::fs::rename(&active_dir, &target_archive_dir)?;
        rollback.change_dir_renamed = true;
        Ok(())
    })();

    match result {
        Ok(()) => {
            // 步驟 9：成功 → 刪除所有 .bak
            for bak in rollback.bak_files.drain(..) {
                let _ = std::fs::remove_file(&bak);
            }
            let archive_rel = relativize_to_base(base, &target_archive_dir);
            Ok(ArchivedChange {
                change_id: change_id.clone(),
                archive_path: to_posix_string(&archive_rel),
                state: State::Archived,
                archived_at,
                spec_sync,
                dry_run: false,
            })
        }
        Err(original_err) => {
            // Rollback 步驟 5-7 中已執行的部分
            let rb = do_rollback(&rollback, &meta_path, &active_dir, &target_archive_dir);
            match rb {
                Ok(()) => Err(original_err),
                Err(rb_err) => Err(LocalProviderError::RollbackFailed {
                    tmp_files: rollback
                        .tmp_main_specs
                        .iter()
                        .map(|(p, _)| to_posix_string(p))
                        .chain(rollback.meta_tmp.iter().map(|(p, _)| to_posix_string(p)))
                        .collect(),
                    backup_files: rollback
                        .bak_files
                        .iter()
                        .map(|p| to_posix_string(p))
                        .collect(),
                    source: Box::new(rb_err.unwrap_or(original_err)),
                }),
            }
        }
    }
}

struct PerCapabilityMerge {
    capability: String,
    new_main_content: String,
    summary: ApplySummary,
}

fn collect_delta_merges(
    base: &Path,
    change_id: &ChangeId,
    active_dir: &Path,
) -> Result<Vec<PerCapabilityMerge>, LocalProviderError> {
    let specs_root = active_dir.join("specs");
    if !specs_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&specs_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let cap = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let delta_path = path.join("spec.md");
        if !delta_path.is_file() {
            continue;
        }
        let delta_text = std::fs::read_to_string(&delta_path)?;
        let parsed: ParsedDelta = parse_delta(&delta_text).map_err(|e| map_delta_err(&cap, e))?;
        let main_target = main_spec_path(base, &cap);
        let main_existing: Option<String> = if main_target.is_file() {
            Some(std::fs::read_to_string(&main_target)?)
        } else {
            None
        };
        let (new_content, summary) =
            apply_delta(main_existing.as_deref(), &parsed).map_err(|e| map_delta_err(&cap, e))?;
        let _ = change_id; // 預留：未來若需 per-change trace 註記，於此處組裝
        out.push(PerCapabilityMerge {
            capability: cap,
            new_main_content: new_content,
            summary,
        });
    }
    Ok(out)
}

fn map_delta_err(capability: &str, e: SpecDeltaError) -> LocalProviderError {
    match e {
        SpecDeltaError::Parse { message } => LocalProviderError::SpecDeltaParseError {
            capability: capability.to_string(),
            message,
        },
        SpecDeltaError::Conflict {
            requirement,
            operation,
        } => LocalProviderError::SpecDeltaConflict {
            capability: capability.to_string(),
            requirement,
            operation,
        },
    }
}

#[derive(Default)]
struct RollbackState {
    /// `(tmp path, final path)` — 寫了但尚未 rename 的 main spec tmp。
    tmp_main_specs: Vec<(PathBuf, PathBuf)>,
    /// `.bak` 副本路徑列表（archive 完成後刪除；rollback 時還原來源）。
    bak_files: Vec<PathBuf>,
    /// `(tmp, final)` — metadata.json.tmp，尚未 rename。
    meta_tmp: Option<(PathBuf, PathBuf)>,
    /// metadata.json 是否已從 .tmp rename 為正式名。
    meta_renamed: bool,
    /// 已 rename 完成的主 spec 正式路徑（rollback 時用同名 `.bak` 還原）。
    renamed_main_specs: Vec<PathBuf>,
    /// change_dir → archive target rename 是否成功。
    change_dir_renamed: bool,
}

fn do_rollback(
    rb: &RollbackState,
    meta_path: &Path,
    active_dir: &Path,
    target_archive_dir: &Path,
) -> Result<(), Option<LocalProviderError>> {
    // 1) 刪除任何殘留的 main spec .tmp
    for (tmp, _) in &rb.tmp_main_specs {
        let _ = std::fs::remove_file(tmp);
    }
    // 2) 刪除殘留 metadata.tmp
    if let Some((tmp, _)) = &rb.meta_tmp {
        let _ = std::fs::remove_file(tmp);
    }
    // 3) 若 change_dir 已搬到 archive，搬回原處
    if rb.change_dir_renamed && target_archive_dir.exists() {
        if let Err(_e) = std::fs::rename(target_archive_dir, active_dir) {
            return Err(None);
        }
    }
    // 4) 若已 rename main spec（步驟 6 成功了部分），從 .bak 還原
    for target in &rb.renamed_main_specs {
        let bak = sibling_with_suffix(target, ".bak");
        if bak.is_file() {
            if let Err(_e) = std::fs::copy(&bak, target) {
                return Err(None);
            }
        }
    }
    // 5) 若 metadata 已 rename，但需還原 — 改寫回原 state（讀取 .bak 不存在；保留 best-effort）。
    //    本實作不為 metadata 建 .bak（archive 流程已將 metadata 改成 archived 狀態）。
    //    若 archive 失敗，metadata 可能殘留 archived state — 由 rollback 將 change_dir 搬回後，
    //    使用者下次嘗試 archive 仍會被 already-archived 擋住。為避免此情形，將 metadata
    //    寫回原內容（讀取 active_dir 內的舊資料是 race-prone，這裡僅 best-effort）。
    if rb.meta_renamed {
        // 嘗試讀回原 metadata 並把 state 改回 "proposed"（保守 fallback）
        if let Ok(raw) = std::fs::read_to_string(meta_path) {
            if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert(
                        "state".to_string(),
                        serde_json::Value::String("proposed".to_string()),
                    );
                    obj.remove("archivedAt");
                    if let Ok(pretty) = serde_json::to_string_pretty(&v) {
                        let _ = std::fs::write(meta_path, pretty);
                    }
                }
            }
        }
    }
    // 6) 刪除 .bak（rollback 完成後）— 此處不刪：rollback 失敗訊息中需列出 .bak 路徑供人工。
    //    成功 rollback 後刪除由 outer caller 處理（archive 成功路徑刪 .bak；rollback 路徑保留）。
    Ok(())
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

fn relativize_to_base(base: &Path, p: &Path) -> PathBuf {
    p.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| p.to_path_buf())
}
