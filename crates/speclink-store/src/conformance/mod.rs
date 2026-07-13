//! The reusable conformance suite: the correctness baseline every TeamStore
//! implementation — official drivers, custom stores, the in-memory
//! reference — must pass through the same entry point. "Declared means
//! owed": the suite reads the manifest, checks the declared level's
//! required capabilities, and runs the gate scenarios for everything the
//! implementation claims. Any failure fails the suite as a whole.

use crate::types::{
    Capability, CapabilityLevel, FaultPoint, ImportMode, OutboxCursor, Revision, CONTRACT_VERSION,
};
use crate::{StoreError, TeamStore};

/// What an implementation plugs into the suite: store lifecycle plus the
/// contract-defined fault hooks. `reset` starts a fresh, empty instance;
/// `restart` simulates a crash restart from durable state; the `arm_*`
/// hooks inject a fault into the current instance's next commit.
pub trait StoreHarness {
    /// Discard any previous instance and hand out a fresh, empty store.
    fn reset(&mut self) -> &dyn TeamStore;
    /// The current store under test.
    fn store(&self) -> &dyn TeamStore;
    /// Crash the current store's next commit at the given stage boundary.
    fn arm_crash(&mut self, point: FaultPoint);
    /// Make the current store's next commit fail its outbox append.
    fn arm_outbox_failure(&mut self);
    /// Rebuild the current store from its durable state.
    fn restart(&mut self);
}

/// One failed check, by stable check name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// `capability-check`, `gate:<scenario>` or `check:<capability>`.
    pub check: String,
    pub detail: String,
}

/// The suite's verdict for one implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    pub contract_version: u32,
    pub driver: String,
    pub level: CapabilityLevel,
    pub passed: bool,
    /// The verified capability list — the declared capabilities, reported
    /// once every corresponding test passed. Empty on failure.
    pub capabilities: Vec<Capability>,
    pub failures: Vec<ConformanceFailure>,
}

/// Run the full suite against any TeamStore implementation. Stage one
/// checks the manifest declaration (a level must be backed by its required
/// capabilities); stage two runs the gate scenarios for everything the
/// manifest declares. Any failure fails the report as a whole.
pub fn run(harness: &mut dyn StoreHarness) -> ConformanceReport {
    let manifest = harness.reset().manifest();
    let mut failures = Vec::new();

    // Stage 1: capability check — a store that misdeclares itself is judged
    // here, before any behavior gate runs.
    if manifest.contract_version != CONTRACT_VERSION {
        failures.push(ConformanceFailure {
            check: "capability-check".into(),
            detail: format!(
                "contract version {} is not the suite's {}",
                manifest.contract_version, CONTRACT_VERSION
            ),
        });
    }
    if manifest.level >= CapabilityLevel::SingleNode {
        for required in [
            Capability::Snapshot,
            Capability::Cas,
            Capability::Transaction,
            Capability::History,
            Capability::Outbox,
        ] {
            if !manifest.capabilities.contains(&required) {
                failures.push(ConformanceFailure {
                    check: "capability-check".into(),
                    detail: format!(
                        "declared level {} requires capability \"{}\"",
                        manifest.level.as_str(),
                        required.as_str()
                    ),
                });
            }
        }
    }
    if manifest.capabilities.contains(&Capability::Cluster)
        != (manifest.level == CapabilityLevel::Cluster)
    {
        failures.push(ConformanceFailure {
            check: "capability-check".into(),
            detail: "cluster capability and cluster level must be declared together".into(),
        });
    }
    if !failures.is_empty() {
        return ConformanceReport {
            contract_version: manifest.contract_version,
            driver: manifest.driver,
            level: manifest.level,
            passed: false,
            capabilities: Vec::new(),
            failures,
        };
    }

    // Stage 2: gate scenarios, graded by declaration — every declared
    // capability has a corresponding test.
    let has = |capability: Capability| manifest.capabilities.contains(&capability);
    let mut gates: Vec<(&str, Result<(), String>)> = Vec::new();
    gates.push(("gate:tenant-scope", gate_tenant_scope(harness)));
    if has(Capability::Snapshot) {
        gates.push(("gate:mixed-snapshot", gate_mixed_snapshot(harness)));
    }
    if has(Capability::Cas) {
        gates.push(("gate:cas-race", gate_cas_race(harness)));
    }
    if has(Capability::Transaction) {
        gates.push(("gate:partial-commit", gate_partial_commit(harness)));
    }
    if has(Capability::Outbox) {
        gates.push(("gate:outbox-failure", gate_outbox_failure(harness)));
        gates.push(("gate:crash-recovery", gate_crash_recovery(harness)));
        gates.push(("check:outbox-ack", check_outbox_ack(harness)));
    }
    if has(Capability::History) {
        gates.push(("check:history-tombstone", check_history_tombstone(harness)));
    }
    if has(Capability::Migration) {
        gates.push(("check:migration", check_migration(harness)));
    }
    if has(Capability::Backup) {
        gates.push(("check:backup-roundtrip", check_backup_roundtrip(harness)));
    }
    for (check, outcome) in gates {
        if let Err(detail) = outcome {
            failures.push(ConformanceFailure {
                check: check.into(),
                detail,
            });
        }
    }

    let passed = failures.is_empty();
    ConformanceReport {
        contract_version: manifest.contract_version,
        driver: manifest.driver,
        level: manifest.level,
        passed,
        capabilities: if passed {
            manifest.capabilities.iter().copied().collect()
        } else {
            Vec::new()
        },
        failures,
    }
}

