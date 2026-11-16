//! Server backup, verification and restore (server-backup capability). A backup
//! is a single self-describing tar file: a manifest (backup format version, UTC
//! creation time, engine version, the kind of the store, identity schema
//! version, the scope list, per-scope document counts and identity counts), one
//! export bundle per registry scope (produced through the host kit's scope
//! export, each document named by its key; never a database-file copy — 決策
//! 2), a time-point-consistent snapshot of the identity database, and a
//! per-member digest chain rooted in a side file. Credentials are never in
//! plaintext: the identity database holds only hashes.
//!
//! This build writes backup format version 2. It also verifies and restores
//! version 1, the format of speclink-server 0.12 and earlier, whose bundles
//! name each document by its kind: a restore turns every kind into its key
//! first (design D4 of server-on-port-examples).

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use speclink_host::scope::{scope_id, ProjectId, RepoId, Scope};
use speclink_host::transfer::{self, BundleDocument, ScopeBundle};
use speclink_protocol::query::ImportDocumentId;
use speclink_store_port::{Actor, MemoryStorePort, ScopeId};

use crate::config::{IdentityConfig, ServerConfig};
use crate::identity::IdentitySqlite;
use crate::identity::IdentityStore;
use crate::storage::{OpenError, ServerStore};

/// The backup format version this build writes. A backup declaring a version
/// this build does not read is refused (fail closed — 決策 4).
pub const BACKUP_FORMAT_VERSION: u32 = 2;

/// The format of speclink-server 0.12 and earlier: read, never written.
const FIRST_FORMAT_VERSION: u32 = 1;

/// The manifest member name, and its self-digest side file (决策 1).
const MANIFEST_NAME: &str = "manifest.json";
const MANIFEST_DIGEST_NAME: &str = "manifest.json.sha256";
/// The identity snapshot member name.
const IDENTITY_MEMBER: &str = "identity.db";

/// Why a backup, verify or restore operation failed.
#[derive(Debug)]
pub enum BackupError {
    /// The configured store could not be opened.
    Open(OpenError),
    /// The store could not be exported or imported.
    Store(String),
    /// The identity database could not be read, snapshotted or placed.
    Identity(String),
    /// Reading or writing the backup file failed.
    Io(String),
    /// The backup file is structurally invalid or a digest did not match.
    Integrity(String),
    /// The restore target already holds data; the reason summarizes what.
    TargetNotEmpty(String),
    /// The configuration cannot host a restore (e.g. an in-memory target).
    Unsupported(String),
    /// The backup holds a document a restore cannot write: one of an old
    /// backup that has no key in this version, or one the import refuses.
    /// Nothing was written.
    NotRestorable(String),
    /// The restore stopped between two scopes: the identity snapshot and the
    /// scopes in `done` are in the target, the scope `failed` and the ones
    /// after it are not.
    Incomplete { done: Vec<String>, failed: String, reason: String },
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::Open(e) => write!(f, "{e}"),
            BackupError::Store(r) => write!(f, "store operation failed: {r}"),
            BackupError::Identity(r) => write!(f, "identity operation failed: {r}"),
            BackupError::Io(r) => write!(f, "backup io failed: {r}"),
            BackupError::Integrity(r) => write!(f, "backup integrity check failed: {r}"),
            BackupError::TargetNotEmpty(r) => {
                write!(
                    f,
                    "restore target is not empty (restore only into an empty target): {r}"
                )
            }
            BackupError::Unsupported(r) => write!(f, "restore is not supported here: {r}"),
            BackupError::NotRestorable(r) => {
                write!(f, "this backup cannot be restored, and nothing was written: {r}")
            }
            BackupError::Incomplete { done, failed, reason } => {
                let done = if done.is_empty() { "none".to_string() } else { done.join(", ") };
                write!(
                    f,
                    "restore stopped at scope {failed}: {reason}; scopes restored before it: {done}. \
                     The target is incomplete: do not put it into service"
                )
            }
        }
    }
}

impl std::error::Error for BackupError {}

/// The outcome of a successful backup, for the admin backup-info record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSummary {
    pub created_at: DateTime<Utc>,
    pub backup_format_version: u32,
    pub scope_count: usize,
    pub member_count: usize,
}

/// The outcome of a successful integrity verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub backup_format_version: u32,
    pub member_count: usize,
    pub scope_count: usize,
}

/// The outcome of restore validation: whether the restored target matches the
/// backup item-for-item, and a human-readable list of any differences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// True only when every checked item matched.
    pub ok: bool,
    /// One entry per item that did not match; empty when `ok`.
    pub differences: Vec<String>,
    pub scopes_checked: usize,
}

