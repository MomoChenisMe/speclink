//! Requirement「開啟時只認自己的配置」.

use crate::support::{all, dir_content, foreign_file, open_error, scope, META_TABLE};
use rusqlite::Connection;
use speclink_store_port::{StorePort, Version};
use speclink_store_port_sqlite::{OpenError, SqlitePort};

#[test]
fn a_missing_file_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();

    assert!(path.is_file());
    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());

    let conn = Connection::open(&path).unwrap();
    let format: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'format'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(format, "speclink-store-port");
    let layout: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'layout_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(layout, "1");
}

#[test]
fn another_marker_is_refused_and_the_file_is_unchanged() {
    let (dir, path) = foreign_file(&format!(
        "{META_TABLE}
         INSERT INTO meta VALUES ('format', 'speclink-team-store');
         INSERT INTO meta VALUES ('schema_version', '1');"
    ));
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: None }
                if format == "speclink-team-store"
        ),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn a_newer_layout_version_is_refused() {
    let (dir, path) = foreign_file(&format!(
        "{META_TABLE}
         INSERT INTO meta VALUES ('format', 'speclink-store-port');
         INSERT INTO meta VALUES ('layout_version', '2');"
    ));
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn tables_without_meta_are_refused() {
    let (dir, path) = foreign_file("CREATE TABLE notes (body TEXT);");
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

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
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn a_meta_table_without_a_marker_is_refused() {
    let (dir, path) = foreign_file(META_TABLE);
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn an_existing_file_with_no_table_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    std::fs::write(&path, "").unwrap();

    let store = SqlitePort::open(&path).unwrap();

    assert_eq!(
        store.list(&scope("any"), &all()).unwrap().version,
        Version::ZERO
    );
    let format: String = Connection::open(&path)
        .unwrap()
        .query_row("SELECT value FROM meta WHERE key = 'format'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(format, "speclink-store-port");
}

#[test]
fn a_meta_table_with_other_columns_is_refused_as_an_unknown_layout() {
    let (dir, path) = foreign_file(
        "CREATE TABLE meta (name TEXT, content TEXT);
         INSERT INTO meta VALUES ('format', 'speclink-store-port');",
    );
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: None,
            layout_version: None,
        }
    );
    assert_eq!(dir_content(dir.path()), before);
}
