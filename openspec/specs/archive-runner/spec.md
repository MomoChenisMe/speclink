# archive-runner Specification

## Purpose

TBD - created by syncing change 'add-archive'. Update Purpose after archive.

## Requirements

### Requirement: `speclink archive` SHALL transition the change from `in_progress` to `archived` when all tasks are done

The CLI command `speclink archive <change-id> [--skip-specs] [--yes] [--no-validate] [--json]` SHALL invoke the `archive.run` operation. Under walking-skeleton mode (`require_code_review=false`, hard-coded by `add-state-machine-and-apply`), the engine SHALL permit the transition only when both `change.state='in_progress'` AND `change.all_tasks_done=1`; any other state or any combination where `all_tasks_done=0` SHALL be rejected. On success the engine SHALL within a single SQLite transaction (1) update `change.state` to `archived`, (2) set `change.archived_at` to the UTC timestamp at commit time, (3) increment `change.version`, and (4) insert one `state_transition` row with `from_state='in_progress'`, `to_state='archived'`, `reason='archive_run'`, `actor_json=NULL` (archive does not require an actor), and `transitioned_at=now`. After the transaction commits, the engine SHALL atomically rename `.speclink/changes/<change-id>/` to `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/` using the UTC date of the `archived_at` timestamp, appending `-2`, `-3`, ... when the target directory already exists.

#### Scenario: Happy path archives in_progress change with all tasks done

- **WHEN** the user runs `speclink archive demo --json` against a change in `in_progress` state with `all_tasks_done=1`
- **THEN** the engine SHALL exit with code 0, SHALL update `change.state='archived'`, SHALL set `change.archived_at` to a UTC ISO-8601 timestamp, SHALL move the change directory to `.speclink/changes/archive/<date>-demo/`, and SHALL return the success envelope below

##### Example: success envelope

```json
{
  "ok": true,
  "data": {
    "change_id": "demo",
    "state": "archived",
    "merged_specs": [
      { "capability": "user-auth", "lines_added": 142, "lines_removed": 0 }
    ],
    "archived_at": "2026-05-22T18:00:00Z",
    "archive_dir": ".speclink/changes/archive/2026-05-22-demo"
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

#### Scenario: Same-day collision appends suffix

- **WHEN** the user runs `speclink archive demo` and `.speclink/changes/archive/2026-05-22-demo/` already exists
- **THEN** the engine SHALL rename to `.speclink/changes/archive/2026-05-22-demo-2/`; if that also exists, to `-3`, etc.

#### Scenario: Cross-state reject — any state other than in_progress

- **WHEN** the user runs `speclink archive demo` against a change in `proposing`, `reviewing`, `ready`, `code_reviewing`, or `archived` state
- **THEN** the engine SHALL emit error code `state.transition_invalid`, SHALL exit with code 7, SHALL NOT mutate the change row, SHALL NOT rename any directory, and SHALL NOT touch `.speclink/specs/`

##### Example: state guard matrix

| Current state | `all_tasks_done` | Result |
| --- | --- | --- |
| `proposing` | any | `state.transition_invalid` (exit 7) |
| `reviewing` | any | `state.transition_invalid` (exit 7) |
| `ready` | any | `state.transition_invalid` (exit 7) |
| `in_progress` | `0` | `change.tasks_incomplete` (exit 2) |
| `in_progress` | `1` | success → `archived` |
| `code_reviewing` | any | `state.transition_invalid` (exit 7) (reserved for future review slice) |
| `archived` | any | `state.transition_invalid` (exit 7) |

#### Scenario: Tasks-incomplete reject within in_progress

- **WHEN** the user runs `speclink archive demo` against a change in `in_progress` state but `all_tasks_done=0`
- **THEN** the engine SHALL emit error code `change.tasks_incomplete`, SHALL exit with code 2, SHALL provide hint `complete all tasks first with \`speclink task done <i> --change <id>\``, and SHALL NOT mutate any state

