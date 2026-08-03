//! The headless management subcommands run the same single-point actions as the
//! admin API and /admin forms, recording source cli and operator system
//! (server-admin spec「管理動作三入口同一實作且功能完備」, 決策 2). Driven against the
//! real server binary over a file-backed identity store, with the resulting state
//! and audit verified directly.

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;

fn server_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"))
}

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

/// Run a management subcommand against `config`; returns the process output.
fn run(config: &Path, args: &[&str]) -> Output {
    Command::new(server_bin())
        .args(args)
        .args(["--config", config.to_str().unwrap()])
        .output()
        .unwrap_or_else(|e| panic!("run {args:?}: {e}"))
}

#[test]
fn the_management_subcommands_run_headless_and_record_source_cli() {
    let _gate = crate::common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let identity_db = dir.path().join("identity.db");
    let config = write_config(dir.path(), &identity_db);

    // Seed an admin (so suspend is never blocked) and a member with a PAT.
    let identity = Arc::new(IdentitySqlite::open(&identity_db).expect("identity"));
    let admin_token = identity
        .create_invitation(NewInvitation {
            email: "admin@example.com".to_string(),
            display: "Admin".to_string(),
            memberships: vec![],
            admin: true,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite admin");
    identity.accept_invitation(&admin_token, "pw").expect("accept admin");
    let member_token = identity
        .create_invitation(NewInvitation {
            email: "member@example.com".to_string(),
            display: "Member".to_string(),
            memberships: vec![],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite member");
    let member_id = identity.accept_invitation(&member_token, "pw").expect("accept member");
    let (member_pat, member_pat_plaintext) = identity.create_pat(&member_id, "tok", None).expect("pat");

    // suspend → reactivate toggles the member's active state.
    assert!(run(&config, &["user", "suspend", "--email", "member@example.com"]).status.success());
    assert!(!identity.get_user(&member_id).unwrap().unwrap().active, "suspend took effect");
    assert!(run(&config, &["user", "reactivate", "--email", "member@example.com"]).status.success());
    assert!(identity.get_user(&member_id).unwrap().unwrap().active, "reactivate took effect");

    // token revoke stops the PAT authenticating.
    assert!(identity.authenticate_pat(&member_pat_plaintext).unwrap().is_some(), "PAT works before revoke");
    assert!(run(&config, &["token", "revoke", "--token-id", &member_pat.id]).status.success());
    assert!(identity.authenticate_pat(&member_pat_plaintext).unwrap().is_none(), "PAT stops after revoke");

    // project create then repo create register a new scope.
    assert!(run(&config, &["project", "create", "--key", "p2", "--name", "Project 2"]).status.success());
    assert!(run(&config, &["repo", "create", "--project", "p2", "--key", "r2"]).status.success());
    assert_eq!(identity.get_project("p2").unwrap().unwrap().name, "Project 2");
    assert_eq!(identity.list_repos("p2").unwrap()[0].key, "r2");

    // A duplicate key is refused with a non-zero exit and no phantom audit.
    let dup = run(&config, &["project", "create", "--key", "p2"]);
    assert!(!dup.status.success(), "a duplicate project key exits non-zero");

    // Every recorded action carries source cli and operator system.
    let audit = identity.list_audit(100, 0).expect("audit");
    let cli_actions: Vec<&str> = audit
        .iter()
        .filter(|e| e.source == "cli")
        .map(|e| e.action.as_str())
        .collect();
    for expected in ["user-suspended", "user-reactivated", "token-revoked", "project-created", "repo-created"] {
        assert!(cli_actions.contains(&expected), "cli recorded a {expected} audit");
    }
    assert!(
        audit.iter().filter(|e| e.source == "cli").all(|e| e.actor_id == "system"),
        "every cli action records the host (system) as operator"
    );
}
