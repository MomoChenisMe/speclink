//! PostgreSQL driver for the TeamStore contract.
//!
//! One database schema persists an entire store: documents, immutable history,
//! the transactional outbox, and the schema/version marker. Every commit is a
//! single SQL transaction, so partial writes never survive a failure, and each
//! scope's writes are serialized by a transaction-scoped advisory lock — the
//! lock dies with the transaction (or with the connection), so a crashed
//! process leaves nothing latched. The driver is single-node: it serializes
//! writers, it does not coordinate a cluster.
//!
//! The client is synchronous, matching the contract's no-async-runtime stance;
//! async hosts adapt at their own boundary.
//!
//! Depends only on `speclink-store` (the contract and its conformance suite)
//! and `postgres`; it does not touch `speclink-core` or any other crate.

mod schema;

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use postgres::error::SqlState;
use postgres::{Client, Config, GenericClient, IsolationLevel, NoTls};

use speclink_store::{
    content_digest, Bundle, BundleDoc, Capability, CapabilityLevel, CommandContext, DocRef,
    Document, DocumentId, EventRecord, ExpectedRevision, ImportMode, ImportOutcome, ImportReport,
    ImportedDoc, Manifest, OutboxCursor, OutboxEntry, Revision, RevisionKind, RevisionRecord,
    FaultPoint, Scope, Snapshot, StagedOp, StoreError, TeamStore, UnitOfWork,
    BUNDLE_FORMAT_VERSION,
    CONTRACT_VERSION,
};

/// Unit separator, used to join the fields of an encoded document id and the
/// components of a `meta` key. No logical identifier contains it.
const SEP: char = '\u{1f}';

/// The environment variable that supplies the password when the connection URL
/// carries none. A deployment secret belongs in the environment, not in a
/// config file that gets copied, diffed and pasted.
pub const PASSWORD_VAR: &str = "SPECLINK_POSTGRES_PASSWORD";

// --- error mapping --------------------------------------------------------

/// Flatten an error and its causes into one line. `postgres::Error` renders as
/// a bare category ("db error") on its own — the server's actual complaint
/// lives in the source chain, and that is the part a reader needs.
fn describe(err: &postgres::Error) -> String {
    let mut description = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        description.push_str(": ");
        description.push_str(&cause.to_string());
        source = cause.source();
    }
    description
}

/// Whether a failure means the link to the server is gone, as opposed to the
/// server having answered with a complaint.
///
/// A severed connection arrives in one of two shapes, and which one lands is a
/// race: the server usually announces its departure with a FATAL message
/// carrying a SQLSTATE, but if the socket closes first the client only sees a
/// transport error with no SQLSTATE at all. Both are the same event, so both
/// are checked here.
///
/// `postgres::Error::is_closed` alone is not enough: it only becomes true from
/// the *second* failure on, because the error that first observes the dropped
/// connection is the one that discovers it.
fn is_connection_failure(err: &postgres::Error) -> bool {
    if err.is_closed() {
        return true;
    }
    if let Some(code) = err.code() {
        // The server answered. Class 08 is the connection exception class.
        // Class 57 is operator intervention, most of which is *not* about the
        // link — a cancelled query is not a lost connection — so only the
        // three members that announce the server's own departure count.
        return code.code().starts_with("08")
            || *code == SqlState::ADMIN_SHUTDOWN
            || *code == SqlState::CRASH_SHUTDOWN
            || *code == SqlState::CANNOT_CONNECT_NOW;
    }
    // No SQLSTATE: the server never answered. A transport failure carries an
    // io error; anything else is a client-side defect worth surfacing as-is.
    let mut source = err.source();
    while let Some(cause) = source {
        if cause.downcast_ref::<std::io::Error>().is_some() {
            return true;
        }
        source = cause.source();
    }
    false
}

/// Map a failure during ordinary operation onto the closed store error set: a
/// severed connection is transient (`unavailable`), everything else is a
/// backend failure carrying the reason.
fn map_pg(err: postgres::Error) -> StoreError {
    if is_connection_failure(&err) {
        StoreError::Unavailable
    } else {
        StoreError::Backend {
            source: describe(&err),
        }
    }
}

/// Map a failure to *open* a store. An open failure is a startup failure and
/// always carries its reason: there is no store yet that could be temporarily
/// away, so it never travels as `unavailable`. Bad credentials and a missing
/// database both land here.
fn open_failure(err: postgres::Error) -> StoreError {
    StoreError::Backend {
        source: describe(&err),
    }
}

// --- connection configuration ---------------------------------------------

/// Parse a connection URL, or say why it cannot be one.
fn parse_config(url: &str) -> Result<Config, StoreError> {
    url.parse::<Config>()
        .map_err(|e| StoreError::Backend {
            source: format!("cannot parse the postgres connection url: {}", describe(&e)),
        })
}