#### Scenario: Change not found

- **WHEN** the user runs `speclink archive nonexistent`
- **THEN** the engine SHALL emit error code `change.not_found`, SHALL exit with code 2, and SHALL NOT touch any filesystem path


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: Spec delta merge SHALL atomically overwrite the target capability spec for each capability under the change

For each `.speclink/changes/<change-id>/specs/<capability>/spec.md` file present in the change directory at archive time, the engine SHALL atomically write its contents to `.speclink/specs/<capability>/spec.md`, creating the `<capability>/` directory if it does not already exist. The write SHALL use a tempfile-then-rename sequence inherited from `add-change-and-artifact-io`. The engine SHALL NOT parse the spec contents, SHALL NOT diff against the existing target file, and SHALL NOT detect or warn about merge conflicts; this slice ships dumb integer overwrite only, with schema-aware delta merge reserved for the future `add-schema-management` slice. For each merged capability the engine SHALL include `{ capability, lines_added, lines_removed }` in the response `data.merged_specs` array, where `lines_added` is the line count of the newly written file and `lines_removed` is the line count of the prior target file (zero if the target did not exist).

#### Scenario: New capability directory is created on first merge

- **WHEN** the user archives a change with `specs/audit-log/spec.md` and `.speclink/specs/audit-log/` does not yet exist
- **THEN** the engine SHALL create `.speclink/specs/audit-log/`, SHALL write `spec.md` atomically, and SHALL emit `merged_specs: [{ "capability": "audit-log", "lines_added": <new_count>, "lines_removed": 0 }]`

#### Scenario: Existing capability spec is overwritten without delta parsing

- **WHEN** the user archives a change with `specs/user-auth/spec.md` and `.speclink/specs/user-auth/spec.md` already exists with 90 lines
- **THEN** the engine SHALL replace the entire file contents (no diff, no conflict detection), and SHALL emit `merged_specs: [{ "capability": "user-auth", "lines_added": <new_count>, "lines_removed": 90 }]`

#### Scenario: Change with no spec directories produces empty merged_specs

- **WHEN** the user archives a change whose `specs/` directory is empty or does not exist
- **THEN** the engine SHALL succeed with `merged_specs: []` and SHALL still complete the state transition and directory rename


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: `--skip-specs` SHALL bypass merge while still transitioning state and emit an audit warning

When the `--skip-specs` flag is set, the engine SHALL NOT read or write any file under `.speclink/specs/`, SHALL NOT include any `merged_specs` entries (the array SHALL be empty), and SHALL still complete the state transition and directory rename. The engine SHALL append one entry to the response `warnings` array with shape `{ "code": "archive.specs_skipped", "message": "Spec delta merge skipped (--skip-specs).", "details": { "capabilities_skipped": [...] } }`, where `capabilities_skipped` lists the capability names that were present under the change's `specs/` directory but not merged.

#### Scenario: Skip-specs path leaves `.speclink/specs/` untouched

- **WHEN** the user runs `speclink archive demo --skip-specs --json` against a change with `specs/user-auth/spec.md` and `specs/audit-log/spec.md` present
- **THEN** `.speclink/specs/user-auth/` and `.speclink/specs/audit-log/` SHALL remain byte-for-byte unchanged, the response `data.merged_specs` SHALL be `[]`, and the response `warnings` SHALL contain exactly one entry with `code='archive.specs_skipped'` and `details.capabilities_skipped=["audit-log","user-auth"]` (sorted)

#### Scenario: Skip-specs with empty specs directory produces no warning

- **WHEN** the user runs `speclink archive demo --skip-specs` against a change whose `specs/` directory is empty or absent
- **THEN** the response `warnings` array SHALL NOT contain `archive.specs_skipped` (there was nothing to skip)


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: `--no-validate` flag SHALL be accepted by the CLI but SHALL be a no-op in this slice

