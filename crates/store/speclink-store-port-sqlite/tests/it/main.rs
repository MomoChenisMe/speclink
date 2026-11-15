//! Integration tests of the SQLite example, one section per requirement of
//! the `store-port-examples` spec.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Barrier;
use std::thread;

use rusqlite::Connection;
use speclink_store_port::conformance::{self, CheckOutcome, PortHarness};
use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, PortError, RecordedEvent, ScopeId,
    StorePort, Timestamp, Version,
};
use speclink_store_port_sqlite::{OpenError, SqlitePort};
use tempfile::TempDir;

// --- fixtures ---------------------------------------------------------------

fn scope(name: &str) -> ScopeId {
    ScopeId::new(name).unwrap()
}

fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

fn all() -> KeyPrefix {
    KeyPrefix::new("").unwrap()
}

fn v(n: u64) -> Version {
    Version::new(n).unwrap()
}

fn put(text: &str, content: &str, expected: Expected) -> Op {
    Op::Put {
        key: key(text),
        content: content.into(),
        expected,
    }
}

fn event(name: &str) -> Event {
    Event::new(name, serde_json::json!({ "name": name, "n": 1.0 })).unwrap()
}

fn commit_at(millis: u64, ops: Vec<Op>, events: Vec<Event>) -> Commit {
    Commit::new(
        Actor::new("u-1", "Momo").unwrap(),
        "test",
        Timestamp::new(millis).unwrap(),
        ops,
        events,
    )
    .unwrap()
}

fn commit(ops: Vec<Op>, events: Vec<Event>) -> Commit {
    commit_at(1, ops, events)
}

