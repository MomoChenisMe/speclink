//! Contract value types: scoped logical addressing (no PathBuf identity),
//! revisions, manifest/capabilities, history records, event records, outbox,
//! and the versioned export bundle.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Version of the TeamStore contract this crate defines. Drivers report the
/// version they implement through [`Manifest::contract_version`]; semantic
/// evolution happens by bumping this, never by silently changing meaning.
pub const CONTRACT_VERSION: u32 = 1;

/// Version of the export bundle format (independent of the contract version
/// so bundles can evolve without a contract bump).
pub const BUNDLE_FORMAT_VERSION: u32 = 1;

/// Tenant identity: the project a document belongs to. Local single-machine
/// deployments map to a fixed default project.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The repository within a project a document belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoId(String);

impl RepoId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The addressing scope for reads, writes, snapshots, export and outbox:
/// one project/repo pair. Cross-scope access is isolated by contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub project: ProjectId,
    pub repo: RepoId,
}

impl Scope {
    pub fn new(project: ProjectId, repo: RepoId) -> Self {
        Self { project, repo }
    }
}

/// Logical document identity — the closed set of domain document kinds.
/// Deliberately not a path: physical location is a driver concern and never
/// crosses this boundary as identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentId {
    /// Metadata document of an active change.
    ChangeMeta { change: String },
    /// An artifact of an active change, by its schema-defined relative name
    /// (e.g. `proposal.md`, `specs/auth/spec.md`).
    ChangeArtifact { change: String, artifact: String },
    /// The canonical spec of a capability.
    CanonicalSpec { capability: String },
    /// A discussion document, live or archived, by slug.
    Discussion { slug: String, archived: bool },
    /// The workflow configuration document of the scope.
    WorkflowConfig,
    /// A document of an archived change, by its relative name.
    ArchivedChange { change: String, doc: String },
    /// The scope's shared-vocabulary document (`LANGUAGE.md`): one per scope,
    /// like [`DocumentId::WorkflowConfig`]. Absent is a normal state.
    Language,
    /// The scope's shared board-order document: one per scope, like
    /// [`DocumentId::WorkflowConfig`]. Opaque presentation content the store
    /// never interprets. Absent is a normal state.
    BoardOrder,
}

/// Full document identity: the (project, repo, document) triple.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocRef {
    pub project: ProjectId,
    pub repo: RepoId,
    pub doc: DocumentId,
}

impl DocRef {
    pub fn new(scope: &Scope, doc: DocumentId) -> Self {
        Self {
            project: scope.project.clone(),
            repo: scope.repo.clone(),
            doc,
        }
    }
}

/// A monotonically increasing project revision. Every committed unit of work
/// advances it by one; a document's revision is the project revision at
/// which it was last written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(pub u64);

/// The CAS precondition a staged write carries: either "the document must
/// not exist yet" (creation) or "the document must still be at the revision
/// I read" (update/delete).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedRevision {
    /// Creation semantics: commit fails with a revision conflict if the
    /// document already exists.
    Absent,
    /// Update semantics: commit fails with a revision conflict unless the
    /// document's current revision equals this one.
    At(Revision),
}

/// A capability a driver declares in its manifest. Closed set: conformance
/// maps each declared capability to tests, so an open string set would make
/// driver behavior unverifiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    Snapshot,
    Cas,
    Transaction,
    History,
    Outbox,
    Migration,
    Backup,
    Cluster,
}

impl Capability {
    /// Stable name, used in manifests and conformance reports.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Snapshot => "snapshot",
            Capability::Cas => "cas",
            Capability::Transaction => "transaction",
            Capability::History => "history",
            Capability::Outbox => "outbox",
            Capability::Migration => "migration",
            Capability::Backup => "backup",
            Capability::Cluster => "cluster",
        }
    }
}

/// The declared capability level. Ordered: declaring a level promises the
/// guarantees of every level below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityLevel {
    LocalSingleWriter,
    SingleNode,
    Cluster,
}

impl CapabilityLevel {
    /// Stable name, used in manifests and conformance reports.
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityLevel::LocalSingleWriter => "local-single-writer",
            CapabilityLevel::SingleNode => "single-node",
            CapabilityLevel::Cluster => "cluster",
        }
    }
}

