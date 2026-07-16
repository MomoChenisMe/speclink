//! In-memory reference implementation of the TeamStore contract — test
//! infrastructure proving the contract is implementable, not a product
//! driver. Deliberately minimal so memory-only conveniences never leak into
//! contract semantics.

use crate::error::StoreError;
use crate::types::{
    content_digest, Bundle, BundleDoc, Capability, CapabilityLevel, DocRef, Document, DocumentId,
    EventRecord, ExpectedRevision, FaultPoint, ImportMode, ImportReport, Manifest, OutboxCursor,
    OutboxEntry, Revision, RevisionKind, RevisionRecord, Scope, BUNDLE_FORMAT_VERSION,
    CONTRACT_VERSION,
};
use crate::uow::{CommandContext, StagedOp, UnitOfWork};
use crate::{Snapshot, TeamStore};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// (project, repo) key of a scope.
type ScopeKey = (String, String);

fn scope_key(scope: &Scope) -> ScopeKey {
    (
        scope.project.as_str().to_string(),
        scope.repo.as_str().to_string(),
    )
}

/// A document as persisted: content, the revision it was written at, and an
/// optional injected corruption (test hook — reads must surface it as a
/// corrupt failure, never as absence).
#[derive(Debug, Clone)]
struct DocCell {
    content: String,
    revision: Revision,
    corrupt: Option<String>,
}

/// Everything one commit did, as journaled. Replaying committed records in
/// order reconstructs every derived table exactly (same revisions, same
/// timestamps, same outbox sequence numbers).
#[derive(Debug, Clone)]
struct CommitRecord {
    scope: Scope,
    ctx: CommandContext,
    at: chrono::DateTime<chrono::Utc>,
    revision: Revision,
    ops: Vec<StagedOp>,
    events: Vec<EventRecord>,
}

/// A journal entry: the intent, plus the commit marker that is the one and
/// only atomic point of a commit. A crash anywhere before the marker leaves
/// an uncommitted intent that recovery discards.
#[derive(Debug, Clone)]
struct JournalEntry {
    record: CommitRecord,
    committed: bool,
}

#[derive(Debug, Default)]
struct Inner {
    // -- durable state (what a rebuild starts from) --
    journal: Vec<JournalEntry>,
    /// Durable outbox consumer position per scope.
    acked: BTreeMap<ScopeKey, u64>,

    // -- derived tables (rebuilt from the journal on recovery) --
    /// Documents per scope (live only — a delete removes the cell and leaves
    /// a tombstone in history). Tenant isolation falls out of the keying: a
    /// scope's snapshot only ever sees its own map.
    docs: BTreeMap<ScopeKey, BTreeMap<DocumentId, DocCell>>,
    /// Immutable per-document history, append-only.
    history: BTreeMap<ScopeKey, BTreeMap<DocumentId, Vec<RevisionRecord>>>,
    /// Monotonic project revision, per project.
    project_revisions: BTreeMap<String, u64>,
    /// Persisted outbox entries per scope, in sequence order.
    outbox: BTreeMap<ScopeKey, Vec<OutboxEntry>>,

    // -- test hooks --
    /// Crash the next commit at this stage boundary.
    pending_crash: Option<FaultPoint>,
    /// Make the next commit's outbox append fail (an error, not a crash).
    pending_outbox_failure: bool,
    /// A crashed store serves nothing until rebuilt.
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

    /// Consume a pending crash injection at this stage boundary. A crashed
    /// store stops serving until rebuilt; partial table mutations stay in
    /// place, exactly like a torn write on real media.
    fn crash_if_injected(&mut self, point: FaultPoint) -> Result<(), StoreError> {
        if self.pending_crash == Some(point) {
            self.pending_crash = None;
            self.crashed = true;
            return Err(StoreError::Unavailable);
        }
        Ok(())
    }

    /// Apply one commit's document writes to the doc table.
    fn apply_docs(&mut self, record: &CommitRecord) {
        let docs = self.docs.entry(scope_key(&record.scope)).or_default();
        for op in &record.ops {
            match op {
                StagedOp::Put { doc, content, .. } => {
                    docs.insert(
                        doc.clone(),
                        DocCell {
                            content: content.clone(),
                            revision: record.revision,
                            corrupt: None,
                        },
                    );
                }
                StagedOp::Delete { doc, .. } => {
                    docs.remove(doc);
                }
            }
        }
    }

    /// Append one commit's per-document history records.
    fn apply_history(&mut self, record: &CommitRecord) {
        let history = self.history.entry(scope_key(&record.scope)).or_default();
        for op in &record.ops {
            let kind = match op {
                StagedOp::Put { content, .. } => RevisionKind::Write {
                    digest: content_digest(content),
                },
                StagedOp::Delete { .. } => RevisionKind::Tombstone,
            };
            history.entry(op.doc().clone()).or_default().push(RevisionRecord {
                revision: record.revision,
                actor: record.ctx.actor.clone(),
                at: record.at,
                command: record.ctx.command.clone(),
                kind,
            });
        }
    }

