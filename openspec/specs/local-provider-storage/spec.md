### Requirement: Local provider directory layout

The local filesystem provider SHALL store all state under a `.speclink/` directory rooted at the project base (the directory containing the project config, or the current working directory when no project config exists). The layout SHALL be:

```
.speclink/
  config.toml              # project-level config (optional, created by CLI when missing)
  state.db                 # SQLite runtime state
  changes/
    <change-id>/
      proposal.md          # the change proposal artifact
      metadata.json        # lifecycle and actor metadata
```

Subdirectories (`design.md`, `tasks.md`, `specs/`, `archive/`, `packs/`, `cache/`) MAY exist when populated by future capabilities; this change SHALL NOT create them eagerly.

All file paths SHALL be constructed with `std::path::PathBuf` join operations. The CLI SHALL NOT contain hard-coded `/` or `\` path separators in source code.

#### Scenario: First-time `propose create` initializes the directory

- **GIVEN** a CWD with no `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test"`
- **THEN** the directory `.speclink/changes/demo/` exists
- **AND** the file `.speclink/changes/demo/proposal.md` exists
- **AND** the file `.speclink/changes/demo/metadata.json` exists
- **AND** the file `.speclink/state.db` exists
- **AND** no other files are created under `.speclink/` by this command

#### Scenario: Cross-platform path separator handling

- **GIVEN** a CWD on Windows `C:\Users\user\proj`
- **WHEN** the local provider writes `.speclink/changes/demo/proposal.md`
- **THEN** the file is created at `C:\Users\user\proj\.speclink\changes\demo\proposal.md`
- **AND** the `--json` data payload's `artifactPath` field uses forward slashes: `.speclink/changes/demo/proposal.md`


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: SQLite state database schema version 1

The local provider SHALL maintain a SQLite database at `.speclink/state.db`. The schema SHALL be version 1 in this change, identified by `PRAGMA user_version = 1`. The schema SHALL contain exactly the following table:

```sql
CREATE TABLE IF NOT EXISTS in_progress_change (
    change_id  TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
```

`change_id` SHALL be the kebab-case change identifier. `created_at` SHALL be an ISO 8601 timestamp in UTC with second precision (e.g., `2026-05-19T12:34:56Z`).

On every connection open, the provider SHALL:

1. Read `PRAGMA user_version`
2. If the value is 0 (new database), apply the schema above and set `PRAGMA user_version = 1`
3. If the value is greater than the CLI's expected version, return an error with `error.code = "internal.error"` indicating an incompatible state database
4. If the value is less than the CLI's expected version, run forward migrations in order (no migrations defined in this change)

#### Scenario: New database initialized at version 1

- **GIVEN** no `.speclink/state.db` exists
- **WHEN** the local provider opens a connection
- **THEN** the file `.speclink/state.db` is created
- **AND** `PRAGMA user_version` returns 1
- **AND** the `in_progress_change` table exists with the schema above

#### Scenario: Existing version-1 database is reused

- **GIVEN** `.speclink/state.db` exists with `PRAGMA user_version = 1` and one row in `in_progress_change`
- **WHEN** the local provider opens a connection
- **THEN** the file is not modified
- **AND** the existing row remains in `in_progress_change`

#### Scenario: Future-version database produces error

- **GIVEN** `.speclink/state.db` exists with `PRAGMA user_version = 2`
- **AND** the CLI expects version 1
- **WHEN** the local provider opens a connection
- **THEN** the process exit code is 1
- **AND** the `--json` output contains a failure envelope with `error.code = "internal.error"`
- **AND** `error.message` indicates the state database version is newer than supported


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Atomic artifact write with metadata pairing

When the local provider writes an artifact (`proposal.md`, `design.md`, `tasks.md`, or any spec file), it SHALL also update the change's `metadata.json` to reflect the new lifecycle state. Both writes SHALL appear atomic from a reader's perspective: a successful command SHALL leave both files consistent; a failed command SHALL NOT leave the change directory in a state where `proposal.md` exists without `metadata.json`.

