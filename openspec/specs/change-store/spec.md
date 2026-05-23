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

**Note**: Requirement heading SHALL be preserved verbatim from the slice A2 baseline for analyzer traceability; the body below supersedes the slice A2 behavior with the slice A3 6-state lifecycle contract.

The `change.state` column SHALL hold one of the six legal lifecycle values defined by the `state-machine` capability (`proposing`, `reviewing`, `ready`, `in_progress`, `code_reviewing`, `archived`). Every newly created change SHALL be inserted with `state='proposing'`. Mutation of `change.state` after creation SHALL be performed exclusively by the `state-machine` capability via the `StateMachineStore` trait; the `change-store` capability SHALL NOT expose any direct setter for `change.state`.

#### Scenario: New change writes `proposing`

- **WHEN** the user creates any change via `speclink new change <name>`
- **THEN** the corresponding row SHALL have `state='proposing'` and `version=1`

#### Scenario: Direct state mutation is forbidden outside the state-machine capability

- **WHEN** any caller attempts to update `change.state` via a `change-store` method
- **THEN** no such method SHALL exist on the `ChangeStore` trait; the compiler SHALL reject the call at build time

#### Scenario: State machine drives lifecycle transitions

- **WHEN** the engine performs a state transition (e.g. `apply.start`, `task.done` auto-trigger, future `review.approve`)
- **THEN** the transition SHALL go through `StateMachineStore::transition_state` and the resulting `change.state` value SHALL be one of the six legal lifecycle values; any other value SHALL trigger error code `state.invalid_value`

#### Scenario: A2 "no transition CLI" constraint SHALL be lifted

- **WHEN** the user runs `speclink --help` after this slice ships
- **THEN** the help output SHALL advertise `speclink apply start` and `speclink apply pause` as commands that mutate `change.state` via the `state-machine` capability; the slice A2 "No transition CLI exists" scenario SHALL no longer apply


<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
code:
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/main.rs
  - crates/cli/src/commands/new_artifact.rs
  - crates/provider/src/types.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider-local/src/state_machine_store.rs
  - crates/cli/src/commands/new_change.rs
  - crates/runtime/src/change_ops.rs
  - crates/provider/Cargo.toml
  - crates/cli/src/commands/show_change.rs
  - crates/cli/src/human.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/commands/apply_pause.rs
  - doc/speclink-design.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/provider-local/Cargo.toml
  - README.md
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - doc/protocol/operations.md
  - crates/cli/src/commands/apply_start.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
  - crates/cli/src/commands/task_undo.rs
  - crates/cli/src/commands/task_done.rs
  - crates/cli/src/commands/init.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/commands/artifact_read.rs
  - crates/provider/src/lib.rs
  - crates/cli/src/lib.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider-local/src/paths.rs
  - crates/runtime/src/ops.rs
  - crates/runtime/src/task_ops.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/state_machine.rs
  - Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/provider/src/error.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/link.rs
  - crates/cli/src/commands/task_list.rs
  - crates/provider-local/src/store.rs
tests:
  - crates/cli/tests/artifact_io.rs
  - crates/cli/tests/json_envelope_v3.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/task_ops.rs
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/cli/tests/task_workflow.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/error_mapping.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v3.rs
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/provider-local/tests/state_machine_store.rs
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/runtime/tests/state_machine.rs
  - crates/runtime/tests/apply_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/runtime/tests/artifact_ops.rs
  - crates/runtime/tests/actor_resolution.rs
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/apply_lifecycle.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/change_crud.rs
  - crates/provider-local/tests/migration_v3.rs
-->

---
### Requirement: Change row Etag (the `version` column) SHALL start at 1 on creation

**Note**: Requirement heading SHALL be preserved verbatim from the slice A2 baseline for analyzer traceability; the body below supersedes the slice A2 "SHALL NOT mutate the change row after creation" clause with the slice A3 compare-and-swap contract.

Every new change row SHALL be inserted with `version=1`. The `version` column SHALL be a monotonic counter incremented by 1 on every successful `StateMachineStore` mutation that touches the change (state transition, actor assignment, actor clear, `all_tasks_done` flag flip). The `change-store` capability SHALL NOT mutate `version` directly; only the `state-machine` capability SHALL update it. A caller that observes a stale `version` and attempts a mutation SHALL receive error code `state.version_conflict` and exit code 7.

