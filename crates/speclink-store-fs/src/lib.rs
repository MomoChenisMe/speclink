//! Filesystem driver for the TeamStore contract.
//!
//! One data directory persists an entire store: a meta file carrying the
//! driver identity and schema version, a lock file backing the single-writer
//! advisory lock, and one subdirectory per scope holding that scope's index,
//! document contents, immutable history and transactional outbox.
//!
//! Atomicity comes from the filesystem alone. Every commit writes its new
//! content, history and outbox files first — inert while nothing references
//! them — and publishes at a single point: the atomic rename of the scope's
//! index. A crash before that rename leaves the old index untouched, so the
//! commit never happened; the files it left behind are orphans the next open
//! sweeps away. Ordering and identity come from index and file sequence
//! numbers only: no timestamp on disk carries meaning, so a filesystem with
//! coarse or tampered mtimes changes nothing.
//!
//! Depends only on `speclink-store` (the contract and its conformance suite)
//! and `fs4` for the advisory lock; it does not touch `speclink-core` or any
//! other crate. Note this is not `speclink-fs`, which is the local engine's
//! `Store` seam over an `openspec/` tree — a different layer entirely.

pub mod layout;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use speclink_store::{
    content_digest, Bundle, BundleDoc, Capability, CapabilityLevel, CommandContext, DocRef,
    Document, DocumentId, EventRecord, ExpectedRevision, FaultPoint, ImportMode, ImportOutcome,
    ImportReport, ImportedDoc, Manifest, OutboxCursor, OutboxEntry, Revision, RevisionKind,
    RevisionRecord, Scope, Snapshot, StagedOp, StoreError, TeamStore, UnitOfWork,
    BUNDLE_FORMAT_VERSION, CONTRACT_VERSION,
};

// --- error mapping --------------------------------------------------------

/// Map a filesystem failure onto the closed store error set.
///
/// A backend that has gone away — a dropped NAS mount, a timed-out network
/// filesystem — is transient: `unavailable` tells the caller to retry, and
/// the data is still whatever it was. Everything else, including a missing
/// path and a refused permission, is a `backend` failure carrying its source:
/// those need an operator, not a retry. Note `permission_denied` in the
/// contract is about the *caller's* authorization in a scope, not about the
/// filesystem's mode bits — mapping a chmod there would tell the caller a
/// lie about which layer refused them.
fn map_io(context: &str, err: io::Error) -> StoreError {
    match err.kind() {
        // A mount that went away is the case this list exists for:
        // `StaleNetworkFileHandle` (ESTALE) is what a dropped NFS export
        // hands back, and the rest are its network-layer cousins.
        io::ErrorKind::StaleNetworkFileHandle
        | io::ErrorKind::NetworkDown
        | io::ErrorKind::NetworkUnreachable
        | io::ErrorKind::HostUnreachable
        | io::ErrorKind::TimedOut
        | io::ErrorKind::Interrupted
        | io::ErrorKind::BrokenPipe
        | io::ErrorKind::NotConnected
        | io::ErrorKind::ConnectionReset
        | io::ErrorKind::ConnectionAborted => StoreError::Unavailable,
        _ => StoreError::Backend {
            source: format!("{context}: {err}"),
        },
    }
}

// --- durable writes -------------------------------------------------------

/// Write `bytes` to `path` and fsync the file, so what the caller wrote is on
/// the medium — not merely in the page cache — before anything points at it.
fn write_durable(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)
        .map_err(|e| map_io(&format!("create {}", path.display()), e))?;
    file.write_all(bytes)
        .map_err(|e| map_io(&format!("write {}", path.display()), e))?;
    file.sync_all()
        .map_err(|e| map_io(&format!("fsync {}", path.display()), e))?;
    Ok(())
}

