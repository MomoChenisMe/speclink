//! An example of the store port on one SQLite file. It is an example to
//! read and to copy, not a production store: it shows how list, read and
//! commit map to one database transaction, and how the events of a commit go
//! into an event log in that same transaction. It has no connection pool, no
//! TLS, no migration between layout versions, no history of old document
//! versions, and it does not deliver events between several host instances.
//!
//! # The layout
//!
//! Four tables:
//!
//! - `meta(key, value)`: the marker `format = speclink-store-port` and
//!   `layout_version = 1`.
//! - `scopes(scope, version, event_seq, event_floor)`: one row per scope
//!   with its version, its last event sequence and its event floor.
//! - `documents(scope, key, content, version, modified_at)`
//! - `events(scope, seq, version, name, payload)`
//!
//! # A commit
//!
//! [`StorePort::commit`] runs in one `BEGIN IMMEDIATE` transaction:
//!
//! 1. Read the scope's version and event sequence.
//! 2. Check the expectation of every operation. One that does not hold
//!    rolls the transaction back and returns the conflict.
//! 3. Apply the writes and deletes with the new scope version.
//! 4. Insert the events with the next sequences.
//! 5. Update the scope's row and commit.
//!
//! # Outside the contract
//!
//! [`SqlitePort::events_after`], [`SqlitePort::event_floor`],
//! [`SqlitePort::trim_events`] and [`SqlitePort::probe`] are for the host
//! that owns the store. The store never trims events by itself.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rusqlite::{params, Connection, ErrorCode, OptionalExtension, TransactionBehavior};
use speclink_store_port::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError,
    RecordedEvent, ScopeId, StorePort, Timestamp, Version,
};

/// The marker this example writes to `meta` and the only one it opens.
const FORMAT: &str = "speclink-store-port";
const LAYOUT_VERSION: &str = "1";

/// How long a statement waits for a lock that another connection holds.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

const CREATE_TABLES: &str = "
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS scopes (
    scope       TEXT PRIMARY KEY NOT NULL,
    version     INTEGER NOT NULL,
    event_seq   INTEGER NOT NULL,
    event_floor INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
    scope       TEXT NOT NULL,
    key         TEXT NOT NULL,
    content     TEXT NOT NULL,
    version     INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    PRIMARY KEY (scope, key)
);
CREATE TABLE IF NOT EXISTS events (
    scope   TEXT NOT NULL,
    seq     INTEGER NOT NULL,
    version INTEGER NOT NULL,
    name    TEXT NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (scope, seq)
);
";

/// Why [`SqlitePort::open`] did not return a store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The file holds data that this example did not write. The two fields
    /// are the values read from `meta`, or `None` when there is none.
    UnknownLayout {
        format: Option<String>,
        layout_version: Option<String>,
    },
    /// Another store object holds the data. The SQLite example never
    /// returns it: two objects can open one file.
    InUse,
    /// The file could not be opened or read.
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
            OpenError::InUse => write!(f, "another store object holds the data"),
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

/// A busy or locked database is temporary; every other failure is not.
fn store_err(err: rusqlite::Error) -> PortError {
    let message = err.to_string();
    match err.sqlite_error_code() {
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) => {
            PortError::Unavailable { message }
        }
        _ => PortError::Backend { message },
    }
}

/// A stored number that the contract's types refuse.
fn out_of_range<E>(_: E) -> PortError {
    PortError::Backend {
        message: "a stored version or time is out of range".into(),
    }
}

/// The smallest string that sorts after every string with this prefix, or
/// `None` when there is no such string. Byte order of UTF-8 is code point
/// order, so the next code point gives the next string.
fn prefix_upper_bound(prefix: &str) -> Option<String> {
    let mut chars: Vec<char> = prefix.chars().collect();
    while let Some(last) = chars.pop() {
        // `find_map` steps over the surrogates, which are not characters.
        if let Some(next) = (last as u32 + 1..=char::MAX as u32).find_map(char::from_u32) {
            chars.push(next);
            return Some(chars.into_iter().collect());
        }
    }
    None
}