#### Scenario: Initial version is 1

- **WHEN** a new change is created
- **THEN** the row's `version` column SHALL equal 1

#### Scenario: Version increments monotonically on state-machine mutation

- **WHEN** the engine successfully invokes `apply.start` against a change with `version=1`
- **THEN** the row's `version` column SHALL equal 2 after commit

#### Scenario: CAS mismatch rejects mutation

- **WHEN** caller A and caller B both read `version=3`, caller A successfully runs `apply.start` (version becomes 4), then caller B attempts `apply.pause` with `expected_version=3`
- **THEN** caller B SHALL receive error code `state.version_conflict`, SHALL exit with code 7, and the row SHALL remain at `version=4` with caller A's state

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
code:
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/main.rs
  - crates/cli/src/commands/new_artifact.rs
  - crates/provider/src/types.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider-local/src/state_machine_store.rs
  - crates/cli/src/commands/new_change.rs
  - crates/runtime/src/change_ops.rs
  - crates/provider/Cargo.toml
  - crates/cli/src/commands/show_change.rs
  - crates/cli/src/human.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/commands/apply_pause.rs
  - doc/speclink-design.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - crates/provider-local/Cargo.toml
  - README.md
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - doc/protocol/operations.md
  - crates/cli/src/commands/apply_start.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
  - crates/cli/src/commands/task_undo.rs
  - crates/cli/src/commands/task_done.rs
  - crates/cli/src/commands/init.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/commands/artifact_read.rs
  - crates/provider/src/lib.rs
  - crates/cli/src/lib.rs
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/unlink.rs
  - crates/cli/src/commands/mod.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider-local/src/paths.rs
  - crates/runtime/src/ops.rs
  - crates/runtime/src/task_ops.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/state_machine.rs
  - Cargo.toml
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/provider/src/error.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/link.rs
  - crates/cli/src/commands/task_list.rs
  - crates/provider-local/src/store.rs
tests:
  - crates/cli/tests/artifact_io.rs
  - crates/cli/tests/json_envelope_v3.rs
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/task_ops.rs
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/cli/tests/task_workflow.rs
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/runtime/tests/error_mapping.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v3.rs
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/provider-local/tests/state_machine_store.rs
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/runtime/tests/state_machine.rs
  - crates/runtime/tests/apply_ops.rs
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/runtime/tests/artifact_ops.rs
  - crates/runtime/tests/actor_resolution.rs
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/apply_lifecycle.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/change_crud.rs
  - crates/provider-local/tests/migration_v3.rs
-->

---
### Requirement: `change.show` response envelope SHALL include `all_tasks_done` and `next_actions`

The `change.show` operation (CLI: `speclink show change <name>`) SHALL extend its response `data` object with two additional top-level fields: `all_tasks_done: bool` and `next_actions: [string]`. The existing `change` and `artifacts` fields SHALL be preserved unchanged. Both new fields SHALL be present in every successful response — they SHALL NOT be omitted, even when their values are `false` or `[]`.

The `all_tasks_done` field SHALL be read directly from the `change` table's `all_tasks_done` column maintained by the apply-task-ops capability. The runtime SHALL NOT re-parse `tasks.md` to compute this field.

The `next_actions` field SHALL be computed by the runtime as a state-driven lookup. The mapping SHALL be exactly as follows:

| state | all_tasks_done | next_actions |
|---|---|---|
| `proposing` | (ignored) | A filtered subset of `["artifact.write proposal", "artifact.write design", "artifact.write tasks"]` containing only those kinds whose artifact file does NOT yet exist on disk under the change directory |
| `reviewing` | (ignored) | `["review.approve", "review.reject"]` |
| `ready` | (ignored) | `["apply.start"]` |
| `in_progress` | `false` | `["task.done <INDEX>"]` where `<INDEX>` is the 1-based sequential checkbox index (the SAME integer index that `speclink task done <INDEX>` CLI accepts) of the FIRST unchecked task in `tasks.md`, computed by counting both `- [ ]` and `- [x]` lines from the top of the file. When `tasks.md` does not exist OR contains no unchecked checkbox line, the value SHALL be `["task.done"]` (no `<INDEX>` suffix). The runtime SHALL NOT emit the markdown label text (e.g. `2.1`) as the suffix, because the CLI rejects non-integer task arguments with `invalid digit found in string`. |
| `in_progress` | `true` | `["archive.run"]` |
| `code_reviewing` | (ignored) | `["review.approve", "review.reject"]` |
| `archived` | (ignored) | `[]` |

