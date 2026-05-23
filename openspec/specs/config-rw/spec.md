# config-rw Specification

## Purpose

TBD - created by archiving change 'add-config-rw'. Update Purpose after archive.

## Requirements

### Requirement: `speclink config show` SHALL read config.yaml and return `Versioned<Config>`

The system SHALL provide a `speclink config show [--key <jsonpath>] [--json]` command that reads `.speclink/config.yaml`, parses it as YAML, and returns the parsed value alongside an etag in the form `v<version>.<sha256[:12]>`. When `--key` is provided, the response SHALL return only the leaf value at the JSONPath subset address; the etag SHALL still reflect the whole-file version. The command SHALL exit with status 0 on success.

The JSONPath subset grammar SHALL be `segment ( '.' segment | '[' index ']' )*` where `segment = [a-zA-Z_][a-zA-Z0-9_-]*` and `index = [0-9]+`. Wildcards, filters, and recursive descent SHALL be rejected.

#### Scenario: Read full config

- **WHEN** the user runs `speclink config show --json` on a freshly initialized project
- **THEN** the command SHALL exit 0 and the JSON envelope `data` SHALL contain `value` (parsed Config object with at least `rules.require_artifact_review` and `rules.require_code_review`) and `etag` matching `^v[0-9]+\.[0-9a-f]{12}$`

#### Scenario: Read leaf via --key

- **WHEN** the user runs `speclink config show --key rules.require_code_review --json`
- **THEN** the command SHALL exit 0 and `data` SHALL contain `key: "rules.require_code_review"`, `value: false` (or whatever the current value is), and `etag` reflecting whole-file version

##### Example: --key data shape

| Input | Output `data` |
|---|---|
| `--key rules.require_code_review` | `{ key: "rules.require_code_review", value: false, etag: "v1.abc123def456" }` |
| `--key project.id` | `{ key: "project.id", value: "<uuid>", etag: "v1.abc123def456" }` |

#### Scenario: Reject unsupported JSONPath syntax

- **WHEN** the user runs `speclink config show --key 'rules.*' --json`
- **THEN** the command SHALL exit 2 and the error code SHALL be `config.key_not_found` with a hint that wildcards are unsupported


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: `speclink config set <key> <value>` SHALL patch config.yaml with optimistic concurrency

The command `speclink config set <key> <value> [--expected-etag <etag>] [--json]` SHALL apply a single-key patch to `.speclink/config.yaml` using the JSONPath subset address from the `<key>` argument. The `<value>` argument SHALL be parsed in this order: `true` / `false` / `null` literal, then integer, then float, then string raw. JSON literals (arrays, objects) SHALL NOT be parsed; users requiring complex values SHALL use `config edit`.

When `--expected-etag` is provided and does not match the current etag, the command SHALL exit with status 7 and emit error code `state.etag_mismatch`. When `--expected-etag` is omitted, the command SHALL still perform an internal CAS using the etag read at the start of the operation; this guards against concurrent writers without requiring user-supplied etags.

The command SHALL be atomic: file write + state.db update + audit row insert SHALL commit in a single SQLite transaction.

#### Scenario: Set a boolean leaf

- **WHEN** the user runs `speclink config set rules.require_code_review true --json`
- **THEN** the command SHALL exit 0, the file `.speclink/config.yaml` SHALL contain `require_code_review: true` under `rules`, `state.db` SHALL contain a `config_change` row with `mode='set'`, `keys_changed='["rules.require_code_review"]'`, and the JSON envelope `data` SHALL contain `value` (full updated Config), `etag` (new), and `keys_changed: ["rules.require_code_review"]`

#### Scenario: Etag mismatch rejects write

- **GIVEN** the current config etag is `v1.abc123def456`
- **WHEN** the user runs `speclink config set rules.require_code_review true --expected-etag v1.WRONGSHA12345 --json`
- **THEN** the command SHALL exit 7, the error code SHALL be `state.etag_mismatch`, the file SHALL NOT be modified, and `state.db` SHALL NOT contain a new `config_change` row

#### Scenario: Value parsing precedence

- **GIVEN** various `<value>` string inputs
- **WHEN** the user runs `speclink config set rules.<key> <value>`
- **THEN** the parsed value SHALL follow the precedence table

##### Example: Value parsing precedence table

| `<value>` argument | Parsed type | Parsed value |
|---|---|---|
| `true` | bool | `true` |
| `false` | bool | `false` |
| `null` | null | `null` |
| `42` | integer | `42` |
| `1.5` | float | `1.5` |
| `abc` | string | `"abc"` |
| `"1.5"` | string | `"1.5"` (quotes preserved by shell, raw value used) |
| `[1,2,3]` | string | `"[1,2,3]"` (JSON literal not parsed) |