/// The value of one `meta` key as text, whatever type the column holds.
fn meta_value(conn: &Connection, key: &str) -> Result<Option<String>, PortError> {
    conn.query_row(
        "SELECT CAST(value AS TEXT) FROM meta WHERE key = ?1",
        [key],
        |row| row.get(0),
    )
    .optional()
    .map(Option::flatten)
    .map_err(store_err)
}

/// Decide what the file is. This only reads, so a refused open leaves the
/// file as it was. `Ok(true)` is an empty file that needs the tables.
fn needs_tables(conn: &Connection) -> Result<bool, OpenError> {
    let tables: u64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(store_err)?;
    if tables == 0 {
        return Ok(true);
    }
    // This example's `meta` table has a `key` and a `value` column. With no
    // such table, or one with other columns, there is no marker to read.
    let meta_columns: u64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('meta') WHERE name IN ('key', 'value')",
            [],
            |row| row.get(0),
        )
        .map_err(store_err)?;
    let (format, layout_version) = if meta_columns == 2 {
        (
            meta_value(conn, "format")?,
            meta_value(conn, "layout_version")?,
        )
    } else {
        (None, None)
    };
    if format.as_deref() == Some(FORMAT) && layout_version.as_deref() == Some(LAYOUT_VERSION) {
        Ok(false)
    } else {
        Err(OpenError::UnknownLayout {
            format,
            layout_version,
        })
    }
}

