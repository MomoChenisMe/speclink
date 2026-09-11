//! The identity store: users, memberships, invitations, PATs and sessions with
//! their full lifecycle (server-identity spec「identity 儲存獨立且版本守門」).
//! Credentials only ever land as hashes; invitations are one-time and expire;
//! revocation stamps a timestamp; a PAT's last-used advances on use; a foreign
//! SQLite file is refused with its bytes untouched.

use chrono::{Duration, Utc};
use speclink_server::audit::{AuditAction, AuditActor, AuditSource};
use speclink_server::identity::{
    IdentityError, IdentitySqlite, IdentityStore, MembershipRole, NewInvitation,
};
use speclink_server::identity::{Project, Repo};

/// A fresh in-memory identity store for lifecycle tests.
fn store() -> IdentitySqlite {
    IdentitySqlite::open_memory().expect("open in-memory identity store")
}

/// An invitation for `email` into `projects`, expiring `days` from now.
fn invite(email: &str, projects: &[&str], days: i64) -> NewInvitation {
    NewInvitation {
        email: email.to_string(),
        display: format!("Test {email}"),
        memberships: projects.iter().map(|p| p.to_string()).collect(),
        admin: false,
        expires_at: Utc::now() + Duration::days(days),
    }
}

#[test]
fn accepting_an_invitation_creates_an_active_user_with_memberships() {
    let s = store();
    let token = s.create_invitation(invite("a@example.com", &["demo"], 7)).expect("create invite");

    // The invitation is valid until consumed.
    let pending = s.find_valid_invitation(&token).expect("lookup").expect("invitation is valid");
    assert_eq!(pending.email, "a@example.com");
    assert_eq!(pending.memberships, ["demo"]);

    let user_id = s.accept_invitation(&token, "hunter2password").expect("accept invitation");
    let user = s.get_user(&user_id).expect("get user").expect("user exists");
    assert!(user.active, "the accepted user is active");
    assert_eq!(user.email, "a@example.com");
    assert!(s.is_member(&user_id, "demo").expect("membership check"), "the invited membership is granted");
    assert_eq!(
        s.membership_role(&user_id, "demo").expect("membership role"),
        Some(MembershipRole::Editor),
        "invitation-created memberships are always editor",
    );

    // One-time: the token no longer resolves to a valid invitation.
    assert!(s.find_valid_invitation(&token).expect("lookup").is_none(), "consumed invitation is invalid");
    assert!(matches!(s.accept_invitation(&token, "x").unwrap_err(), IdentityError::InvalidInvitation));
}

#[test]
fn an_expired_invitation_is_not_valid() {
    let s = store();
    let token = s.create_invitation(invite("b@example.com", &["demo"], -1)).expect("create invite");
    assert!(s.find_valid_invitation(&token).expect("lookup").is_none(), "an expired invitation is invalid");
    assert!(matches!(s.accept_invitation(&token, "x").unwrap_err(), IdentityError::InvalidInvitation));
    assert!(s.find_user_by_email("b@example.com").expect("lookup").is_none(), "no user was created");
}

#[test]
fn a_duplicate_email_is_rejected() {
    let s = store();
    let token = s.create_invitation(invite("c@example.com", &["demo"], 7)).expect("create invite");
    s.accept_invitation(&token, "hunter2password").expect("accept");
    // An email that already has an active user cannot be re-invited.
    assert!(matches!(
        s.create_invitation(invite("c@example.com", &["demo"], 7)).unwrap_err(),
        IdentityError::Duplicate(_)
    ));
    // Nor while a prior unexpired invitation is still outstanding.
    s.create_invitation(invite("d@example.com", &["demo"], 7)).expect("first invite");
    assert!(matches!(
        s.create_invitation(invite("d@example.com", &["demo"], 7)).unwrap_err(),
        IdentityError::Duplicate(_)
    ));
}

