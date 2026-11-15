//! Integration tests of the file system example, one section per
//! requirement of the `store-port-examples` spec.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use speclink_store_port::conformance::{self, CheckOutcome, PortHarness};
use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, PortError, RecordedEvent, ScopeId,
    StorePort, Timestamp, Version,
};
use speclink_store_port_fs::{FsPort, OpenError};
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

/// A store on a new directory.
fn new_store() -> (TempDir, FsPort) {
    let dir = tempfile::tempdir().unwrap();
    let store = FsPort::open(dir.path().join("store")).unwrap();
    (dir, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
fn seed_five_events(store: &FsPort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

fn open_error(result: Result<FsPort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the open must be refused"),
        Err(err) => err,
    }
}

/// Every file and directory below `dir`, with the bytes of each file, to
/// prove that a call changed nothing and added nothing.
fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn walk(root: &Path, dir: &Path, found: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                found.push((relative, None));
                walk(root, &path, found);
            } else {
                found.push((relative, Some(std::fs::read(&path).unwrap())));
            }
        }
    }
    let mut found = Vec::new();
    walk(dir, dir, &mut found);
    found.sort();
    found
}

/// A directory with a `meta.json` that this example did not write.
fn foreign_dir(meta: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("meta.json"), meta).unwrap();
    (dir, root)
}

// --- 三個範例通過合格測試 ---------------------------------------------------

struct Harness {
    store: FsPort,
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

fn assert_new_empty_store(root: &Path, store: &FsPort) {
    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["format"], "speclink-store-port");
    assert_eq!(meta["layout_version"], 1);
}

#[test]
fn a_missing_directory_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = FsPort::open(&root).unwrap();
    assert_new_empty_store(&root, &store);
}

#[test]
fn an_empty_directory_is_initialized() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsPort::open(dir.path()).unwrap();
    assert_new_empty_store(dir.path(), &store);
}

