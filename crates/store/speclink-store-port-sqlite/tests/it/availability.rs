//! Requirement「可用性檢查與失敗的分類」.

use crate::support::{commit, new_store, put, scope};
use rusqlite::Connection;
use speclink_store_port::{Expected, PortError, StorePort};
use speclink_store_port_sqlite::SqlitePort;

#[test]
fn probe_succeeds_on_an_open_store() {
    let (_dir, store) = new_store();
    assert_eq!(store.probe(), Ok(()));
}

/// Unix only: Windows does not let a file move while SQLite has it open, so
/// the situation cannot occur there.
#[cfg(unix)]
#[test]
fn probe_fails_after_the_file_is_moved_away() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();
    store
        .commit(
            &scope("s"),
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    std::fs::rename(&path, dir.path().join("moved.db")).unwrap();

    let result = store.probe();
    assert!(
        matches!(
            result,
            Err(PortError::Backend { .. } | PortError::Unavailable { .. })
        ),
        "{result:?}"
    );
}

#[test]
fn probe_fails_when_the_store_cannot_be_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();
    // Another connection holds the write lock for longer than the busy
    // timeout of the store. A read still works.
    let holder = Connection::open(&path).unwrap();
    holder.execute_batch("BEGIN IMMEDIATE").unwrap();

    let result = store.probe();

    assert!(
        matches!(result, Err(PortError::Unavailable { .. })),
        "{result:?}"
    );
    holder.execute_batch("ROLLBACK").unwrap();
    assert_eq!(store.probe(), Ok(()));
}