/// Parse `url` and fill the password in from the environment when it carries
/// none. A password already in the URL wins: the variable completes a
/// configuration, it does not override one.
///
/// The password is applied to the *parsed* configuration rather than spliced
/// back into the URL text. Rebuilding a connection string by hand would have to
/// re-encode the secret, and would silently drop every parameter it did not
/// think to copy — `sslmode` among them, which would quietly turn a TLS-only
/// deployment into a cleartext one.
fn resolve_config(url: &str) -> Result<Config, StoreError> {
    let mut config = parse_config(url)?;
    if config.get_password().is_none() {
        // An exported-but-empty variable means "unset", not "the empty
        // password": treating it as a secret would send a blank one and blame
        // the server for refusing it.
        if let Ok(password) = std::env::var(PASSWORD_VAR) {
            if !password.is_empty() {
                config.password(password);
            }
        }
    }
    Ok(config)
}

/// Whether `url` carries a password inline.
///
/// Startup uses this to warn. An inline password is accepted — refusing one
/// would strand deployments that have nowhere else to put it — but it is a
/// secret at rest in a file, and worth saying so once.
pub fn url_embeds_password(url: &str) -> Result<bool, StoreError> {
    Ok(parse_config(url)?.get_password().is_some())
}

// --- document id and meta-key encoding ------------------------------------

/// Encode a document id as a stable string key: a short tag followed by the
/// identifying fields, separated by [`SEP`]. Deterministic, and the same
/// encoding the SQLite driver uses.
fn encode_doc(doc: &DocumentId) -> String {
    match doc {
        DocumentId::ChangeMeta { change } => format!("cm{SEP}{change}"),
        DocumentId::ChangeArtifact { change, artifact } => {
            format!("ca{SEP}{change}{SEP}{artifact}")
        }
        DocumentId::CanonicalSpec { capability } => format!("cs{SEP}{capability}"),
        DocumentId::Discussion { slug, archived } => {
            format!("di{SEP}{}{SEP}{slug}", if *archived { 1 } else { 0 })
        }
        DocumentId::WorkflowConfig => "wc".to_string(),
        DocumentId::ArchivedChange { change, doc } => format!("ac{SEP}{change}{SEP}{doc}"),
        DocumentId::Language => "lg".to_string(),
    }
}

/// Reverse of [`encode_doc`]. A key that does not match the closed set of
/// shapes is persisted corruption, surfaced as [`StoreError::Corrupt`].
fn decode_doc(key: &str) -> Result<DocumentId, StoreError> {
    let corrupt = || StoreError::Corrupt {
        reason: format!("undecodable document id: {key:?}"),
    };
    let parts: Vec<&str> = key.split(SEP).collect();
    match parts.as_slice() {
        ["wc"] => Ok(DocumentId::WorkflowConfig),
        ["lg"] => Ok(DocumentId::Language),
        ["cm", change] => Ok(DocumentId::ChangeMeta {
            change: (*change).to_string(),
        }),
        ["cs", capability] => Ok(DocumentId::CanonicalSpec {
            capability: (*capability).to_string(),
        }),
        ["ca", change, artifact] => Ok(DocumentId::ChangeArtifact {
            change: (*change).to_string(),
            artifact: (*artifact).to_string(),
        }),
        ["ac", change, doc] => Ok(DocumentId::ArchivedChange {
            change: (*change).to_string(),
            doc: (*doc).to_string(),
        }),
        ["di", flag, slug] => Ok(DocumentId::Discussion {
            slug: (*slug).to_string(),
            archived: match *flag {
                "1" => true,
                "0" => false,
                _ => return Err(corrupt()),
            },
        }),
        _ => Err(corrupt()),
    }
}

/// `meta` key for a project's monotonic revision counter.
fn project_rev_key(project: &str) -> String {
    format!("project_rev{SEP}{project}")
}

/// `meta` key for a scope's durable outbox ack cursor.
fn acked_key(scope: &Scope) -> String {
    format!(
        "acked{SEP}{}{SEP}{}",
        scope.project.as_str(),
        scope.repo.as_str()
    )
}

// --- row decoding helpers -------------------------------------------------

/// Parse an RFC 3339 timestamp back to UTC; a malformed value is corruption.
fn parse_ts(text: &str) -> Result<DateTime<Utc>, StoreError> {
    DateTime::parse_from_rfc3339(text)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| StoreError::Corrupt {
            reason: format!("unparseable timestamp {text:?}: {e}"),
        })
}

/// Reconstruct a [`RevisionKind`] from its stored discriminant and digest.
fn revision_kind(kind: &str, digest: Option<String>) -> Result<RevisionKind, StoreError> {
    match kind {
        "write" => Ok(RevisionKind::Write {
            digest: digest.ok_or_else(|| StoreError::Corrupt {
                reason: "write history record missing its digest".into(),
            })?,
        }),
        "tombstone" => Ok(RevisionKind::Tombstone),
        other => Err(StoreError::Corrupt {
            reason: format!("unknown history kind {other:?}"),
        }),
    }
}

/// Read a `meta` value as a `u64`, defaulting to 0 when the key is absent.
fn read_meta_u64<C: GenericClient>(client: &mut C, key: &str) -> Result<u64, StoreError> {
    let row = client
        .query_opt("SELECT value FROM meta WHERE key = $1", &[&key])
        .map_err(map_pg)?;
    match row {
        None => Ok(0),
        Some(row) => {
            let text: String = row.get(0);
            text.parse().map_err(|_| StoreError::Corrupt {
                reason: format!("meta value for {key:?} is not a number: {text:?}"),
            })
        }
    }
}

