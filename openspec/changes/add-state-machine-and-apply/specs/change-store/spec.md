## MODIFIED Requirements

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