To achieve this, the provider SHALL:

1. Write artifact content and metadata to temporary sibling files (`proposal.md.tmp`, `metadata.json.tmp`)
2. Rename `metadata.json.tmp` to `metadata.json` last
3. On any write or rename failure, remove all temp files and any newly created `<change-id>/` directory

#### Scenario: Successful write leaves consistent state

- **WHEN** `propose create` succeeds
- **THEN** `proposal.md` and `metadata.json` both exist
- **AND** no `.tmp` files remain in `.speclink/changes/<change-id>/`

#### Scenario: Filesystem error during write triggers cleanup

- **GIVEN** the disk runs out of space mid-write
- **WHEN** the local provider attempts to write `proposal.md`
- **THEN** the process exit code is 1
- **AND** the `--json` output contains a failure envelope with `error.code = "internal.error"`
- **AND** the directory `.speclink/changes/<change-id>/` does not exist (cleanup removed it)
- **AND** no row was inserted into the `in_progress_change` table


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: in_progress_change semantics

The `in_progress_change` table SHALL hold at most one row representing the change the user is currently working on. The `propose create` command SHALL insert a row for the newly created change using `INSERT OR REPLACE`, overwriting any previous in-progress entry.

Future commands (`park`, `unpark`, `finish`, `archive` — not implemented in this change) SHALL be expected to manage this row, but this change SHALL NOT implement them.

#### Scenario: `propose create` replaces previous in-progress entry

- **GIVEN** the `in_progress_change` table contains a row with `change_id = "old-change"`
- **WHEN** the user runs `speclink propose create --change new-change --summary "..."`
- **THEN** after success, the table contains exactly one row with `change_id = "new-change"`


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Change-id validation

The local provider SHALL accept change ids matching the regular expression `^[a-z][a-z0-9-]{0,63}$`. The identifier SHALL:

- Start with a lowercase ASCII letter
- Contain only lowercase ASCII letters, digits, and hyphens
- Be no longer than 64 characters
- Not contain two consecutive hyphens
- Not end with a hyphen

Any change id failing these rules SHALL cause the provider to return an error with `error.code = "change.invalid_id"`.

#### Scenario: Uppercase letter rejected

- **WHEN** the user runs `speclink propose create --change Add-Feature --summary "..."`
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "change.invalid_id"`

#### Scenario: Valid kebab-case accepted

- **WHEN** the user runs `speclink propose create --change add-order-export --summary "..."`
- **THEN** the change id is accepted
- **AND** the directory `.speclink/changes/add-order-export/` is created

##### Example: change-id validation table

| Input              | Valid | Reason                         |
| ------------------ | ----- | ------------------------------ |
| `add-order-export` | yes   | conforms to pattern             |
| `a`                | yes   | minimum length 1                |
| `Add-Feature`      | no    | uppercase letter               |
| `1add`             | no    | starts with digit              |
| `add--feature`     | no    | consecutive hyphens            |
| `add-`             | no    | trailing hyphen                |
| `(empty)`          | no    | empty string                   |


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Concurrent access safety

The local provider's SQLite connection SHALL be opened with WAL journaling mode (`PRAGMA journal_mode = WAL`) to support concurrent reads alongside writes. The provider SHALL NOT serialize all reads through a global mutex; instead, the SQLite connection pool itself provides the necessary serialization for writes while allowing concurrent reads.

This change uses a single connection wrapped in a `tokio::sync::Mutex` for simplicity (only one writer at a time, which is the CLI process itself). Future changes MAY upgrade to a true connection pool when concurrent read latency matters.

#### Scenario: Two CLI invocations against the same project

- **GIVEN** two `speclink propose create` invocations run sequentially in the same project
- **WHEN** both complete
- **THEN** both changes are recorded under `.speclink/changes/`
- **AND** the `in_progress_change` table contains exactly the change id from the second invocation (per `INSERT OR REPLACE` semantics)
- **AND** the SQLite database file is not corrupted

## Requirements


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Local provider directory layout

The local filesystem provider SHALL store all state under a `.speclink/` directory rooted at the project base (the directory containing the project config, or the current working directory when no project config exists). The layout SHALL be:

```
.speclink/
  config.toml              # project-level config (optional, created by CLI when missing)
  state.db                 # SQLite runtime state
  changes/
    <change-id>/
      proposal.md          # the change proposal artifact
      metadata.json        # lifecycle and actor metadata
