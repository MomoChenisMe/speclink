//! An example of the store port on one directory. It is an example to read
//! and to copy, not a production store: it shows how a commit becomes
//! all-or-nothing on a file system with one rename, and how the events of a
//! commit are published by that same rename. It has no connection pool, no
//! TLS, no migration between layout versions, no history of old document
//! versions, and it does not deliver events between several host instances.
//! One store object holds the directory at a time.
//!
//! # The layout
//!
//! - `meta.json`: the marker `format = speclink-store-port` and
//!   `layout_version = 1`.
//! - `lock`: the store object holds an exclusive lock on this file for as
//!   long as it lives. The operating system releases the lock when the
//!   process ends, so a holder that was killed does not block the next open.
//! - `scopes/<escaped scope>/index.json`: the one truth of a scope. It has
//!   the scope version, the last event sequence, the event floor, and for
//!   each key the name of its content file, its version and its
//!   last-modified time.
//! - `scopes/<escaped scope>/blobs/<number>`: one content file. It never
//!   changes after it is written.
//! - `scopes/<escaped scope>/events/<sequence>.json`: one event.
//!
//! A scope name becomes a directory name: lowercase ASCII letters, digits,
//! `-` and `_` stay, every other byte becomes `%XX`. An uppercase letter is
//! escaped because some file systems ignore case; `.` and `/` are escaped so
//! a name cannot leave the directory. A document key is never a file name.
//!
//! Two limits come from this rule. An escaped name can be three times as
//! long as the scope name, so a long scope name can be more than the file
//! system permits for one name; the commit then fails with
//! [`PortError::Backend`]. And on Windows a scope with a device name, such
//! as `con` or `nul`, keeps that name and possibly cannot get its directory.
//!
//! # A commit
//!
//! 1. Read `index.json` and check the expectation of every operation. One
//!    that does not hold returns the conflict; no file was written.
//! 2. Write the new content files and event files, and sync them.
//! 3. Write `index.json.new`, sync it, and rename it to `index.json`. This
//!    rename is the one point that publishes the commit.
//! 4. Delete the content files that the index no longer names.
//!
//! A reader believes only `index.json`. A content file that the index does
//! not name, and an event file with a sequence above the index's, do not
//! exist for a reader. Each open and each commit removes such files.
//!
//! # Outside the contract
//!
//! [`FsPort::events_after`], [`FsPort::event_floor`], [`FsPort::trim_events`]
//! and [`FsPort::probe`] are for the host that owns the store. The store
//! never trims events by itself.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde_json::{json, Value};
use speclink_store_port::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError,
    RecordedEvent, ScopeId, StorePort, Timestamp, Version,
};

/// The marker this example writes to `meta.json` and the only one it opens.
const FORMAT: &str = "speclink-store-port";
const LAYOUT_VERSION: u64 = 1;

const META_FILE: &str = "meta.json";
const META_NEW_FILE: &str = "meta.json.new";
const LOCK_FILE: &str = "lock";
const SCOPES_DIR: &str = "scopes";
const INDEX_FILE: &str = "index.json";
const INDEX_NEW_FILE: &str = "index.json.new";
const BLOBS_DIR: &str = "blobs";
const EVENTS_DIR: &str = "events";

/// Why [`FsPort::open`] did not return a store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The directory holds data that this example did not write. The two
    /// fields are the values read from `meta.json`, or `None` when there is
    /// none.
    UnknownLayout {
        format: Option<String>,
        layout_version: Option<String>,
    },
    /// Another store object, in this process or another, holds the
    /// directory.
    InUse,
    /// The directory could not be opened or read.
    Store(PortError),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenError::UnknownLayout {
                format,
                layout_version,
            } => write!(
                f,
                "not a {FORMAT} store of layout version {LAYOUT_VERSION}: found format {} and layout version {}",
                format.as_deref().unwrap_or("(none)"),
                layout_version.as_deref().unwrap_or("(none)")
            ),
            OpenError::InUse => write!(f, "another store object holds the directory"),
            OpenError::Store(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<PortError> for OpenError {
    fn from(err: PortError) -> Self {
        OpenError::Store(err)
    }
}

/// A timeout or an interrupted call is temporary; every other failure is
/// not.
fn io_err(action: &str, path: &Path, err: io::Error) -> PortError {
    let message = format!("{action} {}: {err}", path.display());
    match err.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock => {
            PortError::Unavailable { message }
        }
        _ => PortError::Backend { message },
    }
}

