//! The three management entry points reach one single-point action with the
//! right audit source (server-admin spec「管理動作三入口同一實作且功能完備」, 決策
//! 2). Suspending one user each through the admin API (bearer, source api), the
//! /admin form (session, source web) and the server CLI subcommand (source cli)
//! leaves all three unable to authenticate and three correctly-sourced audit
//! records. The CLI runs the real server binary against the same identity file
//! the in-process server serves.

mod common;

use chrono::{Duration, Utc};
use speclink_protocol::API_VERSION;
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::EventSettings;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

fn server_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"))
}

/// Write a server config file that locates the identity database at `identity_db`
/// (an in-memory store; only identity matters for admin actions).
fn write_config(dir: &Path, identity_db: &Path) -> PathBuf {
    let path = dir.join("server.yaml");
    let mut file = std::fs::File::create(&path).expect("create config");
    write!(
        file,
        "store:\n  driver: memory\nidentity:\n  driver: sqlite\n  path: {}\n",
        identity_db.display()
    )
    .expect("write config");
    path
}

/// Seed a `demo`-member user with `email` and the given admin flag, minting a PAT
/// and a session. Returns `(pat, session, user_id)`.
fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> (String, String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("U <{email}>"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    let (_, pat) = identity.create_pat(&user_id, "cli", None).expect("pat");
    let session = identity.create_session(&user_id, Duration::days(1)).expect("session");
    (pat, session, user_id)
}

/// `GET /api/speclink/v1/projects/demo/binding` with `pat` — the status of a
/// member's next API request (200 while active, 401 once suspended).
fn binding_status(base: &str, pat: &str) -> u16 {
    let agent = ureq::builder().redirects(0).build();
    let result = agent
        .get(&format!("{base}/api/speclink/v1/projects/demo/binding"))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call();
    match result {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn suspension_is_equivalent_across_api_web_and_cli() {
    let dir = tempfile::tempdir().expect("tempdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);

    // Seed an admin (bearer + session) and three members over a file-backed
    // identity store the CLI can also open.
    let identity = Arc::new(IdentitySqlite::open(&identity_db).expect("identity"));
    common::seed_demo_registry(&*identity);
    let (admin_pat, admin_session, _admin_id) = seed_user(&identity, "admin@example.com", true);
    let (m1_pat, _, m1_id) = seed_user(&identity, "m1@example.com", false);
    let (m2_pat, _, m2_id) = seed_user(&identity, "m2@example.com", false);
    let (m3_pat, _, _m3_id) = seed_user(&identity, "m3@example.com", false);

    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(ServerConfig {
            store: StoreConfig::Memory,
            identity: IdentityConfig::Sqlite { path: identity_db.clone() },
            public_url: "http://127.0.0.1".to_string(),
            events: EventSettings::default(),
        }),
        identity: identity.clone(),
    };
    let base = common::start(state);

    // All three members can authenticate before suspension.
    for pat in [&m1_pat, &m2_pat, &m3_pat] {
        assert_eq!(binding_status(&base, pat), 200, "a member authenticates before suspension");
    }

    let http = ureq::builder().redirects(0).build();

    // Entry 1 — admin API (bearer): suspend m1, source api.
    let api = http
        .post(&format!("{base}/api/speclink/v1/admin/users/{m1_id}/suspend"))
        .set("Authorization", &format!("Bearer {admin_pat}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .call()
        .expect("api suspend");
    assert_eq!(api.status(), 200, "the admin API suspends m1");

    // Entry 2 — /admin form (session, no Origin so same-origin admits it): m2, source web.
    let web = http
        .post(&format!("{base}/admin/users/{m2_id}/suspend"))
        .set("Cookie", &format!("speclink_session={admin_session}"))
        .call()
        .expect("web suspend");
    assert!((300..400).contains(&web.status()), "the /admin form suspends m2 and redirects");

    // Entry 3 — server CLI subcommand: m3, source cli, operator system.
    let out = Command::new(server_bin())
        .args(["user", "suspend"])
        .args(["--config", config.to_str().unwrap()])
        .args(["--email", "m3@example.com"])
        .output()
        .expect("run cli suspend");
    assert!(out.status.success(), "cli suspend failed: {}", String::from_utf8_lossy(&out.stderr));

    // All three members now fail their next API request identically.
    for pat in [&m1_pat, &m2_pat, &m3_pat] {
        assert_eq!(binding_status(&base, pat), 401, "a suspended member's next request is 401");
    }

    // The audit trail carries three user-suspended records, one per source.
    let audit: serde_json::Value = http
        .get(&format!("{base}/api/speclink/v1/admin/audit"))
        .set("Authorization", &format!("Bearer {admin_pat}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .call()
        .expect("read audit")
        .into_json()
        .expect("audit json");
    let entries = audit["entries"].as_array().expect("entries array");
    let suspends: Vec<&serde_json::Value> = entries
        .iter()
        .filter(|e| e["action"] == "user-suspended")
        .collect();
    assert_eq!(suspends.len(), 3, "three suspensions recorded");
    let sources: std::collections::HashSet<&str> =
        suspends.iter().filter_map(|e| e["source"].as_str()).collect();
    assert_eq!(
        sources,
        ["api", "web", "cli"].into_iter().collect(),
        "the three sources are api, web and cli"
    );
    // Each suspension names its member as the subject.
    let subjects: std::collections::HashSet<&str> =
        suspends.iter().filter_map(|e| e["subject"].as_str()).collect();
    assert!(subjects.contains(m1_id.as_str()) && subjects.contains(m2_id.as_str()), "subjects are the suspended users");
}