    /// Append one commit's event records to the scope's outbox.
    fn apply_outbox(&mut self, record: &CommitRecord) {
        let outbox = self.outbox.entry(scope_key(&record.scope)).or_default();
        for event in &record.events {
            let seq = outbox.len() as u64 + 1;
            outbox.push(OutboxEntry {
                seq,
                revision: record.revision,
                record: event.clone(),
            });
        }
    }

    /// Advance the project revision to this commit's.
    fn apply_revision(&mut self, record: &CommitRecord) {
        self.project_revisions
            .insert(record.scope.project.as_str().to_string(), record.revision.0);
    }

    /// Rebuild every derived table from the committed journal — the
    /// recovery procedure. Partial mutations of an uncommitted intent
    /// vanish because replay starts from empty tables.
    fn replay_journal(&mut self) {
        self.docs.clear();
        self.history.clear();
        self.project_revisions.clear();
        self.outbox.clear();
        let committed: Vec<CommitRecord> = self
            .journal
            .iter()
            .filter(|entry| entry.committed)
            .map(|entry| entry.record.clone())
            .collect();
        for record in &committed {
            self.apply_docs(record);
            self.apply_history(record);
            self.apply_outbox(record);
            self.apply_revision(record);
        }
    }
}

/// The in-memory reference store. Interior mutability behind one mutex —
/// commits serialize, which is exactly the single-node guarantee the
/// contract asks for.
#[derive(Debug, Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("memory store mutex poisoned")
    }

    /// Test hook: mark a persisted document as corrupt so reads surface a
    /// typed corrupt failure with this reason.
    pub fn corrupt_document(&self, scope: &Scope, doc: &DocumentId, reason: &str) {
        let mut inner = self.lock();
        if let Some(cell) = inner
            .docs
            .get_mut(&scope_key(scope))
            .and_then(|docs| docs.get_mut(doc))
        {
            cell.corrupt = Some(reason.to_string());
        }
    }

    /// Fault injection: crash the next commit at the given stage boundary.
    /// The store then serves nothing until [`MemoryStore::rebuild`].
    pub fn crash_at(&self, point: FaultPoint) {
        self.lock().pending_crash = Some(point);
    }

    /// Fault injection: make the next commit's outbox append fail with a
    /// backend error (a failure the commit must absorb, not a crash).
    pub fn fail_outbox_append(&self) {
        self.lock().pending_outbox_failure = true;
    }

    /// Restart semantics: a new store built from this one's durable state
    /// (journal and consumer positions). Uncommitted partial effects do not
    /// survive — the invariant crash-recovery conformance asserts.
    pub fn rebuild(&self) -> MemoryStore {
        let inner = self.lock();
        let mut fresh = Inner {
            journal: inner
                .journal
                .iter()
                .filter(|entry| entry.committed)
                .cloned()
                .collect(),
            acked: inner.acked.clone(),
            ..Inner::default()
        };
        fresh.replay_journal();
        MemoryStore {
            inner: Mutex::new(fresh),
        }
    }
}

/// The conformance harness of the in-memory reference: wires the store's
/// fault-injection hooks to the suite's [`crate::conformance::StoreHarness`]
/// interface. Running the full suite against it is the contract's
/// implementability proof.
#[derive(Debug, Default)]
pub struct MemoryHarness {
    store: MemoryStore,
}

impl MemoryHarness {
    pub fn new() -> Self {
        Self::default()
    }
}

impl crate::conformance::StoreHarness for MemoryHarness {
    fn reset(&mut self) -> &dyn TeamStore {
        self.store = MemoryStore::new();
        &self.store
    }

    fn store(&self) -> &dyn TeamStore {
        &self.store
    }

    fn arm_crash(&mut self, point: FaultPoint) {
        self.store.crash_at(point);
    }

    fn arm_outbox_failure(&mut self) {
        self.store.fail_outbox_append();
    }

    fn restart(&mut self) {
        self.store = self.store.rebuild();
    }
}

/// A fixed-point view: owns a clone of the scope's documents and history at
/// snapshot time, so later commits cannot reach into it.
struct MemorySnapshot {
    revision: Revision,
    docs: BTreeMap<DocumentId, DocCell>,
    history: BTreeMap<DocumentId, Vec<RevisionRecord>>,
}

impl Snapshot for MemorySnapshot {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn read(&self, doc: &DocumentId) -> Result<Option<Document>, StoreError> {
        match self.docs.get(doc) {
            None => Ok(None),
            Some(cell) => match &cell.corrupt {
                Some(reason) => Err(StoreError::Corrupt {
                    reason: reason.clone(),
                }),
                None => Ok(Some(Document {
                    content: cell.content.clone(),
                    revision: cell.revision,
                })),
            },
        }
    }