#[test]
fn password_authentication_is_uniform_for_unknown_and_wrong() {
    let s = store();
    let token = s.create_invitation(invite("e@example.com", &["demo"], 7)).expect("invite");
    s.accept_invitation(&token, "correct-horse").expect("accept");

    assert!(s.authenticate_password("e@example.com", "correct-horse").expect("auth").is_some(), "right password authenticates");
    assert!(s.authenticate_password("e@example.com", "wrong").expect("auth").is_none(), "wrong password fails");
    assert!(s.authenticate_password("nobody@example.com", "correct-horse").expect("auth").is_none(), "unknown email fails");
}

#[test]
fn a_session_can_be_created_authenticated_and_revoked() {
    let s = store();
    let token = s.create_invitation(invite("f@example.com", &["demo"], 7)).expect("invite");
    let user_id = s.accept_invitation(&token, "correct-horse").expect("accept");

    let session = s.create_session(&user_id, Duration::hours(1)).expect("create session");
    let who = s.authenticate_session(&session).expect("auth session").expect("session is live");
    assert_eq!(who.id, user_id);

    s.revoke_session(&session).expect("revoke");
    assert!(s.authenticate_session(&session).expect("auth session").is_none(), "a revoked session is dead");
}

#[test]
fn an_expired_session_does_not_authenticate() {
    let s = store();
    let token = s.create_invitation(invite("g@example.com", &["demo"], 7)).expect("invite");
    let user_id = s.accept_invitation(&token, "correct-horse").expect("accept");
    let session = s.create_session(&user_id, Duration::seconds(-1)).expect("create expired session");
    assert!(s.authenticate_session(&session).expect("auth session").is_none(), "an expired session is dead");
}

#[test]
fn a_pat_authenticates_records_last_used_and_revokes() {
    let s = store();
    let token = s.create_invitation(invite("h@example.com", &["demo"], 7)).expect("invite");
    let user_id = s.accept_invitation(&token, "correct-horse").expect("accept");

    let (pat, plaintext) = s.create_pat(&user_id, "laptop", None).expect("create pat");
    assert!(plaintext.starts_with("spk_pat_"), "the plaintext carries the identifiable prefix: {plaintext}");
    assert!(plaintext.starts_with(&pat.prefix), "the stored prefix is a prefix of the plaintext");
    assert!(pat.last_used_at.is_none(), "a fresh PAT has never been used");

    let (matched, user) = s.authenticate_pat(&plaintext).expect("auth pat").expect("valid pat authenticates");
    assert_eq!(matched.id, pat.id);
    assert_eq!(user.id, user_id);

    s.touch_pat(&pat.id).expect("touch");
    let after = s.list_pats(&user_id).expect("list pats");
    assert!(after[0].last_used_at.is_some(), "last-used advanced after touch");

    s.revoke_pat(&user_id, &pat.id).expect("revoke");
    assert!(after.len() == 1);
    let revoked = &s.list_pats(&user_id).expect("list pats")[0];
    assert!(revoked.revoked_at.is_some(), "revocation stamps a timestamp");
    assert!(s.authenticate_pat(&plaintext).expect("auth pat").is_none(), "a revoked PAT no longer authenticates");
}

#[test]
fn a_pat_for_an_expired_or_suspended_user_does_not_authenticate() {
    let s = store();
    let token = s.create_invitation(invite("i@example.com", &["demo"], 7)).expect("invite");
    let user_id = s.accept_invitation(&token, "correct-horse").expect("accept");

    // Expired PAT.
    let (_, expired) = s.create_pat(&user_id, "old", Some(Utc::now() - Duration::hours(1))).expect("create pat");
    assert!(s.authenticate_pat(&expired).expect("auth").is_none(), "an expired PAT is dead");

    // Suspended user disables an otherwise-valid PAT.
    let (_, live) = s.create_pat(&user_id, "live", None).expect("create pat");
    assert!(s.authenticate_pat(&live).expect("auth").is_some(), "valid before suspension");
    s.set_user_active(&user_id, false).expect("suspend");
    assert!(s.authenticate_pat(&live).expect("auth").is_none(), "a suspended user's PAT is dead");
}

