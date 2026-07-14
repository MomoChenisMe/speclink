//! The SQLite identity store: a server-owned database, separate from the
//! TeamStore's (決策 1). A `meta` table records the schema version and an
//! identity marker; opening gates on them read-only before any write, so a
//! foreign or newer database is refused with its bytes untouched — the same
//! discipline as the sqlite TeamStore driver.

use crate::identity::{
    hash_password, hash_token, random_token, verify_password, IdentityError, IdentityStore,
    Invitation, NewInvitation, Pat, SessionInfo, User,
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// The schema version this server reads and writes. A database recording a
/// higher version is refused. Bump and add a migration when the shape changes.
const SCHEMA_VERSION: u32 = 1;

/// The marker written to `meta` that distinguishes a speclink identity store
/// from an unrelated SQLite file.
const IDENTITY_MARKER: &str = "speclink-identity-store";

const SCHEMA_SQL: &str = "\
CREATE TABLE meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE TABLE users (
    id            TEXT PRIMARY KEY NOT NULL,
    display       TEXT NOT NULL,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    active        INTEGER NOT NULL,
    admin         INTEGER NOT NULL,
    created_at    TEXT NOT NULL
);
CREATE TABLE memberships (
    user_id     TEXT NOT NULL,
    project_key TEXT NOT NULL,
    PRIMARY KEY (user_id, project_key)
);
CREATE TABLE invitations (
    id          TEXT PRIMARY KEY NOT NULL,
    email       TEXT NOT NULL,
    display     TEXT NOT NULL,
    admin       INTEGER NOT NULL,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TEXT NOT NULL,
    consumed_at TEXT,
    created_at  TEXT NOT NULL
);
CREATE TABLE invitation_memberships (
    invitation_id TEXT NOT NULL,
    project_key   TEXT NOT NULL,
    PRIMARY KEY (invitation_id, project_key)
);
CREATE TABLE pats (
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT NOT NULL,
    prefix       TEXT NOT NULL,
    token_hash   TEXT NOT NULL UNIQUE,
    name         TEXT NOT NULL,
    expires_at   TEXT,
    revoked_at   TEXT,
    last_used_at TEXT,
    created_at   TEXT NOT NULL
);
CREATE TABLE sessions (
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT NOT NULL,
    session_hash TEXT NOT NULL UNIQUE,
    expires_at   TEXT NOT NULL,
    revoked_at   TEXT,
    created_at   TEXT NOT NULL
);
";

/// A SQLite-backed identity store. Reads and writes are serialized behind a
/// mutex — identity lookups are light single-row queries in the single-node
/// design, so per-request locking is acceptable (決策 5).
pub struct IdentitySqlite {
    inner: Mutex<Connection>,
}

impl std::fmt::Debug for IdentitySqlite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentitySqlite").finish_non_exhaustive()
    }
}

impl IdentitySqlite {
    /// Open (or initialize) a file-backed identity store, fail closed.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, IdentityError> {
        let conn = Connection::open(path.as_ref()).map_err(open_err)?;
        Self::gate_and_init(conn)
    }

    /// A private in-memory identity store — test configurations only. The
    /// database lives only as long as the connection.
    pub fn open_memory() -> Result<Self, IdentityError> {
        let conn = Connection::open_in_memory().map_err(open_err)?;
        Self::gate_and_init(conn)
    }

    /// Gate on the schema marker/version read-only, then initialize if the file
    /// is fresh. No write reaches a database we go on to refuse.
    fn gate_and_init(conn: Connection) -> Result<Self, IdentityError> {
        // Wait on a briefly-held lock rather than failing at once — the invite
        // subcommand and the running server can touch the same file.
        conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(open_err)?;
        let has_meta = table_exists(&conn, "meta")?;
        if has_meta {
            let marker: Option<String> = meta_value(&conn, "format")?;
            if marker.as_deref() != Some(IDENTITY_MARKER) {
                return Err(IdentityError::Open(
                    "database has a meta table but not the speclink identity marker".into(),
                ));
            }
            let version = meta_value(&conn, "schema_version")?
                .and_then(|v| v.parse::<u32>().ok())
                .ok_or_else(|| {
                    IdentityError::Open("identity schema version missing or unparseable".into())
                })?;
            if version > SCHEMA_VERSION {
                return Err(IdentityError::Open(format!(
                    "incompatible identity schema version {version}; this server supports {SCHEMA_VERSION}"
                )));
            }
        } else if has_any_user_table(&conn)? {
            return Err(IdentityError::Open(
                "existing SQLite database is not a speclink identity store".into(),
            ));
        }

        // Past the gate: initialize a fresh database in one transaction.
        conn.execute_batch("PRAGMA foreign_keys=ON;").map_err(open_err)?;
        if !has_meta {
            conn.execute_batch(&format!(
                "BEGIN;\n{SCHEMA_SQL}\nINSERT INTO meta (key, value) VALUES ('format', '{IDENTITY_MARKER}');\nINSERT INTO meta (key, value) VALUES ('schema_version', '{SCHEMA_VERSION}');\nCOMMIT;"
            ))
            .map_err(open_err)?;
        }
        Ok(Self { inner: Mutex::new(conn) })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.inner.lock().expect("identity store mutex poisoned")
    }
}

