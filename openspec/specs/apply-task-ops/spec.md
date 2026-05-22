# apply-task-ops Specification

## Purpose

TBD - created by syncing change 'add-state-machine-and-apply'. Update Purpose after archive.

## Requirements

### Requirement: `speclink apply start` SHALL implement the ensure-actor semantics defined by design.md §6.2

The CLI command `speclink apply start <change-id> [--actor <id>] [--json]` SHALL behave as a stateful ensure-actor operation, not as a pure transition. For each current `change.state` it SHALL act as follows: `ready` → transition to `in_progress` and assign actor; `in_progress` → no-op transition with optional actor reassignment; `code_reviewing` → no transition, return success envelope with `state="code_reviewing"` and a hint message; `archived` → no transition, return success envelope with `state="archived"` and a hint message; `proposing` or `reviewing` → reject with error code `state.transition_invalid` and exit code 7.

#### Scenario: Ready transitions to in_progress with actor assigned

- **WHEN** the user runs `speclink apply start demo --json` against a change in `ready` state
- **THEN** the engine SHALL transition the change to `in_progress`, SHALL populate `change.actor_json` with the resolved actor, SHALL exit with code 0, and SHALL return the success envelope below

##### Example: success envelope

```json
{
  "ok": true,
  "data": {
    "change_id": "demo",
    "state": "in_progress",
    "actor": {
      "agent_host": "claude-code",
      "os_user": "alice",
      "host_id": "macbook-alice"
    },
    "message": null
  },
  "warnings": [],
  "requestId": "01HXXXXXXXXXXXXXXXXXXXXXXX"
}
```

#### Scenario: In_progress reassigns actor without transition

- **WHEN** the user runs `speclink apply start demo --actor cursor` against a change already in `in_progress` whose stored `actor.agent_host` differs from `cursor`
- **THEN** the engine SHALL keep `state="in_progress"`, SHALL overwrite `change.actor_json` with the new actor, SHALL increment `change.version`, SHALL NOT insert a `state_transition` row, and the response envelope SHALL return the updated actor

#### Scenario: Code_reviewing returns hint without transition

- **WHEN** the user runs `speclink apply start demo` against a change in `code_reviewing` state
- **THEN** the engine SHALL exit with code 0, SHALL NOT mutate the change row, and the response `data.message` SHALL equal `Already in code review; nothing to apply.`

#### Scenario: Archived returns hint without transition

- **WHEN** the user runs `speclink apply start demo` against a change in `archived` state
- **THEN** the engine SHALL exit with code 0, SHALL NOT mutate the change row, and the response `data.message` SHALL equal `Change is archived.`

#### Scenario: Proposing rejects with transition_invalid

- **WHEN** the user runs `speclink apply start demo` against a change in `proposing` or `reviewing` state
- **THEN** the engine SHALL exit with code 7, SHALL emit error code `state.transition_invalid`, and the error envelope `error.hint` SHALL describe that the change is not yet approved for apply

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->


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
### Requirement: Actor SHALL be resolved by fallback chain when `--actor` flag is omitted

When `apply.start` is invoked without an explicit `--actor` flag, the engine SHALL resolve the actor by reading three fields independently: `agent_host` from environment variable `SPECLINK_AGENT_HOST` (falling back to the literal string `cli` when unset or empty), `os_user` from cross-platform `whoami` lookup (falling back to the literal string `unknown` when lookup fails), and `host_id` from cross-platform hostname lookup (falling back to the literal string `unknown` when lookup fails). The resolved `Actor { agent_host, os_user, host_id }` SHALL be persisted in `change.actor_json` and reflected in the response envelope.

#### Scenario: Fallback chain populates all three fields

- **WHEN** `SPECLINK_AGENT_HOST` is unset, `whoami` returns `alice`, hostname returns `macbook-alice`, and the user omits `--actor`
- **THEN** the engine SHALL persist `actor = { agent_host: "cli", os_user: "alice", host_id: "macbook-alice" }`

#### Scenario: Explicit flag overrides agent_host only

- **WHEN** the user passes `--actor claude-code` and `whoami` returns `bob`, hostname returns `linux-box`
- **THEN** the engine SHALL persist `actor = { agent_host: "claude-code", os_user: "bob", host_id: "linux-box" }`; the `--actor` flag SHALL be interpreted as `agent_host` only, not as a composite identifier

