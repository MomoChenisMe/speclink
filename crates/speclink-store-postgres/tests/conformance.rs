//! The contract's own suite, run against a real PostgreSQL instance.
//!
//! This is the driver's acceptance surface: the same gates every other driver
//! passes, over the same behaviour, with nothing driver-specific asserted here.

mod support;

use speclink_store::conformance::{run, StoreHarness};
use speclink_store::{Capability, CapabilityLevel, FaultPoint, TeamStore, CONTRACT_VERSION};
use speclink_store_postgres::PostgresTeamStore;
use support::TestDb;

/// Each `reset` hands out a store on a schema of its own, so the suite's
/// fixtures never meet a previous gate's rows.
struct PostgresHarness {
    db: TestDb,
    store: PostgresTeamStore,
}

impl PostgresHarness {
    fn new() -> Self {
        let db = TestDb::new();
        let store = PostgresTeamStore::connect(db.url()).expect("connect");
        Self { db, store }
    }
}

impl StoreHarness for PostgresHarness {
    fn reset(&mut self) -> &dyn TeamStore {
        let db = TestDb::new();
        let store = PostgresTeamStore::connect(db.url()).expect("connect");
        // Assign the store first: this drops the previous one, closing its
        // connection before the line below drops the schema it was sitting on.
        self.store = store;
        self.db = db;
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
        // A brand-new connection onto the same schema. Nothing of the crashed
        // store survives into it, which is what makes it able to say what
        // actually landed.
        self.store = PostgresTeamStore::connect(self.db.url()).expect("reopen");
    }
}

fn postgres_driver_passes_the_full_conformance_suite() {
    let mut harness = PostgresHarness::new();
    let report = run(&mut harness);

    assert!(report.passed, "conformance failures: {:#?}", report.failures);
    assert_eq!(report.contract_version, CONTRACT_VERSION);
    assert_eq!(report.driver, "postgres");
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
            "the manifest does not declare {capability:?}"
        );
    }
    // Serializing writers is not coordinating a cluster, and the manifest must
    // not suggest otherwise.
    assert!(
        !report.capabilities.contains(&Capability::Cluster),
        "a single-node driver declared cluster"
    );
}

fn main() {
    support::run(&[(
        "postgres_driver_passes_the_full_conformance_suite",
        postgres_driver_passes_the_full_conformance_suite,
    )]);
}
