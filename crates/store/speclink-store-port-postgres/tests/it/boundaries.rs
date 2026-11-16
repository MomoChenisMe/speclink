//! The input boundaries from the audit (task 4.2).

use crate::support::{
    commit, event, key, new_store, open_error, put, scope, seed_five_events, sequences, Test,
    TestDb, META_TABLE,
};
use speclink_store_port::{DocKey, Expected, KeyPrefix, PortError, StorePort};
use speclink_store_port_postgres::{url_embeds_password, OpenError, PostgresPort};

fn a_connection_string_that_cannot_be_read_is_refused_with_the_reason() {
    let err = open_error(PostgresPort::connect(
        "postgres://user:s3cret@localhost:not-a-port/db",
    ));
    assert!(
        matches!(
            &err,
            OpenError::Store(PortError::Backend { message })
                if message.contains("cannot read the PostgreSQL connection string")
        ),
        "{err:?}"
    );
    assert!(!err.to_string().contains("s3cret"), "{err}");
    // No password to report for a string that cannot be read.
    assert!(!url_embeds_password(
        "postgres://user:s3cret@localhost:not-a-port/db"
    ));
}

fn events_after_with_limit_zero_or_the_largest_number_is_empty() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(store.events_after(&s, 0, 0).unwrap(), []);
    assert_eq!(store.events_after(&s, u64::MAX, 10).unwrap(), []);
    assert_eq!(
        sequences(&store.events_after(&s, 4, usize::MAX).unwrap()),
        [5]
    );
}

fn a_trim_through_the_largest_number_stops_at_the_last_sequence() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, u64::MAX).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 5);
    store
        .commit(&s, &commit(Vec::new(), vec![event("next")]))
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_prefix_matches_bytes_and_has_no_wildcard() {
    let (_db, store) = new_store();
    let s = scope("s");
    // In byte order. U+10FFFF is the largest character, so a prefix that
    // ends with it has no "next character" to stop the range at.
    let keys = [
        "a%b",
        "a*b",
        "aXb",
        "a[b]",
        "a_b",
        "a\u{10FFFF}",
        "a\u{10FFFF}z",
        "b",
    ];
    let ops = keys
        .iter()
        .map(|key| put(key, "x", Expected::Absent))
        .collect();
    store.commit(&s, &commit(ops, Vec::new())).unwrap();

    let listed = |prefix: &str| -> Vec<String> {
        store
            .list(&s, &KeyPrefix::new(prefix).unwrap())
            .unwrap()
            .entries
            .into_iter()
            .map(|entry| entry.key.as_str().to_string())
            .collect()
    };
    assert_eq!(listed(""), keys);
    assert_eq!(listed("a%"), ["a%b"]);
    assert_eq!(listed("a_"), ["a_b"]);
    assert_eq!(listed("a*"), ["a*b"]);
    assert_eq!(listed("a["), ["a[b]"]);
    assert_eq!(listed("a\u{10FFFF}"), ["a\u{10FFFF}", "a\u{10FFFF}z"]);
    assert_eq!(listed("\u{10FFFF}"), [] as [&str; 0]);
    // A backslash cannot be in a key or a prefix, and a string cannot hold
    // a 0xFF byte: the contract's own types keep both out of the store.
    assert!(DocKey::new("a\\b").is_err());
    assert!(KeyPrefix::new("a\\").is_err());
}

fn a_layout_version_that_is_not_a_number_is_refused_as_it_is() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-store-port'), ('layout_version', 'one');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: Some("speclink-store-port".into()),
            layout_version: Some("one".into()),
        }
    );
    assert_eq!(db.tables(), before);
}

fn a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence() {
    let (db, store) = new_store();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();
    let mut raw = db.client();
    raw.execute(
        "UPDATE scopes SET version = $1",
        &[&(speclink_store_port::MAX_VERSION as i64)],
    )
    .unwrap();

    let result = store.commit(
        &s,
        &commit(vec![put("k", "x", Expected::Absent)], vec![event("lost")]),
    );

    assert!(
        matches!(result, Err(PortError::Backend { .. })),
        "{result:?}"
    );
    assert_eq!(store.read(&s, &[key("k")]).unwrap(), [None]);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [1]);
    let event_seq: i64 = raw
        .query_one("SELECT event_seq FROM scopes", &[])
        .unwrap()
        .get(0);
    assert_eq!(event_seq, 1);
}

/// The tests of this module that need no database.
pub const NO_DATABASE: &[Test] = &[(
    "a_connection_string_that_cannot_be_read_is_refused_with_the_reason",
    a_connection_string_that_cannot_be_read_is_refused_with_the_reason,
)];

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "events_after_with_limit_zero_or_the_largest_number_is_empty",
        events_after_with_limit_zero_or_the_largest_number_is_empty,
    ),
    (
        "a_trim_through_the_largest_number_stops_at_the_last_sequence",
        a_trim_through_the_largest_number_stops_at_the_last_sequence,
    ),
    (
        "a_prefix_matches_bytes_and_has_no_wildcard",
        a_prefix_matches_bytes_and_has_no_wildcard,
    ),
    (
        "a_layout_version_that_is_not_a_number_is_refused_as_it_is",
        a_layout_version_that_is_not_a_number_is_refused_as_it_is,
    ),
    (
        "a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence",
        a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence,
    ),
];
