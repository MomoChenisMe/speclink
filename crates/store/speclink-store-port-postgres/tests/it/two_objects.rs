//! Requirement「兩個儲存物件開同一份資料」.

use crate::support::{all, commit, key, new_store, put, scope, v, Test, TestDb};
use postgres::Client;
use speclink_store_port::{Expected, PortError, StorePort, Version};
use std::sync::Barrier;
use std::thread;
use std::time::{Duration, Instant};

fn a_commit_by_another_object_makes_the_old_version_conflict() {
    let db = TestDb::new();
    let (first, second) = (db.store(), db.store());
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

fn two_objects_that_commit_the_same_absent_key_have_one_winner() {
    let db = TestDb::new();
    let stores = [db.store(), db.store()];
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

/// Another connection holds the row lock of scope `held` for as long as the
/// returned transaction lives.
fn hold_scope_lock<'a>(holder: &'a mut Client, held: &str) -> postgres::Transaction<'a> {
    let mut tx = holder.transaction().unwrap();
    let locked = tx
        .query("SELECT 1 FROM scopes WHERE scope = $1 FOR UPDATE", &[&held])
        .unwrap();
    assert_eq!(locked.len(), 1, "the scope must have its row");
    tx
}

fn commits_on_different_scopes_do_not_wait_for_each_other() {
    let (db, store) = new_store();
    let (a, b) = (scope("A"), scope("B"));
    store
        .commit(
            &a,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let mut holder = db.client();
    let lock_on_a = hold_scope_lock(&mut holder, "A");

    // A commit that waited for scope A would end in a lock timeout.
    let started = Instant::now();
    let result = store.commit(
        &b,
        &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
    );
    assert_eq!(result, Ok(v(1)));
    assert!(started.elapsed() < Duration::from_secs(4));

    lock_on_a.rollback().unwrap();
}

fn a_commit_that_cannot_get_the_scope_lock_reports_unavailable() {
    let (db, store) = new_store();
    let a = scope("A");
    store
        .commit(
            &a,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let mut holder = db.client();
    let lock_on_a = hold_scope_lock(&mut holder, "A");

    let result = store.commit(
        &a,
        &commit(vec![put("k2", "x", Expected::Absent)], Vec::new()),
    );
    assert!(
        matches!(result, Err(PortError::Unavailable { .. })),
        "{result:?}"
    );

    lock_on_a.rollback().unwrap();
    assert_eq!(store.read(&a, &[key("k2")]).unwrap(), [None]);
    assert_eq!(
        store.commit(
            &a,
            &commit(vec![put("k2", "x", Expected::Absent)], Vec::new())
        ),
        Ok(v(2))
    );
}

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "a_commit_by_another_object_makes_the_old_version_conflict",
        a_commit_by_another_object_makes_the_old_version_conflict,
    ),
    (
        "two_objects_that_commit_the_same_absent_key_have_one_winner",
        two_objects_that_commit_the_same_absent_key_have_one_winner,
    ),
    (
        "commits_on_different_scopes_do_not_wait_for_each_other",
        commits_on_different_scopes_do_not_wait_for_each_other,
    ),
    (
        "a_commit_that_cannot_get_the_scope_lock_reports_unavailable",
        a_commit_that_cannot_get_the_scope_lock_reports_unavailable,
    ),
];
