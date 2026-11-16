//! The store the demo server runs on (design D1, D2 of
//! server-on-port-examples).
//!
//! [`ServerStore`] is what the server needs from a store: the three
//! operations of the store port, the event log behind the event stream, a
//! health check and the store's kind. It lives here and not in the store
//! port contract: only the demo server has to treat several stores as one
//! kind of thing. The three store-port examples (SQLite, a directory,
//! PostgreSQL) and the in-memory reference implementation are its stores;
//! [`open`] builds the one the configuration names.

use crate::config::StoreConfig;
use speclink_store_port::{
    Commit, DocKey, Document, KeyPrefix, Listing, MemoryStorePort, PortError, RecordedEvent, ScopeId,
    StorePort, Version,
};
use speclink_store_port_fs::FsPort;
use speclink_store_port_postgres::PostgresPort;
use speclink_store_port_sqlite::SqlitePort;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

/// The store port, plus what the event stream, the health checks and the
/// admin pages need.
pub trait ServerStore: StorePort {
    /// The store's kind, as the configuration names it: `sqlite`,
    /// `serverfs`, `postgres` or `memory`.
    fn kind(&self) -> &'static str;

    /// The events of `scope` with a sequence larger than `after`, smallest
    /// first, at most `limit` of them, each with its sequence.
    fn events_after(
        &self,
        scope: &ScopeId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<(u64, RecordedEvent)>, PortError>;

    /// The largest sequence removed from `scope`; 0 when none was removed.
    fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError>;

    /// Remove the events of `scope` up to and including sequence `through`.
    fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError>;

    /// Check that the store can answer.
    fn probe(&self) -> Result<(), PortError>;
}

/// Read the events of `scope` with a sequence larger than `after`, oldest
/// first, in batches of `batch` events, and hand each batch to `each`. The
/// read ends at the first batch that is not full. `batch` is at least 1: the
/// configuration refuses a smaller event buffer.
pub(crate) fn each_event_batch(
    store: &dyn ServerStore,
    scope: &ScopeId,
    after: u64,
    batch: usize,
    mut each: impl FnMut(&[(u64, RecordedEvent)]),
) -> Result<(), PortError> {
    let mut cursor = after;
    loop {
        let events = store.events_after(scope, cursor, batch)?;
        if let Some((seq, _)) = events.last() {
            cursor = *seq;
        }
        each(&events);
        if events.len() < batch {
            return Ok(());
        }
    }
}

/// The newest event sequence of `scope`, given its floor; the floor when the
/// store keeps no event after it.
pub(crate) fn newest_event(
    store: &dyn ServerStore,
    scope: &ScopeId,
    floor: u64,
    batch: usize,
) -> Result<u64, PortError> {
    let mut newest = floor;
    each_event_batch(store, scope, floor, batch, |events| {
        if let Some((seq, _)) = events.last() {
            newest = *seq;
        }
    })?;
    Ok(newest)
}

// --- the in-memory store ---

/// The store port's in-memory reference implementation as a server store.
/// An event's sequence is its position in the scope's log; the floor is
/// kept here. For test configurations only.
#[derive(Default)]
pub struct Memory {
    port: MemoryStorePort,
    floors: Mutex<HashMap<ScopeId, u64>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory::default()
    }

    fn floor(&self, scope: &ScopeId) -> u64 {
        let floors = self.floors.lock().unwrap_or_else(PoisonError::into_inner);
        floors.get(scope).copied().unwrap_or(0)
    }
}

impl StorePort for Memory {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.port.list(scope, prefix)
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.port.read(scope, keys)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.port.commit(scope, commit)
    }
}

impl ServerStore for Memory {
    fn kind(&self) -> &'static str {
        "memory"
    }

    fn events_after(
        &self,
        scope: &ScopeId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        let after = after.max(self.floor(scope));
        Ok((1u64..)
            .zip(self.port.recorded_events(scope))
            .filter(|(seq, _)| *seq > after)
            .take(limit)
            .collect())
    }

    fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        Ok(self.floor(scope))
    }

    fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        let newest = self.port.recorded_events(scope).len() as u64;
        let mut floors = self.floors.lock().unwrap_or_else(PoisonError::into_inner);
        let floor = floors.entry(scope.clone()).or_insert(0);
        *floor = (*floor).max(through.min(newest));
        Ok(())
    }

    fn probe(&self) -> Result<(), PortError> {
        Ok(())
    }
}

// --- the three examples ---