```

Subdirectories (`design.md`, `tasks.md`, `specs/`, `archive/`, `packs/`, `cache/`) MAY exist when populated by future capabilities; this change SHALL NOT create them eagerly.

All file paths SHALL be constructed with `std::path::PathBuf` join operations. The CLI SHALL NOT contain hard-coded `/` or `\` path separators in source code.

#### Scenario: First-time `propose create` initializes the directory

- **GIVEN** a CWD with no `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test"`
- **THEN** the directory `.speclink/changes/demo/` exists
- **AND** the file `.speclink/changes/demo/proposal.md` exists
- **AND** the file `.speclink/changes/demo/metadata.json` exists
- **AND** the file `.speclink/state.db` exists
- **AND** no other files are created under `.speclink/` by this command

#### Scenario: Cross-platform path separator handling

- **GIVEN** a CWD on Windows `C:\Users\user\proj`
- **WHEN** the local provider writes `.speclink/changes/demo/proposal.md`
- **THEN** the file is created at `C:\Users\user\proj\.speclink\changes\demo\proposal.md`
- **AND** the `--json` data payload's `artifactPath` field uses forward slashes: `.speclink/changes/demo/proposal.md`

---
### Requirement: SQLite state database schema version 1

The local provider SHALL maintain a SQLite database at `.speclink/state.db`. The schema SHALL be version 1 in this change, identified by `PRAGMA user_version = 1`. The schema SHALL contain exactly the following table:

```sql
CREATE TABLE IF NOT EXISTS in_progress_change (
    change_id  TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
```

`change_id` SHALL be the kebab-case change identifier. `created_at` SHALL be an ISO 8601 timestamp in UTC with second precision (e.g., `2026-05-19T12:34:56Z`).

On every connection open, the provider SHALL:

1. Read `PRAGMA user_version`
2. If the value is 0 (new database), apply the schema above and set `PRAGMA user_version = 1`
3. If the value is greater than the CLI's expected version, return an error with `error.code = "internal.error"` indicating an incompatible state database
4. If the value is less than the CLI's expected version, run forward migrations in order (no migrations defined in this change)

#### Scenario: New database initialized at version 1

- **GIVEN** no `.speclink/state.db` exists
- **WHEN** the local provider opens a connection
- **THEN** the file `.speclink/state.db` is created
- **AND** `PRAGMA user_version` returns 1
- **AND** the `in_progress_change` table exists with the schema above

#### Scenario: Existing version-1 database is reused

- **GIVEN** `.speclink/state.db` exists with `PRAGMA user_version = 1` and one row in `in_progress_change`
- **WHEN** the local provider opens a connection
- **THEN** the file is not modified
- **AND** the existing row remains in `in_progress_change`

#### Scenario: Future-version database produces error

- **GIVEN** `.speclink/state.db` exists with `PRAGMA user_version = 2`
- **AND** the CLI expects version 1
- **WHEN** the local provider opens a connection
- **THEN** the process exit code is 1
- **AND** the `--json` output contains a failure envelope with `error.code = "internal.error"`
- **AND** `error.message` indicates the state database version is newer than supported

---
### Requirement: Atomic artifact write with metadata pairing

When the local provider writes an artifact (`proposal.md`, `design.md`, `tasks.md`, or any spec file), it SHALL also update the change's `metadata.json` to reflect the new lifecycle state. Both writes SHALL appear atomic from a reader's perspective: a successful command SHALL leave both files consistent; a failed command SHALL NOT leave the change directory in a state where `proposal.md` exists without `metadata.json`.