/// Shared scenario fixtures: one vocabulary for the gate scenarios here and
/// the reference store's unit tests, so both assert against the same
/// shapes.
pub(crate) mod fixtures {
    use crate::types::{DocumentId, EventRecord, OutboxCursor, ProjectId, RepoId, Revision, Scope};
    use crate::uow::CommandContext;
    use crate::{StoreError, TeamStore};

    pub(crate) fn ctx(command: &str) -> CommandContext {
        CommandContext {
            command: command.into(),
            actor: "conformance".into(),
        }
    }

    pub(crate) fn scope(repo: &str) -> Scope {
        Scope::new(ProjectId::new("conformance"), RepoId::new(repo))
    }

    pub(crate) fn doc(name: &str) -> DocumentId {
        DocumentId::CanonicalSpec {
            capability: name.into(),
        }
    }

    pub(crate) fn event(name: &str) -> EventRecord {
        EventRecord {
            name: name.into(),
            payload: serde_json::json!({ "event": name }),
            actor: "conformance".into(),
            at: chrono::Utc::now(),
        }
    }

    /// Create the given documents in one commit carrying `events`.
    pub(crate) fn create(
        store: &dyn TeamStore,
        scope: &Scope,
        docs: &[(DocumentId, &str)],
        events: Vec<EventRecord>,
    ) -> Result<Revision, String> {
        let mut uow = store
            .begin_unit_of_work(scope, ctx("conformance-create"))
            .map_err(|e| format!("begin failed: {e}"))?;
        for (doc, content) in docs {
            uow.create(doc.clone(), *content);
        }
        store
            .commit(uow, events)
            .map_err(|e| format!("create commit failed: {e}"))
    }

    /// CAS-update one document in one commit carrying `events`.
    pub(crate) fn update(
        store: &dyn TeamStore,
        scope: &Scope,
        doc: &DocumentId,
        content: &str,
        read_at: Revision,
        events: Vec<EventRecord>,
    ) -> Result<Revision, StoreError> {
        let mut uow = store.begin_unit_of_work(scope, ctx("conformance-update"))?;
        uow.update(doc.clone(), content, read_at);
        store.commit(uow, events)
    }

    pub(crate) fn read_content(
        store: &dyn TeamStore,
        scope: &Scope,
        doc: &DocumentId,
    ) -> Result<Option<String>, String> {
        let snap = store
            .snapshot(scope)
            .map_err(|e| format!("snapshot failed: {e}"))?;
        Ok(snap
            .read(doc)
            .map_err(|e| format!("read failed: {e}"))?
            .map(|document| document.content))
    }

    pub(crate) fn outbox_names(
        store: &dyn TeamStore,
        scope: &Scope,
    ) -> Result<Vec<String>, String> {
        Ok(store
            .read_outbox(scope, OutboxCursor(0))
            .map_err(|e| format!("outbox read failed: {e}"))?
            .into_iter()
            .map(|entry| entry.record.name)
            .collect())
    }
}

use fixtures::{create, ctx, doc, event, outbox_names, read_content, scope, update};

// --- gate scenarios ---

