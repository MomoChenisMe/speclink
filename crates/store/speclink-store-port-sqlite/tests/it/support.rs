//! Fixtures that the test modules share.

use rusqlite::Connection;
use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, RecordedEvent, ScopeId, StorePort,
    Timestamp, Version,
};
use speclink_store_port_sqlite::{OpenError, SqlitePort};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tempfile::TempDir;

pub fn scope(name: &str) -> ScopeId {
    ScopeId::new(name).unwrap()
}

pub fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

pub fn all() -> KeyPrefix {
    KeyPrefix::new("").unwrap()
}

pub fn v(n: u64) -> Version {
    Version::new(n).unwrap()
}

pub fn put(text: &str, content: &str, expected: Expected) -> Op {
    Op::Put {
        key: key(text),
        content: content.into(),
        expected,
    }
}

pub fn event(name: &str) -> Event {
    Event::new(name, serde_json::json!({ "name": name, "n": 1.0 })).unwrap()
}

pub fn commit_at(millis: u64, ops: Vec<Op>, events: Vec<Event>) -> Commit {
    Commit::new(
        Actor::new("u-1", "Momo").unwrap(),
        "test",
        Timestamp::new(millis).unwrap(),
        ops,
        events,
    )
    .unwrap()
}

pub fn commit(ops: Vec<Op>, events: Vec<Event>) -> Commit {
    commit_at(1, ops, events)
}

/// A store on a new file in a new directory.
pub fn new_store() -> (TempDir, SqlitePort) {
    let dir = tempfile::tempdir().unwrap();
    let store = SqlitePort::open(dir.path().join("store.db")).unwrap();
    (dir, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
pub fn seed_five_events(store: &SqlitePort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

pub fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

pub fn open_error(result: Result<SqlitePort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the open must be refused"),
        Err(err) => err,
    }
}

/// Every file in `dir` with its bytes, to prove that a refused open changed
/// nothing and added nothing.
pub fn dir_content(dir: &Path) -> Vec<(String, Vec<u8>)> {
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
pub fn foreign_file(sql: &str) -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(sql).unwrap();
    drop(conn);
    (dir, path)
}

pub const META_TABLE: &str =
    "CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);";

/// A helper process is a test of this same binary, run by name. It prints
/// one line that starts with this marker, so the parent does not read the
/// test runner's own output by mistake.
pub const MARK: &str = "speclink-sqlite-child:";
pub const DIR_ENV: &str = "SPECLINK_SQLITE_CHILD_DIR";

/// Start the `#[ignore]` test `test` (its full path) as a new process.
pub fn spawn_child(test: &str, dir: &Path) -> Child {
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
pub fn child_report(child: &mut Child) -> String {
    let out = std::io::BufReader::new(child.stdout.as_mut().unwrap());
    for line in out.lines() {
        let line = line.expect("read the helper's output");
        if let Some((_, report)) = line.split_once(MARK) {
            return report.trim().to_string();
        }
    }
    panic!("the helper process ended without a report");
}

/// In a helper process: the directory that the parent named.
pub fn child_dir() -> PathBuf {
    PathBuf::from(std::env::var(DIR_ENV).expect("the parent names the directory"))
}

/// In a helper process: print the one marked line.
pub fn report(text: &str) {
    use std::io::Write;
    println!("{MARK}{text}");
    std::io::stdout().flush().unwrap();
}
