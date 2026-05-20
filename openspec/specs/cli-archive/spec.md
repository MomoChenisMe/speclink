# cli-archive Specification

## Purpose

TBD - created by archiving change 'add-local-archive'. Update Purpose after archive.

## Requirements

### Requirement: `archive` command surface

The CLI SHALL provide a subcommand `speclink archive <change>` with the following surface:

- **Command form**: `speclink archive <change-id> [--dry-run] [--json] [--no-color] [--quiet]`
- **Positional argument**:
  - `<change-id>`: the kebab-case change id to archive; validation rules follow the `local-provider-storage` capability
- **Optional flags**:
  - `--dry-run`: simulate the archive without modifying any files or SQLite state; output the same JSON envelope shape as a real archive but with `data.dryRun = true`
  - The machine-interface flags `--json`, `--no-color`, `--quiet` as defined in the `cli-machine-interface` capability
- **Stdin**: this command SHALL NOT accept stdin. If invoked with `--stdin`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

The CLI SHALL determine the archive date as `chrono::Local::now().date_naive()` formatted `%Y-%m-%d` and pass it to the runtime. The runtime SHALL pass the same date to the provider — the provider SHALL NOT call the clock itself.

#### Scenario: Real archive moves change directory

- **GIVEN** an active change `demo` with `proposal.md` and `specs/auth/spec.md`
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the directory `.speclink/changes/demo/` does not exist after the command
- **AND** the directory `.speclink/changes/archive/YYYY-MM-DD-demo/` exists (where `YYYY-MM-DD` is today in local timezone)
- **AND** the file `.speclink/specs/auth/spec.md` exists
- **AND** the process exit code is 0

#### Scenario: Dry-run leaves filesystem unchanged

- **GIVEN** an active change `demo` with `proposal.md` and `specs/auth/spec.md`
- **WHEN** the user runs `speclink archive demo --dry-run --json`
- **THEN** the directory `.speclink/changes/demo/` still exists
- **AND** the directory `.speclink/changes/archive/` does not exist (if it did not exist before)
- **AND** the file `.speclink/specs/auth/spec.md` does not exist (if it did not exist before)
- **AND** the stdout JSON `data.dryRun` equals `true`
- **AND** the stdout JSON `data.archivePath` equals the path that would have been used
- **AND** the stdout JSON `data.specSync.capabilitiesSynced` reports the planned add/modify/remove/rename counts
- **AND** the process exit code is 0


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: `archive` JSON output schema

On success, the `data` payload of the `--json` envelope SHALL be a JSON object with the following fields (all required):

- `changeId` (string): the archived change id
- `archivePath` (string): POSIX-style path of the archive directory (relative to project base, forward slashes on all OS); when `dryRun` is true, this is the path that would have been used
- `state` (string): `"archived"` (the new lifecycle state)
- `archivedAt` (string): ISO 8601 UTC timestamp with second precision (e.g., `2026-05-19T12:34:56Z`); when `dryRun` is true, this is the timestamp that would have been recorded
- `dryRun` (boolean): `true` if `--dry-run` was passed, otherwise `false`
- `specSync` (object): the spec sync summary, see `SpecSyncSummary` schema below

`SpecSyncSummary` schema:

- `capabilitiesSynced` (array of objects): one entry per capability spec in the change

Each `CapabilitySyncResult` object schema:

- `capability` (string): the capability name
- `mainSpecPath` (string): POSIX-style path of the resulting main spec file
- `addedCount` (integer): number of `### Requirement:` blocks under `## ADDED Requirements` in the delta
- `modifiedCount` (integer): number of `### Requirement:` blocks under `## MODIFIED Requirements`
- `removedCount` (integer): number of `### Requirement:` blocks under `## REMOVED Requirements`
- `renamedCount` (integer): number of `### Requirement:` blocks under `## RENAMED Requirements`
- `createdMainSpec` (boolean): `true` if `.speclink/specs/<capability>/spec.md` did not exist before this archive (newly created); `false` if it pre-existed

The CLI SHALL serialize all integers as JSON numbers (not strings).

#### Scenario: Archive output for first-time capability

- **GIVEN** a change `demo` with `specs/auth/spec.md` containing 2 ADDED requirements
- **AND** `.speclink/specs/auth/spec.md` does not exist before archive
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the stdout JSON `data.specSync.capabilitiesSynced[0]` equals (modulo paths and timestamps):

  ```json
  {
    "capability": "auth",
    "mainSpecPath": ".speclink/specs/auth/spec.md",
    "addedCount": 2,
    "modifiedCount": 0,
    "removedCount": 0,
    "renamedCount": 0,
    "createdMainSpec": true
  }
  ```

#### Scenario: Multi-capability change

- **GIVEN** a change `demo` with `specs/auth/spec.md` (1 ADDED) and `specs/billing/spec.md` (1 MODIFIED against existing main spec)
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the `data.specSync.capabilitiesSynced` array contains 2 entries
- **AND** entries are sorted ascending by capability name (`auth` before `billing`)
- **AND** the `auth` entry has `createdMainSpec: true` and `addedCount: 1`
- **AND** the `billing` entry has `createdMainSpec: false` and `modifiedCount: 1`


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: `archive` failure mapping

The CLI SHALL map failures to exit codes and error codes as follows:

| Trigger condition                                                                  | error code                            | exit code |
| ---------------------------------------------------------------------------------- | ------------------------------------- | --------- |
| Change does not exist                                                              | `change.not_found`                    | 1         |
| Change id fails kebab-case validation                                              | `change.invalid_id`                   | 2         |
| Change `metadata.json` state is already `"archived"`                               | `archive.change_not_archivable`       | 1         |
| Target archive directory `archive/YYYY-MM-DD-<id>/` already exists                 | `archive.change_not_archivable`       | 1         |
| Delta heading format error (unrecognized `## ` heading, malformed `### Requirement:`) | `spec.delta_parse_error`             | 2         |
| ADDED requirement name already present in main spec                                | `spec.delta_conflict`                 | 7         |
| MODIFIED, REMOVED, or RENAMED requirement name not found in main spec              | `spec.delta_conflict`                 | 7         |
| Filesystem write failure                                                           | `internal.error`                      | 1         |
| SQLite write failure                                                               | `internal.error`                      | 1         |

When `--dry-run` is passed, the same `spec.delta_conflict` and `spec.delta_parse_error` errors SHALL be returned (exit codes 7 and 2 respectively); other errors SHALL NOT trigger in dry-run mode because no filesystem or SQLite writes occur.

#### Scenario: Already archived

- **GIVEN** a change `demo` whose `metadata.json` has `state: "archived"`
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "archive.change_not_archivable"`

#### Scenario: Delta conflict reports exit code 7

- **GIVEN** a change `demo` whose `specs/auth/spec.md` contains `## ADDED Requirements` with `### Requirement: User login`
- **AND** `.speclink/specs/auth/spec.md` already contains `### Requirement: User login`
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the process exit code is 7
- **AND** the stdout JSON contains a failure envelope with `error.code = "spec.delta_conflict"`
- **AND** the `error.message` includes the conflicting requirement name

#### Scenario: Dry-run still reports conflict

- **GIVEN** the same setup as the previous scenario
- **WHEN** the user runs `speclink archive demo --dry-run --json`
- **THEN** the process exit code is 7
- **AND** the stdout JSON contains a failure envelope with `error.code = "spec.delta_conflict"`
- **AND** no filesystem changes occurred

<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->