# change-store Specification

## Purpose

TBD - created by syncing change 'add-change-and-artifact-io'. Update Purpose after archive.

## Requirements

### Requirement: State.db schema MUST be upgraded to version 2 with a `change` table

The `state.db` SQLite database SHALL be migrated forward from schema version 1 to version 2 by adding exactly one new table named `change`. The migration runner SHALL record the new version in `_migrations` and SHALL be idempotent on retry.

#### Scenario: First-time migration from v1 to v2

- **WHEN** the engine opens a `state.db` whose `_migrations` table contains only version 1
- **THEN** the engine SHALL execute migration v2, create the `change` table with the exact columns specified below, insert a row `(version=2, applied_at=<now>)` into `_migrations`, and SHALL NOT alter the `project` table

##### Example: v2 schema

The `change` table SHALL have exactly the following columns:

| Column      | Type      | Constraint                          | Notes                                                |
| ----------- | --------- | ----------------------------------- | ---------------------------------------------------- |
| change_id   | TEXT      | PRIMARY KEY                         | UUID v4                                              |
| name        | TEXT      | NOT NULL UNIQUE                     | kebab-case identifier                                |
| state       | TEXT      | NOT NULL                            | slice A always writes the literal `proposing`        |
| schema_id   | TEXT      | NOT NULL                            | resolved active schema id at creation time           |
| version     | INTEGER   | NOT NULL DEFAULT 1                  | monotonic Etag counter for row-level concurrency     |
| created_at  | TIMESTAMP | NOT NULL                            | ISO 8601 UTC                                         |
| updated_at  | TIMESTAMP | NOT NULL                            | ISO 8601 UTC                                         |

#### Scenario: Migration is idempotent on retry

- **WHEN** the engine opens a `state.db` whose `_migrations` table already contains version 2
- **THEN** the engine SHALL detect the existing v2 row and SHALL NOT re-create the `change` table or alter any existing data

#### Scenario: Migration leaves no partial state on failure