fn corrupt(path: &Path) -> PortError {
    PortError::Backend {
        message: format!("{} does not have the expected content", path.display()),
    }
}

/// A stored number that the contract's types refuse.
fn out_of_range<E>(_: E) -> PortError {
    PortError::Backend {
        message: "a stored version or time is out of range".into(),
    }
}

// --- durable writes ---------------------------------------------------------

/// Write a file and sync it, so its bytes are on the disk before anything
/// names it.
fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), PortError> {
    let mut file = File::create(path).map_err(|err| io_err("create", path, err))?;
    file.write_all(bytes)
        .map_err(|err| io_err("write", path, err))?;
    file.sync_all().map_err(|err| io_err("sync", path, err))
}

/// Sync a directory, so a new name in it stays after a power loss. Windows
/// does not let a directory open as a file, and does not need it.
fn sync_dir(dir: &Path) -> Result<(), PortError> {
    #[cfg(not(windows))]
    File::open(dir)
        .and_then(|handle| handle.sync_all())
        .map_err(|err| io_err("sync", dir, err))?;
    #[cfg(windows)]
    let _ = dir;
    Ok(())
}

/// Put `bytes` at `path` in one step: a reader sees the old file or the new
/// file, never a part of one.
fn publish(path: &Path, staging: &Path, bytes: &[u8]) -> Result<(), PortError> {
    write_synced(staging, bytes)?;
    fs::rename(staging, path).map_err(|err| io_err("rename to", path, err))?;
    sync_dir(path.parent().unwrap_or(Path::new(".")))
}

// --- the index of one scope -------------------------------------------------

struct Entry {
    blob: u64,
    version: u64,
    modified_at: u64,
}

/// The content of `index.json`. A scope with no index file has the default:
/// version 0, no documents, no events.
#[derive(Default)]
struct Index {
    version: u64,
    event_seq: u64,
    event_floor: u64,
    /// The name of the next content file.
    next_blob: u64,
    /// A `BTreeMap` of strings keeps the keys in byte order.
    documents: BTreeMap<String, Entry>,
}

impl Index {
    fn to_json(&self) -> Value {
        let documents: serde_json::Map<String, Value> = self
            .documents
            .iter()
            .map(|(key, entry)| {
                let entry = json!({
                    "blob": entry.blob,
                    "version": entry.version,
                    "modified_at": entry.modified_at,
                });
                (key.clone(), entry)
            })
            .collect();
        json!({
            "version": self.version,
            "event_seq": self.event_seq,
            "event_floor": self.event_floor,
            "next_blob": self.next_blob,
            "documents": documents,
        })
    }

    fn from_json(value: &Value) -> Option<Self> {
        let number = |value: &Value, field: &str| value.get(field)?.as_u64();
        let mut documents = BTreeMap::new();
        for (key, entry) in value.get("documents")?.as_object()? {
            documents.insert(
                key.clone(),
                Entry {
                    blob: number(entry, "blob")?,
                    version: number(entry, "version")?,
                    modified_at: number(entry, "modified_at")?,
                },
            );
        }
        Some(Self {
            version: number(value, "version")?,
            event_seq: number(value, "event_seq")?,
            event_floor: number(value, "event_floor")?,
            next_blob: number(value, "next_blob")?,
            documents,
        })
    }
}

/// The directory of one scope.
struct ScopeDir(PathBuf);

impl ScopeDir {
    fn index(&self) -> PathBuf {
        self.0.join(INDEX_FILE)
    }

    fn blobs(&self) -> PathBuf {
        self.0.join(BLOBS_DIR)
    }

    fn blob(&self, number: u64) -> PathBuf {
        self.blobs().join(number.to_string())
    }

    fn events(&self) -> PathBuf {
        self.0.join(EVENTS_DIR)
    }

    fn event(&self, seq: u64) -> PathBuf {
        self.events().join(format!("{seq}.json"))
    }

