//! Single-writer semantics across connections, and the reading side's
//! consistency.
//!
//! The mutex inside one store instance would serialize writers all by itself,
//! which is why every test here uses **two store instances** — two real
//! connections to the same schema. That is the only shape in which the
//! advisory lock is doing the work, and it is the shape the deployment has
//! when two server processes point at one database.

use crate::support;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use postgres::{Client, NoTls};
use speclink_store::{ExpectedRevision, Revision, StoreError, TeamStore};
use speclink_store_postgres::PostgresTeamStore;
use support::{ctx, event, scope, spec, TestDb};

/// How long a commit that must not be blocked is given before we call it
/// blocked. Generous: this is a liveness assertion, not a benchmark.
const UNBLOCKED: Duration = Duration::from_secs(10);

/// Stage a create of `capability` and commit it.
fn create(store: &dyn TeamStore, repo: &str, capability: &str, body: &str) -> Result<Revision, StoreError> {
    let scope = scope(repo);
    let mut uow = store
        .begin_unit_of_work(&scope, ctx("create"))
        .expect("begin");
    uow.create(spec(capability), body);
    store.commit(uow, vec![event("created")])
}

/// Two connections race the same precondition. Exactly one may win, and the
/// loser must be told what it collided with — not merely that it failed.
fn concurrent_commits_on_the_same_revision_leave_exactly_one_winner() {
    let db = TestDb::new();
    let first = PostgresTeamStore::connect(db.url()).expect("first connection");
    let second = PostgresTeamStore::connect(db.url()).expect("second connection");

    let (left, right) = thread::scope(|s| {
        let left = s.spawn(|| create(&first, "main", "auth", "from the first"));
        let right = s.spawn(|| create(&second, "main", "auth", "from the second"));
        (left.join().expect("left"), right.join().expect("right"))
    });

    let (winner, loser) = match (left, right) {
        (Ok(revision), Err(loss)) => (revision, loss),
        (Err(loss), Ok(revision)) => (revision, loss),
        (Ok(a), Ok(b)) => panic!("both commits won, at {a:?} and {b:?}"),
        (Err(a), Err(b)) => panic!("both commits lost, with {a:?} and {b:?}"),
    };

    match loser {
        StoreError::RevisionConflict {
            doc,
            expected,
            actual,
        } => {
            assert_eq!(doc.doc, spec("auth"));
            assert_eq!(expected, ExpectedRevision::Absent);
            assert_eq!(
                actual,
                Some(winner),
                "the conflict must name the revision that beat it"
            );
        }
        other => panic!("the loser should see a revision conflict, got {other:?}"),
    }

    // The loser's connection goes away. Its lock was transaction-scoped, so
    // nothing it held may outlive it: a fresh connection writes unimpeded.
    drop(second);

    let (done, finished) = mpsc::channel();
    let url = db.url().to_string();
    thread::spawn(move || {
        let third = PostgresTeamStore::connect(&url).expect("third connection");
        let outcome = create(&third, "main", "billing", "after the loser left");
        done.send(outcome).ok();
    });
    match finished.recv_timeout(UNBLOCKED) {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => panic!("the write after the loser left failed: {e:?}"),
        Err(_) => panic!("a write after the loser left blocked on a leftover lock"),
    }
}

/// The lock is per scope, not global. Racing two commits and watching both
/// succeed would prove nothing — a global lock would serialize them and both
/// would still land. So this holds one scope's lock open on a connection of
/// its own, which is precisely what a mid-commit writer holds, and watches who
/// waits for it.
fn commits_in_different_scopes_do_not_block_each_other() {
    let db = TestDb::new();
    PostgresTeamStore::connect(db.url()).expect("initialize the schema");

    let mut holder = Client::connect(db.url(), NoTls).expect("holder connection");
    let mut held = holder.transaction().expect("begin");
    held.execute(
        "SELECT pg_advisory_xact_lock($1)",
        &[&PostgresTeamStore::advisory_lock_key(&scope("main"))],
    )
    .expect("hold the 'main' scope's lock");

    // A commit into a different scope must not wait for it.
    let (elsewhere, elsewhere_done) = mpsc::channel();
    let url = db.url().to_string();
    thread::spawn(move || {
        let store = PostgresTeamStore::connect(&url).expect("connect");
        elsewhere.send(create(&store, "docs", "auth", "in docs")).ok();
    });
    match elsewhere_done.recv_timeout(UNBLOCKED) {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => panic!("a commit in an unlocked scope failed: {e:?}"),
        Err(_) => panic!("a commit in scope 'docs' blocked on scope 'main's lock"),
    }

    // ...and a commit into the *held* scope does wait. Without this half, the
    // assertion above would also pass on a store that took no lock at all.
    let (same_scope, same_scope_done) = mpsc::channel();
    let url = db.url().to_string();
    thread::spawn(move || {
        let store = PostgresTeamStore::connect(&url).expect("connect");
        same_scope.send(create(&store, "main", "auth", "in main")).ok();
    });
    assert!(
        same_scope_done.recv_timeout(Duration::from_secs(2)).is_err(),
        "a commit into the held scope did not wait for the lock"
    );

    held.rollback().expect("release the lock");
    match same_scope_done.recv_timeout(UNBLOCKED) {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => panic!("the queued commit failed once the lock was released: {e:?}"),
        Err(_) => panic!("the queued commit never ran once the lock was released"),
    }
}

