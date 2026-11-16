//! Requirement「三個範例通過合格測試」.

use crate::support::{all, commit, commit_at, key, new_store, put, scope, v};
use speclink_store_port::conformance::{self, CheckOutcome, PortHarness};
use speclink_store_port::{Expected, PortError, RecordedEvent, ScopeId, StorePort};
use speclink_store_port_sqlite::SqlitePort;
use std::sync::atomic::{AtomicU64, Ordering};

struct Harness {
    store: SqlitePort,
    next_scope: AtomicU64,
}

impl PortHarness for Harness {
    fn store(&self) -> &dyn StorePort {
        &self.store
    }

    fn new_scope(&self) -> Result<ScopeId, PortError> {
        let n = self.next_scope.fetch_add(1, Ordering::Relaxed);
        Ok(scope(&format!("conformance-{n}")))
    }

    fn recorded_events(&self, scope: &ScopeId) -> Result<Option<Vec<RecordedEvent>>, PortError> {
        let events = self.store.events_after(scope, 0, usize::MAX)?;
        Ok(Some(events.into_iter().map(|(_, event)| event).collect()))
    }
}

#[test]
fn conformance_passes_all_sixteen_checks_without_a_skip() {
    let (_dir, store) = new_store();
    let report = conformance::run(&Harness {
        store,
        next_scope: AtomicU64::new(0),
    });
    let not_passed: Vec<_> = report
        .checks
        .iter()
        .filter(|check| check.outcome != CheckOutcome::Passed)
        .collect();
    assert!(not_passed.is_empty(), "{not_passed:#?}");
    assert_eq!(report.checks.len(), 16);
}

#[test]
fn modified_at_is_the_commit_time() {
    let (_dir, store) = new_store();
    let s = scope("s");
    let doc = "changes/a/proposal.md";
    store
        .commit(
            &s,
            &commit_at(
                1_700_000_000_000,
                vec![put(doc, "text", Expected::Absent)],
                Vec::new(),
            ),
        )
        .unwrap();
    let found = store.read(&s, &[key(doc)]).unwrap();
    assert_eq!(
        found[0].as_ref().unwrap().modified_at.get(),
        1_700_000_000_000
    );
    let listing = store.list(&s, &all()).unwrap();
    assert_eq!(listing.entries[0].modified_at.get(), 1_700_000_000_000);
}

#[test]
fn each_scope_has_its_own_version() {
    let (_dir, store) = new_store();
    let (a, b) = (scope("A"), scope("B"));
    store
        .commit(
            &a,
            &commit(vec![put("a1", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    store
        .commit(
            &a,
            &commit(vec![put("a2", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    store
        .commit(
            &b,
            &commit(vec![put("b1", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    let in_a = store.list(&a, &all()).unwrap();
    let in_b = store.list(&b, &all()).unwrap();
    assert_eq!(in_a.version, v(2));
    assert_eq!(in_b.version, v(1));
    let keys_in_b: Vec<&str> = in_b.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys_in_b, ["b1"]);
    assert_eq!(store.read(&b, &[key("a1")]).unwrap(), [None]);
}
