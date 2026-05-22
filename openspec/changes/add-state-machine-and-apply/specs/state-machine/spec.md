## ADDED Requirements

### Requirement: Change lifecycle SHALL define exactly six legal states

The change lifecycle SHALL be modelled as a finite state machine with exactly six legal values for `change.state`: `proposing`, `reviewing`, `ready`, `in_progress`, `code_reviewing`, `archived`. Any other value SHALL be treated as an invariant violation; the engine SHALL emit error code `state.invalid_value` and abort the operation.

#### Scenario: Enum values are normative

- **WHEN** the engine reads a `change.state` column whose value is not one of the six legal strings
- **THEN** the engine SHALL emit error code `state.invalid_value`, SHALL exit with code 1, and SHALL NOT mutate the row

##### Example: legal vs illegal state values

| `change.state` value | Legal | Notes |
| --- | --- | --- |
| `proposing` | yes | initial state on `new change` |
| `reviewing` | yes | reached on artifact DAG complete when `require_artifact_review=true` |
| `ready` | yes | reached on artifact DAG complete when `require_artifact_review=false` OR on `apply pause` |
| `in_progress` | yes | reached on `apply start` |
| `code_reviewing` | yes | reached on `task done` auto-trigger when `require_code_review=true` |
| `archived` | yes | reserved for future archive slice; this slice never writes it |
| `Proposing` | no | case-sensitive; uppercase rejected |
| `done` | no | not in enum |
| empty string | no | not in enum |

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
| `code_reviewing` | `archived` | reserved for future archive slice (not implemented in this slice) | `archive_run` |
| `in_progress` | `archived` | reserved for future archive slice when `require_code_review=false` (not implemented in this slice) | `archive_run` |

#### Scenario: Illegal transition is rejected

- **WHEN** the engine is requested to transition from `proposing` directly to `in_progress`
- **THEN** the engine SHALL emit error code `state.transition_invalid`, SHALL exit with code 7, and SHALL NOT update the `change` row or insert any `state_transition` audit row

#### Scenario: Idempotent no-op is not a transition

- **WHEN** the engine is requested to transition from `in_progress` to `in_progress` via `apply.start`
- **THEN** the engine SHALL NOT invoke the transition path, SHALL NOT insert a `state_transition` audit row for the no-op, MAY still update `actor_json` if the requested actor differs, and SHALL return the current state view to the caller

### Requirement: State mutation SHALL be atomic with audit insert via a single SQLite transaction

Every state transition that mutates `change.state` SHALL be performed in the same SQLite transaction as the insert into `state_transition`. The two operations SHALL succeed together or roll back together. The engine SHALL NOT expose any code path that updates `change.state` without writing a corresponding `state_transition` row.

#### Scenario: Transition succeeds atomically

- **WHEN** a transition request commits successfully
- **THEN** the `change` row SHALL show the new `state` and incremented `version`, AND the `state_transition` table SHALL contain a new row with matching `from_state`, `to_state`, `actor_json`, `transitioned_at`, and `reason`

#### Scenario: Audit insert failure rolls back the state update

- **WHEN** the `state_transition` insert fails (e.g. injected disk error) after the `change` row has been updated within the same transaction
- **THEN** the SQLite transaction SHALL be rolled back, the `change` row SHALL retain its pre-transition `state` and `version`, and the engine SHALL surface the underlying I/O error wrapped as a `ProviderError`

### Requirement: `change.version` SHALL serve as the compare-and-swap token for all state-machine mutations

Every `StateMachineStore` write method SHALL accept an `expected_version` parameter and SHALL apply its mutation only if the current `change.version` equals `expected_version`. On mismatch, the engine SHALL emit error code `state.version_conflict` and exit code 7. On match, the engine SHALL increment `change.version` by 1 atomically with the mutation.

#### Scenario: Version match permits mutation

- **WHEN** caller passes `expected_version=3` and the current `change.version` is 3
- **THEN** the mutation SHALL apply and the row SHALL show `version=4` after commit

#### Scenario: Version mismatch rejects mutation

- **WHEN** caller passes `expected_version=3` and the current `change.version` is 5
- **THEN** the engine SHALL emit error code `state.version_conflict`, SHALL exit with code 7, and SHALL NOT mutate the row

#### Scenario: Version is incremented even on idempotent actor reassignment

- **WHEN** `apply.start` is called against a change already in `in_progress`, and the caller's actor differs from the stored actor
- **THEN** the engine SHALL update `actor_json` and SHALL increment `change.version` by 1; the engine SHALL NOT insert a `state_transition` row because the `state` itself did not change

