# cli-status Specification

## Purpose

TBD - created by archiving change 'add-artifact-write-and-status'. Update Purpose after archive.

## Requirements

### Requirement: `status` command surface

The CLI SHALL provide a subcommand `speclink status` with the following surface:

- **Command form**: `speclink status --change <change-id> [--json] [--no-color] [--quiet]`
- **Required flags**:
  - `--change <change-id>`: the kebab-case change id to inspect; validation rules follow the `local-provider-storage` capability
- **Optional flags**: the machine-interface flags `--json`, `--no-color`, `--quiet` as defined in the `cli-machine-interface` capability
- **Stdin**: this command SHALL NOT accept stdin. If invoked with `--stdin`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

The command SHALL be side-effect-free: it SHALL NOT create, modify, or delete any files under `.speclink/`. It MAY open the SQLite database in read-only mode but SHALL NOT mutate it.

#### Scenario: Status of a fresh change

- **GIVEN** the user just ran `speclink propose create --change demo --summary "test" --json`
- **WHEN** the user runs `speclink status --change demo --json`
- **THEN** the process exit code is 0
- **AND** the stdout JSON `data.changeId` equals `"demo"`
- **AND** the stdout JSON `data.state` equals `"proposed"`
- **AND** the `data.artifacts` array contains exactly 3 entries with ids `"proposal"`, `"design"`, `"tasks"`
- **AND** the `proposal` entry has `status: "done"`
- **AND** the `design` and `tasks` entries have `status: "missing"`

#### Scenario: Status reports spec artifacts

- **GIVEN** a change `demo` with `proposal.md`, `design.md`, and two spec capabilities (`auth` and `billing`)
- **WHEN** the user runs `speclink status --change demo --json`
- **THEN** the `data.artifacts` array contains 5 entries
- **AND** the entries appear in this order: `proposal`, `design`, `tasks`, `spec:auth`, `spec:billing`
- **AND** the spec entries are sorted lexicographically by capability name


<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->

---
### Requirement: `status` JSON output schema

On success, the `data` payload of the `--json` envelope SHALL be a JSON object with the following fields (all required):

- `changeId` (string): the change id queried
- `state` (string): the lifecycle state from `metadata.json` (e.g., `"proposed"`)
- `artifacts` (array of objects): the artifact list, see `ArtifactStatus` schema below

`ArtifactStatus` object schema (all fields required):

- `id` (string): the artifact identifier — `"proposal"`, `"design"`, `"tasks"`, or `format!("spec:{capability}")` for spec artifacts
- `kind` (string): one of `"proposal"`, `"design"`, `"tasks"`, `"spec"`
- `path` (string): POSIX-style path relative to project base (forward slashes on all OS)
- `status` (string): `"done"` (file exists) or `"missing"` (file does not exist)
- `required` (boolean): whether this artifact must be present for the change to be considered complete (see Required Rules below)
- `dependencies` (array of strings): ids of artifacts that must be `done` before this one is meaningful (see Dependency Rules below)

**Artifact ordering**: the array SHALL be ordered as follows:

1. `proposal` (always first if present)
2. `design`
3. `tasks`
4. Spec artifacts (`spec:<capability>`) sorted ascending by capability name

**Required Rules** (this change, may evolve later):

- `proposal`: `required = true`
- `design`: `required = false`
- `tasks`: `required = false`
- `spec:<capability>`: `required = true` (at least one spec is required for any non-trivial change)

**Dependency Rules** (this change, may evolve later):

- `proposal`: `dependencies = []`
- `design`: `dependencies = ["proposal"]`
- `tasks`: `dependencies = ["proposal", "spec"]` (the literal string `"spec"`, meaning at least one spec must be done)
- `spec:<capability>`: `dependencies = ["proposal"]`

The CLI SHALL NOT compute or report whether dependencies are satisfied in this change. Computation of `ready` / `blocked` derived states is deferred to the `instructions` capability (future change).

#### Scenario: artifacts array structure

- **GIVEN** a change `demo` with proposal, design, and `spec:auth` present
- **WHEN** the user runs `speclink status --change demo --json`
- **THEN** the `data.artifacts` array equals (modulo `requestId` in envelope):

  ```json
  [
    {"id":"proposal","kind":"proposal","path":".speclink/changes/demo/proposal.md","status":"done","required":true,"dependencies":[]},
    {"id":"design","kind":"design","path":".speclink/changes/demo/design.md","status":"done","required":false,"dependencies":["proposal"]},
    {"id":"tasks","kind":"tasks","path":".speclink/changes/demo/tasks.md","status":"missing","required":false,"dependencies":["proposal","spec"]},
    {"id":"spec:auth","kind":"spec","path":".speclink/changes/demo/specs/auth/spec.md","status":"done","required":true,"dependencies":["proposal"]}
  ]
  ```


<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->

---
### Requirement: `status` failure mapping

The CLI SHALL map failures to exit codes and error codes as follows:

| Trigger condition                                            | error code              | exit code |
| ------------------------------------------------------------ | ----------------------- | --------- |
| Change directory does not exist                              | `change.not_found`      | 1         |
| Change id fails kebab-case validation                        | `change.invalid_id`     | 2         |
| `metadata.json` exists but is malformed                      | `internal.error`        | 1         |
| Filesystem read failure (permissions)                        | `internal.error`        | 1         |
| State database open failure                                  | `internal.error`        | 1         |

The CLI SHALL NOT treat missing optional artifacts (design, tasks, or zero spec capabilities) as a failure — they SHALL appear as `status: "missing"` in the array, with exit code 0.

#### Scenario: Change not found

- **GIVEN** no `.speclink/changes/missing/` directory
- **WHEN** the user runs `speclink status --change missing --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "change.not_found"`

#### Scenario: Malformed metadata

- **GIVEN** `.speclink/changes/broken/metadata.json` contains invalid JSON
- **WHEN** the user runs `speclink status --change broken --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "internal.error"`

<!-- @trace
source: add-artifact-write-and-status
updated: 2026-05-20
code:
  - crates/provider/src/error.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/output.rs
  - crates/runtime/src/artifact.rs
  - crates/runtime/src/status.rs
  - crates/provider-local/src/storage.rs
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/propose.rs
  - crates/provider/src/lib.rs
  - README.md
  - crates/runtime/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/main.rs
tests:
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/status.rs
-->