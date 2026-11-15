//! Integration tests of the PostgreSQL example, one section per requirement
//! of the `store-port-examples` spec.
//!
//! This binary is its own `main` (`harness = false`). The standard test
//! runner cannot decide at run time to leave a test out, and a test that
//! needs a database must not report `ok` when there is none. With no
//! `SPECLINK_TEST_POSTGRES_URL`, each of those tests is printed as `skipped`
//! and the exit code is 0. With it, each test runs in a new schema of its
//! own, which is dropped at the end.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use postgres::config::Host;
use postgres::{Client, Config, NoTls};
use speclink_store_port::conformance::{self, CheckOutcome, PortHarness};
use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, PortError, RecordedEvent, ScopeId,
    StorePort, Timestamp, Version,
};
use speclink_store_port_postgres::{url_embeds_password, OpenError, PostgresPort, PASSWORD_VAR};

// --- the runner -------------------------------------------------------------

/// The environment variable that names the PostgreSQL instance to test with.
const URL_VAR: &str = "SPECLINK_TEST_POSTGRES_URL";

/// One line that starts a database and points the tests at it.
const ENABLE_HINT: &str = "docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=speclink --name speclink-pg postgres:15 && export SPECLINK_TEST_POSTGRES_URL=postgres://postgres:speclink@localhost:5432/postgres";

