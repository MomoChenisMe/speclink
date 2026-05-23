# project-bootstrap Specification

## Purpose

TBD - created by archiving change 'add-project-bootstrap'. Update Purpose after archive.

## Requirements

### Requirement: `speclink init` initializes a SpecLink project in a git working tree

The system SHALL provide a `speclink init` command that, when invoked inside a git working tree without an existing SpecLink project, MUST create the artifact root (`.speclink/`), the state root (under the git common dir, namespace `speclink/`), an initial `link.yaml`, an initial `state.db`, and MUST append `.speclink/link.yaml` to the working tree `.gitignore`. The command MUST exit with status 0 on success.

The command MUST emit a stable JSON envelope when invoked with `--json`. The envelope shape is defined by the requirement "SpecLink CLI commands emit a stable JSON envelope" below.

#### Scenario: Fresh git repository init succeeds

- **WHEN** a user runs `speclink init` inside a freshly created git repository that has no `.speclink/` directory
- **THEN** the command MUST exit with status 0, the file `.speclink/link.yaml` MUST exist, the file under `<git-common-dir>/speclink/state.db` MUST exist, and the working tree `.gitignore` MUST contain exactly one line `.speclink/link.yaml`

##### Example: Effects of a fresh init

- **GIVEN** a tempdir `T` with `git init` executed and no existing `.gitignore`
- **WHEN** the user runs `speclink init --json` from `T`
- **THEN** the JSON output contains `ok: true`, `data.project_id` matches the UUID v4 pattern, `data.artifact_root` equals `.speclink`, `data.state_root` equals `.git/speclink`, the file `T/.speclink/link.yaml` exists, the file `T/.git/speclink/state.db` exists, the file `T/.gitignore` exists and its single line is `.speclink/link.yaml`


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink init` MUST reject non-git working directories

The `speclink init` command MUST detect whether the current working directory is inside a git working tree. When the working directory is not inside a git working tree, the command MUST NOT create any files under `.speclink/` or under the git common dir, MUST exit with status 2, and MUST emit the error code `project.requires_git`. The error envelope MUST include a `hint` field instructing the user to run `git init` first.

#### Scenario: Non-git directory init is rejected

- **WHEN** a user runs `speclink init --json` in a directory that is not inside any git working tree
- **THEN** the command MUST exit with status 2, the JSON output MUST contain `ok: false`, `error.code` MUST equal `project.requires_git`, `error.retryable` MUST be `false`, and no files MUST be created under the current directory


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink init` MUST refuse re-initialization without `--force`

When `.speclink/link.yaml` already exists, the `speclink init` command without the `--force` flag MUST exit with status 7, MUST emit the error code `project.already_initialized`, and MUST NOT modify any existing file under `.speclink/`, the git common dir state root, or the working tree `.gitignore`.

#### Scenario: Re-init without force is rejected

- **WHEN** a user runs `speclink init --json` in a directory where `.speclink/link.yaml` already exists
- **THEN** the command MUST exit with status 7, the JSON output MUST contain `error.code` equal to `project.already_initialized`, and the existing `.speclink/link.yaml` mtime MUST remain unchanged


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink init --force` MUST re-init while preserving `state.db`

When the `--force` flag is supplied, the `speclink init` command MUST overwrite `.speclink/link.yaml` with a new `instance_id` and a new `created_at` timestamp. The command MUST NOT delete or recreate the existing `state.db` file. The command MUST NOT add a duplicate `.speclink/link.yaml` entry to the working tree `.gitignore`.

#### Scenario: Force re-init rotates instance_id but preserves state.db

- **GIVEN** a project initialized at time T0 with `instance_id` equal to `I0` and an existing `state.db`
- **WHEN** a user runs `speclink init --force` at time T1 (T1 > T0)
- **THEN** the command MUST exit with status 0, the new `.speclink/link.yaml` MUST contain `instance_id` not equal to `I0`, the new `link.yaml` MUST contain `created_at` equal to T1, and the file `state.db` MUST have unchanged content (same byte length and same SHA-256)


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink status` reports project metadata for an initialized project

The system SHALL provide a `speclink status` command that, when invoked inside an initialized SpecLink project, MUST emit the project metadata and exit with status 0. The metadata MUST include `project_id`, `provider`, `artifact_root`, `state_root`, `git_head`, and `requires_git`.

#### Scenario: Status of an initialized project returns expected fields

- **WHEN** a user runs `speclink status --json` inside an initialized project on a git commit with SHA `S`
- **THEN** the command MUST exit with status 0, the JSON output MUST contain `data.project_id` matching the UUID v4 pattern, `data.provider` equal to `local`, `data.artifact_root` equal to `.speclink`, `data.state_root` equal to `.git/speclink`, `data.git_head` equal to `S`, and `data.requires_git` equal to `true`


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink status` reports `project.not_initialized` when no project exists

When `.speclink/link.yaml` does not exist in the working tree, the `speclink status` command MUST exit with status 2 and emit the error code `project.not_initialized`.

#### Scenario: Status without prior init is rejected

- **WHEN** a user runs `speclink status --json` inside a git working tree with no `.speclink/link.yaml`
- **THEN** the command MUST exit with status 2 and the JSON output MUST contain `error.code` equal to `project.not_initialized`


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink link <project_id>` binds the working directory to an existing project row

