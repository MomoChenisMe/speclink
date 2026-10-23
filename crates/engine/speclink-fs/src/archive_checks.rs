//! Archive copies are validated before publication; the active proposal is never pruned.
use crate::{checks, security as io, FsStore};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use speclink_core::{
    history::{ArchiveChecksPlan, CheckIndex, INDEX},
    security::{data::sha256, prepared::validate_change},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    format_version: u8,
    change: String,
    dated_name: String,
    transaction_id: String,
}
fn locations(store: &FsStore, j: &Journal) -> (PathBuf, PathBuf, PathBuf) {
    let base = store.layout().spec_dir();
    (
        base.join(format!(".archive-stage-{}", j.transaction_id)),
        base.join(format!(".archive-backup-{}", j.transaction_id)),
        base.join(format!(".archive-transaction-{}.json", j.change)),
    )
}
fn load(store: &FsStore, name: &str) -> Result<Option<Journal>> {
    validate_change(Some(name))?;
    let path = store
        .layout()
        .spec_dir()
        .join(format!(".archive-transaction-{name}.json"));
    if !path.try_exists()? {
        return Ok(None);
    }
    let j: Journal = serde_json::from_slice(&io::read_limited(&path, 16 * 1024)?)?;
    ensure!(
        j.format_version == 1 && j.change == name,
        "history_recovery_required: archive journal"
    );
    validate_change(Some(&j.dated_name))?;
    speclink_core::history::validate_id(&j.transaction_id)?;
    ensure!(
        j.dated_name.get(11..) == Some(name),
        "history_recovery_required: archive identity"
    );
    Ok(Some(j))
}
pub(crate) fn uncommitted(store: &FsStore, dated: &str) -> bool {
    let Some(name) = dated.get(11..) else {
        return false;
    };
    match load(store, name) {
        Ok(Some(j)) if j.dated_name == dated => {
            let (_, backup, _) = locations(store, &j);
            !backup.exists() && store.layout().change_dir(name).exists()
        }
        Ok(_) => false,
        Err(_) => true,
    }
}

