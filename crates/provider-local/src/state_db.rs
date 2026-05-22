//! SQLite-backed state store under `<git-common-dir>/speclink/state.db`.
//!
//! 採內嵌 SQL 陣列做 migration runner；v1 schema 包含 `_migrations` 與 `project` 兩張表。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use rusqlite::{Connection, params};
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

/// 內嵌 migration SQL。
///
/// 索引 0 對應 schema version 1，索引 1 對應 version 2，依此類推。
/// 每個 element 可以包含一條或多條 SQL（用 `;` 分隔，由 SQLite 直接 execute_batch）。
pub const MIGRATIONS: &[&str] = &[
    // v1: 初始 schema
    "
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
    ",
];

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
        // WAL mode 必須持久寫入；某些檔案系統（NFS）不支援會 fallback，這裡接受 SQLite 行為。
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
            let tx = self.conn.unchecked_transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO _migrations (version, applied_at) VALUES (?1, ?2)",
                params![v, now_rfc3339()],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    /// 取得底層 [`Connection`] 的不可變借用（給同 crate 內其他模組使用）。
    #[allow(dead_code)]
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    /// 在 `project` 表插入一個 row。
    ///
    /// 由 runtime 層在 bootstrap staging phase 直接呼叫，避免暴露底層 `Connection`。
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
    /// MVP 一個 state.db 對應一個 project。當表內恰為 1 筆時回 `Ok(Some(id))`；
    /// 0 筆回 `Ok(None)`；多於 1 筆回 `Err(StateDbError::SchemaVersion { ... })`
    /// 作為佔位錯誤（未來多 project capability 會替換）。
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

        // NOT NULL constraint check on every non-PK column.
        let mut stmt = conn
            .prepare("SELECT name, \"notnull\" FROM pragma_table_info('project') ORDER BY cid")
            .expect("prepare pragma");
        let rows: Vec<(String, u32)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
            .expect("query")
            .filter_map(Result::ok)
            .collect();
        // PRIMARY KEY in SQLite reports notnull=0 historically but is implicitly NOT NULL;
        // 其他欄位顯式宣告 NOT NULL，所以 notnull=1。
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
        // Target version exceeds MIGRATIONS length → SchemaVersion error.
        let err = db.migrate(99).expect_err("expected SchemaVersion error");
        assert!(matches!(err, StateDbError::SchemaVersion { .. }));
        // Subsequent valid migrate must still succeed (no partial state was committed).
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
}
