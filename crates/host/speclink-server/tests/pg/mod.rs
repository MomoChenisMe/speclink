//! Scaffolding for the server's PostgreSQL tests.
//!
//! Small on purpose, and deliberately not shared with the driver crate's copy:
//! exporting test scaffolding from `speclink-store-postgres` would put it in
//! that crate's public API, where it would outlive its one caller.
//!
//! The skip rule is the same one the driver crate follows — without an instance
//! named by `SPECLINK_TEST_POSTGRES_URL`, report `skipped`, never `passed`.

#![allow(dead_code)]

use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use postgres::config::Host;
use postgres::{Client, Config, NoTls};

pub const URL_VAR: &str = "SPECLINK_TEST_POSTGRES_URL";

const ENABLE_HINT: &str = "docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=speclink --name speclink-pg postgres:15 && export SPECLINK_TEST_POSTGRES_URL=postgres://postgres:speclink@localhost:5432/postgres";

/// The configured instance, or `None` when these tests cannot run. An empty
/// value counts as absent.
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
/// value goes out of scope.
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
        let url = with_param(&base, "options", &format!("-csearch_path%3D{schema}"));
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

    /// This instance in the "password comes from the environment" shape: a
    /// connection string carrying no password, plus the password the caller
    /// must export for it to connect.
    ///
    /// Built from the parsed configuration rather than by editing the URL text,
    /// because the configured base URL may or may not carry a password —
    /// CI's does, a trust-auth laptop's does not — and a test that assumed
    /// either would pass on one and fail on the other.
    ///
    /// Key/value form: a URL cannot express "explicitly no password".
    pub fn url_without_password(&self) -> (String, Option<String>) {
        let config: Config = self.base.parse().expect("parse the configured url");
        let password = config
            .get_password()
            .map(|password| String::from_utf8_lossy(password).to_string());

        let mut parts = match config.get_hosts().first() {
            Some(Host::Tcp(host)) => vec![format!("host={host}")],
            _ => panic!("{URL_VAR} must name a TCP host for this test"),
        };
        parts.push(format!(
            "port={}",
            config.get_ports().first().copied().unwrap_or(5432)
        ));
        if let Some(dbname) = config.get_dbname() {
            parts.push(format!("dbname={dbname}"));
        }
        if let Some(user) = config.get_user() {
            parts.push(format!("user={user}"));
        }
        parts.push(format!("options=-csearch_path={}", self.schema));
        (parts.join(" "), password)
    }

    /// The same instance, reached by a connection string that carries the
    /// password inline — the shape startup is meant to warn about.
    pub fn url_with_password(&self, password: &str) -> String {
        let (bare, _) = self.url_without_password();
        format!("{bare} password={password}")
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = self
            .admin
            .batch_execute(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", self.schema));
    }
}

fn unique_schema() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the epoch")
        .as_nanos();
    format!(
        "speclink_srv_{}_{nanos}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn with_param(base: &str, key: &str, value: &str) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{key}={value}")
}
