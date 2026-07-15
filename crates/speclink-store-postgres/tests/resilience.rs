//! What an outage does to a running store.
//!
//! A lost link is not corruption and not a backend fault — the durable state is
//! intact and the schema has already passed the open gate. The store says
//! `unavailable` for as long as the server is out of reach, it never panics,
//! and the same instance carries on once the link is back.
//!
//! The tests route the store through a forwarder they can cut (`support::Link`)
//! rather than terminating its backend. Terminating leaves the server up, so
//! the store just reconnects — and there is no window in which `unavailable` is
//! the required answer.

mod support;

use speclink_store::{StoreError, TeamStore};
use speclink_store_postgres::PostgresTeamStore;
use support::{ctx, event, scope, spec, Link, TestDb};

fn an_outage_reports_unavailable_and_the_store_recovers_when_it_ends() {
    let db = TestDb::new();
    let (host, port) = db.upstream();
    let link = Link::to((&host, port));
    let store = PostgresTeamStore::connect(&db.url_via("127.0.0.1", link.port())).expect("store");

    let seeded = {
        let mut uow = store
            .begin_unit_of_work(&scope("main"), ctx("create"))
            .expect("begin");
        uow.create(spec("auth"), "v1");
        store.commit(uow, vec![event("created")]).expect("seed")
    };

    link.cut();

    // Reads report unavailable for as long as the outage lasts. Repeated,
    // because the store must keep saying so — a store that reported the outage
    // once and then found some other answer would be worse than one that never
    // noticed.
    for attempt in 1..=3 {
        match store.snapshot(&scope("main")) {
            Err(StoreError::Unavailable) => {}
            Err(other) => panic!("read {attempt} during the outage: expected unavailable, got {other:?}"),
            Ok(_) => panic!("read {attempt} during the outage somehow succeeded"),
        }
    }

    // Writes too — and a write that cannot reach the server must not report
    // success.
    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("edit"))
        .expect("begin");
    uow.update(spec("auth"), "v2", seeded);
    match store.commit(uow, vec![event("edited")]) {
        Err(StoreError::Unavailable) => {}
        Err(other) => panic!("a commit during the outage: expected unavailable, got {other:?}"),
        Ok(revision) => panic!("a commit during the outage reported success at {revision:?}"),
    }

    link.restore();

    // The same instance recovers: no reopen, no new store.
    let snapshot = store.snapshot(&scope("main")).expect("the store recovers");
    assert_eq!(snapshot.revision(), seeded, "the outage moved the revision");
    assert_eq!(
        snapshot
            .read(&spec("auth"))
            .expect("read")
            .expect("the document exists")
            .content,
        "v1",
        "the commit that failed during the outage left a partial write"
    );
    drop(snapshot);

    // ...and it picks up from where it left off rather than from a fresh count.
    let mut uow = store
        .begin_unit_of_work(&scope("main"), ctx("edit"))
        .expect("begin");
    uow.update(spec("auth"), "v2", seeded);
    let next = store
        .commit(uow, vec![event("edited")])
        .expect("commit after recovery");
    assert!(
        next.0 > seeded.0,
        "the recovered store restarted its revisions at {next:?}, after {seeded:?}"
    );

    let snapshot = store.snapshot(&scope("main")).expect("read back");
    assert_eq!(
        snapshot
            .read(&spec("auth"))
            .expect("read")
            .expect("the document exists")
            .content,
        "v2"
    );
}

fn main() {
    support::run(&[(
        "an_outage_reports_unavailable_and_the_store_recovers_when_it_ends",
        an_outage_reports_unavailable_and_the_store_recovers_when_it_ends,
    )]);
}