/// The revision a document currently sits at, or `None` when it does not exist.
fn current_revision<C: GenericClient>(
    client: &mut C,
    project: &str,
    repo: &str,
    doc_id: &str,
) -> Result<Option<u64>, StoreError> {
    let row = client
        .query_opt(
            "SELECT revision FROM documents WHERE project = $1 AND repo = $2 AND doc_id = $3",
            &[&project, &repo, &doc_id],
        )
        .map_err(map_pg)?;
    Ok(row.map(|row| row.get::<_, i64>(0) as u64))
}

/// Whether a scope holds any document at all — the question create-new asks.
fn scope_holds_any_document<C: GenericClient>(
    client: &mut C,
    project: &str,
    repo: &str,
) -> Result<bool, StoreError> {
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM documents WHERE project = $1 AND repo = $2)",
            &[&project, &repo],
        )
        .map_err(map_pg)?;
    Ok(row.get(0))
}

/// Record a project's revision counter.
fn write_project_revision<C: GenericClient>(
    client: &mut C,
    project: &str,
    revision: Revision,
) -> Result<(), StoreError> {
    client
        .execute(
            "INSERT INTO meta (key, value) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            &[&project_rev_key(project), &revision.0.to_string()],
        )
        .map_err(map_pg)?;
    Ok(())
}

// --- schema inspection ----------------------------------------------------

/// Read a `meta` value, or `None` when the key is absent.
fn read_meta(client: &mut Client, key: &str) -> Result<Option<String>, StoreError> {
    let rows = client
        .query("SELECT value FROM meta WHERE key = $1", &[&key])
        .map_err(map_pg)?;
    Ok(rows.first().map(|row| row.get(0)))
}

/// Whether a table of the given name exists in the connection's schema.
fn table_exists(client: &mut Client, name: &str) -> Result<bool, StoreError> {
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
             WHERE table_schema = current_schema() AND table_name = $1)",
            &[&name],
        )
        .map_err(map_pg)?;
    Ok(row.get(0))
}

/// Whether the connection's schema holds any table at all.
fn has_any_table(client: &mut Client) -> Result<bool, StoreError> {
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
             WHERE table_schema = current_schema())",
            &[],
        )
        .map_err(map_pg)?;
    Ok(row.get(0))
}

/// Create the schema and seed the meta marker/version in one transaction, so a
/// torn init leaves nothing behind.
fn initialize_schema(client: &mut Client) -> Result<(), StoreError> {
    let mut tx = client.transaction().map_err(map_pg)?;
    tx.batch_execute(schema::SCHEMA_SQL).map_err(map_pg)?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES ($1, $2)",
        &[&schema::META_FORMAT_KEY, &schema::STORE_MARKER],
    )
    .map_err(map_pg)?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES ($1, $2)",
        &[
            &schema::META_VERSION_KEY,
            &schema::SCHEMA_VERSION.to_string(),
        ],
    )
    .map_err(map_pg)?;
    tx.commit().map_err(map_pg)
}

// --- the commit transaction -----------------------------------------------