type Test = (&'static str, fn());

/// The configured instance. An empty value is the same as no value.
fn base_url() -> Option<String> {
    match std::env::var(URL_VAR) {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => None,
    }
}

fn run(tests: &[Test], failures: &mut Vec<&'static str>) {
    for (name, test) in tests {
        print!("test {name} ... ");
        io::stdout().flush().ok();
        match panic::catch_unwind(AssertUnwindSafe(test)) {
            Ok(()) => println!("ok"),
            Err(_) => {
                println!("FAILED");
                failures.push(name);
            }
        }
    }
}

fn main() {
    let mut failures = Vec::new();
    run(NO_DATABASE, &mut failures);
    let mut ran = NO_DATABASE.len();
    if base_url().is_some() {
        run(DATABASE, &mut failures);
        ran += DATABASE.len();
    } else {
        for (name, _) in DATABASE {
            println!("test {name} ... skipped");
        }
        println!();
        println!(
            "{} skipped: {URL_VAR} names no PostgreSQL instance. Enable them with:",
            DATABASE.len()
        );
        println!("  {ENABLE_HINT}");
    }
    println!();
    println!(
        "test result: {}. {} passed; {} failed",
        if failures.is_empty() { "ok" } else { "FAILED" },
        ran - failures.len(),
        failures.len()
    );
    for name in &failures {
        println!("    {name}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

const NO_DATABASE: &[Test] = &[
    (
        "url_embeds_password_reports_a_password_in_the_url",
        url_embeds_password_reports_a_password_in_the_url,
    ),
    (
        "a_connect_error_does_not_show_the_password",
        a_connect_error_does_not_show_the_password,
    ),
    (
        "a_connection_string_that_cannot_be_read_is_refused_with_the_reason",
        a_connection_string_that_cannot_be_read_is_refused_with_the_reason,
    ),
];

const DATABASE: &[Test] = &[
    (
        "conformance_passes_all_sixteen_checks_without_a_skip",
        conformance_passes_all_sixteen_checks_without_a_skip,
    ),
    (
        "modified_at_is_the_commit_time",
        modified_at_is_the_commit_time,
    ),
    (
        "each_scope_has_its_own_version",
        each_scope_has_its_own_version,
    ),
    (
        "content_with_nul_and_non_ascii_reads_back_the_same",
        content_with_nul_and_non_ascii_reads_back_the_same,
    ),
    (
        "two_events_of_one_commit_get_sequence_1_and_2",
        two_events_of_one_commit_get_sequence_1_and_2,
    ),
    (
        "a_rejected_commit_keeps_no_event_and_uses_no_sequence",
        a_rejected_commit_keeps_no_event_and_uses_no_sequence,
    ),
    (
        "events_after_reads_from_the_middle_up_to_the_limit",
        events_after_reads_from_the_middle_up_to_the_limit,
    ),
    (
        "a_scope_with_no_events_reads_as_empty",
        a_scope_with_no_events_reads_as_empty,
    ),
    (
        "trim_keeps_only_the_newer_events_and_changes_no_document",
        trim_keeps_only_the_newer_events_and_changes_no_document,
    ),
    (
        "the_sequence_continues_after_a_trim",
        the_sequence_continues_after_a_trim,
    ),
    (
        "a_trim_past_the_last_sequence_stops_at_it",
        a_trim_past_the_last_sequence_stops_at_it,
    ),
    (
        "a_trim_at_or_below_the_floor_does_nothing",
        a_trim_at_or_below_the_floor_does_nothing,
    ),
    (
        "a_schema_without_meta_is_initialized_beside_other_tables",
        a_schema_without_meta_is_initialized_beside_other_tables,
    ),
    (
        "another_marker_is_refused_and_no_table_is_added",
        another_marker_is_refused_and_no_table_is_added,
    ),
    (
        "a_meta_table_without_a_marker_is_refused",
        a_meta_table_without_a_marker_is_refused,
    ),
    (
        "a_newer_layout_version_is_refused",
        a_newer_layout_version_is_refused,
    ),
    (
        "a_reconnected_store_has_the_same_content",
        a_reconnected_store_has_the_same_content,
    ),
    (
        "a_commit_that_fails_on_its_second_operation_leaves_nothing",
        a_commit_that_fails_on_its_second_operation_leaves_nothing,
    ),
    (
        "a_commit_by_another_object_makes_the_old_version_conflict",
        a_commit_by_another_object_makes_the_old_version_conflict,
    ),
    (
        "two_objects_that_commit_the_same_absent_key_have_one_winner",
        two_objects_that_commit_the_same_absent_key_have_one_winner,
    ),
    (
        "commits_on_different_scopes_do_not_wait_for_each_other",
        commits_on_different_scopes_do_not_wait_for_each_other,
    ),
    (
        "a_commit_that_cannot_get_the_scope_lock_reports_unavailable",
        a_commit_that_cannot_get_the_scope_lock_reports_unavailable,
    ),
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
        "events_after_with_limit_zero_or_the_largest_number_is_empty",
        events_after_with_limit_zero_or_the_largest_number_is_empty,
    ),
    (
        "a_trim_through_the_largest_number_stops_at_the_last_sequence",
        a_trim_through_the_largest_number_stops_at_the_last_sequence,
    ),
    (
        "a_prefix_matches_bytes_and_has_no_wildcard",
        a_prefix_matches_bytes_and_has_no_wildcard,
    ),
    (
        "a_layout_version_that_is_not_a_number_is_refused_as_it_is",
        a_layout_version_that_is_not_a_number_is_refused_as_it_is,
    ),
    (
        "a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence",
        a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence,
    ),
];

// --- a schema for each test -------------------------------------------------

/// A schema of its own, dropped when the value goes out of scope. A store
/// that connects with [`TestDb::url`] sees only the tables of this schema.
struct TestDb {
    admin: Client,
    base: String,
    schema: String,
    url: String,
}

impl TestDb {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let base = base_url().expect("main() runs these tests only with a database");
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let schema = format!(
            "speclink_port_test_{}_{nanos}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let mut admin = Client::connect(&base, NoTls).expect("connect to the test database");
        admin
            .batch_execute(&format!("CREATE SCHEMA \"{schema}\""))
            .expect("create the test schema");
        // libpq's `options` parameter sets the search path of the session.
        let url = with_param(&base, "options", &format!("-csearch_path%3D{schema}"));
        Self {
            admin,
            base,
            schema,
            url,
        }
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn store(&self) -> PostgresPort {
        PostgresPort::connect(&self.url).unwrap()
    }

    /// A plain connection whose search path is this schema.
    fn client(&self) -> Client {
        Client::connect(&self.url, NoTls).expect("connect to the test schema")
    }

    /// The names of the tables in this schema.
    fn tables(&mut self) -> Vec<String> {
        self.admin
            .query(
                "SELECT tablename::text FROM pg_tables WHERE schemaname = $1 ORDER BY 1",
                &[&self.schema],
            )
            .unwrap()
            .iter()
            .map(|row| row.get(0))
            .collect()
    }

    fn config(&self) -> Config {
        self.base.parse().expect("parse the configured url")
    }

    /// The password in the configured URL.
    fn password(&self) -> Option<String> {
        self.config()
            .get_password()
            .map(|password| String::from_utf8_lossy(password).into_owned())
    }

    /// The TCP address that the configured URL points at.
    fn upstream(&self) -> (String, u16) {
        let config = self.config();
        let host = match config.get_hosts().first() {
            Some(Host::Tcp(host)) => host.clone(),
            _ => panic!("{URL_VAR} must name a TCP host"),
        };
        (host, config.get_ports().first().copied().unwrap_or(5432))
    }

    /// A key/value connection string for this schema. `address` and
    /// `password` replace those parts of the configured URL.
    ///
    /// Key/value form, not a URL: a host or port given as a URL parameter is
    /// added to the list of addresses and does not replace the first one.
    fn connection_string(&self, address: Option<(&str, u16)>, password: Option<&str>) -> String {
        let config = self.config();
        let (host, port) = match address {
            Some((host, port)) => (host.to_string(), port),
            None => self.upstream(),
        };
        let mut parts = vec![format!("host={host}"), format!("port={port}")];
        if let Some(dbname) = config.get_dbname() {
            parts.push(format!("dbname={dbname}"));
        }
        if let Some(user) = config.get_user() {
            parts.push(format!("user={user}"));
        }
        if let Some(password) = password.map(str::to_string).or_else(|| self.password()) {
            parts.push(format!("password={password}"));
        }
        parts.push(format!("options=-csearch_path={}", self.schema));
        parts.join(" ")
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = self.admin.batch_execute(&format!(
            "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
            self.schema
        ));
    }
}

/// Add one parameter to a connection URL.
fn with_param(url: &str, key: &str, value: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}{key}={value}")
}

/// A TCP forwarder between a store and PostgreSQL, so a test can take the
/// connection away and give it back. To terminate the server's side of a
/// connection is not an outage: the server is still there and the store
/// connects again at once.
struct Link {
    port: u16,
    open: Arc<AtomicBool>,
    live: Arc<Mutex<Vec<TcpStream>>>,
}

impl Link {
    fn to(upstream: (&str, u16)) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind the forwarder");
        let port = listener.local_addr().unwrap().port();
        let open = Arc::new(AtomicBool::new(true));
        let live: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
        let upstream = (upstream.0.to_string(), upstream.1);

        let accepting = Arc::clone(&open);
        let registry = Arc::clone(&live);
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(client) = incoming else { continue };
                if !accepting.load(Ordering::SeqCst) {
                    // Dropped with no answer, as with a server that is away.
                    continue;
                }
                let Ok(server) = TcpStream::connect((upstream.0.as_str(), upstream.1)) else {
                    continue;
                };
                registry
                    .lock()
                    .unwrap()
                    .extend([client.try_clone().unwrap(), server.try_clone().unwrap()]);
                pump(client.try_clone().unwrap(), server.try_clone().unwrap());
                pump(server, client);
            }
        });
        Self { port, open, live }
    }

    /// Refuse new connections and cut the live ones.
    fn cut(&self) {
        self.open.store(false, Ordering::SeqCst);
        for stream in self.live.lock().unwrap().drain(..) {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }

    fn restore(&self) {
        self.open.store(true, Ordering::SeqCst);
    }
}