    fn load_index(&self) -> Result<Index, PortError> {
        let path = self.index();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Index::default()),
            Err(err) => return Err(io_err("read", &path, err)),
        };
        serde_json::from_slice(&bytes)
            .ok()
            .and_then(|value| Index::from_json(&value))
            .ok_or_else(|| corrupt(&path))
    }

    fn publish_index(&self, index: &Index) -> Result<(), PortError> {
        publish(
            &self.index(),
            &self.0.join(INDEX_NEW_FILE),
            index.to_json().to_string().as_bytes(),
        )
    }

    /// Remove what a commit that did not reach its rename left behind, and
    /// return the index. The removal is best effort: a file that stays is
    /// still not visible, and a later commit writes over it.
    fn clean(&self) -> Result<Index, PortError> {
        let _ = fs::remove_file(self.0.join(INDEX_NEW_FILE));
        let index = self.load_index()?;
        let named: HashSet<u64> = index.documents.values().map(|entry| entry.blob).collect();
        remove_files(&self.blobs(), |name| {
            name.parse().is_ok_and(|blob: u64| !named.contains(&blob))
        });
        remove_files(&self.events(), |name| {
            name.strip_suffix(".json")
                .and_then(|seq| seq.parse().ok())
                .is_some_and(|seq: u64| seq > index.event_seq || seq <= index.event_floor)
        });
        Ok(index)
    }
}

/// Remove the files of `dir` whose name `is_stray` accepts. A directory that
/// does not exist has none.
fn remove_files(dir: &Path, is_stray: impl Fn(&str) -> bool) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_str().is_some_and(&is_stray) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// The directory name of a scope.
fn escape(scope: &ScopeId) -> String {
    let mut name = String::new();
    for byte in scope.as_str().bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => name.push(byte as char),
            other => name.push_str(&format!("%{other:02X}")),
        }
    }
    name
}

// --- open -------------------------------------------------------------------

