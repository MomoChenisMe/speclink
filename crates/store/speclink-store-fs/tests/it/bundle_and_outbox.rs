//! Export/import bundles and the durable outbox cursor.
//!
//! A bundle is the contract's own shape, not this driver's: it is how data
//! leaves one driver and arrives in another, so anything of the filesystem
//! layout that leaked into it would quietly make backups driver-locked.

use speclink_store::{
    content_digest, CommandContext, DocumentId, EventRecord, ImportMode, ImportOutcome,
    OutboxCursor, ProjectId, RepoId, Revision, Scope, StoreError, TeamStore, BUNDLE_FORMAT_VERSION,
};
use speclink_store_fs::FsTeamStore;
use speclink_store_sqlite::SqliteTeamStore;
use std::collections::BTreeMap;

fn ctx(command: &str) -> CommandContext {
    CommandContext {
        command: command.into(),
        actor: "tester".into(),
    }
}

fn scope() -> Scope {
    Scope::new(ProjectId::new("acme"), RepoId::new("web"))
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
        at: chrono::Utc::now(),
    }
}

/// The documents every driver in the parity test is given, in the same
/// order, spanning the interesting shapes of the id set.
fn seed_documents() -> Vec<(DocumentId, &'static str)> {
    vec![
        (spec("auth"), "# auth\n\nthe canonical spec\n"),
        (DocumentId::WorkflowConfig, "schema: spec-driven\n"),
        (
            DocumentId::ChangeArtifact {
                change: "add-auth".into(),
                artifact: "specs/auth/spec.md".into(),
            },
            "## ADDED Requirements\n",
        ),
        (DocumentId::Language, "TeamStore: 團隊儲存\n"),
    ]
}

fn seed(store: &dyn TeamStore) -> Revision {
    let mut uow = store.begin_unit_of_work(&scope(), ctx("seed")).unwrap();
    for (doc, content) in seed_documents() {
        uow.create(doc, content);
    }
    store.commit(uow, vec![]).unwrap()
}

// --- the bundle belongs to the contract, not to the driver ----------------

#[test]
fn a_bundle_is_the_same_whichever_driver_exported_it() {
    // The same documents, written through two independent drivers. If the
    // digest is really the contract's, the bundles agree document for
    // document — which is what makes a backup taken from one driver
    // restorable into the other.
    let fs_dir = tempfile::tempdir().unwrap();
    let fs_store = FsTeamStore::open(fs_dir.path()).unwrap();
    seed(&fs_store);

    let sqlite_dir = tempfile::tempdir().unwrap();
    let sqlite_store = SqliteTeamStore::open(sqlite_dir.path().join("store.db")).unwrap();
    seed(&sqlite_store);

    let from_fs = fs_store.export(&scope()).unwrap();
    let from_sqlite = sqlite_store.export(&scope()).unwrap();

    let digests = |bundle: &speclink_store::Bundle| -> BTreeMap<DocumentId, String> {
        bundle
            .documents
            .iter()
            .map(|d| (d.doc.clone(), d.digest.clone()))
            .collect()
    };
    assert_eq!(
        digests(&from_fs),
        digests(&from_sqlite),
        "per-document digests must agree across drivers"
    );
    assert_eq!(from_fs.format_version, from_sqlite.format_version);
    assert_eq!(from_fs.scope, from_sqlite.scope);
    assert_eq!(from_fs.project_revision, from_sqlite.project_revision);
    assert_eq!(from_fs.documents.len(), seed_documents().len());

    // A bundle from one driver imports into the other and reads back whole.
    let target_dir = tempfile::tempdir().unwrap();
    let target = FsTeamStore::open(target_dir.path()).unwrap();
    target.import(from_sqlite, ImportMode::CreateNew).unwrap();
    let snap = target.snapshot(&scope()).unwrap();
    for (doc, content) in seed_documents() {
        assert_eq!(snap.read(&doc).unwrap().unwrap().content, content);
    }
}

