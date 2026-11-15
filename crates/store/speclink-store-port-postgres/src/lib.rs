//! An example of the store port on one PostgreSQL schema. It is an example
//! to read and to copy, not a production store: it shows how list, read and
//! commit map to one database transaction, how commits on one scope take
//! turns with a row lock, and how the events of a commit go into an event log
//! in that same transaction. It has one connection and no connection pool, no
//! TLS, no migration between layout versions, no history of old document
//! versions, and it does not deliver events between several host instances.
//!
//! # The layout
//!
//! Four tables in the connection's current schema:
//!
//! - `meta(key, value)`: the marker `format = speclink-store-port` and
//!   `layout_version = 1`.
//! - `scopes(scope, version, event_seq, event_floor)`: one row per scope
//!   with its version, its last event sequence and its event floor.
//! - `documents(scope, key, content, version, modified_at)`
//! - `events(scope, seq, version, name, payload)`
//!
//! The scope and key columns are `text COLLATE "C"`, because the contract
//! orders keys by their bytes and the default collation of a database
//! follows its locale. The content is `bytea`, because `text` cannot hold a
//! NUL character and the contract permits one. The event payload is JSON in
//! a `text` column, because `jsonb` changes numbers and the order of keys.
//!
//! # A commit
//!
//! [`StorePort::commit`] runs in one transaction:
//!
//! 1. Insert the scope's row if it is not there, then lock it with
//!    `SELECT … FOR UPDATE`. Commits on one scope take turns on this lock,
//!    from every connection and every process. Commits on different scopes
//!    do not wait for each other.
//! 2. Check the expectation of every operation. One that does not hold
//!    rolls the transaction back and returns the conflict.
//! 3. Apply the writes and deletes with the new scope version.
//! 4. Insert the events with the next sequences.
//! 5. Update the scope's row and commit.
//!
//! # The connection
//!
//! A call that finds the connection gone connects again one time and runs
//! again. There is one exception: when the connection breaks after a commit
//! sent its `COMMIT`, the store cannot know whether the server applied it.
//! It does not run the commit again; it returns
//! [`PortError::Unavailable`]. When the caller tries again, the version
//! check rejects a commit that was applied, so nothing is written twice.
//!
//! The password comes from the environment variable [`PASSWORD_VAR`] when it
//! is set, and from the connection string when it is not. No error message
//! has the connection string or the password in it.
//!
//! # Outside the contract
//!
//! [`PostgresPort::events_after`], [`PostgresPort::event_floor`],
//! [`PostgresPort::trim_events`] and [`PostgresPort::probe`] are for the
//! host that owns the store. The store never trims events by itself.

use std::error::Error as _;
use std::fmt;
use std::sync::{Mutex, PoisonError};

use postgres::error::SqlState;
use postgres::{Client, Config, GenericClient, IsolationLevel, NoTls, Row, Transaction};
use speclink_store_port::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError,
    RecordedEvent, ScopeId, StorePort, Timestamp, Version,
};

/// The marker this example writes to `meta` and the only one it opens.
const FORMAT: &str = "speclink-store-port";
const LAYOUT_VERSION: &str = "1";

/// The environment variable that gives the password. When it is set and not
/// empty, it is used in place of a password in the connection string.
pub const PASSWORD_VAR: &str = "SPECLINK_POSTGRES_PASSWORD";

/// How long a commit waits for a scope that another commit holds.
const SESSION_SETUP: &str = "SET lock_timeout = '5s'";

const CREATE_TABLES: &str = r#"
CREATE TABLE meta (
    key   text PRIMARY KEY,
    value text NOT NULL
);
CREATE TABLE scopes (
    scope       text COLLATE "C" PRIMARY KEY,
    version     bigint NOT NULL,
    event_seq   bigint NOT NULL,
    event_floor bigint NOT NULL
);
CREATE TABLE documents (
    scope       text COLLATE "C" NOT NULL,
    key         text COLLATE "C" NOT NULL,
    content     bytea NOT NULL,
    version     bigint NOT NULL,
    modified_at bigint NOT NULL,
    PRIMARY KEY (scope, key)
);
CREATE TABLE events (
    scope   text COLLATE "C" NOT NULL,
    seq     bigint NOT NULL,
    version bigint NOT NULL,
    name    text NOT NULL,
    payload text NOT NULL,
    PRIMARY KEY (scope, seq)
);
"#;