impl IdentityStore for IdentitySqlite {
    fn create_invitation(&self, req: NewInvitation) -> Result<String, IdentityError> {
        let mut guard = self.conn();
        let now = now();
        // Guard: no active user and no outstanding invitation for this email.
        let active_user: Option<i64> = guard
            .query_row(
                "SELECT 1 FROM users WHERE email = ?1 AND active = 1 LIMIT 1",
                params![req.email],
                |r| r.get(0),
            )
            .optional()
            .map_err(backend)?;
        if active_user.is_some() {
            return Err(IdentityError::Duplicate(format!(
                "an active user already exists for '{}'",
                req.email
            )));
        }
        let pending: Option<i64> = guard
            .query_row(
                "SELECT 1 FROM invitations WHERE email = ?1 AND consumed_at IS NULL AND expires_at > ?2 LIMIT 1",
                params![req.email, now],
                |r| r.get(0),
            )
            .optional()
            .map_err(backend)?;
        if pending.is_some() {
            return Err(IdentityError::Duplicate(format!(
                "an unexpired invitation already exists for '{}'",
                req.email
            )));
        }

        let plaintext = random_token();
        let id = new_id("inv");
        let tx = guard.transaction().map_err(backend)?;
        tx.execute(
            "INSERT INTO invitations (id, email, display, admin, token_hash, expires_at, consumed_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
            params![id, req.email, req.display, req.admin as i64, hash_token(&plaintext), ts(req.expires_at), now],
        )
        .map_err(backend)?;
        for project in &req.memberships {
            tx.execute(
                "INSERT INTO invitation_memberships (invitation_id, project_key) VALUES (?1, ?2)",
                params![id, project],
            )
            .map_err(backend)?;
        }
        tx.commit().map_err(backend)?;
        Ok(plaintext)
    }