/// Fsync a directory so a rename or creation inside it is itself durable.
/// Without this a crash can lose the *name*, even though the file's bytes
/// were synced — the publish would evaporate.
fn sync_dir(dir: &Path) -> Result<(), StoreError> {
    // Directory fsync is a no-op on Windows, where opening a directory as a
    // file is not permitted; the rename is durable there by other means.
    #[cfg(unix)]
    {
        let handle =
            std::fs::File::open(dir).map_err(|e| map_io(&format!("open {}", dir.display()), e))?;
        handle
            .sync_all()
            .map_err(|e| map_io(&format!("fsync {}", dir.display()), e))?;
    }
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

/// Publish `bytes` at `path` atomically: write a staging file, fsync it,
/// rename it over the target, then fsync the directory. A reader sees either
/// the whole old file or the whole new one — never a torn one.
fn publish_atomic(path: &Path, staging: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    write_durable(staging, bytes)?;
    std::fs::rename(staging, path).map_err(|e| {
        map_io(
            &format!("rename {} -> {}", staging.display(), path.display()),
            e,
        )
    })?;
    let parent = path.parent().unwrap_or(Path::new("."));
    sync_dir(parent)
}

// --- the meta file --------------------------------------------------------

/// The root meta file: who wrote this directory and at which schema version.
/// Both fields are required — a meta file that cannot be read whole is
/// corruption, and a driver that defaulted its way past it would be writing
/// into a directory it never identified.
#[derive(Serialize, Deserialize)]
struct MetaFile {
    format: String,
    schema_version: u32,
}

/// Whether the directory holds anything other than our own lock file.
///
/// The lock file is excluded on purpose: acquiring the lock is what lets us
/// initialize, so counting it would make a half-initialized directory look
/// foreign to the very next open and lock the store out of its own data.
fn is_empty_but_for_lock(dir: &Path) -> Result<bool, StoreError> {
    for entry in
        std::fs::read_dir(dir).map_err(|e| map_io(&format!("read {}", dir.display()), e))?
    {
        let entry = entry.map_err(|e| map_io(&format!("read {}", dir.display()), e))?;
        if entry.file_name() != layout::LOCK_FILE {
            return Ok(false);
        }
    }
    Ok(true)
}

/// What the version gate decided about an existing directory.
enum Gate {
    /// A directory of ours, at this version.
    Ours { needs_migration: bool },
    /// Nothing of ours is here and nothing else is either: initialize.
    Fresh,
}

/// Inspect `root` read-only and decide whether this driver may use it.
///
/// Fails closed on anything unrecognized, and — this is the point of doing it
/// before any write — a refusal leaves the directory bit-for-bit as it was.
/// A directory we refuse is often someone's real data; the one thing worse
/// than not opening it would be leaving our marker inside it.
fn gate(root: &Path) -> Result<Gate, StoreError> {
    let meta_path = root.join(layout::META_FILE);
    let bytes = match std::fs::read(&meta_path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return if is_empty_but_for_lock(root)? {
                Ok(Gate::Fresh)
            } else {
                Err(StoreError::Corrupt {
                    reason: format!(
                        "{} is not a speclink team store and is not empty; refusing to use it",
                        root.display()
                    ),
                })
            };
        }
        Err(e) => return Err(map_io(&format!("read {}", meta_path.display()), e)),
    };

    let meta: MetaFile = serde_json::from_slice(&bytes).map_err(|e| StoreError::Corrupt {
        reason: format!("meta file is unreadable: {e}"),
    })?;
    if meta.format != layout::STORE_MARKER {
        return Err(StoreError::Corrupt {
            reason: format!(
                "directory belongs to {:?}, not to a speclink team store",
                meta.format
            ),
        });
    }
    if meta.schema_version > layout::SCHEMA_VERSION {
        return Err(StoreError::Corrupt {
            reason: format!(
                "incompatible schema version {}; this driver supports {}",
                meta.schema_version,
                layout::SCHEMA_VERSION
            ),
        });
    }
    Ok(Gate::Ours {
        needs_migration: meta.schema_version < layout::SCHEMA_VERSION,
    })
}

// --- the index ------------------------------------------------------------

/// One scope's index: everything true about the scope, in one file that is
/// replaced atomically. Nothing else on disk is authoritative — a content,
/// history or outbox file exists as fact only once an index published here
/// reaches or names it.
///
/// The content file of a document is derived from its key and revision
/// rather than stored: a second copy of a name is a second thing that can
/// disagree with the first.
#[derive(Serialize, Deserialize, Default, Clone)]
struct Index {
    /// The project revision this scope was last published at. Doubles as the
    /// history watermark: records above it belong to commits that never
    /// happened.
    project_revision: u64,
    /// The highest outbox sequence this scope has published. Entries above
    /// it are orphans of a failed commit.
    outbox_seq: u64,
    /// The durable outbox consumer position.
    acked: u64,
    /// Live documents: key → the revision it was last written at. A deleted
    /// document is absent (its history keeps the tombstone).
    documents: BTreeMap<String, u64>,
}

/// One history record as persisted.
#[derive(Serialize, Deserialize)]
struct HistoryFile {
    revision: u64,
    actor: String,
    at: String,
    command: String,
    /// `write` or `tombstone`.
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    digest: Option<String>,
}

/// One outbox entry as persisted.
#[derive(Serialize, Deserialize)]
struct OutboxFile {
    seq: u64,
    revision: u64,
    name: String,
    payload: serde_json::Value,
    actor: String,
    at: String,
}

/// Parse an RFC 3339 timestamp back to UTC; a malformed value is corruption.
fn parse_ts(text: &str) -> Result<DateTime<Utc>, StoreError> {
    DateTime::parse_from_rfc3339(text)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| StoreError::Corrupt {
            reason: format!("unparseable timestamp {text:?}: {e}"),
        })
}