// --- the manifest ------------------------------------------------------------

/// A backup manifest: everything needed to describe, verify and validate a
/// backup without the original environment present. A field this build does
/// not know is ignored, so the manifest of version 1 (which has a `store`
/// object where version 2 has `store_kind`) reads with the same type.
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    backup_format_version: u32,
    /// UTC, RFC3339 with a `Z` suffix.
    created_at: String,
    engine_version: String,
    /// The kind of the store the backup was taken from: `sqlite`, `serverfs`,
    /// `postgres` or `memory`. Version 1 does not have it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    store_kind: Option<String>,
    identity_schema_version: u32,
    scopes: Vec<ScopeEntry>,
    identity: IdentityEntry,
    /// Per-member digest of every non-manifest tar member (决策 1).
    members: Vec<MemberDigest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScopeEntry {
    project: String,
    repo: String,
    /// The tar member holding this scope's export bundle.
    member: String,
    doc_count: usize,
}

impl ScopeEntry {
    /// The scope as the messages show it.
    fn shown(&self) -> String {
        format!("{}/{}", self.project, self.repo)
    }

    fn scope_id(&self) -> Result<ScopeId, BackupError> {
        scope_id(&Scope::new(ProjectId::new(&self.project), RepoId::new(&self.repo)))
            .map_err(|e| BackupError::Store(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityEntry {
    /// The tar member holding the identity snapshot.
    member: String,
    user_count: usize,
    project_count: usize,
    repo_count: usize,
    audit_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemberDigest {
    name: String,
    digest: String,
}

// --- the bundle of backup format version 1 ------------------------------------
//
// Only read. These are the server's own types for the old format: they do not
// come from the old store layer.

/// A scope's bundle as version 1 wrote it. Its format version and project
/// revision are not looked at.
#[derive(Debug, Deserialize)]
struct FirstBundle {
    project: String,
    repo: String,
    documents: Vec<FirstDoc>,
}

/// One document of a [`FirstBundle`]. `doc` stays as JSON until a restore
/// turns it into a key, so a kind this build does not know is reported with
/// its scope and not as a file that cannot be read.
#[derive(Debug, Deserialize)]
struct FirstDoc {
    doc: serde_json::Value,
    content: String,
    digest: String,
}

/// The nine document kinds of version 1.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FirstDocId {
    ChangeMeta { change: String },
    ChangeArtifact { change: String, artifact: String },
    ChangeEvidence { change: String },
    CanonicalSpec { capability: String },
    Discussion { slug: String, archived: bool },
    WorkflowConfig,
    ArchivedChange { change: String, doc: String },
    Language,
    BoardOrder,
}

impl From<FirstDocId> for ImportDocumentId {
    fn from(doc: FirstDocId) -> Self {
        match doc {
            FirstDocId::ChangeMeta { change } => ImportDocumentId::ChangeMeta { change },
            FirstDocId::ChangeArtifact { change, artifact } => ImportDocumentId::ChangeArtifact { change, artifact },
            FirstDocId::ChangeEvidence { change } => ImportDocumentId::ChangeEvidence { change },
            FirstDocId::CanonicalSpec { capability } => ImportDocumentId::CanonicalSpec { capability },
            FirstDocId::Discussion { slug, archived } => ImportDocumentId::Discussion { slug, archived },
            FirstDocId::WorkflowConfig => ImportDocumentId::WorkflowConfig,
            FirstDocId::ArchivedChange { change, doc } => ImportDocumentId::ArchivedChange { change, doc },
            FirstDocId::Language => ImportDocumentId::Language,
            FirstDocId::BoardOrder => ImportDocumentId::BoardOrder,
        }
    }
}

/// The most characters of a document's name a message shows.
const SHOWN_CHARS: usize = 200;

impl FirstBundle {
    /// The bundle by key, with the same contents in the same order. A
    /// document whose kind this build does not know, or whose names make no
    /// key, stops the conversion: the error names the scope and the document.
    fn into_keyed(self, entry: &ScopeEntry) -> Result<ScopeBundle, BackupError> {
        if (self.project.as_str(), self.repo.as_str()) != (entry.project.as_str(), entry.repo.as_str()) {
            return Err(BackupError::Integrity(format!(
                "bundle {} holds scope {}/{}, the manifest says {}",
                entry.member,
                self.project,
                self.repo,
                entry.shown()
            )));
        }
        let documents = self
            .documents
            .into_iter()
            .map(|doc| {
                let no_key = |why: &str| {
                    let name: String = doc.doc.to_string().chars().take(SHOWN_CHARS).collect();
                    BackupError::NotRestorable(format!("scope {}: document {name} {why}", entry.shown()))
                };
                let id: FirstDocId = serde_json::from_value(doc.doc.clone())
                    .map_err(|_| no_key("is of a kind this version does not know"))?;
                let key = transfer::key_of_document(&id.into())
                    .ok_or_else(|| no_key("has a name that makes no key in this version"))?;
                Ok(BundleDocument { key: key.as_str().to_string(), content: doc.content, digest: doc.digest })
            })
            .collect::<Result<Vec<_>, BackupError>>()?;
        Ok(ScopeBundle { format_version: transfer::FORMAT_VERSION, scope_version: None, documents })
    }
}

/// A scope's bundle as the backup holds it, in the shape of its version.
enum StoredBundle {
    First(FirstBundle),
    Keyed(ScopeBundle),
}

/// Parse the bundle of `entry`: the structure check of a verify, and the
/// first half of a restore.
fn stored_bundle(
    manifest: &Manifest,
    entry: &ScopeEntry,
    members: &BTreeMap<String, Vec<u8>>,
) -> Result<StoredBundle, BackupError> {
    let bytes = members
        .get(&entry.member)
        .ok_or_else(|| BackupError::Integrity(format!("missing bundle {}", entry.member)))?;
    let unparseable = |e: serde_json::Error| BackupError::Integrity(format!("unparseable bundle {}: {e}", entry.member));
    if manifest.backup_format_version == FIRST_FORMAT_VERSION {
        serde_json::from_slice(bytes).map(StoredBundle::First).map_err(unparseable)
    } else {
        serde_json::from_slice(bytes).map(StoredBundle::Keyed).map_err(unparseable)
    }
}

/// The bundle of `entry` by key, whichever version the backup is in.
fn keyed_bundle(
    manifest: &Manifest,
    entry: &ScopeEntry,
    members: &BTreeMap<String, Vec<u8>>,
) -> Result<ScopeBundle, BackupError> {
    match stored_bundle(manifest, entry, members)? {
        StoredBundle::First(bundle) => bundle.into_keyed(entry),
        StoredBundle::Keyed(bundle) => Ok(bundle),
    }
}

// --- digest + tar helpers --------------------------------------------------

/// The backup member digest: SHA-256 over the member's raw bytes, lowercase hex,
/// algorithm-prefixed — the same shape the store's content digest uses.
fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

/// The most bytes one member of a backup holds: 1 GiB. Every member is read
/// into memory whole, so a file that declares more is refused before it is
/// read. The backup refuses to write a member over the limit too, before it
/// makes the output file: a backup this build writes is one it restores.
const MAX_MEMBER_BYTES: u64 = 1024 * 1024 * 1024;

/// `Err` with the reason when a member of `bytes` bytes is over `limit`.
fn within_limit(name: &str, bytes: u64, limit: u64) -> Result<(), String> {
    if bytes > limit {
        return Err(format!("member {name} is {bytes} bytes, over the limit of {limit} bytes for one member"));
    }
    Ok(())
}

/// Append one in-memory member to a tar builder under `name`.
fn append<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    name: &str,
    bytes: &[u8],
) -> Result<(), BackupError> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_cksum();
    builder
        .append_data(&mut header, name, bytes)
        .map_err(|e| BackupError::Io(e.to_string()))
}

/// Write `members` (name, bytes) as the tar at `output`. Every member's size
/// is checked first: a member over `limit` fails the write with `output` not
/// made, and a file already at `output` as it was.
fn write_archive(output: &Path, members: &[(&str, &[u8])], limit: u64) -> Result<(), BackupError> {
    for (name, bytes) in members {
        within_limit(name, bytes.len() as u64, limit).map_err(BackupError::Io)?;
    }
    let file = std::fs::File::create(output).map_err(|e| BackupError::Io(e.to_string()))?;
    let mut builder = tar::Builder::new(file);
    for (name, bytes) in members {
        append(&mut builder, name, bytes)?;
    }
    builder.finish().map_err(|e| BackupError::Io(e.to_string()))
}

// --- backup ----------------------------------------------------------------

/// Produce a backup of the store and identity into `output` (决策 1/2). Offline:
/// the caller guarantees no writes for the backup's duration.
pub fn create(
    store: &dyn ServerStore,
    identity: &IdentitySqlite,
    output: &Path,
) -> Result<BackupSummary, BackupError> {
    // Enumerate every registry scope: one bundle per (project, repo) pair. The
    // registry lives in the identity store (决策 1), not the store or config.
    let projects = identity
        .list_projects()
        .map_err(|e| BackupError::Identity(e.to_string()))?;
    let mut repo_count = 0usize;
    let mut members: Vec<MemberDigest> = Vec::new();
    let mut scope_entries: Vec<ScopeEntry> = Vec::new();
    // Serialized bundle bytes, kept to write into the tar after the manifest.
    let mut bundle_bytes: Vec<(String, Vec<u8>)> = Vec::new();
    for project in &projects {
        let repos = identity
            .list_repos(&project.key)
            .map_err(|e| BackupError::Identity(e.to_string()))?;
        repo_count += repos.len();
        for repo in repos {
            let member = format!("bundles/{}.json", scope_entries.len());
            let mut entry =
                ScopeEntry { project: project.key.clone(), repo: repo.key, member: member.clone(), doc_count: 0 };
            let bundle = transfer::export(store, &entry.scope_id()?)
                .map_err(|e| BackupError::Store(format!("export scope {}: {e}", entry.shown())))?;
            entry.doc_count = bundle.documents.len();
            let bytes = serde_json::to_vec_pretty(&bundle)
                .map_err(|e| BackupError::Store(format!("serialize bundle: {e}")))?;
            members.push(MemberDigest { name: member.clone(), digest: digest_bytes(&bytes) });
            scope_entries.push(entry);
            bundle_bytes.push((member, bytes));
        }
    }

    // Snapshot the identity database through the online backup API into a temp
    // file, then fold its bytes into the tar (决策 2).
    let snapshot = tempfile::NamedTempFile::new().map_err(|e| BackupError::Io(e.to_string()))?;
    identity
        .snapshot_to(snapshot.path())
        .map_err(|e| BackupError::Identity(e.to_string()))?;
    let identity_bytes =
        std::fs::read(snapshot.path()).map_err(|e| BackupError::Io(e.to_string()))?;
    members.push(MemberDigest {
        name: IDENTITY_MEMBER.to_string(),
        digest: digest_bytes(&identity_bytes),
    });

    // Identity counts, for restore validation to compare against.
    let user_count = identity.list_users().map_err(idn)?.len();
    let project_count = projects.len();
    let audit_count = identity.list_audit(u32::MAX, 0).map_err(idn)?.len();
    let identity_schema_version = identity.schema_version().map_err(idn)?;

    let created_at = Utc::now();
    let scope_count = scope_entries.len();
    let manifest = Manifest {
        backup_format_version: BACKUP_FORMAT_VERSION,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Micros, true),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        store_kind: Some(store.kind().to_string()),
        identity_schema_version,
        scopes: scope_entries,
        identity: IdentityEntry {
            member: IDENTITY_MEMBER.to_string(),
            user_count,
            project_count,
            repo_count,
            audit_count,
        },
        members,
    };
    let member_count = manifest.members.len();
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| BackupError::Io(format!("serialize manifest: {e}")))?;
    let manifest_digest = digest_bytes(&manifest_bytes);