/// Copy one direction until it ends, then close the other side.
fn pump(mut from: TcpStream, mut to: TcpStream) {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match from.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    if to.write_all(&buffer[..read]).is_err() {
                        break;
                    }
                }
            }
        }
        let _ = to.shutdown(Shutdown::Both);
    });
}

// --- fixtures ---------------------------------------------------------------

fn scope(name: &str) -> ScopeId {
    ScopeId::new(name).unwrap()
}

fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

fn all() -> KeyPrefix {
    KeyPrefix::new("").unwrap()
}

fn v(n: u64) -> Version {
    Version::new(n).unwrap()
}

fn put(text: &str, content: &str, expected: Expected) -> Op {
    Op::Put {
        key: key(text),
        content: content.into(),
        expected,
    }
}

fn event(name: &str) -> Event {
    Event::new(name, serde_json::json!({ "name": name, "n": 1.0 })).unwrap()
}

fn commit_at(millis: u64, ops: Vec<Op>, events: Vec<Event>) -> Commit {
    Commit::new(
        Actor::new("u-1", "Momo").unwrap(),
        "test",
        Timestamp::new(millis).unwrap(),
        ops,
        events,
    )
    .unwrap()
}

fn commit(ops: Vec<Op>, events: Vec<Event>) -> Commit {
    commit_at(1, ops, events)
}

