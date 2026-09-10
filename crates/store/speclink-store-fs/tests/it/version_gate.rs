//! The version gate of the FS driver: what a data directory must look like
//! before this driver will touch it.
//!
//! A refused open is a read-only event. Every rejection here asserts the
//! directory came out bit-for-bit unchanged — refusing to open something and
//! then leaving a lock file, a meta file or a scope tree inside it would
//! make the refusal a lie, and would be how a foreign directory quietly
//! becomes half a store.

use speclink_store::StoreError;
use speclink_store_fs::layout::{META_FILE, SCHEMA_VERSION, STORE_MARKER};
use speclink_store_fs::FsTeamStore;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Every file under `root`, by relative path, with its bytes. The whole
/// directory tree, so an unwanted write anywhere in it shows up as a diff.
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path.strip_prefix(root).expect("under root").to_path_buf();
                out.insert(relative, std::fs::read(&path).expect("read"));
            }
        }
    }
    out
}

fn meta_text(marker: &str, version: u32) -> String {
    format!("{{\"format\":\"{marker}\",\"schema_version\":{version}}}")
}

#[test]
fn an_empty_directory_initializes_at_the_current_version() {
    let dir = tempfile::tempdir().unwrap();
    FsTeamStore::open(dir.path()).expect("fresh open initializes");

    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.path().join(META_FILE)).unwrap()).unwrap();
    assert_eq!(meta["format"], STORE_MARKER);
    assert_eq!(meta["schema_version"], SCHEMA_VERSION);

    // Reopening an already-initialized directory is the normal case.
    FsTeamStore::open(dir.path()).expect("reopen succeeds");
}

#[test]
fn a_missing_directory_is_created_and_initialized() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("nested").join("store");
    FsTeamStore::open(&root).expect("open creates the data directory");
    assert!(root.join(META_FILE).is_file());
}

#[test]
fn a_newer_schema_version_is_refused_and_bytes_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    FsTeamStore::open(dir.path()).expect("initialize");
    std::fs::write(
        dir.path().join(META_FILE),
        meta_text(STORE_MARKER, SCHEMA_VERSION + 1),
    )
    .unwrap();

    let before = tree(dir.path());
    match FsTeamStore::open(dir.path()) {
        Err(StoreError::Corrupt { reason }) => assert!(
            reason.contains("version"),
            "reason should name the version incompatibility: {reason}"
        ),
        Err(other) => panic!("expected corrupt refusal, got {other:?}"),
        Ok(_) => panic!("expected corrupt refusal, got Ok"),
    }
    assert_eq!(before, tree(dir.path()), "a refused open must not write");
}

#[test]
fn a_corrupt_meta_file_is_refused_and_bytes_unchanged() {
    for (label, meta) in [
        ("unparseable", "{not json".to_string()),
        ("missing version", "{\"format\":\"speclink-team-store-fs\"}".to_string()),
        ("version not a number", meta_text(STORE_MARKER, 1).replace('1', "\"one\"")),
    ] {
        let dir = tempfile::tempdir().unwrap();
        FsTeamStore::open(dir.path()).expect("initialize");
        std::fs::write(dir.path().join(META_FILE), &meta).unwrap();

        let before = tree(dir.path());
        assert!(
            matches!(FsTeamStore::open(dir.path()), Err(StoreError::Corrupt { .. })),
            "{label}: expected corrupt refusal"
        );
        assert_eq!(before, tree(dir.path()), "{label}: a refused open must not write");
    }
}

#[test]
fn a_directory_of_another_driver_is_refused_and_bytes_unchanged() {
    // A meta file shaped right but marking someone else's store.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(META_FILE),
        meta_text("someone-elses-store", SCHEMA_VERSION),
    )
    .unwrap();

    let before = tree(dir.path());
    match FsTeamStore::open(dir.path()) {
        Err(StoreError::Corrupt { reason }) => assert!(
            reason.contains("speclink"),
            "reason should name whose store this is not: {reason}"
        ),
        Err(other) => panic!("expected corrupt refusal, got {other:?}"),
        Ok(_) => panic!("expected corrupt refusal, got Ok"),
    }
    assert_eq!(before, tree(dir.path()), "a refused open must not write");
}

#[test]
fn a_foreign_non_empty_directory_is_refused_and_not_initialized() {
    // Someone points the config at their documents folder. There is no meta
    // file to check, so emptiness is the whole gate: this directory is not
    // ours and must not become ours.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("holiday-photos")).unwrap();
    std::fs::write(dir.path().join("holiday-photos").join("beach.jpg"), b"jpeg").unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"do not delete").unwrap();

    let before = tree(dir.path());
    match FsTeamStore::open(dir.path()) {
        Err(StoreError::Corrupt { reason }) => assert!(
            reason.contains("empty") || reason.contains("not a speclink"),
            "reason should say why it was refused: {reason}"
        ),
        Err(other) => panic!("expected corrupt refusal, got {other:?}"),
        Ok(_) => panic!("expected corrupt refusal, got Ok"),
    }
    assert_eq!(before, tree(dir.path()), "a refused open must not write");
    assert!(!dir.path().join(META_FILE).exists(), "must not initialize over it");
}

#[test]
fn a_path_that_is_a_file_is_a_backend_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    std::fs::write(&path, b"i am a file, not a directory").unwrap();

    match FsTeamStore::open(&path) {
        Err(StoreError::Backend { .. }) => {}
        Err(other) => panic!("expected backend error, got {other:?}"),
        Ok(_) => panic!("expected backend error opening a file as a data directory"),
    }
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"i am a file, not a directory",
        "a refused open must not write"
    );
}
