# cli-instructions Specification

## Purpose

TBD - created by archiving change 'add-instructions-and-task-done'. Update Purpose after archive.

## Requirements

### Requirement: `instructions` command surface

The CLI SHALL provide a subcommand `speclink instructions <artifact>` with four accepted artifact kinds: `proposal`, `design`, `tasks`, `spec`.

- **Command form (proposal / design / tasks)**: `speclink instructions {proposal|design|tasks} --change <change-id> [--json] [--no-color] [--quiet]`
- **Command form (spec)**: `speclink instructions spec --change <change-id> --capability <capability-name> [--json] [--no-color] [--quiet]`
- **Required flags**:
  - `--change <change-id>`: kebab-case change id matching the validation rules in the `local-provider-storage` capability
  - `--capability <capability-name>`: REQUIRED when `<artifact>` is `spec`; FORBIDDEN when `<artifact>` is `proposal`, `design`, or `tasks`
- **Optional flags**: the machine-interface flags `--json`, `--no-color`, `--quiet` as defined in the `cli-machine-interface` capability
- **Stdin**: this command SHALL NOT accept stdin. If invoked with `--stdin`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

The command SHALL be side-effect-free: it SHALL NOT create, modify, or delete any files under `.speclink/`. It SHALL NOT require the artifact target file to exist yet (it returns guidance for how to write the file).

If `--capability` is provided when `<artifact>` is not `spec`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

If `--capability` is omitted when `<artifact>` is `spec`, the CLI SHALL exit with code 2 and `error.code = "artifact.missing_capability"`.

If the capability name does not match the kebab-case rules, the CLI SHALL exit with code 2 and `error.code = "artifact.invalid_capability"`.

#### Scenario: Instructions for design artifact

- **GIVEN** an existing change `demo` (created by `propose create`)
- **WHEN** the user runs `speclink instructions design --change demo --json`
- **THEN** the process exit code is 0
- **AND** the stdout JSON `data.artifactId` equals `"design"`
- **AND** the stdout JSON `data.kind` equals `"design"`
- **AND** the stdout JSON `data.outputPath` equals `.speclink/changes/demo/design.md`
- **AND** the stdout JSON `data.instruction` is a non-empty string
- **AND** the stdout JSON `data.template` is a non-empty string
- **AND** no files under `.speclink/` are created, modified, or deleted by this command

#### Scenario: Instructions for spec artifact with capability

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink instructions spec --change demo --capability user-auth --json`
- **THEN** the process exit code is 0
- **AND** the stdout JSON `data.artifactId` equals `"spec:user-auth"`
- **AND** the stdout JSON `data.kind` equals `"spec"`
- **AND** the stdout JSON `data.outputPath` equals `.speclink/changes/demo/specs/user-auth/spec.md`

#### Scenario: Spec instructions requires --capability

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink instructions spec --change demo --json` (without `--capability`)
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.missing_capability"`

#### Scenario: Design instructions rejects --capability

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink instructions design --change demo --capability foo --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "input.invalid"`


<!-- @trace
source: add-instructions-and-task-done
updated: 2026-05-20
code:
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/task.rs
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/instructions.rs
  - crates/provider-local/src/archive.rs
  - crates/cli/src/output.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/spec_delta.rs
  - crates/runtime/instructions/proposal.md
  - crates/provider/src/model.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/archive.rs
  - crates/runtime/src/status.rs
  - crates/runtime/src/instructions.rs
  - crates/cli/src/commands/archive.rs
  - crates/provider/Cargo.toml
  - crates/cli/src/exit_code.rs
  - crates/runtime/instructions/design.md
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/tasks_parser.rs
  - crates/cli/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/instructions/spec.md
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/cli.rs
  - .gitattributes
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - crates/runtime/instructions/tasks.md
  - README.md
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/task.rs
tests:
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_tasks_success.snap
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_idempotent.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/provider-local/tests/task_done_integration.rs
  - crates/provider-local/tests/instructions_integration.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/instructions.rs
  - crates/cli/tests/status.rs
  - crates/cli/tests/instructions_snapshots.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_spec_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/task_done_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/archive_snapshots.rs
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_change_not_found.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_not_found.snap
  - crates/cli/tests/task_done.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_design_success.snap
-->

---
### Requirement: `instructions` JSON output schema

On success, the `data` payload of the `--json` envelope SHALL be a JSON object with the following fields (all required):

- `artifactId` (string): `"proposal"`, `"design"`, `"tasks"`, or `format!("spec:{capability}")`
- `kind` (string): `"proposal"`, `"design"`, `"tasks"`, or `"spec"`
- `outputPath` (string): POSIX-style path relative to project base (forward slashes on all OS)
- `dependencies` (array of strings): artifact ids that must be `done` before this one is meaningful. The fixed dependency rules SHALL be:
  - `proposal`: `[]`
  - `design`: `["proposal"]`
  - `tasks`: `["proposal", "spec"]`
  - `spec:<capability>`: `["proposal"]`
- `unlocks` (array of strings): artifact ids that become unblocked once this one is done. The fixed unlock rules SHALL be:
  - `proposal`: `["design", "tasks", "spec"]`
  - `design`: `["tasks"]`
  - `tasks`: `[]`
  - `spec:<capability>`: `["tasks"]`
- `instruction` (string): the human-readable directive for writing this artifact (loaded from runtime's hardcoded instructions)
- `template` (string): markdown skeleton with headings/placeholders for the artifact
- `rules` (array of objects): artifact-specific rules; each object has `id` (string, dot-separated), `level` (string, one of `"error"` / `"warning"` / `"info"`), and `description` (string)
- `locale` (string): the locale identifier for the instruction text; SHALL be `"Traditional Chinese (繁體中文)"` in this change

The `instruction`, `template`, and `rules` content SHALL come from runtime's hardcoded instructions and SHALL NOT be empty for any of the four kinds.

`rules` array SHALL contain at least one entry per kind.

#### Scenario: Tasks instructions JSON shape

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink instructions tasks --change demo --json`
- **THEN** the stdout JSON `data` object contains all required fields
- **AND** `data.kind = "tasks"`
- **AND** `data.dependencies = ["proposal", "spec"]`
- **AND** `data.unlocks = []`
- **AND** `data.rules` array is non-empty
- **AND** every entry in `data.rules` has `id`, `level`, and `description` fields