/// A store on a new schema.
fn new_store() -> (TestDb, PostgresPort) {
    let db = TestDb::new();
    let store = db.store();
    (db, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
fn seed_five_events(store: &PostgresPort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

fn open_error(result: Result<PostgresPort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the connect must be refused"),
        Err(err) => err,
    }
}

const META_TABLE: &str = "CREATE TABLE meta (key text PRIMARY KEY, value text NOT NULL);";

// --- 三個範例通過合格測試 ---------------------------------------------------

struct Harness {
    store: PostgresPort,
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

fn conformance_passes_all_sixteen_checks_without_a_skip() {
    let (_db, store) = new_store();
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

fn modified_at_is_the_commit_time() {
    let (_db, store) = new_store();
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

fn each_scope_has_its_own_version() {
    let (_db, store) = new_store();
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

fn content_with_nul_and_non_ascii_reads_back_the_same() {
    let (_db, store) = new_store();
    let s = scope("s");
    let content = "before\u{0}after 規格 ✓ 🚀";
    store
        .commit(
            &s,
            &commit(vec![put("k", content, Expected::Absent)], Vec::new()),
        )
        .unwrap();
    assert_eq!(
        store.read(&s, &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        content
    );
}

// --- 事件與寫入同一次交易並可依序號讀回 -------------------------------------

fn two_events_of_one_commit_get_sequence_1_and_2() {
    let (_db, store) = new_store();
    let s = scope("s");
    let written = event("artifact-written");
    let ranked = event("change-ranked");
    store
        .commit(
            &s,
            &commit(
                vec![put("config.yaml", "x", Expected::Absent)],
                vec![written.clone(), ranked.clone()],
            ),
        )
        .unwrap();

    let events = store.events_after(&s, 0, 10).unwrap();
    assert_eq!(
        events,
        [
            (
                1,
                RecordedEvent {
                    name: "artifact-written".into(),
                    payload: written.payload().clone(),
                    version: v(1),
                }
            ),
            (
                2,
                RecordedEvent {
                    name: "change-ranked".into(),
                    payload: ranked.payload().clone(),
                    version: v(1),
                }
            ),
        ]
    );
}

fn a_rejected_commit_keeps_no_event_and_uses_no_sequence() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();

    let rejected = commit(
        vec![put("config.yaml", "x", Expected::At(v(9)))],
        vec![event("lost")],
    );
    assert!(matches!(
        store.commit(&s, &rejected),
        Err(PortError::Conflict { .. })
    ));

    store
        .commit(&s, &commit(Vec::new(), vec![event("second")]))
        .unwrap();
    let events = store.events_after(&s, 0, 10).unwrap();
    let names: Vec<(u64, &str)> = events
        .iter()
        .map(|(seq, event)| (*seq, event.name.as_str()))
        .collect();
    assert_eq!(names, [(1, "first"), (2, "second")]);
}

fn events_after_reads_from_the_middle_up_to_the_limit() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(sequences(&store.events_after(&s, 2, 2).unwrap()), [3, 4]);
}

fn a_scope_with_no_events_reads_as_empty() {
    let (_db, store) = new_store();
    assert_eq!(store.events_after(&scope("never"), 0, 10).unwrap(), []);
    assert_eq!(store.event_floor(&scope("never")).unwrap(), 0);
}

// --- 清除舊事件與下限 -------------------------------------------------------

fn trim_keeps_only_the_newer_events_and_changes_no_document() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    seed_five_events(&store, &s);
    let before = store.list(&s, &all()).unwrap();

    store.trim_events(&s, 3).unwrap();

    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [4, 5]);
    assert_eq!(store.event_floor(&s).unwrap(), 3);
    assert_eq!(store.list(&s, &all()).unwrap(), before);
    assert_eq!(
        store.read(&s, &[key("doc")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "x"
    );
}

fn the_sequence_continues_after_a_trim() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 5).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], vec![event("next")]),
        )
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_trim_past_the_last_sequence_stops_at_it() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 100).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 5);
    store
        .commit(
            &s,
            &commit(vec![put("doc", "x", Expected::Absent)], vec![event("next")]),
        )
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_trim_at_or_below_the_floor_does_nothing() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, 3).unwrap();
    store.trim_events(&s, 2).unwrap();
    store.trim_events(&s, 3).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 3);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [4, 5]);
}

// --- 開啟時只認自己的配置 ---------------------------------------------------