    // Write the tar: manifest, its self-digest side file, every bundle, then the
    // identity snapshot.
    let mut archive: Vec<(&str, &[u8])> =
        vec![(MANIFEST_NAME, &manifest_bytes), (MANIFEST_DIGEST_NAME, manifest_digest.as_bytes())];
    archive.extend(bundle_bytes.iter().map(|(member, bytes)| (member.as_str(), bytes.as_slice())));
    archive.push((IDENTITY_MEMBER, &identity_bytes));
    write_archive(output, &archive, MAX_MEMBER_BYTES)?;

    Ok(BackupSummary {
        created_at,
        backup_format_version: BACKUP_FORMAT_VERSION,
        scope_count,
        member_count,
    })
}

/// Map an identity error into a backup error.
fn idn(e: crate::identity::IdentityError) -> BackupError {
    BackupError::Identity(e.to_string())
}

// --- verify ----------------------------------------------------------------

/// Read every member of the backup tar at `input` into name → bytes.
fn read_members(input: &Path) -> Result<BTreeMap<String, Vec<u8>>, BackupError> {
    read_members_within(input, MAX_MEMBER_BYTES)
}

/// [`read_members`] with the size limit of one member given.
///
/// A member's name is only ever a key of the returned map: no member is
/// written to disk under its name. A name that is not a plain relative path
/// (one with `..`, or an absolute one) is refused all the same: no backup
/// this server wrote has one, so the file is not a backup.
fn read_members_within(input: &Path, limit: u64) -> Result<BTreeMap<String, Vec<u8>>, BackupError> {
    use std::io::Read;
    let file = std::fs::File::open(input).map_err(|e| BackupError::Io(e.to_string()))?;
    let mut archive = tar::Archive::new(file);
    let mut members = BTreeMap::new();
    for entry in archive
        .entries()
        .map_err(|e| BackupError::Io(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| BackupError::Io(e.to_string()))?;
        let path = entry.path().map_err(|e| BackupError::Io(e.to_string()))?;
        let name = path.to_string_lossy().to_string();
        if !path.components().all(|part| matches!(part, std::path::Component::Normal(_))) {
            return Err(BackupError::Integrity(format!(
                "member name {name:?} is not a plain relative path"
            )));
        }
        // The entry's own size, not the header's: a PAX record can give a
        // size larger than the header's field.
        within_limit(&name, entry.size(), limit).map_err(BackupError::Integrity)?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| BackupError::Io(e.to_string()))?;
        members.insert(name, bytes);
    }
    Ok(members)
}