### Requirement: State.db schema MUST be upgraded to version 3 with `actor_json` column, `all_tasks_done` column, and `state_transition` table

The `state.db` SQLite database SHALL be migrated forward from schema version 2 to version 3 by adding two columns to the `change` table and creating one new table `state_transition`. The migration runner SHALL record version 3 in `_migrations` and SHALL be idempotent on retry.

#### Scenario: First-time migration from v2 to v3

- **WHEN** the engine opens a `state.db` whose `_migrations` table contains versions 1 and 2 only
- **THEN** the engine SHALL execute migration v3, add columns `actor_json TEXT NULL` and `all_tasks_done INTEGER NOT NULL DEFAULT 0` to the `change` table, create the `state_transition` table with the exact columns specified below, insert a row `(version=3, applied_at=<now>)` into `_migrations`, and SHALL NOT alter any existing `change` row data

##### Example: v3 schema

The `change` table SHALL gain exactly the following columns:

| Column | Type | Constraint | Notes |
| --- | --- | --- | --- |
| `actor_json` | TEXT | NULL | JSON encoding of `Actor { agent_host, os_user, host_id }`; populated by `apply.start`, cleared by `apply.pause` |
| `all_tasks_done` | INTEGER | NOT NULL DEFAULT 0 | boolean (0/1); set to 1 when `task.done` completes the last task and `require_code_review=false`; cleared by `task.undo` |

The `state_transition` table SHALL have exactly the following columns:

| Column | Type | Constraint | Notes |
| --- | --- | --- | --- |
| `transition_id` | TEXT | PRIMARY KEY | UUID v4 |
| `change_id` | TEXT | NOT NULL REFERENCES change(change_id) | foreign key to `change` |
| `from_state` | TEXT | NOT NULL | one of the six legal state values |
| `to_state` | TEXT | NOT NULL | one of the six legal state values |
| `actor_json` | TEXT | NULL | snapshot of the actor at transition time |
| `transitioned_at` | TIMESTAMP | NOT NULL | ISO 8601 UTC |
| `reason` | TEXT | NOT NULL | one of the legal reason codes listed in the transition table requirement |

The migration SHALL also create an index `idx_state_transition_change_time` on `(change_id, transitioned_at DESC)` to support future audit query CLI.

#### Scenario: Migration is idempotent on retry

- **WHEN** the engine opens a `state.db` whose `_migrations` table already contains version 3
- **THEN** the engine SHALL detect the existing v3 row, SHALL NOT re-alter the `change` table, SHALL NOT re-create the `state_transition` table, and SHALL NOT alter any existing row data

#### Scenario: Migration leaves no partial state on failure

- **WHEN** an injected failure aborts migration v3 mid-execution
- **THEN** the partial transaction SHALL be rolled back and the next retry SHALL succeed with the same end state as a first-time migration

#### Scenario: v2 binary refuses to open a v3 database

- **WHEN** a binary that ships only migrations 1 and 2 opens a `state.db` whose `_migrations` table contains version 3
- **THEN** the binary SHALL detect `schema_version() > MIGRATIONS.len()`, SHALL emit error code `state.db.schema_invalid`, and SHALL NOT attempt any read or write against the database

### Requirement: Forward state transitions from `proposing` SHALL be triggered automatically by the `artifact.write` DAG evaluator

After every successful `artifact.write` operation, the engine SHALL invoke a DAG-completeness evaluator. If the change is currently in `proposing` state AND all required artifacts are present on the filesystem (proposal.md, tasks.md, and at least one specs/&lt;capability&gt;/spec.md), the evaluator SHALL transition the change forward via the state machine using reason code `artifact_dag_complete`. If the change is in any state other than `proposing`, the evaluator SHALL be a no-op.

#### Scenario: Auto-transition fires on DAG complete

- **WHEN** the user invokes `speclink new artifact tasks --change demo --stdin` against a change that already has `proposal.md` and `specs/auth/spec.md` written, and `require_artifact_review=false`
- **THEN** the `artifact.write` operation SHALL complete normally, the DAG evaluator SHALL fire, the change state SHALL transition from `proposing` to `ready`, the `artifact.write` response `warnings` array SHALL contain a `state_transitioned` warning describing the transition, and a `state_transition` row SHALL exist with `reason='artifact_dag_complete'`

#### Scenario: Auto-transition skipped when DAG incomplete