/// A store on a new file in a new directory.
fn new_store() -> (TempDir, SqlitePort) {
    let dir = tempfile::tempdir().unwrap();
    let store = SqlitePort::open(dir.path().join("store.db")).unwrap();
    (dir, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
fn seed_five_events(store: &SqlitePort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

fn open_error(result: Result<SqlitePort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the open must be refused"),
        Err(err) => err,
    }
}

/// Every file in `dir` with its bytes, to prove that a refused open changed
/// nothing and added nothing.
fn dir_content(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().to_string_lossy().into_owned(),
                std::fs::read(entry.path()).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

/// A SQLite file that this example did not make.
fn foreign_file(sql: &str) -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(sql).unwrap();
    drop(conn);
    (dir, path)
}

const META_TABLE: &str = "CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);";

// --- 三個範例通過合格測試 ---------------------------------------------------

struct Harness {
    store: SqlitePort,
    next_scope: AtomicU64,
}

impl PortHarness for Harness {
    fn store(&self) -> &dyn StorePort {
        &self.store
    }

    fn new_scope(&self) -> Result<ScopeId, PortError> {
        let n = self.next_scope.fetch_add(1, Ordering::Relaxed);
        Ok(scope(&format!("conformance-{n}")))
    }

    fn recorded_events(&self, scope: &ScopeId) -> Result<Option<Vec<RecordedEvent>>, PortError> {
        let events = self.store.events_after(scope, 0, usize::MAX)?;
        Ok(Some(events.into_iter().map(|(_, event)| event).collect()))
    }
}

#[test]
fn conformance_passes_all_sixteen_checks_without_a_skip() {
    let (_dir, store) = new_store();
    let report = conformance::run(&Harness {
        store,
        next_scope: AtomicU64::new(0),
    });
    let not_passed: Vec<_> = report
        .checks
        .iter()
        .filter(|check| check.outcome != CheckOutcome::Passed)
        .collect();
    assert!(not_passed.is_empty(), "{not_passed:#?}");
    assert_eq!(report.checks.len(), 16);
}

#[test]
fn modified_at_is_the_commit_time() {
    let (_dir, store) = new_store();
    let s = scope("s");
    let doc = "changes/a/proposal.md";
    store
        .commit(
            &s,
            &commit_at(
                1_700_000_000_000,
                vec![put(doc, "text", Expected::Absent)],
                Vec::new(),
            ),
        )
        .unwrap();
    let found = store.read(&s, &[key(doc)]).unwrap();
    assert_eq!(
        found[0].as_ref().unwrap().modified_at.get(),
        1_700_000_000_000
    );
    let listing = store.list(&s, &all()).unwrap();
    assert_eq!(listing.entries[0].modified_at.get(), 1_700_000_000_000);
}

#[test]
fn each_scope_has_its_own_version() {
    let (_dir, store) = new_store();
    let (a, b) = (scope("A"), scope("B"));
    store
        .commit(
            &a,
            &commit(vec![put("a1", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    store
        .commit(
            &a,
            &commit(vec![put("a2", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    store
        .commit(
            &b,
            &commit(vec![put("b1", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    let in_a = store.list(&a, &all()).unwrap();
    let in_b = store.list(&b, &all()).unwrap();
    assert_eq!(in_a.version, v(2));
    assert_eq!(in_b.version, v(1));
    let keys_in_b: Vec<&str> = in_b.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys_in_b, ["b1"]);
    assert_eq!(store.read(&b, &[key("a1")]).unwrap(), [None]);
}

// --- 事件與寫入同一次交易並可依序號讀回 -------------------------------------

#[test]
fn two_events_of_one_commit_get_sequence_1_and_2() {
    let (_dir, store) = new_store();
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

#[test]
fn a_rejected_commit_keeps_no_event_and_uses_no_sequence() {
    let (_dir, store) = new_store();
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

#[test]
fn events_after_reads_from_the_middle_up_to_the_limit() {
    let (_dir, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(sequences(&store.events_after(&s, 2, 2).unwrap()), [3, 4]);
}

#[test]
fn a_scope_with_no_events_reads_as_empty() {
    let (_dir, store) = new_store();
    assert_eq!(store.events_after(&scope("never"), 0, 10).unwrap(), []);
    assert_eq!(store.event_floor(&scope("never")).unwrap(), 0);
}

// --- 清除舊事件與下限 -------------------------------------------------------

#[test]
fn trim_keeps_only_the_newer_events_and_changes_no_document() {
    let (_dir, store) = new_store();
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

#[test]
fn the_sequence_continues_after_a_trim() {
    let (_dir, store) = new_store();
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

#[test]
fn a_trim_past_the_last_sequence_stops_at_it() {
    let (_dir, store) = new_store();
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

#[test]
fn a_trim_at_or_below_the_floor_does_nothing() {
    let (_dir, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 3).unwrap();
    store.trim_events(&s, 2).unwrap();
    store.trim_events(&s, 3).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 3);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [4, 5]);
}

// --- 開啟時只認自己的配置 ---------------------------------------------------

#[test]
fn a_missing_file_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let store = SqlitePort::open(&path).unwrap();

    assert!(path.is_file());
    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());

    let conn = Connection::open(&path).unwrap();
    let format: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'format'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(format, "speclink-store-port");
    let layout: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'layout_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(layout, "1");
}

#[test]
fn another_marker_is_refused_and_the_file_is_unchanged() {
    let (dir, path) = foreign_file(&format!(
        "{META_TABLE}
         INSERT INTO meta VALUES ('format', 'speclink-team-store');
         INSERT INTO meta VALUES ('schema_version', '1');"
    ));
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: None }
                if format == "speclink-team-store"
        ),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn a_newer_layout_version_is_refused() {
    let (dir, path) = foreign_file(&format!(
        "{META_TABLE}
         INSERT INTO meta VALUES ('format', 'speclink-store-port');
         INSERT INTO meta VALUES ('layout_version', '2');"
    ));
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn tables_without_meta_are_refused() {
    let (dir, path) = foreign_file("CREATE TABLE notes (body TEXT);");
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout {
                format: None,
                layout_version: None
            }
        ),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

#[test]
fn a_meta_table_without_a_marker_is_refused() {
    let (dir, path) = foreign_file(META_TABLE);
    let before = dir_content(dir.path());

    let err = open_error(SqlitePort::open(&path));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(dir_content(dir.path()), before);
}

// --- 重新開啟與中斷後的一致性 -----------------------------------------------

#[test]
fn a_reopened_store_has_the_same_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let s = scope("s");
    let keys = [key("a"), key("b"), key("c"), key("d")];

    let store = SqlitePort::open(&path).unwrap();
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

    let store = SqlitePort::open(&path).unwrap();
    assert_eq!(store.list(&s, &all()).unwrap(), listing);
    assert_eq!(store.read(&s, &keys).unwrap(), documents);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), events);
    assert_eq!(store.event_floor(&s).unwrap(), 0);

    // The floor is also the same after a reopen.
    store.trim_events(&s, 2).unwrap();
    drop(store);
    let store = SqlitePort::open(&path).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 2);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [3]);
}

#[test]
fn a_commit_that_fails_on_its_second_operation_leaves_nothing() {
    let (_dir, store) = new_store();
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

// --- 兩個儲存物件開同一份資料 -----------------------------------------------

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

// --- 可用性檢查與失敗的分類 -------------------------------------------------

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

// --- 稽核：輸入的邊界 -------------------------------------------------------

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