/// What a driver declares about itself. Read programmatically by host
/// startup validation and by the conformance suite to pick the test set —
/// "declared means owed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub contract_version: u32,
    /// Driver identity (e.g. "memory", "sqlite").
    pub driver: String,
    pub level: CapabilityLevel,
    pub capabilities: BTreeSet<Capability>,
}

/// A document as read from a snapshot: content plus the revision at which
/// it was last written (the CAS token for subsequent updates).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub content: String,
    pub revision: Revision,
}

/// What a history revision did to the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionKind {
    /// Creation or modification; carries the digest of the written content.
    Write { digest: String },
    /// Deletion. History is never rewritten — a delete appends this marker.
    Tombstone,
}

/// One immutable history entry of a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionRecord {
    pub revision: Revision,
    pub actor: String,
    pub at: DateTime<Utc>,
    /// Identity of the command that produced this revision.
    pub command: String,
    pub kind: RevisionKind,
}

/// A domain event as the store sees it: a record to persist, not domain
/// logic. Canonical event semantics stay in the engine; the store only
/// guarantees atomic persistence alongside the commit.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    pub name: String,
    pub payload: serde_json::Value,
    pub actor: String,
    pub at: DateTime<Utc>,
}

/// Position in a scope's outbox. `OutboxCursor(0)` is the beginning; a
/// cursor value `n` means "everything up to and including sequence `n` has
/// been consumed".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutboxCursor(pub u64);

/// One persisted outbox entry: replayable order (`seq`), the project
/// revision of the commit that produced it, and the event record itself.
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEntry {
    pub seq: u64,
    pub revision: Revision,
    pub record: EventRecord,
}

/// One document inside an export bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleDoc {
    pub doc: DocumentId,
    pub content: String,
    /// Digest of `content`, computed with [`content_digest`]. Import
    /// verifies it and rejects the whole bundle on mismatch.
    pub digest: String,
}

/// A versioned export of one scope at one project revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    pub format_version: u32,
    pub scope: Scope,
    pub project_revision: Revision,
    pub documents: Vec<BundleDoc>,
}

/// How import applies a bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// The target scope must hold no documents; import creates everything.
    CreateNew,
    /// Existing documents in the target scope are replaced.
    Overwrite,
}

/// Per-document import outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOutcome {
    Created,
    Overwritten,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedDoc {
    pub doc: DocumentId,
    pub outcome: ImportOutcome,
}

/// Result of a successful import: the project revision the scope ended up
/// at and the per-document outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub project_revision: Revision,
    pub documents: Vec<ImportedDoc>,
}

/// Contract-defined commit stage boundaries. Fault-injecting harnesses (the
/// in-memory reference, driver crash fixtures) crash a commit at one of
/// these points; the atomicity guarantee is judged at these boundaries, not
/// at implementation details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultPoint {
    AfterDocWrites,
    AfterHistoryAppend,
    BeforeOutboxAppend,
    AfterOutboxAppend,
}