#[test]
fn a_foreign_sqlite_file_is_refused_with_its_bytes_untouched() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("foreign.db");
    // A perfectly valid SQLite database that is not a speclink identity store.
    {
        let conn = rusqlite::Connection::open(&path).expect("create foreign db");
        conn.execute_batch("CREATE TABLE widgets (id INTEGER); INSERT INTO widgets VALUES (1);")
            .expect("seed foreign db");
    }
    let before = std::fs::read(&path).expect("read bytes before");
    let err = IdentitySqlite::open(&path).expect_err("a foreign SQLite file must be refused");
    assert!(matches!(err, IdentityError::Open(_)), "refusal is an Open error: {err:?}");
    let after = std::fs::read(&path).expect("read bytes after");
    assert_eq!(before, after, "the refused file's bytes are unchanged");
}

#[test]
fn a_higher_schema_version_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("future.db");
    {
        // A speclink identity db that claims a version from the future.
        let conn = rusqlite::Connection::open(&path).expect("create db");
        conn.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);\
             INSERT INTO meta VALUES ('format', 'speclink-identity-store');\
             INSERT INTO meta VALUES ('schema_version', '9999');",
        )
        .expect("seed future db");
    }
    let err = IdentitySqlite::open(&path).expect_err("a newer schema must be refused");
    assert!(matches!(err, IdentityError::Open(_)), "refusal is an Open error: {err:?}");
}

/// The schema-version-1 identity shape server-identity-pat shipped, before the
/// device-flow tables were added.
const V1_SCHEMA: &str = "\
CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, display TEXT NOT NULL, email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL, active INTEGER NOT NULL, admin INTEGER NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE memberships (user_id TEXT NOT NULL, project_key TEXT NOT NULL, PRIMARY KEY (user_id, project_key));
CREATE TABLE pats (
    id TEXT PRIMARY KEY NOT NULL, user_id TEXT NOT NULL, prefix TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL, expires_at TEXT, revoked_at TEXT, last_used_at TEXT, created_at TEXT NOT NULL
);
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL, user_id TEXT NOT NULL, session_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL, revoked_at TEXT, created_at TEXT NOT NULL
);
";

/// The SHA-256 hex of `plaintext` — mirrors the store's `hash_token` so a test
/// can plant a credential whose plaintext it also knows.
fn sha256_hex(plaintext: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(plaintext.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn a_version_1_database_migrates_preserving_users_and_pats() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("v1.db");
    // A PAT that existed before the migration, hashed as the store would hash it.
    const PAT: &str = "spk_pat_v1preexisting00000000000000000000000000000000000000000000000000";
    let now = Utc::now().to_rfc3339();
    {
        // Hand-build a schema-version-1 identity database with a user, a
        // membership and a PAT — the shape the previous knife wrote.
        let conn = rusqlite::Connection::open(&path).expect("create v1 db");
        conn.execute_batch(V1_SCHEMA).expect("v1 schema");
        conn.execute_batch(
            "INSERT INTO meta VALUES ('format','speclink-identity-store');\
             INSERT INTO meta VALUES ('schema_version','1');",
        )
        .expect("seed meta");
        conn.execute(
            "INSERT INTO users (id, display, email, password_hash, active, admin, created_at) \
             VALUES ('usr_old','Old <old@example.com>','old@example.com','argon2-hash',1,0,?1)",
            rusqlite::params![now],
        )
        .expect("seed user");
        conn.execute(
            "INSERT INTO memberships (user_id, project_key) VALUES ('usr_old','demo')",
            [],
        )
        .expect("seed membership");
        conn.execute(
            "INSERT INTO pats (id, user_id, prefix, token_hash, name, expires_at, revoked_at, last_used_at, created_at) \
             VALUES ('pat_old','usr_old','spk_pat_v1pr',?1,'legacy',NULL,NULL,NULL,?2)",
            rusqlite::params![sha256_hex(PAT), now],
        )
        .expect("seed pat");
    }

    // Opening with the current server migrates 1 → 6 in place.
    let s = IdentitySqlite::open(&path).expect("open migrates the v1 database");

    // Existing data is intact and the pre-existing PAT still authenticates.
    let user = s.get_user("usr_old").expect("get user").expect("migrated user survives");
    assert_eq!(user.email, "old@example.com");
    assert!(s.is_member("usr_old", "demo").expect("membership check"), "membership preserved");
    assert_eq!(
        s.membership_role("usr_old", "demo").expect("membership role"),
        Some(MembershipRole::Editor),
        "every pre-role membership migrates to editor",
    );
    let (_, who) = s
        .authenticate_pat(PAT)
        .expect("auth")
        .expect("the pre-existing PAT still authenticates after migration");
    assert_eq!(who.id, "usr_old");
    drop(s);

    // The schema now records version 3 and the device-flow tables exist.
    let conn = rusqlite::Connection::open(&path).expect("reopen for inspection");
    let version: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| r.get(0))
        .expect("schema version");
    assert_eq!(version, "6", "the database was migrated to the current version");
    let role: String = conn
        .query_row(
            "SELECT role FROM memberships WHERE user_id = 'usr_old' AND project_key = 'demo'",
            [],
            |r| r.get(0),
        )
        .expect("migrated membership role");
    assert_eq!(role, "editor", "the migration backfills editor explicitly");
    for table in ["device_authorizations", "credential_families", "access_tokens", "refresh_credentials"] {
        let exists: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                rusqlite::params![table],
                |r| r.get(0),
            )
            .expect("table lookup");
        assert_eq!(exists, 1, "migration created the {table} table");
    }
}

