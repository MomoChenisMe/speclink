//! SQLite reference driver for the TeamStore contract.
//!
//! One SQLite database file persists an entire store: documents, immutable
//! history, the transactional outbox, and the schema/version marker. The
//! driver is single-node — a single write connection serializes commits, WAL
//! journaling makes reads concurrent, and every commit is one SQL
//! transaction so partial writes never survive a crash.
//!
//! Depends only on `speclink-store` (the contract and its conformance suite)
//! and `rusqlite`; it does not touch `speclink-core` or any other crate.

mod schema;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};

use speclink_store::{
    content_digest, Bundle, BundleDoc, Capability, CapabilityLevel, DocRef, Document, DocumentId,
    EventRecord, ExpectedRevision, FaultPoint, ImportMode, ImportOutcome, ImportReport, ImportedDoc,
    Manifest, OutboxCursor, OutboxEntry, Revision, RevisionKind, RevisionRecord, Scope, Snapshot,
    StagedOp, StoreError, TeamStore, UnitOfWork, BUNDLE_FORMAT_VERSION, CONTRACT_VERSION,
};
use speclink_store::CommandContext;

/// Unit separator, used to join the fields of an encoded document id and the
/// components of a `meta` key. No logical identifier contains it.
const SEP: char = '\u{1f}';

// --- error mapping --------------------------------------------------------

/// Map a rusqlite failure onto the closed store error set. A busy/locked
/// database is transient (`unavailable`); everything else is a backend
/// failure carrying the source description.
fn map_sqlite(err: rusqlite::Error) -> StoreError {
    if let rusqlite::Error::SqliteFailure(ffi, _) = &err {
        if matches!(ffi.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) {
            return StoreError::Unavailable;
        }
    }
    StoreError::Backend {
        source: err.to_string(),
    }
}

// --- document id and meta-key encoding ------------------------------------

/// Encode a document id as a stable string key: a short tag followed by the
/// identifying fields, separated by [`SEP`]. Deterministic and reversible by
/// [`decode_doc`].
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
fn read_meta_u64(conn: &Connection, key: &str) -> Result<u64, StoreError> {
    let value: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
        .optional()
        .map_err(map_sqlite)?;
    match value {
        None => Ok(0),
        Some(text) => text.parse().map_err(|_| StoreError::Corrupt {
            reason: format!("meta value for {key:?} is not a number: {text:?}"),
        }),
    }
}

/// Whether a table of the given name exists.
fn table_exists(conn: &Connection, name: &str) -> Result<bool, StoreError> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |_| Ok(()),
        )
        .optional()
        .map_err(map_sqlite)?
        .is_some())
}

/// Whether the database holds any user table (i.e. is non-empty and not one
/// of SQLite's internal tables).
fn has_any_user_table(conn: &Connection) -> Result<bool, StoreError> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' \
             AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' LIMIT 1",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(map_sqlite)?
        .is_some())
}

/// Create the schema and seed the meta marker/version in one transaction.
fn initialize_schema(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(&format!(
        "BEGIN;\n{sql}\n\
         INSERT INTO meta (key, value) VALUES ('{fmt_key}', '{marker}');\n\
         INSERT INTO meta (key, value) VALUES ('{ver_key}', '{version}');\n\
         COMMIT;",
        sql = schema::SCHEMA_SQL,
        fmt_key = schema::META_FORMAT_KEY,
        marker = schema::STORE_MARKER,
        ver_key = schema::META_VERSION_KEY,
        version = schema::SCHEMA_VERSION,
    ))
    .map_err(map_sqlite)
}

// --- the store ------------------------------------------------------------

/// State behind the store's single mutex: the write connection plus the
/// test-only fault-injection flags. Serializing here is the single-node
/// write guarantee the contract asks for.
struct Inner {
    conn: Connection,
    /// Crash the next commit at this stage boundary (test hook).
    pending_crash: Option<FaultPoint>,
    /// Make the next commit's outbox append fail (test hook).
    pending_outbox_failure: bool,
    /// A crashed store serves nothing until reopened from durable state.
    crashed: bool,
    /// The database records a version below the current one.
    needs_migration: bool,
}

impl Inner {
    fn unavailable_if_crashed(&self) -> Result<(), StoreError> {
        if self.crashed {
            Err(StoreError::Unavailable)
        } else {
            Ok(())
        }
    }

