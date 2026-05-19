//! Local provider 的 SQLite 狀態庫：第一版 schema 包含一張 `in_progress_change` 表。
//!
//! 所有公開 API 為 async；內部呼叫 [`tokio::task::spawn_blocking`] 包裝同步 `rusqlite` 操作，
//! state 由 [`std::sync::Mutex`] 保護以避免併發寫競爭。連線時啟用 `PRAGMA journal_mode = WAL`。
//!
//! 註：設計文件原本提到 `tokio::sync::Mutex`；在 `spawn_blocking` 內無法 await 該 mutex，
//! 因此實際採 `std::sync::Mutex` — 寫操作短暫阻塞 worker thread 由 `spawn_blocking` 承擔。

use crate::error::StateDbError;
use provider::model::ChangeId;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// CLI 支援的最高 schema 版本。
pub const SCHEMA_VERSION: u32 = 1;

/// 初始化 schema v1 的 SQL。
const CREATE_TABLES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS in_progress_change (
    change_id  TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
"#;

/// 對 SQLite state.db 的 async 封裝。
#[derive(Debug)]
pub struct StateDb {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl StateDb {
    /// 開啟或建立 `state.db`，並執行 schema migration（MVP 僅版本 1）。
    pub async fn open(path: &Path) -> Result<Self, StateDbError> {
        let path = path.to_path_buf();
        let path_for_blocking = path.clone();
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection, StateDbError> {
            let conn = Connection::open(&path_for_blocking)?;
            // WAL：允許 reader 與 writer 同時進行。
            conn.pragma_update(None, "journal_mode", "WAL")?;
            let mut version: u32 =
                conn.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))?;
            if version == 0 {
                conn.execute_batch(CREATE_TABLES_SQL)?;
                conn.pragma_update(None, "user_version", SCHEMA_VERSION as i64)?;
                version = SCHEMA_VERSION;
            }
            if version > SCHEMA_VERSION {
                return Err(StateDbError::IncompatibleVersion {
                    expected: SCHEMA_VERSION,
                    found: version,
                });
            }
            Ok(conn)
        })
        .await??;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
        })
    }

    /// 取得當前的 `PRAGMA journal_mode`（測試用途）。
    pub async fn journal_mode(&self) -> Result<String, StateDbError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> Result<String, StateDbError> {
            let conn = conn.lock().map_err(|_| StateDbError::Internal {
                message: "mutex poisoned".to_string(),
            })?;
            let mode: String =
                conn.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))?;
            Ok(mode)
        })
        .await?
    }

    /// 取得當前的 `PRAGMA user_version`（測試用途）。
    pub async fn user_version(&self) -> Result<u32, StateDbError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> Result<u32, StateDbError> {
            let conn = conn.lock().map_err(|_| StateDbError::Internal {
                message: "mutex poisoned".to_string(),
            })?;
            let v = conn.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))?;
            Ok(v)
        })
        .await?
    }

    /// 將指定 change id 寫入 `in_progress_change`，覆寫先前的單筆記錄。
    ///
    /// 表 schema 以 `change_id` 為 PK；為達成 spec `in_progress_change semantics` 規定的
    /// 「at most one row」，本實作以 DELETE + INSERT OR REPLACE 包在 transaction 內：
    /// `INSERT OR REPLACE` 處理同 id 重複呼叫，`DELETE` 處理不同 id 的覆寫。
    pub async fn set_in_progress(&self, change_id: &ChangeId) -> Result<(), StateDbError> {
        let conn = self.conn.clone();
        let id = change_id.as_str().to_string();
        tokio::task::spawn_blocking(move || -> Result<(), StateDbError> {
            let mut conn = conn.lock().map_err(|_| StateDbError::Internal {
                message: "mutex poisoned".to_string(),
            })?;
            // ISO 8601 UTC，秒精度。
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let tx = conn.transaction()?;
            tx.execute("DELETE FROM in_progress_change", [])?;
            tx.execute(
                "INSERT OR REPLACE INTO in_progress_change (change_id, created_at) VALUES (?1, ?2)",
                rusqlite::params![id, now],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await?
    }

    /// 讀取當前 `in_progress_change`，未設定時回傳 `None`。
    pub async fn get_in_progress(&self) -> Result<Option<ChangeId>, StateDbError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<ChangeId>, StateDbError> {
            let conn = conn.lock().map_err(|_| StateDbError::Internal {
                message: "mutex poisoned".to_string(),
            })?;
            let mut stmt = conn.prepare("SELECT change_id FROM in_progress_change LIMIT 1")?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                Ok(Some(ChangeId::from(id)))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    /// 取得當前 DB 檔案路徑。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use crate::state_db::StateDb;
    use provider::model::ChangeId;
    use tempfile::TempDir;

    #[tokio::test]
    async fn new_db_initializes_schema_version_1() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.db");
        let db = StateDb::open(&path).await.expect("open");
        assert_eq!(db.user_version().await.expect("user_version"), 1);
    }

    #[tokio::test]
    async fn insert_or_replace_overwrites_previous_row() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.db");
        let db = StateDb::open(&path).await.expect("open");
        db.set_in_progress(&ChangeId::from("first"))
            .await
            .expect("first insert");
        db.set_in_progress(&ChangeId::from("second"))
            .await
            .expect("second insert");
        let current = db
            .get_in_progress()
            .await
            .expect("get")
            .expect("must have row");
        assert_eq!(current.as_str(), "second");
    }

    #[tokio::test]
    async fn wal_journal_mode_enabled() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.db");
        let db = StateDb::open(&path).await.expect("open");
        let mode = db.journal_mode().await.expect("journal_mode");
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[tokio::test]
    async fn sequential_set_in_progress_does_not_corrupt_db() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.db");
        let db = StateDb::open(&path).await.expect("open");
        // 模擬兩次連續 propose create 對應的 set_in_progress 呼叫
        db.set_in_progress(&ChangeId::from("change-one"))
            .await
            .expect("first");
        db.set_in_progress(&ChangeId::from("change-two"))
            .await
            .expect("second");
        // 確認 DB 仍可讀取，且最後一筆覆寫前一筆
        let current = db
            .get_in_progress()
            .await
            .expect("get")
            .expect("must have row");
        assert_eq!(current.as_str(), "change-two");
        // 重新開啟 DB 確認資料未損毀
        drop(db);
        let db2 = StateDb::open(&path).await.expect("reopen");
        let current2 = db2
            .get_in_progress()
            .await
            .expect("get")
            .expect("must have row");
        assert_eq!(current2.as_str(), "change-two");
    }

    #[tokio::test]
    async fn opening_future_version_db_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.db");
        // 預先建立一個 user_version=2 的 DB
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", 2_i64).unwrap();
        }
        let err = StateDb::open(&path).await.expect_err("expect error");
        assert_eq!(err.error_code(), "internal.error");
    }
}