/// The scope's version and last event sequence; 0 and 0 for a scope that
/// was never written.
fn scope_counters(conn: &Connection, scope: &ScopeId) -> Result<(Version, u64), PortError> {
    let row: Option<(u64, u64)> = conn
        .query_row(
            "SELECT version, event_seq FROM scopes WHERE scope = ?1",
            [scope.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(store_err)?;
    let (version, event_seq) = row.unwrap_or((0, 0));
    Ok((Version::new(version).map_err(out_of_range)?, event_seq))
}

/// A store on one SQLite file. One connection behind one lock; a second
/// object (in this process or another) can open the same file, and SQLite's
/// own locking then makes their commits take turns.
pub struct SqlitePort {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl SqlitePort {
    /// Open the store at `path`.
    ///
    /// - No file, or a file with no table: the example makes its tables.
    /// - A file with this example's marker and layout version: opened as it
    ///   is.
    /// - Anything else: [`OpenError::UnknownLayout`], and the file does not
    ///   change.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, OpenError> {
        // An absolute path: a later change of the process's working
        // directory must not make `probe` look for the file in a new place.
        let path = std::path::absolute(path.as_ref()).map_err(|err| PortError::Backend {
            message: format!("resolve {}: {err}", path.as_ref().display()),
        })?;
        let path = path.as_path();
        // SQLite only says "unable to open database file" for this.
        if path.is_dir() {
            return Err(OpenError::Store(PortError::Backend {
                message: format!(
                    "{} is a directory; the store needs the path of a file",
                    path.display()
                ),
            }));
        }
        let mut conn = Connection::open(path).map_err(store_err)?;
        conn.busy_timeout(BUSY_TIMEOUT).map_err(store_err)?;
        let needs_tables = needs_tables(&conn)?;
        // Past the check: from here the file is ours to write.
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
            .map_err(store_err)?;
        if needs_tables {
            // One transaction makes the tables and the marker, so a stop in
            // the middle leaves an empty file. Another object can do the
            // same work first; `IF NOT EXISTS` and `OR IGNORE` then change
            // nothing.
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(store_err)?;
            tx.execute_batch(CREATE_TABLES).map_err(store_err)?;
            tx.execute(
                "INSERT OR IGNORE INTO meta (key, value) VALUES ('format', ?1), ('layout_version', ?2)",
                [FORMAT, LAYOUT_VERSION],
            )
            .map_err(store_err)?;
            tx.commit().map_err(store_err)?;
        }
        Ok(Self {
            path: path.to_path_buf(),
            conn: Mutex::new(conn),
        })
    }

    /// A transaction that ends in a panic rolls back when it drops, so the
    /// connection behind a poisoned lock is still good.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The events of `scope` with a sequence larger than `after`, smallest
    /// first, at most `limit` of them. Each one comes with its sequence.
    ///
    /// Trimmed events are gone: compare `after` with
    /// [`SqlitePort::event_floor`] to learn whether some were skipped.
    pub fn events_after(
        &self,
        scope: &ScopeId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        // SQLite integers stop at i64::MAX, and so do the sequences.
        let Ok(after) = i64::try_from(after) else {
            return Ok(Vec::new());
        };
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let conn = self.conn();
        let mut statement = conn
            .prepare(
                "SELECT seq, name, payload, version FROM events
                 WHERE scope = ?1 AND seq > ?2 ORDER BY seq LIMIT ?3",
            )
            .map_err(store_err)?;
        let rows = statement
            .query_map(params![scope.as_str(), after, limit], |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, u64>(3)?,
                ))
            })
            .map_err(store_err)?;
        let mut events = Vec::new();
        for row in rows {
            let (seq, name, payload, version) = row.map_err(store_err)?;
            let payload = serde_json::from_str(&payload).map_err(|err| PortError::Backend {
                message: format!("a stored event payload is not JSON: {err}"),
            })?;
            let version = Version::new(version).map_err(out_of_range)?;
            events.push((
                seq,
                RecordedEvent {
                    name,
                    payload,
                    version,
                },
            ));
        }
        Ok(events)
    }

    /// The largest sequence that [`SqlitePort::trim_events`] removed from
    /// `scope`; 0 when nothing was removed.
    pub fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        let floor: Option<u64> = self
            .conn()
            .query_row(
                "SELECT event_floor FROM scopes WHERE scope = ?1",
                [scope.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_err)?;
        Ok(floor.unwrap_or(0))
    }

    /// Remove the events of `scope` up to and including sequence `through`.
    /// It stops at the last sequence in use, so later events continue the
    /// numbering. It changes no document and no version.
    pub fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        let mut conn = self.conn();
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(store_err)?;
        let row: Option<(u64, u64)> = tx
            .query_row(
                "SELECT event_seq, event_floor FROM scopes WHERE scope = ?1",
                [scope.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(store_err)?;
        let Some((event_seq, floor)) = row else {
            return Ok(());
        };
        let new_floor = through.min(event_seq);
        if new_floor <= floor {
            return Ok(());
        }
        tx.execute(
            "DELETE FROM events WHERE scope = ?1 AND seq <= ?2",
            params![scope.as_str(), new_floor],
        )
        .map_err(store_err)?;
        tx.execute(
            "UPDATE scopes SET event_floor = ?2 WHERE scope = ?1",
            params![scope.as_str(), new_floor],
        )
        .map_err(store_err)?;
        tx.commit().map_err(store_err)
    }

    /// Check that the store can answer: the file is at its path, and a
    /// write works. The write leaves nothing: it takes the write lock,
    /// writes the marker over itself and rolls back. As a commit does, it
    /// waits for another writer and reports [`PortError::Unavailable`] when
    /// the wait is too long.
    pub fn probe(&self) -> Result<(), PortError> {
        // SQLite continues to read a file that was moved or deleted while
        // it is open, so a query alone does not see that the file is gone.
        if !self.path.is_file() {
            return Err(PortError::Backend {
                message: "the SQLite file is no longer at its path".into(),
            });
        }
        let mut conn = self.conn();
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(store_err)?;
        tx.execute("UPDATE meta SET value = value WHERE key = 'format'", [])
            .map_err(store_err)?;
        // The drop of the transaction rolls it back.
        Ok(())
    }
}