    /// Apply one unit of work in a single SQL transaction: validate every CAS
    /// precondition, write documents, append history, append the outbox, and
    /// advance the project revision — commit only at the very end, so nothing
    /// survives a failure partway through.
    ///
    /// The test-only fault points model a process crash: at the armed
    /// boundary we return early without committing (the transaction rolls
    /// back on drop) and set `*crashed`, so the whole commit is invisible on
    /// the next open. An armed outbox failure is an ordinary error the commit
    /// absorbs — same rollback, but the store stays usable.
    fn commit_txn(
        &mut self,
        uow: &UnitOfWork,
        events: &[EventRecord],
        pending_crash: Option<FaultPoint>,
        fail_outbox: bool,
        crashed: &mut bool,
    ) -> Result<Revision, StoreError> {
        let scope = uow.scope();
        let project = scope.project.as_str().to_string();
        let repo = scope.repo.as_str().to_string();

        let tx = self.conn.transaction().map_err(map_sqlite)?;
        let next = Revision(read_meta_u64(&tx, &project_rev_key(&project))? + 1);

        // 1. Validate every precondition against the pre-commit state before
        //    touching anything: any mismatch rejects the whole commit.
        for op in uow.ops() {
            let doc_id = encode_doc(op.doc());
            let current: Option<u64> = tx
                .query_row(
                    "SELECT revision FROM documents WHERE project = ?1 AND repo = ?2 AND doc_id = ?3",
                    params![project, repo, doc_id],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(map_sqlite)?
                .map(|v| v as u64);
            let expected = match op {
                StagedOp::Put { expected, .. } => *expected,
                StagedOp::Delete { expected, .. } => ExpectedRevision::At(*expected),
            };
            let satisfied = match expected {
                ExpectedRevision::Absent => current.is_none(),
                ExpectedRevision::At(rev) => current == Some(rev.0),
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
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                         ON CONFLICT(project, repo, doc_id) DO UPDATE SET \
                         content = excluded.content, revision = excluded.revision, \
                         digest = excluded.digest",
                        params![project, repo, doc_id, content, next.0 as i64, content_digest(content)],
                    )
                    .map_err(map_sqlite)?;
                }
                StagedOp::Delete { .. } => {
                    tx.execute(
                        "DELETE FROM documents WHERE project = ?1 AND repo = ?2 AND doc_id = ?3",
                        params![project, repo, doc_id],
                    )
                    .map_err(map_sqlite)?;
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
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    project,
                    repo,
                    doc_id,
                    next.0 as i64,
                    uow.context().actor,
                    now,
                    uow.context().command,
                    kind,
                    digest,
                ],
            )
            .map_err(map_sqlite)?;
        }
        if pending_crash == Some(FaultPoint::AfterHistoryAppend) {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }
        if pending_crash == Some(FaultPoint::BeforeOutboxAppend) {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }

        // An armed outbox failure aborts the commit without crashing.
        if fail_outbox {
            return Err(StoreError::Backend {
                source: "outbox append failed".into(),
            });
        }

        // 4. Outbox append, at monotonic per-scope sequence numbers.
        let mut seq = tx
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM outbox WHERE project = ?1 AND repo = ?2",
                params![project, repo],
                |r| r.get::<_, i64>(0),
            )
            .map_err(map_sqlite)? as u64;
        for event in events {
            seq += 1;
            tx.execute(
                "INSERT INTO outbox \
                 (project, repo, seq, revision, name, payload, actor, at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    project,
                    repo,
                    seq as i64,
                    next.0 as i64,
                    event.name,
                    event.payload.to_string(),
                    event.actor,
                    event.at.to_rfc3339(),
                ],
            )
            .map_err(map_sqlite)?;
        }
        if pending_crash == Some(FaultPoint::AfterOutboxAppend) {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }

        // 5. Advance the project revision, then the atomic commit.
        tx.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![project_rev_key(&project), next.0.to_string()],
        )
        .map_err(map_sqlite)?;
        tx.commit().map_err(map_sqlite)?;
        Ok(next)
    }
}

/// A SQLite-backed [`TeamStore`]. Construct with [`SqliteTeamStore::open`].
pub struct SqliteTeamStore {
    inner: Mutex<Inner>,
}

