//! Requirement「開啟時只認自己的配置」.

use crate::support::{all, open_error, scope, Test, TestDb, META_TABLE};
use speclink_store_port::{PortError, StorePort, Version};
use speclink_store_port_postgres::{OpenError, PostgresPort};

fn a_schema_without_meta_is_initialized_beside_other_tables() {
    let mut db = TestDb::new();
    let mut raw = db.client();
    raw.batch_execute("CREATE TABLE notes (body text); INSERT INTO notes VALUES ('keep me');")
        .unwrap();

    let store = db.store();

    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());
    assert_eq!(
        db.tables(),
        ["documents", "events", "meta", "notes", "scopes"]
    );
    let body: String = raw.query_one("SELECT body FROM notes", &[]).unwrap().get(0);
    assert_eq!(body, "keep me");
    let format: String = raw
        .query_one("SELECT value FROM meta WHERE key = 'format'", &[])
        .unwrap()
        .get(0);
    assert_eq!(format, "speclink-store-port");
    let layout: String = raw
        .query_one("SELECT value FROM meta WHERE key = 'layout_version'", &[])
        .unwrap()
        .get(0);
    assert_eq!(layout, "1");
}

fn another_marker_is_refused_and_no_table_is_added() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-team-store'), ('schema_version', '1');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: None }
                if format == "speclink-team-store"
        ),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

fn a_meta_table_without_a_marker_is_refused() {
    let mut db = TestDb::new();
    db.client().batch_execute(META_TABLE).unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

fn a_newer_layout_version_is_refused() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-store-port'), ('layout_version', '2');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

fn a_meta_table_with_other_columns_is_refused_as_an_unknown_layout() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(
            "CREATE TABLE meta (name text, content text);
             INSERT INTO meta VALUES ('format', 'speclink-store-port');",
        )
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: None,
            layout_version: None,
        }
    );
    assert_eq!(db.tables(), before);
}

fn a_table_with_a_store_name_and_no_meta_is_refused() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute("CREATE TABLE documents (id int);")
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(err, OpenError::Store(PortError::Backend { .. })),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "a_schema_without_meta_is_initialized_beside_other_tables",
        a_schema_without_meta_is_initialized_beside_other_tables,
    ),
    (
        "another_marker_is_refused_and_no_table_is_added",
        another_marker_is_refused_and_no_table_is_added,
    ),
    (
        "a_meta_table_without_a_marker_is_refused",
        a_meta_table_without_a_marker_is_refused,
    ),
    (
        "a_newer_layout_version_is_refused",
        a_newer_layout_version_is_refused,
    ),
    (
        "a_meta_table_with_other_columns_is_refused_as_an_unknown_layout",
        a_meta_table_with_other_columns_is_refused_as_an_unknown_layout,
    ),
    (
        "a_table_with_a_store_name_and_no_meta_is_refused",
        a_table_with_a_store_name_and_no_meta_is_refused,
    ),
];