/// Parse and integrity-check a backup: the manifest self-digest, the known
/// format version, every per-member digest, and every bundle's structure. On
/// success returns the parsed manifest and the members for a caller (restore) to
/// consume; on any failure a [`BackupError::Integrity`] naming the fault.
fn load_verified(input: &Path) -> Result<(Manifest, BTreeMap<String, Vec<u8>>), BackupError> {
    let members = read_members(input)?;

    let manifest_bytes = members
        .get(MANIFEST_NAME)
        .ok_or_else(|| BackupError::Integrity(format!("missing {MANIFEST_NAME}")))?;
    // The manifest is the root of trust: its self-digest side file must match
    // before any value it carries is believed.
    let side = members
        .get(MANIFEST_DIGEST_NAME)
        .ok_or_else(|| BackupError::Integrity(format!("missing {MANIFEST_DIGEST_NAME}")))?;
    let side = String::from_utf8_lossy(side);
    if side.trim() != digest_bytes(manifest_bytes) {
        return Err(BackupError::Integrity(format!(
            "{MANIFEST_NAME} digest mismatch"
        )));
    }
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| BackupError::Integrity(format!("unparseable {MANIFEST_NAME}: {e}")))?;

    // Fail closed on an unknown format version (决策 4).
    if ![FIRST_FORMAT_VERSION, BACKUP_FORMAT_VERSION].contains(&manifest.backup_format_version) {
        return Err(BackupError::Integrity(format!(
            "incompatible backup format version {} (this build reads {FIRST_FORMAT_VERSION} and {BACKUP_FORMAT_VERSION})",
            manifest.backup_format_version
        )));
    }

    // Every listed member must be present and match its digest — this is where
    // a single-bit change to any member is caught.
    for member in &manifest.members {
        let bytes = members
            .get(&member.name)
            .ok_or_else(|| BackupError::Integrity(format!("missing member {}", member.name)))?;
        if digest_bytes(bytes) != member.digest {
            return Err(BackupError::Integrity(format!(
                "digest mismatch for {}",
                member.name
            )));
        }
    }

    // Every bundle must parse as a bundle of the backup's version (structure
    // check). Whether a version 1 document has a key is a restore's question.
    for scope in &manifest.scopes {
        stored_bundle(&manifest, scope, &members)?;
    }

    Ok((manifest, members))
}