#### Scenario: Hostname lookup failure falls back to literal `unknown`

- **WHEN** the cross-platform hostname call returns an OS error (e.g. sandboxed environment with no hostname)
- **THEN** the engine SHALL persist `host_id="unknown"` and SHALL NOT fail the operation

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->


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
### Requirement: `speclink apply pause` SHALL implement symmetric idempotency against `apply.start`

The CLI command `speclink apply pause <change-id> [--json]` SHALL behave as follows: `in_progress` → transition to `ready` and clear `change.actor_json` to NULL; `ready` → no-op transition (already paused); `proposing`, `reviewing`, `code_reviewing`, or `archived` → reject with error code `state.transition_invalid` and exit code 7.

#### Scenario: In_progress transitions to ready and clears actor

- **WHEN** the user runs `speclink apply pause demo --json` against a change in `in_progress` state with a populated actor
- **THEN** the engine SHALL transition the change to `ready`, SHALL set `change.actor_json` to NULL, SHALL insert a `state_transition` row with `reason='apply_pause'`, and the response `data.actor` SHALL be `null`

#### Scenario: Ready is idempotent no-op

- **WHEN** the user runs `speclink apply pause demo` against a change already in `ready` state
- **THEN** the engine SHALL exit with code 0, SHALL NOT mutate the change row, and the response `data.message` SHALL equal `Change is already paused.`

#### Scenario: Code_reviewing rejects with transition_invalid

- **WHEN** the user runs `speclink apply pause demo` against a change in `code_reviewing` or `archived` state
- **THEN** the engine SHALL exit with code 7 and SHALL emit error code `state.transition_invalid`

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->


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
### Requirement: `speclink task list` SHALL enumerate checkbox lines from tasks.md by 1-based index

The CLI command `speclink task list --change <id> [--json]` SHALL read `.speclink/changes/<change-id>/tasks.md`, SHALL parse every line that matches the regular expression `^(\s*)- \[( |x)\] (.+)$`, and SHALL return them as an array in document order, each annotated with a 1-based index, a `done` boolean, and the raw text after the checkbox marker. Lines that do not match the regex (headings, plain text, prose) SHALL be skipped silently. The regex SHALL be case-sensitive: only `x` (lowercase) SHALL count as `done=true`.

#### Scenario: Mixed file produces ordered list

- **WHEN** tasks.md contains the content below
- **THEN** the response `data.tasks` SHALL be exactly the array shown

##### Example: parsed task list

GIVEN tasks.md content:

```
# Tasks

## Setup
- [ ] write proposal
- [x] register schema

## Implementation
Some prose here.
- [ ] implement adapter
  - [x] nested subtask
```

THEN the response `data.tasks` SHALL be:

```json
[
  { "index": 1, "done": false, "text": "write proposal" },
  { "index": 2, "done": true, "text": "register schema" },
  { "index": 3, "done": false, "text": "implement adapter" },
  { "index": 4, "done": true, "text": "nested subtask" }
]
```

#### Scenario: Missing tasks.md is a hard error

- **WHEN** the user runs `speclink task list --change demo` and `.speclink/changes/demo/tasks.md` does not exist
- **THEN** the engine SHALL emit error code `task.no_tasks_file`, SHALL exit with code 2, and the `error.hint` SHALL instruct the user to write tasks.md first

#### Scenario: Case-sensitive checkbox marker

- **WHEN** tasks.md contains a line `- [X] uppercase x marker`
- **THEN** the line SHALL NOT match the parser regex, SHALL be skipped from the result list, and the engine SHALL NOT emit any error or warning

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->


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
### Requirement: `speclink task done` SHALL mark exactly one checkbox by 1-based index and auto-trigger when all tasks complete

The CLI command `speclink task done <task-index> --change <id> [--json]` SHALL parse tasks.md as defined above, SHALL locate the task at the supplied 1-based index, and SHALL atomically rewrite the file with that line's checkbox changed from `[ ]` to `[x]`. The write SHALL use a tempfile-then-rename sequence inherited from `add-change-and-artifact-io`. If the task is already done, the operation SHALL be idempotent (no file write, no audit row). After the write, if all parsed tasks are now `[x]`, the engine SHALL apply the `task_done_auto` trigger via the state machine: under walking-skeleton mode it SHALL set `change.all_tasks_done=1` while keeping `state='in_progress'`.