/// Read a scope's index. An absent index is an empty scope, which is a
/// normal state; an unreadable one is corruption and never an empty scope —
/// treating a torn index as "no documents here" is how a store silently
/// loses everything it holds.
fn read_index(paths: &layout::ScopePaths) -> Result<Index, StoreError> {
    let path = paths.index();
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| StoreError::Corrupt {
            reason: format!("index {} is unreadable: {e}", path.display()),
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Index::default()),
        Err(e) => Err(map_io(&format!("read {}", path.display()), e)),
    }
}

/// Publish an index: the single atomic point at which a commit becomes true.
fn publish_index(paths: &layout::ScopePaths, index: &Index) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec(index).map_err(|e| StoreError::Backend {
        source: format!("cannot encode index: {e}"),
    })?;
    publish_atomic(&paths.index(), &paths.index_staging(), &bytes)
}

/// The current revision of `project`: the highest any of its scopes has
/// published.
///
/// The contract's revision is the *project's* — every committed unit of work
/// advances it by one, whichever repo it lands in — while the FS layout
/// keeps one index per scope so that a commit has exactly one atomic publish
/// point. Deriving the project revision from the scope indexes reconciles
/// the two: it needs no counter of its own to fall out of step, and a crash
/// before an index rename simply leaves the maximum where it was.
fn project_revision(root: &Path, project: &str) -> Result<u64, StoreError> {
    let mut highest = 0;
    for entry in
        std::fs::read_dir(root).map_err(|e| map_io(&format!("read {}", root.display()), e))?
    {
        let entry = entry.map_err(|e| map_io(&format!("read {}", root.display()), e))?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        // Anything at the root that is not a scope directory is not ours to
        // interpret: the gate already vouched for the directory as a whole.
        let Ok(owner) = layout::scope_dir_project(&name) else {
            continue;
        };
        if owner != project {
            continue;
        }
        let index = read_index(&layout::ScopePaths {
            dir: entry.path(),
        })?;
        highest = highest.max(index.project_revision);
    }
    Ok(highest)
}

/// How much of a scope's dead weight a sweep may take.
///
/// The difference is who might still be reading. Both modes remove the
/// records of writes that never published — those are inert by definition.
/// Only [`Sweep::AtOpen`] also reclaims *superseded* content: revisions that
/// really were published and have since been overwritten. Nothing reads
/// those at open, but a live snapshot resolves its fixed point through
/// exactly such files, so reclaiming them mid-life would delete the store's
/// own published history out from under a reader — and report the loss as
/// corruption.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sweep {
    /// Opening: no snapshot exists, so superseded revisions go too.
    AtOpen,
    /// Live: only records above the published watermark, which no snapshot
    /// can be holding.
    Abandoned,
}

/// Delete the files of writes that never published, and — at open only —
/// content superseded by later commits.
///
/// A commit writes its content, history and outbox files before the index
/// rename that makes them real. If it never got to the rename, those files
/// are inert, but they are indistinguishable from a future commit's work at
/// the same revision, so they must be gone before that number is handed out
/// again.
fn sweep_orphans(paths: &layout::ScopePaths, mode: Sweep) -> Result<(), StoreError> {
    let index = read_index(paths)?;

    let live: BTreeSet<String> = index
        .documents
        .iter()
        .map(|(key, revision)| layout::content_file(key, *revision))
        .collect();
    for entry in read_dir_opt(&paths.documents())? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let dead = match mode {
            Sweep::AtOpen => !live.contains(&name),
            Sweep::Abandoned => {
                layout::content_file_revision(&name).is_some_and(|rev| rev > index.project_revision)
            }
        };
        if dead {
            remove_file(&entry.path())?;
        }
    }

    // History and outbox records are vouched for by watermark rather than by
    // name: everything at or below the published revision (or sequence) is
    // part of a commit that happened, and stays in both modes. A file whose
    // name this driver did not write is left alone — it is not an orphan of
    // ours to sweep.
    for entry in read_dir_opt(&paths.history())? {
        let name = entry.file_name().to_string_lossy().into_owned();
        if layout::any_history_revision(&name).is_some_and(|rev| rev > index.project_revision) {
            remove_file(&entry.path())?;
        }
    }
    for entry in read_dir_opt(&paths.outbox())? {
        let name = entry.file_name().to_string_lossy().into_owned();
        if layout::outbox_file_seq(&name).is_some_and(|seq| seq > index.outbox_seq) {
            remove_file(&entry.path())?;
        }
    }

    let staging = paths.index_staging();
    if staging.exists() {
        remove_file(&staging)?;
    }
    Ok(())
}