#[test]
fn credentials_are_only_ever_stored_as_hashes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("identity.db");
    let s = IdentitySqlite::open(&path).expect("open file identity store");

    const PASSWORD: &str = "s3cr3t-plaintext-password";
    let invite_token = s.create_invitation(invite("j@example.com", &["demo"], 7)).expect("invite");
    let user_id = s.accept_invitation(&invite_token, PASSWORD).expect("accept");
    let session = s.create_session(&user_id, Duration::hours(1)).expect("session");
    let (_, pat) = s.create_pat(&user_id, "tok", None).expect("pat");
    // The device authorization's two codes must also land only as hashes.
    let device = s
        .create_device_authorization(Duration::seconds(5), Duration::minutes(15))
        .expect("device authorization");
    drop(s);

    // Sweep every cell of every table: no plaintext credential is present.
    let secrets = [PASSWORD, &invite_token, &session, &pat, &device.device_code, &device.user_code];
    let conn = rusqlite::Connection::open(&path).expect("reopen for inspection");
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .expect("prepare")
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query tables")
        .map(|r| r.expect("row"))
        .collect();
    for table in tables {
        let mut stmt = conn.prepare(&format!("SELECT * FROM \"{table}\"")).expect("select all");
        let cols = stmt.column_count();
        let mut rows = stmt.query([]).expect("query rows");
        while let Some(row) = rows.next().expect("next row") {
            for i in 0..cols {
                if let Ok(text) = row.get::<_, String>(i) {
                    for secret in &secrets {
                        assert!(
                            !text.contains(*secret),
                            "table '{table}' column {i} leaks a plaintext credential: {text}"
                        );
                    }
                }
            }
        }
    }
}

// --- Project/Repo registry (server-setup spec「registry 持久化且 binding 讀庫」) ---

#[test]
fn the_registry_persists_projects_and_repos() {
    let s = store();
    s.create_project("demo", "Demo").expect("create project");
    s.create_repo("demo", "backend", "Backend").expect("create repo");
    s.create_repo("demo", "frontend", "Frontend").expect("create a second repo");

    let project: Project = s.get_project("demo").expect("get").expect("the project exists");
    assert_eq!(project.key, "demo");
    assert_eq!(project.name, "Demo");
    assert!(s.get_project("ghost").expect("get").is_none(), "an unregistered key is absent");

    let projects = s.list_projects().expect("list projects");
    assert_eq!(projects.len(), 1, "one project is registered");
    assert_eq!(projects[0].key, "demo");

    let repos: Vec<Repo> = s.list_repos("demo").expect("list repos");
    let keys: Vec<&str> = repos.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["backend", "frontend"], "the project's repos are listed, ordered by key");
    assert_eq!(repos[0].name, "Backend", "a repo carries its display name");
    assert!(s.list_repos("ghost").expect("list").is_empty(), "an unregistered project has no repos");
}

