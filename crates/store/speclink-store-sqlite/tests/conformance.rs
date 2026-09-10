//! The SQLite driver against the shared TeamStore conformance suite.
//!
//! The harness backs the suite with a real database file in a tempdir.
//! `arm_crash` uses the driver's fault hook: at the armed stage boundary the
//! commit returns without committing its SQL transaction (which rolls back on
//! drop) and poisons the connection; `restart` then opens a fresh connection
//! to the same path, which sees only committed commits — the crash is
//! invisible, exactly as the contract requires.

use speclink_store::conformance::{run, StoreHarness};
use speclink_store::{Capability, CapabilityLevel, FaultPoint, TeamStore, CONTRACT_VERSION};
use speclink_store_sqlite::SqliteTeamStore;
use std::path::PathBuf;
use tempfile::TempDir;

struct SqliteHarness {
    dir: TempDir,
    store: SqliteTeamStore,
}

impl SqliteHarness {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = SqliteTeamStore::open(dir.path().join("store.db")).expect("open");
        Self { dir, store }
    }

    fn db_path(&self) -> PathBuf {
        self.dir.path().join("store.db")
    }
}

impl StoreHarness for SqliteHarness {
    fn reset(&mut self) -> &dyn TeamStore {
        // A fresh, empty store in a fresh tempdir. Assign the store first so
        // the old connection closes before its files are removed.
        let dir = tempfile::tempdir().expect("tempdir");
        let store = SqliteTeamStore::open(dir.path().join("store.db")).expect("open");
        self.store = store;
        self.dir = dir;
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
        // Rebuild from durable state: a new connection to the same file. The
        // crashed store's uncommitted transaction already rolled back, so the
        // reopened store carries only committed state.
        self.store = SqliteTeamStore::open(self.db_path()).expect("reopen");
    }
}

#[test]
fn sqlite_driver_passes_the_full_conformance_suite() {
    let mut harness = SqliteHarness::new();
    let report = run(&mut harness);

    assert!(report.passed, "conformance failures: {:#?}", report.failures);
    assert_eq!(report.contract_version, CONTRACT_VERSION);
    assert_eq!(report.driver, "sqlite");
    assert_eq!(report.level, CapabilityLevel::SingleNode);
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
            "missing verified capability {capability:?}"
        );
    }
    assert!(!report.capabilities.contains(&Capability::Cluster));
}