- **WHEN** an injected failure aborts migration v2 mid-execution
- **THEN** the partial transaction SHALL be rolled back and the next retry SHALL succeed with the same end state as a first-time migration


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink new change` SHALL create a change row and scaffold its directory

The CLI command `speclink new change <name>` SHALL allocate a fresh UUID v4 as `change_id`, insert a new row into the `change` table with `state='proposing'` and `version=1`, and SHALL create the directory `.speclink/changes/<name>/` on the filesystem. The operation SHALL be transactional: if either the database insert or the directory creation fails, neither SHALL persist.

#### Scenario: Successful change creation

- **WHEN** the user runs `speclink new change billing-system` in an initialized project
- **THEN** the CLI SHALL exit with code 0, the `change` table SHALL contain a row where `name='billing-system'`, `state='proposing'`, `version=1`, and the directory `.speclink/changes/billing-system/` SHALL exist as an empty directory

##### Example: success JSON envelope

```json
{
  "ok": true,
  "data": {
    "changeId": "550e8400-e29b-41d4-a716-446655440000",
    "name": "billing-system",
    "state": "proposing",
    "version": 1,
    "schemaId": "spec-driven",
    "artifactDir": ".speclink/changes/billing-system",
    "createdAt": "2026-05-22T10:30:00Z"
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

#### Scenario: Duplicate change name rejected

- **WHEN** the user runs `speclink new change billing-system` and a row with `name='billing-system'` already exists in the `change` table
- **THEN** the CLI SHALL exit with code 7, emit error code `change.duplicate_name`, and SHALL NOT modify the database or filesystem

#### Scenario: Invalid change name rejected

- **WHEN** the user runs `speclink new change <name>` where `<name>` does not match the kebab-case grammar defined below
- **THEN** the CLI SHALL exit with code 2, emit error code `change.invalid_name`, and SHALL NOT modify the database or filesystem

##### Example: name validation table

| Input                  | Valid | Reason                                          |
| ---------------------- | ----- | ----------------------------------------------- |
| `billing-system`       | yes   | lowercase letters and hyphens only              |
| `add-2fa`              | yes   | digits permitted in non-leading positions       |
| `BillingSystem`        | no    | uppercase letters not permitted                 |
| `billing_system`       | no    | underscores not permitted                       |
| `-billing`             | no    | leading hyphen not permitted                    |
| `billing-`             | no    | trailing hyphen not permitted                   |
| `billing--system`      | no    | consecutive hyphens not permitted               |
| `2fa-feature`          | no    | leading digit not permitted                     |
| (empty string)         | no    | minimum length is 1 byte                        |
| 65-byte string         | no    | maximum length is 64 bytes (UTF-8 byte count)   |


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: Change name grammar SHALL match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with byte length 1–64

The CLI SHALL validate change names against the regular expression `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` and SHALL reject names whose UTF-8 byte length is 0 or exceeds 64 bytes.

#### Scenario: Boundary length names

- **WHEN** the user supplies a change name with exactly 1 byte that matches the grammar
- **THEN** the CLI SHALL accept it

- **WHEN** the user supplies a change name with exactly 64 bytes that matches the grammar
- **THEN** the CLI SHALL accept it

- **WHEN** the user supplies a change name with exactly 65 bytes
- **THEN** the CLI SHALL reject it with `change.invalid_name`


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink list --changes` SHALL list all changes from state.db

The CLI command `speclink list --changes` SHALL read all rows from the `change` table and emit them in the success envelope sorted by `updated_at` descending. The command SHALL NOT touch the filesystem.

#### Scenario: Empty change table

- **WHEN** the user runs `speclink list --changes` and no rows exist in the `change` table
- **THEN** the CLI SHALL exit with code 0 and emit `data.changes` as an empty array

#### Scenario: Multiple changes ordered by updated_at descending

- **WHEN** the `change` table contains three rows with distinct `updated_at` values
- **THEN** the CLI SHALL emit them in `data.changes` from newest `updated_at` to oldest

##### Example: success JSON envelope

```json
{
  "ok": true,
  "data": {
    "changes": [
      {
        "changeId": "550e8400-e29b-41d4-a716-446655440000",
        "name": "billing-system",
        "state": "proposing",
        "version": 1,
        "schemaId": "spec-driven",
        "createdAt": "2026-05-22T10:30:00Z",
        "updatedAt": "2026-05-22T10:30:00Z"
      }
    ]
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink show change <name>` SHALL emit change metadata and existing artifact filenames

The CLI command `speclink show change <name>` SHALL look up the row in the `change` table by `name`, SHALL list filenames under `.speclink/changes/<name>/` (one directory level only, plus a single recursion into `specs/`), and SHALL emit both pieces of data in the success envelope.

#### Scenario: Existing change with artifacts

- **WHEN** the user runs `speclink show change billing-system` and the change row exists with files `proposal.md`, `design.md`, and `specs/user-auth/spec.md` on the filesystem
- **THEN** the CLI SHALL exit with code 0 and emit `data.artifacts` as `[{kind:"proposal"}, {kind:"design"}, {kind:"spec", capability:"user-auth"}]` and `data.change` containing the full row

#### Scenario: Existing change with no artifacts

- **WHEN** the user runs `speclink show change billing-system` and the change row exists but `.speclink/changes/billing-system/` is empty
- **THEN** the CLI SHALL exit with code 0 and emit `data.artifacts` as an empty array

#### Scenario: Non-existent change name

- **WHEN** the user runs `speclink show change unknown-name` and no row with that name exists in the `change` table
- **THEN** the CLI SHALL exit with code 2 and emit error code `change.not_found`


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink delete change <name>` SHALL be destructive and require explicit confirmation

The CLI command `speclink delete change <name> --confirm-name <name>` SHALL remove the row from the `change` table and SHALL remove the directory `.speclink/changes/<name>/` recursively. The command SHALL refuse to run without `--confirm-name` matching `<name>` exactly. The command SHALL be transactional: if either the database delete or the directory removal fails, neither SHALL persist.

#### Scenario: Successful delete with confirmation

- **WHEN** the user runs `speclink delete change billing-system --confirm-name billing-system` and the change exists
- **THEN** the CLI SHALL exit with code 0, the `change` table SHALL NOT contain a row with `name='billing-system'`, and the directory `.speclink/changes/billing-system/` SHALL NOT exist

#### Scenario: Missing confirmation flag rejected

- **WHEN** the user runs `speclink delete change billing-system` without `--confirm-name`
- **THEN** the CLI SHALL exit with code 2, emit error code `change.invalid_name` with a hint to supply `--confirm-name`, and SHALL NOT modify the database or filesystem

#### Scenario: Mismatched confirmation rejected

- **WHEN** the user runs `speclink delete change billing-system --confirm-name wrong-name`
- **THEN** the CLI SHALL exit with code 2, emit error code `change.invalid_name`, and SHALL NOT modify the database or filesystem

#### Scenario: Deleting non-existent change

- **WHEN** the user runs `speclink delete change unknown --confirm-name unknown` and no row exists
- **THEN** the CLI SHALL exit with code 2 and emit error code `change.not_found`


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: Change state in slice A SHALL be the literal `proposing`

In slice A every newly created change SHALL have `state='proposing'`. The CLI SHALL NOT expose any command that transitions `state` to any other value. The 6-state lifecycle SHALL be introduced by a subsequent change.

#### Scenario: New change writes `proposing`

- **WHEN** the user creates any change in slice A
- **THEN** the corresponding row SHALL have `state='proposing'`

#### Scenario: No transition CLI exists

- **WHEN** the user runs `speclink --help`
- **THEN** the help output SHALL NOT advertise any subcommand that mutates `change.state`


<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: Change row Etag (the `version` column) SHALL start at 1 on creation

Every new change row SHALL be inserted with `version=1`. Slice A SHALL NOT mutate the `change` row after creation; the column exists to support row-level optimistic concurrency for subsequent slices.

#### Scenario: Initial version is 1

- **WHEN** a new change is created
- **THEN** the row's `version` column SHALL equal 1

<!-- @trace
source: add-change-and-artifact-io
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/lib.rs
  - crates/provider-local/src/store.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/git.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/cli/src/commands/new_artifact.rs
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/paths.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/provider-local/Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - doc/protocol/operations.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/main.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/error.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/show_change.rs
  - Cargo.toml
  - crates/provider/src/types.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - crates/cli/src/commands/init.rs
  - doc/speclink-design.md
  - crates/cli/src/human.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/cli/src/commands/link.rs
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/cli/src/commands/artifact_read.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider-local/src/paths.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider/Cargo.toml
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
tests:
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/cli/tests/change_crud.rs
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/bootstrap.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/runtime/tests/ops.rs
-->