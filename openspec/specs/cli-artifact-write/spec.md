# cli-artifact-write Specification

## Purpose

TBD - created by archiving change 'add-artifact-write-and-status'. Update Purpose after archive.

## Requirements

### Requirement: `artifact write` command surface

The CLI SHALL provide a subcommand `speclink artifact write <kind>` with three accepted kinds: `design`, `tasks`, `spec`.

- **Command form (design / tasks)**: `speclink artifact write {design|tasks} --change <change-id> --stdin [--json] [--no-color] [--quiet]`
- **Command form (spec)**: `speclink artifact write spec --change <change-id> --capability <capability-name> --stdin [--json] [--no-color] [--quiet]`
- **Required flags**:
  - `--change <change-id>`: kebab-case change id matching the validation rules in the `local-provider-storage` capability
  - `--stdin`: artifact content is read from stdin until EOF; this flag is REQUIRED for all three kinds
  - `--capability <capability-name>`: REQUIRED when `<kind>` is `spec`; FORBIDDEN when `<kind>` is `design` or `tasks`
- **Optional flags**: the machine-interface flags `--json`, `--no-color`, `--quiet` as defined in the `cli-machine-interface` capability
- **Capability name validation**: SHALL follow the same kebab-case rules as change ids — `^[a-z][a-z0-9-]{0,63}$`, no consecutive hyphens, no trailing hyphen

If `--capability` is provided when `<kind>` is `design` or `tasks`, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

If `--capability` is omitted when `<kind>` is `spec`, the CLI SHALL exit with code 2 and `error.code = "artifact.missing_capability"`.

If `--stdin` is omitted, clap SHALL produce a parse error and the CLI SHALL exit with code 2.

If the capability name does not match the validation rules, the CLI SHALL exit with code 2 and `error.code = "artifact.invalid_capability"`.

#### Scenario: Write design artifact via stdin

- **GIVEN** an existing change `demo` (with `.speclink/changes/demo/proposal.md` already present)
- **WHEN** the user runs `echo "design body" | speclink artifact write design --change demo --stdin --json`
- **THEN** the file `.speclink/changes/demo/design.md` is created with the content `design body\n`
- **AND** the process exit code is 0
- **AND** the stdout JSON `data.artifactId` equals `"design"` and `data.kind` equals `"design"`

#### Scenario: Write spec artifact requires capability flag

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `echo "spec body" | speclink artifact write spec --change demo --stdin --json` (without `--capability`)
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.missing_capability"`

#### Scenario: Capability name validation

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability Bad-Name --stdin --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.invalid_capability"`

#### Scenario: design / tasks reject --capability

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `echo "x" | speclink artifact write design --change demo --capability foo --stdin --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "input.invalid"`

##### Example: artifact write invocations

| Kind     | Invocation                                                                                | Output path                                         |
| -------- | ----------------------------------------------------------------------------------------- | --------------------------------------------------- |
| design   | `speclink artifact write design --change c --stdin --json`                                | `.speclink/changes/c/design.md`                     |
| tasks    | `speclink artifact write tasks --change c --stdin --json`                                 | `.speclink/changes/c/tasks.md`                      |
| spec     | `speclink artifact write spec --change c --capability auth --stdin --json`                | `.speclink/changes/c/specs/auth/spec.md`            |


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
### Requirement: `artifact write` JSON output schema

On success, the `data` payload of the `--json` envelope SHALL be a JSON object with the following fields (all required):

- `changeId` (string): the change id that was written to
- `artifactId` (string): for kinds `design` / `tasks` / `proposal`, equal to the kind name; for kind `spec`, equal to `format!("spec:{capability}")`
- `kind` (string): one of `"design"`, `"tasks"`, `"spec"` (lowercase)
- `path` (string): POSIX-style path of the written file, relative to the project base (e.g., `.speclink/changes/demo/design.md`). On Windows, the path SHALL use forward slashes in the JSON output regardless of the OS path separator used internally.
- `mode` (string): the resolved provider mode (in this change always `"local"`)

The envelope `warnings` array SHALL inherit `provider.not_authenticated` warning from the resolution layer when applicable (e.g., a configured remote provider with no auth, falling back to local).

#### Scenario: Spec write JSON output

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `echo "spec body" | speclink artifact write spec --change demo --capability user-auth --stdin --json`
- **THEN** the stdout JSON `data` object equals (modulo `requestId`):

  ```json
  {
    "changeId": "demo",
    "artifactId": "spec:user-auth",
    "kind": "spec",
    "path": ".speclink/changes/demo/specs/user-auth/spec.md",
    "mode": "local"
  }
  ```
- **AND** process exit code is 0


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
### Requirement: `artifact write` failure mapping

The CLI SHALL map failures to exit codes and error codes as follows:

| Trigger condition                                              | error code                       | exit code |
| -------------------------------------------------------------- | -------------------------------- | --------- |
| Change does not exist                                          | `change.not_found`               | 1         |
| Spec kind missing `--capability`                               | `artifact.missing_capability`    | 2         |
| Design or tasks kind with `--capability`                       | `input.invalid`                  | 2         |
| Capability name fails kebab-case validation                    | `artifact.invalid_capability`    | 2         |
| Target artifact file already exists                            | `artifact.already_exists`        | 1         |
| Stdin is empty (EOF immediately, no bytes read)                | `input.invalid`                  | 2         |
| Stdin contains invalid UTF-8                                   | `input.invalid`                  | 2         |
| Filesystem write failure (permissions, disk full)              | `internal.error`                 | 1         |
| State database error (e.g., schema version mismatch)           | `internal.error`                 | 1         |

The CLI SHALL NOT overwrite an existing artifact file in this change. Future changes MAY introduce a `--force` flag.

#### Scenario: Change not found

- **GIVEN** no `.speclink/changes/missing/` directory
- **WHEN** the user runs `echo "x" | speclink artifact write design --change missing --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "change.not_found"`

#### Scenario: Existing artifact is not overwritten

- **GIVEN** `.speclink/changes/demo/design.md` already exists with content `OLD`
- **WHEN** the user runs `echo "NEW" | speclink artifact write design --change demo --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.already_exists"`
- **AND** the existing `design.md` content remains `OLD`

#### Scenario: Empty stdin rejected

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `speclink artifact write design --change demo --stdin --json < /dev/null`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "input.invalid"`


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
### Requirement: `artifact write` stdin content rules

The CLI SHALL read stdin until EOF using a `BufReader` over `io::stdin()`. Stdin content SHALL be interpreted as UTF-8 text. If the bytes do not form valid UTF-8, the CLI SHALL exit with code 2 and `error.code = "input.invalid"`.

The CLI SHALL append a single trailing newline (`\n`, LF only — never CRLF) to the stdin content if and only if the last byte of stdin is not already `\n`. The CLI SHALL NOT normalize internal line endings (existing CRLF in the input is preserved). This rule SHALL apply uniformly to design, tasks, and spec kinds.

The CLI SHALL NOT impose a maximum size on stdin content in this change. (Operating system limits or filesystem limits MAY apply.)

#### Scenario: Trailing newline appended

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `printf "no newline" | speclink artifact write design --change demo --stdin --json`
- **THEN** `.speclink/changes/demo/design.md` contains exactly `no newline\n`

#### Scenario: Existing trailing newline preserved

- **GIVEN** an existing change `demo`
- **WHEN** the user runs `printf "with newline\n" | speclink artifact write design --change demo --stdin --json`
- **THEN** `.speclink/changes/demo/design.md` contains exactly `with newline\n` (one newline, not two)

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