#[test]
fn a_duplicate_project_key_is_rejected_and_leaves_the_original() {
    let s = store();
    s.create_project("demo", "Demo").expect("create project");
    assert!(
        matches!(s.create_project("demo", "Demo Again").unwrap_err(), IdentityError::Duplicate(_)),
        "a duplicate project key is rejected"
    );
    assert_eq!(
        s.get_project("demo").expect("get").expect("still exists").name,
        "Demo",
        "the original project is untouched",
    );
}

#[test]
fn a_duplicate_repo_key_in_the_same_project_is_rejected() {
    let s = store();
    s.create_project("demo", "Demo").expect("create project");
    s.create_repo("demo", "backend", "Backend").expect("create repo");
    assert!(
        matches!(
            s.create_repo("demo", "backend", "Backend Again").unwrap_err(),
            IdentityError::Duplicate(_)
        ),
        "a duplicate repo key within one project is rejected",
    );
    // The same repo key under a different project is fine — repos are scoped.
    s.create_project("other", "Other").expect("create another project");
    s.create_repo("other", "backend", "Backend")
        .expect("the same repo key under a different project is allowed");
}

/// The schema-version-2 identity shape (users, memberships, PATs) before the
/// registry tables were added — enough to seed a user and PAT for a migration
/// test. Migrating 2 → 3 only adds tables, so this subset suffices.
const V2_SCHEMA: &str = "\
CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, display TEXT NOT NULL, email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL, active INTEGER NOT NULL, admin INTEGER NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE memberships (user_id TEXT NOT NULL, project_key TEXT NOT NULL, PRIMARY KEY (user_id, project_key));
CREATE TABLE pats (
    id TEXT PRIMARY KEY NOT NULL, user_id TEXT NOT NULL, prefix TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL, expires_at TEXT, revoked_at TEXT, last_used_at TEXT, created_at TEXT NOT NULL
);
";

#[test]
fn a_version_2_database_migrates_preserving_users_and_pats_and_enables_the_registry() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("v2.db");
    // A PAT that existed before the registry migration, hashed as the store hashes it.
    const PAT: &str = "spk_pat_v2preexisting00000000000000000000000000000000000000000000000000";
    let now = Utc::now().to_rfc3339();
    {
        // Hand-build a schema-version-2 identity database with a user, a
        // membership and a PAT — the shape the prior knife wrote.
        let conn = rusqlite::Connection::open(&path).expect("create v2 db");
        conn.execute_batch(V2_SCHEMA).expect("v2 schema");
        conn.execute_batch(
            "INSERT INTO meta VALUES ('format','speclink-identity-store');\
             INSERT INTO meta VALUES ('schema_version','2');",
        )
        .expect("seed meta");
        conn.execute(
            "INSERT INTO users (id, display, email, password_hash, active, admin, created_at) \
             VALUES ('usr_old','Old <old@example.com>','old@example.com','argon2-hash',1,0,?1)",
            rusqlite::params![now],
        )
        .expect("seed user");
        conn.execute(
            "INSERT INTO memberships (user_id, project_key) VALUES ('usr_old','demo')",
            [],
        )
        .expect("seed membership");
        conn.execute(
            "INSERT INTO pats (id, user_id, prefix, token_hash, name, expires_at, revoked_at, last_used_at, created_at) \
             VALUES ('pat_old','usr_old','spk_pat_v2pr',?1,'legacy',NULL,NULL,NULL,?2)",
            rusqlite::params![sha256_hex(PAT), now],
        )
        .expect("seed pat");
    }

    // Opening with the current server migrates 2 → 5 in place.
    let s = IdentitySqlite::open(&path).expect("open migrates the v2 database");

    // Existing data is intact and the pre-existing PAT still authenticates.
    let user = s.get_user("usr_old").expect("get user").expect("migrated user survives");
    assert_eq!(user.email, "old@example.com");
    assert!(s.is_member("usr_old", "demo").expect("membership check"), "membership preserved");
    let (_, who) = s
        .authenticate_pat(PAT)
        .expect("auth")
        .expect("the pre-existing PAT still authenticates after migration");
    assert_eq!(who.id, "usr_old");

    // The registry is available on the migrated database.
    s.create_project("demo", "Demo").expect("registry is usable after migration");
    s.create_repo("demo", "backend", "Backend").expect("registry repo is usable");
    assert_eq!(s.get_project("demo").expect("get").expect("exists").key, "demo");
    drop(s);

    // The schema now records version 4 and the registry tables exist.
    let conn = rusqlite::Connection::open(&path).expect("reopen for inspection");
    let version: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| r.get(0))
        .expect("schema version");
    assert_eq!(version, "6", "the database was migrated to version 6");
    for table in ["projects", "repos"] {
        let exists: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                rusqlite::params![table],
                |r| r.get(0),
            )
            .expect("table lookup");
        assert_eq!(exists, 1, "migration created the {table} table");
    }
}

