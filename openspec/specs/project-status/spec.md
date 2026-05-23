# project-status Specification

## Purpose

TBD - created by syncing change 'add-project-status'. Update Purpose after archive.

## Requirements

### Requirement: `speclink status` SHALL emit a stable envelope conforming to the project.status output schema

The `speclink status` CLI subcommand SHALL invoke the `project.status` operation and emit a JSON envelope `{ ok, data, warnings, requestId }` whose `data` object satisfies the `project.status` output schema documented in `doc/protocol/operations.md` §1389. The `data` object SHALL include exactly these fields: `provider_type`, `project_id`, `working_dir`, `current_change` (nullable), `changes_count`, `discussions_count`, `schema_active`. The operation SHALL be read-only, SHALL NOT acquire any lock, and SHALL NOT write an audit event.

#### Scenario: Successful invocation in a SpecLink project emits the full data shape

- **WHEN** the user runs `speclink status --json` inside the working tree of a SpecLink project that has `.speclink/link.yaml`
- **THEN** the command SHALL exit with status 0 AND emit a JSON envelope whose `ok` is `true` AND whose `data` contains all seven required fields with values derived from `link.yaml`, `state.db`, and `config.yaml`

##### Example: envelope shape

```json
{
  "ok": true,
  "data": {
    "provider_type": "local",
    "project_id": "speclink",
    "working_dir": "/Users/alice/repos/speclink",
    "current_change": null,
    "changes_count": { "proposing": 0, "reviewing": 0, "ready": 0, "in_progress": 0, "code_reviewing": 0, "archived": 8 },
    "discussions_count": { "active": 0, "converged": 0 },
    "schema_active": "spec-driven"
  },
  "warnings": [],
  "requestId": "<uuid-v4>"
}
```

#### Scenario: Read-only invocation SHALL NOT mutate state

- **WHEN** the user runs `speclink status --json` twice in succession against the same project
- **THEN** both invocations SHALL succeed AND the `state.db` mtime SHALL be unchanged between invocations AND no row in any state.db table SHALL have a higher `version` column value than before

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/project_ops.rs
  - crates/runtime/src/lib.rs
tests:
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/status__status_envelope.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/project_ops.rs
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

---
### Requirement: `project.status` SHALL aggregate change counts from state.db grouped by lifecycle state

The `project.status` operation SHALL populate the `changes_count` field by enumerating all rows in the `change` table and grouping them by the `state` column. The result SHALL be an object with exactly six integer fields, one per legal state value from the state-machine capability: `proposing`, `reviewing`, `ready`, `in_progress`, `code_reviewing`, `archived`. Each field SHALL be the count of rows whose `state` column equals the field name. The runtime SHALL perform this aggregation in memory after a single `list_changes()` call against the provider; the runtime SHALL NOT add a new provider trait method for this purpose.

#### Scenario: Counts reflect rows grouped by state

- **GIVEN** a `state.db` whose `change` table contains 1 row with `state='in_progress'`, 2 rows with `state='ready'`, and 8 rows with `state='archived'`
- **WHEN** the user runs `speclink status --json`
- **THEN** `data.changes_count` SHALL equal `{ "proposing": 0, "reviewing": 0, "ready": 2, "in_progress": 1, "code_reviewing": 0, "archived": 8 }`

#### Scenario: All six state buckets SHALL always be present even when empty

- **GIVEN** a fresh SpecLink project with zero `change` rows
- **WHEN** the user runs `speclink status --json`
- **THEN** `data.changes_count` SHALL equal `{ "proposing": 0, "reviewing": 0, "ready": 0, "in_progress": 0, "code_reviewing": 0, "archived": 0 }`

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/runtime/src/project_ops.rs
tests:
  - crates/runtime/tests/project_ops.rs
  - crates/cli/tests/status.rs
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

---
### Requirement: `project.status` SHALL freeze `discussions_count` at zero pending discuss-ops implementation

Until the `add-discuss-ops` capability ships discussion storage, the `project.status` operation SHALL emit `discussions_count` with the exact value `{ "active": 0, "converged": 0 }`. The two integer fields SHALL be present so that downstream consumers parsing the envelope today do not break when real counts arrive in a future slice.