/// Cross-tenant reads must be isolated: a scope never sees another scope's
/// documents.
fn gate_tenant_scope(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let repo_a = scope("repo-a");
    let repo_b = scope("repo-b");
    create(store, &repo_b, &[(doc("auth"), "repo-b secret")], vec![])?;

    let snap = store
        .snapshot(&repo_a)
        .map_err(|e| format!("snapshot failed: {e}"))?;
    match snap.read(&doc("auth")) {
        Ok(None) => Ok(()),
        Err(StoreError::PermissionDenied) | Err(StoreError::NotFound) => Ok(()),
        Ok(Some(document)) => Err(format!(
            "repo A read returned repo B's content: {:?}",
            document.content
        )),
        Err(other) => Err(format!("expected isolation, got failure: {other}")),
    }
}

/// A snapshot is a fixed-point view: a concurrent commit must not bleed in,
/// in full or in part.
fn gate_mixed_snapshot(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let scope = scope("main");
    let seeded_at = create(
        store,
        &scope,
        &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        vec![],
    )?;

    let snap = store
        .snapshot(&scope)
        .map_err(|e| format!("snapshot failed: {e}"))?;
    let revision_before = snap.revision();

    let mut uow = store
        .begin_unit_of_work(&scope, ctx("conformance-edit"))
        .map_err(|e| format!("begin failed: {e}"))?;
    uow.update(doc("auth"), "auth v2", seeded_at);
    uow.update(doc("billing"), "billing v2", seeded_at);
    store
        .commit(uow, vec![])
        .map_err(|e| format!("writer commit failed: {e}"))?;

    let auth = snap
        .read(&doc("auth"))
        .map_err(|e| format!("snapshot read failed: {e}"))?
        .ok_or("snapshot lost a document")?;
    let billing = snap
        .read(&doc("billing"))
        .map_err(|e| format!("snapshot read failed: {e}"))?
        .ok_or("snapshot lost a document")?;
    if auth.content != "auth v1" || billing.content != "billing v1" {
        return Err(format!(
            "mixed snapshot: saw ({:?}, {:?}) instead of the fixed point",
            auth.content, billing.content
        ));
    }
    if snap.revision() != revision_before {
        return Err("snapshot revision moved under the reader".into());
    }
    Ok(())
}

/// Two units of work racing on the same document: exactly one wins; the
/// loser learns the conflicting document and the actual revision.
fn gate_cas_race(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let scope = scope("main");
    let seeded_at = create(store, &scope, &[(doc("auth"), "base")], vec![])?;

    let mut first = store
        .begin_unit_of_work(&scope, ctx("conformance-race-1"))
        .map_err(|e| format!("begin failed: {e}"))?;
    first.update(doc("auth"), "winner", seeded_at);
    let mut second = store
        .begin_unit_of_work(&scope, ctx("conformance-race-2"))
        .map_err(|e| format!("begin failed: {e}"))?;
    second.update(doc("auth"), "loser", seeded_at);

    let winning_revision = store
        .commit(first, vec![])
        .map_err(|e| format!("first commit should win: {e}"))?;
    match store.commit(second, vec![]) {
        Err(StoreError::RevisionConflict {
            doc: conflicting,
            actual,
            ..
        }) => {
            if conflicting.doc != doc("auth") {
                return Err(format!("conflict names the wrong document: {conflicting:?}"));
            }
            if actual != Some(winning_revision) {
                return Err(format!(
                    "conflict reports actual {actual:?}, winner left {winning_revision:?}"
                ));
            }
        }
        Ok(_) => return Err("both racing commits succeeded".into()),
        Err(other) => return Err(format!("expected revision conflict, got: {other}")),
    }

    match read_content(store, &scope, &doc("auth"))? {
        Some(content) if content == "winner" => Ok(()),
        other => Err(format!("store content is not the winner's write: {other:?}")),
    }
}

/// A commit crashed mid-way must be invisible in full after restart.
fn gate_partial_commit(harness: &mut dyn StoreHarness) -> Result<(), String> {
    harness.reset();
    let scope = scope("main");
    let seeded_at = create(harness.store(), &scope, &[(doc("auth"), "v1")], vec![])?;

    harness.arm_crash(FaultPoint::AfterDocWrites);
    if update(
        harness.store(),
        &scope,
        &doc("auth"),
        "v2",
        seeded_at,
        vec![event("edited")],
    )
    .is_ok()
    {
        return Err("commit reported success while armed to crash".into());
    }

    harness.restart();
    let store = harness.store();
    match read_content(store, &scope, &doc("auth"))? {
        Some(content) if content == "v1" => {}
        other => return Err(format!("partial commit leaked document state: {other:?}")),
    }
    let snap = store
        .snapshot(&scope)
        .map_err(|e| format!("snapshot failed: {e}"))?;
    if snap.revision() != seeded_at {
        return Err(format!(
            "project revision moved: {:?} != {seeded_at:?}",
            snap.revision()
        ));
    }
    let history = snap
        .history(&doc("auth"))
        .map_err(|e| format!("history failed: {e}"))?;
    if history.len() != 1 {
        return Err(format!("history leaked partial records: {}", history.len()));
    }
    Ok(())
}