/// Entries of `dir`, or none when the directory does not exist yet.
fn read_dir_opt(dir: &Path) -> Result<Vec<std::fs::DirEntry>, StoreError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(map_io(&format!("read {}", dir.display()), e)),
    };
    entries
        .map(|entry| entry.map_err(|e| map_io(&format!("read {}", dir.display()), e)))
        .collect()
}

fn remove_file(path: &Path) -> Result<(), StoreError> {
    std::fs::remove_file(path).map_err(|e| map_io(&format!("remove {}", path.display()), e))
}

/// Sweep every scope of the data directory.
fn sweep_all(root: &Path) -> Result<(), StoreError> {
    for entry in read_dir_opt(root)? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if layout::scope_dir_project(&entry.file_name().to_string_lossy()).is_err() {
            continue;
        }
        sweep_orphans(&layout::ScopePaths { dir: path }, Sweep::AtOpen)?;
    }
    Ok(())
}

// --- the store ------------------------------------------------------------

// --- the single-writer lock -----------------------------------------------

/// Take the data directory's exclusive advisory lock, or report the store as
/// unavailable.
///
/// The lock is the OS's, not ours, and that is the entire design. A lock the
/// kernel owns is released when its holder's process ends — killed, panicked
/// or unplugged — so a dead writer leaves nothing behind to detect, adopt or
/// break. The alternative (a lock file whose existence or mtime means
/// "held") is precisely the failure mode this driver is required to avoid:
/// it cannot tell a live holder from a corpse, so it either wedges the store
/// forever or steals the lock from a writer that is very much alive.
///
/// A held directory returns `unavailable` immediately. Waiting would turn a
/// misconfiguration — two servers pointed at one directory — into a hang
/// with no explanation; failing now says what is wrong while someone is
/// still watching.
///
/// The lock file is left in place on release: its bytes never mattered, only
/// the kernel's lock on the open file did.
fn acquire_lock(root: &Path) -> Result<std::fs::File, StoreError> {
    use fs4::{FileExt, TryLockError};

    let path = root.join(layout::LOCK_FILE);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| map_io(&format!("open {}", path.display()), e))?;
    // Called fully qualified: `std::fs::File` grew an inherent `try_lock` of
    // its own, which would otherwise silently shadow the trait method.
    match FileExt::try_lock(&file) {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(StoreError::Unavailable),
        Err(TryLockError::Error(e)) => Err(map_io(&format!("lock {}", path.display()), e)),
    }
}

/// State behind the store's single mutex: the fault-injection flags and the
/// crash marker. Serializing writes here is the in-process half of the
/// single-writer guarantee; the advisory lock on the data directory is the
/// other half.
struct Inner {
    /// Crash the next commit at this stage boundary (test hook).
    pending_crash: Option<FaultPoint>,
    /// Make the next commit's outbox append fail (test hook).
    pending_outbox_failure: bool,
    /// A crashed store serves nothing until reopened from durable state.
    crashed: bool,
    /// The directory records a version below the current one.
    needs_migration: bool,
    /// Scope directories that may hold records above their published
    /// watermark, left by a write that failed without crashing the process.
    ///
    /// A crash needs no such note — the store stops serving and the reopen
    /// sweeps. But an absorbed failure leaves the store running, so nothing
    /// reopens, nothing sweeps, and the revision it abandoned is still the
    /// next one to be handed out. Whoever takes that number would publish a
    /// watermark that vouches for the abandoned records. This set is what
    /// makes the sweep happen first.
    dirty: BTreeSet<String>,
}

impl Inner {
    fn unavailable_if_crashed(&self) -> Result<(), StoreError> {
        if self.crashed {
            Err(StoreError::Unavailable)
        } else {
            Ok(())
        }
    }
}

/// A filesystem-backed [`speclink_store::TeamStore`]. Construct with
/// [`FsTeamStore::open`].
pub struct FsTeamStore {
    root: PathBuf,
    inner: Mutex<Inner>,
    /// The held advisory lock. Never read — dropping it (with the store, or
    /// with the whole process) is what releases the directory.
    _lock: std::fs::File,
}

