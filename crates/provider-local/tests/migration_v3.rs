//! State.db v3 migration integration tests + v2-binary-refuses-v3 regression test。
//!
//! 對應 `state-machine` capability requirement「State.db schema MUST be upgraded to
//! version 3 with `actor_json` column, `all_tasks_done` column, and `state_transition`
//! table」與「v2 binary refuses to open a v3 database」scenario。

use rusqlite::Connection;
use speclink_provider_local::{MIGRATIONS, StateDb, StateDbError};
use tempfile::TempDir;

fn open_v2_then_v3(tmp: &TempDir) -> (StateDb, std::path::PathBuf) {
    let db_path = tmp.path().join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(3).expect("migrate v3 from empty");
    (db, db_path)
}

#[test]
fn v3_migration_adds_actor_json_and_all_tasks_done_columns_to_change_table() {
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v2_then_v3(&tmp);
    assert_eq!(db.schema_version().expect("version"), 3);

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
    assert!(names.contains(&"actor_json"), "missing actor_json column");
    assert!(
        names.contains(&"all_tasks_done"),
        "missing all_tasks_done column"
    );
    let actor_col = cols.iter().find(|(n, _, _)| n == "actor_json").unwrap();
    assert_eq!(actor_col.1, 0, "actor_json SHALL be NULLABLE");
    let done_col = cols.iter().find(|(n, _, _)| n == "all_tasks_done").unwrap();
    assert_eq!(done_col.1, 1, "all_tasks_done SHALL be NOT NULL");
    assert_eq!(done_col.2.as_deref(), Some("0"), "default SHALL be 0");
}

#[test]
fn v3_migration_creates_state_transition_table_with_exact_columns() {
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v2_then_v3(&tmp);
    let conn = Connection::open(&db_path).expect("reopen");
    let mut stmt = conn
        .prepare("SELECT name, \"notnull\" FROM pragma_table_info('state_transition') ORDER BY cid")
        .expect("prepare");
    let cols: Vec<(String, u32)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
        .expect("query")
        .filter_map(Result::ok)
        .collect();
    let names: Vec<_> = cols.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "transition_id",
            "change_id",
            "from_state",
            "to_state",
            "actor_json",
            "transitioned_at",
            "reason",
        ],
        "state_transition column ordering / set mismatch"
    );
    // 6 of 7 cols NOT NULL (everything except actor_json)
    let nn: std::collections::HashMap<_, _> = cols.iter().cloned().collect();
    for col in [
        "transition_id",
        "change_id",
        "from_state",
        "to_state",
        "transitioned_at",
        "reason",
    ] {
        assert_eq!(nn.get(col).copied(), Some(1), "{col} SHALL be NOT NULL");
    }
    assert_eq!(
        nn.get("actor_json").copied(),
        Some(0),
        "actor_json SHALL be NULLABLE"
    );
}

#[test]
fn v3_migration_creates_change_time_index_with_descending_order() {
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v2_then_v3(&tmp);
    let conn = Connection::open(&db_path).expect("reopen");
    // Confirm the index exists by name.
    let count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_state_transition_change_time'",
            [],
            |r| r.get(0),
        )
        .expect("query index");
    assert_eq!(count, 1, "expected idx_state_transition_change_time");
    // Confirm covered columns via pragma_index_info.
    let mut stmt = conn
        .prepare(
            "SELECT name FROM pragma_index_info('idx_state_transition_change_time') ORDER BY seqno",
        )
        .expect("prepare");
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(cols, vec!["change_id", "transitioned_at"]);
}

#[test]
fn v3_migration_is_idempotent_on_retry() {
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v2_then_v3(&tmp);
    db.migrate(3).expect("second v3 (no-op)");
    let conn = Connection::open(&db_path).expect("reopen");
    let v3_rows: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = 3",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(v3_rows, 1, "v3 _migrations row must not duplicate");
}

#[test]
fn v3_migration_preserves_existing_v2_data() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(2).expect("v2");
    db.insert_change_row(
        "cid-1",
        "demo",
        "proposing",
        "spec-driven",
        "2026-05-22T10:00:00Z",
        "2026-05-22T10:00:00Z",
    )
    .expect("insert change row pre-v3");
    db.migrate(3).expect("v3");

    // existing row preserved
    let row = db
        .get_change_by_name("demo")
        .expect("query")
        .expect("row exists");
    assert_eq!(row.change_id, "cid-1");
    assert_eq!(row.state, "proposing");
    assert_eq!(row.version, 1);

    // new state machine view shows defaults
    let view = db.read_change_state_row("demo").expect("state view");
    assert_eq!(view.change_id, "cid-1");
    assert_eq!(view.state, "proposing");
    assert_eq!(view.version, 1);
    assert!(view.actor_json.is_none(), "actor_json SHALL default NULL");
    assert!(!view.all_tasks_done, "all_tasks_done SHALL default false");
}

#[test]
fn v3_migration_partial_rollback_leaves_no_orphan_state() {
    // Generic partial-rollback semantic is covered by `migrate_v2_rolls_back_on_mid_migration_fault`
    // in state_db inline tests; here we confirm the equivalent for v3 retry path：
    // attempt to migrate to a version above MIGRATIONS.len() must not corrupt schema.
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v2_then_v3(&tmp);
    let db2 = StateDb::open(&db_path).expect("reopen");
    let err = db2
        .migrate(99)
        .expect_err("target above MIGRATIONS.len() should fail");
    assert!(matches!(err, StateDbError::SchemaVersion { .. }));
    assert_eq!(db2.schema_version().expect("version"), 3);
}

#[test]
fn v2_binary_refuses_to_open_a_v3_database() {
    // Seed a v3 db, then simulate a v2 binary by asking the runtime helper for
    // supported_max = 2. The helper SHALL reject with SchemaVersion { expected: 2, found: 3 }.
    let tmp = TempDir::new().expect("tempdir");
    let (_db, db_path) = open_v2_then_v3(&tmp);
    let v2_binary = StateDb::open(&db_path).expect("reopen");
    let err = v2_binary
        .assert_schema_supported(2)
        .expect_err("v2 binary opening v3 db SHALL fail");
    match err {
        StateDbError::SchemaVersion { expected, found } => {
            assert_eq!(expected, 2, "expected = supported_max");
            assert_eq!(found, 3, "found = on-disk schema_version");
        }
        other => panic!("expected SchemaVersion, got {other:?}"),
    }
}

#[test]
fn v3_migrations_array_has_at_least_three_entries() {
    // 原本 A3 期間鎖死 len == 3；A4 起 MIGRATIONS 隨 schema version 增長，改成 >=。
    // 對應 schema version 的精確 len 檢查由各 slice 自己的 migration_v{N}.rs 維護。
    assert!(
        MIGRATIONS.len() >= 3,
        "MIGRATIONS array SHALL be at least length 3 (A3 baseline)"
    );
}
