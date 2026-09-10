//! Shared scaffolding for the PostgreSQL driver's tests.
//!
//! Every test target in this crate needs a real PostgreSQL instance, named by
//! `SPECLINK_TEST_POSTGRES_URL`. When it is absent the suite reports `skipped`
//! and prints one line that enables it — reporting `passed` without having run
//! is exactly the silent green the design rules out.
//!
//! That rule is why these targets set `harness = false`. libtest can only skip
//! at compile time (`#[ignore]`), which would keep the tests hidden even when a
//! database *is* configured; it has no way to skip on a runtime condition. The
//! runner below is the smallest thing that can decide at runtime.

#![allow(dead_code)] // Each test binary compiles its own copy and uses a subset.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use postgres::config::Host;
use postgres::{Client, Config, NoTls};
use speclink_store::{CommandContext, DocumentId, EventRecord, ProjectId, RepoId, Scope};

/// The environment variable naming the PostgreSQL instance under test.
pub const URL_VAR: &str = "SPECLINK_TEST_POSTGRES_URL";

/// One line that provisions a database and points the suite at it.
const ENABLE_HINT: &str = "docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=speclink --name speclink-pg postgres:15 && export SPECLINK_TEST_POSTGRES_URL=postgres://postgres:speclink@localhost:5432/postgres";

/// The configured instance, or `None` when the suite cannot run. An empty
/// value counts as absent: an exported-but-blank variable means the same thing
/// as an unset one, and treating it as a URL would fail obscurely later.
pub fn base_url() -> Option<String> {
    match std::env::var(URL_VAR) {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => None,
    }
}