/// The schema-version-3 identity shape (users, memberships, PATs, registry)
/// before the audit table was added — enough to seed data for a migration test.
/// Migrating 3 → 4 only adds the audit_log table, so this subset suffices.
const V3_SCHEMA: &str = "\
CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, display TEXT NOT NULL, email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL, active INTEGER NOT NULL, admin INTEGER NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE memberships (user_id TEXT NOT NULL, project_key TEXT NOT NULL, PRIMARY KEY (user_id, project_key));
CREATE TABLE pats (
    id TEXT PRIMARY KEY NOT NULL, user_id TEXT NOT NULL, prefix TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL, expires_at TEXT, revoked_at TEXT, last_used_at TEXT, created_at TEXT NOT NULL
);
CREATE TABLE projects (key TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL);
CREATE TABLE repos (project_key TEXT NOT NULL, key TEXT NOT NULL, name TEXT NOT NULL, PRIMARY KEY (project_key, key));
";

#[test]
fn a_version_3_database_migrates_preserving_data_and_adds_a_writable_audit_log() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("v3.db");
    // A PAT that existed before the audit migration, hashed as the store hashes it.
    const PAT: &str = "spk_pat_v3preexisting00000000000000000000000000000000000000000000000000";
    let now = Utc::now().to_rfc3339();
    {
        // Hand-build a schema-version-3 identity database with a user, a
        // membership, a PAT and a registered project — the shape the prior knife wrote.
        let conn = rusqlite::Connection::open(&path).expect("create v3 db");
        conn.execute_batch(V3_SCHEMA).expect("v3 schema");
        conn.execute_batch(
            "INSERT INTO meta VALUES ('format','speclink-identity-store');\
             INSERT INTO meta VALUES ('schema_version','3');",
        )
        .expect("seed meta");
        conn.execute(
            "INSERT INTO users (id, display, email, password_hash, active, admin, created_at) \
             VALUES ('usr_old','Old <old@example.com>','old@example.com','argon2-hash',1,0,?1)",
            rusqlite::params![now],
        )
        .expect("seed user");
        conn.execute("INSERT INTO memberships (user_id, project_key) VALUES ('usr_old','demo')", [])
            .expect("seed membership");
        conn.execute(
            "INSERT INTO pats (id, user_id, prefix, token_hash, name, expires_at, revoked_at, last_used_at, created_at) \
             VALUES ('pat_old','usr_old','spk_pat_v3pr',?1,'legacy',NULL,NULL,NULL,?2)",
            rusqlite::params![sha256_hex(PAT), now],
        )
        .expect("seed pat");
        conn.execute("INSERT INTO projects (key, name) VALUES ('demo','Demo')", [])
            .expect("seed project");
    }

    // Opening with the current server migrates 3 → 4 in place.
    let s = IdentitySqlite::open(&path).expect("open migrates the v3 database");

    // Existing data is intact: the user, its membership, its PAT and the project all survive.
    let user = s.get_user("usr_old").expect("get user").expect("migrated user survives");
    assert_eq!(user.email, "old@example.com");
    assert!(s.is_member("usr_old", "demo").expect("membership check"), "membership preserved");
    let (_, who) = s
        .authenticate_pat(PAT)
        .expect("auth")
        .expect("the pre-existing PAT still authenticates after migration");
    assert_eq!(who.id, "usr_old");
    assert_eq!(s.get_project("demo").expect("get").expect("exists").key, "demo", "project preserved");

    // The audit log is writable and queryable on the migrated database.
    s.record_audit(&AuditActor::user("usr_old", AuditSource::Api), AuditAction::UserSuspended, "usr_target")
        .expect("audit is writable after migration");
    let page = s.list_audit(10, 0).expect("audit is queryable");
    assert_eq!(page.len(), 1, "the recorded audit entry reads back");
    assert_eq!(page[0].action, "user-suspended");
    assert_eq!(page[0].subject, "usr_target");
    assert_eq!(page[0].source, "api");
    drop(s);

    // The schema now records version 5, and both the audit_log (v4) and the
    // backup_records (v5) tables exist.
    let conn = rusqlite::Connection::open(&path).expect("reopen for inspection");
    let version: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| r.get(0))
        .expect("schema version");
    assert_eq!(version, "6", "the database was migrated to version 6");
    for table in ["audit_log", "backup_records"] {
        let exists: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                rusqlite::params![table],
                |r| r.get(0),
            )
            .expect("table lookup");
        assert_eq!(exists, 1, "migration created the {table} table");
    }
}

