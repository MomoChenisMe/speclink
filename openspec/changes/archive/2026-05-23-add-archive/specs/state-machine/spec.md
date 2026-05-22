## ADDED Requirements

### Requirement: Transitions to `archived` SHALL be driven exclusively by `archive.run`

The state machine SHALL accept transitions whose `to_state='archived'` ONLY when invoked by the `archive.run` operation (via `ArchiveStore::archive_change`). No other code path (including `apply.start`, `apply.pause`, `task.done`, `task.undo`, the `artifact.write` DAG evaluator, or any future review operation) SHALL set `change.state='archived'`. The `state_transition` audit row inserted for an archive transition SHALL carry `reason='archive_run'`. In this slice the only legal `from_state` for an archive transition is `in_progress`, gated by `change.all_tasks_done=1`; `code_reviewing → archived` remains reserved for the future review slice (which SHALL extend `archive.run` to accept that path conditional on review approval).

#### Scenario: archive.run is the only writer of `archived`

- **WHEN** any operation other than `archive.run` requests a transition with `to_state='archived'`
- **THEN** the engine SHALL reject the request with error code `state.transition_invalid` and exit code 7, and SHALL NOT update the `change` row

#### Scenario: archive.run from in_progress is legal

- **WHEN** `archive.run` is invoked against a change with `state='in_progress'` AND `all_tasks_done=1`
- **THEN** the state machine SHALL accept the transition, SHALL update `change.state='archived'`, and SHALL insert a `state_transition` row with `reason='archive_run'`

#### Scenario: archive.run from code_reviewing is rejected in this slice

- **WHEN** `archive.run` is invoked against a change with `state='code_reviewing'`
- **THEN** the engine SHALL reject the request with error code `state.transition_invalid` (the `code_reviewing → archived` table row is reserved for a future review slice that has not yet attached approval-gating logic)

### Requirement: `archived` state SHALL be terminal — all subsequent `apply.*` and `task.*` operations SHALL be rejected or returned as hints without mutation

Once `change.state='archived'`, the engine SHALL treat the change as a terminal record. Any subsequent invocation of `apply.start`, `apply.pause`, `task.done`, or `task.undo` against the archived change SHALL conform to the behavior already specified by `add-state-machine-and-apply` (e.g., `apply.start` returns a hint envelope with `data.message='Change is archived.'` and exit code 0; `apply.pause` rejects with `state.transition_invalid` and exit code 7; `task.done` / `task.undo` reject with `state.transition_invalid`). This slice activates those previously-unreachable scenarios; the wording in `add-state-machine-and-apply` SHALL NOT change, but archive-driven `archived` state SHALL be the trigger that exercises them in end-to-end tests.

#### Scenario: apply.start on archived returns hint

- **WHEN** `archive` has driven a change to `archived` AND the user runs `speclink apply start <change-id>`
- **THEN** the engine SHALL exit with code 0, SHALL NOT mutate the change row, and the response `data.message` SHALL equal `Change is archived.`

#### Scenario: task.done on archived is rejected

- **WHEN** `archive` has driven a change to `archived` AND the user runs `speclink task done 1 --change <change-id>`
- **THEN** the engine SHALL exit with code 7 and SHALL emit error code `state.transition_invalid`

#### Scenario: task.undo on archived is rejected

- **WHEN** `archive` has driven a change to `archived` AND the user runs `speclink task undo 1 --change <change-id>`
- **THEN** the engine SHALL exit with code 7 and SHALL emit error code `state.transition_invalid`

#### Scenario: apply.pause on archived is rejected

- **WHEN** `archive` has driven a change to `archived` AND the user runs `speclink apply pause <change-id>`
- **THEN** the engine SHALL exit with code 7 and SHALL emit error code `state.transition_invalid`

### Requirement: `StateTransitionReason` enum SHALL include `ArchiveRun`

The enum `StateTransitionReason` in `crates/provider/src/types.rs` SHALL include the variant `ArchiveRun` with serialized string form `archive_run`. This variant SHALL be persisted in the `state_transition.reason` column of state.db and SHALL be returned by any future query path. The variant SHALL NOT be used by any code path other than `LocalArchiveStore::archive_change` (or its future revert path, which uses a separate variant `ArchiveRunRevert`).