#[test]
fn export_import_round_trip_and_tamper_rejection() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    seed(&store);

    let bundle = store.export(&scope()).unwrap();
    assert_eq!(bundle.format_version, BUNDLE_FORMAT_VERSION);
    for doc in &bundle.documents {
        assert_eq!(doc.digest, content_digest(&doc.content));
    }

    // Round-trip into a fresh store: contents match, and history starts at
    // the import rather than pretending to be the original's.
    let fresh_dir = tempfile::tempdir().unwrap();
    let fresh = FsTeamStore::open(fresh_dir.path()).unwrap();
    let report = fresh.import(bundle.clone(), ImportMode::CreateNew).unwrap();
    assert_eq!(report.documents.len(), seed_documents().len());
    assert!(report
        .documents
        .iter()
        .all(|d| d.outcome == ImportOutcome::Created));
    let snap = fresh.snapshot(&scope()).unwrap();
    assert_eq!(snap.revision(), report.project_revision);
    for doc in &bundle.documents {
        assert_eq!(snap.read(&doc.doc).unwrap().unwrap().content, doc.content);
        assert_eq!(snap.history(&doc.doc).unwrap().len(), 1);
    }

    // Overwrite mode replaces what is there and says so.
    let mut edited = bundle.clone();
    let auth = edited
        .documents
        .iter_mut()
        .find(|d| d.doc == spec("auth"))
        .expect("the bundle carries the auth spec");
    auth.content = "# auth\n\nrewritten\n".into();
    auth.digest = content_digest(&auth.content);
    let report = fresh.import(edited, ImportMode::Overwrite).unwrap();
    assert!(report
        .documents
        .iter()
        .all(|d| d.outcome == ImportOutcome::Overwritten));
    assert_eq!(
        fresh
            .snapshot(&scope())
            .unwrap()
            .read(&spec("auth"))
            .unwrap()
            .unwrap()
            .content,
        "# auth\n\nrewritten\n"
    );

    // Create-new into an occupied scope is refused rather than merged.
    assert!(fresh.import(bundle.clone(), ImportMode::CreateNew).is_err());

    // Tampering is rejected whole, leaving the target bit-for-bit untouched.
    let mut tampered = bundle;
    tampered.documents[0].content.push_str(" tampered");
    let target_dir = tempfile::tempdir().unwrap();
    let target = FsTeamStore::open(target_dir.path()).unwrap();
    assert_eq!(
        target.import(tampered, ImportMode::CreateNew).unwrap_err().code(),
        "corrupt"
    );
    assert_eq!(target.snapshot(&scope()).unwrap().revision(), Revision(0));
    assert_eq!(
        target.export(&scope()).unwrap().documents.len(),
        0,
        "a rejected import leaves nothing behind"
    );
}

#[test]
fn an_unknown_bundle_format_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    seed(&store);
    let mut bundle = store.export(&scope()).unwrap();
    bundle.format_version = BUNDLE_FORMAT_VERSION + 1;

    let target_dir = tempfile::tempdir().unwrap();
    let target = FsTeamStore::open(target_dir.path()).unwrap();
    match target.import(bundle, ImportMode::CreateNew) {
        Err(StoreError::Backend { source }) => assert!(
            source.contains("format version"),
            "the reason names the version: {source}"
        ),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

// --- the outbox cursor is durable -----------------------------------------

#[test]
fn the_ack_cursor_survives_a_reopen_and_never_replays_confirmed_entries() {
    let dir = tempfile::tempdir().unwrap();
    let scope = scope();

    {
        let store = FsTeamStore::open(dir.path()).unwrap();
        let mut uow = store.begin_unit_of_work(&scope, ctx("seed")).unwrap();
        uow.create(spec("auth"), "v1");
        let r1 = store.commit(uow, vec![event("first"), event("second")]).unwrap();
        let mut uow = store.begin_unit_of_work(&scope, ctx("edit")).unwrap();
        uow.update(spec("auth"), "v2", r1);
        store.commit(uow, vec![event("third")]).unwrap();

        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(0));
        store.ack_outbox(&scope, OutboxCursor(1)).unwrap();

        // The position is monotonic: acknowledging backwards is a no-op, not
        // a rewind that would redeliver what a consumer already handled.
        store.ack_outbox(&scope, OutboxCursor(0)).unwrap();
        assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(1));
    }

    // Reopened: the consumer resumes where it left off, not from the start.
    let store = FsTeamStore::open(dir.path()).unwrap();
    assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(1));
    let pending: Vec<String> = store
        .read_outbox(&scope, store.outbox_acked(&scope).unwrap())
        .unwrap()
        .into_iter()
        .map(|e| e.record.name)
        .collect();
    assert_eq!(pending, ["second", "third"]);

    // Acknowledging past the end is refused: accepting it would silently
    // skip everything committed later.
    assert!(store.ack_outbox(&scope, OutboxCursor(99)).is_err());
    assert_eq!(store.outbox_acked(&scope).unwrap(), OutboxCursor(1));

    // Entries carry the revision of the commit that produced them.
    let entries = store.read_outbox(&scope, OutboxCursor(0)).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[0].revision, Revision(1));
    assert_eq!(entries[2].revision, Revision(2));
    assert_eq!(entries[0].record.payload, serde_json::json!({ "event": "first" }));
}