/// Apply one unit of work in a single SQL transaction: take the scope's lock,
/// validate every CAS precondition, write documents, append history, append the
/// outbox, and advance the project revision — commit only at the very end, so
/// nothing survives a failure partway through. Any early return drops the
/// transaction, which rolls it back and releases the lock.
fn commit_txn(
    client: &mut Client,
    uow: &UnitOfWork,
    events: &[EventRecord],
    pending_crash: Option<FaultPoint>,
    fail_outbox: bool,
    crashed: &mut bool,
) -> Result<Revision, StoreError> {
    let scope = uow.scope();
    let project = scope.project.as_str();
    let repo = scope.repo.as_str();

    let mut tx = client.transaction().map_err(map_pg)?;

    // Serialize this scope's writers across every connection, before reading
    // anything — that is what makes the read-then-write of the revision
    // counter and the CAS checks below safe. The lock is transaction-scoped:
    // it releases on commit or abort and dies with the connection, so a writer
    // that vanishes mid-commit latches nothing.
    tx.execute(
        "SELECT pg_advisory_xact_lock($1)",
        &[&PostgresTeamStore::advisory_lock_key(scope)],
    )
    .map_err(map_pg)?;

    let next = Revision(read_meta_u64(&mut tx, &project_rev_key(project))? + 1);

    // 1. Validate every precondition against the pre-commit state before
    //    touching anything: any mismatch rejects the whole commit.
    for op in uow.ops() {
        let doc_id = encode_doc(op.doc());
        let current = current_revision(&mut tx, project, repo, &doc_id)?;
        let expected = match op {
            StagedOp::Put { expected, .. } => *expected,
            StagedOp::Delete { expected, .. } => ExpectedRevision::At(*expected),
        };
        let satisfied = match expected {
            ExpectedRevision::Absent => current.is_none(),
            ExpectedRevision::At(revision) => current == Some(revision.0),
        };
        if !satisfied {
            return Err(StoreError::RevisionConflict {
                doc: DocRef::new(scope, op.doc().clone()),
                expected,
                actual: current.map(Revision),
            });
        }
    }

    // 2. Document writes.
    for op in uow.ops() {
        let doc_id = encode_doc(op.doc());
        match op {
            StagedOp::Put { content, .. } => {
                tx.execute(
                    "INSERT INTO documents (project, repo, doc_id, content, revision, digest) \
                     VALUES ($1, $2, $3, $4, $5, $6) \
                     ON CONFLICT (project, repo, doc_id) DO UPDATE SET \
                     content = EXCLUDED.content, revision = EXCLUDED.revision, \
                     digest = EXCLUDED.digest",
                    &[
                        &project,
                        &repo,
                        &doc_id,
                        content,
                        &(next.0 as i64),
                        &content_digest(content),
                    ],
                )
                .map_err(map_pg)?;
            }
            StagedOp::Delete { .. } => {
                tx.execute(
                    "DELETE FROM documents WHERE project = $1 AND repo = $2 AND doc_id = $3",
                    &[&project, &repo, &doc_id],
                )
                .map_err(map_pg)?;
            }
        }
    }

    if pending_crash == Some(FaultPoint::AfterDocWrites) {
        *crashed = true;
        return Err(StoreError::Unavailable);
    }

    // 3. History append (all records share this commit's timestamp).
    let now = Utc::now().to_rfc3339();
    for op in uow.ops() {
        let doc_id = encode_doc(op.doc());
        let (kind, digest): (&str, Option<String>) = match op {
            StagedOp::Put { content, .. } => ("write", Some(content_digest(content))),
            StagedOp::Delete { .. } => ("tombstone", None),
        };
        tx.execute(
            "INSERT INTO history \
             (project, repo, doc_id, revision, actor, at, command, kind, digest) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &project,
                &repo,
                &doc_id,
                &(next.0 as i64),
                &uow.context().actor,
                &now,
                &uow.context().command,
                &kind,
                &digest,
            ],
        )
        .map_err(map_pg)?;
    }

    if pending_crash == Some(FaultPoint::AfterHistoryAppend) {
        *crashed = true;
        return Err(StoreError::Unavailable);
    }
    if pending_crash == Some(FaultPoint::BeforeOutboxAppend) {
        *crashed = true;
        return Err(StoreError::Unavailable);
    }

    // An armed outbox failure is a real statement error rather than a
    // synthetic early return, so the abort travels the path a genuine one
    // takes: PostgreSQL marks the transaction failed, and the rollback on drop
    // is what undoes the stages above. The commit absorbs it — this is an
    // error, not a crash, and the store stays usable.
    if fail_outbox {
        tx.execute(
            "INSERT INTO outbox (project, repo, seq, revision, name, payload, actor, at) \
             VALUES ($1, $2, NULL, NULL, NULL, NULL, NULL, NULL)",
            &[&project, &repo],
        )
        .map_err(map_pg)?;
    }

    // 4. Outbox append, at monotonic per-scope sequence numbers. The scope's
    //    lock is what makes reading the high-water mark and writing past it
    //    safe from another connection doing the same.
    let mut seq = tx
        .query_one(
            "SELECT COALESCE(MAX(seq), 0) FROM outbox WHERE project = $1 AND repo = $2",
            &[&project, &repo],
        )
        .map_err(map_pg)?
        .get::<_, i64>(0) as u64;
    for event in events {
        seq += 1;
        tx.execute(
            "INSERT INTO outbox \
             (project, repo, seq, revision, name, payload, actor, at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &project,
                &repo,
                &(seq as i64),
                &(next.0 as i64),
                &event.name,
                &event.payload.to_string(),
                &event.actor,
                &event.at.to_rfc3339(),
            ],
        )
        .map_err(map_pg)?;
    }

    if pending_crash == Some(FaultPoint::AfterOutboxAppend) {
        *crashed = true;
        return Err(StoreError::Unavailable);
    }

    // 5. Advance the project revision, then the atomic commit.
    write_project_revision(&mut tx, project, next)?;
    tx.commit().map_err(map_pg)?;
    Ok(next)
}

// --- the store ------------------------------------------------------------

/// State behind the store's single mutex: the connection plus what the open
/// gate learned about it. Serializing here is this instance's write guarantee;
/// across instances the advisory lock is.
struct Inner {
    /// Kept so a dead connection can be rebuilt without reopening the store —
    /// resolved once, so a reconnect cannot lose the password the environment
    /// supplied.
    config: Config,
    /// `None` once the connection is known to be gone.
    client: Option<Client>,
    /// The schema records a version below the current one.
    needs_migration: bool,
    /// Crash the next commit at this stage boundary (test hook).
    pending_crash: Option<FaultPoint>,
    /// Make the next commit's outbox append fail (test hook).
    pending_outbox_failure: bool,
    /// A crashed store serves nothing until rebuilt from durable state.
    crashed: bool,
}

impl Inner {
    fn unavailable_if_crashed(&self) -> Result<(), StoreError> {
        if self.crashed {
            Err(StoreError::Unavailable)
        } else {
            Ok(())
        }
    }