#### Scenario: archive_run reason is persisted

- **WHEN** `archive.run` successfully transitions a change
- **THEN** the inserted `state_transition` row SHALL have `reason='archive_run'`

#### Scenario: archive_run_revert reason is used only on failed-rename revert

- **WHEN** the SQLite commit succeeds but the subsequent filesystem rename fails and a revert transaction runs
- **THEN** the compensating `state_transition` row SHALL have `reason='archive_run_revert'`; this variant SHALL be added alongside `ArchiveRun` and SHALL serialize as `archive_run_revert`

## MODIFIED Requirements

### Requirement: State machine SHALL enforce the legal transition table

The state machine SHALL accept only the transitions listed below. Any other transition request SHALL be rejected with error code `state.transition_invalid` and exit code 7. The transition table SHALL be hard-coded in `crates/runtime/src/state_machine.rs` and SHALL NOT be configurable at runtime.

#### Scenario: Legal forward transitions

- **WHEN** the engine is requested to transition `change.state` from `from` to `to` and `(from, to)` matches a row in the table below
- **THEN** the engine SHALL permit the transition

##### Example: legal transition table

| From | To | Trigger source | Reason code |
| --- | --- | --- | --- |
| `proposing` | `reviewing` | `artifact.write` hook when DAG complete AND `require_artifact_review=true` | `artifact_dag_complete` |
| `proposing` | `ready` | `artifact.write` hook when DAG complete AND `require_artifact_review=false` | `artifact_dag_complete` |
| `reviewing` | `ready` | reserved for future review slice (not implemented in this slice) | `review_approved_artifact` |
| `ready` | `in_progress` | `apply.start` | `apply_start` |
| `in_progress` | `ready` | `apply.pause` | `apply_pause` |
| `in_progress` | `code_reviewing` | `task.done` auto-trigger when last task done AND `require_code_review=true` | `task_done_auto` |
| `code_reviewing` | `in_progress` | `task.undo` OR reserved for future review slice (reject re-entry) | `task_undo_revert` OR `review_rejected_code` |
| `code_reviewing` | `archived` | reserved for future review slice (review approval AND archive.run) | `archive_run` |
| `in_progress` | `archived` | `archive.run` when `require_code_review=false` AND `change.all_tasks_done=1` (implemented by `add-archive`) | `archive_run` |
| `archived` | `in_progress` | best-effort revert path inside `LocalArchiveStore::archive_change` when post-commit filesystem rename fails (implemented by `add-archive`) | `archive_run_revert` |

#### Scenario: Illegal transition is rejected

- **WHEN** the engine is requested to transition from `proposing` directly to `in_progress`
- **THEN** the engine SHALL emit error code `state.transition_invalid`, SHALL exit with code 7, and SHALL NOT update the `change` row or insert any `state_transition` audit row

#### Scenario: Idempotent no-op is not a transition

- **WHEN** the engine is requested to transition from `in_progress` to `in_progress` via `apply.start`
- **THEN** the engine SHALL NOT invoke the transition path, SHALL NOT insert a `state_transition` audit row for the no-op, MAY still update `actor_json` if the requested actor differs, and SHALL return the current state view to the caller

#### Scenario: in_progress to archived is legal when all_tasks_done is set

- **WHEN** the engine is requested to transition from `in_progress` to `archived` via `archive.run` AND `change.all_tasks_done=1`
- **THEN** the engine SHALL permit the transition, SHALL insert a `state_transition` row with `reason='archive_run'`, and SHALL NOT require any code review approval (review slice is responsible for the `code_reviewing → archived` path separately)

#### Scenario: archived to in_progress is permitted only as a revert path

- **WHEN** the engine inserts a compensating `state_transition` with `reason='archive_run_revert'` because a post-commit filesystem rename failed
- **THEN** the engine SHALL permit the reverse transition with `from='archived'` and `to='in_progress'`; no other caller SHALL produce this row