To achieve this, the provider SHALL:

1. Write artifact content and metadata to temporary sibling files (`proposal.md.tmp`, `metadata.json.tmp`)
2. Rename `metadata.json.tmp` to `metadata.json` last
3. On any write or rename failure, remove all temp files and any newly created `<change-id>/` directory

#### Scenario: Successful write leaves consistent state

- **WHEN** `propose create` succeeds
- **THEN** `proposal.md` and `metadata.json` both exist
- **AND** no `.tmp` files remain in `.speclink/changes/<change-id>/`

#### Scenario: Filesystem error during write triggers cleanup

- **GIVEN** the disk runs out of space mid-write
- **WHEN** the local provider attempts to write `proposal.md`
- **THEN** the process exit code is 1
- **AND** the `--json` output contains a failure envelope with `error.code = "internal.error"`
- **AND** the directory `.speclink/changes/<change-id>/` does not exist (cleanup removed it)
- **AND** no row was inserted into the `in_progress_change` table

---
### Requirement: in_progress_change semantics

The `in_progress_change` table SHALL hold at most one row representing the change the user is currently working on. The `propose create` command SHALL insert a row for the newly created change using `INSERT OR REPLACE`, overwriting any previous in-progress entry.

Future commands (`park`, `unpark`, `finish`, `archive` — not implemented in this change) SHALL be expected to manage this row, but this change SHALL NOT implement them.

#### Scenario: `propose create` replaces previous in-progress entry

- **GIVEN** the `in_progress_change` table contains a row with `change_id = "old-change"`
- **WHEN** the user runs `speclink propose create --change new-change --summary "..."`
- **THEN** after success, the table contains exactly one row with `change_id = "new-change"`

---
### Requirement: Change-id validation

The local provider SHALL accept change ids matching the regular expression `^[a-z][a-z0-9-]{0,63}$`. The identifier SHALL:

- Start with a lowercase ASCII letter
- Contain only lowercase ASCII letters, digits, and hyphens
- Be no longer than 64 characters
- Not contain two consecutive hyphens
- Not end with a hyphen

Any change id failing these rules SHALL cause the provider to return an error with `error.code = "change.invalid_id"`.

#### Scenario: Uppercase letter rejected

- **WHEN** the user runs `speclink propose create --change Add-Feature --summary "..."`
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "change.invalid_id"`

#### Scenario: Valid kebab-case accepted

- **WHEN** the user runs `speclink propose create --change add-order-export --summary "..."`
- **THEN** the change id is accepted
- **AND** the directory `.speclink/changes/add-order-export/` is created

##### Example: change-id validation table

| Input              | Valid | Reason                         |
| ------------------ | ----- | ------------------------------ |
| `add-order-export` | yes   | conforms to pattern             |
| `a`                | yes   | minimum length 1                |
| `Add-Feature`      | no    | uppercase letter               |
| `1add`             | no    | starts with digit              |
| `add--feature`     | no    | consecutive hyphens            |
| `add-`             | no    | trailing hyphen                |
| `(empty)`          | no    | empty string                   |

---
### Requirement: Concurrent access safety

The local provider's SQLite connection SHALL be opened with WAL journaling mode (`PRAGMA journal_mode = WAL`) to support concurrent reads alongside writes. The provider SHALL NOT serialize all reads through a global mutex; instead, the SQLite connection pool itself provides the necessary serialization for writes while allowing concurrent reads.

This change uses a single connection wrapped in a `tokio::sync::Mutex` for simplicity (only one writer at a time, which is the CLI process itself). Future changes MAY upgrade to a true connection pool when concurrent read latency matters.

#### Scenario: Two CLI invocations against the same project

- **GIVEN** two `speclink propose create` invocations run sequentially in the same project
- **WHEN** both complete
- **THEN** both changes are recorded under `.speclink/changes/`
- **AND** the `in_progress_change` table contains exactly the change id from the second invocation (per `INSERT OR REPLACE` semantics)
- **AND** the SQLite database file is not corrupted