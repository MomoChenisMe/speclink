-- v5 schema：追加 config_state singleton 表（id=1 CHECK 約束）與 config_change
-- audit log 表。對齊 `config-rw` capability「state.db SHALL be upgraded to
-- version 5 with `config_state` and `config_change` tables」requirement、
-- `local-storage-layout` delta 同名 requirement，以及 design decisions
-- 「Config_state singleton 表 via CHECK 約束」、「Config_change audit 表設計沿
-- A3 state_transition 範式」。
--
-- Singleton row 由 `StateDb::seed_config_state(config_path)` helper 在 migration
-- 完成後以 INSERT OR IGNORE 形式種入；migration SQL 不執行 INSERT，因為需要
-- runtime 計算 config.yaml 的 sha256 / size / mtime（無法在純 SQL 中讀檔）。
CREATE TABLE config_state (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    content_sha256 TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL,
    mtime_ns       INTEGER NOT NULL,
    version        INTEGER NOT NULL DEFAULT 1,
    updated_at     TEXT NOT NULL,
    written_by     TEXT
);

CREATE TABLE config_change (
    change_seq    INTEGER PRIMARY KEY AUTOINCREMENT,
    changed_at    TEXT NOT NULL,
    mode          TEXT NOT NULL CHECK (mode IN ('set', 'edit', 'external_edit')),
    keys_changed  TEXT NOT NULL,
    etag_before   TEXT,
    etag_after    TEXT NOT NULL,
    actor_json    TEXT,
    reason        TEXT NOT NULL CHECK (reason IN ('config_write', 'config_external_edit'))
);
