//! Fixtures that the test modules share.

use postgres::config::Host;
use postgres::{Client, Config, NoTls};
use speclink_store_port::{
    Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, RecordedEvent, ScopeId, StorePort,
    Timestamp, Version,
};
use speclink_store_port_postgres::{OpenError, PostgresPort};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// The environment variable that names the PostgreSQL instance to test with.
pub const URL_VAR: &str = "SPECLINK_TEST_POSTGRES_URL";

/// One test: its name and its function.
pub type Test = (&'static str, fn());

/// The configured instance. An empty value is the same as no value.
pub fn base_url() -> Option<String> {
    match std::env::var(URL_VAR) {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => None,
    }
}

/// A schema of its own, dropped when the value goes out of scope. A store
/// that connects with [`TestDb::url`] sees only the tables of this schema.
pub struct TestDb {
    pub admin: Client,
    base: String,
    pub schema: String,
    url: String,
}

impl TestDb {
    pub fn new() -> Self {
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

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn store(&self) -> PostgresPort {
        PostgresPort::connect(&self.url).unwrap()
    }

    /// A plain connection whose search path is this schema.
    pub fn client(&self) -> Client {
        Client::connect(&self.url, NoTls).expect("connect to the test schema")
    }

    /// The names of the tables in this schema.
    pub fn tables(&mut self) -> Vec<String> {
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

    pub fn config(&self) -> Config {
        self.base.parse().expect("parse the configured url")
    }

    /// The password in the configured URL.
    pub fn password(&self) -> Option<String> {
        self.config()
            .get_password()
            .map(|password| String::from_utf8_lossy(password).into_owned())
    }

    /// The TCP address that the configured URL points at.
    pub fn upstream(&self) -> (String, u16) {
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
    pub fn connection_string(
        &self,
        address: Option<(&str, u16)>,
        password: Option<&str>,
    ) -> String {
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

impl TestDb {
    /// A connection string whose sessions can only read, as on a standby
    /// server.
    pub fn read_only_connection_string(&self) -> String {
        let options = format!("options=-csearch_path={}", self.schema);
        let read_only = format!(
            "options='-csearch_path={} -cdefault_transaction_read_only=on'",
            self.schema
        );
        self.connection_string(None, None)
            .replace(&options, &read_only)
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
pub fn with_param(url: &str, key: &str, value: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}{key}={value}")
}

/// A TCP forwarder between a store and PostgreSQL, so a test can take the
/// connection away and give it back. To terminate the server's side of a
/// connection is not an outage: the server is still there and the store
/// connects again at once.
pub struct Link {
    pub port: u16,
    open: Arc<AtomicBool>,
    live: Arc<Mutex<Vec<TcpStream>>>,
}

impl Link {
    pub fn to(upstream: (&str, u16)) -> Self {
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
    pub fn cut(&self) {
        self.open.store(false, Ordering::SeqCst);
        for stream in self.live.lock().unwrap().drain(..) {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }

    pub fn restore(&self) {
        self.open.store(true, Ordering::SeqCst);
    }
}

/// Copy one direction until it ends, then close the other side.
pub fn pump(mut from: TcpStream, mut to: TcpStream) {
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

pub fn scope(name: &str) -> ScopeId {
    ScopeId::new(name).unwrap()
}

pub fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

pub fn all() -> KeyPrefix {
    KeyPrefix::new("").unwrap()
}

pub fn v(n: u64) -> Version {
    Version::new(n).unwrap()
}

pub fn put(text: &str, content: &str, expected: Expected) -> Op {
    Op::Put {
        key: key(text),
        content: content.into(),
        expected,
    }
}

pub fn event(name: &str) -> Event {
    Event::new(name, serde_json::json!({ "name": name, "n": 1.0 })).unwrap()
}

pub fn commit_at(millis: u64, ops: Vec<Op>, events: Vec<Event>) -> Commit {
    Commit::new(
        Actor::new("u-1", "Momo").unwrap(),
        "test",
        Timestamp::new(millis).unwrap(),
        ops,
        events,
    )
    .unwrap()
}

pub fn commit(ops: Vec<Op>, events: Vec<Event>) -> Commit {
    commit_at(1, ops, events)
}

/// A store on a new schema.
pub fn new_store() -> (TestDb, PostgresPort) {
    let db = TestDb::new();
    let store = db.store();
    (db, store)
}

/// Five commits with one event each: sequences 1 to 5, scope version 5.
pub fn seed_five_events(store: &PostgresPort, scope: &ScopeId) {
    for n in 1..=5 {
        store
            .commit(scope, &commit(Vec::new(), vec![event(&format!("e{n}"))]))
            .unwrap();
    }
}

pub fn sequences(events: &[(u64, RecordedEvent)]) -> Vec<u64> {
    events.iter().map(|(seq, _)| *seq).collect()
}

pub fn open_error(result: Result<PostgresPort, OpenError>) -> OpenError {
    match result {
        Ok(_) => panic!("the connect must be refused"),
        Err(err) => err,
    }
}

pub const META_TABLE: &str = "CREATE TABLE meta (key text PRIMARY KEY, value text NOT NULL);";

/// What a test ends with when this database gives it nothing to check. The
/// runner prints the test as `skipped`, with the reason.
pub struct NotChecked(pub &'static str);

/// End the test as not checked.
pub fn not_checked(reason: &'static str) -> ! {
    // Not `panic!`: this is not a failure, and it must not print as one.
    std::panic::resume_unwind(Box::new(NotChecked(reason)))
}