/// Verify a backup file's integrity without touching any target (决策 4). Returns
/// a report on success; a [`BackupError::Integrity`] naming the fault otherwise.
pub fn verify(input: &Path) -> Result<VerifyReport, BackupError> {
    let (manifest, _) = load_verified(input)?;
    Ok(VerifyReport {
        backup_format_version: manifest.backup_format_version,
        member_count: manifest.members.len(),
        scope_count: manifest.scopes.len(),
    })
}

// --- restore ---------------------------------------------------------------

/// Every scope of the backup with its bundle by key, in the manifest's order.
/// A version 1 bundle is converted here, so a document with no key stops a
/// restore before anything is written.
fn keyed_bundles<'m>(
    manifest: &'m Manifest,
    members: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<(&'m ScopeEntry, ScopeId, ScopeBundle)>, BackupError> {
    manifest
        .scopes
        .iter()
        .map(|entry| Ok((entry, entry.scope_id()?, keyed_bundle(manifest, entry, members)?)))
        .collect()
}

/// Run the import of every bundle against a scratch store in memory. The
/// import checks each document (its key follows a naming rule, its digest is
/// its content's, a board order is within its size): a bundle it would
/// refuse stops a restore here, before anything is written.
fn check_importable(bundles: &[(&ScopeEntry, ScopeId, ScopeBundle)], actor: &Actor) -> Result<(), BackupError> {
    let scratch = MemoryStorePort::new();
    for (entry, scope, bundle) in bundles {
        speclink_host::port_exec::now()
            .map_err(transfer::TransferError::Store)
            .and_then(|at| transfer::import(&scratch, scope, actor, bundle.clone(), at))
            .map_err(|e| BackupError::NotRestorable(format!("scope {}: {e}", entry.shown())))?;
    }
    Ok(())
}