#### Scenario: Spec artifact dependencies and unlocks

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink instructions spec --change demo --capability auth --json`
- **THEN** `data.dependencies = ["proposal"]`
- **AND** `data.unlocks = ["tasks"]`


<!-- @trace
source: add-instructions-and-task-done
updated: 2026-05-20
code:
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/task.rs
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/instructions.rs
  - crates/provider-local/src/archive.rs
  - crates/cli/src/output.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/spec_delta.rs
  - crates/runtime/instructions/proposal.md
  - crates/provider/src/model.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/archive.rs
  - crates/runtime/src/status.rs
  - crates/runtime/src/instructions.rs
  - crates/cli/src/commands/archive.rs
  - crates/provider/Cargo.toml
  - crates/cli/src/exit_code.rs
  - crates/runtime/instructions/design.md
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/tasks_parser.rs
  - crates/cli/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/instructions/spec.md
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/cli.rs
  - .gitattributes
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - crates/runtime/instructions/tasks.md
  - README.md
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/task.rs
tests:
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_tasks_success.snap
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_idempotent.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/provider-local/tests/task_done_integration.rs
  - crates/provider-local/tests/instructions_integration.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/instructions.rs
  - crates/cli/tests/status.rs
  - crates/cli/tests/instructions_snapshots.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_spec_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/task_done_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/archive_snapshots.rs
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_change_not_found.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_not_found.snap
  - crates/cli/tests/task_done.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_design_success.snap
-->

---
### Requirement: `instructions` failure mapping

The CLI SHALL map failures to exit codes and error codes as follows:

| Trigger condition                                                  | error code                       | exit code |
| ------------------------------------------------------------------ | -------------------------------- | --------- |
| Change directory does not exist                                    | `change.not_found`               | 1         |
| Change id fails kebab-case validation                              | `change.invalid_id`              | 2         |
| Spec kind missing `--capability`                                   | `artifact.missing_capability`    | 2         |
| Non-spec kind with `--capability`                                  | `input.invalid`                  | 2         |
| Capability name fails kebab-case validation                        | `artifact.invalid_capability`    | 2         |

When the change is missing, the CLI SHALL fail rather than return instructions — `instructions` is scoped to an existing change so that the `outputPath` field is meaningful.

#### Scenario: Change not found

- **GIVEN** no `.speclink/changes/missing/` directory
- **WHEN** the user runs `speclink instructions design --change missing --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "change.not_found"`

<!-- @trace
source: add-instructions-and-task-done
updated: 2026-05-20
code:
  - crates/runtime/Cargo.toml
  - crates/cli/src/commands/status.rs
  - crates/runtime/src/task.rs
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/instructions.rs
  - crates/provider-local/src/archive.rs
  - crates/cli/src/output.rs
  - crates/provider/src/error.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/spec_delta.rs
  - crates/runtime/instructions/proposal.md
  - crates/provider/src/model.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/archive.rs
  - crates/runtime/src/status.rs
  - crates/runtime/src/instructions.rs
  - crates/cli/src/commands/archive.rs
  - crates/provider/Cargo.toml
  - crates/cli/src/exit_code.rs
  - crates/runtime/instructions/design.md
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/tasks_parser.rs
  - crates/cli/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/instructions/spec.md
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/cli.rs
  - .gitattributes
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - crates/runtime/instructions/tasks.md
  - README.md
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/task.rs
tests:
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_tasks_success.snap
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_idempotent.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/provider-local/tests/task_done_integration.rs
  - crates/provider-local/tests/instructions_integration.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/cli/tests/instructions.rs
  - crates/cli/tests/status.rs
  - crates/cli/tests/instructions_snapshots.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_spec_success.snap
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/task_done_snapshots.rs
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/archive_snapshots.rs
  - crates/provider-local/tests/multi_artifact_integration.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_change_not_found.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/snapshots/task_done_snapshots__task_done_not_found.snap
  - crates/cli/tests/task_done.rs
  - crates/cli/tests/status_snapshots.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/instructions_snapshots__instructions_design_success.snap
-->