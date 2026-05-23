//! State.db v5 migration integration tests.
//!
//! 對應 `config-rw` capability requirement「state.db SHALL be upgraded to
//! version 5 with `config_state` and `config_change` tables」與
//! `local-storage-layout` delta「`state.db` schema MUST be upgraded to version
//! 5 with the `config_state` and `config_change` tables」；同時對應 design
//! decision「Config_state singleton 表 via CHECK 約束」與「Config_change audit
//! 表設計沿 A3 state_transition 範式」。

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use speclink_provider_local::StateDb;
use tempfile::TempDir;

/// 在 tempdir 寫一份 config.yaml 範例，回傳 (path, sha256_hex, size_bytes)。
fn write_sample_config(tmp: &TempDir) -> (std::path::PathBuf, String, u64) {
    let path = tmp.path().join("config.yaml");
    let body = b"rules:\n  require_artifact_review: false\n  require_code_review: false\n";
    std::fs::write(&path, body).expect("write config.yaml");
    let sha = hex::encode(Sha256::digest(body));
    (path, sha, body.len() as u64)
}

fn open_v4(tmp: &TempDir) -> (StateDb, std::path::PathBuf) {
    let db_path = tmp.path().join("state.db");
    let db = StateDb::open(&db_path).expect("open");
    db.migrate(4).expect("migrate to v4");
    (db, db_path)
}

#[test]
fn v5_migration_writes_migrations_row_with_version_5() {
    // (a)：v4 → v5 升版後 `_migrations` 含 version=5 row。
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v4(&tmp);
    db.migrate(5).expect("migrate v5");
    assert_eq!(db.schema_version().expect("version"), 5);

    let conn = Connection::open(&db_path).expect("reopen");
    let v5_rows: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = 5",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(v5_rows, 1, "_migrations SHALL contain exactly one v5 row");
}

#[test]
fn v5_migration_creates_config_state_singleton_with_seeded_row() {
    // (b)：`config_state` 表存在、有 id=1 row、`content_sha256` 等於當前
    // config.yaml 的 sha256、`version=1`。
    //
    // 「migration runner SHALL apply v5 ... and the config_state table SHALL
    // contain exactly one row populated by the migration's INSERT OR IGNORE
    // step」(local-storage-layout delta scenario「v5 migration produces
    // additive schema」)。種子步驟由 LocalStore-level helper
    // `StateDb::seed_config_state(config_path)` 落實；本測試呼叫該 API。
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v4(&tmp);
    let (config_path, expected_sha, expected_size) = write_sample_config(&tmp);
    db.migrate(5).expect("migrate v5");
    db.seed_config_state(&config_path)
        .expect("seed config_state row");

    let conn = Connection::open(&db_path).expect("reopen");

    // Table structure: id INTEGER PK CHECK(id=1), content_sha256 / size_bytes /
    // mtime_ns / version / updated_at / written_by。
    let mut stmt = conn
        .prepare("SELECT name FROM pragma_table_info('config_state') ORDER BY cid")
        .expect("prepare pragma");
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query columns")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(
        cols,
        vec![
            "id",
            "content_sha256",
            "size_bytes",
            "mtime_ns",
            "version",
            "updated_at",
            "written_by",
        ],
        "config_state columns mismatch"
    );

    // Exactly one row, with id=1 and the seeded sha / size / version=1.
    #[allow(clippy::type_complexity)]
    let row: (i64, String, i64, i64, Option<String>) = conn
        .query_row(
            "SELECT id, content_sha256, size_bytes, version, written_by FROM config_state",
            [],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .expect("config_state row exists");
    assert_eq!(row.0, 1, "id SHALL be the singleton value 1");
    assert_eq!(
        row.1, expected_sha,
        "content_sha256 SHALL match the seeded config.yaml sha"
    );
    assert_eq!(row.2 as u64, expected_size, "size_bytes SHALL match");
    assert_eq!(row.3, 1, "version SHALL be 1 on first seed");
    assert!(row.4.is_none(), "written_by SHALL default to NULL");
}