/// Why [`PostgresPort::connect`] did not return a store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The schema holds data that this example did not write. The two
    /// fields are the values read from `meta`, or `None` when there is none.
    UnknownLayout {
        format: Option<String>,
        layout_version: Option<String>,
    },
    /// Another store object holds the data. The PostgreSQL example never
    /// returns it: two objects can connect to one schema.
    InUse,
    /// The database could not be reached or read.
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

impl From<postgres::Error> for OpenError {
    fn from(err: postgres::Error) -> Self {
        OpenError::Store(store_err(&err))
    }
}

// --- errors -----------------------------------------------------------------

/// The error and its causes in one line. `postgres::Error` alone prints only
/// a category, such as "db error"; the server's message is in the causes.
/// None of them has the connection string.
fn describe(err: &postgres::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// Whether the connection to the server is gone, as opposed to an answer
/// from the server that it refuses the statement.
fn is_connection_failure(err: &postgres::Error) -> bool {
    // True only from the second failure on a dead connection.
    if err.is_closed() {
        return true;
    }
    if let Some(code) = err.code() {
        // Class 08 is "connection exception". The other three are how a
        // server says that it stops or is not ready.
        return code.code().starts_with("08")
            || *code == SqlState::ADMIN_SHUTDOWN
            || *code == SqlState::CRASH_SHUTDOWN
            || *code == SqlState::CANNOT_CONNECT_NOW;
    }
    // No code: the server did not answer. A network failure has an io error
    // in its causes.
    let mut source = err.source();
    while let Some(cause) = source {
        if cause.downcast_ref::<std::io::Error>().is_some() {
            return true;
        }
        source = cause.source();
    }
    false
}

/// A lost connection and a lock that did not come in time are temporary;
/// every other failure is not.
fn store_err(err: &postgres::Error) -> PortError {
    let message = describe(err);
    let no_lock_in_time = [
        SqlState::LOCK_NOT_AVAILABLE,
        SqlState::T_R_DEADLOCK_DETECTED,
        SqlState::T_R_SERIALIZATION_FAILURE,
    ]
    .iter()
    .any(|code| err.code() == Some(code));
    if no_lock_in_time || is_connection_failure(err) {
        PortError::Unavailable { message }
    } else {
        PortError::Backend { message }
    }
}

/// A stored number that the contract's types refuse.
fn out_of_range<E>(_: E) -> PortError {
    PortError::Backend {
        message: "a stored version or time is out of range".into(),
    }
}

/// How one attempt of a call ended without a result.
enum Failure {
    /// An answer for the caller, such as a conflict.
    Port(PortError),
    /// The database failed before the call changed anything for good, so
    /// the call can run again.
    Db(postgres::Error),
    /// The database failed while a commit's `COMMIT` was on its way. The
    /// commit is possibly applied, so the call must not run again.
    Commit(postgres::Error),
}

impl From<PortError> for Failure {
    fn from(err: PortError) -> Self {
        Failure::Port(err)
    }
}

impl From<postgres::Error> for Failure {
    fn from(err: postgres::Error) -> Self {
        Failure::Db(err)
    }
}

// --- the connection ---------------------------------------------------------

fn parse_config(url: &str) -> Result<Config, PortError> {
    // The parser's message names an option or a position, never a value.
    url.parse()
        .map_err(|err: postgres::Error| PortError::Backend {
            message: format!(
                "cannot read the PostgreSQL connection string: {}",
                describe(&err)
            ),
        })
}

/// Whether the connection string has a password in it. A host can use this
/// to tell its operator that the password is better kept in
/// [`PASSWORD_VAR`]. A string that cannot be read has no password to report;
/// [`PostgresPort::connect`] says why it cannot be read.
pub fn url_embeds_password(url: &str) -> bool {
    url.parse::<Config>()
        .is_ok_and(|config| config.get_password().is_some())
}

fn connect_client(config: &Config) -> Result<Client, postgres::Error> {
    let mut client = config.connect(NoTls)?;
    client.batch_execute(SESSION_SETUP)?;
    Ok(client)
}

/// Bind a count from the contract as a `bigint`. Versions and times stop at
/// 2^53 − 1 and no sequence gets near `i64::MAX`, so only a number that a
/// caller made up is cut here.
fn db(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn number(row: &Row, column: usize) -> Result<u64, PortError> {
    u64::try_from(row.get::<_, i64>(column)).map_err(out_of_range)
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

fn meta_value(tx: &mut Transaction<'_>, key: &str) -> Result<Option<String>, postgres::Error> {
    let row = tx.query_opt("SELECT value::text FROM meta WHERE key = $1", &[&key])?;
    Ok(row.and_then(|row| row.get(0)))
}

/// Decide what the schema is, and make the tables in an empty one. All of it
/// is one transaction: a refused connect and a stop in the middle both leave
/// the schema as it was.
fn open_layout(client: &mut Client) -> Result<(), OpenError> {
    let mut tx = client.transaction()?;
    // Two objects that connect to a new schema at the same moment take turns
    // here, so the second one finds the tables of the first.
    tx.execute(
        "SELECT pg_advisory_xact_lock(hashtext($1), hashtext(current_schema()::text))",
        &[&FORMAT],
    )?;
    let has_meta: bool = tx
        .query_one(
            "SELECT EXISTS (
                 SELECT 1 FROM pg_tables
                 WHERE schemaname = current_schema() AND tablename = 'meta'
             )",
            &[],
        )?
        .get(0);
    if has_meta {
        let format = meta_value(&mut tx, "format")?;
        let layout_version = meta_value(&mut tx, "layout_version")?;
        if format.as_deref() != Some(FORMAT) || layout_version.as_deref() != Some(LAYOUT_VERSION) {
            return Err(OpenError::UnknownLayout {
                format,
                layout_version,
            });
        }
    } else {
        // Not `IF NOT EXISTS`: a table with one of these names that is
        // already there is not ours, and the failure rolls all of this back.
        tx.batch_execute(CREATE_TABLES)?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('format', $1), ('layout_version', $2)",
            &[&FORMAT, &LAYOUT_VERSION],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// The scope's version and last event sequence; 0 and 0 for a scope that
/// was never written.
fn scope_counters(
    client: &mut impl GenericClient,
    scope: &ScopeId,
) -> Result<(Version, u64), Failure> {
    let row = client.query_opt(
        "SELECT version, event_seq FROM scopes WHERE scope = $1",
        &[&scope.as_str()],
    )?;
    let (version, event_seq) = match row {
        Some(row) => (number(&row, 0)?, number(&row, 1)?),
        None => (0, 0),
    };
    Ok((Version::new(version).map_err(out_of_range)?, event_seq))
}

/// A transaction for a list or a read: every statement in it sees the same
/// moment, so a result never mixes two commits.
fn snapshot(client: &mut Client) -> Result<Transaction<'_>, postgres::Error> {
    client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()
}

struct Inner {
    /// Kept to connect again. The password is already in it, so a new
    /// connection does not read the environment a second time.
    config: Config,
    /// `None` after the connection was found gone.
    client: Option<Client>,
}

/// A store on one PostgreSQL schema. One connection behind one lock; a
/// second object (in this process or another) can connect to the same
/// schema, and the row lock of a scope then makes their commits take turns.
pub struct PostgresPort {
    inner: Mutex<Inner>,
}

impl PostgresPort {
    /// Connect to the database that `url` names and open the store in the
    /// connection's current schema. Put `search_path` in the connection
    /// string to choose the schema.
    ///
    /// - A schema with no `meta` table: the example makes its tables. Other
    ///   tables in the schema stay as they are.
    /// - A schema with this example's marker and layout version: opened as
    ///   it is.
    /// - Anything else: [`OpenError::UnknownLayout`], and the schema does
    ///   not change.
    pub fn connect(url: &str) -> Result<Self, OpenError> {
        let mut config = parse_config(url)?;
        // An empty variable is the same as no variable.
        if let Some(password) = std::env::var(PASSWORD_VAR).ok().filter(|p| !p.is_empty()) {
            config.password(password);
        }
        let mut client = connect_client(&config)?;
        open_layout(&mut client)?;
        Ok(Self {
            inner: Mutex::new(Inner {
                config,
                client: Some(client),
            }),
        })
    }

    /// Run `call` on the connection. When the connection is gone, connect
    /// again one time and run `call` again.
    fn run<T>(&self, call: impl Fn(&mut Client) -> Result<T, Failure>) -> Result<T, PortError> {
        // A transaction that ends in a panic rolls back when it drops, so
        // the state behind a poisoned lock is still good.
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        let mut ran_again = false;
        loop {
            if inner.client.as_ref().is_some_and(Client::is_closed) {
                inner.client = None;
            }
            if inner.client.is_none() {
                let client = connect_client(&inner.config).map_err(|err| store_err(&err))?;
                inner.client = Some(client);
            }
            let client = inner.client.as_mut().expect("connected just above");
            let err = match call(client) {
                Ok(value) => return Ok(value),
                Err(Failure::Port(err)) => return Err(err),
                Err(Failure::Db(err)) if is_connection_failure(&err) && !ran_again => {
                    inner.client = None;
                    ran_again = true;
                    continue;
                }
                Err(Failure::Db(err) | Failure::Commit(err)) => err,
            };
            if is_connection_failure(&err) {
                inner.client = None;
            }
            return Err(store_err(&err));
        }
    }

    /// The events of `scope` with a sequence larger than `after`, smallest
    /// first, at most `limit` of them. Each one comes with its sequence.
    ///
    /// Trimmed events are gone: compare `after` with
    /// [`PostgresPort::event_floor`] to learn whether some were skipped.
    pub fn events_after(
        &self,
        scope: &ScopeId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        self.run(|client| {
            let rows = client.query(
                "SELECT seq, name, payload, version FROM events
                 WHERE scope = $1 AND seq > $2 ORDER BY seq LIMIT $3",
                &[&scope.as_str(), &db(after), &limit],
            )?;
            let mut events = Vec::with_capacity(rows.len());
            for row in rows {
                let payload: String = row.get(2);
                let payload = serde_json::from_str(&payload).map_err(|err| PortError::Backend {
                    message: format!("a stored event payload is not JSON: {err}"),
                })?;
                events.push((
                    number(&row, 0)?,
                    RecordedEvent {
                        name: row.get(1),
                        payload,
                        version: Version::new(number(&row, 3)?).map_err(out_of_range)?,
                    },
                ));
            }
            Ok(events)
        })
    }

    /// The largest sequence that [`PostgresPort::trim_events`] removed from
    /// `scope`; 0 when nothing was removed.
    pub fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        self.run(|client| {
            let row = client.query_opt(
                "SELECT event_floor FROM scopes WHERE scope = $1",
                &[&scope.as_str()],
            )?;
            Ok(match row {
                Some(row) => number(&row, 0)?,
                None => 0,
            })
        })
    }

    /// Remove the events of `scope` up to and including sequence `through`.
    /// It stops at the last sequence in use, so later events continue the
    /// numbering. It changes no document and no version.
    pub fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        self.run(|client| {
            let mut tx = client.transaction()?;
            let row = tx.query_opt(
                "SELECT event_seq, event_floor FROM scopes WHERE scope = $1 FOR UPDATE",
                &[&scope.as_str()],
            )?;
            let Some(row) = row else {
                return Ok(());
            };
            let (event_seq, floor) = (number(&row, 0)?, number(&row, 1)?);
            let new_floor = through.min(event_seq);
            if new_floor <= floor {
                return Ok(());
            }
            tx.execute(
                "DELETE FROM events WHERE scope = $1 AND seq <= $2",
                &[&scope.as_str(), &db(new_floor)],
            )?;
            tx.execute(
                "UPDATE scopes SET event_floor = $2 WHERE scope = $1",
                &[&scope.as_str(), &db(new_floor)],
            )?;
            // To trim two times gives the same result, so this can run again.
            tx.commit()?;
            Ok(())
        })
    }

    /// Check that the store can answer: the database is there and the
    /// store's tables can be read.
    pub fn probe(&self) -> Result<(), PortError> {
        self.run(|client| {
            client.query("SELECT 1 FROM meta WHERE key = 'format'", &[])?;
            Ok(())
        })
    }
}

impl StorePort for PostgresPort {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        let upper = prefix_upper_bound(prefix.as_str());
        self.run(|client| {
            let mut tx = snapshot(client)?;
            let (version, _) = scope_counters(&mut tx, scope)?;
            // A range on the key, not LIKE: `%` and `_` in a key are
            // ordinary characters. The column's "C" collation gives the byte
            // order that the contract asks for.
            let rows = tx.query(
                "SELECT key, version, modified_at FROM documents
                 WHERE scope = $1 AND key >= $2 AND ($3::text IS NULL OR key < $3)
                 ORDER BY key",
                &[&scope.as_str(), &prefix.as_str(), &upper],
            )?;
            let mut entries = Vec::with_capacity(rows.len());
            for row in rows {
                let key: String = row.get(0);
                entries.push(ListEntry {
                    key: DocKey::new(key).map_err(|err| PortError::Backend {
                        message: format!("a stored key is not valid: {err}"),
                    })?,
                    version: Version::new(number(&row, 1)?).map_err(out_of_range)?,
                    modified_at: Timestamp::new(number(&row, 2)?).map_err(out_of_range)?,
                });
            }
            Ok(Listing { version, entries })
        })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.run(|client| {
            let mut tx = snapshot(client)?;
            let mut documents = Vec::with_capacity(keys.len());
            for key in keys {
                let row = tx.query_opt(
                    "SELECT content, version, modified_at FROM documents
                     WHERE scope = $1 AND key = $2",
                    &[&scope.as_str(), &key.as_str()],
                )?;
                documents.push(match row {
                    Some(row) => Some(Document {
                        content: String::from_utf8(row.get(0)).map_err(|_| PortError::Backend {
                            message: "a stored document is not UTF-8".into(),
                        })?,
                        version: Version::new(number(&row, 1)?).map_err(out_of_range)?,
                        modified_at: Timestamp::new(number(&row, 2)?).map_err(out_of_range)?,
                    }),
                    None => None,
                });
            }
            Ok(documents)
        })
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.run(|client| {
            // Every early return drops the transaction, which rolls it back.
            let mut tx = client.transaction()?;

            // 1. The scope's row, locked. A commit on the same scope from
            //    any connection waits here until this transaction ends, and
            //    then reads the versions that this one wrote.
            tx.execute(
                "INSERT INTO scopes (scope, version, event_seq, event_floor)
                 VALUES ($1, 0, 0, 0) ON CONFLICT (scope) DO NOTHING",
                &[&scope.as_str()],
            )?;
            let row = tx.query_one(
                "SELECT version, event_seq FROM scopes WHERE scope = $1 FOR UPDATE",
                &[&scope.as_str()],
            )?;
            let version = Version::new(number(&row, 0)?).map_err(out_of_range)?;
            let event_seq = number(&row, 1)?;

            // 2. Every expectation, before the first write.
            for op in commit.ops() {
                let row = tx.query_opt(
                    "SELECT version FROM documents WHERE scope = $1 AND key = $2",
                    &[&scope.as_str(), &op.key().as_str()],
                )?;
                let actual = match row {
                    Some(row) => Some(Version::new(number(&row, 0)?).map_err(out_of_range)?),
                    None => None,
                };
                let expected = match op {
                    Op::Put { expected, .. } => *expected,
                    Op::Delete { expected, .. } => Expected::At(*expected),
                };
                let holds = match expected {
                    Expected::Absent => actual.is_none(),
                    Expected::At(version) => actual == Some(version),
                };
                if !holds {
                    return Err(Failure::Port(PortError::Conflict {
                        key: op.key().clone(),
                        expected,
                        actual,
                    }));
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
                         VALUES ($1, $2, $3, $4, $5)
                         ON CONFLICT (scope, key) DO UPDATE SET
                             content = excluded.content,
                             version = excluded.version,
                             modified_at = excluded.modified_at",
                        &[
                            &scope.as_str(),
                            &key.as_str(),
                            &content.as_bytes(),
                            &db(new_version.get()),
                            &db(commit.at().get()),
                        ],
                    )?,
                    Op::Delete { key, .. } => tx.execute(
                        "DELETE FROM documents WHERE scope = $1 AND key = $2",
                        &[&scope.as_str(), &key.as_str()],
                    )?,
                };
            }

            // 4. The events, in the same transaction as the writes.
            let mut seq = event_seq;
            for event in commit.events() {
                seq += 1;
                tx.execute(
                    "INSERT INTO events (scope, seq, version, name, payload)
                     VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &scope.as_str(),
                        &db(seq),
                        &db(new_version.get()),
                        &event.name(),
                        &event.payload().to_string(),
                    ],
                )?;
            }

            // 5. The scope's row, then the one commit point. A failure of
            //    the COMMIT itself must not make the call run again.
            tx.execute(
                "UPDATE scopes SET version = $2, event_seq = $3 WHERE scope = $1",
                &[&scope.as_str(), &db(new_version.get()), &db(seq)],
            )?;
            tx.commit().map_err(Failure::Commit)?;
            Ok(new_version)
        })
    }
}