#### Scenario: discussions_count is always zero in the current slice

- **WHEN** the user runs `speclink status --json` against any SpecLink project at any point during the lifetime of this requirement
- **THEN** `data.discussions_count` SHALL equal `{ "active": 0, "converged": 0 }` regardless of repository state

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/runtime/src/project_ops.rs
tests:
  - crates/runtime/tests/project_ops.rs
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

---
### Requirement: `project.status` SHALL populate `current_change` only when an in_progress change matches the current host

The `project.status` operation SHALL set `data.current_change` to `null` unless the `change` table contains at least one row whose `state` column equals `"in_progress"` AND whose `actor.host_id` field equals the current host's hostname identifier. The current host's hostname identifier SHALL be derived using the exact same resolution chain that the `apply-task-ops` capability uses when constructing an actor for `apply.start` (i.e., the value the runtime would write into `change.actor_json.host_id` if a fresh `apply.start` ran right now from this working directory). This SHALL NOT compare against `link.yaml.instance_id` or `state.db.speclink_meta.instance_id`; those are project-scoped identifiers, not host-scoped.

When multiple rows satisfy both conditions, the runtime SHALL select the row with the most recent `updated_at` value. When `current_change` is populated, it SHALL be an object with exactly three fields: `change_id` (string), `state` (string, always `"in_progress"`), and `actor` (object).

#### Scenario: No in_progress change resolves current_change to null

- **GIVEN** a `state.db` with zero rows whose `state='in_progress'`
- **WHEN** the user runs `speclink status --json`
- **THEN** `data.current_change` SHALL be `null`

#### Scenario: in_progress change owned by current host SHALL appear in current_change

- **GIVEN** a `state.db` with one row where `state='in_progress'`, `change_id='abc'`, `actor.host_id` equals the current host's hostname identifier (same value `apply.start` would produce now), and `updated_at='2026-05-23T10:00:00Z'`
- **WHEN** the user runs `speclink status --json`
- **THEN** `data.current_change` SHALL equal `{ "change_id": "abc", "state": "in_progress", "actor": <the row's actor object> }`

#### Scenario: in_progress change owned by a different host SHALL resolve current_change to null

- **GIVEN** a `state.db` with one row where `state='in_progress'` and `actor.host_id` does NOT equal the current host's hostname identifier
- **WHEN** the user runs `speclink status --json`
- **THEN** `data.current_change` SHALL be `null`

#### Scenario: in_progress change owned by the CLI invocation's host SHALL appear in current_change after a real apply.start

