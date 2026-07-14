//! First-run bootstrap: the one-time setup token, minted only while no admin
//! exists, and the /setup gate that closes permanently once an admin is created
//! (server-setup spec「bootstrap token 一次性且以無 admin 為條件」, 決策 3).

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::setup::{ensure_setup_token, setup_token_ttl};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

// --- helpers ---

/// Seed an active admin user through the invitation flow (its admin flag carries
/// through), so `has_admin` is true — the gate's "setup closed" condition.
fn seed_admin(identity: &dyn IdentityStore) {
    let token = identity
        .create_invitation(NewInvitation {
            email: "admin@example.com".to_string(),
            display: "Admin <admin@example.com>".to_string(),
            memberships: vec![],
            admin: true,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite admin");
    identity.accept_invitation(&token, "admin-correct-horse").expect("accept admin");
}

/// Start an in-process server over `identity` (fresh memory store, demo config);
/// returns the base URL. The caller keeps its own clone of `identity` to mutate
/// the setup-token state the running server reads.
fn start_with(identity: Arc<IdentitySqlite>) -> String {
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity,
    };
    common::start(state)
}

/// `GET /setup?token=…` — returns `(status, body)`. A missing token omits the
/// query entirely.
fn get_setup(base: &str, token: Option<&str>) -> (u16, String) {
    let url = match token {
        Some(t) => format!("{base}/setup?token={t}"),
        None => format!("{base}/setup"),
    };
    match ureq::get(&url).call() {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

/// `POST /setup?token=…` with a form body — returns `(status, body)`. No Origin
/// header (a non-browser client), so the same-origin check admits it.
fn post_setup(base: &str, token: &str, fields: &[(&str, &str)]) -> (u16, String) {
    let agent = ureq::builder().redirects(0).build();
    match agent.post(&format!("{base}/setup?token={token}")).send_form(fields) {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

// --- module-level token logic (決策 3) ---

#[test]
fn a_fresh_store_mints_a_setup_token_and_a_live_one_is_not_reminted() {
    let identity = IdentitySqlite::open_memory().expect("identity");
    let token = ensure_setup_token(&identity).expect("ensure").expect("a fresh store mints a token");
    assert!(!token.is_empty(), "the minted token is non-empty");
    assert!(identity.is_valid_setup_token(&token).expect("valid"), "the minted token validates");
    // A second startup while the token is still live does not mint another.
    assert!(
        ensure_setup_token(&identity).expect("ensure").is_none(),
        "a still-valid token is not reminted on restart",
    );
}

#[test]
fn an_admin_closes_setup_and_no_token_is_minted() {
    let identity = IdentitySqlite::open_memory().expect("identity");
    seed_admin(&identity);
    assert!(
        ensure_setup_token(&identity).expect("ensure").is_none(),
        "an existing admin means setup is closed — no token is minted",
    );
}

#[test]
fn the_setup_token_hash_lands_with_a_24h_expiry_and_no_plaintext() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("identity.db");
    let identity = IdentitySqlite::open(&path).expect("identity");
    let before = Utc::now();
    let token = ensure_setup_token(&identity).expect("ensure").expect("mints");
    drop(identity);

    // Only a hash lands, and its expiry is ~24h out.
    let conn = rusqlite::Connection::open(&path).expect("reopen");
    let (hash, expires): (String, String) = conn
        .query_row(
            "SELECT token_hash, expires_at FROM setup_tokens WHERE consumed_at IS NULL",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("an outstanding setup token row");
    assert_ne!(hash, token, "the plaintext is never stored");
    let expires = chrono::DateTime::parse_from_rfc3339(&expires)
        .expect("expiry timestamp")
        .with_timezone(&Utc);
    let drift = (expires - before - setup_token_ttl()).num_seconds().abs();
    assert!(drift < 60, "expiry is ~24h from mint (drift {drift}s): {expires}");
    assert_eq!(setup_token_ttl(), Duration::hours(24), "the default lifetime is 24 hours");
}

#[test]
fn an_expired_token_is_replaced_on_restart_and_the_old_value_is_dead() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("identity.db");
    let identity = IdentitySqlite::open(&path).expect("identity");
    let old = ensure_setup_token(&identity).expect("ensure").expect("mints");
    drop(identity);

    // Expire the outstanding token in place, then restart.
    {
        let conn = rusqlite::Connection::open(&path).expect("reopen");
        conn.execute(
            "UPDATE setup_tokens SET expires_at = ?1",
            rusqlite::params![(Utc::now() - Duration::hours(1)).to_rfc3339()],
        )
        .expect("expire the token");
    }
    let identity = IdentitySqlite::open(&path).expect("reopen identity");
    let fresh = ensure_setup_token(&identity).expect("ensure").expect("re-mints after expiry");
    assert_ne!(fresh, old, "a fresh token replaces the expired one");
    assert!(!identity.is_valid_setup_token(&old).expect("old"), "the expired token no longer validates");
    assert!(identity.is_valid_setup_token(&fresh).expect("fresh"), "the fresh token validates");
}

// --- the /setup gate over HTTP (決策 3) ---

#[test]
fn setup_is_404_once_an_admin_exists() {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    seed_admin(&*identity);
    let base = start_with(identity);
    let (status, _body) = get_setup(&base, None);
    assert_eq!(status, 404, "/setup is closed (404) once an admin exists");
    // A token in hand does not reopen it.
    let (with_token, _) = get_setup(&base, Some("anything"));
    assert_eq!(with_token, 404, "no token reopens a closed setup");
}

#[test]
fn invalid_expired_and_consumed_tokens_get_the_same_invalid_setup_response() {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let base = start_with(identity.clone());

    // An unknown token.
    let unknown = get_setup(&base, Some("spk_setup_unknownvalue"));
    // An expired token (minting replaces any prior, which is fine — tested one at a time).
    let expired_token = identity.create_setup_token(Duration::seconds(-1)).expect("mint expired");
    let expired = get_setup(&base, Some(&expired_token));
    // A consumed token.
    let consumed_token = identity.create_setup_token(setup_token_ttl()).expect("mint");
    identity.consume_setup_token(&consumed_token).expect("consume");
    let consumed = get_setup(&base, Some(&consumed_token));

    assert_eq!(unknown, expired, "an expired token gets the same response as an unknown one");
    assert_eq!(unknown, consumed, "a consumed token gets the same response as an unknown one");
    assert_ne!(unknown.0, 200, "the invalid response is not the open flow");

    // A genuinely valid token is NOT the invalid response.
    let good = identity.create_setup_token(setup_token_ttl()).expect("mint good");
    let (good_status, _) = get_setup(&base, Some(&good));
    assert_eq!(good_status, 200, "a valid token opens the setup flow");
}

// --- the /setup flow: the four opening elements (決策 4) ---

#[test]
fn the_setup_flow_creates_admin_shows_store_status_and_registers_the_first_project_and_repo() {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let token = identity.create_setup_token(setup_token_ttl()).expect("mint token");
    let base = start_with(identity.clone());

    // GET: the flow opens at the admin section.
    let (status, body) = get_setup(&base, Some(&token));
    assert_eq!(status, 200, "a valid token opens the flow");
    assert!(body.contains("name=\"email\""), "the admin form asks for an email: {body}");

    // Section 1 — create the first admin (active + admin, not via invitation).
    let (s1, b1) = post_setup(
        &base,
        &token,
        &[
            ("email", "root@example.com"),
            ("display", "Root <root@example.com>"),
            ("password", "root-correct-horse"),
        ],
    );
    assert_eq!(s1, 200, "admin creation succeeds");
    assert!(b1.contains("memory"), "section 2 shows the store driver: {b1}");
    assert!(b1.contains('3'), "section 2 shows the identity schema version: {b1}");
    assert!(b1.contains("name=\"project_key\""), "the project/repo form is shown next");

    let admin = identity
        .find_user_by_email("root@example.com")
        .expect("lookup")
        .expect("the admin exists");
    assert!(admin.active && admin.admin, "the first admin is active with the admin flag");

    // Section 3 — register the first project and repo.
    let (s2, b2) = post_setup(
        &base,
        &token,
        &[
            ("project_key", "acme"),
            ("project_name", "Acme"),
            ("repo_key", "backend"),
            ("repo_name", "Backend"),
        ],
    );
    assert_eq!(s2, 200, "project/repo creation succeeds");
    // Section 4 — connection info: the configured public url and the created keys.
    assert!(b2.contains("127.0.0.1"), "connection info shows the configured public url: {b2}");
    assert!(b2.contains("acme"), "connection info names the project key: {b2}");
    assert!(b2.contains("backend"), "connection info names the repo key: {b2}");

    assert!(identity.get_project("acme").expect("get").is_some(), "the project is registered");
    assert_eq!(identity.list_repos("acme").expect("repos")[0].key, "backend", "the repo is registered");

    // Completion consumes the token and closes /setup.
    assert!(!identity.is_valid_setup_token(&token).expect("valid"), "completion consumes the token");
    let (closed, _) = get_setup(&base, Some(&token));
    assert_eq!(closed, 404, "a completed setup is closed");
}

#[test]
fn a_resume_after_creating_the_admin_shows_it_done_and_continues_to_the_project_step() {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let token = identity.create_setup_token(setup_token_ttl()).expect("mint");
    let base = start_with(identity.clone());

    // Create the admin, then "close the browser" (the token is not yet consumed).
    let (s1, _) = post_setup(
        &base,
        &token,
        &[("email", "root@example.com"), ("display", "Root"), ("password", "root-correct-horse")],
    );
    assert_eq!(s1, 200);
    assert!(identity.has_admin().expect("has_admin"), "the admin was created");
    assert!(identity.is_valid_setup_token(&token).expect("valid"), "the token is still live mid-flow");

    // Re-enter with the same token: the admin section is done (not rebuilt) and
    // the project/repo step is shown.
    let (s2, b2) = get_setup(&base, Some(&token));
    assert_eq!(s2, 200, "the same token resumes the flow");
    assert!(b2.contains("name=\"project_key\""), "resume continues at the project step: {b2}");
    assert!(!b2.contains("name=\"email\""), "the admin form is not shown again");

    // Completing the project step finishes setup.
    let (s3, _) = post_setup(&base, &token, &[("project_key", "acme"), ("repo_key", "backend")]);
    assert_eq!(s3, 200);
    assert!(identity.get_project("acme").expect("get").is_some(), "the project is registered");
    assert!(!identity.is_valid_setup_token(&token).expect("valid"), "the flow completes and consumes the token");
}

#[test]
fn a_duplicate_project_key_in_the_setup_form_is_a_form_error_not_a_crash() {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let token = identity.create_setup_token(setup_token_ttl()).expect("mint");
    let base = start_with(identity.clone());

    // Reach the project step by creating the admin.
    post_setup(
        &base,
        &token,
        &[("email", "root@example.com"), ("display", "Root"), ("password", "root-correct-horse")],
    );

    // A project with the key the operator will submit already exists.
    identity.create_project("acme", "Existing Acme").expect("pre-seed a colliding project");

    // Submitting the colliding key is a form error, not a 500; the token survives.
    let (status, body) = post_setup(&base, &token, &[("project_key", "acme"), ("repo_key", "backend")]);
    assert_eq!(status, 200, "a duplicate key re-renders the form, not a 500");
    assert!(body.contains("name=\"project_key\""), "the project form is shown again with the error: {body}");
    assert!(identity.is_valid_setup_token(&token).expect("valid"), "a failed step does not consume the token");
    assert_eq!(
        identity.get_project("acme").expect("get").expect("exists").name,
        "Existing Acme",
        "the existing project is untouched",
    );
}

// --- the binary prints the token with guidance on a fresh start (決策 3) ---

fn server_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"))
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind free port")
        .local_addr()
        .unwrap()
        .port()
}

/// Write a server config with a SQLite identity at `identity_db`.
fn write_config(dir: &Path, identity_db: &Path) -> PathBuf {
    let path = dir.join("server.yaml");
    std::fs::write(
        &path,
        format!(
            "store:\n  driver: memory\nidentity:\n  driver: sqlite\n  path: {}\npublic_url: https://speclink.example\n",
            identity_db.display()
        ),
    )
    .expect("write config");
    path
}

/// Spawn the server binary with stdout captured; wait until `/healthz` answers,
/// then return the child, base URL and a receiver of its stdout lines.
fn spawn_capture(config: &Path) -> (Child, String, Receiver<String>) {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let mut child = Command::new(server_bin())
        .args(["--config", config.to_str().unwrap(), "--addr", &addr])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn speclink-server");
    let stdout = child.stdout.take().expect("piped stdout");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
    let base = format!("http://{addr}");
    let deadline = Instant::now() + StdDuration::from_secs(10);
    while Instant::now() < deadline {
        if ureq::get(&format!("{base}/healthz")).call().map(|r| r.status() == 200).unwrap_or(false) {
            break;
        }
        std::thread::sleep(StdDuration::from_millis(100));
    }
    (child, base, rx)
}

#[test]
fn a_fresh_database_prints_the_setup_token_with_guidance_on_stdout() {
    let dir = tempfile::tempdir().expect("workdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);

    let (mut child, _base, rx) = spawn_capture(&config);
    // The token prints before serve binds; give the line time to arrive.
    std::thread::sleep(StdDuration::from_millis(300));
    let lines: Vec<String> = rx.try_iter().collect();
    let _ = child.kill();
    let _ = child.wait();

    let setup_line = lines
        .iter()
        .find(|l| l.contains("/setup"))
        .unwrap_or_else(|| panic!("stdout carries a /setup guidance line: {lines:?}"));

    // The printed token validates against the store's stored hash.
    let token = setup_line
        .split("token=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_else(|| panic!("the guidance line carries a token= value: {setup_line}"))
        .trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
    let identity = IdentitySqlite::open(&identity_db).expect("reopen identity store");
    assert!(
        identity.is_valid_setup_token(token).expect("validate"),
        "the printed token validates against the stored hash: {token}",
    );
}
