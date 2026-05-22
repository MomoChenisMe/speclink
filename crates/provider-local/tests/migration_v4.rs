//! State.db v4 migration integration tests + v3-binary-refuses-v4 regression test。
//!
//! 對應 `archive-runner` capability requirement「state.db SHALL be upgraded to
//! version 4 with `archived_at` column on the `change` table」與 design 決策
//! 「State.db v4 migration：加 column vs 新表」。

use rusqlite::Connection;
use speclink_provider_local::{MIGRATIONS, StateDb, StateDbError};
use tempfile::TempDir;

fn open_v3_then_v4(tmp: &TempDir) -> (StateDb, std::path::PathBuf) {
    let db_path = tmp.path().join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(4).expect("migrate v4 from empty");
    (db, db_path)
}

#[test]
fn v4_migration_adds_archived_at_column_to_change_table() {
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v3_then_v4(&tmp);
    assert_eq!(db.schema_version().expect("version"), 4);

    let conn = Connection::open(&db_path).expect("reopen");
    let mut stmt = conn
        .prepare(
            "SELECT name, \"notnull\", dflt_value FROM pragma_table_info('change') ORDER BY cid",
        )
        .expect("prepare pragma");
    #[allow(clippy::type_complexity)]
    let cols: Vec<(String, u32, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .expect("query columns")
        .filter_map(Result::ok)
        .collect();
    let names: Vec<_> = cols.iter().map(|(n, _, _)| n.as_str()).collect();
    assert!(names.contains(&"archived_at"), "missing archived_at column");
    let col = cols.iter().find(|(n, _, _)| n == "archived_at").unwrap();
    assert_eq!(col.1, 0, "archived_at SHALL be NULLABLE");
    assert!(
        col.2.is_none(),
        "archived_at SHALL NOT have a default value"
    );
}

#[test]
fn v4_migration_writes_migrations_row_with_version_4() {
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v3_then_v4(&tmp);
    let conn = Connection::open(&db_path).expect("reopen");
    let v4_rows: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = 4",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(v4_rows, 1, "exactly one v4 _migrations row");
}

#[test]
fn v4_migration_leaves_existing_change_rows_archived_at_null() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(3).expect("v3");
    db.insert_change_row(
        "cid-1",
        "demo",
        "proposing",
        "spec-driven",
        "2026-05-22T10:00:00Z",
        "2026-05-22T10:00:00Z",
    )
    .expect("insert change row pre-v4");
    db.migrate(4).expect("v4");

    let conn = Connection::open(&db_path).expect("reopen");
    let archived_at: Option<String> = conn
        .query_row(
            "SELECT archived_at FROM change WHERE change_id = ?1",
            ["cid-1"],
            |r| r.get(0),
        )
        .expect("query");
    assert!(
        archived_at.is_none(),
        "existing change rows SHALL have NULL archived_at after v4 migration"
    );

    let row = db
        .get_change_by_name("demo")
        .expect("query")
        .expect("row exists");
    assert_eq!(row.change_id, "cid-1");
    assert_eq!(row.state, "proposing");
    assert_eq!(row.version, 1);
}

#[test]
fn v4_migration_is_idempotent_on_retry() {
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v3_then_v4(&tmp);
    db.migrate(4).expect("second v4 (no-op)");
    let conn = Connection::open(&db_path).expect("reopen");
    let v4_rows: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = 4",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(v4_rows, 1, "v4 _migrations row must not duplicate");
}

#[test]
fn v4_migration_partial_rollback_leaves_no_orphan_state() {
    // 與 v3 partial-rollback test 對齊：target 超過 MIGRATIONS.len() 失敗
    // 後 schema_version 仍維持 4。
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v3_then_v4(&tmp);
    let db2 = StateDb::open(&db_path).expect("reopen");
    let err = db2
        .migrate(99)
        .expect_err("target above MIGRATIONS.len() should fail");
    assert!(matches!(err, StateDbError::SchemaVersion { .. }));
    assert_eq!(db2.schema_version().expect("version"), 4);
}

#[test]
fn v3_binary_refuses_to_open_a_v4_database() {
    // Seed a v4 db, then simulate a v3 binary by asking the runtime helper for
    // supported_max = 3. The helper SHALL reject with SchemaVersion { expected: 3, found: 4 }.
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v3_then_v4(&tmp);
    let v3_binary = StateDb::open(&db_path).expect("reopen");
    let err = v3_binary
        .assert_schema_supported(3)
        .expect_err("v3 binary opening v4 db SHALL fail");
    match err {
        StateDbError::SchemaVersion { expected, found } => {
            assert_eq!(expected, 3, "expected = supported_max");
            assert_eq!(found, 4, "found = on-disk schema_version");
        }
        other => panic!("expected SchemaVersion, got {other:?}"),
    }
}

#[test]
fn v4_migrations_array_has_exactly_four_entries() {
    assert_eq!(MIGRATIONS.len(), 4, "MIGRATIONS array SHALL be length 4");
}
