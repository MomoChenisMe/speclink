//! Requirement「兩個儲存物件開同一份資料」.

use crate::support::{all, commit, key, put, scope, v};
use rusqlite::Connection;
use speclink_store_port::{Expected, PortError, StorePort, Version};
use speclink_store_port_sqlite::SqlitePort;
use std::sync::Barrier;
use std::thread;

#[test]
fn a_commit_by_another_object_makes_the_old_version_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let first = SqlitePort::open(&path).unwrap();
    let second = SqlitePort::open(&path).unwrap();
    let s = scope("s");

    first
        .commit(
            &s,
            &commit(vec![put("k", "v1", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    first
        .commit(
            &s,
            &commit(vec![put("k", "from first", Expected::At(v(1)))], Vec::new()),
        )
        .unwrap();

    let stale = commit(
        vec![put("k", "from second", Expected::At(v(1)))],
        Vec::new(),
    );
    assert_eq!(
        second.commit(&s, &stale),
        Err(PortError::Conflict {
            key: key("k"),
            expected: Expected::At(v(1)),
            actual: Some(v(2)),
        })
    );
    assert_eq!(
        second.read(&s, &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "from first"
    );
}

#[test]
fn two_objects_that_commit_the_same_absent_key_have_one_winner() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let stores = [
        SqlitePort::open(&path).unwrap(),
        SqlitePort::open(&path).unwrap(),
    ];
    let s = scope("s");
    let barrier = Barrier::new(2);

    let results: Vec<Result<Version, PortError>> = thread::scope(|threads| {
        let handles: Vec<_> = stores
            .iter()
            .map(|store| {
                threads.spawn(|| {
                    barrier.wait();
                    store.commit(
                        &s,
                        &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
                    )
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let winners = results.iter().filter(|r| r.is_ok()).count();
    let conflicts = results
        .iter()
        .filter(|r| matches!(r, Err(PortError::Conflict { .. })))
        .count();
    assert_eq!((winners, conflicts), (1, 1), "{results:?}");
    assert_eq!(stores[0].list(&s, &all()).unwrap().version, v(1));
}

#[test]
fn a_writer_that_cannot_get_the_lock_reports_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();
    let s = scope("s");

    // Another connection holds the write lock for longer than the busy
    // timeout of the store.
    let holder = Connection::open(&path).unwrap();
    holder.execute_batch("BEGIN IMMEDIATE").unwrap();

    let result = store.commit(
        &s,
        &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
    );
    assert!(
        matches!(result, Err(PortError::Unavailable { .. })),
        "{result:?}"
    );

    holder.execute_batch("ROLLBACK").unwrap();
    assert_eq!(store.read(&s, &[key("k")]).unwrap(), [None]);
}