The system SHALL provide a `speclink link <project_id>` command that, when the target `project_id` matches an existing row in `state.db`, MUST create or overwrite `.speclink/link.yaml` so that its `project_id` equals the supplied argument. The command MUST exit with status 0 on success. The command MUST NOT modify `state.db`.

#### Scenario: Link to known project_id succeeds

- **GIVEN** a state.db that contains a `project` row with `id` equal to `P`
- **WHEN** a user runs `speclink link P --json`
- **THEN** the command MUST exit with status 0, the JSON output MUST contain `data.project_id` equal to `P`, and `.speclink/link.yaml` MUST contain `project_id: P`


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink link` MUST reject unknown `project_id`

When the supplied `project_id` does not match any row in `state.db`, the `speclink link` command MUST exit with status 2, MUST emit the error code `project.link_target_not_found`, and MUST NOT create or modify `.speclink/link.yaml`.

#### Scenario: Link to unknown project_id is rejected

- **GIVEN** a state.db that contains no `project` row with `id` equal to `U`
- **WHEN** a user runs `speclink link U --json`
- **THEN** the command MUST exit with status 2 and the JSON output MUST contain `error.code` equal to `project.link_target_not_found`


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink unlink` removes binding metadata but preserves state and artifacts

The system SHALL provide a `speclink unlink` command that, when invoked inside an initialized project, MUST delete `.speclink/link.yaml`. The command MUST NOT delete other files under `.speclink/` (including `schemas/`) and MUST NOT delete `state.db` or any other file under the state root. The command MUST exit with status 0 on success.

#### Scenario: Unlink keeps state.db and schemas

- **GIVEN** an initialized project with `.speclink/link.yaml`, `.speclink/schemas/spec.json`, and `.git/speclink/state.db`
- **WHEN** a user runs `speclink unlink --json`
- **THEN** the command MUST exit with status 0, `.speclink/link.yaml` MUST NOT exist, `.speclink/schemas/spec.json` MUST exist, and `.git/speclink/state.db` MUST exist


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: SpecLink CLI commands emit a stable JSON envelope

When invoked with `--json`, every SpecLink CLI command MUST emit exactly one JSON document to stdout. The document MUST conform to one of two shapes.

Success shape: an object with keys `ok` (always `true`), `data` (object, command-specific), `warnings` (array of objects with keys `code`, `message`), and `requestId` (string, UUID v4).

Error shape: an object with keys `ok` (always `false`), `error` (object with keys `code` (string), `message` (string), `hint` (string or null), `retryable` (boolean), `retry_after_ms` (integer or null)), and `requestId` (string, UUID v4).

stderr MUST NOT contain JSON when `--json` is supplied. Logs and trace output MUST be routed to stderr only when `--json` is absent.

#### Scenario: Success envelope shape

- **GIVEN** a SpecLink CLI command that completes successfully
- **WHEN** the user invokes it with `--json`
- **THEN** stdout MUST contain exactly one JSON document, the document MUST validate against the success shape, `ok` MUST equal `true`, `requestId` MUST match the UUID v4 pattern, `data` MUST be an object, and `warnings` MUST be an array

#### Scenario: Error envelope shape

- **GIVEN** a SpecLink CLI command that fails with a known error code
- **WHEN** the user invokes it with `--json`
- **THEN** stdout MUST contain exactly one JSON document, the document MUST validate against the error shape, `ok` MUST equal `false`, `error.code` MUST be a non-empty string using dot-separated namespace (for example `project.requires_git`), `error.retryable` MUST be a boolean, and `requestId` MUST match the UUID v4 pattern


<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: SpecLink CLI exit codes follow a fixed mapping

The CLI MUST map outcome categories to exit codes as follows: 0 for success; 2 for user input errors including `project.requires_git`, `project.not_initialized`, and `project.link_target_not_found`; 7 for conflict including `project.already_initialized`; 1 for unclassified internal errors. The exit code MUST be deterministic for a given error code and MUST NOT depend on whether `--json` is supplied.

#### Scenario: Exit code mapping for declared error codes

- **WHEN** a SpecLink CLI command fails with the error codes in the table below
- **THEN** the exit code MUST equal the value in the right column

##### Example: Error code to exit code mapping

| error.code                       | Exit code |
| -------------------------------- | --------- |
| project.requires_git             | 2         |
| project.not_initialized          | 2         |
| project.link_target_not_found    | 2         |
| project.already_initialized      | 7         |

<!-- @trace
source: add-project-bootstrap
updated: 2026-05-22
code:
  - crates/provider/src/error.rs
  - crates/cli/src/main.rs
  - crates/runtime/src/bootstrap.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/lib.rs
  - Cargo.toml
  - crates/provider-local/src/snapshots/speclink_provider_local__link_yaml__tests__link_yaml_v1_fixed.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/link_yaml.rs
  - crates/provider/src/types.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/ops.rs
  - crates/cli/src/output.rs
  - crates/provider-local/src/lib.rs
  - crates/cli/src/commands/link.rs
  - crates/runtime/src/git.rs
  - crates/provider-local/Cargo.toml
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/status.rs
  - crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap
  - doc/speclink-design.md
  - README.md
  - crates/cli/src/lib.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/init.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/paths.rs
  - crates/runtime/src/gitignore.rs
  - crates/provider/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/commands/unlink.rs
