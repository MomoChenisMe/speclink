//! Fixtures that the test modules share.

use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, RecordedEvent, ScopeId, StorePort,
    Timestamp, Version,
};
use speclink_store_port_fs::{FsPort, OpenError};
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

/// A store on a new directory.
pub fn new_store() -> (TempDir, FsPort) {
    let dir = tempfile::tempdir().unwrap();
    let store = FsPort::open(dir.path().join("store")).unwrap();
    (dir, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
pub fn seed_five_events(store: &FsPort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

pub fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

pub fn open_error(result: Result<FsPort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the open must be refused"),
        Err(err) => err,
    }
}

/// Every file and directory below `dir`, with the bytes of each file, to
/// prove that a call changed nothing and added nothing. The `lock` file is
/// listed without its bytes: while a store holds its lock, Windows does not
/// let another handle read it, and its bytes mean nothing.
pub fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn walk(root: &Path, dir: &Path, found: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                found.push((relative, None));
                walk(root, &path, found);
            } else if relative == Path::new("lock") {
                found.push((relative, Some(Vec::new())));
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
pub fn foreign_dir(meta: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("meta.json"), meta).unwrap();
    (dir, root)
}

/// A helper process is a test of this same binary, run by name. It prints
/// one line that starts with this marker, so the parent does not read the
/// test runner's own output by mistake.
pub const MARK: &str = "speclink-fs-child:";
pub const DIR_ENV: &str = "SPECLINK_FS_CHILD_DIR";

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