fn a_schema_without_meta_is_initialized_beside_other_tables() {
    let mut db = TestDb::new();
    let mut raw = db.client();
    raw.batch_execute("CREATE TABLE notes (body text); INSERT INTO notes VALUES ('keep me');")
        .unwrap();

    let store = db.store();

    let listing = store.list(&scope("any"), &all()).unwrap();
    assert_eq!(listing.version, Version::ZERO);
    assert!(listing.entries.is_empty());
    assert_eq!(
        db.tables(),
        ["documents", "events", "meta", "notes", "scopes"]
    );
    let body: String = raw.query_one("SELECT body FROM notes", &[]).unwrap().get(0);
    assert_eq!(body, "keep me");
    let format: String = raw
        .query_one("SELECT value FROM meta WHERE key = 'format'", &[])
        .unwrap()
        .get(0);
    assert_eq!(format, "speclink-store-port");
    let layout: String = raw
        .query_one("SELECT value FROM meta WHERE key = 'layout_version'", &[])
        .unwrap()
        .get(0);
    assert_eq!(layout, "1");
}

fn another_marker_is_refused_and_no_table_is_added() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-team-store'), ('schema_version', '1');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: None }
                if format == "speclink-team-store"
        ),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

fn a_meta_table_without_a_marker_is_refused() {
    let mut db = TestDb::new();
    db.client().batch_execute(META_TABLE).unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(&err, OpenError::UnknownLayout { format: None, .. }),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

fn a_newer_layout_version_is_refused() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-store-port'), ('layout_version', '2');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert!(
        matches!(
            &err,
            OpenError::UnknownLayout { format: Some(format), layout_version: Some(layout) }
                if format == "speclink-store-port" && layout == "2"
        ),
        "{err:?}"
    );
    assert_eq!(db.tables(), before);
}

// --- 重新開啟與中斷後的一致性 -----------------------------------------------

fn a_reconnected_store_has_the_same_content() {
    let db = TestDb::new();
    let s = scope("s");
    let keys = [key("a"), key("b"), key("c"), key("d")];

    let store = db.store();
    // Three commits: four documents written, one deleted, three events.
    store
        .commit(
            &s,
            &commit_at(
                10,
                vec![
                    put("a", "A", Expected::Absent),
                    put("b", "B", Expected::Absent),
                ],
                vec![event("one")],
            ),
        )
        .unwrap();
    store
        .commit(
            &s,
            &commit_at(
                20,
                vec![
                    put("c", "C", Expected::Absent),
                    put("d", "D", Expected::Absent),
                ],
                vec![event("two")],
            ),
        )
        .unwrap();
    store
        .commit(
            &s,
            &commit_at(
                30,
                vec![Op::Delete {
                    key: key("a"),
                    expected: v(1),
                }],
                vec![event("three")],
            ),
        )
        .unwrap();
    let listing = store.list(&s, &all()).unwrap();
    let documents = store.read(&s, &keys).unwrap();
    let events = store.events_after(&s, 0, 10).unwrap();
    assert_eq!(listing.version, v(3));
    assert_eq!(listing.entries.len(), 3);
    assert_eq!(events.len(), 3);
    drop(store);

    let store = db.store();
    assert_eq!(store.list(&s, &all()).unwrap(), listing);
    assert_eq!(store.read(&s, &keys).unwrap(), documents);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), events);
    assert_eq!(store.event_floor(&s).unwrap(), 0);

    // The floor is also the same after a new connect.
    store.trim_events(&s, 2).unwrap();
    drop(store);
    let store = db.store();
    assert_eq!(store.event_floor(&s).unwrap(), 2);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [3]);
}