The CLI SHALL parse the `--no-validate` flag without rejecting it and SHALL forward it through the runtime layer. The runtime SHALL hard-code the "skip validation" path regardless of the flag's value in this slice; the `validation.archive_failed` error code SHALL be reserved in `crates/provider/src/error.rs::codes` and `crates/runtime/src/error.rs` but SHALL NOT be emitted by any code path in this slice. The CLI `--help` text for `archive` SHALL document the flag as "reserved for future analyze slice; currently no-op". A future slice activating real validation SHALL NOT require a CLI surface change.

#### Scenario: --no-validate is accepted and ignored

- **WHEN** the user runs `speclink archive demo --no-validate` against a valid `in_progress` change with all tasks done
- **THEN** the engine SHALL succeed identically to running without the flag (no validation step exists yet)

#### Scenario: Help text documents flag as no-op

- **WHEN** the user runs `speclink archive --help`
- **THEN** the help output for `--no-validate` SHALL include the text "reserved for future analyze slice; currently no-op"


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: `--yes` flag SHALL be accepted by the CLI; archive SHALL never prompt interactively in this slice

The `archive` CLI SHALL NOT issue interactive confirmation prompts in this slice (archive is classified as an AI-workflow operation, non-destructive in the sense that artifacts are preserved under `archive/`). The `--yes` flag SHALL be parsed and accepted to preserve forward compatibility with the catalogue surface defined in `doc/speclink-design.md §16.9`, but SHALL NOT alter runtime behavior in this slice.

#### Scenario: --yes is a no-op pass-through

- **WHEN** the user runs `speclink archive demo --yes`
- **THEN** the engine SHALL behave identically to `speclink archive demo` (no prompt either way)


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: state.db SHALL be upgraded to version 4 with `archived_at` column on the `change` table

The `crates/provider-local/src/state_db.rs::MIGRATIONS` array SHALL append a migration to version 4 that adds the column `archived_at TIMESTAMP NULL` to the `change` table, leaving existing rows with `archived_at=NULL`. The `LocalProjectStore::open_state_db()` call site SHALL be bumped from `db.migrate(3)` to `db.migrate(4)`. The migration SHALL be idempotent under repeat invocation (re-running v4 against a v4-current DB SHALL be a no-op). The migration SHALL NOT create any new tables.

#### Scenario: Migration from v3 to v4 adds column and bumps version

- **WHEN** a state.db at schema version 3 is opened by a CLI built against this slice
- **THEN** the migration SHALL add the `archived_at` column to the `change` table, SHALL insert a row in `_migrations` with `version=4`, and SHALL leave all existing `change` rows with `archived_at=NULL`

#### Scenario: Migration is idempotent against a v4-current DB

- **WHEN** a state.db already at schema version 4 is opened
- **THEN** the migration runner SHALL detect `_migrations.version=4` and SHALL skip the v4 step; the column already exists and SHALL NOT be re-added


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: New error codes SHALL be registered with stable exit codes

The slice SHALL register exactly two new user-facing error codes plus one audit / warning code with the following mapping:

| Code | Kind | Exit code | `retryable` | Hint |
| --- | --- | --- | --- | --- |
| `change.tasks_incomplete` | user-facing error | 2 | `no` | `complete all tasks first with \`speclink task done <i> --change <id>\`` |
| `validation.archive_failed` | user-facing error (reserved) | 3 | `no` | `run \`speclink validate <id>\` first` |
| `archive.specs_skipped` | warning carrier (envelope `warnings` array) | n/a (success path) | n/a | n/a |

Both error codes SHALL be added as `pub const` in `crates/provider/src/error.rs::codes` and as variants in `ProviderError`. They SHALL be added to `RuntimeError` with matching variants and propagated through `RuntimeError::code()` / `exit_code()` / `retryable()`. The CLI's `error_code_to_exit` and `hint_for` match arms SHALL be extended to cover both. `validation.archive_failed` SHALL be reserved (no code path emits it) until a future analyze slice activates validation.

