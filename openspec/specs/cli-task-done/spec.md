# cli-task-done Specification

## Purpose

TBD - created by archiving change 'add-instructions-and-task-done'. Update Purpose after archive.

## Requirements

### Requirement: `task done` command surface

The CLI SHALL provide a subcommand `speclink task done <task-id>` with the following surface:

- **Command form**: `speclink task done <task-id> --change <change-id> [--json] [--no-color] [--quiet]`
- **Positional argument**:
  - `<task-id>`: a string matching the pattern `^\d+\.\d+$` (decimal section number, dot, decimal task number; no leading zeros, no trailing characters)
- **Required flags**:
  - `--change <change-id>`: the kebab-case change id; validation rules follow the `local-provider-storage` capability
- **Optional flags**: the machine-interface flags `--json`, `--no-color`, `--quiet`
- **Stdin**: this command SHALL NOT accept stdin. If invoked with `--stdin`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

The CLI SHALL nest `task done` under a `task` namespace so that future subcommands (`task list`, `task undone`, etc.) can be added without breaking the existing surface.

#### Scenario: Mark a pending task as done

- **GIVEN** an existing change `demo` with `.speclink/changes/demo/tasks.md` containing `- [ ] 1.1 Write tests`
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the process exit code is 0
- **AND** the file `tasks.md` is updated so the line is `- [x] 1.1 Write tests`
- **AND** the stdout JSON `data.previousStatus` equals `"todo"`
- **AND** the stdout JSON `data.currentStatus` equals `"done"`
- **AND** the stdout JSON `data.taskDescription` equals `"Write tests"`

#### Scenario: Idempotent on already-done task

- **GIVEN** an existing change `demo` with `tasks.md` containing `- [x] 1.1 Write tests`
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the process exit code is 0
- **AND** the file `tasks.md` content is unchanged
- **AND** the stdout JSON `data.previousStatus` equals `"done"`
- **AND** the stdout JSON `data.currentStatus` equals `"done"`

#### Scenario: Task id format validation

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink task done 1.1.2 --change demo --json` (three-level id)
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "task.invalid_id"`


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
### Requirement: `task done` JSON output schema

On success, the `data` payload of the `--json` envelope SHALL be a JSON object with the following fields (all required):

- `changeId` (string): the change id
- `taskId` (string): the task id that was marked done (echoed from input)
- `previousStatus` (string): one of `"todo"` or `"done"` — the status before this command ran
- `currentStatus` (string): always `"done"` on success
- `taskDescription` (string): the task description from the tasks.md line, with the `- [ ] N.M ` or `- [x] N.M ` prefix removed and any leading `[P]` marker preserved as part of the description

The CLI SHALL preserve the `[P]` parallel marker (if present) verbatim in `taskDescription`. The CLI SHALL NOT interpret or strip the marker — that is the responsibility of future `apply` commands.

#### Scenario: Task description preserves [P] marker

- **GIVEN** an existing change `demo` with `tasks.md` containing `- [ ] 2.3 [P] Refactor parser`
- **WHEN** the user runs `speclink task done 2.3 --change demo --json`
- **THEN** the stdout JSON `data.taskDescription` equals `"[P] Refactor parser"`
- **AND** the file `tasks.md` line is updated to `- [x] 2.3 [P] Refactor parser`

#### Scenario: Task description with multi-paragraph notes

- **GIVEN** an existing change `demo` with `tasks.md` containing `- [ ] 1.1 Write tests\n  Additional notes here`
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the stdout JSON `data.taskDescription` equals `"Write tests"` (only the first line, not the continuation)


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
### Requirement: `task done` failure mapping

The CLI SHALL map failures to exit codes and error codes as follows:

| Trigger condition                                                  | error code              | exit code |
| ------------------------------------------------------------------ | ----------------------- | --------- |
| Change directory does not exist                                    | `change.not_found`      | 1         |
| Change id fails kebab-case validation                              | `change.invalid_id`     | 2         |
| `tasks.md` does not exist                                          | `artifact.missing`      | 1         |
| Task id fails `\d+\.\d+` format check                              | `task.invalid_id`       | 2         |
| Task id not found in `tasks.md`                                    | `task.not_found`        | 2         |
| `tasks.md` parsing fails (missing section heading, malformed)      | `tasks.parse_error`     | 1         |
| Filesystem write failure                                           | `internal.error`        | 1         |

`artifact.missing` SHALL be returned (not `change.not_found`) when the change directory exists but `tasks.md` is absent, because the change is valid but the user has not yet written tasks.

#### Scenario: tasks.md missing

- **GIVEN** an existing change `demo` with no `tasks.md` file
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.missing"`

#### Scenario: Task id not found

- **GIVEN** an existing change `demo` with `tasks.md` containing only `## 1. Section\n\n- [ ] 1.1 First task`
- **WHEN** the user runs `speclink task done 1.2 --change demo --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "task.not_found"`

#### Scenario: tasks.md missing section heading

- **GIVEN** an existing change `demo` with `tasks.md` containing `- [ ] 1.1 Floating task` (no `## 1.` section heading)
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "tasks.parse_error"`


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
### Requirement: `task done` atomic update

The CLI SHALL update `tasks.md` atomically using the same `.tmp` + rename pattern established by `Atomic artifact write with metadata pairing` in the `local-provider-storage` capability. Specifically:

1. Read the entire content of `<change-dir>/tasks.md` into memory
2. Locate the matching `- [ ] N.M ` or `- [x] N.M ` line by parsing the file line-by-line and tracking the current section heading (`^## N\. `)
3. If found and currently `- [ ]`, change to `- [x]` in memory (only that line; all other content preserved verbatim including line endings, whitespace, and trailing content)
4. Write modified content to `<change-dir>/tasks.md.tmp`
5. Rename `tasks.md.tmp` to `tasks.md`
6. On any write or rename failure, remove `tasks.md.tmp` if it exists; the original `tasks.md` SHALL remain unchanged

The CLI SHALL NOT normalize line endings, trim trailing whitespace, or reformat the file in any way other than flipping the single checkbox character.

When `previousStatus == Done` (idempotent case), the CLI MAY skip the write entirely (no `.tmp` created); the JSON output SHALL still report `currentStatus = "done"`.

#### Scenario: File content preservation on update

- **GIVEN** an existing change `demo` with `tasks.md` containing `## 1. Section\n\n- [ ] 1.1 Task\nExtra text\n  Continuation\n`
- **WHEN** the user runs `speclink task done 1.1 --change demo --json`
- **THEN** the resulting `tasks.md` content is `## 1. Section\n\n- [x] 1.1 Task\nExtra text\n  Continuation\n`
- **AND** no `tasks.md.tmp` remains in the change directory

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