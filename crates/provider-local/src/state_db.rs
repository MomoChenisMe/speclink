//! SQLite-backed state store under `<git-common-dir>/speclink/state.db`.
//!
//! 採內嵌 SQL 陣列做 migration runner；v1 schema 包含 `_migrations` 與 `project` 兩張表，
//! v2 schema 追加 `change` 表。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use rusqlite::{Connection, params};
use speclink_provider::ChangeRow;
use thiserror::Error;

/// State.db 操作錯誤。
#[derive(Debug, Error)]
pub enum StateDbError {
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("invalid schema version: expected {expected}, found {found}")]
    SchemaVersion { expected: u32, found: u32 },
    #[error("invalid target version {0}: state.db is already at a higher version")]
    DowngradeNotSupported(u32),
}

impl From<rusqlite::Error> for StateDbError {
    fn from(value: rusqlite::Error) -> Self {
        StateDbError::Sqlite(value.to_string())
    }
}

/// v1 schema：建立 `_migrations` 與 `project` 兩張表。
const MIGRATION_V1_SQL: &str = "
CREATE TABLE _migrations (
    version    INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);
CREATE TABLE project (
    id          TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    working_dir TEXT NOT NULL,
    created_at  TEXT NOT NULL
);
";

/// v2 schema：追加 `change` 表，欄位對應 `spec/change-store/spec.md` example 表。
const MIGRATION_V2_SQL: &str = "
CREATE TABLE change (
    change_id   TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    state       TEXT NOT NULL,
    schema_id   TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
";

/// 內嵌 migration SQL。
///
/// 索引對照表：
///
/// | index | schema version | 內容                              |
/// |-------|----------------|-----------------------------------|
/// | 0     | 1              | `_migrations`、`project` 表       |
/// | 1     | 2              | `change` 表                       |
pub const MIGRATIONS: &[&str] = &[MIGRATION_V1_SQL, MIGRATION_V2_SQL];

/// state.db 的 handle。
pub struct StateDb {
    conn: Connection,
}

impl StateDb {
    /// 開啟或建立 state.db，並設定 `journal_mode=WAL`。
    ///
    /// # Errors
    /// 當 SQLite open 或 PRAGMA 設定失敗時回 [`StateDbError::Sqlite`]。
    pub fn open(path: &Path) -> Result<Self, StateDbError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "wal")?;
        Ok(Self { conn })
    }

    /// 取得目前 schema version；若 `_migrations` 表尚未存在則回 0。
    ///
    /// # Errors
    /// 當 SQLite 查詢失敗（非 missing table）時回 [`StateDbError::Sqlite`]。
    pub fn schema_version(&self) -> Result<u32, StateDbError> {
        let exists: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(0);
        }
        let max: Option<u32> =
            self.conn
                .query_row("SELECT MAX(version) FROM _migrations", [], |r| r.get(0))?;
        Ok(max.unwrap_or(0))
    }

    /// 將 schema 升級到 `target_version`。
    ///
    /// 重複呼叫至同一 `target_version` 為 no-op。每個 migration 在獨立的
    /// transaction 中執行：若某個 migration 中途失敗，已套用的較低版本保留，
    /// 失敗的 version 不會留下半套狀態。
    ///
    /// # Errors
    /// * 當 `target_version` 低於目前 schema version 時回 [`StateDbError::DowngradeNotSupported`]。
    /// * 當 SQL 執行失敗時回 [`StateDbError::Sqlite`]。
    pub fn migrate(&self, target_version: u32) -> Result<(), StateDbError> {
        let current = self.schema_version()?;
        if target_version < current {
            return Err(StateDbError::DowngradeNotSupported(target_version));
        }
        if target_version > MIGRATIONS.len() as u32 {
            return Err(StateDbError::SchemaVersion {
                expected: MIGRATIONS.len() as u32,
                found: target_version,
            });
        }

        for v in (current + 1)..=target_version {
            let sql = MIGRATIONS[(v as usize) - 1];
            self.apply_one_migration(v, sql)?;
        }
        Ok(())
    }

    /// 將單一 migration 套用於目前 connection（transaction-wrapped）。
    ///
    /// 公開為 `pub(crate)` 是為了讓 inline test 注入 fault SQL；prod 路徑只透過
    /// [`Self::migrate`] 呼叫。
    pub(crate) fn apply_one_migration(&self, version: u32, sql: &str) -> Result<(), StateDbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, now_rfc3339()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// 取得底層 [`Connection`] 的不可變借用（給同 crate 內其他模組使用）。
    #[allow(dead_code)]
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    /// 在 `project` 表插入一個 row。
    ///
    /// # Errors
    /// 當 SQLite execute 失敗時回 [`StateDbError::Sqlite`]。
    pub fn insert_project_row(
        &self,
        id: &str,
        instance_id: &str,
        working_dir: &str,
        created_at: &str,
    ) -> Result<(), StateDbError> {
        self.conn.execute(
            "INSERT INTO project (id, instance_id, working_dir, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, instance_id, working_dir, created_at],
        )?;
        Ok(())
    }

    /// 檢查 `project.id` 是否存在。
    ///
    /// # Errors
    /// 當 SQLite 查詢失敗時回 [`StateDbError::Sqlite`]。
    pub fn has_project(&self, id: &str) -> Result<bool, StateDbError> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM project WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    /// 取得 `project` 表內唯一一筆 row 的 `id`。
    ///
    /// # Errors
    /// 當 SQLite 查詢失敗或表內含多筆時回錯。
    pub fn single_project_id(&self) -> Result<Option<String>, StateDbError> {
        let count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))?;
        match count {
            0 => Ok(None),
            1 => {
                let id: String =
                    self.conn
                        .query_row("SELECT id FROM project LIMIT 1", [], |r| r.get(0))?;
                Ok(Some(id))
            }
            _ => Err(StateDbError::Sqlite(format!(
                "expected exactly one project row in state.db, found {count}"
            ))),
        }
    }

    /// 在 `change` 表插入一個 row。`version` 固定為 1。
    ///
    /// # Errors
    /// 當 SQLite execute 失敗（含 UNIQUE constraint）時回 [`StateDbError::Sqlite`]。
    pub fn insert_change_row(
        &self,
        change_id: &str,
        name: &str,
        state: &str,
        schema_id: &str,
        created_at: &str,
        updated_at: &str,
    ) -> Result<(), StateDbError> {
        self.conn.execute(
            "INSERT INTO change (change_id, name, state, schema_id, version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            params![change_id, name, state, schema_id, created_at, updated_at],
        )?;
        Ok(())
    }

    /// 從 `change` 表依 `name` 取得 row；找不到回 `Ok(None)`。
    ///
    /// # Errors
    /// 當 SQLite 查詢失敗時回 [`StateDbError::Sqlite`]。
    pub fn get_change_by_name(&self, name: &str) -> Result<Option<ChangeRow>, StateDbError> {
        let mut stmt = self.conn.prepare(
            "SELECT change_id, name, state, schema_id, version, created_at, updated_at
             FROM change WHERE name = ?1",
        )?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ChangeRow {
                change_id: row.get(0)?,
                name: row.get(1)?,
                state: row.get(2)?,
                schema_id: row.get(3)?,
                version: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 列出所有 change row，依 `updated_at` desc 排序。
    ///
    /// # Errors
    /// 當 SQLite 查詢失敗時回 [`StateDbError::Sqlite`]。
    pub fn list_changes(&self) -> Result<Vec<ChangeRow>, StateDbError> {
        let mut stmt = self.conn.prepare(
            "SELECT change_id, name, state, schema_id, version, created_at, updated_at
             FROM change ORDER BY updated_at DESC, change_id ASC",
        )?;
        let iter = stmt.query_map([], |row| {
            Ok(ChangeRow {
                change_id: row.get(0)?,
                name: row.get(1)?,
                state: row.get(2)?,
                schema_id: row.get(3)?,
                version: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in iter {
            out.push(r?);
        }
        Ok(out)
    }

    /// 從 `change` 表依 `name` 刪除一個 row；回傳是否真的刪到（找不到時為 `false`）。
    ///
    /// # Errors
    /// 當 SQLite execute 失敗時回 [`StateDbError::Sqlite`]。
    pub fn delete_change_by_name(&self, name: &str) -> Result<bool, StateDbError> {
        let affected = self
            .conn
            .execute("DELETE FROM change WHERE name = ?1", params![name])?;
        Ok(affected > 0)
    }
}

fn now_rfc3339() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn open_creates_state_db_in_empty_path() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("state.db");
        assert!(!db_path.exists());
        let _db = StateDb::open(&db_path).expect("open new state.db");
        assert!(db_path.exists(), "state.db file should be created");
    }

    #[test]
    fn migrate_v1_creates_expected_tables() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("state.db");
        let db = StateDb::open(&db_path).expect("open");
        db.migrate(1).expect("migrate to v1");
        assert_eq!(db.schema_version().expect("read version"), 1);

        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let migrations_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = 1",
                [],
                |r| r.get(0),
            )
            .expect("query _migrations");
        assert_eq!(
            migrations_count, 1,
            "_migrations should contain version=1 row"
        );

        let mut stmt = conn
            .prepare("SELECT name FROM pragma_table_info('project') ORDER BY cid")
            .expect("prepare pragma");
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query columns")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(
            cols,
            vec!["id", "instance_id", "working_dir", "created_at"],
            "project columns mismatch"
        );

        let mut stmt = conn
            .prepare("SELECT name, \"notnull\" FROM pragma_table_info('project') ORDER BY cid")
            .expect("prepare pragma");
        let rows: Vec<(String, u32)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
            .expect("query")
            .filter_map(Result::ok)
            .collect();
        let nn: std::collections::HashMap<_, _> = rows.into_iter().collect();
        assert_eq!(nn.get("instance_id").copied(), Some(1));
        assert_eq!(nn.get("working_dir").copied(), Some(1));
        assert_eq!(nn.get("created_at").copied(), Some(1));
    }

    #[test]
    fn migrate_is_idempotent() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("state.db");
        let db = StateDb::open(&db_path).expect("open");
        db.migrate(1).expect("first migrate");
        db.migrate(1).expect("second migrate (no-op)");
        assert_eq!(db.schema_version().expect("version"), 1);

        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
            .expect("count");
        assert_eq!(
            count, 1,
            "migrating twice should not duplicate _migrations rows"
        );
    }

    #[test]
    fn migrate_to_unknown_version_fails_without_partial_state() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("state.db");
        let db = StateDb::open(&db_path).expect("open");
        let err = db.migrate(99).expect_err("expected SchemaVersion error");
        assert!(matches!(err, StateDbError::SchemaVersion { .. }));
        db.migrate(1).expect("retry to v1 should succeed");
        assert_eq!(db.schema_version().expect("version"), 1);
    }

    #[test]
    fn open_sets_wal_journal_mode() {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("state.db");
        let _db = StateDb::open(&db_path).expect("open");
        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .expect("query journal_mode");
        assert_eq!(mode.to_lowercase(), "wal");
    }

    // --- slice-A v2 migration tests ----------------------------------------

    fn open_v1(tmp: &TempDir) -> (StateDb, std::path::PathBuf) {
        let db_path = tmp.path().join("state.db");
        let db = StateDb::open(&db_path).expect("open");
        db.migrate(1).expect("migrate v1");
        (db, db_path)
    }

    #[test]
    fn migrate_v2_first_time_creates_change_table_with_seven_cols() {
        let tmp = TempDir::new().expect("tempdir");
        let (db, db_path) = open_v1(&tmp);

        db.migrate(2).expect("migrate to v2");
        assert_eq!(db.schema_version().expect("read version"), 2);

        let conn = rusqlite::Connection::open(&db_path).expect("reopen");

        // _migrations should contain version=2 row
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = 2",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1, "_migrations should contain version=2 row");

        // change table columns
        let mut stmt = conn
            .prepare("SELECT name FROM pragma_table_info('change') ORDER BY cid")
            .expect("prepare pragma");
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query columns")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(
            cols,
            vec![
                "change_id",
                "name",
                "state",
                "schema_id",
                "version",
                "created_at",
                "updated_at",
            ],
            "change columns mismatch"
        );

        // NOT NULL constraints
        let mut stmt = conn
            .prepare("SELECT name, \"notnull\" FROM pragma_table_info('change') ORDER BY cid")
            .expect("prepare pragma");
        let rows: Vec<(String, u32)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
            .expect("query")
            .filter_map(Result::ok)
            .collect();
        let nn: std::collections::HashMap<_, _> = rows.into_iter().collect();
        for col in [
            "name",
            "state",
            "schema_id",
            "version",
            "created_at",
            "updated_at",
        ] {
            assert_eq!(
                nn.get(col).copied(),
                Some(1),
                "column `{col}` should be NOT NULL"
            );
        }

        // UNIQUE constraint on name: pragma_index_list returns each index;
        // pragma_index_info returns the columns each one covers. We need at least one
        // unique=1 index whose single column is `name`.
        let mut stmt = conn
            .prepare("SELECT name, \"unique\" FROM pragma_index_list('change')")
            .expect("prepare index list");
        let idx_rows: Vec<(String, u32)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
            .expect("query indexes")
            .filter_map(Result::ok)
            .collect();
        let unique_on_name = idx_rows.iter().any(|(idx_name, is_unique)| {
            if *is_unique != 1 {
                return false;
            }
            let cols: Vec<String> = conn
                .prepare(&format!("SELECT name FROM pragma_index_info('{idx_name}')"))
                .ok()
                .and_then(|mut s| {
                    s.query_map([], |r| r.get::<_, String>(0))
                        .ok()
                        .map(|i| i.filter_map(Result::ok).collect())
                })
                .unwrap_or_default();
            cols == vec!["name"]
        });
        assert!(
            unique_on_name,
            "expected a UNIQUE index covering change.name, indexes were: {idx_rows:?}"
        );

        // project table content untouched (no project rows means trivially preserved)
        let project_rows: u32 = conn
            .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
            .expect("count project rows");
        assert_eq!(project_rows, 0);
    }

    #[test]
    fn migrate_v2_idempotent_on_v2_db() {
        let tmp = TempDir::new().expect("tempdir");
        let (db, db_path) = open_v1(&tmp);
        db.migrate(2).expect("first v2");
        db.migrate(2).expect("second v2 (no-op)");
        assert_eq!(db.schema_version().expect("version"), 2);
        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let v2_rows: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = 2",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(v2_rows, 1, "v2 _migrations row must not duplicate");
    }

    #[test]
    fn migrate_v2_rolls_back_on_mid_migration_fault() {
        let tmp = TempDir::new().expect("tempdir");
        let (db, db_path) = open_v1(&tmp);
        // Inject SQL that creates a sentinel table, then a syntactically invalid statement.
        // The whole batch must roll back; sentinel SHALL NOT remain after the call.
        let bad_sql = "
            CREATE TABLE fault_sentinel (id INTEGER PRIMARY KEY);
            SELECT this_function_does_not_exist();
        ";
        let res = db.apply_one_migration(99, bad_sql);
        assert!(res.is_err(), "bad migration SHOULD return error");

        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let sentinel: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fault_sentinel'",
                [],
                |r| r.get(0),
            )
            .expect("query");
        assert_eq!(sentinel, 0, "fault_sentinel table SHALL NOT remain");
        let v99_rows: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = 99",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(
            v99_rows, 0,
            "failed migration SHALL NOT record _migrations row"
        );
    }

    #[test]
    fn migrate_v2_after_v1_creates_change_table() {
        // For the spec scenario "First-time migration from v1 to v2":
        // confirm project table is unchanged after the v2 migration.
        let tmp = TempDir::new().expect("tempdir");
        let (db, db_path) = open_v1(&tmp);
        db.insert_project_row("pid", "iid", "/tmp", "2026-05-22T10:00:00Z")
            .expect("seed project row");

        db.migrate(2).expect("migrate v2");

        let conn = rusqlite::Connection::open(&db_path).expect("reopen");
        let proj_count: u32 = conn
            .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
            .expect("count project");
        assert_eq!(proj_count, 1, "v2 migration MUST NOT alter project table");
        let pid: String = conn
            .query_row("SELECT id FROM project LIMIT 1", [], |r| r.get(0))
            .expect("query");
        assert_eq!(pid, "pid");
    }

    // --- slice-A `change` table helpers ------------------------------------

    fn open_v2(tmp: &TempDir) -> StateDb {
        let db_path = tmp.path().join("state.db");
        let db = StateDb::open(&db_path).expect("open");
        db.migrate(2).expect("migrate v2");
        db
    }

    #[test]
    fn insert_change_row_then_get_change_by_name_roundtrips() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        db.insert_change_row(
            "cid-1",
            "billing-system",
            "proposing",
            "spec-driven",
            "2026-05-22T10:00:00Z",
            "2026-05-22T10:00:00Z",
        )
        .expect("insert change row");
        let row = db
            .get_change_by_name("billing-system")
            .expect("query")
            .expect("row exists");
        assert_eq!(row.change_id, "cid-1");
        assert_eq!(row.name, "billing-system");
        assert_eq!(row.state, "proposing");
        assert_eq!(row.schema_id, "spec-driven");
        assert_eq!(row.version, 1);
        assert_eq!(row.created_at, "2026-05-22T10:00:00Z");
        assert_eq!(row.updated_at, "2026-05-22T10:00:00Z");
    }

    #[test]
    fn get_change_by_name_returns_none_when_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        assert!(db.get_change_by_name("missing").expect("query").is_none());
    }

    #[test]
    fn insert_change_row_rejects_duplicate_name() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        db.insert_change_row(
            "cid-1",
            "billing-system",
            "proposing",
            "spec-driven",
            "2026-05-22T10:00:00Z",
            "2026-05-22T10:00:00Z",
        )
        .expect("first insert");
        let err = db
            .insert_change_row(
                "cid-2",
                "billing-system",
                "proposing",
                "spec-driven",
                "2026-05-22T11:00:00Z",
                "2026-05-22T11:00:00Z",
            )
            .expect_err("duplicate name should fail");
        let msg = format!("{err}");
        assert!(
            msg.to_lowercase().contains("unique"),
            "expected UNIQUE constraint failure, got {msg}"
        );
    }

    #[test]
    fn list_changes_sorted_by_updated_at_desc() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        db.insert_change_row(
            "cid-a",
            "alpha",
            "proposing",
            "spec-driven",
            "2026-05-22T10:00:00Z",
            "2026-05-22T10:00:00Z",
        )
        .expect("insert a");
        db.insert_change_row(
            "cid-c",
            "gamma",
            "proposing",
            "spec-driven",
            "2026-05-22T12:00:00Z",
            "2026-05-22T12:00:00Z",
        )
        .expect("insert c");
        db.insert_change_row(
            "cid-b",
            "beta",
            "proposing",
            "spec-driven",
            "2026-05-22T11:00:00Z",
            "2026-05-22T11:00:00Z",
        )
        .expect("insert b");

        let rows = db.list_changes().expect("list");
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["gamma", "beta", "alpha"]);
    }

    #[test]
    fn list_changes_empty_returns_empty_vec() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        assert!(db.list_changes().expect("list").is_empty());
    }

    #[test]
    fn delete_change_by_name_removes_row() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        db.insert_change_row(
            "cid-1",
            "billing-system",
            "proposing",
            "spec-driven",
            "2026-05-22T10:00:00Z",
            "2026-05-22T10:00:00Z",
        )
        .expect("insert");
        let deleted = db.delete_change_by_name("billing-system").expect("delete");
        assert!(deleted);
        assert!(
            db.get_change_by_name("billing-system")
                .expect("query")
                .is_none()
        );
    }

    #[test]
    fn delete_change_by_name_returns_false_when_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let db = open_v2(&tmp);
        assert!(!db.delete_change_by_name("missing").expect("delete"));
    }
}