/// Import every bundle into `store`: each scope is one commit into an empty
/// scope. The imports are not one transaction over every scope: a failure
/// between two scopes is [`BackupError::Incomplete`].
fn import_all(
    store: &dyn ServerStore,
    bundles: Vec<(&ScopeEntry, ScopeId, ScopeBundle)>,
    actor: &Actor,
) -> Result<(), BackupError> {
    let mut done = Vec::new();
    for (entry, scope, bundle) in bundles {
        let imported = speclink_host::port_exec::now()
            .map_err(transfer::TransferError::Store)
            .and_then(|at| transfer::import(store, &scope, actor, bundle, at));
        if let Err(e) = imported {
            return Err(BackupError::Incomplete { done, failed: entry.shown(), reason: e.to_string() });
        }
        done.push(entry.shown());
    }
    Ok(())
}

/// Restore a backup into the empty target the config declares (决策 3), then
/// validate. Order: integrity verify → every bundle by key → the import's
/// document checks on a scratch store → empty-target guard → identity
/// snapshot → per-scope import → validation. A non-empty target, an
/// integrity failure, a document with no key or one the import refuses stops
/// the restore before any write.
pub fn restore(config: &ServerConfig, input: &Path) -> Result<ValidationReport, BackupError> {
    // 决策 3: the integrity verification is restore's first step.
    let (manifest, members) = load_verified(input)?;
    let bundles = keyed_bundles(&manifest, &members)?;
    let actor = Actor::new("speclink-server", "speclink-server restore").expect("a fixed actor");
    check_importable(&bundles, &actor)?;

    // The identity snapshot is placed at the file level, so a restore target must
    // be a persistent sqlite database.
    let identity_path = match &config.identity {
        IdentityConfig::Sqlite { path } => path.clone(),
        IdentityConfig::Memory => {
            return Err(BackupError::Unsupported(
                "the target identity must be a sqlite database".into(),
            ))
        }
    };

    // Empty-target guard (决策 3). Read-only: on refusal the target's existing
    // bytes are left untouched.
    let store = crate::build_store(&config.store).map_err(BackupError::Open)?;
    {
        let mut existing = Vec::new();
        for (entry, scope, _) in &bundles {
            let held = transfer::export(&*store, scope)
                .map_err(|e| BackupError::Store(e.to_string()))?
                .documents
                .len();
            if held > 0 {
                existing.push(format!("scope {} holds {held} document(s)", entry.shown()));
            }
        }
        let identity = crate::build_identity(&config.identity)
            .map_err(|e| BackupError::Identity(e.to_string()))?;
        let users = identity.list_users().map_err(idn)?.len();
        if users > 0 {
            existing.push(format!("{users} user(s) exist"));
        }
        if !existing.is_empty() {
            return Err(BackupError::TargetNotEmpty(existing.join("; ")));
        }
    }

    // Place the identity snapshot at the target path (决策 3).
    let snapshot_bytes = members.get(&manifest.identity.member).ok_or_else(|| {
        BackupError::Integrity(format!(
            "missing identity member {}",
            manifest.identity.member
        ))
    })?;
    std::fs::write(&identity_path, snapshot_bytes).map_err(|e| BackupError::Io(e.to_string()))?;

    import_all(&*store, bundles, &actor)?;
    drop(store);

    validate(config, input)
}

