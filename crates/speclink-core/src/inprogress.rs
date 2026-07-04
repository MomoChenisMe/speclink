//! In-progress change markers — stored in `.git/speclink-app/speclink.db` (SQLite), the same
//! storage approach Spectra uses (`.git/spectra-app/spectra.db`). The database is created lazily
//! by the first `in-progress add`; Spectra creates the `.git` directory itself when the project
//! is not a git repository, and so do we.

use crate::workspace::Workspace;
use anyhow::Result;
use std::path::PathBuf;

/// Exact bootstrap DDL Spectra's CLI uses (verbatim, including whitespace, so the schema text
/// stored in sqlite_master matches). `parked_changes` exists for schema compatibility only —
/// the park feature itself is removed from speclink.
const BOOTSTRAP_DDL: &str = "CREATE TABLE parked_changes (
            change_id TEXT PRIMARY KEY,
            original_modified INTEGER,
            tasks_total INTEGER DEFAULT 0,
            tasks_done INTEGER DEFAULT 0,
            has_proposal INTEGER DEFAULT 0,
            has_tasks INTEGER DEFAULT 0,
            created_by TEXT,
            created_with TEXT
        );
CREATE TABLE in_progress_change (
            change_id TEXT PRIMARY KEY
        );";

fn app_dir(ws: &Workspace) -> PathBuf {
    ws.root.join(".git").join("speclink-app")
}

/// Open (creating on first use) the app database, mirroring Spectra's bootstrap:
/// `.migrate.lock` marker + minimal two-table schema + one-time legacy migration.
fn open_db(ws: &Workspace) -> Result<rusqlite::Connection> {
    let dir = app_dir(ws);
    std::fs::create_dir_all(&dir)?;
    let lock = dir.join(".migrate.lock");
    if !lock.exists() {
        std::fs::write(&lock, b"")?;
    }
    let conn = rusqlite::Connection::open(dir.join("speclink.db"))?;
    let have: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='in_progress_change'",
        [],
        |r| r.get(0),
    )?;
    if have == 0 {
        conn.execute_batch(BOOTSTRAP_DDL)?;
    }
    migrate_legacy(ws, &conn)?;
    Ok(conn)
}

/// One-time import of the legacy `.speclink/in_progress.json` (the pre-SQLite storage), stamped
/// with a `.migrated` marker like Spectra's migration pass.
fn migrate_legacy(ws: &Workspace, conn: &rusqlite::Connection) -> Result<()> {
    let done = app_dir(ws).join(".migrated");
    if done.exists() {
        return Ok(());
    }
    let legacy = ws.work_dir().join("in_progress.json");
    if let Some(text) = crate::util::read_opt(&legacy) {
        #[derive(serde::Deserialize, Default)]
        struct Legacy {
            #[serde(default)]
            changes: Vec<String>,
        }
        let parsed: Legacy = serde_json::from_str(&text).unwrap_or_default();
        for name in parsed.changes {
            conn.execute(
                "INSERT OR IGNORE INTO in_progress_change (change_id) VALUES (?1)",
                [&name],
            )?;
        }
        let _ = std::fs::remove_file(&legacy);
        // The marker is only stamped when a migration actually ran — a fresh project's
        // app dir holds just .migrate.lock + the db, exactly like Spectra's.
        std::fs::write(&done, b"")?;
    }
    Ok(())
}

/// Mark a change as in-progress. Silent and idempotent; the name is not validated against
/// existing changes and the marker survives archive — all matching Spectra.
pub fn add(ws: &Workspace, name: &str) -> Result<()> {
    let conn = open_db(ws)?;
    conn.execute(
        "INSERT OR IGNORE INTO in_progress_change (change_id) VALUES (?1)",
        [name],
    )?;
    Ok(())
}