impl FsTeamStore {
    /// Open (or initialize) a store in the data directory at `root`.
    ///
    /// - an absent or empty directory → initialized at the current version;
    /// - a directory of ours at a lower version → opens, flagged for migration;
    /// - a higher version, an unreadable meta file, or another driver's
    ///   marker → [`StoreError::Corrupt`], nothing written;
    /// - a non-empty directory that is not ours → `Corrupt`, nothing written;
    /// - a path that is not a directory → [`StoreError::Backend`].
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Self, StoreError> {
        let root = root.as_ref();
        if root.exists() && !root.is_dir() {
            return Err(StoreError::Backend {
                source: format!("{} is not a directory", root.display()),
            });
        }
        std::fs::create_dir_all(root)
            .map_err(|e| map_io(&format!("create {}", root.display()), e))?;

        // The gate runs read-only and first: creating the lock file inside a
        // directory we are about to refuse would leave our litter in
        // someone else's data and break the promise that a refusal writes
        // nothing.
        let decision = gate(root)?;
        let lock = acquire_lock(root)?;

        let needs_migration = match decision {
            Gate::Ours { needs_migration } => needs_migration,
            Gate::Fresh => {
                initialize(root)?;
                false
            }
        };

        // Whatever a previous process left half-written is swept now: we
        // hold the lock, so no other writer exists, and no snapshot can yet
        // be holding a reference into the data.
        sweep_all(root)?;

        Ok(Self {
            root: root.to_path_buf(),
            inner: Mutex::new(Inner {
                pending_crash: None,
                pending_outbox_failure: false,
                crashed: false,
                needs_migration,
                dirty: BTreeSet::new(),
            }),
            _lock: lock,
        })
    }

    /// The data directory this store persists to.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("fs store mutex poisoned")
    }

    fn paths(&self, scope: &Scope) -> layout::ScopePaths {
        layout::ScopePaths::new(&self.root, scope)
    }

    /// Clear any records an absorbed failure left above this scope's
    /// watermark, before a write takes a revision number.
    ///
    /// Failing here fails the write and keeps the note: a write must never
    /// proceed while records it does not own sit at the number it is about
    /// to publish. This is [`Sweep::Abandoned`] because the store is live —
    /// an open snapshot may still be resolving superseded content, and only
    /// records above the published watermark are certainly unread.
    fn sweep_if_dirty(&self, inner: &mut Inner, scope: &Scope) -> Result<(), StoreError> {
        let dir = layout::scope_dir(scope);
        if inner.dirty.contains(&dir) {
            sweep_orphans(&self.paths(scope), Sweep::Abandoned)?;
            inner.dirty.remove(&dir);
        }
        Ok(())
    }

    /// Test-only fault hook: crash the next commit at `point`. Not part of
    /// the stable contract surface (hidden from docs); it exists so the
    /// conformance harness can drive crash-recovery. See design decision 5.
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

    /// Apply one unit of work.
    ///
    /// Order is the whole guarantee: preconditions are judged against the
    /// published index before anything is written; then content, history and
    /// outbox files land and are fsynced while still referenced by nothing;
    /// then one rename publishes the new index. Every fault point below sits
    /// before that rename, which is why a crash at any of them leaves a
    /// commit that simply never happened.
    fn commit_scope(
        &self,
        uow: &UnitOfWork,
        events: &[EventRecord],
        pending_crash: Option<FaultPoint>,
        fail_outbox: bool,
        crashed: &mut bool,
        wrote: &mut bool,
    ) -> Result<Revision, StoreError> {
        let scope = uow.scope();
        let paths = self.paths(scope);
        let mut index = read_index(&paths)?;

        // 1. Judge every precondition against the pre-commit index before
        //    touching anything: one mismatch rejects the whole commit, so no
        //    other op of this unit may have reached the disk.
        for op in uow.ops() {
            let key = layout::doc_key(op.doc());
            let current = index.documents.get(&key).copied();
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

        let next = Revision(project_revision(&self.root, scope.project.as_str())? + 1);
        create_dir(&paths.documents())?;
        create_dir(&paths.history())?;
        create_dir(&paths.outbox())?;
        // Past this point the commit owns records on disk: if it does not
        // reach the publish, someone has to sweep them before the revision
        // number it took is handed out again.
        *wrote = true;

        // 2. Content files, at their own revision and therefore immutable:
        //    a snapshot that resolved an older revision keeps reading it.
        for op in uow.ops() {
            let key = layout::doc_key(op.doc());
            if let StagedOp::Put { content, .. } = op {
                write_durable(
                    &paths.documents().join(layout::content_file(&key, next.0)),
                    content.as_bytes(),
                )?;
            }
        }
        if pending_crash == Some(FaultPoint::AfterDocWrites) {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }

        // 3. History records (all sharing this commit's timestamp). The
        //    recorded instant is data the caller asked us to keep; it is not
        //    what orders anything.
        let now = Utc::now().to_rfc3339();
        for op in uow.ops() {
            let key = layout::doc_key(op.doc());
            let (kind, digest) = match op {
                StagedOp::Put { content, .. } => ("write", Some(content_digest(content))),
                StagedOp::Delete { .. } => ("tombstone", None),
            };
            write_record(
                &paths.history().join(layout::history_file(&key, next.0)),
                &HistoryFile {
                    revision: next.0,
                    actor: uow.context().actor.clone(),
                    at: now.clone(),
                    command: uow.context().command.clone(),
                    kind: kind.to_string(),
                    digest,
                },
            )?;
        }
        if pending_crash == Some(FaultPoint::AfterHistoryAppend)
            || pending_crash == Some(FaultPoint::BeforeOutboxAppend)
        {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }

        // An armed outbox failure is an ordinary error, not a crash: the
        // commit is abandoned before the publish and the store stays usable.
        if fail_outbox {
            return Err(StoreError::Backend {
                source: "outbox append failed".into(),
            });
        }

        // 4. Outbox entries, at monotonic per-scope sequence numbers.
        let mut seq = index.outbox_seq;
        for event in events {
            seq += 1;
            write_record(
                &paths.outbox().join(layout::outbox_file(seq)),
                &OutboxFile {
                    seq,
                    revision: next.0,
                    name: event.name.clone(),
                    payload: event.payload.clone(),
                    actor: event.actor.clone(),
                    at: event.at.to_rfc3339(),
                },
            )?;
        }
        if pending_crash == Some(FaultPoint::AfterOutboxAppend) {
            *crashed = true;
            return Err(StoreError::Unavailable);
        }

        // 5. The publish. Everything above becomes true here, at once.
        for op in uow.ops() {
            let key = layout::doc_key(op.doc());
            match op {
                StagedOp::Put { .. } => {
                    index.documents.insert(key, next.0);
                }
                StagedOp::Delete { .. } => {
                    index.documents.remove(&key);
                }
            }
        }
        index.project_revision = next.0;
        index.outbox_seq = seq;
        publish_index(&paths, &index)?;
        Ok(next)
    }
}