    fn find_valid_invitation(&self, token: &str) -> Result<Option<Invitation>, IdentityError> {
        let guard = self.conn();
        let hash = hash_token(token);
        let row = guard
            .query_row(
                "SELECT id, email, display, admin, expires_at FROM invitations WHERE token_hash = ?1 AND consumed_at IS NULL AND expires_at > ?2",
                params![hash, now()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(backend)?;
        let Some((id, email, display, admin, expires_at)) = row else {
            return Ok(None);
        };
        let memberships = invitation_memberships(&guard, &id)?;
        Ok(Some(Invitation {
            id,
            email,
            display,
            admin: admin != 0,
            memberships,
            expires_at: parse_ts(&expires_at)?,
        }))
    }

    fn accept_invitation(&self, token: &str, password: &str) -> Result<String, IdentityError> {
        let hash = hash_token(token);
        let password_hash = hash_password(password)?;
        let mut guard = self.conn();
        let now = now();
        let tx = guard.transaction().map_err(backend)?;
        // Re-check validity inside the transaction so a race or expiry between
        // the GET and the POST cannot slip a used/expired invitation through.
        let row = tx
            .query_row(
                "SELECT id, email, display, admin FROM invitations WHERE token_hash = ?1 AND consumed_at IS NULL AND expires_at > ?2",
                params![hash, now],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(backend)?;
        let Some((inv_id, email, display, admin)) = row else {
            return Err(IdentityError::InvalidInvitation);
        };

        let user_id = new_id("usr");
        tx.execute(
            "INSERT INTO users (id, display, email, password_hash, active, admin, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            params![user_id, display, email, password_hash, admin, now],
        )
        .map_err(backend)?;
        let projects: Vec<String> = {
            let mut stmt = tx
                .prepare("SELECT project_key FROM invitation_memberships WHERE invitation_id = ?1")
                .map_err(backend)?;
            let keys = stmt
                .query_map(params![inv_id], |r| r.get::<_, String>(0))
                .map_err(backend)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(backend)?;
            keys
        };
        for project in projects {
            tx.execute(
                "INSERT INTO memberships (user_id, project_key) VALUES (?1, ?2)",
                params![user_id, project],
            )
            .map_err(backend)?;
        }
        tx.execute(
            "UPDATE invitations SET consumed_at = ?2 WHERE id = ?1",
            params![inv_id, now],
        )
        .map_err(backend)?;
        tx.commit().map_err(backend)?;
        Ok(user_id)
    }

    fn get_user(&self, user_id: &str) -> Result<Option<User>, IdentityError> {
        self.conn()
            .query_row(
                "SELECT id, display, email, active, admin FROM users WHERE id = ?1",
                params![user_id],
                map_user,
            )
            .optional()
            .map_err(backend)
    }

    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, IdentityError> {
        self.conn()
            .query_row(
                "SELECT id, display, email, active, admin FROM users WHERE email = ?1",
                params![email],
                map_user,
            )
            .optional()
            .map_err(backend)
    }

    fn authenticate_password(&self, email: &str, password: &str) -> Result<Option<User>, IdentityError> {
        let guard = self.conn();
        let row = guard
            .query_row(
                "SELECT id, display, email, active, admin, password_hash FROM users WHERE email = ?1",
                params![email],
                |r| {
                    Ok((
                        User {
                            id: r.get(0)?,
                            display: r.get(1)?,
                            email: r.get(2)?,
                            active: r.get::<_, i64>(3)? != 0,
                            admin: r.get::<_, i64>(4)? != 0,
                        },
                        r.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(backend)?;
        match row {
            Some((user, hash)) => {
                // Always run exactly one verify, even for a suspended user, so
                // every found-email path costs the same — a suspended account is
                // not distinguishable by timing from a wrong password.
                let ok = verify_password(&hash, password);
                if user.active && ok {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => {
                // Equalize timing with the found path: hash the password and
                // discard, so an unknown email is indistinguishable from a wrong
                // one.
                let _ = hash_password(password);
                Ok(None)
            }
        }
    }

    fn set_user_active(&self, user_id: &str, active: bool) -> Result<(), IdentityError> {
        self.conn()
            .execute(
                "UPDATE users SET active = ?2 WHERE id = ?1",
                params![user_id, active as i64],
            )
            .map(|_| ())
            .map_err(backend)
    }

    fn is_member(&self, user_id: &str, project_key: &str) -> Result<bool, IdentityError> {
        let found: Option<i64> = self
            .conn()
            .query_row(
                "SELECT 1 FROM memberships WHERE user_id = ?1 AND project_key = ?2",
                params![user_id, project_key],
                |r| r.get(0),
            )
            .optional()
            .map_err(backend)?;
        Ok(found.is_some())
    }

    fn create_session(&self, user_id: &str, ttl: Duration) -> Result<String, IdentityError> {
        let plaintext = random_token();
        let now = Utc::now();
        self.conn()
            .execute(
                "INSERT INTO sessions (id, user_id, session_hash, expires_at, revoked_at, created_at) VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
                params![new_id("ses"), user_id, hash_token(&plaintext), ts(now + ttl), ts(now)],
            )
            .map_err(backend)?;
        Ok(plaintext)
    }

    fn authenticate_session(&self, session_id: &str) -> Result<Option<User>, IdentityError> {
        self.conn()
            .query_row(
                "SELECT u.id, u.display, u.email, u.active, u.admin \
                 FROM sessions s JOIN users u ON u.id = s.user_id \
                 WHERE s.session_hash = ?1 AND s.revoked_at IS NULL AND s.expires_at > ?2 AND u.active = 1",
                params![hash_token(session_id), now()],
                map_user,
            )
            .optional()
            .map_err(backend)
    }

    fn revoke_session(&self, session_id: &str) -> Result<(), IdentityError> {
        self.conn()
            .execute(
                "UPDATE sessions SET revoked_at = ?2 WHERE session_hash = ?1 AND revoked_at IS NULL",
                params![hash_token(session_id), now()],
            )
            .map(|_| ())
            .map_err(backend)
    }

    fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionInfo>, IdentityError> {
        let guard = self.conn();
        let mut stmt = guard
            .prepare(
                "SELECT id, created_at, expires_at, revoked_at FROM sessions WHERE user_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(backend)?;
        let rows = stmt
            .query_map(params![user_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(backend)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, created_at, expires_at, revoked_at) = row.map_err(backend)?;
            out.push(SessionInfo {
                id,
                created_at: parse_ts(&created_at)?,
                expires_at: parse_ts(&expires_at)?,
                revoked_at: parse_opt_ts(revoked_at)?,
            });
        }
        Ok(out)
    }

    fn create_pat(
        &self,
        user_id: &str,
        name: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(Pat, String), IdentityError> {
        let plaintext = format!("spk_pat_{}", random_token());
        let prefix: String = plaintext.chars().take(12).collect();
        let id = new_id("pat");
        let created_at = Utc::now();
        self.conn()
            .execute(
                "INSERT INTO pats (id, user_id, prefix, token_hash, name, expires_at, revoked_at, last_used_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7)",
                params![id, user_id, prefix, hash_token(&plaintext), name, expires_at.map(ts), ts(created_at)],
            )
            .map_err(backend)?;
        let pat = Pat {
            id,
            user_id: user_id.to_string(),
            prefix,
            name: name.to_string(),
            expires_at,
            revoked_at: None,
            last_used_at: None,
            created_at,
        };
        Ok((pat, plaintext))
    }

    fn list_pats(&self, user_id: &str) -> Result<Vec<Pat>, IdentityError> {
        let guard = self.conn();
        let mut stmt = guard
            .prepare(
                "SELECT id, user_id, prefix, name, expires_at, revoked_at, last_used_at, created_at FROM pats WHERE user_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(backend)?;
        let rows = stmt.query_map(params![user_id], map_pat_row).map_err(backend)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(backend)?.into_pat()?);
        }
        Ok(out)
    }

    fn revoke_pat(&self, user_id: &str, pat_id: &str) -> Result<(), IdentityError> {
        self.conn()
            .execute(
                "UPDATE pats SET revoked_at = ?3 WHERE id = ?1 AND user_id = ?2 AND revoked_at IS NULL",
                params![pat_id, user_id, now()],
            )
            .map(|_| ())
            .map_err(backend)
    }

    fn authenticate_pat(&self, token: &str) -> Result<Option<(Pat, User)>, IdentityError> {
        let guard = self.conn();
        let row = guard
            .query_row(
                "SELECT p.id, p.user_id, p.prefix, p.name, p.expires_at, p.revoked_at, p.last_used_at, p.created_at, \
                        u.id, u.display, u.email, u.active, u.admin \
                 FROM pats p JOIN users u ON u.id = p.user_id \
                 WHERE p.token_hash = ?1 AND p.revoked_at IS NULL AND (p.expires_at IS NULL OR p.expires_at > ?2) AND u.active = 1",
                params![hash_token(token), now()],
                |r| {
                    let raw = PatRow {
                        id: r.get(0)?,
                        user_id: r.get(1)?,
                        prefix: r.get(2)?,
                        name: r.get(3)?,
                        expires_at: r.get(4)?,
                        revoked_at: r.get(5)?,
                        last_used_at: r.get(6)?,
                        created_at: r.get(7)?,
                    };
                    let user = User {
                        id: r.get(8)?,
                        display: r.get(9)?,
                        email: r.get(10)?,
                        active: r.get::<_, i64>(11)? != 0,
                        admin: r.get::<_, i64>(12)? != 0,
                    };
                    Ok((raw, user))
                },
            )
            .optional()
            .map_err(backend)?;
        match row {
            Some((raw, user)) => Ok(Some((raw.into_pat()?, user))),
            None => Ok(None),
        }
    }

    fn touch_pat(&self, pat_id: &str) -> Result<(), IdentityError> {
        self.conn()
            .execute("UPDATE pats SET last_used_at = ?2 WHERE id = ?1", params![pat_id, now()])
            .map(|_| ())
            .map_err(backend)
    }
}

// --- row mapping ---

/// Map a `(id, display, email, active, admin)` row to a [`User`].
fn map_user(r: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: r.get(0)?,
        display: r.get(1)?,
        email: r.get(2)?,
        active: r.get::<_, i64>(3)? != 0,
        admin: r.get::<_, i64>(4)? != 0,
    })
}

/// The raw text columns of a `pats` row, before timestamp parsing.
struct PatRow {
    id: String,
    user_id: String,
    prefix: String,
    name: String,
    expires_at: Option<String>,
    revoked_at: Option<String>,
    last_used_at: Option<String>,
    created_at: String,
}

impl PatRow {
    fn into_pat(self) -> Result<Pat, IdentityError> {
        Ok(Pat {
            id: self.id,
            user_id: self.user_id,
            prefix: self.prefix,
            name: self.name,
            expires_at: parse_opt_ts(self.expires_at)?,
            revoked_at: parse_opt_ts(self.revoked_at)?,
            last_used_at: parse_opt_ts(self.last_used_at)?,
            created_at: parse_ts(&self.created_at)?,
        })
    }
}

fn map_pat_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PatRow> {
    Ok(PatRow {
        id: r.get(0)?,
        user_id: r.get(1)?,
        prefix: r.get(2)?,
        name: r.get(3)?,
        expires_at: r.get(4)?,
        revoked_at: r.get(5)?,
        last_used_at: r.get(6)?,
        created_at: r.get(7)?,
    })
}

/// The invitation's granted project keys.
fn invitation_memberships(conn: &Connection, invitation_id: &str) -> Result<Vec<String>, IdentityError> {
    let mut stmt = conn
        .prepare("SELECT project_key FROM invitation_memberships WHERE invitation_id = ?1 ORDER BY project_key")
        .map_err(backend)?;
    let rows = stmt
        .query_map(params![invitation_id], |r| r.get::<_, String>(0))
        .map_err(backend)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(backend)
}

// --- gate helpers ---

fn table_exists(conn: &Connection, name: &str) -> Result<bool, IdentityError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |r| r.get(0),
        )
        .optional()
        .map_err(open_err)?;
    Ok(found.is_some())
}

fn has_any_user_table(conn: &Connection) -> Result<bool, IdentityError> {
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )
        .map_err(open_err)?;
    Ok(count > 0)
}

fn meta_value(conn: &Connection, key: &str) -> Result<Option<String>, IdentityError> {
    conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0))
        .optional()
        .map_err(open_err)
}

// --- timestamp + id + error helpers ---

/// Fixed-width RFC3339 (microseconds, `Z`) so lexicographic string comparison
/// in SQL matches chronological order.
fn ts(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn now() -> String {
    ts(Utc::now())
}

fn parse_ts(text: &str) -> Result<DateTime<Utc>, IdentityError> {
    DateTime::parse_from_rfc3339(text)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| IdentityError::Backend(format!("unparseable timestamp {text:?}: {e}")))
}

fn parse_opt_ts(text: Option<String>) -> Result<Option<DateTime<Utc>>, IdentityError> {
    match text {
        Some(t) => Ok(Some(parse_ts(&t)?)),
        None => Ok(None),
    }
}

/// A random, collision-resistant id with a type prefix.
fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", random_token())
}

fn open_err(e: rusqlite::Error) -> IdentityError {
    IdentityError::Open(e.to_string())
}

fn backend(e: rusqlite::Error) -> IdentityError {
    IdentityError::Backend(e.to_string())
}
