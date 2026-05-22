//! `LocalChangeStore` — `change` 表 + `.speclink/changes/<name>/` 目錄的 CRUD。
//!
//! 沿用 bootstrap `LocalProjectStore` 的「working_dir + state_root 兩個 PathBuf 欄位 + `new()` constructor」pattern。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use speclink_provider::{ChangeRow, ChangeStore, ProviderError};
use uuid::Uuid;

use crate::paths::change_dir;
use crate::state_db::StateDb;

/// LocalProvider 的 `ChangeStore` 實作。
pub struct LocalChangeStore {
    working_dir: PathBuf,
    state_root: PathBuf,
}

impl LocalChangeStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(working_dir: PathBuf, state_root: PathBuf) -> Self {
        Self {
            working_dir,
            state_root,
        }
    }

    /// Working tree root 路徑。
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// State root 路徑（state.db 所在目錄）。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn open_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let path = self.state_root.join("state.db");
        let db = StateDb::open(&path)
            .map_err(|e| ProviderError::Internal(format!("open state.db: {e}")))?;
        db.migrate(2)
            .map_err(|e| ProviderError::Internal(format!("migrate state.db: {e}")))?;
        Ok(db)
    }
}

#[async_trait]
impl ChangeStore for LocalChangeStore {
    async fn create_change(&self, name: &str, schema_id: &str) -> Result<ChangeRow, ProviderError> {
        let db = self.open_db()?;
        // 先檢查 duplicate（UNIQUE constraint 兜底，但 declared error 比 SQL 失敗訊息漂亮）
        let existing = db
            .get_change_by_name(name)
            .map_err(|e| ProviderError::Internal(format!("query change row: {e}")))?;
        if existing.is_some() {
            return Err(ProviderError::ChangeDuplicateName {
                name: name.to_string(),
            });
        }

        let change_id = Uuid::new_v4().to_string();
        let now = now_rfc3339();

        db.insert_change_row(&change_id, name, "proposing", schema_id, &now, &now)
            .map_err(|e| ProviderError::Internal(format!("insert change row: {e}")))?;

        let dir = change_dir(&self.working_dir, name);
        if let Err(e) = fs::create_dir_all(&dir) {
            // Rollback row when directory cannot be created.
            let _ = db.delete_change_by_name(name);
            return Err(ProviderError::Internal(format!(
                "create change dir {}: {e}",
                dir.display()
            )));
        }

        Ok(ChangeRow {
            change_id,
            name: name.to_string(),
            state: "proposing".to_string(),
            schema_id: schema_id.to_string(),
            version: 1,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError> {
        let db = self.open_db()?;
        db.list_changes()
            .map_err(|e| ProviderError::Internal(format!("list changes: {e}")))
    }

    async fn get_change(&self, name: &str) -> Result<ChangeRow, ProviderError> {
        let db = self.open_db()?;
        db.get_change_by_name(name)
            .map_err(|e| ProviderError::Internal(format!("query change row: {e}")))?
            .ok_or_else(|| ProviderError::ChangeNotFound {
                name: name.to_string(),
            })
    }

    async fn delete_change(&self, name: &str) -> Result<(), ProviderError> {
        let db = self.open_db()?;
        // existence check
        if db
            .get_change_by_name(name)
            .map_err(|e| ProviderError::Internal(format!("query change row: {e}")))?
            .is_none()
        {
            return Err(ProviderError::ChangeNotFound {
                name: name.to_string(),
            });
        }
        // 移除目錄（不存在當 no-op）；任何 IO 失敗回 Internal
        let dir = change_dir(&self.working_dir, name);
        match fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(ProviderError::Internal(format!(
                    "remove change dir {}: {e}",
                    dir.display()
                )));
            }
        }
        // 刪除 row
        let _affected = db
            .delete_change_by_name(name)
            .map_err(|e| ProviderError::Internal(format!("delete change row: {e}")))?;
        Ok(())
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

    fn make_store(tmp: &TempDir) -> LocalChangeStore {
        let working = tmp.path().to_path_buf();
        let state = working.join(".git").join("speclink");
        std::fs::create_dir_all(&state).expect("state dir");
        LocalChangeStore::new(working, state)
    }

    #[tokio::test]
    async fn create_change_writes_row_and_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let row = store
            .create_change("billing-system", "spec-driven")
            .await
            .expect("create");
        assert_eq!(row.name, "billing-system");
        assert_eq!(row.state, "proposing");
        assert_eq!(row.schema_id, "spec-driven");
        assert_eq!(row.version, 1);
        assert!(!row.change_id.is_empty());
        // dir created
        let dir = tmp
            .path()
            .join(".speclink")
            .join("changes")
            .join("billing-system");
        assert!(dir.exists() && dir.is_dir());
    }

    #[tokio::test]
    async fn create_change_then_get_change_roundtrips() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let row = store
            .create_change("billing-system", "spec-driven")
            .await
            .expect("create");
        let fetched = store.get_change("billing-system").await.expect("get");
        assert_eq!(fetched, row);
    }

    #[tokio::test]
    async fn create_change_duplicate_name_rejected() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        store
            .create_change("billing-system", "spec-driven")
            .await
            .expect("first");
        let err = store
            .create_change("billing-system", "spec-driven")
            .await
            .expect_err("second create should fail");
        assert!(
            matches!(err, ProviderError::ChangeDuplicateName { name } if name == "billing-system")
        );
    }

    #[tokio::test]
    async fn get_change_missing_returns_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let err = store
            .get_change("unknown")
            .await
            .expect_err("missing change should error");
        assert!(matches!(err, ProviderError::ChangeNotFound { name } if name == "unknown"));
    }

    #[tokio::test]
    async fn list_changes_empty_returns_empty_vec() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        assert!(store.list_changes().await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn list_changes_sorted_by_updated_at_desc() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        store
            .create_change("alpha", "spec-driven")
            .await
            .expect("a");
        // Sleep 1 second so RFC3339 timestamps differ at second resolution.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        store.create_change("beta", "spec-driven").await.expect("b");
        std::thread::sleep(std::time::Duration::from_millis(1100));
        store
            .create_change("gamma", "spec-driven")
            .await
            .expect("c");
        let rows = store.list_changes().await.expect("list");
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["gamma", "beta", "alpha"]);
    }

    #[tokio::test]
    async fn delete_change_removes_row_and_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        store
            .create_change("billing-system", "spec-driven")
            .await
            .expect("create");
        let dir = tmp
            .path()
            .join(".speclink")
            .join("changes")
            .join("billing-system");
        assert!(dir.exists(), "dir should exist before delete");
        store.delete_change("billing-system").await.expect("delete");
        assert!(!dir.exists(), "dir should be removed after delete");
        let err = store
            .get_change("billing-system")
            .await
            .expect_err("row should be gone");
        assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
    }

    #[tokio::test]
    async fn delete_change_missing_returns_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let err = store
            .delete_change("unknown")
            .await
            .expect_err("missing change should error");
        assert!(matches!(err, ProviderError::ChangeNotFound { name } if name == "unknown"));
    }
}