impl SqliteTeamStore {
    /// Open (or initialize) a store at `path`.
    ///
    /// Fails closed on anything it does not recognize. State detection is
    /// read-only and happens **before** any write pragma, so a refused open
    /// leaves the target file's bytes untouched:
    ///
    /// - a database recording a higher schema version → [`StoreError::Corrupt`];
    /// - an existing SQLite file that is not a speclink store → `Corrupt`;
    /// - an empty (or absent) file → schema initialized at the current version;
    /// - a valid store at a lower version → opens, flagged as needing migration.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let conn = Connection::open(path.as_ref()).map_err(map_sqlite)?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(map_sqlite)?;

        // Read-only detection first — no write reaches a database we go on to
        // refuse.
        let has_meta = table_exists(&conn, "meta")?;
        let mut needs_migration = false;
        if has_meta {
            let marker: Option<String> = conn
                .query_row(
                    "SELECT value FROM meta WHERE key = ?1",
                    [schema::META_FORMAT_KEY],
                    |r| r.get(0),
                )
                .optional()
                .map_err(map_sqlite)?;
            if marker.as_deref() != Some(schema::STORE_MARKER) {
                return Err(StoreError::Corrupt {
                    reason: "database has a meta table but not the speclink store marker".into(),
                });
            }
            let version_text: Option<String> = conn
                .query_row(
                    "SELECT value FROM meta WHERE key = ?1",
                    [schema::META_VERSION_KEY],
                    |r| r.get(0),
                )
                .optional()
                .map_err(map_sqlite)?;
            let version: u32 = match version_text.as_deref().map(str::parse::<u32>) {
                Some(Ok(v)) => v,
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
        } else if has_any_user_table(&conn)? {
            return Err(StoreError::Corrupt {
                reason: "existing SQLite database is not a speclink team store".into(),
            });
        }

        // Past the gate: a valid store or a fresh, empty file. Only now do we
        // touch the file — enable WAL, and initialize the schema if new.
        let _mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
            .map_err(map_sqlite)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(map_sqlite)?;
        if !has_meta {
            initialize_schema(&conn)?;
        }

        Ok(Self {
            inner: Mutex::new(Inner {
                conn,
                pending_crash: None,
                pending_outbox_failure: false,
                crashed: false,
                needs_migration,
            }),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("sqlite store mutex poisoned")
    }

    /// Test-only fault hook: crash the next commit at `point`. Not part of
    /// the stable contract surface (hidden from docs); it exists so the
    /// conformance harness can drive crash-recovery. See design decision 4.
    #[doc(hidden)]
    pub fn crash_at(&self, point: FaultPoint) {
        self.lock().pending_crash = Some(point);
    }

    /// Test-only fault hook: make the next commit's outbox append fail (an
    /// error the commit must absorb, not a crash). Hidden from docs.
    #[doc(hidden)]
    pub fn fail_outbox_append(&self) {
        self.lock().pending_outbox_failure = true;
    }
}

/// A fixed-point view: owns a copy of the scope's documents and history taken
/// at snapshot time, so commits that land afterwards cannot reach into it.
/// Keyed by encoded document id (see [`encode_doc`]).
struct SqliteSnapshot {
    revision: Revision,
    docs: BTreeMap<String, (String, Revision)>,
    history: BTreeMap<String, Vec<RevisionRecord>>,
}

impl Snapshot for SqliteSnapshot {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn read(&self, doc: &DocumentId) -> Result<Option<Document>, StoreError> {
        Ok(self.docs.get(&encode_doc(doc)).map(|(content, revision)| Document {
            content: content.clone(),
            revision: *revision,
        }))
    }

    fn history(&self, doc: &DocumentId) -> Result<Vec<RevisionRecord>, StoreError> {
        Ok(self.history.get(&encode_doc(doc)).cloned().unwrap_or_default())
    }
}

impl TeamStore for SqliteTeamStore {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: CONTRACT_VERSION,
            driver: "sqlite".into(),
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
        // A store opened at a lower schema version is not ready to serve
        // until migrated.
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
        // to current is a no-op beyond clearing the flag and recording the
        // version. The guard is complete for when a future version adds real
        // migration steps here.
        if inner.needs_migration {
            inner
                .conn
                .execute(
                    "INSERT INTO meta (key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![schema::META_VERSION_KEY, schema::SCHEMA_VERSION.to_string()],
                )
                .map_err(map_sqlite)?;
            inner.needs_migration = false;
        }
        Ok(())
    }

    fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str();
        let repo = scope.repo.as_str();

        // Materialize the scope's documents and history into an owned copy, so
        // this fixed-point view is unaffected by commits that land later.
        let revision = Revision(read_meta_u64(&inner.conn, &project_rev_key(project))?);

        let mut docs: BTreeMap<String, (String, Revision)> = BTreeMap::new();
        {
            let mut stmt = inner
                .conn
                .prepare(
                    "SELECT doc_id, content, revision FROM documents \
                     WHERE project = ?1 AND repo = ?2",
                )
                .map_err(map_sqlite)?;
            let rows = stmt
                .query_map(params![project, repo], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                })
                .map_err(map_sqlite)?;
            for row in rows {
                let (doc_id, content, rev) = row.map_err(map_sqlite)?;
                docs.insert(doc_id, (content, Revision(rev as u64)));
            }
        }

