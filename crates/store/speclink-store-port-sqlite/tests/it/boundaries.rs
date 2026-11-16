//! The input boundaries from the audit (task 4.2).

use crate::support::{
    child_dir, child_report, commit, dir_content, event, foreign_file, key, new_store, open_error,
    put, report, scope, seed_five_events, sequences, spawn_child, META_TABLE,
};
use rusqlite::Connection;
use speclink_store_port::{DocKey, Expected, KeyPrefix, PortError, StorePort};
use speclink_store_port_sqlite::{OpenError, SqlitePort};

#[test]
fn events_after_with_limit_zero_or_the_largest_number_is_empty() {
    let (_dir, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(store.events_after(&s, 0, 0).unwrap(), []);
    assert_eq!(store.events_after(&s, u64::MAX, 10).unwrap(), []);
    assert_eq!(
        sequences(&store.events_after(&s, 4, usize::MAX).unwrap()),
        [5]
    );
}

#[test]
fn a_trim_through_the_largest_number_stops_at_the_last_sequence() {
    let (_dir, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, u64::MAX).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 5);
    store
        .commit(&s, &commit(Vec::new(), vec![event("next")]))
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

#[test]
fn a_prefix_matches_bytes_and_has_no_wildcard() {
    let (_dir, store) = new_store();
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

#[test]
fn a_directory_path_is_refused_with_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    let err = open_error(SqlitePort::open(dir.path()));
    assert!(
        matches!(
            &err,
            OpenError::Store(PortError::Backend { message }) if message.contains("is a directory")
        ),
        "{err:?}"
    );
}

#[test]
fn a_layout_version_that_is_not_a_number_is_refused_as_it_is() {
    let (dir, path) = foreign_file(&format!(
        "{META_TABLE}
         INSERT INTO meta VALUES ('format', 'speclink-store-port');
         INSERT INTO meta VALUES ('layout_version', 'one');"
    ));
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: Some("speclink-store-port".into()),
            layout_version: Some("one".into()),
        }
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();
    let raw = Connection::open(&path).unwrap();
    raw.execute(
        "UPDATE scopes SET version = ?1",
        [speclink_store_port::MAX_VERSION as i64],
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
        .query_row("SELECT event_seq FROM scopes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(event_seq, 1);
}

#[test]
#[ignore = "helper process of a_relative_path_still_names_the_file_after_the_directory_changes"]
fn child_opens_a_relative_path_then_changes_directory() {
    let dir = child_dir();
    std::env::set_current_dir(&dir).unwrap();
    let store = SqlitePort::open("store.db").unwrap();
    std::fs::create_dir(dir.join("elsewhere")).unwrap();
    std::env::set_current_dir(dir.join("elsewhere")).unwrap();
    report(&format!("{:?}", store.probe()));
}

/// The working directory belongs to the whole process, so the part that
/// changes it runs in a process of its own.
#[test]
fn a_relative_path_still_names_the_file_after_the_directory_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = spawn_child(
        "boundaries::child_opens_a_relative_path_then_changes_directory",
        dir.path(),
    );
    assert_eq!(child_report(&mut child), "Ok(())");
    child.wait().unwrap();
}
