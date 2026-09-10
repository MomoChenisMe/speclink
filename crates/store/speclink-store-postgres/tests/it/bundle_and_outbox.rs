//! Bundles and the outbox read side.
//!
//! A bundle is the driver-independent shape of a scope: the same documents
//! exported from any driver must digest identically, which is what makes
//! migrating between drivers a copy rather than a conversion.

use crate::support;

use speclink_store::{
    Bundle, ImportMode, ImportOutcome, OutboxCursor, RevisionKind, StoreError, TeamStore,
};
use speclink_store_postgres::PostgresTeamStore;
use speclink_store_sqlite::SqliteTeamStore;
use support::{ctx, event, scope, spec, TestDb};

/// Write the same two documents into any driver, the same way.
fn seed(store: &dyn TeamStore) {
    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("auth"), "# Auth\n\nbody\n");
    uow.create(spec("billing"), "# Billing\n");
    store.commit(uow, vec![event("created")]).expect("commit");
}

fn export_and_import_round_trip() {
    let source_db = TestDb::new();
    let source = PostgresTeamStore::connect(source_db.url()).expect("source store");
    seed(&source);
    let bundle = source.export(&scope("main")).expect("export");
    assert_eq!(bundle.documents.len(), 2);

    let target_db = TestDb::new();
    let target = PostgresTeamStore::connect(target_db.url()).expect("target store");
    let report = target
        .import(bundle, ImportMode::CreateNew)
        .expect("import into an empty scope");
    assert_eq!(report.documents.len(), 2);
    for imported in &report.documents {
        assert_eq!(imported.outcome, ImportOutcome::Created);
    }

    let snapshot = target.snapshot(&scope("main")).expect("snapshot");
    assert_eq!(
        snapshot
            .read(&spec("auth"))
            .expect("read")
            .expect("exists")
            .content,
        "# Auth\n\nbody\n"
    );
    // An imported document starts its history at the import, not with the
    // source's whole past.
    assert_eq!(snapshot.history(&spec("auth")).expect("history").len(), 1);
}

/// The contract is explicit: create-new means the *scope* holds nothing. A
/// check that only looked for the bundle's own documents would wave through an
/// import into a scope holding unrelated ones, and quietly interleave two
/// stores' histories.
fn import_create_new_refuses_a_scope_that_holds_any_document() {
    let source_db = TestDb::new();
    let source = PostgresTeamStore::connect(source_db.url()).expect("source store");
    let mut uow = source
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("auth"), "# Auth\n");
    source.commit(uow, vec![event("created")]).expect("commit");
    let bundle = source.export(&scope("main")).expect("export");

    // The target holds a *different* document — nothing the bundle names.
    let target_db = TestDb::new();
    let target = PostgresTeamStore::connect(target_db.url()).expect("target store");
    let mut uow = target
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("unrelated"), "# Unrelated\n");
    let occupied = target.commit(uow, vec![event("created")]).expect("commit");

    match target.import(bundle, ImportMode::CreateNew) {
        Err(StoreError::Backend { source }) => assert!(
            source.contains("already holds documents"),
            "the reason should say why: {source}"
        ),
        Err(other) => panic!("expected backend, got {other:?}"),
        Ok(report) => panic!("create-new imported into an occupied scope: {report:?}"),
    }

    // The refusal changed nothing.
    let snapshot = target.snapshot(&scope("main")).expect("snapshot");
    assert_eq!(snapshot.revision(), occupied);
    assert!(snapshot.read(&spec("auth")).expect("read").is_none());
}

fn import_rejects_a_tampered_bundle_and_writes_nothing() {
    let source_db = TestDb::new();
    let source = PostgresTeamStore::connect(source_db.url()).expect("source store");
    seed(&source);
    let mut bundle = source.export(&scope("main")).expect("export");
    bundle.documents[0].content.push_str("tampered");

    let target_db = TestDb::new();
    let target = PostgresTeamStore::connect(target_db.url()).expect("target store");
    match target.import(bundle, ImportMode::CreateNew) {
        Err(StoreError::Corrupt { .. }) => {}
        Err(other) => panic!("expected corrupt, got {other:?}"),
        Ok(report) => panic!("a tampered bundle imported: {report:?}"),
    }

    let snapshot = target.snapshot(&scope("main")).expect("snapshot");
    assert_eq!(
        snapshot.revision().0,
        0,
        "a rejected bundle moved the target's revision"
    );
}