impl FsTeamStore {
    /// Write a verified bundle's records and publish them as one commit.
    /// Split out so the caller can note the scope as needing a sweep on any
    /// failure between the first write and the publish.
    fn import_records(
        &self,
        bundle: &Bundle,
        paths: &layout::ScopePaths,
        index: &mut Index,
    ) -> Result<ImportReport, StoreError> {
        let next = Revision(project_revision(&self.root, bundle.scope.project.as_str())? + 1);
        create_dir(&paths.documents())?;
        create_dir(&paths.history())?;
        let now = Utc::now().to_rfc3339();
        let mut documents = Vec::with_capacity(bundle.documents.len());
        for doc in &bundle.documents {
            let key = layout::doc_key(&doc.doc);
            write_durable(
                &paths.documents().join(layout::content_file(&key, next.0)),
                doc.content.as_bytes(),
            )?;
            write_record(
                &paths.history().join(layout::history_file(&key, next.0)),
                &HistoryFile {
                    revision: next.0,
                    actor: "import".into(),
                    at: now.clone(),
                    command: "import".into(),
                    kind: "write".into(),
                    digest: Some(doc.digest.clone()),
                },
            )?;
            documents.push(ImportedDoc {
                doc: doc.doc.clone(),
                outcome: match index.documents.insert(key, next.0) {
                    None => ImportOutcome::Created,
                    Some(_) => ImportOutcome::Overwritten,
                },
            });
        }
        index.project_revision = next.0;
        publish_index(paths, index)?;

        Ok(ImportReport {
            project_revision: next,
            documents,
        })
    }
}

fn create_dir(dir: &Path) -> Result<(), StoreError> {
    std::fs::create_dir_all(dir).map_err(|e| map_io(&format!("create {}", dir.display()), e))
}

/// Write one JSON record file durably.
fn write_record<T: Serialize>(path: &Path, record: &T) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec(record).map_err(|e| StoreError::Backend {
        source: format!("cannot encode record {}: {e}", path.display()),
    })?;
    write_durable(path, &bytes)
}

/// Read one document revision's content file.
///
/// The index vouched for this file, so its absence is not "no document" — it
/// is a store that has lost data, and saying `None` here would report that
/// loss as a perfectly ordinary empty result.
fn read_content(
    paths: &layout::ScopePaths,
    key: &str,
    revision: u64,
) -> Result<String, StoreError> {
    let path = paths.documents().join(layout::content_file(key, revision));
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(e) if matches!(e.kind(), io::ErrorKind::NotFound | io::ErrorKind::InvalidData) => {
            Err(StoreError::Corrupt {
                reason: format!(
                    "index references content file {} which cannot be read back: {e}",
                    path.display()
                ),
            })
        }
        Err(e) => Err(map_io(&format!("read {}", path.display()), e)),
    }
}

/// A fixed-point view of one scope.
///
/// It holds the index it read, and nothing else. Content is resolved lazily
/// through that index — safe precisely because content files are immutable
/// and revision-named: later commits publish new files beside the ones this
/// view refers to, and never touch them. The single index read *is* the
/// consistency boundary; a commit that lands afterwards cannot bleed in, in
/// whole or in part.
struct FsSnapshot {
    revision: Revision,
    index: Index,
    paths: layout::ScopePaths,
}

