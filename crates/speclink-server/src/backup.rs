//! Server backup, verification and restore (server-backup capability). A backup
//! is a single self-describing tar file: a manifest (backup format version, UTC
//! creation time, engine version, store manifest, identity schema version, the
//! scope list, per-scope document counts and identity counts), one export bundle
//! per registry scope (produced through the TeamStore export contract, never a
//! database-file copy — 決策 2), a time-point-consistent snapshot of the identity
//! database, and a per-member digest chain rooted in a side file. Credentials are
//! never in plaintext: the identity database holds only hashes.

use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use speclink_store::{
    Bundle, BundleDoc, DocumentId, ImportMode, ProjectId, RepoId, Revision, Scope, TeamStore,
};

use crate::config::{IdentityConfig, ServerConfig};
use crate::identity::IdentityStore;
use crate::identity_sqlite::IdentitySqlite;

/// The backup format version this build writes and reads. A backup declaring an
/// unknown version is refused (fail closed — 決策 4).
pub const BACKUP_FORMAT_VERSION: u32 = 1;

/// The manifest member name, and its self-digest side file (决策 1).
const MANIFEST_NAME: &str = "manifest.json";
const MANIFEST_DIGEST_NAME: &str = "manifest.json.sha256";
/// The identity snapshot member name.
const IDENTITY_MEMBER: &str = "identity.db";