#[test]
fn v5_migration_creates_empty_config_change_audit_table() {
    // (c)：`config_change` 表存在且為空。
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v4(&tmp);
    db.migrate(5).expect("migrate v5");

    let conn = Connection::open(&db_path).expect("reopen");

    // Table structure assertion：欄位順序 + CHECK 約束。
    let mut stmt = conn
        .prepare("SELECT name FROM pragma_table_info('config_change') ORDER BY cid")
        .expect("prepare pragma");
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query columns")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(
        cols,
        vec![
            "change_seq",
            "changed_at",
            "mode",
            "keys_changed",
            "etag_before",
            "etag_after",
            "actor_json",
            "reason",
        ],
        "config_change columns mismatch"
    );

    // Empty by default。
    let cnt: u32 = conn
        .query_row("SELECT COUNT(*) FROM config_change", [], |r| r.get(0))
        .expect("count");
    assert_eq!(cnt, 0, "config_change SHALL be empty after migration");
}

#[test]
fn config_state_check_constraint_rejects_second_row() {
    // Task 2.3：CHECK (id = 1) singleton 約束 — 任何嘗試 INSERT id != 1 都應
    // 被 SQLite 拒絕。對應 spec scenario「Schema constraint rejects second
    // row in config_state」。
    let tmp = TempDir::new().expect("tempdir");
    let (db, db_path) = open_v4(&tmp);
    let (config_path, _sha, _size) = write_sample_config(&tmp);
    db.migrate(5).expect("migrate v5");
    db.seed_config_state(&config_path).expect("seed");

    let conn = Connection::open(&db_path).expect("reopen");
    let err = conn.execute(
        "INSERT INTO config_state (id, content_sha256, size_bytes, mtime_ns, version, updated_at, written_by) \
         VALUES (2, 'deadbeef', 0, 0, 1, '2026-05-23T00:00:00Z', NULL)",
        [],
    );
    assert!(
        err.is_err(),
        "INSERT with id=2 SHALL be rejected by CHECK constraint"
    );
    let msg = format!("{}", err.unwrap_err());
    assert!(
        msg.to_lowercase().contains("check"),
        "expected CHECK constraint failure, got: {msg}"
    );

    // Existing id=1 row still intact.
    let cnt: u32 = conn
        .query_row("SELECT COUNT(*) FROM config_state", [], |r| r.get(0))
        .expect("count");
    assert_eq!(cnt, 1, "config_state SHALL still contain exactly one row");
}

#[test]
fn seed_config_state_is_idempotent_under_repeated_calls() {
    // 對齊 local-storage-layout delta scenario「Idempotent re-open after v5
    // migration」：reopen 後 _migrations / config_state 都不應重複。實作 contract
    // 為 `INSERT OR IGNORE`：同一 sha 重 seed 為 no-op；不同 sha 進「external_edit
    // detection」路徑（由 task 5.1 cover），本測試只驗證 INSERT OR IGNORE 不會
    // 把第一筆 row 改寫。
    let tmp = TempDir::new().expect("tempdir");
    let (db, _db_path) = open_v4(&tmp);
    let (config_path, sha, _size) = write_sample_config(&tmp);
    db.migrate(5).expect("migrate v5");
    db.seed_config_state(&config_path).expect("first seed");
    db.seed_config_state(&config_path).expect("re-seed no-op");

    let conn = Connection::open(tmp.path().join("state.db")).expect("reopen");
    let row_sha: String = conn
        .query_row(
            "SELECT content_sha256 FROM config_state WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .expect("read sha");
    assert_eq!(
        row_sha, sha,
        "re-seed SHALL leave the original sha untouched (INSERT OR IGNORE)"
    );
    let v: i64 = conn
        .query_row("SELECT version FROM config_state WHERE id = 1", [], |r| {
            r.get(0)
        })
        .expect("read version");
    assert_eq!(v, 1, "version SHALL remain at 1 across re-seed calls");
}