#### Scenario: change.tasks_incomplete is wired through all three layers

- **WHEN** the user runs `speclink archive demo --json` against `in_progress + all_tasks_done=0`
- **THEN** the JSON envelope `error.code` SHALL be `change.tasks_incomplete`, `error.retryable` SHALL be `no`, `error.hint` SHALL contain `speclink task done`, and the CLI process SHALL exit with code 2

#### Scenario: validation.archive_failed is reserved but unused

- **WHEN** any code path in this slice runs
- **THEN** `validation.archive_failed` SHALL NOT appear in any test fixture's stdout, stderr, or JSON envelope, but SHALL be present as a `pub const` in `crates/provider/src/error.rs::codes` and as a variant in both `ProviderError` and `RuntimeError`


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: Filesystem rename SHALL happen after SQLite transaction commit; best-effort revert SHALL run if rename fails

The engine SHALL execute the SQLite transaction (state update + audit insert) first and ONLY rename `.speclink/changes/<id>/` to `.speclink/changes/archive/<date>-<id>/` AFTER the transaction has committed. If the rename fails (cross-device, permission, target already exists after suffix exhaustion, or any other `std::fs::rename` error), the engine SHALL attempt a best-effort revert: open a new SQLite transaction, set `change.state` back to `in_progress`, clear `change.archived_at` to NULL, and insert a compensating `state_transition` row with `from_state='archived'`, `to_state='in_progress'`, `reason='archive_run_revert'`, `actor_json=NULL`. If the revert transaction itself fails, the engine SHALL bubble up the original error with exit code 1 and SHALL NOT attempt further compensation; the user is expected to inspect state.db and the filesystem manually (doctor slice will provide diagnostics in a future slice).

#### Scenario: Rename success after commit produces consistent end state

- **WHEN** archive succeeds end-to-end
- **THEN** `change.state='archived'` AND `.speclink/changes/<id>/` does not exist AND `.speclink/changes/archive/<date>-<id>/` exists

#### Scenario: Rename failure triggers DB revert

- **WHEN** the SQLite transaction has committed but `std::fs::rename` fails
- **THEN** the engine SHALL run a revert transaction restoring `change.state='in_progress'`, clearing `archived_at`, and inserting the compensating `state_transition` row; the CLI SHALL exit with code 1 and surface the rename error to the user


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: Spec merge SHALL happen after the directory rename

Within a successful archive (without `--skip-specs`), the engine SHALL perform spec delta merge AFTER the change directory rename completes successfully. The merge SHALL read spec files from the NEW location (`.speclink/changes/archive/<date>-<id>/specs/`) and write to `.speclink/specs/<capability>/spec.md`. This ordering ensures that if the directory rename fails (and DB is reverted), no spec writes have happened.

#### Scenario: Order — DB commit, directory rename, then spec merge

- **WHEN** archive runs successfully
- **THEN** the on-disk state SHALL transition through exactly these stages, in this order: (1) state.db reflects `archived` + `archived_at`, (2) `.speclink/changes/<id>/` no longer exists; `.speclink/changes/archive/<date>-<id>/` does, (3) `.speclink/specs/<capability>/spec.md` files for each capability under the change have been written


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: Provider trait SHALL expose `ArchiveStore` with a single `archive_change` method

The crate `crates/provider` SHALL define a new trait `ArchiveStore` with one method `fn archive_change(&self, req: ArchiveRequest) -> Result<ArchiveResult, ProviderError>`. The crate `crates/provider-local` SHALL provide the implementation `LocalArchiveStore` that wires all DB writes, directory rename, and spec merge as described in the requirements above. The trait SHALL be re-exported from `crates/provider/src/lib.rs` alongside existing `ProjectStore` / `ChangeStore` / `ArtifactStore` / `StateMachineStore` traits.

