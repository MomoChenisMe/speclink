//! Requirement「重新開啟與中斷後的一致性」.

use crate::support::{
    all, commit, commit_at, event, key, new_store, put, scope, sequences, tree, v,
};
use speclink_store_port::{Expected, Op, PortError, StorePort};
use speclink_store_port_fs::FsPort;

#[test]
fn a_reopened_store_has_the_same_content() {
    let dir = tempfile::tempdir().unwrap();
    let s = scope("s");
    let keys = [key("a"), key("b"), key("c"), key("d")];

    let store = FsPort::open(dir.path()).unwrap();
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

    let store = FsPort::open(dir.path()).unwrap();
    assert_eq!(store.list(&s, &all()).unwrap(), listing);
    assert_eq!(store.read(&s, &keys).unwrap(), documents);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), events);
    assert_eq!(store.event_floor(&s).unwrap(), 0);

    // The floor is also the same after a reopen.
    store.trim_events(&s, 2).unwrap();
    drop(store);
    let store = FsPort::open(dir.path()).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 2);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [3]);
}

#[test]
fn a_commit_that_fails_on_its_second_operation_writes_no_file() {
    let (dir, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("base", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let before = tree(dir.path());

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
    assert_eq!(tree(dir.path()), before);
}

#[test]
fn files_written_before_the_publish_are_not_visible_and_do_not_stop_the_next_commit() {
    let dir = tempfile::tempdir().unwrap();
    let s = scope("s");
    let store = FsPort::open(dir.path()).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("a", "A", Expected::Absent)], vec![event("one")]),
        )
        .unwrap();
    let listing = store.list(&s, &all()).unwrap();
    drop(store);

    // What a commit leaves when it stops after it wrote its files and before
    // the rename that publishes them.
    let scope_dir = dir.path().join("scopes").join("s");
    let stray_blob = scope_dir.join("blobs").join("999");
    let stray_event = scope_dir.join("events").join("2.json");
    let stray_index = scope_dir.join("index.json.new");
    std::fs::write(&stray_blob, "content that was never published").unwrap();
    std::fs::write(
        &stray_event,
        r#"{ "name": "never-published", "payload": null, "version": 2 }"#,
    )
    .unwrap();
    std::fs::write(&stray_index, "{ half an index").unwrap();

    let store = FsPort::open(dir.path()).unwrap();
    assert_eq!(store.list(&s, &all()).unwrap(), listing);
    assert_eq!(
        store.read(&s, &[key("a")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "A"
    );
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [1]);
    assert!(!stray_blob.exists());
    assert!(!stray_index.exists());

    let version = store
        .commit(
            &s,
            &commit(vec![put("b", "B", Expected::Absent)], vec![event("two")]),
        )
        .unwrap();
    assert_eq!(version, v(2));
    let events = store.events_after(&s, 0, 10).unwrap();
    assert_eq!(sequences(&events), [1, 2]);
    assert_eq!(events[1].1.name, "two");
}