/// Why a backup, verify or restore operation failed.
#[derive(Debug)]
pub enum BackupError {
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
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

// --- serializable manifest and bundle mirrors ------------------------------
//
// The TeamStore contract types intentionally carry no serde derives, and this
// knife is a consumer of the contract, not its author (决策 2 / Non-Goals). The
// backup format therefore owns its own JSON shapes and converts at the boundary.

/// A backup manifest: everything needed to describe, verify and validate a
/// backup without the original environment present.
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    backup_format_version: u32,
    /// UTC, RFC3339 with a `Z` suffix.
    created_at: String,
    engine_version: String,
    store: StoreManifest,
    identity_schema_version: u32,
    scopes: Vec<ScopeEntry>,
    identity: IdentityEntry,
    /// Per-member digest of every non-manifest tar member (决策 1).
    members: Vec<MemberDigest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreManifest {
    driver: String,
    contract_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScopeEntry {
    project: String,
    repo: String,
    /// The tar member holding this scope's export bundle.
    member: String,
    doc_count: usize,
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

#[derive(Debug, Serialize, Deserialize)]
struct BundleDto {
    format_version: u32,
    project: String,
    repo: String,
    project_revision: u64,
    documents: Vec<BundleDocDto>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleDocDto {
    doc: DocIdDto,
    content: String,
    digest: String,
}

/// A serializable mirror of the closed [`DocumentId`] set.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DocIdDto {
    ChangeMeta { change: String },
    ChangeArtifact { change: String, artifact: String },
    CanonicalSpec { capability: String },
    Discussion { slug: String, archived: bool },
    WorkflowConfig,
    ArchivedChange { change: String, doc: String },
    Language,
    BoardOrder,
}

impl From<&DocumentId> for DocIdDto {
    fn from(doc: &DocumentId) -> Self {
        match doc {
            DocumentId::ChangeMeta { change } => DocIdDto::ChangeMeta {
                change: change.clone(),
            },
            DocumentId::ChangeArtifact { change, artifact } => DocIdDto::ChangeArtifact {
                change: change.clone(),
                artifact: artifact.clone(),
            },
            DocumentId::CanonicalSpec { capability } => DocIdDto::CanonicalSpec {
                capability: capability.clone(),
            },
            DocumentId::Discussion { slug, archived } => DocIdDto::Discussion {
                slug: slug.clone(),
                archived: *archived,
            },
            DocumentId::WorkflowConfig => DocIdDto::WorkflowConfig,
            DocumentId::ArchivedChange { change, doc } => DocIdDto::ArchivedChange {
                change: change.clone(),
                doc: doc.clone(),
            },
            DocumentId::Language => DocIdDto::Language,
            DocumentId::BoardOrder => DocIdDto::BoardOrder,
        }
    }
}

impl From<DocIdDto> for DocumentId {
    fn from(dto: DocIdDto) -> Self {
        match dto {
            DocIdDto::ChangeMeta { change } => DocumentId::ChangeMeta { change },
            DocIdDto::ChangeArtifact { change, artifact } => {
                DocumentId::ChangeArtifact { change, artifact }
            }
            DocIdDto::CanonicalSpec { capability } => DocumentId::CanonicalSpec { capability },
            DocIdDto::Discussion { slug, archived } => DocumentId::Discussion { slug, archived },
            DocIdDto::WorkflowConfig => DocumentId::WorkflowConfig,
            DocIdDto::ArchivedChange { change, doc } => DocumentId::ArchivedChange { change, doc },
            DocIdDto::Language => DocumentId::Language,
            DocIdDto::BoardOrder => DocumentId::BoardOrder,
        }
    }
}

impl BundleDto {
    fn from_bundle(bundle: &Bundle) -> Self {
        BundleDto {
            format_version: bundle.format_version,
            project: bundle.scope.project.as_str().to_string(),
            repo: bundle.scope.repo.as_str().to_string(),
            project_revision: bundle.project_revision.0,
            documents: bundle
                .documents
                .iter()
                .map(|d| BundleDocDto {
                    doc: DocIdDto::from(&d.doc),
                    content: d.content.clone(),
                    digest: d.digest.clone(),
                })
                .collect(),
        }
    }

    fn into_bundle(self) -> Bundle {
        Bundle {
            format_version: self.format_version,
            scope: Scope::new(ProjectId::new(self.project), RepoId::new(self.repo)),
            project_revision: Revision(self.project_revision),
            documents: self
                .documents
                .into_iter()
                .map(|d| BundleDoc {
                    doc: d.doc.into(),
                    content: d.content,
                    digest: d.digest,
                })
                .collect(),
        }
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

// --- backup ----------------------------------------------------------------

/// Produce a backup of the store and identity into `output` (决策 1/2). Offline:
/// the caller guarantees no writes for the backup's duration.
pub fn create(
    store: &dyn TeamStore,
    identity: &IdentitySqlite,
    output: &Path,
) -> Result<BackupSummary, BackupError> {
    // Enumerate every registry scope: one bundle per (project, repo) pair. The
    // registry lives in the identity store (决策 1), not the store or config.
    let projects = identity
        .list_projects()
        .map_err(|e| BackupError::Identity(e.to_string()))?;
    let mut repo_count = 0usize;
    let mut scopes: Vec<Scope> = Vec::new();
    for project in &projects {
        let repos = identity
            .list_repos(&project.key)
            .map_err(|e| BackupError::Identity(e.to_string()))?;
        repo_count += repos.len();
        for repo in repos {
            scopes.push(Scope::new(
                ProjectId::new(&project.key),
                RepoId::new(&repo.key),
            ));
        }
    }

    let mut members: Vec<MemberDigest> = Vec::new();
    let mut scope_entries: Vec<ScopeEntry> = Vec::new();
    // Serialized bundle bytes, kept to write into the tar after the manifest.
    let mut bundle_bytes: Vec<(String, Vec<u8>)> = Vec::new();
    for (i, scope) in scopes.iter().enumerate() {
        let bundle = store
            .export(scope)
            .map_err(|e| BackupError::Store(e.to_string()))?;
        let dto = BundleDto::from_bundle(&bundle);
        let bytes = serde_json::to_vec_pretty(&dto)
            .map_err(|e| BackupError::Store(format!("serialize bundle: {e}")))?;
        let member = format!("bundles/{i}.json");
        members.push(MemberDigest {
            name: member.clone(),
            digest: digest_bytes(&bytes),
        });
        scope_entries.push(ScopeEntry {
            project: scope.project.as_str().to_string(),
            repo: scope.repo.as_str().to_string(),
            member: member.clone(),
            doc_count: dto.documents.len(),
        });
        bundle_bytes.push((member, bytes));
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

    let store_manifest = store.manifest();
    let created_at = Utc::now();
    let manifest = Manifest {
        backup_format_version: BACKUP_FORMAT_VERSION,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Micros, true),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        store: StoreManifest {
            driver: store_manifest.driver,
            contract_version: store_manifest.contract_version,
        },
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
    let file = std::fs::File::create(output).map_err(|e| BackupError::Io(e.to_string()))?;
    let mut builder = tar::Builder::new(file);
    append(&mut builder, MANIFEST_NAME, &manifest_bytes)?;
    append(
        &mut builder,
        MANIFEST_DIGEST_NAME,
        manifest_digest.as_bytes(),
    )?;
    for (member, bytes) in &bundle_bytes {
        append(&mut builder, member, bytes)?;
    }
    append(&mut builder, IDENTITY_MEMBER, &identity_bytes)?;
    builder
        .finish()
        .map_err(|e| BackupError::Io(e.to_string()))?;

    Ok(BackupSummary {
        created_at,
        backup_format_version: BACKUP_FORMAT_VERSION,
        scope_count: scopes.len(),
        member_count,
    })
}

/// Map an identity error into a backup error.
fn idn(e: crate::identity::IdentityError) -> BackupError {
    BackupError::Identity(e.to_string())
}

// --- verify ----------------------------------------------------------------

/// Read every member of the backup tar at `input` into name → bytes.
fn read_members(input: &Path) -> Result<std::collections::BTreeMap<String, Vec<u8>>, BackupError> {
    use std::io::Read;
    let file = std::fs::File::open(input).map_err(|e| BackupError::Io(e.to_string()))?;
    let mut archive = tar::Archive::new(file);
    let mut members = std::collections::BTreeMap::new();
    for entry in archive
        .entries()
        .map_err(|e| BackupError::Io(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| BackupError::Io(e.to_string()))?;
        let name = entry
            .path()
            .map_err(|e| BackupError::Io(e.to_string()))?
            .to_string_lossy()
            .to_string();
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
fn load_verified(
    input: &Path,
) -> Result<(Manifest, std::collections::BTreeMap<String, Vec<u8>>), BackupError> {
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
    if manifest.backup_format_version != BACKUP_FORMAT_VERSION {
        return Err(BackupError::Integrity(format!(
            "incompatible backup format version {} (this build reads {})",
            manifest.backup_format_version, BACKUP_FORMAT_VERSION
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

    // Every bundle must parse as an export bundle (structure check).
    for scope in &manifest.scopes {
        let bytes = members
            .get(&scope.member)
            .ok_or_else(|| BackupError::Integrity(format!("missing bundle {}", scope.member)))?;
        serde_json::from_slice::<BundleDto>(bytes).map_err(|e| {
            BackupError::Integrity(format!("unparseable bundle {}: {e}", scope.member))
        })?;
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

/// Restore a backup into the empty target the config declares (决策 3), then
/// validate. Order: integrity verify → empty-target guard → identity snapshot →
/// per-scope import → validation. A non-empty target or an integrity failure is
/// refused before any write.
pub fn restore(config: &ServerConfig, input: &Path) -> Result<ValidationReport, BackupError> {
    // 决策 3: the integrity verification is restore's first step.
    let (manifest, members) = load_verified(input)?;

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
    {
        let store =
            crate::build_store(&config.store).map_err(|e| BackupError::Store(e.to_string()))?;
        let mut existing = Vec::new();
        for entry in &manifest.scopes {
            let scope = Scope::new(ProjectId::new(&entry.project), RepoId::new(&entry.repo));
            let bundle = store
                .export(&scope)
                .map_err(|e| BackupError::Store(e.to_string()))?;
            if !bundle.documents.is_empty() {
                existing.push(format!(
                    "scope {}/{} holds {} document(s)",
                    entry.project,
                    entry.repo,
                    bundle.documents.len()
                ));
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

    // Per-scope import into the target store, in fresh-create mode.
    {
        let store =
            crate::build_store(&config.store).map_err(|e| BackupError::Store(e.to_string()))?;
        for entry in &manifest.scopes {
            let bytes = members.get(&entry.member).ok_or_else(|| {
                BackupError::Integrity(format!("missing bundle {}", entry.member))
            })?;
            let dto: BundleDto = serde_json::from_slice(bytes).map_err(|e| {
                BackupError::Integrity(format!("unparseable bundle {}: {e}", entry.member))
            })?;
            store
                .import(dto.into_bundle(), ImportMode::CreateNew)
                .map_err(|e| BackupError::Store(e.to_string()))?;
        }
    }

    validate(config, input)
}

/// Re-read the restored target and compare it item-for-item against the backup:
/// per-scope content digests and document count, identity counts and schema
/// version against the manifest. Separable so it can be re-run after a restore.
pub fn validate(config: &ServerConfig, input: &Path) -> Result<ValidationReport, BackupError> {
    let (manifest, members) = load_verified(input)?;
    let mut differences = Vec::new();

    // Per-scope: the target's re-exported content digest multiset and document
    // count must match the backup's bundle.
    let store = crate::build_store(&config.store).map_err(|e| BackupError::Store(e.to_string()))?;
    for entry in &manifest.scopes {
        let scope = Scope::new(ProjectId::new(&entry.project), RepoId::new(&entry.repo));
        let target = store
            .export(&scope)
            .map_err(|e| BackupError::Store(e.to_string()))?;
        if target.documents.len() != entry.doc_count {
            differences.push(format!(
                "scope {}/{}: {} document(s), expected {}",
                entry.project,
                entry.repo,
                target.documents.len(),
                entry.doc_count
            ));
            continue;
        }
        let bytes = members
            .get(&entry.member)
            .ok_or_else(|| BackupError::Integrity(format!("missing bundle {}", entry.member)))?;
        let expected: BundleDto = serde_json::from_slice(bytes).map_err(|e| {
            BackupError::Integrity(format!("unparseable bundle {}: {e}", entry.member))
        })?;
        if digest_multiset(target.documents.iter().map(|d| d.digest.as_str()))
            != digest_multiset(expected.documents.iter().map(|d| d.digest.as_str()))
        {
            differences.push(format!(
                "scope {}/{}: content digests differ from the backup",
                entry.project, entry.repo
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

/// A sorted multiset of digests, for order-independent set comparison.
fn digest_multiset<'a>(digests: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut v: Vec<&str> = digests.collect();
    v.sort_unstable();
    v
}

/// Export one scope as the same self-verifying bundle JSON a backup member holds
/// (决策 5) — the admin scope-download reuses the backup export shape, so a
/// downloaded bundle passes the same structure and digest verification.
pub fn export_bundle_json(store: &dyn TeamStore, scope: &Scope) -> Result<Vec<u8>, BackupError> {
    let bundle = store
        .export(scope)
        .map_err(|e| BackupError::Store(e.to_string()))?;
    serde_json::to_vec_pretty(&BundleDto::from_bundle(&bundle))
        .map_err(|e| BackupError::Store(format!("serialize bundle: {e}")))
}