    fn history(&self, doc: &DocumentId) -> Result<Vec<RevisionRecord>, StoreError> {
        Ok(self.history.get(doc).cloned().unwrap_or_default())
    }
}

impl TeamStore for MemoryStore {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: CONTRACT_VERSION,
            driver: "memory".into(),
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
        self.lock().unavailable_if_crashed()
    }

    fn migrate(&self, target_version: u32) -> Result<(), StoreError> {
        self.lock().unavailable_if_crashed()?;
        if target_version == CONTRACT_VERSION {
            Ok(())
        } else {
            Err(StoreError::Backend {
                source: format!("unknown schema version {target_version}"),
            })
        }
    }

    fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let revision = Revision(
            inner
                .project_revisions
                .get(scope.project.as_str())
                .copied()
                .unwrap_or(0),
        );
        let docs = inner.docs.get(&scope_key(scope)).cloned().unwrap_or_default();
        let history = inner
            .history
            .get(&scope_key(scope))
            .cloned()
            .unwrap_or_default();
        Ok(Box::new(MemorySnapshot {
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
        let project = uow.scope().project.as_str().to_string();
        let key = scope_key(uow.scope());
        let next = Revision(inner.project_revisions.get(&project).copied().unwrap_or(0) + 1);

        // Validate every CAS precondition against the pre-commit state
        // before touching anything: any mismatch rejects the whole commit.
        for op in uow.ops() {
            let current = inner
                .docs
                .get(&key)
                .and_then(|docs| docs.get(op.doc()))
                .map(|cell| cell.revision);
            let expected = match op {
                StagedOp::Put { expected, .. } => *expected,
                StagedOp::Delete { expected, .. } => ExpectedRevision::At(*expected),
            };
            let satisfied = match expected {
                ExpectedRevision::Absent => current.is_none(),
                ExpectedRevision::At(revision) => current == Some(revision),
            };
            if !satisfied {
                return Err(StoreError::RevisionConflict {
                    doc: DocRef::new(uow.scope(), op.doc().clone()),
                    expected,
                    actual: current,
                });
            }
        }

        // Journal the intent, then apply stage by stage. The commit marker
        // at the end is the one atomic point: a crash at any boundary in
        // between leaves an uncommitted intent that recovery discards.
        let record = CommitRecord {
            scope: uow.scope().clone(),
            ctx: uow.context().clone(),
            at: chrono::Utc::now(),
            revision: next,
            ops: uow.ops().to_vec(),
            events,
        };
        inner.journal.push(JournalEntry {
            record: record.clone(),
            committed: false,
        });

        inner.apply_docs(&record);
        inner.crash_if_injected(FaultPoint::AfterDocWrites)?;
        inner.apply_history(&record);
        inner.crash_if_injected(FaultPoint::AfterHistoryAppend)?;
        inner.crash_if_injected(FaultPoint::BeforeOutboxAppend)?;
        if inner.pending_outbox_failure {
            // The append failed but the process lives: absorb the failure by
            // discarding the intent and restoring the derived tables — the
            // whole commit never happened.
            inner.pending_outbox_failure = false;
            inner.journal.pop();
            inner.replay_journal();
            return Err(StoreError::Backend {
                source: "outbox append failed".into(),
            });
        }
        inner.apply_outbox(&record);
        inner.crash_if_injected(FaultPoint::AfterOutboxAppend)?;
        inner.apply_revision(&record);
        inner
            .journal
            .last_mut()
            .expect("intent journaled above")
            .committed = true;
        Ok(next)
    }

    fn rollback(&self, uow: UnitOfWork) -> Result<(), StoreError> {
        drop(uow);
        Ok(())
    }

    fn export(&self, scope: &Scope) -> Result<Bundle, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        let mut documents = Vec::new();
        if let Some(docs) = inner.docs.get(&scope_key(scope)) {
            for (doc, cell) in docs {
                if let Some(reason) = &cell.corrupt {
                    return Err(StoreError::Corrupt {
                        reason: reason.clone(),
                    });
                }
                documents.push(BundleDoc {
                    doc: doc.clone(),
                    content: cell.content.clone(),
                    digest: content_digest(&cell.content),
                });
            }
        }
        Ok(Bundle {
            format_version: BUNDLE_FORMAT_VERSION,
            scope: scope.clone(),
            project_revision: Revision(
                inner
                    .project_revisions
                    .get(scope.project.as_str())
                    .copied()
                    .unwrap_or(0),
            ),
            documents,
        })
    }

    fn import(&self, bundle: Bundle, mode: ImportMode) -> Result<ImportReport, StoreError> {
        // Verify everything before applying anything: a rejected bundle
        // leaves the store untouched.
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
        let key = scope_key(&bundle.scope);
        let existing: Vec<Option<Revision>> = bundle
            .documents
            .iter()
            .map(|doc| {
                inner
                    .docs
                    .get(&key)
                    .and_then(|docs| docs.get(&doc.doc))
                    .map(|cell| cell.revision)
            })
            .collect();
        // Create-new is gated on the whole scope, not on the bundle's own
        // documents: anything already there rejects the import.
        let scope_holds_any_document = inner.docs.get(&key).is_some_and(|docs| !docs.is_empty());
        if mode == ImportMode::CreateNew && scope_holds_any_document {
            return Err(StoreError::Backend {
                source: "import (create-new): target scope already holds documents".into(),
            });
        }

        // Apply as one committed journal record so a rebuild reproduces the
        // import exactly, and history starts (or continues) at it.
        let project = bundle.scope.project.as_str().to_string();
        let next = Revision(inner.project_revisions.get(&project).copied().unwrap_or(0) + 1);
        let mut ops = Vec::new();
        let mut documents = Vec::new();
        for (doc, found) in bundle.documents.iter().zip(&existing) {
            ops.push(StagedOp::Put {
                doc: doc.doc.clone(),
                content: doc.content.clone(),
                expected: match found {
                    None => ExpectedRevision::Absent,
                    Some(revision) => ExpectedRevision::At(*revision),
                },
            });
            documents.push(crate::ImportedDoc {
                doc: doc.doc.clone(),
                outcome: match found {
                    None => crate::ImportOutcome::Created,
                    Some(_) => crate::ImportOutcome::Overwritten,
                },
            });
        }
        let record = CommitRecord {
            scope: bundle.scope.clone(),
            ctx: CommandContext {
                command: "import".into(),
                actor: "import".into(),
            },
            at: chrono::Utc::now(),
            revision: next,
            ops,
            events: Vec::new(),
        };
        inner.journal.push(JournalEntry {
            record: record.clone(),
            committed: false,
        });
        inner.apply_docs(&record);
        inner.apply_history(&record);
        inner.apply_outbox(&record);
        inner.apply_revision(&record);
        inner
            .journal
            .last_mut()
            .expect("intent journaled above")
            .committed = true;
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
        Ok(inner
            .outbox
            .get(&scope_key(scope))
            .map(|entries| {
                entries
                    .iter()
                    .filter(|entry| entry.seq > from.0)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError> {
        let mut inner = self.lock();
        inner.unavailable_if_crashed()?;
        let key = scope_key(scope);
        let newest = inner
            .outbox
            .get(&key)
            .and_then(|entries| entries.last())
            .map(|entry| entry.seq)
            .unwrap_or(0);
        if up_to.0 > newest {
            return Err(StoreError::Backend {
                source: format!(
                    "ack cursor {} is beyond the outbox end {newest}",
                    up_to.0
                ),
            });
        }
        let current = inner.acked.get(&key).copied().unwrap_or(0);
        if up_to.0 > current {
            inner.acked.insert(key, up_to.0);
        }
        Ok(())
    }

    fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError> {
        let inner = self.lock();
        inner.unavailable_if_crashed()?;
        Ok(OutboxCursor(
            inner.acked.get(&scope_key(scope)).copied().unwrap_or(0),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::conformance::fixtures::{create, ctx, doc, event, scope, update};
    use crate::{CommandContext, DocumentId, Revision, Scope, StoreError, TeamStore};

    /// Create the given documents in one commit; returns the resulting
    /// project revision.
    fn seed(store: &MemoryStore, scope: &Scope, docs: &[(DocumentId, &str)]) -> Revision {
        create(store, scope, docs, vec![]).unwrap()
    }

    #[test]
    fn snapshot_is_a_fixed_point_view_while_writer_commits() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let seeded_at = seed(
            &store,
            &scope,
            &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        );

        let snap = store.snapshot(&scope).unwrap();
        let revision_before = snap.revision();

        // Writer commits a change to both documents in the view.
        let mut uow = store.begin_unit_of_work(&scope, ctx("edit")).unwrap();
        uow.update(doc("auth"), "auth v2", seeded_at);
        uow.update(doc("billing"), "billing v2", seeded_at);
        store.commit(uow, vec![]).unwrap();

        // The reader keeps seeing the fixed point: both documents old — never
        // one new and one old — and the snapshot revision unchanged.
        let auth = snap.read(&doc("auth")).unwrap().unwrap();
        let billing = snap.read(&doc("billing")).unwrap().unwrap();
        assert_eq!(auth.content, "auth v1");
        assert_eq!(billing.content, "billing v1");
        assert_eq!(snap.revision(), revision_before);

        // Sanity: a fresh snapshot sees the new contents at a later revision.
        let fresh = store.snapshot(&scope).unwrap();
        assert_eq!(fresh.read(&doc("auth")).unwrap().unwrap().content, "auth v2");
        assert!(fresh.revision() > revision_before);
    }

    #[test]
    fn corrupt_content_reads_as_corrupt_while_absent_reads_as_none() {
        let store = MemoryStore::new();
        let scope = scope("main");
        seed(&store, &scope, &[(doc("auth"), "auth v1")]);
        store.corrupt_document(&scope, &doc("auth"), "checksum mismatch");

        let snap = store.snapshot(&scope).unwrap();

        // Corrupt persisted content is a failure, not an absence.
        match snap.read(&doc("auth")) {
            Err(StoreError::Corrupt { reason }) => {
                assert!(reason.contains("checksum mismatch"), "reason: {reason}")
            }
            other => panic!("expected corrupt error, got {other:?}"),
        }

        // The same call distinguishes a truly absent document: Ok(None).
        assert_eq!(snap.read(&doc("does-not-exist")).unwrap(), None);
    }

    #[test]
    fn cross_repo_read_never_returns_other_tenants_content() {
        let store = MemoryStore::new();
        let repo_a = scope("repo-a");
        let repo_b = scope("repo-b");
        // The canonical spec exists only in repo B.
        seed(&store, &repo_b, &[(doc("auth"), "repo-b secret")]);

        let snap_a = store.snapshot(&repo_a).unwrap();
        match snap_a.read(&doc("auth")) {
            Ok(None) => {}
            Err(err) => assert_eq!(err.code(), "permission_denied"),
            Ok(Some(document)) => {
                panic!("leaked repo B content into repo A: {}", document.content)
            }
        }

        // Repo B itself still reads its own document.
        let snap_b = store.snapshot(&repo_b).unwrap();
        assert_eq!(
            snap_b.read(&doc("auth")).unwrap().unwrap().content,
            "repo-b secret"
        );
    }

    #[test]
    fn cas_race_exactly_one_of_two_concurrent_commits_wins() {
        use std::sync::Arc;

        let store = Arc::new(MemoryStore::new());
        let scope = scope("main");
        let seeded_at = seed(&store, &scope, &[(doc("auth"), "base")]);

        // Two units of work race on the same document, both carrying the
        // revision they read.
        let handles: Vec<_> = (0..2)
            .map(|i| {
                let store = Arc::clone(&store);
                let scope = scope.clone();
                std::thread::spawn(move || {
                    let mut uow = store
                        .begin_unit_of_work(&scope, ctx(&format!("edit-{i}")))
                        .unwrap();
                    uow.update(doc("auth"), format!("content-{i}"), seeded_at);
                    store.commit(uow, vec![])
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let winners: Vec<usize> = (0..2).filter(|&i| results[i].is_ok()).collect();
        assert_eq!(winners.len(), 1, "exactly one commit wins: {results:?}");
        let winner = winners[0];
        let winning_revision = *results[winner].as_ref().unwrap();

        // The loser gets a revision conflict naming the document, what it
        // expected, and the revision the winner actually left behind.
        match &results[1 - winner] {
            Err(StoreError::RevisionConflict {
                doc: conflicting,
                expected,
                actual,
            }) => {
                assert_eq!(conflicting.doc, doc("auth"));
                assert_eq!(*expected, crate::ExpectedRevision::At(seeded_at));
                assert_eq!(*actual, Some(winning_revision));
            }
            other => panic!("expected revision conflict, got {other:?}"),
        }

        // Store content equals the winner's write.
        let snap = store.snapshot(&scope).unwrap();
        let document = snap.read(&doc("auth")).unwrap().unwrap();
        assert_eq!(document.content, format!("content-{winner}"));
        assert_eq!(document.revision, winning_revision);
    }

    #[test]
    fn create_requires_the_document_to_be_absent() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let seeded_at = seed(&store, &scope, &[(doc("auth"), "base")]);

        let mut uow = store.begin_unit_of_work(&scope, ctx("recreate")).unwrap();
        uow.create(doc("auth"), "clobbered");
        match store.commit(uow, vec![]) {
            Err(StoreError::RevisionConflict {
                doc: conflicting,
                expected,
                actual,
            }) => {
                assert_eq!(conflicting.doc, doc("auth"));
                assert_eq!(expected, crate::ExpectedRevision::Absent);
                assert_eq!(actual, Some(seeded_at));
            }
            other => panic!("expected revision conflict, got {other:?}"),
        }

        // The rejected commit left nothing behind.
        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(snap.read(&doc("auth")).unwrap().unwrap().content, "base");
        assert_eq!(snap.revision(), seeded_at);
    }

    #[test]
    fn rollback_leaves_no_trace() {
        let store = MemoryStore::new();
        let scope = scope("main");

        let mut uow = store.begin_unit_of_work(&scope, ctx("abandoned")).unwrap();
        uow.create(doc("auth"), "never lands");
        store.rollback(uow).unwrap();

        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(snap.read(&doc("auth")).unwrap(), None);
        assert_eq!(snap.revision(), Revision(0));
    }

    #[test]
    fn history_records_create_modify_and_tombstone() {
        use crate::types::{content_digest, RevisionKind};

        let started = chrono::Utc::now();
        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");

        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "create-spec".into(),
                    actor: "alice".into(),
                },
            )
            .unwrap();
        uow.create(doc.clone(), "v1");
        let r1 = store.commit(uow, vec![]).unwrap();

        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "edit-spec".into(),
                    actor: "bob".into(),
                },
            )
            .unwrap();
        uow.update(doc.clone(), "v2", r1);
        let r2 = store.commit(uow, vec![]).unwrap();

        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "remove-spec".into(),
                    actor: "carol".into(),
                },
            )
            .unwrap();
        uow.delete(doc.clone(), r2);
        let r3 = store.commit(uow, vec![]).unwrap();

        let snap = store.snapshot(&scope).unwrap();
        let history = snap.history(&doc).unwrap();
        assert_eq!(history.len(), 3, "create, modify, tombstone: {history:?}");

        assert_eq!(history[0].revision, r1);
        assert_eq!(history[0].actor, "alice");
        assert_eq!(history[0].command, "create-spec");
        assert_eq!(
            history[0].kind,
            RevisionKind::Write {
                digest: content_digest("v1")
            }
        );

        assert_eq!(history[1].revision, r2);
        assert_eq!(history[1].actor, "bob");
        assert_eq!(history[1].command, "edit-spec");
        assert_eq!(
            history[1].kind,
            RevisionKind::Write {
                digest: content_digest("v2")
            }
        );

        assert_eq!(history[2].revision, r3);
        assert_eq!(history[2].actor, "carol");
        assert_eq!(history[2].command, "remove-spec");
        assert_eq!(history[2].kind, RevisionKind::Tombstone);

        // Every record carries a plausible UTC timestamp, in order.
        let now = chrono::Utc::now();
        for pair in history.windows(2) {
            assert!(pair[0].at <= pair[1].at);
        }
        assert!(history[0].at >= started && history[2].at <= now);

        // After the tombstone the document reads as the normal absent case.
        assert_eq!(snap.read(&doc).unwrap(), None);
    }

    #[test]
    fn revert_appends_a_new_revision_instead_of_rewriting() {
        use crate::types::{content_digest, RevisionKind};

        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");
        let r1 = seed(&store, &scope, &[(doc.clone(), "v1")]);

        let mut uow = store.begin_unit_of_work(&scope, ctx("edit")).unwrap();
        uow.update(doc.clone(), "v2", r1);
        let r2 = store.commit(uow, vec![]).unwrap();

        // Reverting to v1 is a new revision on top — history is never
        // rewritten or truncated.
        let mut uow = store.begin_unit_of_work(&scope, ctx("revert")).unwrap();
        uow.update(doc.clone(), "v1", r2);
        let r3 = store.commit(uow, vec![]).unwrap();

        let snap = store.snapshot(&scope).unwrap();
        let history = snap.history(&doc).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(
            (history[0].revision, history[1].revision, history[2].revision),
            (r1, r2, r3)
        );
        assert_eq!(
            history[2].kind,
            RevisionKind::Write {
                digest: content_digest("v1")
            }
        );
        assert_eq!(history[2].command, "revert");

        let document = snap.read(&doc).unwrap().unwrap();
        assert_eq!(document.content, "v1");
        assert_eq!(document.revision, r3);
    }

    // --- transactional outbox and fault injection ---

    use crate::types::{FaultPoint, OutboxCursor};

    #[test]
    fn events_land_atomically_and_replay_one_to_one_with_effective_commits() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");

        let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
        uow.create(doc.clone(), "v1");
        let r1 = store.commit(uow, vec![event("spec.created")]).unwrap();

        // A rejected commit's events must never surface.
        let stale = update(
            &store,
            &scope,
            &doc,
            "lost",
            Revision(r1.0 + 999),
            vec![event("never.happened")],
        );
        assert!(stale.is_err());

        let r2 = update(
            &store,
            &scope,
            &doc,
            "v2",
            r1,
            vec![event("spec.edited"), event("spec.reviewed")],
        )
        .unwrap();

        // Replay from cursor 0: the sequence corresponds one-to-one with the
        // commits that took effect, in replayable order.
        let entries = store.read_outbox(&scope, OutboxCursor(0)).unwrap();
        let named: Vec<(&str, Revision, u64)> = entries
            .iter()
            .map(|e| (e.record.name.as_str(), e.revision, e.seq))
            .collect();
        assert_eq!(
            named,
            vec![
                ("spec.created", r1, 1),
                ("spec.edited", r2, 2),
                ("spec.reviewed", r2, 3),
            ]
        );
    }

    #[test]
    fn acked_entries_are_not_redelivered() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");

        let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
        uow.create(doc.clone(), "v1");
        let r1 = store.commit(uow, vec![event("e1")]).unwrap();
        let r2 = update(&store, &scope, &doc, "v2", r1, vec![event("e2")]).unwrap();
        update(&store, &scope, &doc, "v3", r2, vec![event("e3")]).unwrap();

        store.ack_outbox(&scope, OutboxCursor(2)).unwrap();
        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(2));

        // Reading from the confirmed position never repeats consumed entries.
        let pending = store
            .read_outbox(&scope, store.outbox_acked(&scope).unwrap())
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].record.name, "e3");

        // Confirmation is monotonic: acknowledging backwards is a no-op.
        store.ack_outbox(&scope, OutboxCursor(1)).unwrap();
        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(2));

        // And durable: it survives a rebuild.
        let rebuilt = store.rebuild();
        assert_eq!(rebuilt.outbox_acked(&scope).unwrap(), OutboxCursor(2));
    }

    #[test]
    fn ack_beyond_the_outbox_end_fails_loudly() {
        let store = MemoryStore::new();
        let scope = scope("main");
        create(&store, &scope, &[(doc("auth"), "v1")], vec![event("e1")]).unwrap();

        // Confirming past the newest entry would silently skip everything
        // committed later — reject it instead of recording it.
        assert_eq!(
            store
                .ack_outbox(&scope, OutboxCursor(99))
                .unwrap_err()
                .code(),
            "backend"
        );
        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(0));
    }

    #[test]
    fn partial_commit_never_leaks() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");

        let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
        uow.create(doc.clone(), "v1");
        let r1 = store.commit(uow, vec![event("e1")]).unwrap();

        // Crash after document writes, before the outbox append: the commit
        // must be invisible in full after rebuild.
        store.crash_at(FaultPoint::AfterDocWrites);
        let crashed = update(&store, &scope, &doc, "v2", r1, vec![event("e2")]);
        assert!(crashed.is_err());

        let rebuilt = store.rebuild();
        let snap = rebuilt.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), r1);
        assert_eq!(snap.read(&doc).unwrap().unwrap().content, "v1");
        assert_eq!(snap.history(&doc).unwrap().len(), 1);
        let entries = rebuilt.read_outbox(&scope, OutboxCursor(0)).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].record.name, "e1");
    }

    #[test]
    fn outbox_append_failure_fails_the_whole_commit() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let doc = doc("auth");

        let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
        uow.create(doc.clone(), "v1");
        let r1 = store.commit(uow, vec![event("e1")]).unwrap();

        // The outbox append itself fails (an error, not a crash): the whole
        // commit must not take effect, immediately and without a rebuild.
        store.fail_outbox_append();
        let failed = update(&store, &scope, &doc, "v2", r1, vec![event("e2")]);
        assert_eq!(failed.unwrap_err().code(), "backend");

        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), r1);
        assert_eq!(snap.read(&doc).unwrap().unwrap().content, "v1");
        assert_eq!(snap.history(&doc).unwrap().len(), 1);
        assert_eq!(store.read_outbox(&scope, OutboxCursor(0)).unwrap().len(), 1);

        // The store stays usable: the next commit lands normally.
        let r2 = update(&store, &scope, &doc, "v2", r1, vec![event("e2")]).unwrap();
        assert!(r2 > r1);
        assert_eq!(store.read_outbox(&scope, OutboxCursor(0)).unwrap().len(), 2);
    }

    #[test]
    fn crash_recovery_keeps_docs_revision_history_and_outbox_consistent() {
        for point in [
            FaultPoint::AfterDocWrites,
            FaultPoint::AfterHistoryAppend,
            FaultPoint::BeforeOutboxAppend,
            FaultPoint::AfterOutboxAppend,
        ] {
            let store = MemoryStore::new();
            let scope = scope("main");
            let doc = doc("auth");

            let mut uow = store.begin_unit_of_work(&scope, ctx("create")).unwrap();
            uow.create(doc.clone(), "v1");
            let r1 = store.commit(uow, vec![event("e1")]).unwrap();

            store.crash_at(point);
            let crashed = update(&store, &scope, &doc, "v2", r1, vec![event("e2")]);
            assert!(crashed.is_err(), "{point:?}: crashed commit reports failure");

            // After rebuild the four facets — documents, project revision,
            // history, outbox — agree on one of exactly two worlds: the
            // commit fully happened, or it never did.
            let rebuilt = store.rebuild();
            let snap = rebuilt.snapshot(&scope).unwrap();
            let content = snap.read(&doc).unwrap().unwrap().content;
            let entries = rebuilt.read_outbox(&scope, OutboxCursor(0)).unwrap();
            let names: Vec<&str> = entries.iter().map(|e| e.record.name.as_str()).collect();
            if content == "v2" {
                assert_eq!(snap.revision().0, r1.0 + 1, "{point:?}");
                assert_eq!(snap.history(&doc).unwrap().len(), 2, "{point:?}");
                assert_eq!(names, vec!["e1", "e2"], "{point:?}");
            } else {
                assert_eq!(content, "v1", "{point:?}");
                assert_eq!(snap.revision(), r1, "{point:?}");
                assert_eq!(snap.history(&doc).unwrap().len(), 1, "{point:?}");
                assert_eq!(names, vec!["e1"], "{point:?}");
            }
        }
    }

    // --- export / import ---

    use crate::types::{content_digest, ImportMode, ImportOutcome, BUNDLE_FORMAT_VERSION};

    #[test]
    fn export_bundle_carries_version_scope_revision_and_digests() {
        let store = MemoryStore::new();
        let scope = scope("main");
        let r1 = seed(
            &store,
            &scope,
            &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        );

        let bundle = store.export(&scope).unwrap();
        assert_eq!(bundle.format_version, BUNDLE_FORMAT_VERSION);
        assert_eq!(bundle.scope, scope);
        assert_eq!(bundle.project_revision, r1);
        assert_eq!(bundle.documents.len(), 2);
        for doc in &bundle.documents {
            assert_eq!(doc.digest, content_digest(&doc.content));
        }
        let auth = bundle
            .documents
            .iter()
            .find(|d| d.doc == doc("auth"))
            .unwrap();
        assert_eq!(auth.content, "auth v1");
    }

    #[test]
    fn round_trip_into_a_fresh_store_matches_per_document() {
        use crate::types::RevisionKind;

        let store = MemoryStore::new();
        let scope = scope("main");
        let r1 = seed(
            &store,
            &scope,
            &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        );
        // Deepen history so "history starts at import" is observable.
        update(&store, &scope, &doc("auth"), "auth v2", r1, vec![]).unwrap();

        let bundle = store.export(&scope).unwrap();
        let fresh = MemoryStore::new();
        let report = fresh.import(bundle.clone(), ImportMode::CreateNew).unwrap();

        assert_eq!(report.documents.len(), 2);
        assert!(report
            .documents
            .iter()
            .all(|d| d.outcome == ImportOutcome::Created));

        let snap = fresh.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), report.project_revision);
        for doc in &bundle.documents {
            let imported = snap.read(&doc.doc).unwrap().unwrap();
            assert_eq!(imported.content, doc.content, "{:?}", doc.doc);
            // History starts at the import: exactly one record, a write
            // whose digest passed verification.
            let history = snap.history(&doc.doc).unwrap();
            assert_eq!(history.len(), 1, "{:?}", doc.doc);
            assert_eq!(history[0].command, "import");
            assert_eq!(
                history[0].kind,
                RevisionKind::Write {
                    digest: doc.digest.clone()
                }
            );
        }
    }

    #[test]
    fn digest_mismatch_rejects_the_whole_bundle() {
        let store = MemoryStore::new();
        let scope = scope("main");
        seed(
            &store,
            &scope,
            &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        );

        let mut bundle = store.export(&scope).unwrap();
        // Tamper with one document; its digest no longer matches.
        bundle.documents[1].content.push_str(" tampered");

        let fresh = MemoryStore::new();
        assert_eq!(
            fresh
                .import(bundle, ImportMode::CreateNew)
                .unwrap_err()
                .code(),
            "corrupt"
        );

        // Nothing was partially applied — not even the untampered document.
        let snap = fresh.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), Revision(0));
        assert_eq!(snap.read(&doc("auth")).unwrap(), None);
        assert_eq!(snap.read(&doc("billing")).unwrap(), None);
    }

    #[test]
    fn unknown_format_version_rejects_the_whole_bundle() {
        let store = MemoryStore::new();
        let scope = scope("main");
        seed(&store, &scope, &[(doc("auth"), "auth v1")]);

        let mut bundle = store.export(&scope).unwrap();
        bundle.format_version = BUNDLE_FORMAT_VERSION + 1;

        let fresh = MemoryStore::new();
        assert_eq!(
            fresh
                .import(bundle, ImportMode::CreateNew)
                .unwrap_err()
                .code(),
            "backend"
        );
        let snap = fresh.snapshot(&scope).unwrap();
        assert_eq!(snap.revision(), Revision(0));
        assert_eq!(snap.read(&doc("auth")).unwrap(), None);
    }
}