        let mut history: BTreeMap<String, Vec<RevisionRecord>> = BTreeMap::new();
        {
            let mut stmt = inner
                .conn
                .prepare(
                    "SELECT doc_id, revision, actor, at, command, kind, digest FROM history \
                     WHERE project = ?1 AND repo = ?2 ORDER BY id",
                )
                .map_err(map_sqlite)?;
            let rows = stmt
                .query_map(params![project, repo], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, Option<String>>(6)?,
                    ))
                })
                .map_err(map_sqlite)?;
            for row in rows {
                let (doc_id, rev, actor, at, command, kind, digest) = row.map_err(map_sqlite)?;
                history.entry(doc_id).or_default().push(RevisionRecord {
                    revision: Revision(rev as u64),
                    actor,
                    at: parse_ts(&at)?,
                    command,
                    kind: revision_kind(&kind, digest)?,
                });
            }
        }

        Ok(Box::new(SqliteSnapshot {
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
        let mut crashed = false;
        let result = inner.commit_txn(&uow, &events, pending_crash, fail_outbox, &mut crashed);
        if crashed {
            inner.crashed = true;
        }
        result
    }

    fn rollback(&self, uow: UnitOfWork) -> Result<(), StoreError> {
        // A unit of work stages nothing durable until commit, so discarding
        // it is just dropping it.
        drop(uow);
        Ok(())
    }

    fn export(&self, scope: &Scope) -> Result<Bundle, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str();
        let repo = scope.repo.as_str();
        let project_revision = Revision(read_meta_u64(&inner.conn, &project_rev_key(project))?);

        let mut documents = Vec::new();
        let mut stmt = inner
            .conn
            .prepare(
                "SELECT doc_id, content FROM documents \
                 WHERE project = ?1 AND repo = ?2 ORDER BY doc_id",
            )
            .map_err(map_sqlite)?;
        let rows = stmt
            .query_map(params![project, repo], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(map_sqlite)?;
        for row in rows {
            let (doc_id, content) = row.map_err(map_sqlite)?;
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

        let tx = inner.conn.transaction().map_err(map_sqlite)?;

        // Pre-read existing revisions to classify each document.
        let mut existing: Vec<Option<u64>> = Vec::with_capacity(bundle.documents.len());
        for doc in &bundle.documents {
            let doc_id = encode_doc(&doc.doc);
            let current: Option<u64> = tx
                .query_row(
                    "SELECT revision FROM documents WHERE project = ?1 AND repo = ?2 AND doc_id = ?3",
                    params![project, repo, doc_id],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(map_sqlite)?
                .map(|v| v as u64);
            existing.push(current);
        }
        // Create-new is gated on the whole scope, not on the bundle's own
        // documents: anything already there rejects the import.
        let scope_holds_any_document: bool = tx
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM documents WHERE project = ?1 AND repo = ?2)",
                params![project, repo],
                |r| r.get(0),
            )
            .map_err(map_sqlite)?;
        if mode == ImportMode::CreateNew && scope_holds_any_document {
            return Err(StoreError::Backend {
                source: "import (create-new): target scope already holds documents".into(),
            });
        }

        // Apply as one commit: history for every imported document starts (or
        // continues) at this import revision.
        let next = Revision(read_meta_u64(&tx, &project_rev_key(&project))? + 1);
        let now = Utc::now().to_rfc3339();
        let mut documents = Vec::new();
        for (doc, found) in bundle.documents.iter().zip(&existing) {
            let doc_id = encode_doc(&doc.doc);
            tx.execute(
                "INSERT INTO documents (project, repo, doc_id, content, revision, digest) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                 ON CONFLICT(project, repo, doc_id) DO UPDATE SET \
                 content = excluded.content, revision = excluded.revision, \
                 digest = excluded.digest",
                params![project, repo, doc_id, doc.content, next.0 as i64, doc.digest],
            )
            .map_err(map_sqlite)?;
            tx.execute(
                "INSERT INTO history \
                 (project, repo, doc_id, revision, actor, at, command, kind, digest) \
                 VALUES (?1, ?2, ?3, ?4, 'import', ?5, 'import', 'write', ?6)",
                params![project, repo, doc_id, next.0 as i64, now, doc.digest],
            )
            .map_err(map_sqlite)?;
            documents.push(ImportedDoc {
                doc: doc.doc.clone(),
                outcome: match found {
                    None => ImportOutcome::Created,
                    Some(_) => ImportOutcome::Overwritten,
                },
            });
        }
        tx.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![project_rev_key(&project), next.0.to_string()],
        )
        .map_err(map_sqlite)?;
        tx.commit().map_err(map_sqlite)?;

        Ok(ImportReport {
            project_revision: next,
            documents,
        })
    }

    fn read_outbox(
        &self,
        scope: &Scope,
        from: OutboxCursor,
    ) -> Result<Vec<OutboxEntry>, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str();
        let repo = scope.repo.as_str();
        let mut stmt = inner
            .conn
            .prepare(
                "SELECT seq, revision, name, payload, actor, at FROM outbox \
                 WHERE project = ?1 AND repo = ?2 AND seq > ?3 ORDER BY seq",
            )
            .map_err(map_sqlite)?;
        let rows = stmt
            .query_map(params![project, repo, from.0 as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(map_sqlite)?;
        let mut entries = Vec::new();
        for row in rows {
            let (seq, rev, name, payload, actor, at) = row.map_err(map_sqlite)?;
            let payload = serde_json::from_str(&payload).map_err(|e| StoreError::Corrupt {
                reason: format!("outbox payload is not valid json: {e}"),
            })?;
            entries.push(OutboxEntry {
                seq: seq as u64,
                revision: Revision(rev as u64),
                record: EventRecord {
                    name,
                    payload,
                    actor,
                    at: parse_ts(&at)?,
                },
            });
        }
        Ok(entries)
    }

    fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let project = scope.project.as_str();
        let repo = scope.repo.as_str();
        let newest = inner
            .conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM outbox WHERE project = ?1 AND repo = ?2",
                params![project, repo],
                |r| r.get::<_, i64>(0),
            )
            .map_err(map_sqlite)? as u64;
        // Acknowledging past the newest entry would silently skip everything
        // committed later — reject it.
        if up_to.0 > newest {
            return Err(StoreError::Backend {
                source: format!("ack cursor {} is beyond the outbox end {newest}", up_to.0),
            });
        }
        // The durable position is monotonic: acknowledging backwards is a no-op.
        let current = read_meta_u64(&inner.conn, &acked_key(scope))?;
        if up_to.0 > current {
            inner
                .conn
                .execute(
                    "INSERT INTO meta (key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![acked_key(scope), up_to.0.to_string()],
                )
                .map_err(map_sqlite)?;
        }
        Ok(())
    }

    fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        Ok(OutboxCursor(read_meta_u64(&inner.conn, &acked_key(scope))?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use speclink_store::{ProjectId, RepoId};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn tmp() -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        (dir, path)
    }

    fn ctx(command: &str) -> CommandContext {
        CommandContext {
            command: command.into(),
            actor: "tester".into(),
        }
    }

    fn scope(repo: &str) -> Scope {
        Scope::new(ProjectId::new("acme"), RepoId::new(repo))
    }

    fn spec(capability: &str) -> DocumentId {
        DocumentId::CanonicalSpec {
            capability: capability.into(),
        }
    }

    fn event(name: &str) -> EventRecord {
        EventRecord {
            name: name.into(),
            payload: serde_json::json!({ "event": name }),
            actor: "tester".into(),
            at: Utc::now(),
        }
    }

    // --- task 1.2: schema version gate (fail closed) ---

    #[test]
    fn empty_database_initializes_at_version_1() {
        let (_dir, path) = tmp();
        SqliteTeamStore::open(&path).expect("fresh open initializes");

        // The four tables exist and the marker/version are seeded.
        let conn = Connection::open(&path).unwrap();
        for table in ["meta", "documents", "history", "outbox"] {
            assert!(table_exists(&conn, table).unwrap(), "table {table} missing");
        }
        let version: String = conn
            .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, "1");
        let marker: String = conn
            .query_row("SELECT value FROM meta WHERE key = 'format'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(marker, schema::STORE_MARKER);

        // Reopening an already-initialized store is fine.
        SqliteTeamStore::open(&path).expect("reopen succeeds");
    }

    #[test]
    fn database_with_newer_version_is_refused_and_bytes_unchanged() {
        let (_dir, path) = tmp();
        SqliteTeamStore::open(&path).unwrap();

        // Forge a database that records a schema version one above current,
        // then checkpoint so the main file carries the committed state.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute(
                "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                [(schema::SCHEMA_VERSION + 1).to_string()],
            )
            .unwrap();
            conn.pragma_update(None, "wal_checkpoint", "TRUNCATE").unwrap();
        }

        let before = std::fs::read(&path).unwrap();
        match SqliteTeamStore::open(&path) {
            Err(StoreError::Corrupt { reason }) => {
                assert!(
                    reason.contains("version"),
                    "reason should name the version incompatibility: {reason}"
                );
            }
            Err(other) => panic!("expected corrupt refusal, got {other:?}"),
            Ok(_) => panic!("expected corrupt refusal, got Ok"),
        }
        let after = std::fs::read(&path).unwrap();
        assert_eq!(before, after, "refused open must not write to the file");
    }

    #[test]
    fn foreign_sqlite_database_is_refused_and_not_initialized() {
        let (_dir, path) = tmp();

        // A perfectly valid SQLite database built by some other application.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE foo (x INTEGER); INSERT INTO foo VALUES (1);")
                .unwrap();
        }

        let before = std::fs::read(&path).unwrap();
        match SqliteTeamStore::open(&path) {
            Err(StoreError::Corrupt { .. }) => {}
            Err(other) => panic!("expected corrupt refusal, got {other:?}"),
            Ok(_) => panic!("expected corrupt refusal, got Ok"),
        }
        let after = std::fs::read(&path).unwrap();
        assert_eq!(before, after, "refused open must not write to the file");

        // The driver must not have grafted its schema onto the foreign file.
        let conn = Connection::open(&path).unwrap();
        assert!(!table_exists(&conn, "meta").unwrap());
        assert!(!table_exists(&conn, "documents").unwrap());
    }

    // --- task 2.1: CAS atomicity ---

    #[test]
    fn concurrent_commits_on_same_revision_conflict_and_loser_leaves_no_trace() {
        let (_dir, path) = tmp();
        let store = SqliteTeamStore::open(&path).unwrap();
        let scope = scope("web");
        let auth = spec("auth");
        let billing = spec("billing");

        let mut uow = store.begin_unit_of_work(&scope, ctx("seed")).unwrap();
        uow.create(auth.clone(), "base");
        let seeded_at = store.commit(uow, vec![]).unwrap();

        // Two writers read the same revision; the loser also stages a second,
        // independent op that must not survive its failed commit.
        let mut winner = store.begin_unit_of_work(&scope, ctx("win")).unwrap();
        winner.update(auth.clone(), "winner", seeded_at);
        let mut loser = store.begin_unit_of_work(&scope, ctx("lose")).unwrap();
        loser.update(auth.clone(), "loser", seeded_at);
        loser.create(billing.clone(), "loser-only");

        let winning_rev = store.commit(winner, vec![]).unwrap();
        match store.commit(loser, vec![]) {
            Err(StoreError::RevisionConflict {
                doc,
                expected,
                actual,
            }) => {
                assert_eq!(doc.doc, auth);
                assert_eq!(expected, ExpectedRevision::At(seeded_at));
                assert_eq!(actual, Some(winning_rev));
            }
            other => panic!("expected revision conflict, got {other:?}"),
        }

        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(snap.read(&auth).unwrap().unwrap().content, "winner");
        assert_eq!(
            snap.read(&billing).unwrap(),
            None,
            "the loser's other op must not have landed"
        );
        assert_eq!(snap.revision(), winning_rev);
    }

    // --- task 2.3: persistence and reopen consistency ---

    #[test]
    fn persists_and_reopens_consistently() {
        let (_dir, path) = tmp();
        let scope = scope("web");
        let docs = [spec("auth"), spec("billing"), spec("workflow")];

        {
            let store = SqliteTeamStore::open(&path).unwrap();
            for (i, doc) in docs.iter().enumerate() {
                let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
                uow.create(doc.clone(), format!("content-{i}"));
                store.commit(uow, vec![event(&format!("e{i}"))]).unwrap();
            }
            // Acknowledge only the first outbox event, then close.
            store.ack_outbox(&scope, OutboxCursor(1)).unwrap();
        }

        // Reopen the same path: documents, revisions and history read back
        // whole.
        let store = SqliteTeamStore::open(&path).unwrap();
        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), Revision(3));
        for (i, doc) in docs.iter().enumerate() {
            let document = snap.read(doc).unwrap().unwrap();
            assert_eq!(document.content, format!("content-{i}"));
            assert_eq!(document.revision, Revision((i + 1) as u64));
            assert_eq!(snap.history(doc).unwrap().len(), 1);
        }

        // The durable ack cursor survived; resuming from it yields only the
        // still-unacknowledged events.
        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(1));
        let pending = store
            .read_outbox(&scope, store.outbox_acked(&scope).unwrap())
            .unwrap();
        let names: Vec<&str> = pending.iter().map(|e| e.record.name.as_str()).collect();
        assert_eq!(names, vec!["e1", "e2"]);
    }

    #[test]
    fn leftover_wal_replays_on_reopen() {
        let (_dir, path) = tmp();
        let scope = scope("web");
        let store = SqliteTeamStore::open(&path).unwrap();
        let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
        uow.create(spec("auth"), "v1");
        store.commit(uow, vec![event("created")]).unwrap();

        // With the connection still open, the committed frame lives in the
        // WAL. Copy the file trio to a fresh location to mimic what a crash
        // leaves behind, then open that copy.
        let (_dir2, path2) = tmp();
        for suffix in ["", "-wal", "-shm"] {
            let from = PathBuf::from(format!("{}{suffix}", path.display()));
            if from.exists() {
                let to = PathBuf::from(format!("{}{suffix}", path2.display()));
                std::fs::copy(&from, &to).unwrap();
            }
        }
        drop(store);

        let reopened = SqliteTeamStore::open(&path2).unwrap();
        let snap = reopened.snapshot(&scope).unwrap();
        assert_eq!(snap.read(&spec("auth")).unwrap().unwrap().content, "v1");
        assert_eq!(
            reopened.read_outbox(&scope, OutboxCursor(0)).unwrap().len(),
            1,
            "the WAL-committed outbox event replayed on reopen"
        );
    }

    // --- task 3.3: tenant scope isolation (beyond the conformance read gate) ---

    #[test]
    fn tenant_scopes_isolate_documents_outbox_and_history() {
        let (_dir, path) = tmp();
        let store = SqliteTeamStore::open(&path).unwrap();
        let a = scope("repo-a");
        let b = scope("repo-b");
        let auth = spec("auth");

        // Both scopes write a document of the same name, each with its own
        // event.
        let mut ua = store.begin_unit_of_work(&a, ctx("a")).unwrap();
        ua.create(auth.clone(), "A content");
        store.commit(ua, vec![event("a-event")]).unwrap();

        let mut ub = store.begin_unit_of_work(&b, ctx("b")).unwrap();
        ub.create(auth.clone(), "B content");
        store.commit(ub, vec![event("b-event")]).unwrap();

        // Each scope reads only its own document.
        assert_eq!(
            store.snapshot(&a).unwrap().read(&auth).unwrap().unwrap().content,
            "A content"
        );
        assert_eq!(
            store.snapshot(&b).unwrap().read(&auth).unwrap().unwrap().content,
            "B content"
        );

        // The outbox does not cross scopes.
        let a_events: Vec<String> = store
            .read_outbox(&a, OutboxCursor(0))
            .unwrap()
            .into_iter()
            .map(|e| e.record.name)
            .collect();
        let b_events: Vec<String> = store
            .read_outbox(&b, OutboxCursor(0))
            .unwrap()
            .into_iter()
            .map(|e| e.record.name)
            .collect();
        assert_eq!(a_events, vec!["a-event"]);
        assert_eq!(b_events, vec!["b-event"]);

        // History does not cross scopes.
        assert_eq!(store.snapshot(&a).unwrap().history(&auth).unwrap().len(), 1);
        assert_eq!(store.snapshot(&b).unwrap().history(&auth).unwrap().len(), 1);
    }

    // --- task 3.1: export / import ---

    #[test]
    fn export_import_round_trip_and_tamper_rejection() {
        let (_dir, path) = tmp();
        let store = SqliteTeamStore::open(&path).unwrap();
        let scope = scope("web");
        let mut uow = store.begin_unit_of_work(&scope, ctx("seed")).unwrap();
        uow.create(spec("auth"), "auth v1");
        uow.create(spec("billing"), "billing v1");
        store.commit(uow, vec![]).unwrap();

        let bundle = store.export(&scope).unwrap();
        assert_eq!(bundle.format_version, BUNDLE_FORMAT_VERSION);
        assert_eq!(bundle.documents.len(), 2);
        for doc in &bundle.documents {
            assert_eq!(doc.digest, content_digest(&doc.content));
        }

        // Round-trip into a fresh store: contents match and history starts at
        // the import.
        let (_dir2, path2) = tmp();
        let fresh = SqliteTeamStore::open(&path2).unwrap();
        let report = fresh.import(bundle.clone(), ImportMode::CreateNew).unwrap();
        assert_eq!(report.documents.len(), 2);
        assert!(report
            .documents
            .iter()
            .all(|d| d.outcome == ImportOutcome::Created));
        let snap = fresh.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), report.project_revision);
        for doc in &bundle.documents {
            assert_eq!(snap.read(&doc.doc).unwrap().unwrap().content, doc.content);
            assert_eq!(snap.history(&doc.doc).unwrap().len(), 1);
        }

        // Tampering is rejected whole, leaving the target untouched.
        let mut tampered = bundle;
        tampered.documents[0].content.push_str(" tampered");
        let (_dir3, path3) = tmp();
        let target = SqliteTeamStore::open(&path3).unwrap();
        assert_eq!(
            target
                .import(tampered, ImportMode::CreateNew)
                .unwrap_err()
                .code(),
            "corrupt"
        );
        assert_eq!(target.snapshot(&scope).unwrap().revision(), Revision(0));
    }

    // --- task 3.4: driver-specific boundaries ---

    #[test]
    fn two_instances_on_the_same_file_serialize_and_share_state() {
        // Chosen behavior for two same-process instances of one file:
        // serialize through SQLite's own locking (busy timeout), sharing the
        // committed state. Pinned here so it cannot regress silently.
        let (_dir, path) = tmp();
        let a = SqliteTeamStore::open(&path).unwrap();
        let b = SqliteTeamStore::open(&path).unwrap();
        let scope = scope("web");
        let doc = spec("auth");

        let mut ua = a.begin_unit_of_work(&scope, ctx("a-create")).unwrap();
        ua.create(doc.clone(), "v1");
        let r1 = a.commit(ua, vec![]).unwrap();

        // B sees A's commit and builds on it.
        let snap_b = b.snapshot(&scope).unwrap();
        let seen = snap_b.read(&doc).unwrap().unwrap();
        assert_eq!(seen.content, "v1");
        assert_eq!(seen.revision, r1);
        let mut ub = b.begin_unit_of_work(&scope, ctx("b-update")).unwrap();
        ub.update(doc.clone(), "v2", r1);
        let r2 = b.commit(ub, vec![]).unwrap();
        assert!(r2 > r1);

        // A now sees B's commit — one serialized, shared history.
        let snap_a = a.snapshot(&scope).unwrap();
        let seen = snap_a.read(&doc).unwrap().unwrap();
        assert_eq!(seen.content, "v2");
        assert_eq!(seen.revision, r2);
    }

    #[test]
    fn unwritable_path_is_a_backend_error() {
        // A directory cannot be opened as a database file: the driver reports
        // this as a backend failure, not a panic or a silent success.
        let dir = tempfile::tempdir().unwrap();
        match SqliteTeamStore::open(dir.path()) {
            Err(StoreError::Backend { .. }) => {}
            Err(other) => panic!("expected backend error, got {other:?}"),
            Ok(_) => panic!("expected backend error opening a directory as a db"),
        }
    }
}