#### Scenario: Unknown key rejected

- **WHEN** the user runs `speclink config set rules.unknown_key true --json`
- **THEN** the command SHALL exit 2 and the error code SHALL be `config.key_not_found`


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: `speclink config edit` SHALL replace config.yaml contents via interactive editor or stdin

The command `speclink config edit [--editor <cmd>] [--stdin] [--expected-etag <etag>] [--json]` SHALL replace the entire `.speclink/config.yaml` contents.

When `--stdin` is specified, the command SHALL read the new content from standard input. When `--stdin` is omitted, the command SHALL invoke `$EDITOR` (or the `--editor`-specified command) with the current config loaded; on editor exit, the command SHALL read the saved buffer back as the new content. In non-TTY environments (skill subprocess, CI), `--stdin` SHALL be used.

The new content SHALL be validated as YAML before any write happens. Schema validation in this slice SHALL be limited to syntactic YAML parse plus type checks on the two review-flag leaves (`rules.require_artifact_review` and `rules.require_code_review` SHALL be booleans when present). Other fields SHALL be accepted as opaque values.

The `config_change` audit row SHALL record `mode='edit'` and `keys_changed='["__edit__"]'` (no deep diff in this slice).

#### Scenario: Edit via stdin

- **WHEN** the user runs `speclink config edit --stdin --json` with new YAML content on stdin
- **THEN** the command SHALL exit 0, the file `.speclink/config.yaml` SHALL contain the new content byte-for-byte (after YAML normalization), `state.db` SHALL contain a `config_change` row with `mode='edit'` and `keys_changed='["__edit__"]'`, and the JSON envelope `data` SHALL contain the new `value` and `etag`

#### Scenario: Malformed YAML rejected with config.malformed

- **WHEN** the user runs `speclink config edit --stdin --json` with YAML content that fails parsing
- **THEN** the command SHALL exit 3, the error code SHALL be `config.malformed`, the file SHALL NOT be modified, and `state.db` SHALL NOT contain a new `config_change` row

#### Scenario: Type-invalid review flag rejected

- **WHEN** the user runs `speclink config edit --stdin --json` with YAML where `rules.require_code_review` is a string instead of bool
- **THEN** the command SHALL exit 3, the error code SHALL be `config.malformed`, and the hint SHALL identify the offending key


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: Read path SHALL detect external file edits and reconcile via audit log

When `ConfigStore::read_config()` is invoked, the implementation SHALL compute the current file's sha256 and compare it with `config_state.content_sha256`. If the values differ, the implementation SHALL treat this as an external edit:

1. Open a SQLite transaction
2. UPDATE `config_state` SET `content_sha256=<new>`, `size_bytes=<new>`, `mtime_ns=<new>`, `version=version+1`, `updated_at=now`, `written_by=NULL` WHERE `id=1` AND `version=<expected>`
3. INSERT `config_change` row with `mode='external_edit'`, `keys_changed='["__external_edit__"]'`, `etag_before=<old>`, `etag_after=<new>`, `actor_json=NULL`, `reason='config_external_edit'`
4. Commit transaction
5. Return `Versioned<Config>` with the new etag and add a JSON envelope `warnings` entry of code `config.external_edit_detected`

Read path SHALL NOT raise an error on external edit; reconciliation SHALL be transparent.

#### Scenario: User vim edit produces warning and audit row

- **GIVEN** a project with `state.db config_state.version=1`, `content_sha256=A`, and config.yaml content matching sha A
- **WHEN** the user manually edits `.speclink/config.yaml` to produce sha B, then runs `speclink config show --json`
- **THEN** the command SHALL exit 0, the response `data.etag` SHALL be `v2.<sha-B[:12]>`, the response `warnings` SHALL contain `config.external_edit_detected`, and `state.db config_change` SHALL contain a new row with `mode='external_edit'`, `keys_changed='["__external_edit__"]'`, `etag_before='v1.<sha-A[:12]>'`, `etag_after='v2.<sha-B[:12]>'`

#### Scenario: Concurrent write after external edit fails CAS

- **GIVEN** the user vim-edited the file (sha drifted to B) and a stale process holds `expected_etag='v1.<sha-A[:12]>'`
- **WHEN** the stale process runs `speclink config set rules.require_code_review true --expected-etag v1.<sha-A[:12]>`
- **THEN** the command SHALL exit 7, the error code SHALL be `state.etag_mismatch`, the file SHALL NOT be modified


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: Read path SHALL fall back to defaults when config is missing or malformed

When `.speclink/config.yaml` is absent, unreadable, or fails YAML parsing, `ConfigStore::read_config()` SHALL return `Versioned<Config>` with `value` produced by `read_defaults()` and `etag="v0.malformed-fallback"`. The response SHALL include a JSON envelope `warnings` entry of code `config.malformed_using_defaults`. Read path SHALL NOT raise an error in this case.