/// Re-read the restored target and compare it item-for-item against the backup:
/// per scope the keys, each document's content digest and the document count;
/// identity counts and schema version against the manifest. Separable so it can
/// be re-run after a restore.
pub fn validate(config: &ServerConfig, input: &Path) -> Result<ValidationReport, BackupError> {
    let (manifest, members) = load_verified(input)?;
    let mut differences = Vec::new();

    // Per scope: the target's re-exported documents must be the backup's, key
    // by key.
    let store = crate::build_store(&config.store).map_err(BackupError::Open)?;
    for (entry, scope, expected) in keyed_bundles(&manifest, &members)? {
        let target = transfer::export(&*store, &scope).map_err(|e| BackupError::Store(e.to_string()))?;
        if target.documents.len() != entry.doc_count {
            differences.push(format!(
                "scope {}: {} document(s), expected {}",
                entry.shown(),
                target.documents.len(),
                entry.doc_count
            ));
            continue;
        }
        if digests_by_key(&target) != digests_by_key(&expected) {
            differences.push(format!(
                "scope {}: the keys or the content digests differ from the backup",
                entry.shown()
            ));
        }
    }

    // Identity: counts and schema version against the manifest.
    let identity = crate::build_identity(&config.identity)
        .map_err(|e| BackupError::Identity(e.to_string()))?;
    let user_count = identity.list_users().map_err(idn)?.len();
    let projects = identity.list_projects().map_err(idn)?;
    let project_count = projects.len();
    let mut repo_count = 0usize;
    for project in &projects {
        repo_count += identity.list_repos(&project.key).map_err(idn)?.len();
    }
    let audit_count = identity.list_audit(u32::MAX, 0).map_err(idn)?.len();
    let schema_version = identity.schema_version().map_err(idn)?;
    for (name, got, want) in [
        ("user count", user_count, manifest.identity.user_count),
        (
            "project count",
            project_count,
            manifest.identity.project_count,
        ),
        ("repo count", repo_count, manifest.identity.repo_count),
        ("audit count", audit_count, manifest.identity.audit_count),
    ] {
        if got != want {
            differences.push(format!("identity {name}: {got}, expected {want}"));
        }
    }
    if schema_version != manifest.identity_schema_version {
        differences.push(format!(
            "identity schema version: {schema_version}, expected {}",
            manifest.identity_schema_version
        ));
    }

    Ok(ValidationReport {
        ok: differences.is_empty(),
        differences,
        scopes_checked: manifest.scopes.len(),
    })
}

/// Every document of a bundle as key → content digest.
fn digests_by_key(bundle: &ScopeBundle) -> BTreeMap<&str, &str> {
    bundle.documents.iter().map(|doc| (doc.key.as_str(), doc.digest.as_str())).collect()
}