    /// A live client, rebuilt if the previous one died.
    ///
    /// A connection that broke while the store was running is not a reason to
    /// throw the store away: the durable state is untouched and the schema has
    /// already passed the open gate, so reconnecting is the whole recovery. A
    /// server that is still away leaves the store `unavailable` — the link is
    /// missing, not the data.
    fn client(&mut self) -> Result<&mut Client, StoreError> {
        if self.client.as_ref().is_some_and(Client::is_closed) {
            self.client = None;
        }
        if self.client.is_none() {
            self.client = Some(
                self.config
                    .connect(NoTls)
                    .map_err(|_| StoreError::Unavailable)?,
            );
        }
        Ok(self.client.as_mut().expect("just connected"))
    }
}

/// A PostgreSQL-backed [`TeamStore`](speclink_store::TeamStore). Construct with
/// [`PostgresTeamStore::connect`].
pub struct PostgresTeamStore {
    inner: Mutex<Inner>,
}

impl PostgresTeamStore {
    /// Open (or initialize) a store on the schema `url` selects.
    ///
    /// The tables land in the connection's current schema, so a `search_path`
    /// in the URL isolates a store within a shared database.
    ///
    /// Fails closed on anything it does not recognize. State detection is
    /// read-only and happens **before** any write, so a refused open leaves the
    /// target untouched:
    ///
    /// - unreachable server, bad credentials, missing database → [`StoreError::Backend`];
    /// - a schema recording a higher version → [`StoreError::Corrupt`];
    /// - a schema holding tables that are not a speclink store → `Corrupt`;
    /// - an empty schema → initialized at the current version;
    /// - a valid store at a lower version → opens, flagged as needing migration.
    pub fn connect(url: &str) -> Result<Self, StoreError> {
        let config = resolve_config(url)?;
        let mut client = config.connect(NoTls).map_err(open_failure)?;

        // Read-only detection first — no write reaches a schema we go on to
        // refuse.
        let has_meta = table_exists(&mut client, "meta")?;
        let mut needs_migration = false;
        if has_meta {
            let marker = read_meta(&mut client, schema::META_FORMAT_KEY)?;
            if marker.as_deref() != Some(schema::STORE_MARKER) {
                return Err(StoreError::Corrupt {
                    reason: "schema has a meta table but not the speclink store marker".into(),
                });
            }
            let version_text = read_meta(&mut client, schema::META_VERSION_KEY)?;
            let version: u32 = match version_text.as_deref().map(str::parse::<u32>) {
                Some(Ok(version)) => version,
                _ => {
                    return Err(StoreError::Corrupt {
                        reason: "schema version is missing or unparseable".into(),
                    })
                }
            };
            if version > schema::SCHEMA_VERSION {
                return Err(StoreError::Corrupt {
                    reason: format!(
                        "incompatible schema version {version}; this driver supports {}",
                        schema::SCHEMA_VERSION
                    ),
                });
            }
            needs_migration = version < schema::SCHEMA_VERSION;
        } else if has_any_table(&mut client)? {
            return Err(StoreError::Corrupt {
                reason: "existing PostgreSQL schema is not a speclink team store".into(),
            });
        }

        // Past the gate: a valid store, or an empty schema to initialize.
        if !has_meta {
            initialize_schema(&mut client)?;
        }

        Ok(Self {
            inner: Mutex::new(Inner {
                config,
                client: Some(client),
                needs_migration,
                pending_crash: None,
                pending_outbox_failure: false,
                crashed: false,
            }),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("postgres store mutex poisoned")
    }

    /// The 64-bit key a scope's writers serialize on.
    ///
    /// FNV-1a over the scope's identity — deterministic across processes,
    /// builds and PostgreSQL versions, which a `DefaultHasher` would not be,
    /// and two processes that disagreed on the key would not be serialized at
    /// all. A collision would make two unrelated scopes take turns: a
    /// throughput cost, never a correctness one.
    ///
    /// Public so tests can hold a scope's lock the way a mid-commit writer
    /// does; not part of the stable contract surface.
    #[doc(hidden)]
    pub fn advisory_lock_key(scope: &Scope) -> i64 {
        const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;

        let mut hash = OFFSET_BASIS;
        for byte in scope
            .project
            .as_str()
            .bytes()
            .chain(std::iter::once(SEP as u8))
            .chain(scope.repo.as_str().bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
        // PostgreSQL advisory-lock keys are signed; the whole 64-bit space is
        // addressable either way, so the reinterpretation loses nothing.
        hash as i64
    }

    /// Test-only fault hook: crash the next commit at `point`. Not part of the
    /// stable contract surface (hidden from docs); it exists so the conformance
    /// harness can drive crash recovery. See design decision 4.
    #[doc(hidden)]
    pub fn crash_at(&self, point: FaultPoint) {
        self.lock().pending_crash = Some(point);
    }

    /// Test-only fault hook: make the next commit's outbox append fail — an
    /// error the commit must absorb, not a crash. Hidden from docs.
    #[doc(hidden)]
    pub fn fail_outbox_append(&self) {
        self.lock().pending_outbox_failure = true;
    }
}

/// A fixed-point view: owns a copy of the scope's documents and history taken
/// at snapshot time, so commits that land afterwards cannot reach into it.
/// Keyed by encoded document id (see [`encode_doc`]).
struct PostgresSnapshot {
    revision: Revision,
    docs: BTreeMap<String, (String, Revision)>,
    history: BTreeMap<String, Vec<RevisionRecord>>,
}

impl Snapshot for PostgresSnapshot {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn read(&self, doc: &DocumentId) -> Result<Option<Document>, StoreError> {
        Ok(self
            .docs
            .get(&encode_doc(doc))
            .map(|(content, revision)| Document {
                content: content.clone(),
                revision: *revision,
            }))
    }

    fn history(&self, doc: &DocumentId) -> Result<Vec<RevisionRecord>, StoreError> {
        Ok(self
            .history
            .get(&encode_doc(doc))
            .cloned()
            .unwrap_or_default())
    }
}

impl TeamStore for PostgresTeamStore {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: CONTRACT_VERSION,
            driver: "postgres".into(),
            level: CapabilityLevel::SingleNode,
            capabilities: [
                Capability::Snapshot,
                Capability::Cas,
                Capability::Transaction,
                Capability::History,
                Capability::Outbox,
                Capability::Migration,
                Capability::Backup,
            ]
            .into_iter()
            .collect(),
        }
    }

    fn health(&self) -> Result<(), StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        // A store opened at a lower schema version is not ready to serve until
        // migrated.
        if inner.needs_migration {
            return Err(StoreError::Unavailable);
        }
        Ok(())
    }