The defaults SHALL set `rules.require_artifact_review=false` and `rules.require_code_review=false` to preserve walking-skeleton 4-state behavior.

Write paths (`config.write` mode=set / edit) SHALL NOT fall back on malformed content; they SHALL raise `config.malformed` (exit 3).

#### Scenario: Missing file returns defaults with warning

- **GIVEN** a project where `.speclink/config.yaml` has been deleted by the user
- **WHEN** any op invokes `read_config()`
- **THEN** the result SHALL contain `value.rules.require_artifact_review=false`, `value.rules.require_code_review=false`, `etag="v0.malformed-fallback"`, and `warnings` SHALL contain `config.malformed_using_defaults`

#### Scenario: Malformed YAML returns defaults with warning

- **GIVEN** a project where `.speclink/config.yaml` contains syntactically invalid YAML
- **WHEN** any op invokes `read_config()`
- **THEN** the result SHALL be identical to the missing-file case, and `state.db config_change` SHALL NOT have a new row (read path performs no write on malformed)


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: state.db SHALL be upgraded to version 5 with `config_state` and `config_change` tables

The state.db migration system SHALL include a v5 step that creates the `config_state` and `config_change` tables and inserts the `_migrations` row with `version=5`. Migration v5 SHALL be forward-only; downgrade SHALL NOT be supported (per design §12.7).

`config_state` schema:

| Column          | Type      | Constraints                                  |
| --------------- | --------- | -------------------------------------------- |
| id              | INTEGER   | PRIMARY KEY CHECK (id = 1)                   |
| content_sha256  | TEXT      | NOT NULL                                     |
| size_bytes      | INTEGER   | NOT NULL                                     |
| mtime_ns        | INTEGER   | NOT NULL                                     |
| version         | INTEGER   | NOT NULL DEFAULT 1                           |
| updated_at      | TIMESTAMP | NOT NULL                                     |
| written_by      | TEXT      | NULL                                         |

`config_change` schema:

| Column        | Type      | Constraints                                                          |
| ------------- | --------- | -------------------------------------------------------------------- |
| change_seq    | INTEGER   | PRIMARY KEY AUTOINCREMENT                                            |
| changed_at    | TIMESTAMP | NOT NULL                                                             |
| mode          | TEXT      | NOT NULL CHECK (mode IN ('set', 'edit', 'external_edit'))            |
| keys_changed  | TEXT      | NOT NULL (JSON array of string)                                      |
| etag_before   | TEXT      | NULL                                                                 |
| etag_after    | TEXT      | NOT NULL                                                             |
| actor_json    | TEXT      | NULL                                                                 |
| reason        | TEXT      | NOT NULL CHECK (reason IN ('config_write', 'config_external_edit'))  |

Migration v5 SHALL include an `INSERT OR IGNORE INTO config_state (id, content_sha256, size_bytes, mtime_ns, version, updated_at, written_by) VALUES (1, <sha-of-current-config.yaml>, <size>, <mtime>, 1, <now>, NULL)` step so that legacy v4 databases that pre-date config_rw are populated correctly during upgrade. Fresh-init projects (`speclink init` on a v5-capable binary) SHALL have the row inserted in the init transaction (see project-bootstrap delta).

#### Scenario: Migration v5 runs on a v4 database

- **GIVEN** a state.db at schema version 4 (with `.speclink/config.yaml` present)
- **WHEN** the engine opens the db
- **THEN** the migration runner SHALL apply v5, the `_migrations` table SHALL contain a row with `version=5`, the `config_state` table SHALL contain exactly one row with `id=1` and `content_sha256` matching the current config.yaml sha, and the `config_change` table SHALL exist (empty)

#### Scenario: Schema constraint rejects second row in config_state

- **WHEN** any code attempts `INSERT INTO config_state (id, ...) VALUES (2, ...)`
- **THEN** SQLite SHALL reject the insert with a CHECK constraint violation


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: `ConfigStore` trait SHALL be exposed via `Provider::config_store()`

The `Provider` trait in `crates/provider/src/lib.rs` SHALL gain a `fn config_store(&self) -> &dyn ConfigStore` method. The trait `ConfigStore` SHALL declare `read_config(&self) -> Result<Versioned<Config>, ProviderError>`, `write_config(&self, request: WriteConfigRequest) -> Result<Versioned<Config>, ProviderError>`, and `read_defaults(&self) -> Config`. `WriteConfigRequest` SHALL be an enum with variants `Set { key: JsonPath, value: ConfigValue, expected_etag: Option<Etag>, actor: Option<ActorJson> }` and `Edit { content: String, expected_etag: Option<Etag>, actor: Option<ActorJson> }`.