/// Run `tests` in order, or report every one as skipped when no database is
/// configured. A failure exits non-zero, so a red suite fails the build.
pub fn run(tests: &[(&str, fn())]) {
    if base_url().is_none() {
        for (name, _) in tests {
            println!("test {name} ... skipped");
        }
        println!();
        println!(
            "test result: skipped. {} skipped; {URL_VAR} names no PostgreSQL instance.",
            tests.len()
        );
        println!("Enable them with:");
        println!("  {ENABLE_HINT}");
        return;
    }

    let mut failures = Vec::new();
    for (name, test) in tests {
        print!("test {name} ... ");
        io::stdout().flush().ok();
        match panic::catch_unwind(AssertUnwindSafe(test)) {
            Ok(()) => println!("ok"),
            Err(_) => {
                println!("FAILED");
                failures.push(*name);
            }
        }
    }

    println!();
    println!(
        "test result: {}. {} passed; {} failed",
        if failures.is_empty() { "ok" } else { "FAILED" },
        tests.len() - failures.len(),
        failures.len()
    );
    for name in &failures {
        println!("    {name}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

/// An isolated PostgreSQL namespace: a schema of its own, dropped when the
/// value goes out of scope. Stores built from [`TestDb::url`] see only their
/// own tables, so the whole suite can share one server.
pub struct TestDb {
    admin: Client,
    base: String,
    schema: String,
    url: String,
}

impl TestDb {
    pub fn new() -> Self {
        let base = base_url().expect("run() gates on a configured database");
        let schema = unique_schema();
        let mut admin = Client::connect(&base, NoTls).expect("connect to the test database");
        admin
            .batch_execute(&format!("CREATE SCHEMA \"{schema}\""))
            .expect("create the test schema");
        let url = with_search_path(&base, &schema);
        Self {
            admin,
            base,
            schema,
            url,
        }
    }

    /// A connection URL whose `search_path` is this instance's private schema.
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// The TCP endpoint the configured URL points at.
    pub fn upstream(&self) -> (String, u16) {
        let config: Config = self.base.parse().expect("parse the configured url");
        let host = match config.get_hosts().first() {
            Some(Host::Tcp(host)) => host.clone(),
            _ => panic!("{URL_VAR} must name a TCP host for this test"),
        };
        let port = config.get_ports().first().copied().unwrap_or(5432);
        (host, port)
    }

    /// A connection string for this instance's schema, routed through
    /// `host:port` instead of the configured endpoint — so a test can put a
    /// link it controls in front of the server.
    ///
    /// Key/value form rather than a URL: `host`/`port` given as URL query
    /// parameters *append* to the endpoint list rather than replacing it, so a
    /// rewritten URL would still reach the real server.
    pub fn url_via(&self, host: &str, port: u16) -> String {
        let config: Config = self.base.parse().expect("parse the configured url");
        let mut parts = vec![format!("host={host}"), format!("port={port}")];
        if let Some(dbname) = config.get_dbname() {
            parts.push(format!("dbname={dbname}"));
        }
        if let Some(user) = config.get_user() {
            parts.push(format!("user={user}"));
        }
        if let Some(password) = config.get_password() {
            parts.push(format!("password={}", String::from_utf8_lossy(password)));
        }
        parts.push(format!("options=-csearch_path={}", self.schema));
        parts.join(" ")
    }
}

/// A TCP forwarder standing between a store and PostgreSQL, so a test can take
/// the link away and give it back.
///
/// Terminating a backend cannot model an outage: the server is still there, so
/// the store reconnects immediately, and whether it notices the dead socket
/// before or after the next call is a race — a test asserting either outcome is
/// betting on the scheduler. Here the link is genuinely gone until
/// [`Link::restore`], so `unavailable` is the only correct answer in between.
pub struct Link {
    port: u16,
    open: Arc<AtomicBool>,
    live: Arc<Mutex<Vec<TcpStream>>>,
}

impl Link {
    /// Start forwarding from a fresh loopback port to `upstream`.
    pub fn to(upstream: (&str, u16)) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind the forwarder");
        let port = listener.local_addr().expect("forwarder address").port();
        let open = Arc::new(AtomicBool::new(true));
        let live: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
        let upstream = (upstream.0.to_string(), upstream.1);

        let accepting = Arc::clone(&open);
        let registry = Arc::clone(&live);
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(client) = incoming else { continue };
                if !accepting.load(Ordering::SeqCst) {
                    // Dropped unanswered: the caller sees the connection go
                    // away, which is what an absent server looks like.
                    continue;
                }
                let Ok(server) = TcpStream::connect((upstream.0.as_str(), upstream.1)) else {
                    continue;
                };
                let (client_read, server_read) = (
                    client.try_clone().expect("clone"),
                    server.try_clone().expect("clone"),
                );
                registry
                    .lock()
                    .expect("forwarder registry")
                    .extend([client.try_clone().expect("clone"), server
                        .try_clone()
                        .expect("clone")]);
                pump(client_read, server.try_clone().expect("clone"));
                pump(server_read, client);
            }
        });

        Self { port, open, live }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Take the link away: refuse new connections, cut the live ones.
    pub fn cut(&self) {
        self.open.store(false, Ordering::SeqCst);
        for stream in self.live.lock().expect("forwarder registry").drain(..) {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }

    /// Give the link back.
    pub fn restore(&self) {
        self.open.store(true, Ordering::SeqCst);
    }
}

/// Copy one direction until it ends, then close the other side so the peer
/// notices rather than waiting on a half-open socket.
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

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = self
            .admin
            .batch_execute(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", self.schema));
    }
}

// --- contract fixtures ----------------------------------------------------

pub fn ctx(command: &str) -> CommandContext {
    CommandContext {
        command: command.into(),
        actor: "tester".into(),
    }
}

pub fn scope(repo: &str) -> Scope {
    Scope::new(ProjectId::new("acme"), RepoId::new(repo))
}

pub fn spec(capability: &str) -> DocumentId {
    DocumentId::CanonicalSpec {
        capability: capability.into(),
    }
}

pub fn event(name: &str) -> EventRecord {
    EventRecord {
        name: name.into(),
        payload: serde_json::json!({ "event": name }),
        actor: "tester".into(),
        at: Utc::now(),
    }
}

/// A schema name unique across concurrent test binaries and repeated runs.
fn unique_schema() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the epoch")
        .as_nanos();
    format!(
        "speclink_test_{}_{nanos}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Point a base URL at `schema` by way of libpq's `options` parameter.
fn with_search_path(base: &str, schema: &str) -> String {
    with_param(base, "options", &format!("-csearch_path%3D{schema}"))
}

/// Override one connection parameter on a URL. Later parameters win, so this
/// works whatever the configured base URL already carries — which is what lets
/// a test force a bad `user` or `dbname` without parsing the URL apart.
pub fn with_param(base: &str, key: &str, value: &str) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{key}={value}")
}
