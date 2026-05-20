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


<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
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
      proposal.md          # the change proposal artifact (required, created by `propose create`)
      design.md            # the design artifact (optional, created by `artifact write design`)
      tasks.md             # the tasks artifact (optional, created by `artifact write tasks`)
      specs/               # spec artifacts directory (optional, created when first spec is written)
        <capability>/
          spec.md          # one spec file per capability
      metadata.json        # lifecycle and actor metadata
```

Subdirectories that are not listed above (`archive/`, `packs/`, `cache/`) MAY exist when populated by future capabilities; this change SHALL NOT create them eagerly. Within `changes/<change-id>/`, only files corresponding to artifacts that have been written SHALL exist — `design.md`, `tasks.md`, and `specs/` SHALL NOT be created until the corresponding `artifact write` invocation succeeds.

All file paths SHALL be constructed with `std::path::PathBuf` join operations. The CLI SHALL NOT contain hard-coded `/` or `\` path separators in source code.

#### Scenario: First-time `propose create` initializes the directory

- **GIVEN** a CWD with no `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test"`
- **THEN** the directory `.speclink/changes/demo/` exists
- **AND** the file `.speclink/changes/demo/proposal.md` exists
- **AND** the file `.speclink/changes/demo/metadata.json` exists
- **AND** the file `.speclink/state.db` exists
- **AND** `design.md`, `tasks.md`, and `specs/` are not created by this command

#### Scenario: `artifact write` creates subdirectories on demand

- **GIVEN** a change `demo` with only `proposal.md` and `metadata.json` present
- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the file `.speclink/changes/demo/specs/auth/spec.md` exists
- **AND** the directory `.speclink/changes/demo/specs/` was created by this command
- **AND** `design.md` and `tasks.md` were not created

#### Scenario: Cross-platform path separator handling

- **GIVEN** a CWD on Windows `C:\Users\user\proj`
- **WHEN** the local provider writes `.speclink/changes/demo/design.md`
- **THEN** the file is created at `C:\Users\user\proj\.speclink\changes\demo\design.md`
- **AND** the `--json` data payload's `path` field uses forward slashes: `.speclink/changes/demo/design.md`

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

---
### Requirement: Multi-artifact atomic write

The local provider SHALL support atomic writes for four artifact kinds: `proposal`, `design`, `tasks`, and `spec`. Each write SHALL follow the same atomic write protocol established for proposal in the bootstrap change:

1. Create the target directory (and any missing parent directories such as `specs/<capability>/`) if it does not exist
2. Write artifact content to a temporary sibling file with the `.tmp` suffix (e.g., `design.md.tmp`, `specs/auth/spec.md.tmp`)
3. Rename the `.tmp` file to its final name
4. On any failure (write error, rename error), remove all created `.tmp` files; if the change directory or its `specs/<capability>/` subdirectory was newly created by this invocation, remove it as well (do not remove pre-existing files)

Unlike `propose create`, `artifact write` for `design` / `tasks` / `spec` SHALL NOT update `metadata.json`. The `metadata.json` file is the responsibility of `propose create` (initial write) and future commands (`archive`, `finish`); `artifact write` only writes the artifact file itself.

The local provider SHALL refuse to write an artifact when:
- The change directory does not exist — SHALL return `ProviderError::ChangeNotFound`
- The target artifact file already exists — SHALL return a domain error mapped to `error.code = "artifact.already_exists"` (exit code 1)

#### Scenario: Sequential multi-artifact writes succeed

- **GIVEN** a change `demo` initialized by `propose create`
- **WHEN** the user runs in sequence:
  1. `echo "design body" | speclink artifact write design --change demo --stdin --json`
  2. `echo "tasks body" | speclink artifact write tasks --change demo --stdin --json`
  3. `echo "auth spec" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** all three commands exit with code 0
- **AND** `.speclink/changes/demo/design.md`, `tasks.md`, and `specs/auth/spec.md` all exist with their respective content
- **AND** no `.tmp` files remain in the change directory

#### Scenario: Spec write fails when capability dir cannot be created

- **GIVEN** `.speclink/changes/demo/specs/` is a regular file (not a directory) due to corruption
- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "internal.error"`
- **AND** no `.tmp` files remain anywhere under `.speclink/changes/demo/`