/// The acceptance criterion for this task: two drivers, one bundle shape.
fn bundles_agree_with_the_sqlite_driver() {
    let db = TestDb::new();
    let from_postgres = {
        let store = PostgresTeamStore::connect(db.url()).expect("postgres store");
        seed(&store);
        store.export(&scope("main")).expect("export from postgres")
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let from_sqlite = {
        let store = SqliteTeamStore::open(dir.path().join("store.db")).expect("sqlite store");
        seed(&store);
        store.export(&scope("main")).expect("export from sqlite")
    };

    let shape = |bundle: &Bundle| {
        bundle
            .documents
            .iter()
            .map(|document| (document.doc.clone(), document.digest.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        shape(&from_postgres),
        shape(&from_sqlite),
        "the two drivers disagree on a bundle's documents or their digests"
    );
    assert_eq!(from_postgres.format_version, from_sqlite.format_version);
    assert_eq!(from_postgres.project_revision, from_sqlite.project_revision);
}

fn outbox_reads_from_a_cursor_and_acks_monotonically() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("store");

    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("auth"), "v1");
    store
        .commit(uow, vec![event("first"), event("second")])
        .expect("commit");

    assert_eq!(store.outbox_acked(&scope("main")).expect("acked"), OutboxCursor(0));
    let entries = store
        .read_outbox(&scope("main"), OutboxCursor(0))
        .expect("read");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].record.name, "first");
    assert_eq!(entries[1].record.name, "second");
    assert!(entries[0].seq < entries[1].seq, "outbox sequence is not monotonic");
    assert_eq!(entries[0].record.payload, serde_json::json!({ "event": "first" }));

    store
        .ack_outbox(&scope("main"), OutboxCursor(entries[0].seq))
        .expect("ack the first");
    let cursor = store.outbox_acked(&scope("main")).expect("acked");
    assert_eq!(cursor, OutboxCursor(entries[0].seq));

    let remaining = store.read_outbox(&scope("main"), cursor).expect("read");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].record.name, "second");

    // Acking backwards never rewinds the durable position.
    store
        .ack_outbox(&scope("main"), OutboxCursor(0))
        .expect("ack backwards");
    assert_eq!(store.outbox_acked(&scope("main")).expect("acked"), cursor);
}

fn acking_past_the_outbox_end_is_refused() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("store");
    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("auth"), "v1");
    store.commit(uow, vec![event("only")]).expect("commit");

    // Acknowledging past the end would silently skip everything committed
    // later.
    match store.ack_outbox(&scope("main"), OutboxCursor(99)) {
        Err(StoreError::Backend { source }) => {
            assert!(source.contains("99"), "the reason should name the cursor: {source}")
        }
        other => panic!("expected backend, got {other:?}"),
    }
    assert_eq!(store.outbox_acked(&scope("main")).expect("acked"), OutboxCursor(0));
}

fn history_records_writes_and_tombstones() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("store");

    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("create"))
        .expect("begin");
    uow.create(spec("auth"), "v1");
    let created = store.commit(uow, vec![event("created")]).expect("create");

    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("edit"))
        .expect("begin");
    uow.update(spec("auth"), "v2", created);
    let edited = store.commit(uow, vec![event("edited")]).expect("edit");

    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("delete"))
        .expect("begin");
    uow.delete(spec("auth"), edited);
    store.commit(uow, vec![event("deleted")]).expect("delete");

    let snapshot = store.snapshot(&scope("main")).expect("snapshot");
    let history = snapshot.history(&spec("auth")).expect("history");
    assert_eq!(history.len(), 3);
    assert!(matches!(history[0].kind, RevisionKind::Write { .. }));
    assert!(matches!(history[1].kind, RevisionKind::Write { .. }));
    assert!(matches!(history[2].kind, RevisionKind::Tombstone));
    assert_eq!(history[2].command, "delete");
    assert_eq!(history[0].actor, "tester");

    // A deleted document reads as absent, and its history survives it.
    assert!(snapshot.read(&spec("auth")).expect("read").is_none());
}

pub fn tests() -> &'static [(&'static str, fn())] {
    &[
        ("export_and_import_round_trip", export_and_import_round_trip),
        (
            "import_create_new_refuses_a_scope_that_holds_any_document",
            import_create_new_refuses_a_scope_that_holds_any_document,
        ),
        (
            "import_rejects_a_tampered_bundle_and_writes_nothing",
            import_rejects_a_tampered_bundle_and_writes_nothing,
        ),
        (
            "bundles_agree_with_the_sqlite_driver",
            bundles_agree_with_the_sqlite_driver,
        ),
        (
            "outbox_reads_from_a_cursor_and_acks_monotonically",
            outbox_reads_from_a_cursor_and_acks_monotonically,
        ),
        (
            "acking_past_the_outbox_end_is_refused",
            acking_past_the_outbox_end_is_refused,
        ),
        (
            "history_records_writes_and_tombstones",
            history_records_writes_and_tombstones,
        ),
    ]
}