/// A snapshot is a fixed point. A commit landing on another connection while
/// the snapshot is alive must not reach into it — not the content, and not the
/// revision it reports.
fn a_snapshot_does_not_tear_under_a_concurrent_write() {
    let db = TestDb::new();
    let reader = PostgresTeamStore::connect(db.url()).expect("reader connection");
    let writer = PostgresTeamStore::connect(db.url()).expect("writer connection");

    let seeded = create(&reader, "main", "auth", "v1").expect("seed");

    let snapshot = reader.snapshot(&scope("main")).expect("snapshot");
    assert_eq!(snapshot.revision(), seeded);

    // A whole commit lands on another connection, underneath the open snapshot.
    let scope = scope("main");
    let mut uow = writer
        .begin_unit_of_work(&scope, ctx("edit"))
        .expect("begin");
    uow.update(spec("auth"), "v2", seeded);
    let after = writer.commit(uow, vec![event("edited")]).expect("commit");
    assert_ne!(after, seeded);

    let seen = snapshot
        .read(&spec("auth"))
        .expect("read")
        .expect("the document exists");
    assert_eq!(seen.content, "v1", "the snapshot saw a later write");
    assert_eq!(
        snapshot.revision(),
        seeded,
        "the snapshot's revision moved underneath it"
    );

    // And a snapshot taken after the write does see it — otherwise the test
    // above would pass on a store that simply never reads anything new.
    let fresh = reader.snapshot(&scope).expect("fresh snapshot");
    assert_eq!(fresh.revision(), after);
    assert_eq!(
        fresh.read(&spec("auth")).expect("read").expect("exists").content,
        "v2"
    );
}

/// Readers take no lock. A snapshot that queued behind whichever writer
/// happened to be mid-commit would turn every write into a read stall.
fn a_snapshot_does_not_wait_for_a_scopes_lock() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("store");
    create(&store, "main", "auth", "v1").expect("seed");

    // Hold the scope's write lock, exactly as a mid-commit writer does.
    let mut holder = Client::connect(db.url(), NoTls).expect("holder connection");
    let mut held = holder.transaction().expect("begin");
    held.execute(
        "SELECT pg_advisory_xact_lock($1)",
        &[&PostgresTeamStore::advisory_lock_key(&scope("main"))],
    )
    .expect("hold the 'main' scope's lock");

    let (done, finished) = mpsc::channel();
    let url = db.url().to_string();
    thread::spawn(move || {
        let reader = PostgresTeamStore::connect(&url).expect("reader connection");
        let read = reader.snapshot(&scope("main")).map(|snapshot| {
            snapshot
                .read(&spec("auth"))
                .expect("read")
                .map(|document| document.content)
        });
        done.send(read.map_err(|e| format!("{e:?}"))).ok();
    });

    match finished.recv_timeout(UNBLOCKED) {
        Ok(Ok(content)) => assert_eq!(content.as_deref(), Some("v1")),
        Ok(Err(e)) => panic!("a snapshot failed while a writer held the lock: {e}"),
        Err(_) => panic!("a snapshot waited for the scope's write lock"),
    }

    held.rollback().expect("release the lock");
}

pub fn tests() -> &'static [(&'static str, fn())] {
    &[
        (
            "concurrent_commits_on_the_same_revision_leave_exactly_one_winner",
            concurrent_commits_on_the_same_revision_leave_exactly_one_winner,
        ),
        (
            "a_snapshot_does_not_wait_for_a_scopes_lock",
            a_snapshot_does_not_wait_for_a_scopes_lock,
        ),
        (
            "commits_in_different_scopes_do_not_block_each_other",
            commits_in_different_scopes_do_not_block_each_other,
        ),
        (
            "a_snapshot_does_not_tear_under_a_concurrent_write",
            a_snapshot_does_not_tear_under_a_concurrent_write,
        ),
    ]
}