#### Scenario: archived change returns empty next_actions and all_tasks_done is preserved

- **GIVEN** a change `c1` whose `state='archived'` and whose `change` table row's `all_tasks_done` column is `true`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** the response `data.all_tasks_done` SHALL be `true` AND `data.next_actions` SHALL be `[]`

#### Scenario: in_progress change with unfinished tasks suggests task.done with the first pending index

- **GIVEN** a change `c1` whose `state='in_progress'` and whose `all_tasks_done` column is `false`, AND whose `tasks.md` first unchecked line is `- [ ] 2.1 Implement parser` while three earlier `- [x]` lines have already been checked off
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["task.done 4"]` (the 4th checkbox line, 1-based; NOT `["task.done 2.1"]`, since the CLI takes integer index, not label) AND `data.all_tasks_done` SHALL be `false`

#### Scenario: next_actions emits index that the task.done CLI actually accepts (no AI retry loop)

- **GIVEN** a change `c1` whose `state='in_progress'`, whose `all_tasks_done` is `false`, and whose `tasks.md` contains any mixture of `- [ ]` and `- [x]` lines with arbitrary label content (numeric, dotted, slugged, or absent)
- **WHEN** the runtime resolves `change.show` for `c1` AND the caller pipes the resulting `next_actions[0]` (with the leading `task.done ` stripped) into `speclink task done <INDEX>` against the SAME `tasks.md`
- **THEN** the `speclink task done` invocation SHALL succeed (i.e., SHALL NOT fail with `invalid digit found in string` because of label-vs-index mismatch), AND the toggled task SHALL be the same row identified by the next_actions hint

#### Scenario: in_progress change with all tasks done suggests archive.run

- **GIVEN** a change `c1` whose `state='in_progress'` and whose `all_tasks_done` column is `true`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["archive.run"]` AND `data.all_tasks_done` SHALL be `true`

#### Scenario: ready change suggests apply.start

- **GIVEN** a change `c1` whose `state='ready'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["apply.start"]`

#### Scenario: code_reviewing change offers both approve and reject

- **GIVEN** a change `c1` whose `state='code_reviewing'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["review.approve", "review.reject"]`

#### Scenario: reviewing change offers both approve and reject

- **GIVEN** a change `c1` whose `state='reviewing'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["review.approve", "review.reject"]`

#### Scenario: proposing change recommends writing only the missing artifact kinds

- **GIVEN** a change `c1` whose `state='proposing'` and whose change directory contains `proposal.md` only (no `design.md`, no `tasks.md`)
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["artifact.write design", "artifact.write tasks"]`

#### Scenario: proposing change with all three core artifacts present returns empty hint

- **GIVEN** a change `c1` whose `state='proposing'` and whose change directory contains `proposal.md`, `design.md`, and `tasks.md`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `[]`

#### Scenario: in_progress change without tasks.md falls back to bare task.done hint

- **GIVEN** a change `c1` whose `state='in_progress'` and `all_tasks_done=false`, AND whose change directory does NOT contain a `tasks.md` file
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["task.done"]`

#### Scenario: existing show change fields SHALL be preserved exactly

- **WHEN** the runtime resolves `change.show` for any change
- **THEN** the response `data` SHALL still contain a `change` field (with the same row metadata as before this requirement) AND an `artifacts` field (with the same array shape as before this requirement) AND no field SHALL be removed or renamed

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/cli/src/commands/show_change.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
tests:
  - crates/cli/tests/change_crud.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/runtime/tests/change_ops.rs
-->

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/cli/src/commands/show_change.rs
  - doc/protocol/operations.md
  - doc/speclink-design.md
  - crates/runtime/src/lib.rs
  - crates/runtime/src/project_ops.rs
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/catalogue/mod.rs
  - crates/runtime/src/tool_ops/render.rs
  - crates/runtime/src/catalogue/schemas.rs
tests:
  - crates/runtime/tests/project_ops.rs
  - crates/cli/tests/status.rs
  - crates/cli/tests/change_crud.rs
  - crates/cli/tests/snapshots/describe_tools__describe_tools_envelope.snap
  - crates/cli/tests/describe_tools.rs
  - crates/cli/tests/snapshots/status__status_envelope.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
-->