pub(crate) fn ensure_committed(store: &FsStore, dated: &str) -> Result<()> {
    let Some(name) = dated.get(11..) else {
        return Ok(());
    };
    if let Some(journal) = load(store, name)? {
        if journal.dated_name == dated {
            let (_, backup, _) = locations(store, &journal);
            ensure!(
                backup.exists() || !store.layout().change_dir(name).exists(),
                "history_recovery_required"
            );
        }
    }
    Ok(())
}
fn validate_tree(root: &Path) -> Result<()> {
    if root.join(INDEX).try_exists()? {
        let index: CheckIndex =
            serde_json::from_slice(&io::read_limited(&root.join(INDEX), 64 * 1024 * 1024)?)?;
        index.validate()?;
        for e in index.entries {
            if e.outcome == "omitted" { continue; }
            io::owned_directory(root.join(&e.relative_path).parent().unwrap(), root, false)?;
            let bytes = io::read_limited(&root.join(&e.relative_path), 64 * 1024 * 1024)?;
            ensure!(
                sha256(&bytes) == e.content_digest,
                "integrity_error: archive check record"
            );
            if e.kind == speclink_core::history::CheckKind::Security {
                let json = io::read_limited(
                    &root.join(e.relative_path.replace("report.md", "report.json")),
                    64 * 1024 * 1024,
                )?;
                speclink_core::security::report::validate_report_pair(
                    &e.record_id,
                    std::str::from_utf8(&json)?,
                    std::str::from_utf8(&bytes)?,
                )?;
            }
        }
    }
    if root.join("checks/acceptance.json").try_exists()? {
        let mut receipt: speclink_core::history::AcceptanceReceipt = serde_json::from_slice(
            &io::read_limited(&root.join("checks/acceptance.json"), 64 * 1024 * 1024)?,
        )?;
        receipt.validate()?;
    }
    Ok(())
}
fn recover(store: &FsStore, name: &str) -> Result<()> {
    let Some(j) = load(store, name)? else {
        return Ok(());
    };
    let (stage, backup, journal) = locations(store, &j);
    let source = store.layout().change_dir(name);
    let final_path = store.layout().archived_change_dir(&j.dated_name);
    if backup.try_exists()? {
        if final_path.try_exists()? {
            io::check_directory(&final_path)?;
            validate_tree(&final_path)?;
            fs::remove_dir_all(&backup)?;
        } else {
            ensure!(
                !source.try_exists()?,
                "history_recovery_required: ambiguous archive backup"
            );
            fs::rename(&backup, &source)?;
        }
    } else if source.try_exists()? {
        if final_path.try_exists()? {
            io::check_directory(&final_path)?;
            fs::remove_dir_all(&final_path)?;
        }
    } else {
        anyhow::bail!("history_recovery_required: archive source missing")
    }
    if stage.try_exists()? {
        io::check_directory(&stage)?;
        fs::remove_dir_all(&stage)?;
    }
    fs::remove_file(journal)?;
    io::sync_directory(&store.layout().spec_dir())?;
    Ok(())
}
fn copy_tree(
    source: &Path,
    stage: &Path,
    relative: &str,
    plan: &ArchiveChecksPlan,
    total: &mut u64,
) -> Result<()> {
    let directory = source.join(relative);
    io::owned_directory(&directory, source, false)?;
    fs::create_dir_all(stage.join(relative))?;
    for item in fs::read_dir(&directory)? {
        let item = item?;
        let name = item.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid_path: archive filename"))?;
        if relative.is_empty()
            && (name == ".checks.lock"
                || name == speclink_core::history::JOURNAL
                || name.starts_with(".checks-stage-")
                || name.starts_with(".checks-backup-"))
        {
            continue;
        }
        let rel = if relative.is_empty() {
            name.into()
        } else {
            format!("{relative}/{name}")
        };
        if !plan.retains(&rel) {
            continue;
        }
        let metadata = item.path().symlink_metadata()?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "invalid_path: archive symlink"
        );
        if metadata.is_dir() {
            copy_tree(source, stage, &rel, plan, total)?;
        } else {
            ensure!(metadata.is_file(), "invalid_path: archive special file");
            *total = total.saturating_add(metadata.len());
            ensure!(*total <= 512 * 1024 * 1024, "storage_limit: archive copy");
            let bytes = io::read_limited(&item.path(), 64 * 1024 * 1024)?;
            io::write_new(&stage.join(rel), &bytes, &|| false)?;
        }
    }
    io::sync_directory(&stage.join(relative))?;
    Ok(())
}
fn replace(root: &Path, path: &str, text: &str) -> Result<()> {
    let destination = root.join(path);
    io::owned_directory(destination.parent().unwrap(), root, true)?;
    let temp = root.join(format!(".archive-write-{}", ulid::Ulid::new()));
    io::write_new(&temp, text.as_bytes(), &|| false)?;
    io::atomic_replace(&temp, &destination)?;
    io::sync_directory(destination.parent().unwrap())
}
fn fault(_point: &str) -> Result<()> {
    #[cfg(test)]
    FAULTS.with(|points| {
        let mut points = points.borrow_mut();
        if points.first().is_some_and(|p| *p == _point) {
            points.remove(0);
            anyhow::bail!("injected archive failure: {_point}");
        }
        Ok(())
    })?;
    Ok(())
}
#[cfg(test)]
thread_local! {static FAULTS:std::cell::RefCell<Vec<&'static str>>=const{std::cell::RefCell::new(Vec::new())};}

pub(crate) fn archive(
    store: &FsStore,
    name: &str,
    dated: &str,
    plan: &ArchiveChecksPlan,
) -> Result<()> {
    validate_change(Some(name))?;
    validate_change(Some(dated))?;
    ensure!(
        dated.get(11..) == Some(name),
        "invalid_change: archive identity"
    );
    let root = store.layout().root.canonicalize()?;
    let base = root.join(&store.layout().spec_dir_name);
    ensure!(
        speclink_core::security::prepared::valid_relative(&store.layout().spec_dir_name),
        "invalid_spec_directory"
    );
    io::owned_directory(&base, &root, false)?;
    let lock = io::open_lock(&base.join(format!(".archive-lock-{name}")))?;
    lock.try_lock()
        .map_err(|_| anyhow::anyhow!("busy: archive transaction"))?;
    let _archive_lock = io::FileLock(lock);
    recover(store, name).map_err(|e| anyhow::anyhow!("history_recovery_required: {e}"))?;
    let j = Journal {
        format_version: 1,
        change: name.into(),
        dated_name: dated.into(),
        transaction_id: ulid::Ulid::new().to_string(),
    };
    let (stage, backup, journal) = locations(store, &j);
    let result = checks::with_write(store, name, || {
        plan.validate_base(store, name)?;
        let source = store.layout().change_dir(name);
        let final_path = store.layout().archived_change_dir(dated);
        io::owned_directory(
            &store.layout().archive_dir(),
            &store.layout().spec_dir(),
            true,
        )?;
        ensure!(
            !final_path.try_exists()?,
            "file_exists: archive destination"
        );
        fs::create_dir(&stage)?;
        let result = (|| -> Result<()> {
            copy_tree(&source, &stage, "", plan, &mut 0)?;
            if let Some(index) = &plan.index {
                replace(&stage, INDEX, index)?;
            }
            if let Some(receipt) = &plan.receipt {
                replace(&stage, "checks/acceptance.json", receipt)?;
            }
            if let Some(meta) = &plan.archived_meta {
                replace(&stage, ".openspec.yaml", meta)?;
            }
            validate_tree(&stage)?;
            fault("staged")?;
            // Still under the same checks lock: a changed plan fails before canonical writes.
            plan.write_canonical_specs(store)?;
            io::write_new(&journal, &serde_json::to_vec(&j)?, &|| false)?;
            io::sync_directory(&store.layout().spec_dir())?;
            fs::rename(&stage, &final_path)?;
            io::sync_directory(&store.layout().archive_dir())?;
            fault("published")?;
            // This rename is the commit point: all archived bytes are already durable.
            fs::rename(&source, &backup)?;
            io::sync_directory(&store.layout().spec_dir())?;
            Ok(())
        })();
        if let Err(error) = result {
            if backup.try_exists()? {
                return Err(anyhow::anyhow!(
                    "history_recovery_required: archive publication interrupted; backup retained"
                ));
            }
            fault("rollback").map_err(|_| {
                anyhow::anyhow!("history_recovery_required: archive rollback pending")
            })?;
            if journal.try_exists()? {
                recover(store, name)?;
            } else if stage.try_exists()? {
                fs::remove_dir_all(&stage)?;
            }
            return Err(error);
        }
        Ok(())
    });
    if result.is_ok() {
        // A failed cleanup leaves the durable journal and backup for recovery; publication remains valid.
        if recover(store, name).is_err() {
            return Ok(());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::store::Store;
    fn fixture() -> (tempfile::TempDir, FsStore, ArchiveChecksPlan) {
        let root = tempfile::tempdir().unwrap();
        let store = FsStore::new(root.path(), "openspec");
        let dir = store.layout().change_dir("demo");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".openspec.yaml"), "schema: spec-driven\n").unwrap();
        fs::write(dir.join("tasks.md"), "- [x] 1.1 Done\n").unwrap();
        fs::write(
            root.path().join("openspec/config.yaml"),
            "check_records:\n  archive: {review: false, verify: true, security: false}\n",
        )
        .unwrap();
        speclink_core::station::add_round(
            &speclink_core::station::REVIEW,
            &store,
            "demo",
            "**Scope**: app.ts\n",
        )
        .unwrap();
        speclink_core::station::stamp_with_scope(
            &speclink_core::station::REVIEW,
            &store,
            "demo",
            false,
            Some("Tester"),
            None,
            vec![speclink_core::model::ReviewedScopeEntry {
                path: "app.ts".into(),
                hash: "a".repeat(64),
            }],
            vec![],
        )
        .unwrap();
        fs::write(dir.join("review.md"), "carried working ticket").unwrap();
        let plan = speclink_core::history::prepare_archive_checks(
            &store,
            "demo",
            Some("Archivist"),
            "2026-10-21",
            "2026-10-21T00:00:00Z",
            None,
        )
        .unwrap();
        (root, store, plan)
    }
    #[test]
    fn canonical_specs_are_written_only_after_archive_base_validation() {
        for conflict in [false, true] {
            let (_root, store, mut plan) = fixture();
            store.write_canonical_spec("auth", "original").unwrap();
            plan.canonical_specs.insert("auth".into(), "updated".into());
            plan.canonical_specs.insert("new-cap".into(), "new".into());
            if conflict {
                store.write_change_meta("demo", "schema: spec-driven\nboard_rank: changed\n").unwrap();
            }
            let result = archive(&store, "demo", "2026-10-21-demo", &plan);
            if conflict {
                assert!(result.unwrap_err().to_string().contains("revision_conflict"));
                assert_eq!(store.read_canonical_spec("auth").as_deref(), Some("original"));
                assert_eq!(store.read_canonical_spec("new-cap"), None);
                assert!(store.change_exists("demo"));
            } else {
                result.unwrap();
                assert_eq!(store.read_canonical_spec("auth").as_deref(), Some("updated"));
                assert_eq!(store.read_canonical_spec("new-cap").as_deref(), Some("new"));
                assert!(store.archived_change_exists("2026-10-21-demo"));
            }
        }
    }

    #[test]
    fn archive_rejects_metadata_changed_after_planning() {
        let (_root, store, plan) = fixture();
        let latest = format!("{}board_rank: newest\n", store.read_change_meta("demo").unwrap());
        store.write_change_meta("demo", &latest).unwrap();
        let error = archive(&store, "demo", "2026-10-21-demo", &plan).unwrap_err();
        assert!(error.to_string().contains("revision_conflict"), "{error}");
        assert_eq!(store.read_change_meta("demo").as_deref(), Some(latest.as_str()));
        assert!(!store.archived_change_exists("2026-10-21-demo"));
    }
    #[test]
    fn concurrent_archive_cannot_recover_another_inflight_publication() {
        let (_root, store, plan) = fixture();
        let lock = io::open_lock(&store.layout().spec_dir().join(".archive-lock-demo")).unwrap();
        lock.try_lock().unwrap();
        let _guard = io::FileLock(lock);
        assert!(archive(&store, "demo", "2026-10-21-demo", &plan)
            .unwrap_err()
            .to_string()
            .contains("busy"));
        assert!(store.change_exists("demo"));
        assert!(!store.archived_change_exists("2026-10-21-demo"));
    }
    #[test]
    fn staged_archive_keeps_active_records_on_every_precommit_failure() {
        for point in ["staged", "published"] {
            let (_root, store, plan) = fixture();
            let before = store.read_artifact("demo", INDEX);
            let meta = store.read_change_meta("demo");
            FAULTS.with(|f| *f.borrow_mut() = vec![point]);
            assert!(archive(&store, "demo", "2026-10-21-demo", &plan).is_err());
            assert!(store.change_exists("demo"));
            assert_eq!(store.read_artifact("demo", INDEX), before);
            assert_eq!(store.read_change_meta("demo"), meta);
            assert!(!store.archived_change_exists("2026-10-21-demo"));
        }
    }
    #[test]
    fn published_copy_with_failed_rollback_is_hidden_then_recovers_without_losing_source() {
        let (_root, store, plan) = fixture();
        let before = store.read_artifact("demo", INDEX);
        FAULTS.with(|f| *f.borrow_mut() = vec!["published", "rollback"]);
        assert!(archive(&store, "demo", "2026-10-21-demo", &plan)
            .unwrap_err()
            .to_string()
            .starts_with("history_recovery_required"));
        assert_eq!(store.read_artifact("demo", INDEX), before);
        assert!(!store.archived_change_exists("2026-10-21-demo"));
        assert!(!store
            .list_archived_changes()
            .contains(&"2026-10-21-demo".into()));
        archive(&store, "demo", "2026-10-21-demo", &plan).unwrap();
        assert!(!store.change_exists("demo"));
        let entries = speclink_core::history::index(&store, "2026-10-21-demo").unwrap().entries;
        assert_eq!(entries.len(), 1);
        assert!(entries.iter().all(|entry| entry.outcome == "omitted"));
        assert!(speclink_core::history::record(&store, "2026-10-21-demo", &entries[0].record_id, entries[0].kind).unwrap_err().to_string().contains("check_record_omitted"));
        assert_eq!(
            store
                .read_archived_artifact("2026-10-21-demo", "review.md")
                .as_deref(),
            Some("carried working ticket")
        );
        assert!(store
            .read_archived_meta("2026-10-21-demo")
            .unwrap()
            .contains("archived_by: Archivist"));
    }
    #[cfg(unix)]
    #[test]
    fn archive_never_follows_source_links_or_publishes_a_partial_copy() {
        let (root, store, mut plan) = fixture();
        plan.retention.review = true;
        let outside = root.path().join("outside.md");
        fs::write(&outside, "outside secret").unwrap();
        std::os::unix::fs::symlink(
            &outside,
            store.layout().change_dir("demo").join("linked.md"),
        )
        .unwrap();
        assert!(archive(&store, "demo", "2026-10-21-demo", &plan).is_err());
        assert!(store.change_exists("demo"));
        assert_eq!(fs::read_to_string(outside).unwrap(), "outside secret");
        assert!(!store.archived_change_exists("2026-10-21-demo"));
    }
}
