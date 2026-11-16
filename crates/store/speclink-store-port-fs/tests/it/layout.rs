//! Requirement「開啟時只認自己的配置」.

use crate::support::{all, commit, foreign_dir, open_error, put, scope, tree, v};
use speclink_store_port::{Expected, StorePort, Version};
use speclink_store_port_fs::{FsPort, OpenError};
use std::path::Path;

fn assert_new_empty_store(root: &Path, store: &FsPort) {
    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["format"], "speclink-store-port");
    assert_eq!(meta["layout_version"], 1);
}

#[test]
fn a_missing_directory_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = FsPort::open(&root).unwrap();
    assert_new_empty_store(&root, &store);
}

#[test]
fn an_empty_directory_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsPort::open(dir.path()).unwrap();
    assert_new_empty_store(dir.path(), &store);
}

#[test]
fn another_marker_is_refused_and_the_directory_is_unchanged() {
    let (_dir, root) =
        foreign_dir(r#"{ "format": "speclink-team-store-fs", "layout_version": 1 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), .. }
                if format == "speclink-team-store-fs"
        ),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_newer_layout_version_is_refused() {
    let (_dir, root) = foreign_dir(r#"{ "format": "speclink-store-port", "layout_version": 2 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_directory_with_other_files_and_no_meta_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not a store").unwrap();
    let before = tree(dir.path());

    let err = open_error(FsPort::open(dir.path()));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout {
                format: None,
                layout_version: None
            }
        ),
        "{err:?}"
    );
    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_meta_file_without_a_marker_is_refused() {
    let (_dir, root) = foreign_dir(r#"{ "layout_version": 1 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_half_finished_initialization_opens_as_a_new_empty_store() {
    // What an interrupted first open leaves: the example's own empty
    // directory and its lock file, and no `meta.json`.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("scopes")).unwrap();
    std::fs::write(dir.path().join("lock"), "").unwrap();

    let store = FsPort::open(dir.path()).unwrap();

    assert_new_empty_store(dir.path(), &store);
    store
        .commit(
            &scope("s"),
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    assert_eq!(store.list(&scope("s"), &all()).unwrap().version, v(1));
}

#[test]
fn a_file_in_the_scopes_directory_does_not_stop_the_open() {
    let dir = tempfile::tempdir().unwrap();
    let s = scope("s");
    let store = FsPort::open(dir.path()).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    drop(store);
    // What a file manager leaves behind, such as `.DS_Store` on macOS.
    std::fs::write(dir.path().join("scopes").join(".DS_Store"), "not a scope").unwrap();

    let store = FsPort::open(dir.path()).unwrap();
    assert_eq!(store.list(&s, &all()).unwrap().version, v(1));
}