/// An example is a server store by its own four methods of the same names:
/// there is no logic here.
macro_rules! example_store {
    ($store:ty, $kind:literal) => {
        impl ServerStore for $store {
            fn kind(&self) -> &'static str {
                $kind
            }

            fn events_after(
                &self,
                scope: &ScopeId,
                after: u64,
                limit: usize,
            ) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
                <$store>::events_after(self, scope, after, limit)
            }

            fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
                <$store>::event_floor(self, scope)
            }

            fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
                <$store>::trim_events(self, scope, through)
            }

            fn probe(&self) -> Result<(), PortError> {
                <$store>::probe(self)
            }
        }
    };
}

example_store!(SqlitePort, "sqlite");
example_store!(FsPort, "serverfs");
example_store!(PostgresPort, "postgres");

// --- opening the configured store ---

/// The markers speclink-server 0.12 and earlier wrote: the one of its SQLite
/// and PostgreSQL stores, and the one of its directory store.
const OLDER_VERSION_FORMATS: [&str; 2] = ["speclink-team-store", "speclink-team-store-fs"];

/// Why the configured store was not opened. In every case the data at the
/// location is as it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The location holds data of speclink-server 0.12 or earlier.
    OlderVersion { location: String },
    /// The location holds data this version does not open: another
    /// program's, or a speclink store of a layout this version does not
    /// know. The two fields are the marker and the layout version that were
    /// read, when there were any.
    NotSpeclink { location: String, format: Option<String>, layout_version: Option<String> },
    /// Any other failure: the data is in use, cannot be reached, cannot be
    /// read.
    Failed(String),
}

impl OpenError {
    /// The message with the path of the server's configuration file in the
    /// commands of [`OpenError::OlderVersion`]. [`fmt::Display`] leaves
    /// `<config>` there, for a caller that does not have the path.
    pub fn shown(&self, config: &Path) -> String {
        self.message(&command_argument(&config.display().to_string()))
    }

    fn message(&self, config: &str) -> String {
        match self {
            OpenError::OlderVersion { location } => format!(
                "error: the store at {location} was created by speclink-server 0.12 or earlier.\n\
                 This version uses a different layout and did not change your data.\n\
                 \n\
                 To move the data to this version:\n  \
                   1. With the old version, run:\n       \
                        speclink-server backup --config {config} --output <file>\n  \
                   2. Point the config at an empty store location and an empty identity database.\n  \
                   3. With this version, run:\n       \
                        speclink-server restore --config {config} --input <file>"
            ),
            OpenError::NotSpeclink { location, format, layout_version } => format!(
                "error: the data at {location} is not a speclink store of a layout this version opens \
                 (found format {} and layout version {}). The data was not changed.",
                format.as_deref().unwrap_or("(none)"),
                layout_version.as_deref().unwrap_or("(none)")
            ),
            OpenError::Failed(message) => message.clone(),
        }
    }
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message("<config>"))
    }
}

impl std::error::Error for OpenError {}

/// A path as one argument of a command a person copies: in double quotes
/// when it has a space.
fn command_argument(path: &str) -> String {
    if path.contains(char::is_whitespace) {
        format!("\"{path}\"")
    } else {
        path.to_string()
    }
}

/// An example's refusal of a layout as the server's: data of an older
/// version when the marker is one of the old drivers', else data that is
/// not this version's.
fn foreign(location: String, format: Option<String>, layout_version: Option<String>) -> OpenError {
    if format.as_deref().is_some_and(|format| OLDER_VERSION_FORMATS.contains(&format)) {
        OpenError::OlderVersion { location }
    } else {
        OpenError::NotSpeclink { location, format, layout_version }
    }
}