impl Snapshot for FsSnapshot {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn read(&self, doc: &DocumentId) -> Result<Option<Document>, StoreError> {
        let key = layout::doc_key(doc);
        match self.index.documents.get(&key) {
            None => Ok(None),
            Some(revision) => Ok(Some(Document {
                content: read_content(&self.paths, &key, *revision)?,
                revision: Revision(*revision),
            })),
        }
    }

    fn history(&self, doc: &DocumentId) -> Result<Vec<RevisionRecord>, StoreError> {
        let key = layout::doc_key(doc);
        let mut records = Vec::new();
        for entry in read_dir_opt(&self.paths.history())? {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Records above this view's published revision belong either to
            // a commit that never happened or to one that landed after the
            // view was taken. Neither is part of this fixed point.
            match layout::history_file_revision(&name, &key) {
                Some(revision) if revision <= self.index.project_revision => {}
                _ => continue,
            }
            let bytes = std::fs::read(entry.path())
                .map_err(|e| map_io(&format!("read {}", entry.path().display()), e))?;
            let record: HistoryFile =
                serde_json::from_slice(&bytes).map_err(|e| StoreError::Corrupt {
                    reason: format!("history record {name} is unreadable: {e}"),
                })?;
            records.push(RevisionRecord {
                revision: Revision(record.revision),
                actor: record.actor,
                at: parse_ts(&record.at)?,
                command: record.command,
                kind: match (record.kind.as_str(), record.digest) {
                    ("write", Some(digest)) => RevisionKind::Write { digest },
                    ("write", None) => {
                        return Err(StoreError::Corrupt {
                            reason: format!("write history record {name} has no digest"),
                        })
                    }
                    ("tombstone", _) => RevisionKind::Tombstone,
                    (other, _) => {
                        return Err(StoreError::Corrupt {
                            reason: format!("unknown history kind {other:?} in {name}"),
                        })
                    }
                },
            });
        }
        // Oldest first, by revision — the sequence numbers are the order.
        // Directory listings arrive in whatever order the filesystem feels
        // like, and no timestamp on any of these files means anything.
        records.sort_by_key(|record| record.revision);
        Ok(records)
    }
}

/// Write the meta file of a fresh store. Published atomically so a crash
/// mid-initialization leaves an empty directory the next open initializes
/// again, never a half-written marker it would have to call corrupt.
fn initialize(root: &Path) -> Result<(), StoreError> {
    let meta = MetaFile {
        format: layout::STORE_MARKER.to_string(),
        schema_version: layout::SCHEMA_VERSION,
    };
    let bytes = serde_json::to_vec(&meta).map_err(|e| StoreError::Backend {
        source: format!("cannot encode meta file: {e}"),
    })?;
    publish_atomic(
        &root.join(layout::META_FILE),
        &root.join(format!("{}.new", layout::META_FILE)),
        &bytes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backend_that_went_away_is_transient_everything_else_needs_an_operator() {
        // The two halves of the failure model, which the caller acts on very
        // differently: `unavailable` means the data is fine and the mount
        // is not — come back later. `backend` means someone has to go and
        // look. Misfiling a dropped NAS as `backend` would turn a blip into
        // a page; misfiling a bad path as `unavailable` would have a caller
        // retry forever against a directory that will never exist.
        for kind in [
            io::ErrorKind::StaleNetworkFileHandle,
            io::ErrorKind::NetworkDown,
            io::ErrorKind::NetworkUnreachable,
            io::ErrorKind::HostUnreachable,
            io::ErrorKind::TimedOut,
            io::ErrorKind::Interrupted,
            io::ErrorKind::BrokenPipe,
            io::ErrorKind::NotConnected,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted,
        ] {
            assert_eq!(
                map_io("read", io::Error::from(kind)),
                StoreError::Unavailable,
                "{kind:?} is a backend that went away"
            );
        }

        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::NotFound,
            io::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput,
        ] {
            let mapped = map_io("read /data", io::Error::from(kind));
            assert_eq!(mapped.code(), "backend", "{kind:?} needs an operator");
            // The operator gets told where to look, not just that something
            // broke.
            match mapped {
                StoreError::Backend { source } => assert!(
                    source.contains("read /data"),
                    "the failure names its context: {source}"
                ),
                other => panic!("expected backend, got {other:?}"),
            }
        }
    }
}