/// The contract-defined content digest: SHA-256, lowercase hex, prefixed
/// with the algorithm. Fixed by the contract so bundles and history digests
/// agree across drivers.
pub fn content_digest(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_id_covers_eight_logical_kinds() {
        let kinds = [
            DocumentId::ChangeMeta {
                change: "add-auth".into(),
            },
            DocumentId::ChangeArtifact {
                change: "add-auth".into(),
                artifact: "specs/auth/spec.md".into(),
            },
            DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
            DocumentId::Discussion {
                slug: "auth-scope".into(),
                archived: false,
            },
            DocumentId::WorkflowConfig,
            DocumentId::ArchivedChange {
                change: "old-change".into(),
                doc: "proposal.md".into(),
            },
            DocumentId::Language,
            DocumentId::BoardOrder,
        ];
        assert_eq!(kinds.len(), 8);
        // Closed enum: exhaustive match without a wildcard arm — a new kind
        // breaks compilation here instead of silently passing drivers by.
        for kind in &kinds {
            match kind {
                DocumentId::ChangeMeta { .. } => {}
                DocumentId::ChangeArtifact { .. } => {}
                DocumentId::CanonicalSpec { .. } => {}
                DocumentId::Discussion { .. } => {}
                DocumentId::WorkflowConfig => {}
                DocumentId::ArchivedChange { .. } => {}
                DocumentId::Language => {}
                DocumentId::BoardOrder => {}
            }
        }
    }

    #[test]
    fn doc_ref_is_a_project_repo_document_triple() {
        let doc_ref = DocRef {
            project: ProjectId::new("acme"),
            repo: RepoId::new("web"),
            doc: DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
        };
        assert_eq!(doc_ref.project.as_str(), "acme");
        assert_eq!(doc_ref.repo.as_str(), "web");
        assert_eq!(
            doc_ref.doc,
            DocumentId::CanonicalSpec {
                capability: "auth".into()
            }
        );
    }

    #[test]
    fn manifest_carries_contract_version_capabilities_and_level() {
        let manifest = Manifest {
            contract_version: CONTRACT_VERSION,
            driver: "memory".into(),
            level: CapabilityLevel::SingleNode,
            capabilities: [
                Capability::Snapshot,
                Capability::Cas,
                Capability::Transaction,
                Capability::History,
                Capability::Outbox,
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(manifest.contract_version, CONTRACT_VERSION);
        assert_eq!(manifest.level, CapabilityLevel::SingleNode);
        assert!(manifest.capabilities.contains(&Capability::Outbox));
        assert!(!manifest.capabilities.contains(&Capability::Cluster));
    }

    #[test]
    fn capability_set_is_closed_with_eight_members() {
        let all = [
            Capability::Snapshot,
            Capability::Cas,
            Capability::Transaction,
            Capability::History,
            Capability::Outbox,
            Capability::Migration,
            Capability::Backup,
            Capability::Cluster,
        ];
        for capability in &all {
            match capability {
                Capability::Snapshot => {}
                Capability::Cas => {}
                Capability::Transaction => {}
                Capability::History => {}
                Capability::Outbox => {}
                Capability::Migration => {}
                Capability::Backup => {}
                Capability::Cluster => {}
            }
        }
        let unique: std::collections::BTreeSet<_> = all.into_iter().collect();
        assert_eq!(unique.len(), 8);
    }

    #[test]
    fn capability_levels_are_three_and_ordered() {
        assert!(CapabilityLevel::LocalSingleWriter < CapabilityLevel::SingleNode);
        assert!(CapabilityLevel::SingleNode < CapabilityLevel::Cluster);
        let all = [
            CapabilityLevel::LocalSingleWriter,
            CapabilityLevel::SingleNode,
            CapabilityLevel::Cluster,
        ];
        for level in &all {
            match level {
                CapabilityLevel::LocalSingleWriter => {}
                CapabilityLevel::SingleNode => {}
                CapabilityLevel::Cluster => {}
            }
        }
    }

    #[test]
    fn capability_and_level_names_are_stable() {
        assert_eq!(Capability::Outbox.as_str(), "outbox");
        assert_eq!(Capability::Cas.as_str(), "cas");
        assert_eq!(CapabilityLevel::LocalSingleWriter.as_str(), "local-single-writer");
        assert_eq!(CapabilityLevel::SingleNode.as_str(), "single-node");
        assert_eq!(CapabilityLevel::Cluster.as_str(), "cluster");
    }

    #[test]
    fn content_digest_is_stable_and_contract_defined() {
        // The digest algorithm is fixed by the contract so bundles and
        // history digests agree across drivers: sha256, lowercase hex.
        assert_eq!(content_digest("abc"), content_digest("abc"));
        assert_ne!(content_digest("abc"), content_digest("abd"));
        assert_eq!(
            content_digest("abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn expected_revision_distinguishes_create_from_cas() {
        // Creation carries "must not already exist"; updates carry the
        // revision the writer read. The two must be distinct values.
        assert_ne!(ExpectedRevision::Absent, ExpectedRevision::At(Revision(0)));
        match ExpectedRevision::At(Revision(7)) {
            ExpectedRevision::Absent => panic!("At(7) is not Absent"),
            ExpectedRevision::At(revision) => assert_eq!(revision, Revision(7)),
        }
    }
}