fn a_commit_that_fails_on_its_second_operation_leaves_nothing() {
    let (_db, store) = new_store();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("base", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    let failing = commit(
        vec![
            put("first", "written before the failure", Expected::Absent),
            put("second", "x", Expected::At(v(7))),
        ],
        vec![event("lost")],
    );
    assert_eq!(
        store.commit(&s, &failing),
        Err(PortError::Conflict {
            key: key("second"),
            expected: Expected::At(v(7)),
            actual: None,
        })
    );

    assert_eq!(store.read(&s, &[key("first")]).unwrap(), [None]);
    assert_eq!(store.events_after(&s, 0, 10).unwrap(), []);
    assert_eq!(store.list(&s, &all()).unwrap().version, v(1));
}

// --- 兩個儲存物件開同一份資料 -----------------------------------------------

fn a_commit_by_another_object_makes_the_old_version_conflict() {
    let db = TestDb::new();
    let (first, second) = (db.store(), db.store());
    let s = scope("s");

    first
        .commit(
            &s,
            &commit(vec![put("k", "v1", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    first
        .commit(
            &s,
            &commit(vec![put("k", "from first", Expected::At(v(1)))], Vec::new()),
        )
        .unwrap();

    let stale = commit(
        vec![put("k", "from second", Expected::At(v(1)))],
        Vec::new(),
    );
    assert_eq!(
        second.commit(&s, &stale),
        Err(PortError::Conflict {
            key: key("k"),
            expected: Expected::At(v(1)),
            actual: Some(v(2)),
        })
    );
    assert_eq!(
        second.read(&s, &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "from first"
    );
}

fn two_objects_that_commit_the_same_absent_key_have_one_winner() {
    let db = TestDb::new();
    let stores = [db.store(), db.store()];
    let s = scope("s");
    let barrier = Barrier::new(2);

    let results: Vec<Result<Version, PortError>> = thread::scope(|threads| {
        let handles: Vec<_> = stores
            .iter()
            .map(|store| {
                threads.spawn(|| {
                    barrier.wait();
                    store.commit(
                        &s,
                        &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
                    )
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let winners = results.iter().filter(|r| r.is_ok()).count();
    let conflicts = results
        .iter()
        .filter(|r| matches!(r, Err(PortError::Conflict { .. })))
        .count();
    assert_eq!((winners, conflicts), (1, 1), "{results:?}");
    assert_eq!(stores[0].list(&s, &all()).unwrap().version, v(1));
}

/// Another connection holds the row lock of scope `held` for as long as the
/// returned transaction lives.
fn hold_scope_lock<'a>(holder: &'a mut Client, held: &str) -> postgres::Transaction<'a> {
    let mut tx = holder.transaction().unwrap();
    let locked = tx
        .query("SELECT 1 FROM scopes WHERE scope = $1 FOR UPDATE", &[&held])
        .unwrap();
    assert_eq!(locked.len(), 1, "the scope must have its row");
    tx
}

fn commits_on_different_scopes_do_not_wait_for_each_other() {
    let (db, store) = new_store();
    let (a, b) = (scope("A"), scope("B"));
    store
        .commit(
            &a,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let mut holder = db.client();
    let lock_on_a = hold_scope_lock(&mut holder, "A");

    // A commit that waited for scope A would end in a lock timeout.
    let started = Instant::now();
    let result = store.commit(
        &b,
        &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
    );
    assert_eq!(result, Ok(v(1)));
    assert!(started.elapsed() < Duration::from_secs(4));

    lock_on_a.rollback().unwrap();
}

fn a_commit_that_cannot_get_the_scope_lock_reports_unavailable() {
    let (db, store) = new_store();
    let a = scope("A");
    store
        .commit(
            &a,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let mut holder = db.client();
    let lock_on_a = hold_scope_lock(&mut holder, "A");

    let result = store.commit(
        &a,
        &commit(vec![put("k2", "x", Expected::Absent)], Vec::new()),
    );
    assert!(
        matches!(result, Err(PortError::Unavailable { .. })),
        "{result:?}"
    );

    lock_on_a.rollback().unwrap();
    assert_eq!(store.read(&a, &[key("k2")]).unwrap(), [None]);
    assert_eq!(
        store.commit(
            &a,
            &commit(vec![put("k2", "x", Expected::Absent)], Vec::new())
        ),
        Ok(v(2))
    );
}

// --- 可用性檢查與失敗的分類 -------------------------------------------------

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
        eprintln!("note: {URL_VAR} has no password, so the order cannot be seen");
        return;
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
    if server_checks {
        let err = open_error(wrong_in_the_environment);
        assert!(matches!(err, OpenError::Store(_)), "{err:?}");
        assert!(!err.to_string().contains("wrong-in-the-environment"));
        assert!(!err.to_string().contains(&right));
    }
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

// --- 稽核：輸入的邊界 -------------------------------------------------------

fn a_connection_string_that_cannot_be_read_is_refused_with_the_reason() {
    let err = open_error(PostgresPort::connect(
        "postgres://user:s3cret@localhost:not-a-port/db",
    ));
    assert!(
        matches!(
            &err,
            OpenError::Store(PortError::Backend { message })
                if message.contains("cannot read the PostgreSQL connection string")
        ),
        "{err:?}"
    );
    assert!(!err.to_string().contains("s3cret"), "{err}");
    // No password to report for a string that cannot be read.
    assert!(!url_embeds_password(
        "postgres://user:s3cret@localhost:not-a-port/db"
    ));
}

fn events_after_with_limit_zero_or_the_largest_number_is_empty() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    assert_eq!(store.events_after(&s, 0, 0).unwrap(), []);
    assert_eq!(store.events_after(&s, u64::MAX, 10).unwrap(), []);
    assert_eq!(
        sequences(&store.events_after(&s, 4, usize::MAX).unwrap()),
        [5]
    );
}

fn a_trim_through_the_largest_number_stops_at_the_last_sequence() {
    let (_db, store) = new_store();
    let s = scope("s");
    seed_five_events(&store, &s);
    store.trim_events(&s, u64::MAX).unwrap();
    assert_eq!(store.event_floor(&s).unwrap(), 5);
    store
        .commit(&s, &commit(Vec::new(), vec![event("next")]))
        .unwrap();
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [6]);
}

fn a_prefix_matches_bytes_and_has_no_wildcard() {
    let (_db, store) = new_store();
    let s = scope("s");
    // In byte order. U+10FFFF is the largest character, so a prefix that
    // ends with it has no "next character" to stop the range at.
    let keys = [
        "a%b",
        "a*b",
        "aXb",
        "a[b]",
        "a_b",
        "a\u{10FFFF}",
        "a\u{10FFFF}z",
        "b",
    ];
    let ops = keys
        .iter()
        .map(|key| put(key, "x", Expected::Absent))
        .collect();
    store.commit(&s, &commit(ops, Vec::new())).unwrap();

    let listed = |prefix: &str| -> Vec<String> {
        store
            .list(&s, &KeyPrefix::new(prefix).unwrap())
            .unwrap()
            .entries
            .into_iter()
            .map(|entry| entry.key.as_str().to_string())
            .collect()
    };
    assert_eq!(listed(""), keys);
    assert_eq!(listed("a%"), ["a%b"]);
    assert_eq!(listed("a_"), ["a_b"]);
    assert_eq!(listed("a*"), ["a*b"]);
    assert_eq!(listed("a["), ["a[b]"]);
    assert_eq!(listed("a\u{10FFFF}"), ["a\u{10FFFF}", "a\u{10FFFF}z"]);
    assert_eq!(listed("\u{10FFFF}"), [] as [&str; 0]);
    // A backslash cannot be in a key or a prefix, and a string cannot hold
    // a 0xFF byte: the contract's own types keep both out of the store.
    assert!(DocKey::new("a\\b").is_err());
    assert!(KeyPrefix::new("a\\").is_err());
}

fn a_layout_version_that_is_not_a_number_is_refused_as_it_is() {
    let mut db = TestDb::new();
    db.client()
        .batch_execute(&format!(
            "{META_TABLE}
             INSERT INTO meta VALUES ('format', 'speclink-store-port'), ('layout_version', 'one');"
        ))
        .unwrap();
    let before = db.tables();

    let err = open_error(PostgresPort::connect(db.url()));

    assert_eq!(
        err,
        OpenError::UnknownLayout {
            format: Some("speclink-store-port".into()),
            layout_version: Some("one".into()),
        }
    );
    assert_eq!(db.tables(), before);
}

fn a_scope_at_the_version_limit_rejects_the_commit_and_keeps_its_sequence() {
    let (db, store) = new_store();
    let s = scope("s");
    store
        .commit(&s, &commit(Vec::new(), vec![event("first")]))
        .unwrap();
    let mut raw = db.client();
    raw.execute(
        "UPDATE scopes SET version = $1",
        &[&(speclink_store_port::MAX_VERSION as i64)],
    )
    .unwrap();

    let result = store.commit(
        &s,
        &commit(vec![put("k", "x", Expected::Absent)], vec![event("lost")]),
    );

    assert!(
        matches!(result, Err(PortError::Backend { .. })),
        "{result:?}"
    );
    assert_eq!(store.read(&s, &[key("k")]).unwrap(), [None]);
    assert_eq!(sequences(&store.events_after(&s, 0, 10).unwrap()), [1]);
    let event_seq: i64 = raw
        .query_one("SELECT event_seq FROM scopes", &[])
        .unwrap()
        .get(0);
    assert_eq!(event_seq, 1);
}