/// A failed outbox append must fail the whole commit, immediately and
/// without a restart, leaving the store usable.
fn gate_outbox_failure(harness: &mut dyn StoreHarness) -> Result<(), String> {
    harness.reset();
    let scope = scope("main");
    let seeded_at = create(
        harness.store(),
        &scope,
        &[(doc("auth"), "v1")],
        vec![event("created")],
    )?;

    harness.arm_outbox_failure();
    let store = harness.store();
    if update(store, &scope, &doc("auth"), "v2", seeded_at, vec![event("edited")]).is_ok() {
        return Err("commit reported success while its outbox append failed".into());
    }

    match read_content(store, &scope, &doc("auth"))? {
        Some(content) if content == "v1" => {}
        other => return Err(format!("failed commit leaked document state: {other:?}")),
    }
    let names = outbox_names(store, &scope)?;
    if names != ["created"] {
        return Err(format!("failed commit leaked outbox entries: {names:?}"));
    }

    // The store absorbs the failure: the next commit lands normally.
    update(store, &scope, &doc("auth"), "v2", seeded_at, vec![event("edited")])
        .map_err(|e| format!("store unusable after absorbed failure: {e}"))?;
    let names = outbox_names(store, &scope)?;
    if names != ["created", "edited"] {
        return Err(format!("outbox out of step after recovery: {names:?}"));
    }
    Ok(())
}

/// Crashing at every commit stage boundary and restarting: documents,
/// project revision, history and outbox must agree on "fully happened" or
/// "never happened", and replay from cursor 0 must match effective commits.
fn gate_crash_recovery(harness: &mut dyn StoreHarness) -> Result<(), String> {
    for point in [
        FaultPoint::AfterDocWrites,
        FaultPoint::AfterHistoryAppend,
        FaultPoint::BeforeOutboxAppend,
        FaultPoint::AfterOutboxAppend,
    ] {
        harness.reset();
        let scope = scope("main");
        let seeded_at = create(
            harness.store(),
            &scope,
            &[(doc("auth"), "v1")],
            vec![event("created")],
        )?;

        harness.arm_crash(point);
        if update(
            harness.store(),
            &scope,
            &doc("auth"),
            "v2",
            seeded_at,
            vec![event("edited")],
        )
        .is_ok()
        {
            return Err(format!("{point:?}: commit reported success while crashing"));
        }

        harness.restart();
        let store = harness.store();
        let content = read_content(store, &scope, &doc("auth"))?
            .ok_or(format!("{point:?}: document vanished"))?;
        let snap = store
            .snapshot(&scope)
            .map_err(|e| format!("{point:?}: snapshot failed: {e}"))?;
        let history_len = snap
            .history(&doc("auth"))
            .map_err(|e| format!("{point:?}: history failed: {e}"))?
            .len();
        let names = outbox_names(store, &scope)?;

        let consistent = if content == "v2" {
            snap.revision().0 == seeded_at.0 + 1
                && history_len == 2
                && names == ["created", "edited"]
        } else {
            content == "v1"
                && snap.revision() == seeded_at
                && history_len == 1
                && names == ["created"]
        };
        if !consistent {
            return Err(format!(
                "{point:?}: facets disagree — content {content:?}, revision {:?}, {history_len} history records, outbox {names:?}",
                snap.revision()
            ));
        }
    }
    Ok(())
}

