//! The database schema of the PostgreSQL TeamStore driver: four tables and the
//! version marker that gates every open.
//!
//! The shape mirrors the SQLite driver so the two stay behaviourally
//! interchangeable. `documents` holds live documents (a delete removes the
//! row); `history` is the append-only per-document revision log; `outbox` is
//! the per-scope monotonic event log; `meta` is the key/value table carrying
//! the schema version, the store-identity marker, per-project revision
//! counters and per-scope outbox ack cursors. Project-level scalars live in
//! `meta` rather than in their own tables so the shape stays the four the
//! design fixes.
//!
//! Scope is carried as the `(project, repo)` pair leading every non-meta
//! primary key; `meta` carries it inside its composed key strings instead.
//! Outbox sequence numbers are per-scope and monotonic, allocated as
//! `MAX(seq) + 1` inside the commit transaction — the scope's advisory lock
//! makes that read-then-write safe across connections.
//!
//! The tables land in the connection's current schema, so pointing two stores
//! at different `search_path`s isolates them within one database.

/// The schema version this driver reads and writes. A database recording a
/// higher version is refused (fail closed); a lower one reports needing
/// migration. Bump this and add a migrate path when the shape changes.
pub const SCHEMA_VERSION: u32 = 1;

/// The store-identity marker written to `meta` at initialization. Its
/// presence distinguishes a speclink store from an unrelated database.
pub const STORE_MARKER: &str = "speclink-team-store";

/// `meta` key for the identity marker.
pub const META_FORMAT_KEY: &str = "format";

/// `meta` key for the schema version.
pub const META_VERSION_KEY: &str = "schema_version";

/// The `CREATE TABLE` statements for a fresh store. Idempotent
/// (`IF NOT EXISTS`) so re-running is harmless; the caller wraps this plus
/// the meta seed in one transaction so a torn init leaves nothing behind.
pub const SCHEMA_SQL: &str = "\
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
    project  TEXT NOT NULL,
    repo     TEXT NOT NULL,
    doc_id   TEXT NOT NULL,
    content  TEXT NOT NULL,
    revision BIGINT NOT NULL,
    digest   TEXT NOT NULL,
    PRIMARY KEY (project, repo, doc_id)
);
CREATE TABLE IF NOT EXISTS history (
    id       BIGSERIAL PRIMARY KEY,
    project  TEXT NOT NULL,
    repo     TEXT NOT NULL,
    doc_id   TEXT NOT NULL,
    revision BIGINT NOT NULL,
    actor    TEXT NOT NULL,
    at       TEXT NOT NULL,
    command  TEXT NOT NULL,
    kind     TEXT NOT NULL,
    digest   TEXT
);
CREATE INDEX IF NOT EXISTS history_by_doc ON history (project, repo, doc_id, id);
CREATE TABLE IF NOT EXISTS outbox (
    project  TEXT NOT NULL,
    repo     TEXT NOT NULL,
    seq      BIGINT NOT NULL,
    revision BIGINT NOT NULL,
    name     TEXT NOT NULL,
    payload  TEXT NOT NULL,
    actor    TEXT NOT NULL,
    at       TEXT NOT NULL,
    PRIMARY KEY (project, repo, seq)
);
";