#[test]
fn another_marker_is_refused_and_the_directory_is_unchanged() {
    let (_dir, root) =
        foreign_dir(r#"{ "format": "speclink-team-store-fs", "layout_version": 1 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), .. }
                if format == "speclink-team-store-fs"
        ),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_newer_layout_version_is_refused() {
    let (_dir, root) = foreign_dir(r#"{ "format": "speclink-store-port", "layout_version": 2 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_directory_with_other_files_and_no_meta_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not a store").unwrap();
    let before = tree(dir.path());

    let err = open_error(FsPort::open(dir.path()));

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
    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_meta_file_without_a_marker_is_refused() {
    let (_dir, root) = foreign_dir(r#"{ "layout_version": 1 }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_half_finished_initialization_opens_as_a_new_empty_store() {
    // What an interrupted first open leaves: the example's own empty
    // directory and its lock file, and no `meta.json`.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("scopes")).unwrap();
    std::fs::write(dir.path().join("lock"), "").unwrap();

    let store = FsPort::open(dir.path()).unwrap();

    assert_new_empty_store(dir.path(), &store);
    store
        .commit(
            &scope("s"),
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    assert_eq!(store.list(&scope("s"), &all()).unwrap().version, v(1));
}

// --- 重新開啟與中斷後的一致性 -----------------------------------------------

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

// --- 兩個儲存物件開同一份資料 -----------------------------------------------

/// The helper processes are tests of this same binary, run by name. Each one
/// prints one line that starts with this marker, so the parent does not read
/// the test runner's own output by mistake.
const MARK: &str = "speclink-fs-child:";
const DIR_ENV: &str = "SPECLINK_FS_CHILD_DIR";

fn spawn_child(test: &str, dir: &Path) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(DIR_ENV, dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn the helper process")
}

/// The text after the marker on the first marked line of the child's output.
/// The pipe stays with the child, so the test runner in the child can print
/// its last lines after the report.
fn child_report(child: &mut Child) -> String {
    let out = std::io::BufReader::new(child.stdout.as_mut().unwrap());
    for line in out.lines() {
        let line = line.expect("read the helper's output");
        if let Some((_, report)) = line.split_once(MARK) {
            return report.trim().to_string();
        }
    }
    panic!("the helper process ended without a report");
}

fn child_dir() -> PathBuf {
    PathBuf::from(std::env::var(DIR_ENV).expect("the parent names the directory"))
}

fn report(text: &str) {
    use std::io::Write;
    println!("{MARK}{text}");
    std::io::stdout().flush().unwrap();
}

#[test]
#[ignore = "helper process of a_second_process_finds_the_directory_in_use"]
fn child_tries_to_open() {
    match FsPort::open(child_dir()) {
        Ok(_) => report("opened"),
        Err(OpenError::InUse) => report("in-use"),
        Err(other) => report(&format!("other: {other:?}")),
    }
}

#[test]
#[ignore = "helper process of a_killed_holder_does_not_keep_the_directory"]
fn child_holds_the_directory() {
    let store = FsPort::open(child_dir()).expect("the holder opens the directory");
    store
        .commit(
            &scope("s"),
            &commit(
                vec![put("k", "written by the holder", Expected::Absent)],
                Vec::new(),
            ),
        )
        .unwrap();
    report("ready");
    // Wait for the parent to kill this process. The limit only bounds a
    // process that the parent did not stop.
    std::thread::sleep(std::time::Duration::from_secs(120));
}

#[test]
fn a_second_process_finds_the_directory_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let s = scope("s");
    let store = FsPort::open(dir.path()).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("k", "v1", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let before = tree(dir.path());

    let mut child = spawn_child("child_tries_to_open", dir.path());
    let outcome = child_report(&mut child);
    child.wait().unwrap();

    assert_eq!(outcome, "in-use");
    assert_eq!(tree(dir.path()), before);
    // The first object continues to work.
    assert_eq!(
        store.commit(
            &s,
            &commit(vec![put("k", "v2", Expected::At(v(1)))], Vec::new())
        ),
        Ok(v(2))
    );
}

#[test]
fn a_second_object_in_the_same_process_finds_the_directory_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let _store = FsPort::open(dir.path()).unwrap();
    assert_eq!(open_error(FsPort::open(dir.path())), OpenError::InUse);
}

#[test]
fn a_killed_holder_does_not_keep_the_directory() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = spawn_child("child_holds_the_directory", dir.path());
    assert_eq!(child_report(&mut child), "ready");
    assert_eq!(open_error(FsPort::open(dir.path())), OpenError::InUse);

    // No unlock and no cleanup: the operating system releases the lock.
    child.kill().expect("kill the holder");
    child.wait().expect("reap the holder");

    let store = FsPort::open(dir.path()).expect("open after the holder ended");
    assert_eq!(
        store.read(&scope("s"), &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "written by the holder"
    );
}

#[test]
fn scope_names_stay_inside_the_directory_and_do_not_mix() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = FsPort::open(&root).unwrap();
    let names = [
        "abc",
        "ABC",
        "a.b",
        "..",
        "../../escape",
        "Org/Repo",
        "/tmp/x",
    ];

    for name in names {
        store
            .commit(
                &scope(name),
                &commit(vec![put("k", name, Expected::Absent)], Vec::new()),
            )
            .unwrap();
    }

    for name in names {
        let listing = store.list(&scope(name), &all()).unwrap();
        assert_eq!(listing.version, v(1), "scope {name}");
        assert_eq!(
            store.read(&scope(name), &[key("k")]).unwrap()[0]
                .as_ref()
                .unwrap()
                .content,
            name
        );
    }
    // Nothing was written next to the store's directory.
    let beside: Vec<_> = std::fs::read_dir(parent.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(beside, ["store"]);
    // One directory per scope, directly below `scopes`.
    assert_eq!(
        std::fs::read_dir(root.join("scopes")).unwrap().count(),
        names.len()
    );
}

// --- 可用性檢查與失敗的分類 -------------------------------------------------

#[test]
fn probe_succeeds_on_an_open_store() {
    let (_dir, store) = new_store();
    assert_eq!(store.probe(), Ok(()));
}

/// Unix only: Windows does not let a directory move while a file in it is
/// open, so the situation cannot occur there.
#[cfg(unix)]
#[test]
fn probe_fails_after_the_directory_is_moved_away() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = FsPort::open(&root).unwrap();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    std::fs::rename(&root, dir.path().join("moved")).unwrap();

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
    // In byte order. U+10FFFF is the largest character.
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
fn a_file_path_is_refused_with_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not-a-directory");
    std::fs::write(&path, "a file").unwrap();

    let err = open_error(FsPort::open(&path));

    assert!(
        matches!(
            &err,
            OpenError::Store(PortError::Backend { message }) if message.contains("is a file")
        ),
        "{err:?}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"a file");
}

#[test]
fn a_layout_version_that_is_not_a_number_is_refused_as_it_is() {
    let (_dir, root) =
        foreign_dir(r#"{ "format": "speclink-store-port", "layout_version": "one" }"#);
    let before = tree(&root);

    let err = open_error(FsPort::open(&root));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: Some("speclink-store-port".into()),
            layout_version: Some("one".into()),
        }
    );
    assert_eq!(tree(&root), before);
}

#[test]
fn a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence() {
    let (dir, store) = new_store();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();
    // The store reads the index for each call, so the test can put the
    // scope at the limit while the store is open.
    let index_path = dir
        .path()
        .join("store")
        .join("scopes")
        .join("s")
        .join("index.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&index_path).unwrap()).unwrap();
    index["version"] = serde_json::json!(speclink_store_port::MAX_VERSION);
    std::fs::write(&index_path, index.to_string()).unwrap();
    let before = tree(dir.path());

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
    assert_eq!(tree(dir.path()), before);
}