#### Scenario: First-time mark on a not-yet-done task

- **WHEN** the user runs `speclink task done 1 --change demo` against a change in `in_progress` state and tasks.md line 1 is `- [ ] do thing`
- **THEN** the engine SHALL rewrite line 1 to `- [x] do thing`, SHALL increment `change.version`, and the response `data` SHALL be `{ index: 1, done: true, all_tasks_done: false, state: "in_progress", auto_transitioned: false }`

#### Scenario: Idempotent re-mark on already-done task

- **WHEN** the user runs `speclink task done 1 --change demo` against a task already marked `- [x]`
- **THEN** the engine SHALL NOT rewrite tasks.md, SHALL NOT mutate `change.version`, SHALL exit with code 0, and the response `data` SHALL still return `done: true` with the current `state` and `all_tasks_done` snapshot

#### Scenario: Out-of-range index rejected

- **WHEN** the user runs `speclink task done 99 --change demo` and tasks.md has only 5 checkbox lines
- **THEN** the engine SHALL emit error code `task.index_out_of_range`, SHALL exit with code 2, and SHALL NOT mutate tasks.md or the change row

#### Scenario: Completing last task under walking-skeleton sets all_tasks_done flag

- **WHEN** the user runs `speclink task done 5 --change demo` and this completes the final remaining `[ ]` task
- **THEN** the engine SHALL rewrite line 5, SHALL set `change.all_tasks_done=1`, SHALL keep `state='in_progress'`, SHALL NOT insert a `state_transition` row (state did not change under walking-skeleton mode), and the response `data` SHALL be `{ index: 5, done: true, all_tasks_done: true, state: "in_progress", auto_transitioned: false }`

#### Scenario: Caller wrong about apply state

- **WHEN** the user runs `speclink task done 1 --change demo` against a change in `proposing`, `reviewing`, `ready`, or `archived` state
- **THEN** the engine SHALL emit error code `state.transition_invalid` with hint `task.done requires the change to be in in_progress or code_reviewing state`, SHALL exit with code 7, and SHALL NOT mutate tasks.md

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->

---
### Requirement: `speclink task undo` SHALL unmark exactly one checkbox by 1-based index and revert auto-trigger when needed

The CLI command `speclink task undo <task-index> --change <id> [--json]` SHALL parse tasks.md, SHALL locate the task at the supplied 1-based index, and SHALL atomically rewrite the file with that line's checkbox changed from `[x]` to `[ ]`. If the task is already unmarked, the operation SHALL be idempotent. If the change is currently in `code_reviewing` state, the engine SHALL first transition the change back to `in_progress` with reason `task_undo_revert` within the same SQLite transaction as the `all_tasks_done` clear. The `all_tasks_done` flag SHALL always be cleared to 0 by a successful undo.

#### Scenario: First-time undo on an already-done task

- **WHEN** the user runs `speclink task undo 1 --change demo` against a change in `in_progress` state and tasks.md line 1 is `- [x] do thing`
- **THEN** the engine SHALL rewrite line 1 to `- [ ] do thing`, SHALL set `change.all_tasks_done=0`, SHALL keep `state='in_progress'`, and the response `data` SHALL be `{ index: 1, done: false, all_tasks_done: false, state: "in_progress", reverted_from: null }`

#### Scenario: Undo reverts from code_reviewing

- **WHEN** the user runs `speclink task undo 5 --change demo` against a change in `code_reviewing` state with `all_tasks_done=1`
- **THEN** the engine SHALL transition the change from `code_reviewing` to `in_progress` with `state_transition.reason='task_undo_revert'`, SHALL set `change.all_tasks_done=0`, SHALL rewrite tasks.md line 5 to `- [ ]`, and the response `data` SHALL be `{ index: 5, done: false, all_tasks_done: false, state: "in_progress", reverted_from: "code_reviewing" }`

#### Scenario: Idempotent undo on already-unmarked task