/// Decide what the directory is. This only reads, so a refused open leaves
/// the directory as it was. `Ok(true)` is an empty place that needs the
/// layout.
fn needs_layout(root: &Path) -> Result<bool, OpenError> {
    let meta_path = root.join(META_FILE);
    match fs::read(&meta_path) {
        Ok(bytes) => {
            let meta: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            // A string comes out as it is; any other value as its JSON text.
            let text = |field: &str| match meta.get(field)? {
                Value::String(text) => Some(text.clone()),
                other => Some(other.to_string()),
            };
            let ours = meta.get("format") == Some(&json!(FORMAT))
                && meta.get("layout_version") == Some(&json!(LAYOUT_VERSION));
            if ours {
                Ok(false)
            } else {
                Err(OpenError::UnknownLayout {
                    format: text("format"),
                    layout_version: text("layout_version"),
                })
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let entries = match fs::read_dir(root) {
                Ok(entries) => entries,
                Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(true),
                Err(err) => return Err(io_err("read", root, err).into()),
            };
            // A first open that stopped before it wrote `meta.json` leaves
            // only these names. Anything else belongs to someone else.
            for entry in entries {
                let entry = entry.map_err(|err| io_err("read", root, err))?;
                let path = entry.path();
                let ours = match entry.file_name().to_str() {
                    Some(LOCK_FILE | META_NEW_FILE) => path.is_file(),
                    Some(SCOPES_DIR) => {
                        fs::read_dir(&path).is_ok_and(|mut inside| inside.next().is_none())
                    }
                    _ => false,
                };
                if !ours {
                    return Err(OpenError::UnknownLayout {
                        format: None,
                        layout_version: None,
                    });
                }
            }
            Ok(true)
        }
        Err(err) => Err(io_err("read", &meta_path, err).into()),
    }
}

/// A store on one directory. Every operation takes the object's one lock, so
/// a reader never sees a commit between its rename and its cleanup.
pub struct FsPort {
    root: PathBuf,
    /// The open lock file. The file lock keeps other store objects out; the
    /// mutex makes the calls on this object take turns.
    lock: Mutex<File>,
}

impl FsPort {
    /// Open the store in `dir`.
    ///
    /// - No directory, or an empty one: the example makes its layout.
    /// - A directory with this example's marker and layout version: opened
    ///   as it is.
    /// - A directory that another store object holds: [`OpenError::InUse`].
    /// - Anything else: [`OpenError::UnknownLayout`], and the directory does
    ///   not change.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, OpenError> {
        let root = dir.as_ref();
        // The operating systems each say this in a different way.
        if root.is_file() {
            return Err(OpenError::Store(PortError::Backend {
                message: format!(
                    "{} is a file; the store needs the path of a directory",
                    root.display()
                ),
            }));
        }
        let needs_layout = needs_layout(root)?;
        // Past the check: from here the directory is ours to write.
        fs::create_dir_all(root).map_err(|err| io_err("create", root, err))?;
        let lock = acquire_lock(root)?;
        let scopes = root.join(SCOPES_DIR);
        if needs_layout {
            // `meta.json` comes last and in one step, so a stop before it
            // leaves a directory that the next open makes again.
            fs::create_dir_all(&scopes).map_err(|err| io_err("create", &scopes, err))?;
            let meta = json!({ "format": FORMAT, "layout_version": LAYOUT_VERSION });
            publish(
                &root.join(META_FILE),
                &root.join(META_NEW_FILE),
                meta.to_string().as_bytes(),
            )?;
        }
        let entries = fs::read_dir(&scopes).map_err(|err| io_err("read", &scopes, err))?;
        for entry in entries {
            let entry = entry.map_err(|err| io_err("read", &scopes, err))?;
            ScopeDir(entry.path()).clean()?;
        }
        Ok(Self {
            root: root.to_path_buf(),
            lock: Mutex::new(lock),
        })
    }

    /// No call writes a part of `index.json`, so the directory behind a
    /// poisoned lock is still whole.
    fn guard(&self) -> MutexGuard<'_, File> {
        self.lock.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn scope_dir(&self, scope: &ScopeId) -> ScopeDir {
        ScopeDir(self.root.join(SCOPES_DIR).join(escape(scope)))
    }

    /// The events of `scope` with a sequence larger than `after`, smallest
    /// first, at most `limit` of them. Each one comes with its sequence.
    ///
    /// Trimmed events are gone: compare `after` with [`FsPort::event_floor`]
    /// to learn whether some were skipped.
    pub fn events_after(
        &self,
        scope: &ScopeId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        let _guard = self.guard();
        let dir = self.scope_dir(scope);
        let index = dir.load_index()?;
        let mut events = Vec::new();
        let mut seq = after.max(index.event_floor);
        while seq < index.event_seq && events.len() < limit {
            seq += 1;
            let path = dir.event(seq);
            let bytes = fs::read(&path).map_err(|err| io_err("read", &path, err))?;
            let event = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|event| {
                    Some(RecordedEvent {
                        name: event.get("name")?.as_str()?.to_string(),
                        payload: event.get("payload")?.clone(),
                        version: Version::new(event.get("version")?.as_u64()?).ok()?,
                    })
                })
                .ok_or_else(|| corrupt(&path))?;
            events.push((seq, event));
        }
        Ok(events)
    }

    /// The largest sequence that [`FsPort::trim_events`] removed from
    /// `scope`; 0 when nothing was removed.
    pub fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        let _guard = self.guard();
        Ok(self.scope_dir(scope).load_index()?.event_floor)
    }

    /// Remove the events of `scope` up to and including sequence `through`.
    /// It stops at the last sequence in use, so later events continue the
    /// numbering. It changes no document and no version.
    pub fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        let _guard = self.guard();
        let dir = self.scope_dir(scope);
        let mut index = dir.load_index()?;
        let old_floor = index.event_floor;
        let new_floor = through.min(index.event_seq);
        if new_floor <= old_floor {
            return Ok(());
        }
        // The floor first, the files second: a stop between the two leaves
        // files below the floor, which no reader sees.
        index.event_floor = new_floor;
        dir.publish_index(&index)?;
        for seq in old_floor + 1..=new_floor {
            let _ = fs::remove_file(dir.event(seq));
        }
        Ok(())
    }

    /// Check that the store can answer: `meta.json` is in the directory and
    /// can be read.
    pub fn probe(&self) -> Result<(), PortError> {
        let _guard = self.guard();
        let path = self.root.join(META_FILE);
        fs::read(&path)
            .map(|_| ())
            .map_err(|err| io_err("read", &path, err))
    }
}

/// Take the directory's lock without waiting. The lock file stays in place
/// when the store closes: only the operating system's lock on the open file
/// counts.
fn acquire_lock(root: &Path) -> Result<File, OpenError> {
    use fs4::{FileExt, TryLockError};

    let path = root.join(LOCK_FILE);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|err| io_err("open", &path, err))?;
    // By its full name: `std::fs::File` has its own `try_lock`, which would
    // be called in place of the trait's.
    match FileExt::try_lock(&file) {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(OpenError::InUse),
        Err(TryLockError::Error(err)) => Err(io_err("lock", &path, err).into()),
    }
}