#### Scenario: Pre-existing artifact is not overwritten

- **GIVEN** `.speclink/changes/demo/specs/auth/spec.md` already exists with content `OLD`
- **WHEN** the user runs `echo "NEW" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.already_exists"`
- **AND** the existing file content remains `OLD`


<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->

---
### Requirement: Spec capability routing

When writing a spec artifact, the local provider SHALL route the content to `<change-dir>/specs/<capability>/spec.md` where `<capability>` is the value of the `--capability` flag. The capability name SHALL match `^[a-z][a-z0-9-]{0,63}$` (same rules as change-id).

If multiple spec artifacts are written for the same change (different capabilities), each SHALL occupy its own subdirectory under `specs/`. The provider SHALL NOT impose a limit on the number of capabilities per change.

The provider SHALL NOT interpret the spec content (e.g., it SHALL NOT parse `## ADDED Requirements` / `## MODIFIED Requirements` headings). The content is treated as opaque markdown. Delta heading parsing is the responsibility of a future archive capability.

#### Scenario: Multiple spec capabilities in one change

- **GIVEN** a change `demo` initialized by `propose create`
- **WHEN** the user runs `artifact write spec` twice with capability `auth` and `billing` respectively
- **THEN** both `.speclink/changes/demo/specs/auth/spec.md` and `.speclink/changes/demo/specs/billing/spec.md` exist
- **AND** each contains its respective stdin content

#### Scenario: Capability name validation matches change-id rules

- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability Auth-Module --stdin --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.invalid_capability"`


<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->

---
### Requirement: Change status filesystem scan

The local provider SHALL implement `Provider::get_status` by scanning the filesystem under `<change-dir>/`:

1. If `<change-dir>/metadata.json` does not exist, SHALL return `ProviderError::ChangeNotFound`
2. Read and parse `metadata.json` to obtain `changeId` and `state`; parsing failure SHALL return a domain error mapped to `error.code = "internal.error"`
3. Check existence of `<change-dir>/proposal.md`, `design.md`, `tasks.md` and produce one `ArtifactStatus` entry per kind regardless of whether the file exists (`status` field reflects presence)
4. If `<change-dir>/specs/` exists, enumerate its immediate subdirectories; for each subdirectory `<capability>/` containing a `spec.md` file, produce one `ArtifactStatus` entry with `id = "spec:<capability>"` and `status = "done"`. Subdirectories without `spec.md` SHALL be ignored.
5. The returned `ChangeStatus.artifacts` array SHALL be ordered: `proposal`, `design`, `tasks`, then spec entries sorted ascending by capability name

The scan SHALL be read-only: no files SHALL be created, modified, or deleted by `get_status`.

The scan SHALL NOT recurse beyond depth 2 under `<change-dir>/` (it MUST NOT walk arbitrary subdirectories). Unknown files or subdirectories under `<change-dir>/` SHALL be ignored.

#### Scenario: Status of partially complete change

- **GIVEN** `.speclink/changes/demo/` containing only `proposal.md`, `design.md`, `metadata.json`, and `specs/auth/spec.md`
- **WHEN** the local provider's `get_status` is invoked for change `demo`
- **THEN** the returned `ChangeStatus.artifacts` contains 4 entries in this order: `proposal` (done), `design` (done), `tasks` (missing), `spec:auth` (done)
- **AND** `ChangeStatus.state` equals the state from `metadata.json` (e.g., `"proposed"`)

#### Scenario: Empty specs dir produces no spec entries

- **GIVEN** `.speclink/changes/demo/specs/` exists but contains no subdirectories
- **WHEN** `get_status` is invoked for change `demo`
- **THEN** the returned `artifacts` array contains only `proposal`, `design`, `tasks` entries
- **AND** no `spec:*` entries are present

#### Scenario: Subdirectory under specs without spec.md is ignored

- **GIVEN** `.speclink/changes/demo/specs/auth/` exists but contains no `spec.md` (e.g., a stale empty directory)
- **WHEN** `get_status` is invoked for change `demo`
- **THEN** no `spec:auth` entry appears in the returned `artifacts` array

<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->