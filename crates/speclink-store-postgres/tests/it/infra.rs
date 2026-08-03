//! The test scaffolding's own gate.
//!
//! Every other test target in this crate leans on `support::TestDb` for
//! isolation and on `support::run` for the skip-when-unconfigured rule, so the
//! scaffolding itself is worth holding to a test.

use crate::support;

use postgres::{Client, NoTls};
use support::TestDb;

/// A store built from the handed-out URL lands in the private schema.
fn isolated_schema_is_created_and_reachable() {
    let db = TestDb::new();
    let mut client = Client::connect(db.url(), NoTls).expect("connect via the test url");
    let current: String = client
        .query_one("SELECT current_schema()::text", &[])
        .expect("read current_schema")
        .get(0);
    assert_eq!(current, db.schema());
}

/// Two instances never share a namespace: what one creates, the other cannot
/// see. This is what lets the suite run against one shared server.
fn each_test_db_gets_its_own_schema() {
    let first = TestDb::new();
    let second = TestDb::new();
    assert_ne!(first.schema(), second.schema());

    let mut in_first = Client::connect(first.url(), NoTls).expect("connect to the first schema");
    in_first
        .batch_execute("CREATE TABLE marker (x INT)")
        .expect("create a table in the first schema");

    let mut in_second = Client::connect(second.url(), NoTls).expect("connect to the second schema");
    let visible: bool = in_second
        .query_one("SELECT to_regclass('marker') IS NOT NULL", &[])
        .expect("look for the first schema's table")
        .get(0);
    assert!(!visible, "a table created in one test schema was visible from another");
}

/// Cleanup is not best-effort housekeeping: without it a shared server
/// accumulates a schema per test forever.
fn dropping_the_test_db_removes_its_schema() {
    let name = {
        let db = TestDb::new();
        let mut client = Client::connect(db.url(), NoTls).expect("connect via the test url");
        client
            .batch_execute("CREATE TABLE leftover (x INT)")
            .expect("leave a populated schema behind");
        db.schema().to_string()
    };

    let mut admin = Client::connect(&support::base_url().expect("configured"), NoTls)
        .expect("connect to the base database");
    let exists: bool = admin
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = $1)",
            &[&name],
        )
        .expect("look the schema up")
        .get(0);
    assert!(!exists, "schema {name} survived its TestDb");
}

pub fn tests() -> &'static [(&'static str, fn())] {
    &[
        (
            "isolated_schema_is_created_and_reachable",
            isolated_schema_is_created_and_reachable,
        ),
        (
            "each_test_db_gets_its_own_schema",
            each_test_db_gets_its_own_schema,
        ),
        (
            "dropping_the_test_db_removes_its_schema",
            dropping_the_test_db_removes_its_schema,
        ),
    ]
}