- **WHEN** the user writes only `proposal.md` (tasks.md and specs/* still missing)
- **THEN** the `artifact.write` operation SHALL complete normally, the DAG evaluator SHALL NOT transition the change, the state SHALL remain `proposing`, and the `warnings` array SHALL NOT contain `state_transitioned`

#### Scenario: Evaluator no-op for non-proposing states

- **WHEN** the user writes a `design.md` against a change already in `ready`, `in_progress`, `code_reviewing`, or `archived` state
- **THEN** the DAG evaluator SHALL detect the non-proposing state and SHALL skip transition; the `artifact.write` operation SHALL complete normally with no `state_transitioned` warning

### Requirement: Walking-skeleton mode SHALL hard-code both review flags to `false`

In this slice, `require_artifact_review` and `require_code_review` SHALL both be hard-coded to `false` in `crates/runtime/src/state_machine.rs` via a `ReviewPolicy::walking_skeleton()` constructor. The engine SHALL NOT read `.speclink/config.yaml` for these values in this slice. The hard-coding SHALL be replaced by config-driven values in a future slice without changing the transition table.

#### Scenario: Walking-skeleton mode skips reviewing state

- **WHEN** the DAG evaluator fires on a change in `proposing` state under walking-skeleton mode
- **THEN** the transition SHALL go directly from `proposing` to `ready`, SHALL NOT enter `reviewing`, and the `state_transition` audit row SHALL show `from_state='proposing'`, `to_state='ready'`, `reason='artifact_dag_complete'`

#### Scenario: Walking-skeleton mode skips code_reviewing state

- **WHEN** `task.done` completes the last task on a change under walking-skeleton mode
- **THEN** the engine SHALL set `all_tasks_done=1`, SHALL keep `state='in_progress'`, SHALL NOT transition to `code_reviewing`, and the response `data` SHALL include `auto_transitioned: false`, `all_tasks_done: true`, `state: "in_progress"`

### Requirement: The transition `code_reviewing → in_progress` triggered by `task.undo` SHALL precede the unmark

When `task.undo` is invoked against a change in `code_reviewing` state, the engine SHALL first transition the change back to `in_progress` (reason `task_undo_revert`) and SHALL clear `all_tasks_done` to 0 within the same SQLite transaction as the tasks.md write. If the state transition fails, the tasks.md write SHALL NOT be performed.

#### Scenario: Revert from code_reviewing on undo

- **WHEN** `task.undo` is invoked while change state is `code_reviewing`
- **THEN** the engine SHALL transition the change to `in_progress` with reason `task_undo_revert`, SHALL set `all_tasks_done=0`, SHALL write back tasks.md with the target line changed from `[x]` to `[ ]`, and the response SHALL include `reverted_from: "code_reviewing"`

#### Scenario: No revert needed for non-code_reviewing states

- **WHEN** `task.undo` is invoked while change state is `in_progress` or `ready`
- **THEN** the engine SHALL skip the transition, SHALL still clear `all_tasks_done` if previously set, SHALL write back tasks.md, and the response SHALL include `reverted_from: null`

### Requirement: New error codes SHALL be registered with stable exit codes

This slice introduces the following error codes. The CLI `output.rs::error_code_to_exit` mapping SHALL be extended for each. Code names SHALL follow the dot-separated convention from design.md §17.4.

#### Scenario: Error code mapping is exhaustive

- **WHEN** the engine emits one of the new error codes
- **THEN** the CLI SHALL exit with the mapped exit code listed below and SHALL print a hint matching the listed phrase

##### Example: error code registry

| Error code | Exit code | Hint phrase | Trigger condition |
| --- | --- | --- | --- |
| `state.invalid_value` | 1 | `change.state column contains a value outside the legal six-state enum; database corruption suspected` | engine reads an illegal `change.state` value |
| `state.transition_invalid` | 7 | `transition not permitted from current state; see legal transition table` | request violates transition table |
| `state.version_conflict` | 7 | `change row was modified by another agent; reread state and retry` | CAS mismatch on `change.version` |
| `state.db.schema_invalid` | 1 | `state.db schema version is newer than this binary supports; upgrade binary` | binary opens database whose `_migrations` max version exceeds `MIGRATIONS.len()` |
| `change.dag_incomplete` | 2 | `change is missing required artifacts; write proposal.md, tasks.md, and at least one specs/<capability>/spec.md` | reserved for future doctor slice manual override; not surfaced by any CLI in this slice |