impl StorePort for SqlitePort {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        let conn = self.conn();
        // One read transaction: the version and the entries come from the
        // same commit.
        let tx = conn.unchecked_transaction().map_err(store_err)?;
        let (version, _) = scope_counters(&tx, scope)?;
        // A range on the key, not LIKE or GLOB: `%`, `_`, `*` and `[` in a
        // key are ordinary characters. The column compares bytes, which is
        // the order the contract asks for.
        let mut statement = tx
            .prepare(
                "SELECT key, version, modified_at FROM documents
                 WHERE scope = ?1 AND key >= ?2 AND (?3 IS NULL OR key < ?3)
                 ORDER BY key",
            )
            .map_err(store_err)?;
        let rows = statement
            .query_map(
                params![
                    scope.as_str(),
                    prefix.as_str(),
                    prefix_upper_bound(prefix.as_str())
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u64>(1)?,
                        row.get::<_, u64>(2)?,
                    ))
                },
            )
            .map_err(store_err)?;
        let mut entries = Vec::new();
        for row in rows {
            let (key, version, modified_at) = row.map_err(store_err)?;
            entries.push(ListEntry {
                key: DocKey::new(key).map_err(|err| PortError::Backend {
                    message: format!("a stored key is not valid: {err}"),
                })?,
                version: Version::new(version).map_err(out_of_range)?,
                modified_at: Timestamp::new(modified_at).map_err(out_of_range)?,
            });
        }
        Ok(Listing { version, entries })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        let conn = self.conn();
        // One read transaction: every document comes from the same commit.
        let tx = conn.unchecked_transaction().map_err(store_err)?;
        let mut statement = tx
            .prepare(
                "SELECT content, version, modified_at FROM documents WHERE scope = ?1 AND key = ?2",
            )
            .map_err(store_err)?;
        let mut documents = Vec::with_capacity(keys.len());
        for key in keys {
            let row: Option<(String, u64, u64)> = statement
                .query_row([scope.as_str(), key.as_str()], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .optional()
                .map_err(store_err)?;
            documents.push(match row {
                Some((content, version, modified_at)) => Some(Document {
                    content,
                    version: Version::new(version).map_err(out_of_range)?,
                    modified_at: Timestamp::new(modified_at).map_err(out_of_range)?,
                }),
                None => None,
            });
        }
        Ok(documents)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        let mut conn = self.conn();
        // IMMEDIATE takes the write lock now, so the versions read below
        // cannot change before this transaction writes. Every early return
        // drops the transaction, which rolls it back.
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(store_err)?;

        // 1. The scope's counters.
        let (version, event_seq) = scope_counters(&tx, scope)?;

        // 2. Every expectation, before the first write.
        for op in commit.ops() {
            let actual: Option<u64> = tx
                .query_row(
                    "SELECT version FROM documents WHERE scope = ?1 AND key = ?2",
                    [scope.as_str(), op.key().as_str()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(store_err)?;
            let actual = actual.map(Version::new).transpose().map_err(out_of_range)?;
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

        // 3. The writes and deletes, with the new scope version.
        let new_version = Version::new(version.get() + 1).map_err(|_| PortError::Backend {
            message: "the scope version reached its limit".into(),
        })?;
        for op in commit.ops() {
            match op {
                Op::Put { key, content, .. } => tx.execute(
                    "INSERT INTO documents (scope, key, content, version, modified_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT (scope, key) DO UPDATE SET
                         content = excluded.content,
                         version = excluded.version,
                         modified_at = excluded.modified_at",
                    params![
                        scope.as_str(),
                        key.as_str(),
                        content,
                        new_version.get(),
                        commit.at().get()
                    ],
                ),
                Op::Delete { key, .. } => tx.execute(
                    "DELETE FROM documents WHERE scope = ?1 AND key = ?2",
                    [scope.as_str(), key.as_str()],
                ),
            }
            .map_err(store_err)?;
        }

        // 4. The events, in the same transaction as the writes.
        let mut seq = event_seq;
        for event in commit.events() {
            seq += 1;
            tx.execute(
                "INSERT INTO events (scope, seq, version, name, payload) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    scope.as_str(),
                    seq,
                    new_version.get(),
                    event.name(),
                    event.payload().to_string()
                ],
            )
            .map_err(store_err)?;
        }

        // 5. The scope's row, then the one commit point.
        tx.execute(
            "INSERT INTO scopes (scope, version, event_seq, event_floor) VALUES (?1, ?2, ?3, 0)
             ON CONFLICT (scope) DO UPDATE SET
                 version = excluded.version,
                 event_seq = excluded.event_seq",
            params![scope.as_str(), new_version.get(), seq],
        )
        .map_err(store_err)?;
        tx.commit().map_err(store_err)?;
        Ok(new_version)
    }
}