`LocalProvider` SHALL provide the only impl in this slice. HttpProvider impl SHALL be reserved for a future slice and SHALL NOT block A5 completion.

#### Scenario: Provider trait surface stable across crates

- **WHEN** a downstream crate compiles against `crates/provider` with `Provider` and `ConfigStore` in scope
- **THEN** `provider.config_store().read_config()` SHALL compile and return `Result<Versioned<Config>, ProviderError>`


<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->

---
### Requirement: New error codes SHALL be registered with stable exit codes

The following error codes SHALL be added to the global error registry with the listed exit codes (per design §17.4):

| Code | Exit | Hint behavior |
|---|---|---|
| `config.not_found` | 2 | Suggest running `speclink init` (only raised by write paths when file missing) |
| `config.malformed` | 3 | Cite the YAML parse error line / column when available |
| `config.key_not_found` | 2 | List the closest matching key paths in the current config (best-effort). When raised from a JSONPath subset parse failure (wildcard / filter / recursive-descent), the `key` field SHALL preserve the user's original `--key` argument literally; the diagnostic reason SHALL appear in `message`, not in `key`. |
| `config.edit_mode_required` | 2 | Tell the caller that `speclink config edit` requires `--stdin`, `--editor <cmd>`, or `$EDITOR` to be set. Raised when none of the three are supplied. |

The error code `state.etag_mismatch` (exit 7) is already registered by earlier slices and SHALL apply to `config.write` without redefinition. Its human-readable `message` SHALL format `expected` / `actual` as plain etag strings (e.g., `expected=v1.abc123def456, actual=v2.000000000000`) and SHALL NOT leak Rust `Debug` formatting wrappers (no literal `Some(...)`, no `None` keyword); when `expected` is absent the message SHALL render it as `<none>`.

Audit-only codes SHALL also be added (these surface only via warnings or audit rows, never as errors):

| Code | Surface |
|---|---|
| `config.external_edit_detected` | JSON envelope `warnings`; one row in `config_change` with `mode='external_edit'` |
| `config.malformed_using_defaults` | JSON envelope `warnings`; no `config_change` row |

#### Scenario: All six codes appear in CLI error registry

- **WHEN** the CLI binary is built with A5 + polish-config-error-messages patches
- **THEN** `speclink describe-errors --json` (or the equivalent test harness query) SHALL include `config.not_found`, `config.malformed`, `config.key_not_found`, `config.edit_mode_required`, `config.external_edit_detected`, and `config.malformed_using_defaults` in the registry output

#### Scenario: `config edit` without input mode emits `config.edit_mode_required`

- **GIVEN** an initialized project where `$EDITOR` env var is unset
- **WHEN** the user runs `speclink config edit --json` (no `--stdin`, no `--editor`)
- **THEN** the command SHALL exit 2, the JSON envelope `error.code` SHALL be `config.edit_mode_required`, and `error.message` SHALL include the literal substring `--stdin` and `$EDITOR` so the caller knows the two ways to retry

#### Scenario: JSONPath parse failure preserves the user's `--key` argument in the error envelope

- **GIVEN** an initialized project with default config
- **WHEN** the user runs `speclink config show --key 'rules.*' --json`
- **THEN** the command SHALL exit 2, `error.code` SHALL be `config.key_not_found`, `error.message` SHALL include the literal substring `rules.*` (the user's original argument), and `error.message` SHALL NOT mention `wildcards not supported` as if it were the key name

#### Scenario: `state.etag_mismatch` message does not leak Rust Debug formatting

- **GIVEN** an initialized project with current config etag `v1.<sha[:12]>`
- **WHEN** the user runs `speclink config set rules.require_code_review true --expected-etag v99.bogus0000000 --json`
- **THEN** the command SHALL exit 7, `error.code` SHALL be `state.etag_mismatch`, `error.message` SHALL contain `v99.bogus0000000` and the current actual etag as plain strings, and SHALL NOT contain the literal substring `Some(` nor end with `)` as a Debug wrapper

<!-- @trace
source: polish-config-error-messages
updated: 2026-05-23
code:
  - crates/provider-local/src/config_store.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/src/change_store.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/Cargo.toml
  - crates/provider-local/src/store.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/src/jsonpath.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/ops.rs
  - crates/runtime/src/state_machine.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/types.rs
  - crates/provider-local/src/archive_store.rs
  - crates/cli/src/commands/config.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/state_machine_store.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/task_ops.rs
  - doc/speclink-design.md
  - crates/provider/src/config_store.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/Cargo.toml
  - crates/provider-local/src/artifact_store.rs
  - doc/protocol/operations.md
tests:
  - crates/cli/tests/init_config_state.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/config_store.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/config_cli.rs
-->