/// Open the store the configuration declares, fail closed: an open failure
/// (unreadable path, data of another layout, a directory already held by
/// another server) propagates rather than starting on a broken store, and
/// leaves the data as it was. Data of speclink-server 0.12 or earlier is
/// [`OpenError::OlderVersion`], whose message says how to move it. The crate
/// root names this function `build_store`.
pub fn open(store: &StoreConfig) -> Result<Arc<dyn ServerStore>, OpenError> {
    // The three examples each have their own error type, of one shape.
    macro_rules! opened {
        ($opened:expr, $example:ident, $location:expr, $failed:expr) => {
            match $opened {
                Ok(store) => Ok(Arc::new(store) as Arc<dyn ServerStore>),
                Err($example::OpenError::UnknownLayout { format, layout_version }) => {
                    Err(foreign($location, format, layout_version))
                }
                Err(e) => Err(OpenError::Failed(format!("{}: {e}", $failed))),
            }
        };
    }
    match store {
        StoreConfig::Memory => Ok(Arc::new(Memory::new())),
        StoreConfig::Sqlite { path } => opened!(
            SqlitePort::open(path),
            speclink_store_port_sqlite,
            path.display().to_string(),
            format!("cannot open sqlite store at '{}'", path.display())
        ),
        StoreConfig::ServerFs { path } => opened!(
            FsPort::open(path),
            speclink_store_port_fs,
            path.display().to_string(),
            format!("cannot open serverfs store at '{}'", path.display())
        ),
        StoreConfig::Postgres { url } => {
            // A password in the config file is accepted — some deployments have
            // nowhere else to put it — but it is a secret at rest in a file that
            // gets copied, diffed and pasted, so say so once at startup. The
            // URL itself is never echoed, here or in a failure: that would
            // print the very secret this warns about.
            if speclink_store_port_postgres::url_embeds_password(url) {
                eprintln!(
                    "warning: the postgres store url carries a password; \
                     prefer leaving it out and setting {} instead",
                    speclink_store_port_postgres::PASSWORD_VAR
                );
            }
            opened!(
                PostgresPort::connect(url),
                speclink_store_port_postgres,
                "the configured PostgreSQL database".to_string(),
                "cannot open postgres store"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store_port::{Actor, Event, Timestamp};

    fn scope() -> ScopeId {
        ScopeId::new("demo/backend").unwrap()
    }

    /// Commit `count` events to the scope, one commit each.
    fn commit_events(store: &Memory, count: u64) {
        for n in 1..=count {
            let event = Event::new("task-completed", serde_json::json!({ "n": n })).unwrap();
            let commit = Commit::new(
                Actor::new("u", "U").unwrap(),
                "t",
                Timestamp::new(1).unwrap(),
                Vec::new(),
                vec![event],
            )
            .unwrap();
            store.commit(&scope(), &commit).unwrap();
        }
    }

    fn seqs(events: Vec<(u64, RecordedEvent)>) -> Vec<u64> {
        events.into_iter().map(|(seq, _)| seq).collect()
    }

    #[test]
    fn the_memory_store_numbers_its_events_from_one_and_reads_them_in_batches() {
        let store = Memory::new();
        commit_events(&store, 5);
        assert_eq!(seqs(store.events_after(&scope(), 0, 10).unwrap()), [1, 2, 3, 4, 5]);
        assert_eq!(seqs(store.events_after(&scope(), 2, 2).unwrap()), [3, 4], "after 2, at most 2");
        assert_eq!(seqs(store.events_after(&scope(), 5, 10).unwrap()), [0u64; 0]);
        assert_eq!(seqs(store.events_after(&scope(), u64::MAX, 10).unwrap()), [0u64; 0]);
    }

    #[test]
    fn the_memory_store_trims_up_to_its_newest_event_and_never_lowers_the_floor() {
        let store = Memory::new();
        assert_eq!(store.event_floor(&scope()).unwrap(), 0, "a scope never written has floor 0");
        commit_events(&store, 5);
        store.trim_events(&scope(), 3).unwrap();
        assert_eq!(store.event_floor(&scope()).unwrap(), 3);
        assert_eq!(seqs(store.events_after(&scope(), 0, 10).unwrap()), [4, 5], "trimmed events are gone");
        store.trim_events(&scope(), 1).unwrap();
        assert_eq!(store.event_floor(&scope()).unwrap(), 3, "the floor does not go down");
        store.trim_events(&scope(), 500).unwrap();
        assert_eq!(store.event_floor(&scope()).unwrap(), 5, "the floor stops at the newest sequence");
        commit_events(&store, 1);
        assert_eq!(seqs(store.events_after(&scope(), 0, 10).unwrap()), [6], "the numbering continues");
    }

    #[test]
    fn the_newest_event_is_found_across_several_batches() {
        let store = Memory::new();
        assert_eq!(newest_event(&store, &scope(), 0, 2).unwrap(), 0, "no event: the floor");
        commit_events(&store, 5);
        assert_eq!(newest_event(&store, &scope(), 0, 2).unwrap(), 5);
        assert_eq!(newest_event(&store, &scope(), 0, 5).unwrap(), 5, "a full last batch reads once more");
        store.trim_events(&scope(), 5).unwrap();
        assert_eq!(newest_event(&store, &scope(), 5, 2).unwrap(), 5, "all removed: the floor");
    }
}