impl TeamStore for FsTeamStore {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: CONTRACT_VERSION,
            driver: "serverfs".into(),
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
        // Version 1 has no lower version to climb from, so bringing the
        // store to current is a no-op beyond recording the version. The
        // guard is complete for when a future version adds real steps here.
        if inner.needs_migration {
            initialize(&self.root)?;
            inner.needs_migration = false;
        }
        Ok(())
    }

    fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let paths = self.paths(scope);
        // One index read fixes this view. The project revision it reports is
        // the project's, which the sibling scopes' indexes carry.
        let index = read_index(&paths)?;
        let revision = Revision(project_revision(&self.root, scope.project.as_str())?);
        Ok(Box::new(FsSnapshot {
            revision,
            index,
            paths,
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
        self.sweep_if_dirty(&mut inner, uow.scope())?;

        let pending_crash = inner.pending_crash.take();
        let fail_outbox = std::mem::replace(&mut inner.pending_outbox_failure, false);
        let mut crashed = false;
        let mut wrote = false;
        let result = self.commit_scope(
            &uow,
            &events,
            pending_crash,
            fail_outbox,
            &mut crashed,
            &mut wrote,
        );
        if crashed {
            // A crashed store serves nothing until it is reopened, and the
            // reopen sweeps — there is no later commit here to mislead.
            inner.crashed = true;
        } else if wrote && result.is_err() {
            inner.dirty.insert(layout::scope_dir(uow.scope()));
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
        let paths = self.paths(scope);
        let index = read_index(&paths)?;

        let mut documents = Vec::with_capacity(index.documents.len());
        for (key, revision) in &index.documents {
            let content = read_content(&paths, key, *revision)?;
            documents.push(BundleDoc {
                digest: content_digest(&content),
                doc: layout::decode_doc_key(key)?,
                content,
            });
        }
        // Ordered by logical identity, not by anything of this driver's
        // layout: a bundle is the contract's shape and must not depend on
        // how one driver happens to name its files.
        documents.sort_by(|a, b| a.doc.cmp(&b.doc));

        Ok(Bundle {
            format_version: BUNDLE_FORMAT_VERSION,
            scope: scope.clone(),
            project_revision: Revision(project_revision(&self.root, scope.project.as_str())?),
            documents,
        })
    }

    fn import(&self, bundle: Bundle, mode: ImportMode) -> Result<ImportReport, StoreError> {
        // Verify everything before applying anything: a rejected bundle must
        // leave the target exactly as it was, not half-populated with the
        // documents that happened to pass before the bad one.
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
        let scope = &bundle.scope;
        self.sweep_if_dirty(&mut inner, scope)?;
        let paths = self.paths(scope);
        let mut index = read_index(&paths)?;

        if mode == ImportMode::CreateNew && !index.documents.is_empty() {
            return Err(StoreError::Backend {
                source: "import (create-new): target scope already holds documents".into(),
            });
        }

        // Applied as one commit, published by the same single rename: an
        // interrupted import is an import that never happened. Like a commit,
        // it owns records on disk from its first write until the publish, so
        // a failure in between leaves the scope needing a sweep.
        let result = self.import_records(&bundle, &paths, &mut index);
        if result.is_err() {
            inner.dirty.insert(layout::scope_dir(scope));
        }
        result
    }

    fn read_outbox(&self, scope: &Scope, from: OutboxCursor) -> Result<Vec<OutboxEntry>, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let paths = self.paths(scope);
        let index = read_index(&paths)?;

        let mut entries = Vec::new();
        for entry in read_dir_opt(&paths.outbox())? {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Above the published sequence lies a failed commit's work, not
            // an event anyone may consume.
            match layout::outbox_file_seq(&name) {
                Some(seq) if seq > from.0 && seq <= index.outbox_seq => {}
                _ => continue,
            }
            let bytes = std::fs::read(entry.path())
                .map_err(|e| map_io(&format!("read {}", entry.path().display()), e))?;
            let record: OutboxFile =
                serde_json::from_slice(&bytes).map_err(|e| StoreError::Corrupt {
                    reason: format!("outbox entry {name} is unreadable: {e}"),
                })?;
            entries.push(OutboxEntry {
                seq: record.seq,
                revision: Revision(record.revision),
                record: EventRecord {
                    name: record.name,
                    payload: record.payload,
                    actor: record.actor,
                    at: parse_ts(&record.at)?,
                },
            });
        }
        // Replayable order is the sequence order.
        entries.sort_by_key(|entry| entry.seq);
        Ok(entries)
    }

    fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let paths = self.paths(scope);
        let mut index = read_index(&paths)?;

        // Acknowledging past the newest entry would silently skip everything
        // committed later — reject it.
        if up_to.0 > index.outbox_seq {
            return Err(StoreError::Backend {
                source: format!(
                    "ack cursor {} is beyond the outbox end {}",
                    up_to.0, index.outbox_seq
                ),
            });
        }
        // The durable position is monotonic: acknowledging backwards is a
        // no-op, never a rewind that redelivers confirmed entries.
        if up_to.0 > index.acked {
            index.acked = up_to.0;
            publish_index(&paths, &index)?;
        }
        Ok(())
    }

    fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        Ok(OutboxCursor(read_index(&self.paths(scope))?.acked))
    }
}