- **GIVEN** a SpecLink project where the user has just run `speclink apply start <name>` from this working directory (so the change row's `actor_json` was written by the same `state_machine::resolve_actor()` codepath that this op compares against)
- **WHEN** the user immediately runs `speclink status --json` from the SAME working directory on the SAME machine
- **THEN** `data.current_change.change_id` SHALL equal `<name>` AND `data.current_change.state` SHALL equal `"in_progress"` (i.e., the dogfood headline UX is reachable end-to-end via the CLI without needing extra `--host-id` flags)

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/runtime/src/project_ops.rs
  - crates/runtime/src/state_machine.rs
tests:
  - crates/runtime/tests/project_ops.rs
  - crates/cli/tests/status.rs
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

---
### Requirement: `speclink status` SHALL reject invocation outside a SpecLink project with `project.not_initialized`

When the working directory cannot resolve to a SpecLink project (i.e., no `.speclink/link.yaml` is reachable), the `speclink status` command SHALL emit an error envelope `{ ok: false, error: { code: "project.not_initialized", ... }, requestId }` and SHALL exit with status code 2. The command SHALL NOT create any files, SHALL NOT initialize any database, and SHALL NOT print a stack trace.

#### Scenario: Running status in an empty directory exits 2 with project.not_initialized

- **GIVEN** an empty temporary directory with no `.speclink/` subdirectory and no git repository
- **WHEN** the user runs `speclink status --json` with that directory as the working directory
- **THEN** the command SHALL exit with code 2 AND emit a JSON envelope where `ok` is `false` AND `error.code` equals `"project.not_initialized"`

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/project_ops.rs
tests:
  - crates/cli/tests/status.rs
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

---
### Requirement: `speclink status` human-mode output SHALL reuse the cli-human-output YAML pretty-printer

When `speclink status` is invoked without `--json`, the command SHALL render the same `data` object via the existing `cli-human-output` YAML pretty-printer. The command SHALL NOT implement a dedicated dashboard, table, or colored renderer. The YAML output SHALL be deterministic for a given input.

#### Scenario: Human-mode output is YAML, not a custom dashboard

- **WHEN** the user runs `speclink status` (no `--json` flag) inside a SpecLink project
- **THEN** stdout SHALL be a YAML document that represents the same `data` fields as the JSON envelope's `data` object AND SHALL NOT contain ANSI color escape sequences AND SHALL NOT contain ASCII box-drawing characters

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/status.rs
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

---
### Requirement: catalogue `project_status` SHALL expose both inputs and outputs schemas via describe-tools

After this capability ships, the JSON entry for `project.status` returned by `speclink describe-tools --filter project.status --full --json` SHALL include a top-level `parameters` field containing the inputs schema AND a top-level sibling field `outputs_schema` containing the outputs schema. The `parameters` SHALL equal `{ "type": "object", "additionalProperties": false, "properties": {} }` (i.e., the operation accepts no input). The `outputs_schema` SHALL declare at least the six required field names from the project.status output schema and SHALL NOT be the placeholder empty-object stub used for unimplemented operations.

The catalogue `Operation` struct SHALL expose `inputs_schema: fn() -> Value` AND `outputs_schema: fn() -> Value` as two independent function pointer fields. Every one of the 37 operations in the catalogue SHALL have BOTH fields populated; the compiler SHALL reject any `Operation` initializer that omits `outputs_schema`. Operations not yet implemented MAY use a shared empty-object stub for their `outputs_schema` until their owning SDD slice ships, but SHALL NOT leave the field unset.

The `JsonRenderer` consumed by `speclink describe-tools --format json` SHALL emit `outputs_schema` as a sibling of `parameters`. The `CopilotSdkRenderer` (format=`copilot-sdk`) SHALL NOT include `outputs_schema` (AI tool function-call descriptors are inputs-only by convention).

#### Scenario: describe-tools json output emits both parameters and outputs_schema for project.status

- **WHEN** the user runs `speclink describe-tools --filter project.status --full --json`
- **THEN** the returned content array SHALL contain exactly one element AND that element SHALL have a key `parameters` equal to `{ "type": "object", "additionalProperties": false, "properties": {} }` (with the JSON Schema `$schema` annotation preserved) AND SHALL have a key `outputs_schema` whose `required` array SHALL include every name from `["provider_type", "project_id", "working_dir", "changes_count", "discussions_count", "schema_active"]`

#### Scenario: describe-tools copilot-sdk format omits outputs_schema

- **WHEN** the user runs `speclink describe-tools --filter project.status --full --format copilot-sdk`
- **THEN** the returned content array SHALL contain exactly one element AND that element SHALL NOT have a key named `outputs_schema` AND SHALL still contain `name`, `description`, and `parameters` keys

#### Scenario: catalogue Operation struct enforces outputs_schema for every entry

- **GIVEN** the `crates/runtime/src/catalogue/mod.rs` source defines 37 `Operation` const entries
- **WHEN** the workspace is compiled
- **THEN** the compiler SHALL succeed only if every `Operation` const initializer provides a value for `outputs_schema` AND the `Operation` struct SHALL declare both `inputs_schema: fn() -> Value` and `outputs_schema: fn() -> Value` as separate non-`Option` fields

<!-- @trace
source: add-project-status
updated: 2026-05-23
code:
  - crates/runtime/src/catalogue/mod.rs
  - crates/runtime/src/catalogue/schemas.rs
  - crates/runtime/src/tool_ops/render.rs
tests:
  - crates/cli/tests/describe_tools.rs
  - crates/cli/tests/snapshots/describe_tools__describe_tools_envelope.snap
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