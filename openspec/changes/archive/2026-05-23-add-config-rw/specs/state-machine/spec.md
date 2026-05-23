## MODIFIED Requirements

### Requirement: Walking-skeleton mode SHALL hard-code both review flags to `false`

The behavior in this requirement SHALL be replaced by config-driven values. The engine SHALL read `require_artifact_review` and `require_code_review` from `Provider::config_store().read_config().value.rules` instead of hard-coding them in `crates/runtime/src/state_machine.rs`. The transition table SHALL remain unchanged.

The walking-skeleton **defaults** SHALL continue to be `require_artifact_review=false` and `require_code_review=false`, applied via `ConfigStore::read_defaults()` whenever the live config is missing or malformed (see config-rw capability). The engine SHALL NOT cache the read result across transitions; each DAG evaluator firing and each terminal `task.done` SHALL invoke `read_config()` fresh, so that a `speclink config set` taking effect mid-cycle changes the next evaluator output.

When `read_config()` returns a warning of code `config.malformed_using_defaults`, the engine SHALL pass that warning through the JSON envelope of the triggering op (`artifact.write` / `task.done`) so users see the fallback explicitly.

The previous `ReviewPolicy::walking_skeleton()` constructor SHALL be removed; a new `ReviewPolicy::from_config(&Config)` constructor SHALL replace it and SHALL be the single source of `(require_artifact_review, require_code_review)` tuples consumed by the state machine evaluator.

#### Scenario: Default config still skips reviewing state

- **WHEN** the DAG evaluator fires on a change in `proposing` state and `read_config()` returns defaults (config missing or malformed)
- **THEN** the transition SHALL go directly from `proposing` to `ready`, SHALL NOT enter `reviewing`, the `state_transition` audit row SHALL show `from_state='proposing'`, `to_state='ready'`, `reason='artifact_dag_complete'`, AND the op's JSON envelope `warnings` SHALL contain `config.malformed_using_defaults` when config was malformed (the warning SHALL NOT be emitted when config is simply absent in a fresh project)

#### Scenario: Default config still skips code_reviewing state

- **WHEN** `task.done` completes the last task on a change while `read_config()` returns defaults
- **THEN** the engine SHALL set `all_tasks_done=1`, SHALL keep `state='in_progress'`, SHALL NOT transition to `code_reviewing`, and the response `data` SHALL include `auto_transitioned: false`, `all_tasks_done: true`, `state: "in_progress"`

#### Scenario: Setting require_artifact_review=true diverts to reviewing

- **GIVEN** the user has run `speclink config set rules.require_artifact_review true`
- **WHEN** the DAG evaluator fires on a change in `proposing` state with a full artifact set
- **THEN** the transition SHALL go from `proposing` to `reviewing` (not `ready`), and the `state_transition` audit row SHALL show `to_state='reviewing'`

#### Scenario: Setting require_code_review=true holds in_progress through code_reviewing

- **GIVEN** the user has run `speclink config set rules.require_code_review true`
- **WHEN** `task.done` completes the last task on a change in `in_progress`
- **THEN** the engine SHALL transition from `in_progress` to `code_reviewing`, the `state_transition` audit row SHALL show `to_state='code_reviewing'`, and the response `data` SHALL include `auto_transitioned: true`, `all_tasks_done: true`, `state: "code_reviewing"`

#### Scenario: Mid-cycle config flip is reflected on next transition

- **GIVEN** a change in `proposing` state and `read_config()` returning defaults
- **WHEN** the user runs `speclink config set rules.require_artifact_review true` then immediately writes the final artifact triggering the DAG evaluator
- **THEN** the evaluator SHALL observe the new config value and transition to `reviewing`, demonstrating that no per-process cache holds the prior value
