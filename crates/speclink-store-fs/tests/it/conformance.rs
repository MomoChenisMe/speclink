//! The FS driver against the shared TeamStore conformance suite.
//!
//! The harness backs the suite with a real data directory in a tempdir.
//! `arm_crash` uses the driver's fault hook: at the armed stage boundary the
//! commit abandons its work *before* the index rename, so the index on disk
//! still describes the world as it was; `restart` then drops the store —
//! releasing the OS lock exactly as a dying process would — and opens the
//! same directory afresh, which sweeps the abandoned files and sees only
//! commits that were published. The crash is invisible, as the contract
//! requires.

use speclink_store::conformance::{run, StoreHarness};
use speclink_store::{Capability, CapabilityLevel, FaultPoint, TeamStore, CONTRACT_VERSION};
use speclink_store_fs::FsTeamStore;
use tempfile::TempDir;

struct FsHarness {
    dir: TempDir,
    /// Optional so the store can be dropped — and its lock released — before
    /// the next one opens the same directory. A store held while its
    /// successor opens would collide with its own single-writer lock.
    store: Option<FsTeamStore>,
}

impl FsHarness {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = FsTeamStore::open(dir.path()).expect("open");
        Self {
            dir,
            store: Some(store),
        }
    }

    fn driver(&self) -> &FsTeamStore {
        self.store.as_ref().expect("a store is open")
    }
}

impl StoreHarness for FsHarness {
    fn reset(&mut self) -> &dyn TeamStore {
        // Release the old store before its directory goes away, then hand
        // out a fresh, empty one.
        self.store = None;
        let dir = tempfile::tempdir().expect("tempdir");
        self.store = Some(FsTeamStore::open(dir.path()).expect("open"));
        self.dir = dir;
        self.driver()
    }

    fn store(&self) -> &dyn TeamStore {
        self.driver()
    }

    fn arm_crash(&mut self, point: FaultPoint) {
        self.driver().crash_at(point);
    }

    fn arm_outbox_failure(&mut self) {
        self.driver().fail_outbox_append();
    }

    fn restart(&mut self) {
        // Rebuild from durable state. Dropping first is what a crash does to
        // a process: the lock goes with it, and nothing gets a chance to
        // tidy up the half-written files — the reopen sweeps them.
        self.store = None;
        self.store = Some(FsTeamStore::open(self.dir.path()).expect("reopen"));
    }
}

#[test]
fn fs_driver_passes_the_full_conformance_suite() {
    let mut harness = FsHarness::new();
    let report = run(&mut harness);

    assert!(report.passed, "conformance failures: {:#?}", report.failures);
    assert_eq!(report.contract_version, CONTRACT_VERSION);
    assert_eq!(report.driver, "serverfs");
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
    // Single-node is a promise about what this driver does *not* do: one
    // writer, one node. Declaring cluster would claim guarantees an advisory
    // file lock cannot make.
    assert!(!report.capabilities.contains(&Capability::Cluster));
}