/// Export one scope as the same self-verifying bundle JSON a backup member holds
/// (决策 5) — the admin scope-download reuses the backup export shape, so a
/// downloaded bundle passes the same structure and digest verification.
pub fn export_bundle_json(store: &dyn ServerStore, scope: &Scope) -> Result<Vec<u8>, BackupError> {
    let scope = scope_id(scope).map_err(|e| BackupError::Store(e.to_string()))?;
    let bundle = transfer::export(store, &scope).map_err(|e| BackupError::Store(e.to_string()))?;
    serde_json::to_vec_pretty(&bundle).map_err(|e| BackupError::Store(format!("serialize bundle: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_member_over_the_size_limit_is_refused_by_its_declared_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.tar");
        let mut builder = tar::Builder::new(std::fs::File::create(&path).unwrap());
        append(&mut builder, "small.json", &[b'a'; 64]).unwrap();
        append(&mut builder, "big.json", &[b'b'; 65]).unwrap();
        builder.finish().unwrap();
        drop(builder);

        let err = read_members_within(&path, 64).expect_err("65 bytes is over a limit of 64");
        match err {
            BackupError::Integrity(reason) => {
                assert!(reason.contains("big.json") && reason.contains("65 bytes"), "{reason}")
            }
            other => panic!("not an integrity failure: {other}"),
        }
        let members = read_members_within(&path, 65).expect("at the limit is within it");
        assert_eq!(members["big.json"].len(), 65);
    }

    #[test]
    fn a_pax_size_record_does_not_get_a_member_past_the_limit() {
        // The header says 0 bytes; a PAX record before it says 65.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.tar");
        let mut builder = tar::Builder::new(std::fs::File::create(&path).unwrap());
        builder.append_pax_extensions([("size", "65".as_bytes())]).unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_path("big.json").unwrap();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &[b'b'; 65][..]).unwrap();
        builder.finish().unwrap();
        drop(builder);

        match read_members_within(&path, 64) {
            Err(BackupError::Integrity(reason)) => assert!(reason.contains("65 bytes"), "{reason}"),
            Ok(members) => panic!("read {} bytes past a limit of 64", members["big.json"].len()),
            Err(other) => panic!("not an integrity failure: {other}"),
        }
    }

    #[test]
    fn a_backup_with_a_member_over_the_limit_leaves_the_file_at_its_path_as_it_was() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("backup.tar");
        std::fs::write(&output, b"the backup of last night").unwrap();

        let members: [(&str, &[u8]); 2] = [("manifest.json", b"{}"), ("identity.db", &[0u8; 65])];
        let err = write_archive(&output, &members, 64).expect_err("65 bytes is over a limit of 64");
        assert!(err.to_string().contains("identity.db"), "{err}");
        assert_eq!(std::fs::read(&output).unwrap(), b"the backup of last night", "the old backup is whole");

        let fresh = dir.path().join("fresh.tar");
        assert!(write_archive(&fresh, &members, 64).is_err());
        assert!(!fresh.exists(), "no half-written file is left");
        write_archive(&fresh, &members, 65).expect("within the limit");
        assert_eq!(read_members_within(&fresh, 65).unwrap()["identity.db"].len(), 65);
    }

    /// The in-memory store, with one scope whose commits fail.
    struct FailsScope {
        inner: crate::storage::Memory,
        fails: ScopeId,
    }

    impl speclink_store_port::StorePort for FailsScope {
        fn list(
            &self,
            scope: &ScopeId,
            prefix: &speclink_store_port::KeyPrefix,
        ) -> Result<speclink_store_port::Listing, speclink_store_port::PortError> {
            self.inner.list(scope, prefix)
        }
        fn read(
            &self,
            scope: &ScopeId,
            keys: &[speclink_store_port::DocKey],
        ) -> Result<Vec<Option<speclink_store_port::Document>>, speclink_store_port::PortError> {
            self.inner.read(scope, keys)
        }
        fn commit(
            &self,
            scope: &ScopeId,
            commit: &speclink_store_port::Commit,
        ) -> Result<speclink_store_port::Version, speclink_store_port::PortError> {
            if *scope == self.fails {
                return Err(speclink_store_port::PortError::Backend { message: "disk full".into() });
            }
            self.inner.commit(scope, commit)
        }
    }

    impl ServerStore for FailsScope {
        fn kind(&self) -> &'static str {
            self.inner.kind()
        }
        fn events_after(
            &self,
            scope: &ScopeId,
            after: u64,
            limit: usize,
        ) -> Result<Vec<(u64, speclink_store_port::RecordedEvent)>, speclink_store_port::PortError> {
            self.inner.events_after(scope, after, limit)
        }
        fn event_floor(&self, scope: &ScopeId) -> Result<u64, speclink_store_port::PortError> {
            self.inner.event_floor(scope)
        }
        fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), speclink_store_port::PortError> {
            self.inner.trim_events(scope, through)
        }
        fn probe(&self) -> Result<(), speclink_store_port::PortError> {
            self.inner.probe()
        }
    }

    fn entry(repo: &str) -> ScopeEntry {
        ScopeEntry { project: "demo".into(), repo: repo.into(), member: format!("bundles/{repo}.json"), doc_count: 1 }
    }

    fn one_document() -> ScopeBundle {
        let content = "schema: spec-driven\n";
        ScopeBundle {
            format_version: transfer::FORMAT_VERSION,
            scope_version: None,
            documents: vec![BundleDocument {
                key: "config.yaml".into(),
                content: content.into(),
                digest: digest_bytes(content.as_bytes()),
            }],
        }
    }

    /// design「還原在逐範圍匯入的中途失敗」: the failure names the scopes that
    /// were restored and the one that failed, and says the target is not to
    /// be used.
    #[test]
    fn an_import_that_fails_between_two_scopes_reports_what_was_done_and_what_failed() {
        let (backend, web, api) = (entry("backend"), entry("web"), entry("api"));
        let store = FailsScope { inner: crate::storage::Memory::new(), fails: web.scope_id().unwrap() };
        let bundles = [&backend, &web, &api]
            .into_iter()
            .map(|entry| (entry, entry.scope_id().unwrap(), one_document()))
            .collect();
        let actor = Actor::new("t", "t").unwrap();

        let err = import_all(&store, bundles, &actor).expect_err("the second scope fails");

        match &err {
            BackupError::Incomplete { done, failed, reason } => {
                assert_eq!(done, &["demo/backend"]);
                assert_eq!(failed, "demo/web");
                assert!(reason.contains("disk full"), "{reason}");
            }
            other => panic!("not an incomplete restore: {other}"),
        }
        let shown = err.to_string();
        assert!(shown.contains("scopes restored before it: demo/backend"), "{shown}");
        assert!(shown.contains("do not put it into service"), "{shown}");
        // The first scope was written, the one after the failure was not.
        let everything = speclink_store_port::KeyPrefix::new("").unwrap();
        let held = |entry: &ScopeEntry| {
            speclink_store_port::StorePort::list(&store, &entry.scope_id().unwrap(), &everything).unwrap().entries.len()
        };
        assert_eq!((held(&backend), held(&web), held(&api)), (1, 0, 0));
    }

    #[test]
    fn the_limit_holds_for_what_the_backup_writes_too() {
        assert!(within_limit("identity.db", MAX_MEMBER_BYTES, MAX_MEMBER_BYTES).is_ok());
        let reason = within_limit("identity.db", MAX_MEMBER_BYTES + 1, MAX_MEMBER_BYTES).unwrap_err();
        assert!(reason.contains("identity.db"), "{reason}");
    }
}