#### Scenario: Trait surface is callable from runtime layer

- **WHEN** the runtime crate's `ArchiveOperations<G>` invokes `provider.archive_change(req)` against a `LocalArchiveStore`
- **THEN** the call SHALL return `Ok(ArchiveResult { state, merged_specs, archived_at, archive_dir })` on success or one of the `ProviderError` variants `ChangeNotFound`, `StateTransitionInvalid`, `ChangeTasksIncomplete`, or `StateVersionConflict` on failure


<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->

---
### Requirement: JSON envelope SHALL conform to the bootstrap / A2 / A3 contract

The `archive` CLI SHALL emit responses in the standard envelope shape inherited from the bootstrap slice: success `{ ok: true, data, warnings, requestId }` or error `{ ok: false, error: { code, message, hint, retryable, retry_after_ms }, requestId }`. The `data` shape SHALL be exactly `{ change_id: string, state: "archived", merged_specs: Array<{ capability: string, lines_added: integer, lines_removed: integer }>, archived_at: string (UTC ISO-8601), archive_dir: string }`. Each element of the `warnings` array SHALL have shape `{ code: string, message: string, details?: object }`, where `details` is an optional structured payload omitted when the warning has no carrier-specific fields. The `requestId` SHALL be a fresh ULID generated per invocation.

#### Scenario: Success envelope field order is stable

- **WHEN** `archive` succeeds
- **THEN** the JSON output SHALL contain top-level keys `ok`, `data`, `warnings`, `requestId` in exactly that order, and `data` SHALL contain keys `change_id`, `state`, `merged_specs`, `archived_at`, `archive_dir` in exactly that order

#### Scenario: Error envelope retryable values

- **WHEN** `archive` fails with `state.transition_invalid`, `change.tasks_incomplete`, or `change.not_found`
- **THEN** the `error.retryable` field SHALL be `no` for all three; `archive` SHALL NEVER emit a `retryable=backoff` or `retryable=read-then-retry` error in this slice

#### Scenario: Skip-specs warning rides along with success

- **WHEN** `archive` succeeds with `--skip-specs` AND the change had at least one capability under `specs/`
- **THEN** the success envelope `warnings` array SHALL contain exactly one entry with `code='archive.specs_skipped'`, `message` SHALL describe the skip, and `details.capabilities_skipped` SHALL list the affected capability names sorted alphabetically

<!-- @trace
source: add-archive
updated: 2026-05-23
-->

<!-- @trace
source: add-archive
updated: 2026-05-23
code:
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/ops.rs
  - crates/provider-local/src/artifact_store.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider-local/src/store.rs
  - crates/runtime/src/dev_precheck.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/change_ops.rs
  - doc/protocol/operations.md
  - README.md
  - crates/provider-local/src/state_machine_store.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/archive_store.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/commands/archive.rs
  - crates/runtime/src/archive_ops.rs
  - crates/provider-local/src/change_store.rs
  - doc/speclink-design.md
tests:
  - crates/runtime/tests/archive_ops.rs
  - crates/cli/tests/archive_terminal_state.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_skip_specs_envelope.snap
  - crates/cli/tests/archive_state_guards.rs
  - crates/provider-local/tests/migration_v3.rs
  - crates/cli/tests/archive_skip_specs.rs
  - crates/cli/tests/archive_walking_skeleton.rs
  - crates/runtime/tests/error_mapping.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/json_envelope_v4.rs
  - crates/provider-local/tests/archive_store.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_success_envelope.snap
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_tasks_incomplete_envelope.snap
  - crates/provider/tests/error_codes.rs
  - crates/cli/tests/human_output_v4.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_state_transition_invalid_envelope.snap
  - crates/provider/tests/trait_surface.rs
  - crates/cli/tests/snapshots/json_envelope_v4__archive_change_not_found_envelope.snap
-->