/// Confirmed outbox entries are never redelivered when reading from the
/// durable consumer position.
fn check_outbox_ack(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let scope = scope("main");
    let seeded_at = create(
        store,
        &scope,
        &[(doc("auth"), "v1")],
        vec![event("first")],
    )?;
    update(store, &scope, &doc("auth"), "v2", seeded_at, vec![event("second")])
        .map_err(|e| format!("update failed: {e}"))?;

    let entries = store
        .read_outbox(&scope, OutboxCursor(0))
        .map_err(|e| format!("outbox read failed: {e}"))?;
    if entries.len() != 2 {
        return Err(format!("expected 2 outbox entries, got {}", entries.len()));
    }
    store
        .ack_outbox(&scope, OutboxCursor(entries[0].seq))
        .map_err(|e| format!("ack failed: {e}"))?;
    let acked = store
        .outbox_acked(&scope)
        .map_err(|e| format!("acked position failed: {e}"))?;
    let pending = store
        .read_outbox(&scope, acked)
        .map_err(|e| format!("outbox read failed: {e}"))?;
    if pending.len() != 1 || pending[0].record.name != "second" {
        return Err(format!(
            "confirmed entries redelivered: {:?}",
            pending.iter().map(|e| &e.record.name).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

/// Create, modify, delete: three immutable history records, the last a
/// tombstone, after which the document reads as the normal absent case.
fn check_history_tombstone(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let scope = scope("main");
    let r1 = create(store, &scope, &[(doc("auth"), "v1")], vec![])?;
    let r2 = update(store, &scope, &doc("auth"), "v2", r1, vec![])
        .map_err(|e| format!("update failed: {e}"))?;
    let mut uow = store
        .begin_unit_of_work(&scope, ctx("conformance-delete"))
        .map_err(|e| format!("begin failed: {e}"))?;
    uow.delete(doc("auth"), r2);
    store
        .commit(uow, vec![])
        .map_err(|e| format!("delete commit failed: {e}"))?;

    let snap = store
        .snapshot(&scope)
        .map_err(|e| format!("snapshot failed: {e}"))?;
    let history = snap
        .history(&doc("auth"))
        .map_err(|e| format!("history failed: {e}"))?;
    if history.len() != 3 {
        return Err(format!("expected 3 history records, got {}", history.len()));
    }
    match (&history[0].kind, &history[1].kind, &history[2].kind) {
        (
            crate::RevisionKind::Write { .. },
            crate::RevisionKind::Write { .. },
            crate::RevisionKind::Tombstone,
        ) => {}
        kinds => return Err(format!("unexpected history shape: {kinds:?}")),
    }
    if history.iter().any(|record| record.actor.is_empty()) {
        return Err("history records missing the actor".into());
    }
    match snap.read(&doc("auth")) {
        Ok(None) => Ok(()),
        other => Err(format!("tombstoned document still reads: {other:?}")),
    }
}

/// Migration to the current contract version succeeds.
fn check_migration(harness: &mut dyn StoreHarness) -> Result<(), String> {
    harness
        .reset()
        .migrate(CONTRACT_VERSION)
        .map_err(|e| format!("migrate to current version failed: {e}"))
}

/// Export/import round-trip into a fresh store: contents match per
/// document, history starts at the import, tampering is rejected whole.
fn check_backup_roundtrip(harness: &mut dyn StoreHarness) -> Result<(), String> {
    let store = harness.reset();
    let scope = scope("main");
    create(
        store,
        &scope,
        &[(doc("auth"), "auth v1"), (doc("billing"), "billing v1")],
        vec![],
    )?;
    let bundle = store
        .export(&scope)
        .map_err(|e| format!("export failed: {e}"))?;

    let fresh = harness.reset();
    fresh
        .import(bundle.clone(), ImportMode::CreateNew)
        .map_err(|e| format!("import failed: {e}"))?;
    for doc in &bundle.documents {
        match read_content(fresh, &scope, &doc.doc)? {
            Some(content) if content == doc.content => {}
            other => return Err(format!("round-trip mismatch for {:?}: {other:?}", doc.doc)),
        }
        let snap = fresh
            .snapshot(&scope)
            .map_err(|e| format!("snapshot failed: {e}"))?;
        let history = snap
            .history(&doc.doc)
            .map_err(|e| format!("history failed: {e}"))?;
        if history.len() != 1 {
            return Err(format!(
                "imported history should start at the import, got {} records",
                history.len()
            ));
        }
    }

    // Tampering is rejected without partial application.
    let mut tampered = bundle;
    if let Some(first) = tampered.documents.first_mut() {
        first.content.push_str(" tampered");
    }
    let target = harness.reset();
    if target.import(tampered, ImportMode::CreateNew).is_ok() {
        return Err("import accepted a tampered bundle".into());
    }
    let snap = target
        .snapshot(&scope)
        .map_err(|e| format!("snapshot failed: {e}"))?;
    if snap.revision() != Revision(0) {
        return Err("rejected import still moved the store".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run, ConformanceReport, StoreHarness};
    use crate::memory::MemoryHarness;
    use crate::types::{
        Bundle, Capability, CapabilityLevel, EventRecord, FaultPoint, ImportMode, ImportReport,
        Manifest, OutboxCursor, OutboxEntry, Revision, Scope, CONTRACT_VERSION,
    };
    use crate::uow::{CommandContext, UnitOfWork};
    use crate::{Snapshot, StoreError, TeamStore};

    /// A deliberately defective implementation: the manifest declares
    /// single-node yet the capability set lacks `outbox`. The suite must
    /// fail it at the capability-check stage, before any gate runs.
    struct MissingOutboxStore;

    impl TeamStore for MissingOutboxStore {
        fn manifest(&self) -> Manifest {
            Manifest {
                contract_version: CONTRACT_VERSION,
                driver: "missing-outbox".into(),
                level: CapabilityLevel::SingleNode,
                capabilities: [
                    Capability::Snapshot,
                    Capability::Cas,
                    Capability::Transaction,
                    Capability::History,
                ]
                .into_iter()
                .collect(),
            }
        }
        fn health(&self) -> Result<(), StoreError> {
            Ok(())
        }
        fn migrate(&self, _target_version: u32) -> Result<(), StoreError> {
            Err(StoreError::Unavailable)
        }
        fn snapshot<'a>(&'a self, _scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn begin_unit_of_work(
            &self,
            _scope: &Scope,
            _ctx: CommandContext,
        ) -> Result<UnitOfWork, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn commit(
            &self,
            _uow: UnitOfWork,
            _events: Vec<EventRecord>,
        ) -> Result<Revision, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn rollback(&self, _uow: UnitOfWork) -> Result<(), StoreError> {
            Ok(())
        }
        fn export(&self, _scope: &Scope) -> Result<Bundle, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn import(&self, _bundle: Bundle, _mode: ImportMode) -> Result<ImportReport, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn read_outbox(
            &self,
            _scope: &Scope,
            _from: OutboxCursor,
        ) -> Result<Vec<OutboxEntry>, StoreError> {
            Err(StoreError::Unavailable)
        }
        fn ack_outbox(&self, _scope: &Scope, _up_to: OutboxCursor) -> Result<(), StoreError> {
            Err(StoreError::Unavailable)
        }
        fn outbox_acked(&self, _scope: &Scope) -> Result<OutboxCursor, StoreError> {
            Err(StoreError::Unavailable)
        }
    }

    /// The simplest possible harness around the defective double — enough
    /// to prove the entry point takes any TeamStore trait object. Fault
    /// hooks are no-ops: the suite never gets past the capability check.
    struct DefectiveHarness {
        store: MissingOutboxStore,
    }

    impl StoreHarness for DefectiveHarness {
        fn reset(&mut self) -> &dyn TeamStore {
            self.store = MissingOutboxStore;
            &self.store
        }
        fn store(&self) -> &dyn TeamStore {
            &self.store
        }
        fn arm_crash(&mut self, _point: FaultPoint) {}
        fn arm_outbox_failure(&mut self) {}
        fn restart(&mut self) {}
    }

    #[test]
    fn declared_single_node_without_outbox_fails_at_the_capability_check() {
        let mut harness = DefectiveHarness {
            store: MissingOutboxStore,
        };
        let report: ConformanceReport = run(&mut harness);

        assert!(!report.passed);
        assert_eq!(report.driver, "missing-outbox");
        assert_eq!(report.level, CapabilityLevel::SingleNode);

        // The verdict falls at the capability-check stage — no behavior
        // gate ever runs against a store that lies about its level.
        assert!(
            !report.failures.is_empty()
                && report.failures.iter().all(|f| f.check == "capability-check"),
            "failures: {:?}",
            report.failures
        );
        // …and the report names what is missing.
        assert!(
            report.failures.iter().any(|f| f.detail.contains("outbox")),
            "failures: {:?}",
            report.failures
        );
    }

    #[test]
    fn in_memory_reference_passes_the_full_suite() {
        let mut harness = MemoryHarness::new();
        let report = run(&mut harness);

        assert!(report.passed, "failures: {:?}", report.failures);
        // The suite reports the contract version and the verified
        // capability list of the implementation under test.
        assert_eq!(report.contract_version, CONTRACT_VERSION);
        assert_eq!(report.driver, "memory");
        for capability in [
            Capability::Snapshot,
            Capability::Cas,
            Capability::Transaction,
            Capability::History,
            Capability::Outbox,
            Capability::Migration,
            Capability::Backup,
        ] {
            assert!(
                report.capabilities.contains(&capability),
                "verified capability list should contain {capability:?}: {:?}",
                report.capabilities
            );
        }
    }
}
