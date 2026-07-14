//! The identity store: users, memberships, invitations, PATs and sessions with
//! their full lifecycle (server-identity spec「identity 儲存獨立且版本守門」).
//! Credentials only ever land as hashes; invitations are one-time and expire;
//! revocation stamps a timestamp; a PAT's last-used advances on use; a foreign
//! SQLite file is refused with its bytes untouched.

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentityError, IdentityStore, IdentitySqlite, NewInvitation};

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
    drop(s);

    // Sweep every cell of every table: no plaintext credential is present.
    let secrets = [PASSWORD, &invite_token, &session, &pat];
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
