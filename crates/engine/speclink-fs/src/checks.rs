//! Journal only for proposal checks. Never discards the sole recovery copy.
use crate::{security as io, FsStore};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::{
    history::{valid_check_path, validate_id, ChecksCommit, INDEX, JOURNAL},
    security::{data::sha256, prepared::valid_relative},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Backup {
    name: String,
    digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    transaction_id: String,
    before: BTreeMap<String, Option<Backup>>,
}
fn directory(store: &FsStore, change: &str, create: bool) -> Result<Option<PathBuf>> {
    ensure!(
        valid_relative(change) && !change.contains('/'),
        "invalid_change"
    );
    ensure!(
        valid_relative(&store.layout().spec_dir_name),
        "invalid_spec_directory"
    );
    if !store.layout().root.try_exists()? && !create {
        return Ok(None);
    }
    let root = store.layout().root.canonicalize()?;
    let path = root
        .join(&store.layout().spec_dir_name)
        .join("changes")
        .join(change);
    if !io::owned_directory(&path, &root, create)? {
        return Ok(None);
    }
    Ok(Some(path))
}
fn current(root: &Path, relative: &str) -> Result<Option<Vec<u8>>> {
    let path = root.join(relative);
    if !io::owned_directory(path.parent().unwrap(), root, false)? {
        return Ok(None);
    }
    match path.symlink_metadata() {
        Ok(_) => io::read_limited(&path, LIMIT).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
/// Check documents are bounded and opened without following file or parent links.
pub(crate) fn read_document(
    store: &FsStore,
    change: &str,
    relative: &str,
    archived: bool,
) -> Result<Option<String>> {
    ensure!(
        valid_relative(change)
            && !change.contains('/')
            && valid_relative(&store.layout().spec_dir_name)
            && valid_check_path(relative),
        "invalid_check_path"
    );
    let base = store.layout().root.canonicalize()?;
    let changes = base.join(&store.layout().spec_dir_name).join("changes");
    let root = if archived {
        changes.join("archive").join(change)
    } else {
        changes.join(change)
    };
    if !io::owned_directory(&root, &base, false)? {
        return Ok(None);
    }
    current(&root, relative)?
        .map(String::from_utf8)
        .transpose()
        .map_err(Into::into)
}
pub(crate) fn readable(store: &FsStore, change: &str) -> Result<()> {
    if let Some(root) = directory(store, change, false)? {
        ensure!(
            current(&root, JOURNAL)
                .context("history_recovery_required")?
                .is_none(),
            "history_recovery_required"
        );
        if let Some(bytes) = current(&root, INDEX)? {
            let index: speclink_core::history::CheckIndex =
                serde_json::from_slice(&bytes).context("invalid_check_index")?;
            index.validate()?;
        }
    }
    Ok(())
}
fn lock(root: &Path) -> Result<io::FileLock> {
    let lock = io::open_lock(&root.join(".checks.lock"))?;
    lock.try_lock()
        .map_err(|_| anyhow::anyhow!("busy: checks transaction"))?;
    Ok(io::FileLock(lock))
}
fn fault(_point: &str) -> Result<()> {
    #[cfg(test)]
    FAULTS.with(|faults| {
        let mut faults = faults.borrow_mut();
        if faults.first().is_some_and(|point| point == _point) {
            faults.remove(0);
            anyhow::bail!("injected check failure: {_point}");
        }
        Ok(())
    })?;
    Ok(())
}
fn cleanup(root: &Path, id: &str) {
    for prefix in [".checks-backup-", ".checks-stage-"] {
        let _ = fs::remove_dir_all(root.join(format!("{prefix}{id}")));
    }
}
fn recover_locked(root: &Path) -> Result<()> {
    let Some(bytes) = current(root, JOURNAL)? else {
        return Ok(());
    };
    let journal: Journal =
        serde_json::from_slice(&bytes).context("history_recovery_required: invalid journal")?;
    validate_id(&journal.transaction_id).context("history_recovery_required: journal identity")?;
    ensure!(
        journal.before.len() <= 18,
        "history_recovery_required: journal entries"
    );
    let backup_dir = root.join(format!(".checks-backup-{}", journal.transaction_id));
    io::owned_directory(&backup_dir, root, false)?;
    let mut originals = BTreeMap::new();
    for (index, (path, backup)) in journal.before.iter().enumerate() {
        ensure!(
            valid_check_path(path)
                || matches!(path.as_str(), ".openspec.yaml" | "review.md" | "verify.md"),
            "history_recovery_required: unsafe target"
        );
        let original = if let Some(backup) = backup {
            ensure!(
                backup.name == index.to_string(),
                "history_recovery_required: unsafe backup"
            );
            let bytes = io::read_limited(&backup_dir.join(&backup.name), LIMIT)?;
            ensure!(
                sha256(&bytes) == backup.digest,
                "history_recovery_required: backup fingerprint"
            );
            Some(bytes)
        } else {
            None
        };
        originals.insert(path, original);
    }
    for (index, (path, bytes)) in originals.into_iter().enumerate() {
        fault(&format!("recover:{index}"))?;
        let destination = root.join(path);
        if let Some(bytes) = bytes {
            io::owned_directory(destination.parent().unwrap(), root, true)?;
            let temp = backup_dir.join(format!("restore-{index}-{}", ulid::Ulid::new()));
            io::write_new(&temp, &bytes, &|| false)?;
            // Validate existing targets too; never replace an external link.
            let _ = current(root, path)?;
            io::atomic_replace(&temp, &destination)?;
        } else if current(root, path)?.is_some() {
            fs::remove_file(destination)?;
        }
    }
    io::sync_directory(root)?;
    fs::remove_file(root.join(JOURNAL))?;
    cleanup(root, &journal.transaction_id);
    Ok(())
}
pub(crate) fn recover(store: &FsStore, change: &str) -> Result<()> {
    if let Some(root) = directory(store, change, false)? {
        let _lock = lock(&root)?;
        recover_locked(&root).context("history_recovery_required")?;
    }
    Ok(())
}
pub(crate) fn with_write<T>(
    store: &FsStore,
    change: &str,
    action: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let root = directory(store, change, true)?.expect("created directory");
    let _lock = lock(&root)?;
    recover_locked(&root).context("history_recovery_required")?;
    action()
}
pub(crate) fn commit(store: &FsStore, change: &str, commit: &ChecksCommit) -> Result<()> {
    commit.validate()?;
    with_write(store, change, || {
        let root =
            directory(store, change, false)?.ok_or_else(|| anyhow::anyhow!("change not found"))?;
        for (path, expected) in &commit.expected {
            ensure!(
                current(&root, path)?.map(|bytes| sha256(&bytes)) == *expected,
                "revision_conflict"
            );
        }
        let id = ulid::Ulid::new().to_string();
        let stage = root.join(format!(".checks-stage-{id}"));
        let backup = root.join(format!(".checks-backup-{id}"));
        io::owned_directory(&stage, &root, true)?;
        io::owned_directory(&backup, &root, true)?;
        let mut before = BTreeMap::new();
        let targets: BTreeMap<_, _> = commit
            .writes
            .keys()
            .chain(commit.deletes.iter())
            .map(|path| (path.clone(), ()))
            .collect();
        for (index, path) in targets.keys().enumerate() {
            let old = current(&root, path)?;
            let old = if let Some(bytes) = old {
                io::write_new(&backup.join(index.to_string()), &bytes, &|| false)?;
                Some(Backup {
                    name: index.to_string(),
                    digest: sha256(&bytes),
                })
            } else {
                None
            };
            before.insert(path.clone(), old);
        }
        for (index, (_, text)) in commit.writes.iter().enumerate() {
            ensure!(text.len() <= LIMIT, "size_limit: check document");
            io::write_new(&stage.join(index.to_string()), text.as_bytes(), &|| false)?;
        }
        io::sync_directory(&stage)?;
        io::sync_directory(&backup)?;
        let journal = Journal {
            transaction_id: id.clone(),
            before,
        };
        io::write_new(&root.join(JOURNAL), &serde_json::to_vec(&journal)?, &|| {
            false
        })?;
        io::sync_directory(&root)?;
        let result = (|| -> Result<()> {
            fault("journal")?;
            let publish = |path: &str| -> Result<()> {
                let index = commit
                    .writes
                    .keys()
                    .position(|key| key == path)
                    .expect("planned target");
                let destination = root.join(path);
                io::owned_directory(destination.parent().unwrap(), &root, true)?;
                let _ = current(&root, path)?;
                io::atomic_replace(&stage.join(index.to_string()), &destination)?;
                io::sync_directory(destination.parent().unwrap())
            };
            for (index, path) in commit
                .writes
                .keys()
                .filter(|p| !matches!(p.as_str(), INDEX | ".openspec.yaml"))
                .enumerate()
            {
                publish(path)?;
                fault(&format!("publish:{index}"))?;
            }
            for (index, path) in commit.deletes.iter().enumerate() {
                if current(&root, path)?.is_some() {
                    fs::remove_file(root.join(path))?;
                }
                fault(&format!("delete:{index}"))?;
            }
            if commit.writes.contains_key(INDEX) {
                publish(INDEX)?;
                fault("index")?;
            }
            if commit.writes.contains_key(".openspec.yaml") {
                publish(".openspec.yaml")?;
                fault("metadata")?;
            }
            io::sync_directory(&root)?;
            fs::remove_file(root.join(JOURNAL))?;
            Ok(())
        })();
        if let Err(error) = result {
            if let Err(recovery) = recover_locked(&root) {
                anyhow::bail!("history_recovery_required: {recovery}");
            }
            return Err(error);
        }
        cleanup(&root, &id);
        Ok(())
    })
}

#[cfg(test)]
thread_local! { static FAULTS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) }; }
#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::{
        history::{ChecksCommit, JOURNAL},
        security::data::sha256,
        store::Store,
    };
    use std::collections::BTreeMap;
    fn setup() -> (tempfile::TempDir, crate::FsStore, ChecksCommit) {
        let root = tempfile::tempdir().unwrap();
        let store = crate::FsStore::new(root.path(), "openspec");
        store
            .create_change("demo", "schema: spec-driven\n")
            .unwrap();
        store
            .write_artifact("demo", "review.md", "original ticket")
            .unwrap();
        let path = "checks/review/01M5CP8YW64XQ0Y57ACZRH3PPE.md";
        let commit = ChecksCommit {
            writes: BTreeMap::from([
                (path.into(), "completed ticket".into()),
                (
                    "checks/index.json".into(),
                    "{\"formatVersion\":1,\"entries\":[]}".into(),
                ),
                (
                    ".openspec.yaml".into(),
                    "schema: spec-driven\nreviewed_at: 2026-10-20\n".into(),
                ),
            ]),
            deletes: vec!["review.md".into()],
            expected: BTreeMap::from([
                (path.into(), None),
                ("checks/index.json".into(), None),
                (
                    ".openspec.yaml".into(),
                    Some(sha256(b"schema: spec-driven\n")),
                ),
                ("review.md".into(), Some(sha256(b"original ticket"))),
            ]),
        };
        (root, store, commit)
    }
    #[test]
    fn every_check_publish_failure_restores_the_only_ticket_and_metadata() {
        for point in ["journal", "publish:0", "delete:0", "index", "metadata"] {
            let (_root, store, commit) = setup();
            FAULTS.with(|faults| *faults.borrow_mut() = vec![point.into()]);
            assert!(store.commit_checks("demo", &commit).is_err(), "{point}");
            assert_eq!(
                store.read_artifact("demo", "review.md").as_deref(),
                Some("original ticket")
            );
            assert_eq!(
                store.read_change_meta("demo").as_deref(),
                Some("schema: spec-driven\n")
            );
            assert!(!store.artifact_exists("demo", "checks/index.json"));
            assert!(!store.layout().change_dir("demo").join(JOURNAL).exists());
        }
    }
    #[test]
    fn interrupted_recovery_blocks_readers_and_next_write_recovers_before_retrying() {
        let (_root, store, commit) = setup();
        FAULTS.with(|faults| *faults.borrow_mut() = vec!["delete:0".into(), "recover:0".into()]);
        assert!(store
            .commit_checks("demo", &commit)
            .unwrap_err()
            .to_string()
            .contains("history_recovery_required"));
        assert!(store
            .ensure_checks_readable("demo")
            .unwrap_err()
            .to_string()
            .contains("history_recovery_required"));
        assert!(store.layout().change_dir("demo").join(JOURNAL).exists());
        store
            .write_artifact("demo", "proposal.md", "new proposal")
            .unwrap();
        assert_eq!(
            store.read_artifact("demo", "review.md").as_deref(),
            Some("original ticket")
        );
        store.commit_checks("demo", &commit).unwrap();
        assert!(!store.artifact_exists("demo", "review.md"));
        assert!(store
            .read_change_meta("demo")
            .unwrap()
            .contains("reviewed_at"));
    }
    #[test]
    fn comparison_conflicts_and_unsafe_journals_never_publish() {
        let (_root, store, commit) = setup();
        store
            .write_artifact("demo", "review.md", "changed")
            .unwrap();
        assert!(store
            .commit_checks("demo", &commit)
            .unwrap_err()
            .to_string()
            .contains("revision_conflict"));
        assert_eq!(
            store.read_artifact("demo", "review.md").as_deref(),
            Some("changed")
        );
        std::fs::write(
            store.layout().change_dir("demo").join(JOURNAL),
            "{\"transactionId\":\"../../outside\"}",
        )
        .unwrap();
        assert!(store.recover_checks("demo").is_err());
        assert!(store.layout().change_dir("demo").join(JOURNAL).exists());
    }
    #[cfg(unix)]
    #[test]
    fn linked_check_directories_cannot_publish_outside_the_proposal() {
        let (_root, store, commit) = setup();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(
            outside.path(),
            store.layout().change_dir("demo").join("checks"),
        )
        .unwrap();
        assert!(store.commit_checks("demo", &commit).is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
        assert_eq!(
            store.read_artifact("demo", "review.md").as_deref(),
            Some("original ticket")
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_locked_metadata_keeps_the_original_ticket() {
        use std::os::windows::fs::OpenOptionsExt;
        let (_root, store, commit) = setup();
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(store.layout().change_dir("demo").join(".openspec.yaml"))
            .unwrap();
        assert!(store.commit_checks("demo", &commit).is_err());
        drop(held);
        assert_eq!(
            store.read_artifact("demo", "review.md").as_deref(),
            Some("original ticket")
        );
    }
}