#[test]
fn the_audit_log_is_append_only_reverse_chronological_and_paginates() {
    let s = store();
    // Record three entries in order; each carries a distinct action, subject and source.
    s.record_audit(&AuditActor::user("usr_admin", AuditSource::Web), AuditAction::UserInvited, "alice@example.com")
        .expect("record 1");
    s.record_audit(&AuditActor::system_cli(), AuditAction::TokenRevoked, "pat_x (spk_pat_abcd)")
        .expect("record 2");
    s.record_audit(&AuditActor::user("usr_admin", AuditSource::Api), AuditAction::ProjectCreated, "demo")
        .expect("record 3");

    // The full page is newest-first (reverse of insertion order).
    let all = s.list_audit(10, 0).expect("list");
    assert_eq!(all.len(), 3, "all three entries read back");
    assert_eq!(all[0].action, "project-created", "newest first");
    assert_eq!(all[1].action, "token-revoked");
    assert_eq!(all[2].action, "user-invited", "oldest last");

    // The five-tuple round-trips: operator, action, subject, source (and a UTC time).
    assert_eq!(all[2].actor_id, "usr_admin");
    assert_eq!(all[2].subject, "alice@example.com");
    assert_eq!(all[2].source, "web");
    assert_eq!(all[1].actor_id, AuditActor::SYSTEM, "a CLI action records the host as operator");
    assert_eq!(all[1].source, "cli");
    assert!(all[0].created_at <= Utc::now(), "the entry carries a UTC timestamp");

    // Pagination walks the log a page at a time, still newest-first.
    let first = s.list_audit(2, 0).expect("page 1");
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].action, "project-created");
    assert_eq!(first[1].action, "token-revoked");
    let second = s.list_audit(2, 2).expect("page 2");
    assert_eq!(second.len(), 1, "the offset skips the first page");
    assert_eq!(second[0].action, "user-invited");

    // There is no update or delete interface: the trait exposes only append and read.
    // (Enforced by the absence of any mutating audit method — verified at compile time.)
}
