//! Requirement「可用性檢查與失敗的分類」.

use crate::support::{
    all, commit, key, new_store, not_checked, open_error, put, scope, v, with_param, Link, Test,
    TestDb,
};
use postgres::{Client, NoTls};
use speclink_store_port::{Expected, PortError, StorePort, Version};
use speclink_store_port_postgres::{url_embeds_password, OpenError, PostgresPort, PASSWORD_VAR};
use std::thread;
use std::time::{Duration, Instant};

fn probe_succeeds_on_an_open_store() {
    let (_db, store) = new_store();
    assert_eq!(store.probe(), Ok(()));
}

fn the_next_call_after_a_terminated_connection_succeeds() {
    let mut db = TestDb::new();
    let name = format!("port-test-{}", db.schema);
    let store = PostgresPort::connect(&with_param(db.url(), "application_name", &name)).unwrap();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("k", "before", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    // The server ends the store's connection; the server itself stays up.
    let terminated = db
        .admin
        .query(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE application_name = $1",
            &[&name],
        )
        .unwrap();
    assert_eq!(terminated.len(), 1);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let live: i64 = db
            .admin
            .query_one(
                "SELECT count(*) FROM pg_stat_activity WHERE application_name = $1",
                &[&name],
            )
            .unwrap()
            .get(0);
        if live == 0 {
            break;
        }
        assert!(Instant::now() < deadline, "the connection did not end");
        thread::sleep(Duration::from_millis(20));
    }

    let listing = store.list(&s, &all()).unwrap();
    assert_eq!(listing.version, v(1));
    assert_eq!(listing.entries[0].key, key("k"));
    assert_eq!(
        store.commit(
            &s,
            &commit(vec![put("k", "after", Expected::At(v(1)))], Vec::new())
        ),
        Ok(v(2))
    );
}

fn an_outage_reports_unavailable_and_the_store_recovers() {
    let db = TestDb::new();
    let (host, port) = db.upstream();
    let link = Link::to((&host, port));
    let store =
        PostgresPort::connect(&db.connection_string(Some(("127.0.0.1", link.port)), None)).unwrap();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("k", "v1", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    link.cut();

    // More than once: the store must continue to say so.
    for attempt in 1..=3 {
        let result = store.read(&s, &[key("k")]);
        assert!(
            matches!(result, Err(PortError::Unavailable { .. })),
            "read {attempt} in the outage: {result:?}"
        );
    }
    let probe = store.probe();
    assert!(
        matches!(probe, Err(PortError::Unavailable { .. })),
        "{probe:?}"
    );
    let result = store.commit(
        &s,
        &commit(vec![put("k", "v2", Expected::At(v(1)))], Vec::new()),
    );
    assert!(
        matches!(result, Err(PortError::Unavailable { .. })),
        "a commit in the outage: {result:?}"
    );

    link.restore();

    // The same object recovers, and the failed commit left nothing.
    assert_eq!(
        store.read(&s, &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "v1"
    );
    assert_eq!(store.probe(), Ok(()));
    assert_eq!(
        store.commit(
            &s,
            &commit(vec![put("k", "v2", Expected::At(v(1)))], Vec::new())
        ),
        Ok(v(2))
    );
}

fn the_environment_password_wins_over_the_url_password() {
    let db = TestDb::new();
    let Some(right) = db.password() else {
        not_checked("the test URL has no password, so the order cannot be seen");
    };
    // A server that trusts every local connection accepts a wrong password.
    let server_checks = Client::connect(
        &db.connection_string(None, Some("certainly-not-the-password")),
        NoTls,
    )
    .is_err();

    // The variable belongs to the whole process; this runner has one thread.
    std::env::set_var(PASSWORD_VAR, &right);
    let wrong_in_the_url =
        PostgresPort::connect(&db.connection_string(None, Some("wrong-in-the-url")));
    std::env::set_var(PASSWORD_VAR, "wrong-in-the-environment");
    let wrong_in_the_environment = PostgresPort::connect(db.url());
    std::env::remove_var(PASSWORD_VAR);

    assert!(wrong_in_the_url.is_ok(), "{:?}", wrong_in_the_url.err());
    if !server_checks {
        not_checked("the server accepts a wrong password, so the order cannot be seen");
    }
    let err = open_error(wrong_in_the_environment);
    assert!(matches!(err, OpenError::Store(_)), "{err:?}");
    assert!(!err.to_string().contains("wrong-in-the-environment"));
    assert!(!err.to_string().contains(&right));
}

fn url_embeds_password_reports_a_password_in_the_url() {
    assert!(url_embeds_password("postgres://user:s3cret@localhost/db"));
    assert!(!url_embeds_password("postgres://user@localhost/db"));
    assert!(url_embeds_password("host=localhost user=u password=s3cret"));
    assert!(!url_embeds_password("host=localhost user=u"));
}

fn a_connect_error_does_not_show_the_password() {
    // `.invalid` is a reserved name that never resolves.
    let url = "postgres://user:s3cret@speclink-no-such-host.invalid:5432/db?connect_timeout=2";
    let err = open_error(PostgresPort::connect(url));
    assert!(matches!(err, OpenError::Store(_)), "{err:?}");
    assert!(!err.to_string().contains("s3cret"), "{err}");
    assert!(!format!("{err:?}").contains("s3cret"), "{err:?}");
}

fn probe_fails_when_the_database_cannot_be_written() {
    let db = TestDb::new();
    drop(db.store());
    // A session that can only read, as on a standby server.
    let store = PostgresPort::connect(&db.read_only_connection_string()).unwrap();

    assert_eq!(
        store.list(&scope("s"), &all()).unwrap().version,
        Version::ZERO
    );
    let result = store.probe();
    assert!(
        matches!(result, Err(PortError::Backend { .. })),
        "{result:?}"
    );
}

/// The tests of this module that need no database.
pub const NO_DATABASE: &[Test] = &[
    (
        "url_embeds_password_reports_a_password_in_the_url",
        url_embeds_password_reports_a_password_in_the_url,
    ),
    (
        "a_connect_error_does_not_show_the_password",
        a_connect_error_does_not_show_the_password,
    ),
];

/// The tests of this module that need a database.
pub const TESTS: &[Test] = &[
    (
        "probe_succeeds_on_an_open_store",
        probe_succeeds_on_an_open_store,
    ),
    (
        "the_next_call_after_a_terminated_connection_succeeds",
        the_next_call_after_a_terminated_connection_succeeds,
    ),
    (
        "an_outage_reports_unavailable_and_the_store_recovers",
        an_outage_reports_unavailable_and_the_store_recovers,
    ),
    (
        "the_environment_password_wins_over_the_url_password",
        the_environment_password_wins_over_the_url_password,
    ),
    (
        "probe_fails_when_the_database_cannot_be_written",
        probe_fails_when_the_database_cannot_be_written,
    ),
];