tests:
  - crates/cli/tests/snapshots/cli__envelope_link_failure.snap
  - crates/cli/tests/snapshots/cli__envelope_status_success.snap
  - crates/runtime/tests/worktree.rs
  - crates/cli/tests/snapshots/cli__envelope_init_non_git.snap
  - crates/cli/tests/snapshots/cli__envelope_init_success.snap
  - crates/runtime/tests/paths.rs
  - crates/cli/tests/cli.rs
  - crates/runtime/tests/bootstrap.rs
  - crates/runtime/tests/ops.rs
-->

---
### Requirement: `speclink init` MUST insert the `config_state` singleton row in the same transaction as the project row

When `speclink init` runs against a state.db that has been migrated to schema version 5, the prepare/commit phase SHALL compute the sha256, byte size, and mtime_ns of the freshly written `.speclink/config.yaml`, then insert `config_state (id=1, content_sha256=<sha>, size_bytes=<size>, mtime_ns=<mtime>, version=1, updated_at=now, written_by=NULL)` in the **same** SQLite transaction that inserts the `project` row. The transaction SHALL be all-or-nothing: if any prepare step (including config_state computation or insert) fails, no row SHALL be left in state.db and no file SHALL remain on disk under `.speclink/`.

This requirement SHALL apply to fresh-init paths only. Legacy v4-to-v5 upgrade paths SHALL rely on the `INSERT OR IGNORE` clause inside the v5 migration step (see config-rw capability) and SHALL NOT re-run the init logic.

The existing requirement "Init MUST commit artifact and state changes only after every prepare step succeeds" continues to govern the wider all-or-nothing semantics; this requirement extends it to cover the new row.

#### Scenario: Fresh init populates config_state row

- **WHEN** a user runs `speclink init` in a fresh git repository with a v5-capable binary
- **THEN** the command SHALL exit 0, `state.db` SHALL contain exactly one `project` row and exactly one `config_state` row with `id=1`, `version=1`, `content_sha256` matching `sha256(.speclink/config.yaml bytes)`, `size_bytes` matching the file size, and `written_by=NULL`

#### Scenario: Failed init leaves no config_state row

- **GIVEN** a directory where `.speclink/` cannot be created (e.g. read-only filesystem)
- **WHEN** the user runs `speclink init`
- **THEN** the command SHALL exit non-zero, no `.speclink/` directory SHALL exist, no `project` row SHALL be present in any state.db that was opened, and no `config_state` row SHALL be present either

#### Scenario: Re-init with --force preserves config_state row alignment

- **GIVEN** an already-initialized project at schema v5 with `config_state.version=N`
- **WHEN** the user runs `speclink init --force`
- **THEN** the command SHALL preserve the existing state.db (per the existing `--force` requirement), SHALL recompute the sha256 of the rewritten config.yaml, and SHALL update `config_state` to `content_sha256=<new>`, `size_bytes=<new>`, `mtime_ns=<new>`, `version=version+1`, `updated_at=now` if and only if the new bytes differ from the old sha; if the bytes match, the row SHALL NOT be touched

<!-- @trace
source: add-config-rw
updated: 2026-05-23
code:
  - crates/provider-local/src/archive_store.rs
  - crates/provider-local/src/lib.rs
  - crates/provider/Cargo.toml
  - crates/provider/src/error.rs
  - crates/provider/src/config_store.rs
  - crates/runtime/src/state_machine.rs
  - crates/runtime/src/task_ops.rs
  - doc/protocol/operations.md
  - crates/cli/src/main.rs
  - crates/cli/src/commands/task_done.rs
  - crates/provider-local/src/config_store.rs
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/change_ops.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/ops.rs
  - doc/speclink-design.md
  - crates/runtime/src/bootstrap.rs
  - crates/runtime/src/apply_ops.rs
  - crates/provider/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/config.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/runtime/src/archive_ops.rs
  - crates/runtime/src/config_ops.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider/src/jsonpath.rs
  - crates/provider-local/src/migrations/v5_config_tables.sql
  - crates/provider-local/src/artifact_store.rs
  - crates/provider-local/src/store.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/types.rs
  - crates/provider-local/src/change_store.rs
  - crates/provider-local/src/state_machine_store.rs
tests:
  - crates/runtime/tests/task_ops.rs
  - crates/provider-local/tests/migration_v5.rs
  - crates/cli/tests/init_config_state.rs
  - crates/provider/tests/config_store_trait.rs
  - crates/runtime/tests/state_machine_config.rs
  - crates/provider-local/tests/config_store.rs
  - crates/cli/tests/state_machine_e2e.rs
  - crates/provider/tests/error_codes.rs
  - crates/provider-local/tests/migration_v4.rs
  - crates/cli/tests/config_cli.rs
-->