- **WHEN** the user runs `speclink task undo 1 --change demo` against a task already `- [ ]`
- **THEN** the engine SHALL NOT rewrite tasks.md, SHALL still clear `all_tasks_done` if previously set, and SHALL return the current state in the response envelope

#### Scenario: Out-of-range index rejected

- **WHEN** the user runs `speclink task undo 99 --change demo` and tasks.md has only 5 checkbox lines
- **THEN** the engine SHALL emit error code `task.index_out_of_range`, SHALL exit with code 2, and SHALL NOT mutate tasks.md or the change row

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->

---
### Requirement: All five CLI commands SHALL emit JSON envelopes compatible with the bootstrap and A2 contract

Every command added by this slice (`apply start`, `apply pause`, `task list`, `task done`, `task undo`) SHALL emit responses in the standard envelope shape: success `{ ok: true, data, warnings, requestId }` or error `{ ok: false, error: { code, message, hint, retryable, retry_after_ms }, requestId }`. The `data` shape per command SHALL match the requirements above. The `warnings` array SHALL be empty unless a `state_transitioned` warning is appended by the auto-transition path. The `requestId` SHALL be a fresh ULID generated per invocation. Each element of the `warnings` array SHALL have shape `{ code: string, message: string, details?: object }`, where `details` is an optional structured payload that SHALL be omitted from the JSON output when the warning has no carrier-specific fields.

#### Scenario: Success envelope shape

- **WHEN** any of the five commands succeeds
- **THEN** the JSON output SHALL contain top-level keys `ok`, `data`, `warnings`, `requestId`, in exactly that field order

#### Scenario: Error envelope shape

- **WHEN** any of the five commands fails with a recognized error code
- **THEN** the JSON output SHALL contain top-level keys `ok` (value `false`), `error`, `requestId`; the `error` object SHALL contain keys `code`, `message`, `hint`, `retryable`, `retry_after_ms`; `retryable` SHALL be `false` for `state.transition_invalid`, `task.index_out_of_range`, `task.no_tasks_file`, and `change.not_found`; `retryable` SHALL be `true` for `state.version_conflict`

#### Scenario: State_transitioned warning rides along on auto-transition

- **WHEN** an `apply.start` or `task.done` invocation triggers an auto-transition path that the caller did not explicitly request (e.g. `task.done` completes last task and triggers `task_done_auto`)
- **THEN** the response `warnings` array SHALL contain an entry `{ "code": "state_transitioned", "message": "Change state advanced to <new_state>", "details": { "from": "<old>", "to": "<new>", "reason": "<reason_code>" } }`

#### Scenario: Warning without carrier details omits the `details` field

- **WHEN** a CLI command appends a warning that has no structured carrier payload (e.g. `artifact.capability_ignored` returned by `new artifact --capability` on a non-spec kind)
- **THEN** the warning JSON SHALL serialize as exactly `{ "code": "...", "message": "..." }` with no `details` key present in the object

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->

---
### Requirement: Task index stability contract SHALL be explicit in the spec and warned to users

The 1-based index used by `task.done` and `task.undo` SHALL be derived from the current document order of checkbox lines at parse time. The engine SHALL NOT persist task identifiers; reordering, inserting, or deleting checkbox lines in tasks.md between `task.list` and `task.done` calls SHALL invalidate previously-seen indices. The CLI SHALL document this limitation in `--help` output for `task done` and `task undo`. A future slice introducing HTML comment markers SHALL supersede this contract.

#### Scenario: Help text documents index instability

- **WHEN** the user runs `speclink task done --help`
- **THEN** the help output SHALL include a sentence warning that task indices are derived from current document order and SHALL be invalidated by edits to tasks.md between `task list` and `task done`

#### Scenario: Reorder between list and done targets wrong task

- **WHEN** the user runs `speclink task list` (sees task 3 as `add tests`), then manually edits tasks.md to insert a new checkbox at line 1 (shifting `add tests` to index 4), then runs `speclink task done 3`
- **THEN** the engine SHALL mark whatever task is currently at index 3 (now `add tests` is at index 4); the engine SHALL NOT detect or warn about this case in this slice

<!-- @trace
source: add-state-machine-and-apply
updated: 2026-05-22
-->