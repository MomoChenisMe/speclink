//! Requirement「重新開啟與中斷後的一致性」.

use crate::support::{
    all, commit, commit_at, event, key, new_store, put, scope, sequences, v, Test, TestDb,
};
use speclink_store_port::{Event, Expected, Op, PortError, StorePort};

fn a_reconnected_store_has_the_same_content() {
    let db = TestDb::new();
    let s = scope("s");
    let keys = [key("a"), key("b"), key("c"), key("d")];

    let store = db.store();
    // Three commits: four documents written, one deleted, three events.
    store
        .commit(
            &s,
            &commit_at(
                10,
                vec![
                    put("a", "A", Expected::Absent),
                    put("b", "B", Expected::Absent),
                ],
                vec![event("one")],
            ),
        )
        .unwrap();
    store
        .commit(
            &s,
            &commit_at(
                20,
                vec![
                    put("c", "C", Expected::Absent),
                    put("d", "D", Expected::Absent),
                ],
                vec![event("two")],
            ),
        )
        .unwrap();
    store
        .commit(
            &s,
            &commit_at(
                30,
                vec![Op::Delete {
                    key: key("a"),
                    expected: v(1),
                }],
                vec![event("three")],
            ),
        )
        .unwrap();
    let listing = store.list(&s, &all()).unwrap();
    let documents = store.read(&s, &keys).unwrap();
    let events = store.events_after(&s, 0, 10).unwrap();
    assert_eq!(listing.version, v(3));
    assert_eq!(listing.entries.len(), 3);
    assert_eq!(events.len(), 3);
    drop(store);

    let store = db.store();
    assert_eq!(store.list(&s, &all()).unwrap(), listing);
    assert_eq!(store.read(&s, &keys).unwrap(), documents);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), events);
    assert_eq!(store.event_floor(&s).unwrap(), 0);

    // The floor is also the same after a new connect.
    store.trim_events(&s, 2).unwrap();
    drop(store);
    let store = db.store();
    assert_eq!(store.event_floor(&s).unwrap(), 2);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [3]);
}

fn a_commit_that_fails_on_its_second_operation_leaves_nothing() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("base", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    let failing = commit(
        vec![
            put("first", "written before the failure", Expected::Absent),
            put("second", "x", Expected::At(v(7))),
        ],
        vec![event("lost")],
    );
    assert_eq!(
        store.commit(&s, &failing),
        Err(PortError::Conflict {
            key: key("second"),
            expected: Expected::At(v(7)),
            actual: None,
        })
    );

    assert_eq!(store.read(&s, &[key("first")]).unwrap(), [None]);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), []);
    assert_eq!(store.list(&s, &all()).unwrap().version, v(1));
}

fn a_commit_that_fails_after_its_first_write_leaves_nothing() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("base", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    // The database refuses a NUL character in the name of the event, after
    // the commit wrote its document in the same transaction.
    let refused = Event::new("refused\u{0}name", serde_json::Value::Null).unwrap();

    let result = store.commit(
        &s,
        &commit(vec![put("written", "x", Expected::Absent)], vec![refused]),
    );

    assert!(
        matches!(result, Err(PortError::Backend { .. })),
        "{result:?}"
    );
    assert_eq!(store.read(&s, &[key("written")]).unwrap(), [None]);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), []);
    assert_eq!(store.list(&s, &all()).unwrap().version, v(1));
}

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "a_reconnected_store_has_the_same_content",
        a_reconnected_store_has_the_same_content,
    ),
    (
        "a_commit_that_fails_on_its_second_operation_leaves_nothing",
        a_commit_that_fails_on_its_second_operation_leaves_nothing,
    ),
    (
        "a_commit_that_fails_after_its_first_write_leaves_nothing",
        a_commit_that_fails_after_its_first_write_leaves_nothing,
    ),
];
