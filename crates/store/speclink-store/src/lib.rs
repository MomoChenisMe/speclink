//! The TeamStore contract: the single Rust definition of the team-mode
//! storage boundary — typed errors, Project/Repo-scoped logical addressing,
//! consistent snapshots, Unit-of-Work/CAS write semantics, immutable history,
//! the transactional outbox, versioned export/import bundles, and the
//! reusable conformance suite every driver must pass.
//!
//! This crate has zero dependency on `speclink-core`; the existing local
//! `Store` seam is untouched. No product driver lives here — only the
//! contract, an in-memory reference implementation, and the conformance
//! entry point.
//!
//! The trait is synchronous and object-safe on purpose, matching the
//! engine's no-async-runtime stance: single-node drivers are naturally
//! synchronous, and async hosts adapt at their own boundary
//! (`spawn_blocking` or equivalent).

pub mod conformance;
pub mod error;
pub mod memory;
pub mod types;
pub mod uow;

pub use error::StoreError;
pub use types::{
    content_digest, Bundle, BundleDoc, Capability, CapabilityLevel, DocRef, Document, DocumentId,
    EventRecord, ExpectedRevision, FaultPoint, ImportMode, ImportOutcome, ImportReport,
    ImportedDoc, Manifest, OutboxCursor, OutboxEntry, ProjectId, RepoId, Revision, RevisionKind,
    RevisionRecord, Scope, BUNDLE_FORMAT_VERSION, CONTRACT_VERSION,
};
pub use uow::{CommandContext, StagedOp, UnitOfWork};

/// A consistent, fixed-point view of one scope, bound to a single project
/// revision. Reads through a snapshot are unaffected by commits that land
/// after it was taken.
pub trait Snapshot {
    /// The project revision this view is bound to. Never changes for the
    /// lifetime of the snapshot.
    fn revision(&self) -> Revision;

    /// Read one document of the snapshot's scope. `Ok(None)` is the normal
    /// "does not exist (or is deleted)" case; failures — including corrupt
    /// persisted content — travel as `Err` and are never flattened into
    /// `None`.
    fn read(&self, doc: &DocumentId) -> Result<Option<Document>, StoreError>;

    /// The immutable history of one document as of this snapshot, oldest
    /// first. A document with no history yields `Ok` with an empty list.
    fn history(&self, doc: &DocumentId) -> Result<Vec<RevisionRecord>, StoreError>;
}

/// The TeamStore contract. Synchronous and object-safe; see the crate docs
/// for the stance. All reads go through [`TeamStore::snapshot`]; all writes
/// go through a [`UnitOfWork`] and take effect atomically at
/// [`TeamStore::commit`].
pub trait TeamStore {
    /// What this driver declares: contract version, identity, capability
    /// level and capability set. Conformance and host startup validation
    /// read this — "declared means owed".
    fn manifest(&self) -> Manifest;

    /// Liveness/readiness of the backend. `Ok(())` means the store can
    /// serve requests now.
    fn health(&self) -> Result<(), StoreError>;

    /// Bring the backend's persisted schema to `target_version`.
    fn migrate(&self, target_version: u32) -> Result<(), StoreError>;

    /// A consistent view of `scope` at the current project revision.
    fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError>;

    /// Open the only write path: a unit of work in `scope` on behalf of
    /// `ctx`. Staged operations take effect only at [`TeamStore::commit`].
    fn begin_unit_of_work(
        &self,
        scope: &Scope,
        ctx: CommandContext,
    ) -> Result<UnitOfWork, StoreError>;

    /// Atomically apply a unit of work: all document writes, the project
    /// revision increment, per-document history appends, and the outbox
    /// append of `events` — all take effect or none do. Any CAS mismatch
    /// rejects the whole commit with a revision conflict naming the
    /// document, expected and actual. Returns the new project revision.
    fn commit(&self, uow: UnitOfWork, events: Vec<EventRecord>) -> Result<Revision, StoreError>;

    /// Discard a unit of work; staged operations leave no trace.
    fn rollback(&self, uow: UnitOfWork) -> Result<(), StoreError>;

    /// Export `scope` as a versioned bundle with per-document digests.
    fn export(&self, scope: &Scope) -> Result<Bundle, StoreError>;

    /// Verify and apply a bundle in the given mode. Verification failure
    /// (format version, digest) rejects the whole import — nothing is
    /// partially applied.
    fn import(&self, bundle: Bundle, mode: ImportMode) -> Result<ImportReport, StoreError>;

    /// Read the scope's outbox entries with sequence greater than `from`,
    /// in replayable order. `OutboxCursor(0)` replays from the beginning.
    fn read_outbox(
        &self,
        scope: &Scope,
        from: OutboxCursor,
    ) -> Result<Vec<OutboxEntry>, StoreError>;

    /// Confirm consumption up to and including `up_to`. The position is
    /// durable and monotonic; acknowledging backwards is a no-op.
    /// Acknowledging past the newest entry is a failure — accepting it
    /// would silently skip everything committed later.
    fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError>;

    /// The scope's durable consumer position, as last acknowledged.
    /// Reading from this cursor never repeats confirmed entries.
    fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time proof that both traits stay object-safe: the contract is
    // consumed as `&dyn TeamStore` by the conformance suite and hosts.
    #[allow(dead_code)]
    fn assert_object_safe(_store: &dyn TeamStore, _snapshot: &dyn Snapshot) {}
}
