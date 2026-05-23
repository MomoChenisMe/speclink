## ADDED Requirements

### Requirement: `speclink init` MUST insert the `config_state` singleton row in the same transaction as the project row

When `speclink init` runs against a state.db that has been migrated to schema version 5, the prepare/commit phase SHALL compute the sha256, byte size, and mtime_ns of the freshly written `.speclink/config.yaml`, then insert `config_state (id=1, content_sha256=<sha>, size_bytes=<size>, mtime_ns=<mtime>, version=1, updated_at=now, written_by=NULL)` in the **same** SQLite transaction that inserts the `project` row. The transaction SHALL be all-or-nothing: if any prepare step (including config_state computation or insert) fails, no row SHALL be left in state.db and no file SHALL remain on disk under `.speclink/`.

This requirement SHALL apply to fresh-init paths only. Legacy v4-to-v5 upgrade paths SHALL rely on the `INSERT OR IGNORE` clause inside the v5 migration step (see config-rw capability) and SHALL NOT re-run the init logic.

The existing requirement "Init MUST commit artifact and state changes only after every prepare step succeeds" continues to govern the wider all-or-nothing semantics; this requirement extends it to cover the new row.

#### Scenario: Fresh init populates config_state row

- **WHEN** a user runs `speclink init` in a fresh git repository with a v5-capable binary
- **THEN** the command SHALL exit 0, `state.db` SHALL contain exactly one `project` row and exactly one `config_state` row with `id=1`, `version=1`, `content_sha256` matching `sha256(.speclink/config.yaml bytes)`, `size_bytes` matching the file size, and `written_by=NULL`

#### Scenario: Failed init leaves no config_state row

- **GIVEN** a directory where `.speclink/` cannot be created (e.g. read-only filesystem)
- **WHEN** the user runs `speclink init`
- **THEN** the command SHALL exit non-zero, no `.speclink/` directory SHALL exist, no `project` row SHALL be present in any state.db that was opened, and no `config_state` row SHALL be present either

#### Scenario: Re-init with --force preserves config_state row alignment

- **GIVEN** an already-initialized project at schema v5 with `config_state.version=N`
- **WHEN** the user runs `speclink init --force`
- **THEN** the command SHALL preserve the existing state.db (per the existing `--force` requirement), SHALL recompute the sha256 of the rewritten config.yaml, and SHALL update `config_state` to `content_sha256=<new>`, `size_bytes=<new>`, `mtime_ns=<new>`, `version=version+1`, `updated_at=now` if and only if the new bytes differ from the old sha; if the bytes match, the row SHALL NOT be touched