    fn migrate(&self, target_version: u32) -> Result<(), StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        if target_version != CONTRACT_VERSION {
            return Err(StoreError::Backend {
                source: format!("unknown schema version {target_version}"),
            });
        }
        // Version 1 has no lower version to climb from, so bringing the store
        // to current is a no-op beyond recording the version and clearing the
        // flag. The guard is complete for when a future version adds real
        // migration steps here.
        if inner.needs_migration {
            inner
                .client()?
                .execute(
                    "INSERT INTO meta (key, value) VALUES ($1, $2) \
                     ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
                    &[
                        &schema::META_VERSION_KEY,
                        &schema::SCHEMA_VERSION.to_string(),
                    ],
                )
                .map_err(map_pg)?;
            inner.needs_migration = false;
        }
        Ok(())
    }

    fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str().to_string();
        let repo = scope.repo.as_str().to_string();

        // One repeatable-read transaction spans every read below, so a commit
        // landing on another connection cannot slip between them and hand back
        // a view where the revision, the documents and the history disagree.
        // Reads take no lock; the transaction is read-only, which says so.
        //
        // The rows are materialized before the transaction ends rather than
        // read lazily through it: this store has one connection, so a snapshot
        // that held its transaction open for its own lifetime would pin that
        // connection and deadlock the next commit — and the contract expects a
        // snapshot to stay readable while writes land.
        let mut tx = inner
            .client()?
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(map_pg)?;

        let revision = Revision(read_meta_u64(&mut tx, &project_rev_key(&project))?);

        let mut docs: BTreeMap<String, (String, Revision)> = BTreeMap::new();
        for row in tx
            .query(
                "SELECT doc_id, content, revision FROM documents \
                 WHERE project = $1 AND repo = $2",
                &[&project, &repo],
            )
            .map_err(map_pg)?
        {
            let doc_id: String = row.get(0);
            let content: String = row.get(1);
            let doc_revision: i64 = row.get(2);
            docs.insert(doc_id, (content, Revision(doc_revision as u64)));
        }

        let mut history: BTreeMap<String, Vec<RevisionRecord>> = BTreeMap::new();
        for row in tx
            .query(
                "SELECT doc_id, revision, actor, at, command, kind, digest FROM history \
                 WHERE project = $1 AND repo = $2 ORDER BY id",
                &[&project, &repo],
            )
            .map_err(map_pg)?
        {
            let doc_id: String = row.get(0);
            let record_revision: i64 = row.get(1);
            let at: String = row.get(3);
            let kind: String = row.get(5);
            let digest: Option<String> = row.get(6);
            history.entry(doc_id).or_default().push(RevisionRecord {
                revision: Revision(record_revision as u64),
                actor: row.get(2),
                at: parse_ts(&at)?,
                command: row.get(4),
                kind: revision_kind(&kind, digest)?,
            });
        }

        tx.commit().map_err(map_pg)?;

        Ok(Box::new(PostgresSnapshot {
            revision,
            docs,
            history,
        }))
    }

    fn begin_unit_of_work(
        &self,
        scope: &Scope,
        ctx: CommandContext,
    ) -> Result<UnitOfWork, StoreError> {
        self.lock().unavailable_if_crashed()?;
        Ok(UnitOfWork::new(scope.clone(), ctx))
    }

    fn commit(&self, uow: UnitOfWork, events: Vec<EventRecord>) -> Result<Revision, StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let pending_crash = inner.pending_crash.take();
        let fail_outbox = std::mem::replace(&mut inner.pending_outbox_failure, false);

        // The client is taken out for the duration so a crash can discard it
        // outright rather than hand it back.
        inner.client()?;
        let mut client = inner.client.take().expect("client() left one connected");
        let mut crashed = false;
        let result = commit_txn(
            &mut client,
            &uow,
            &events,
            pending_crash,
            fail_outbox,
            &mut crashed,
        );

        if crashed {
            // The crash model is the process dying with its connection:
            // dropping the client without COMMIT leaves the server to abort
            // the transaction, and this store serves nothing until it is
            // rebuilt from durable state.
            inner.crashed = true;
            drop(client);
        } else if !client.is_closed() {
            inner.client = Some(client);
        }
        result
    }

    fn rollback(&self, uow: UnitOfWork) -> Result<(), StoreError> {
        // A unit of work stages nothing durable until commit, so discarding it
        // is just dropping it.
        drop(uow);
        Ok(())
    }

    fn export(&self, scope: &Scope) -> Result<Bundle, StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str().to_string();
        let repo = scope.repo.as_str().to_string();
        let client = inner.client()?;

        let project_revision = Revision(read_meta_u64(client, &project_rev_key(&project))?);

        let mut documents = Vec::new();
        for row in client
            .query(
                "SELECT doc_id, content FROM documents \
                 WHERE project = $1 AND repo = $2 ORDER BY doc_id COLLATE \"C\"",
                &[&project, &repo],
            )
            .map_err(map_pg)?
        {
            let doc_id: String = row.get(0);
            let content: String = row.get(1);
            // Digest the content rather than trusting the stored column: the
            // bundle's digest is a statement about what it carries.
            let digest = content_digest(&content);
            documents.push(BundleDoc {
                doc: decode_doc(&doc_id)?,
                content,
                digest,
            });
        }

        Ok(Bundle {
            format_version: BUNDLE_FORMAT_VERSION,
            scope: scope.clone(),
            project_revision,
            documents,
        })
    }

    fn import(&self, bundle: Bundle, mode: ImportMode) -> Result<ImportReport, StoreError> {
        // Verify everything before applying anything: a rejected bundle leaves
        // the store untouched.
        if bundle.format_version != BUNDLE_FORMAT_VERSION {
            return Err(StoreError::Backend {
                source: format!(
                    "unsupported bundle format version {} (supported: {})",
                    bundle.format_version, BUNDLE_FORMAT_VERSION
                ),
            });
        }
        for doc in &bundle.documents {
            if content_digest(&doc.content) != doc.digest {
                return Err(StoreError::Corrupt {
                    reason: format!("bundle digest mismatch for {:?}", doc.doc),
                });
            }
        }

        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = bundle.scope.project.as_str().to_string();
        let repo = bundle.scope.repo.as_str().to_string();
        let client = inner.client()?;

        let mut tx = client.transaction().map_err(map_pg)?;
        // An import is a write to the scope, so it queues behind the scope's
        // writers like any other.
        tx.execute(
            "SELECT pg_advisory_xact_lock($1)",
            &[&PostgresTeamStore::advisory_lock_key(&bundle.scope)],
        )
        .map_err(map_pg)?;

        // Create-new means the target scope holds *nothing*. Asking only
        // whether the bundle's own documents are absent would wave through an
        // import into a scope holding unrelated ones, interleaving two stores'
        // histories under one revision counter.
        if mode == ImportMode::CreateNew && scope_holds_any_document(&mut tx, &project, &repo)? {
            return Err(StoreError::Backend {
                source: "import (create-new): target scope already holds documents".into(),
            });
        }

        // Pre-read existing revisions to classify each document.
        let mut existing = Vec::with_capacity(bundle.documents.len());
        for doc in &bundle.documents {
            existing.push(current_revision(
                &mut tx,
                &project,
                &repo,
                &encode_doc(&doc.doc),
            )?);
        }

        // Apply as one commit: history for every imported document starts (or
        // continues) at this import revision.
        let next = Revision(read_meta_u64(&mut tx, &project_rev_key(&project))? + 1);
        let now = Utc::now().to_rfc3339();
        let mut documents = Vec::new();
        for (doc, found) in bundle.documents.iter().zip(&existing) {
            let doc_id = encode_doc(&doc.doc);
            tx.execute(
                "INSERT INTO documents (project, repo, doc_id, content, revision, digest) \
                 VALUES ($1, $2, $3, $4, $5, $6) \
                 ON CONFLICT (project, repo, doc_id) DO UPDATE SET \
                 content = EXCLUDED.content, revision = EXCLUDED.revision, \
                 digest = EXCLUDED.digest",
                &[
                    &project,
                    &repo,
                    &doc_id,
                    &doc.content,
                    &(next.0 as i64),
                    &doc.digest,
                ],
            )
            .map_err(map_pg)?;
            tx.execute(
                "INSERT INTO history \
                 (project, repo, doc_id, revision, actor, at, command, kind, digest) \
                 VALUES ($1, $2, $3, $4, 'import', $5, 'import', 'write', $6)",
                &[&project, &repo, &doc_id, &(next.0 as i64), &now, &doc.digest],
            )
            .map_err(map_pg)?;
            documents.push(ImportedDoc {
                doc: doc.doc.clone(),
                outcome: match found {
                    None => ImportOutcome::Created,
                    Some(_) => ImportOutcome::Overwritten,
                },
            });
        }
        write_project_revision(&mut tx, &project, next)?;
        tx.commit().map_err(map_pg)?;

        Ok(ImportReport {
            project_revision: next,
            documents,
        })
    }

    fn read_outbox(&self, scope: &Scope, from: OutboxCursor) -> Result<Vec<OutboxEntry>, StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str().to_string();
        let repo = scope.repo.as_str().to_string();
        let client = inner.client()?;

        let mut entries = Vec::new();
        for row in client
            .query(
                "SELECT seq, revision, name, payload, actor, at FROM outbox \
                 WHERE project = $1 AND repo = $2 AND seq > $3 ORDER BY seq",
                &[&project, &repo, &(from.0 as i64)],
            )
            .map_err(map_pg)?
        {
            let payload: String = row.get(3);
            let at: String = row.get(5);
            entries.push(OutboxEntry {
                seq: row.get::<_, i64>(0) as u64,
                revision: Revision(row.get::<_, i64>(1) as u64),
                record: EventRecord {
                    name: row.get(2),
                    payload: serde_json::from_str(&payload).map_err(|e| StoreError::Corrupt {
                        reason: format!("outbox payload is not valid json: {e}"),
                    })?,
                    actor: row.get(4),
                    at: parse_ts(&at)?,
                },
            });
        }
        Ok(entries)
    }

    fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str().to_string();
        let repo = scope.repo.as_str().to_string();
        let key = acked_key(scope);
        let client = inner.client()?;

        let newest = client
            .query_one(
                "SELECT COALESCE(MAX(seq), 0) FROM outbox WHERE project = $1 AND repo = $2",
                &[&project, &repo],
            )
            .map_err(map_pg)?
            .get::<_, i64>(0) as u64;
        // Acknowledging past the newest entry would silently skip everything
        // committed later — reject it.
        if up_to.0 > newest {
            return Err(StoreError::Backend {
                source: format!("ack cursor {} is beyond the outbox end {newest}", up_to.0),
            });
        }
        // The durable position is monotonic: acknowledging backwards is a no-op.
        let current = read_meta_u64(client, &key)?;
        if up_to.0 > current {
            client
                .execute(
                    "INSERT INTO meta (key, value) VALUES ($1, $2) \
                     ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
                    &[&key, &up_to.0.to_string()],
                )
                .map_err(map_pg)?;
        }
        Ok(())
    }

    fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let key = acked_key(scope);
        Ok(OutboxCursor(read_meta_u64(inner.client()?, &key)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store::{ProjectId, RepoId};

    /// The password variable is process-global, so every assertion about it
    /// lives in one test rather than racing the others across threads.
    #[test]
    fn the_password_variable_fills_in_only_what_the_url_leaves_out() {
        std::env::remove_var(PASSWORD_VAR);
        let bare = resolve_config("postgres://user@localhost/db").expect("parse");
        assert_eq!(bare.get_password(), None);

        std::env::set_var(PASSWORD_VAR, "from-the-environment");
        let completed = resolve_config("postgres://user@localhost/db").expect("parse");
        assert_eq!(
            completed.get_password(),
            Some("from-the-environment".as_bytes())
        );

        // A password already in the URL wins.
        let embedded = resolve_config("postgres://user:in-the-url@localhost/db").expect("parse");
        assert_eq!(embedded.get_password(), Some("in-the-url".as_bytes()));

        // An exported-but-empty variable is not a password.
        std::env::set_var(PASSWORD_VAR, "");
        let still_bare = resolve_config("postgres://user@localhost/db").expect("parse");
        assert_eq!(
            still_bare.get_password(),
            None,
            "an empty variable was taken for a blank password"
        );

        std::env::remove_var(PASSWORD_VAR);
    }

    #[test]
    fn an_unparseable_url_is_a_backend_error_naming_the_problem() {
        match resolve_config("nonsense without an equals sign") {
            Err(StoreError::Backend { source }) => assert!(
                source.contains("url"),
                "the reason should say what it could not parse: {source}"
            ),
            other => panic!("expected backend, got {other:?}"),
        }
    }

    #[test]
    fn embedded_passwords_are_visible_to_startup() {
        assert!(!url_embeds_password("postgres://user@localhost/db").expect("parse"));
        assert!(url_embeds_password("postgres://user:secret@localhost/db").expect("parse"));
    }

    /// Two scopes must not share a lock, and the same scope must derive the
    /// same key in every process — a hash that varied per build would leave two
    /// servers unserialized.
    #[test]
    fn advisory_lock_keys_are_per_scope_and_stable() {
        let main = Scope::new(ProjectId::new("acme"), RepoId::new("main"));
        let docs = Scope::new(ProjectId::new("acme"), RepoId::new("docs"));
        assert_ne!(
            PostgresTeamStore::advisory_lock_key(&main),
            PostgresTeamStore::advisory_lock_key(&docs)
        );
        assert_eq!(
            PostgresTeamStore::advisory_lock_key(&main),
            PostgresTeamStore::advisory_lock_key(&Scope::new(
                ProjectId::new("acme"),
                RepoId::new("main")
            ))
        );
        // The separator is what keeps the halves apart: without it
        // ("ac" + "me/main") and ("acme" + "/main") would collide.
        let split_differently = Scope::new(ProjectId::new("acmemain"), RepoId::new(""));
        assert_ne!(
            PostgresTeamStore::advisory_lock_key(&main),
            PostgresTeamStore::advisory_lock_key(&split_differently)
        );
    }
}