impl StorePort for FsPort {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        let _guard = self.guard();
        let index = self.scope_dir(scope).load_index()?;
        // The keys are in byte order, so the ones with the prefix are one
        // run that starts at the first key that is not smaller than it.
        let from = (Bound::Included(prefix.as_str()), Bound::Unbounded);
        let entries = index
            .documents
            .range::<str, _>(from)
            .take_while(|(key, _)| key.starts_with(prefix.as_str()))
            .map(|(key, entry)| {
                Ok(ListEntry {
                    key: DocKey::new(key.clone()).map_err(|err| PortError::Backend {
                        message: format!("a stored key is not valid: {err}"),
                    })?,
                    version: Version::new(entry.version).map_err(out_of_range)?,
                    modified_at: Timestamp::new(entry.modified_at).map_err(out_of_range)?,
                })
            })
            .collect::<Result<_, PortError>>()?;
        Ok(Listing {
            version: Version::new(index.version).map_err(out_of_range)?,
            entries,
        })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        let _guard = self.guard();
        let dir = self.scope_dir(scope);
        let index = dir.load_index()?;
        keys.iter()
            .map(|key| {
                let Some(entry) = index.documents.get(key.as_str()) else {
                    return Ok(None);
                };
                let path = dir.blob(entry.blob);
                Ok(Some(Document {
                    content: fs::read_to_string(&path).map_err(|err| io_err("read", &path, err))?,
                    version: Version::new(entry.version).map_err(out_of_range)?,
                    modified_at: Timestamp::new(entry.modified_at).map_err(out_of_range)?,
                }))
            })
            .collect()
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        let _guard = self.guard();
        let dir = self.scope_dir(scope);
        let mut index = dir.clean()?;

        // 1. Every expectation, before the first file.
        for op in commit.ops() {
            let actual = index
                .documents
                .get(op.key().as_str())
                .map(|entry| Version::new(entry.version))
                .transpose()
                .map_err(out_of_range)?;
            let expected = match op {
                Op::Put { expected, .. } => *expected,
                Op::Delete { expected, .. } => Expected::At(*expected),
            };
            let holds = match expected {
                Expected::Absent => actual.is_none(),
                Expected::At(version) => actual == Some(version),
            };
            if !holds {
                return Err(PortError::Conflict {
                    key: op.key().clone(),
                    expected,
                    actual,
                });
            }
        }
        let new_version = Version::new(index.version + 1).map_err(|_| PortError::Backend {
            message: "the scope version reached its limit".into(),
        })?;

        // 2. The new content files and event files. Nothing names them yet.
        if !dir.0.exists() {
            fs::create_dir_all(&dir.0).map_err(|err| io_err("create", &dir.0, err))?;
            sync_dir(&self.root.join(SCOPES_DIR))?;
        }
        let (blobs, events) = (dir.blobs(), dir.events());
        fs::create_dir_all(&blobs).map_err(|err| io_err("create", &blobs, err))?;
        fs::create_dir_all(&events).map_err(|err| io_err("create", &events, err))?;
        let mut replaced = Vec::new();
        for op in commit.ops() {
            let old = match op {
                Op::Put { key, content, .. } => {
                    let blob = index.next_blob;
                    index.next_blob += 1;
                    write_synced(&dir.blob(blob), content.as_bytes())?;
                    index.documents.insert(
                        key.as_str().to_string(),
                        Entry {
                            blob,
                            version: new_version.get(),
                            modified_at: commit.at().get(),
                        },
                    )
                }
                Op::Delete { key, .. } => index.documents.remove(key.as_str()),
            };
            replaced.extend(old.map(|entry| entry.blob));
        }
        for event in commit.events() {
            index.event_seq += 1;
            let record = json!({
                "name": event.name(),
                "payload": event.payload(),
                "version": new_version.get(),
            });
            write_synced(&dir.event(index.event_seq), record.to_string().as_bytes())?;
        }
        sync_dir(&blobs)?;
        sync_dir(&events)?;

        // 3. The one point that publishes the commit.
        index.version = new_version.get();
        dir.publish_index(&index)?;

        // 4. Best effort: a file that stays is removed by a later commit.
        for blob in replaced {
            let _ = fs::remove_file(dir.blob(blob));
        }
        Ok(new_version)
    }
}
