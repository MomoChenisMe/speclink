//! Requirements「事件與寫入同一次交易並可依序號讀回」and「清除舊事件與下限」.

use crate::support::{
    all, commit, event, key, new_store, put, scope, seed_five_events, sequences, v, Test,
};
use speclink_store_port::{Expected, PortError, RecordedEvent, StorePort};

fn two_events_of_one_commit_get_sequence_1_and_2() {
    let (_db, store) = new_store();
    let s = scope("s");
    let written = event("artifact-written");
    let ranked = event("change-ranked");
    store
        .commit(
            &s,
            &commit(
                vec![put("config.yaml", "x", Expected::Absent)],
                vec![written.clone(), ranked.clone()],
            ),
        )
        .unwrap();

    let events = store.events_after(&s, 0, 10).unwrap();
    assert_eq!(
        events,
        [
            (
                1,
                RecordedEvent {
                    name: "artifact-written".into(),
                    payload: written.payload().clone(),
                    version: v(1),
                }
            ),
            (
                2,
                RecordedEvent {
                    name: "change-ranked".into(),
                    payload: ranked.payload().clone(),
                    version: v(1),
                }
            ),
        ]
    );
}

fn a_rejected_commit_keeps_no_event_and_uses_no_sequence() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();

    let rejected = commit(
        vec![put("config.yaml", "x", Expected::At(v(9)))],
        vec![event("lost")],
    );
    assert!(matches!(
        store.commit(&s, &rejected),
        Err(PortError::Conflict { .. })
    ));

    store
        .commit(&s, &commit(Vec::new(), vec![event("second")]))
        .unwrap();
    let events = store.events_after(&s, 0, 10).unwrap();
    let names: Vec<(u64, &str)> = events
        .iter()
        .map(|(seq, event)| (*seq, event.name.as_str()))
        .collect();
    assert_eq!(names, [(1, "first"), (2, "second")]);
}

fn events_after_reads_from_the_middle_up_to_the_limit() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(sequences(&store.events_after(&s, 2, 2).unwrap()), [3, 4]);
}

fn a_scope_with_no_events_reads_as_empty() {
    let (_db, store) = new_store();
    assert_eq!(store.events_after(&scope("never"), 0, 10).unwrap(), []);
    assert_eq!(store.event_floor(&scope("never")).unwrap(), 0);
}

// --- 清除舊事件與下限 ---

fn trim_keeps_only_the_newer_events_and_changes_no_document() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    seed_five_events(&store, &s);
    let before = store.list(&s, &all()).unwrap();

    store.trim_events(&s, 3).unwrap();

    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [4, 5]);
    assert_eq!(store.event_floor(&s).unwrap(), 3);
    assert_eq!(store.list(&s, &all()).unwrap(), before);
    assert_eq!(
        store.read(&s, &[key("doc")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "x"
    );
}

fn the_sequence_continues_after_a_trim() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 5).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], vec![event("next")]),
        )
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_trim_past_the_last_sequence_stops_at_it() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 100).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 5);
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], vec![event("next")]),
        )
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_trim_at_or_below_the_floor_does_nothing() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 3).unwrap();
    store.trim_events(&s, 2).unwrap();
    store.trim_events(&s, 3).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 3);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [4, 5]);
}

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "two_events_of_one_commit_get_sequence_1_and_2",
        two_events_of_one_commit_get_sequence_1_and_2,
    ),
    (
        "a_rejected_commit_keeps_no_event_and_uses_no_sequence",
        a_rejected_commit_keeps_no_event_and_uses_no_sequence,
    ),
    (
        "events_after_reads_from_the_middle_up_to_the_limit",
        events_after_reads_from_the_middle_up_to_the_limit,
    ),
    (
        "a_scope_with_no_events_reads_as_empty",
        a_scope_with_no_events_reads_as_empty,
    ),
    (
        "trim_keeps_only_the_newer_events_and_changes_no_document",
        trim_keeps_only_the_newer_events_and_changes_no_document,
    ),
    (
        "the_sequence_continues_after_a_trim",
        the_sequence_continues_after_a_trim,
    ),
    (
        "a_trim_past_the_last_sequence_stops_at_it",
        a_trim_past_the_last_sequence_stops_at_it,
    ),
    (
        "a_trim_at_or_below_the_floor_does_nothing",
        a_trim_at_or_below_the_floor_does_nothing,
    ),
];
