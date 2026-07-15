//! The open gate: what the driver accepts, what it refuses, and what a refusal
//! is allowed to touch.
//!
//! The rule under test is fail-closed — detection is read-only, so a database
//! the driver goes on to refuse keeps every byte it had.

mod support;

use postgres::{Client, NoTls};
use speclink_store::{StoreError, TeamStore, CONTRACT_VERSION};
use speclink_store_postgres::PostgresTeamStore;
use support::TestDb;

/// Open a store, or panic naming the error — `PostgresTeamStore` is not
/// `Debug`, so `expect` on the `Result` is not available.
fn expect_corrupt(url: &str) -> String {
    match PostgresTeamStore::connect(url) {
        Err(StoreError::Corrupt { reason }) => reason,
        Err(other) => panic!("expected corrupt, got {other:?}"),
        Ok(_) => panic!("expected corrupt, but the store opened"),
    }
}

fn expect_backend(url: &str) -> String {
    match PostgresTeamStore::connect(url) {
        Err(StoreError::Backend { source }) => source,
        Err(other) => panic!("expected backend, got {other:?}"),
        Ok(_) => panic!("expected backend, but the store opened"),
    }
}

fn empty_schema_initializes_at_the_current_version() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("initialize an empty schema");
    assert_eq!(store.health(), Ok(()));

    let mut client = Client::connect(db.url(), NoTls).expect("inspect the schema");
    let marker: String = client
        .query_one("SELECT value FROM meta WHERE key = 'format'", &[])
        .expect("read the marker")
        .get(0);
    assert_eq!(marker, "speclink-team-store");
    let version: String = client
        .query_one("SELECT value FROM meta WHERE key = 'schema_version'", &[])
        .expect("read the version")
        .get(0);
    assert_eq!(version, "1");

    // Re-opening an initialized store is the ordinary path, not a re-init.
    let reopened = PostgresTeamStore::connect(db.url()).expect("reopen");
    assert_eq!(reopened.health(), Ok(()));
}

fn newer_version_is_refused_and_nothing_is_written() {
    let db = TestDb::new();
    PostgresTeamStore::connect(db.url()).expect("initialize");

    let mut client = Client::connect(db.url(), NoTls).expect("doctor the schema");
    client
        .execute("UPDATE meta SET value = '2' WHERE key = 'schema_version'", &[])
        .expect("record a version this driver does not know");
    client
        .execute(
            "INSERT INTO documents (project, repo, doc_id, content, revision, digest) \
             VALUES ('acme', 'main', 'wc', 'body', 1, 'sha256:x')",
            &[],
        )
        .expect("seed a document the refusal must not touch");

    let reason = expect_corrupt(db.url());
    assert!(
        reason.contains('2'),
        "the reason should name the version it found: {reason}"
    );

    let version: String = client
        .query_one("SELECT value FROM meta WHERE key = 'schema_version'", &[])
        .expect("re-read the version")
        .get(0);
    assert_eq!(version, "2", "the refused open rewrote the version");
    let content: String = client
        .query_one("SELECT content FROM documents WHERE doc_id = 'wc'", &[])
        .expect("re-read the document")
        .get(0);
    assert_eq!(content, "body", "the refused open touched the data");
}

fn foreign_schema_is_refused_and_not_initialized() {
    let db = TestDb::new();
    let mut client = Client::connect(db.url(), NoTls).expect("seed an unrelated schema");
    client
        .batch_execute("CREATE TABLE unrelated (x INT)")
        .expect("create a table that is not ours");

    expect_corrupt(db.url());

    let initialized: bool = client
        .query_one("SELECT to_regclass('meta') IS NOT NULL", &[])
        .expect("look for our meta table")
        .get(0);
    assert!(
        !initialized,
        "a refused open initialized the schema it had just rejected"
    );
}

fn older_version_migrates_and_keeps_data() {
    let db = TestDb::new();
    PostgresTeamStore::connect(db.url()).expect("initialize");

    let mut client = Client::connect(db.url(), NoTls).expect("doctor the schema");
    client
        .execute(
            "INSERT INTO documents (project, repo, doc_id, content, revision, digest) \
             VALUES ('acme', 'main', 'wc', 'spec body', 1, 'sha256:x')",
            &[],
        )
        .expect("seed data that must survive the migration");
    client
        .execute("UPDATE meta SET value = '0' WHERE key = 'schema_version'", &[])
        .expect("wind the version back");

    let store = PostgresTeamStore::connect(db.url()).expect("an older version still opens");
    assert_eq!(
        store.health(),
        Err(StoreError::Unavailable),
        "an unmigrated store is not ready to serve"
    );
    store.migrate(CONTRACT_VERSION).expect("migrate to current");
    assert_eq!(store.health(), Ok(()));

    let version: String = client
        .query_one("SELECT value FROM meta WHERE key = 'schema_version'", &[])
        .expect("re-read the version")
        .get(0);
    assert_eq!(version, "1");
    let content: String = client
        .query_one("SELECT content FROM documents WHERE doc_id = 'wc'", &[])
        .expect("re-read the document")
        .get(0);
    assert_eq!(content, "spec body", "the migration lost data");
}

fn migrating_to_an_unknown_version_is_refused() {
    let db = TestDb::new();
    let store = PostgresTeamStore::connect(db.url()).expect("initialize");
    match store.migrate(CONTRACT_VERSION + 1) {
        Err(StoreError::Backend { source }) => assert!(
            source.contains(&(CONTRACT_VERSION + 1).to_string()),
            "the reason should name the version asked for: {source}"
        ),
        other => panic!("expected backend, got {other:?}"),
    }
}

fn authentication_failure_is_a_backend_error_naming_the_cause() {
    let db = TestDb::new();
    let source = expect_backend(&support::with_param(db.url(), "user", "speclink_no_such_role"));
    assert!(
        source.contains("speclink_no_such_role"),
        "the reason should carry the server's complaint: {source}"
    );
}

fn missing_database_is_a_backend_error_naming_the_cause() {
    let db = TestDb::new();
    let source = expect_backend(&support::with_param(db.url(), "dbname", "speclink_no_such_db"));
    assert!(
        source.contains("speclink_no_such_db"),
        "the reason should carry the server's complaint: {source}"
    );
}

fn main() {
    support::run(&[
        (
            "empty_schema_initializes_at_the_current_version",
            empty_schema_initializes_at_the_current_version,
        ),
        (
            "newer_version_is_refused_and_nothing_is_written",
            newer_version_is_refused_and_nothing_is_written,
        ),
        (
            "foreign_schema_is_refused_and_not_initialized",
            foreign_schema_is_refused_and_not_initialized,
        ),
        (
            "older_version_migrates_and_keeps_data",
            older_version_migrates_and_keeps_data,
        ),
        (
            "migrating_to_an_unknown_version_is_refused",
            migrating_to_an_unknown_version_is_refused,
        ),
        (
            "authentication_failure_is_a_backend_error_naming_the_cause",
            authentication_failure_is_a_backend_error_naming_the_cause,
        ),
        (
            "missing_database_is_a_backend_error_naming_the_cause",
            missing_database_is_a_backend_error_naming_the_cause,
        ),
    ]);
}
