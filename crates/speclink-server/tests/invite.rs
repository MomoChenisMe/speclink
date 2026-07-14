//! The `invite` subcommand: the headless management entry that mints a one-time
//! invitation against the identity database named by the config, prints its URL,
//! and refuses a duplicate email (server-identity spec「邀請一次性且到期失效」,
//! 決策 3). The existing `run` behaviour (`--config --addr`) is untouched.

use speclink_server::identity::{IdentitySqlite, IdentityStore};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn server_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"))
}

/// Write a server config whose identity store is the SQLite file at `identity_db`.
fn write_config(dir: &Path, identity_db: &Path) -> PathBuf {
    let path = dir.join("server.yaml");
    let mut file = std::fs::File::create(&path).expect("create config");
    write!(
        file,
        "store:\n  driver: memory\nidentity:\n  driver: sqlite\n  path: {}\npublic_url: https://speclink.example\n",
        identity_db.display()
    )
    .expect("write config");
    path
}

fn run_invite(config: &Path, args: &[&str]) -> Output {
    Command::new(server_bin())
        .arg("invite")
        .args(["--config", config.to_str().unwrap()])
        .args(args)
        .output()
        .expect("spawn invite subcommand")
}

/// Register `key` (with a `backend` repo) in the identity store's registry, so
/// the invite subcommand's `--project` check admits it. The connection is
/// dropped before the subcommand opens the same file.
fn seed_project(identity_db: &Path, key: &str) {
    let identity = IdentitySqlite::open(identity_db).expect("open identity to seed registry");
    identity.create_project(key, key).expect("seed project");
    identity.create_repo(key, "backend", "backend").expect("seed repo");
}

/// Extract the `/invite/<token>` token from the printed URL.
fn token_from(stdout: &str) -> String {
    let url = stdout
        .split_whitespace()
        .find(|w| w.contains("/invite/"))
        .unwrap_or_else(|| panic!("stdout carries an invite URL: {stdout}"));
    url.rsplit("/invite/").next().unwrap().trim().to_string()
}

#[test]
fn invite_mints_a_usable_one_time_url() {
    let dir = tempfile::tempdir().expect("tempdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);
    seed_project(&identity_db, "demo");

    let out = run_invite(
        &config,
        &["--email", "alice@example.com", "--display", "Alice", "--project", "demo", "--expires-in-days", "7"],
    );
    assert!(out.status.success(), "invite exits zero: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("https://speclink.example/invite/"), "URL uses the configured public origin: {stdout}");

    // The printed token resolves to a valid invitation in the identity store.
    let token = token_from(&stdout);
    let store = IdentitySqlite::open(&identity_db).expect("reopen identity store");
    let invitation = store.find_valid_invitation(&token).expect("lookup").expect("the invitation is valid");
    assert_eq!(invitation.email, "alice@example.com");
    assert_eq!(invitation.memberships, ["demo"]);
}

#[test]
fn a_duplicate_email_is_refused_non_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);
    seed_project(&identity_db, "demo");

    let first = run_invite(&config, &["--email", "bob@example.com", "--display", "Bob", "--project", "demo"]);
    assert!(first.status.success(), "first invite succeeds: {}", String::from_utf8_lossy(&first.stderr));

    // A second invitation for the same email, while the first is unexpired, is refused.
    let second = run_invite(&config, &["--email", "bob@example.com", "--display", "Bob", "--project", "demo"]);
    assert!(!second.status.success(), "a duplicate email is refused with a non-zero exit");
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("bob@example.com"), "the refusal names the email: {stderr}");
}

#[test]
fn an_unregistered_project_is_refused_and_lists_the_registered_keys() {
    let dir = tempfile::tempdir().expect("tempdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);
    seed_project(&identity_db, "demo");

    // Inviting into a project the registry does not hold is refused non-zero and
    // the stderr lists the registered keys; no invitation is minted.
    let out = run_invite(&config, &["--email", "carol@example.com", "--display", "Carol", "--project", "ghost"]);
    assert!(!out.status.success(), "an unregistered project is refused with a non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ghost"), "the refusal names the unregistered project: {stderr}");
    assert!(stderr.contains("demo"), "the refusal lists the registered project keys: {stderr}");

    // Nothing was written: the email has no outstanding invitation.
    let store = IdentitySqlite::open(&identity_db).expect("reopen identity store");
    assert!(
        store.create_invitation(speclink_server::identity::NewInvitation {
            email: "carol@example.com".to_string(),
            display: "Carol".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: chrono::Utc::now() + chrono::Duration::days(1),
        }).is_ok(),
        "no prior invitation for carol was minted, so a fresh one is accepted",
    );
}
