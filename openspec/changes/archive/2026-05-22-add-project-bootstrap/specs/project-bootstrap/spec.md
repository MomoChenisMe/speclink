## ADDED Requirements

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

### Requirement: `speclink init` MUST reject non-git working directories

The `speclink init` command MUST detect whether the current working directory is inside a git working tree. When the working directory is not inside a git working tree, the command MUST NOT create any files under `.speclink/` or under the git common dir, MUST exit with status 2, and MUST emit the error code `project.requires_git`. The error envelope MUST include a `hint` field instructing the user to run `git init` first.

#### Scenario: Non-git directory init is rejected

- **WHEN** a user runs `speclink init --json` in a directory that is not inside any git working tree
- **THEN** the command MUST exit with status 2, the JSON output MUST contain `ok: false`, `error.code` MUST equal `project.requires_git`, `error.retryable` MUST be `false`, and no files MUST be created under the current directory

### Requirement: `speclink init` MUST refuse re-initialization without `--force`

When `.speclink/link.yaml` already exists, the `speclink init` command without the `--force` flag MUST exit with status 7, MUST emit the error code `project.already_initialized`, and MUST NOT modify any existing file under `.speclink/`, the git common dir state root, or the working tree `.gitignore`.

#### Scenario: Re-init without force is rejected

- **WHEN** a user runs `speclink init --json` in a directory where `.speclink/link.yaml` already exists
- **THEN** the command MUST exit with status 7, the JSON output MUST contain `error.code` equal to `project.already_initialized`, and the existing `.speclink/link.yaml` mtime MUST remain unchanged

### Requirement: `speclink init --force` MUST re-init while preserving `state.db`

When the `--force` flag is supplied, the `speclink init` command MUST overwrite `.speclink/link.yaml` with a new `instance_id` and a new `created_at` timestamp. The command MUST NOT delete or recreate the existing `state.db` file. The command MUST NOT add a duplicate `.speclink/link.yaml` entry to the working tree `.gitignore`.

#### Scenario: Force re-init rotates instance_id but preserves state.db

- **GIVEN** a project initialized at time T0 with `instance_id` equal to `I0` and an existing `state.db`
- **WHEN** a user runs `speclink init --force` at time T1 (T1 > T0)
- **THEN** the command MUST exit with status 0, the new `.speclink/link.yaml` MUST contain `instance_id` not equal to `I0`, the new `link.yaml` MUST contain `created_at` equal to T1, and the file `state.db` MUST have unchanged content (same byte length and same SHA-256)

### Requirement: `speclink status` reports project metadata for an initialized project

The system SHALL provide a `speclink status` command that, when invoked inside an initialized SpecLink project, MUST emit the project metadata and exit with status 0. The metadata MUST include `project_id`, `provider`, `artifact_root`, `state_root`, `git_head`, and `requires_git`.

#### Scenario: Status of an initialized project returns expected fields

- **WHEN** a user runs `speclink status --json` inside an initialized project on a git commit with SHA `S`
- **THEN** the command MUST exit with status 0, the JSON output MUST contain `data.project_id` matching the UUID v4 pattern, `data.provider` equal to `local`, `data.artifact_root` equal to `.speclink`, `data.state_root` equal to `.git/speclink`, `data.git_head` equal to `S`, and `data.requires_git` equal to `true`

### Requirement: `speclink status` reports `project.not_initialized` when no project exists

When `.speclink/link.yaml` does not exist in the working tree, the `speclink status` command MUST exit with status 2 and emit the error code `project.not_initialized`.

#### Scenario: Status without prior init is rejected

- **WHEN** a user runs `speclink status --json` inside a git working tree with no `.speclink/link.yaml`
- **THEN** the command MUST exit with status 2 and the JSON output MUST contain `error.code` equal to `project.not_initialized`

### Requirement: `speclink link <project_id>` binds the working directory to an existing project row

The system SHALL provide a `speclink link <project_id>` command that, when the target `project_id` matches an existing row in `state.db`, MUST create or overwrite `.speclink/link.yaml` so that its `project_id` equals the supplied argument. The command MUST exit with status 0 on success. The command MUST NOT modify `state.db`.

#### Scenario: Link to known project_id succeeds

- **GIVEN** a state.db that contains a `project` row with `id` equal to `P`
- **WHEN** a user runs `speclink link P --json`
- **THEN** the command MUST exit with status 0, the JSON output MUST contain `data.project_id` equal to `P`, and `.speclink/link.yaml` MUST contain `project_id: P`

### Requirement: `speclink link` MUST reject unknown `project_id`

When the supplied `project_id` does not match any row in `state.db`, the `speclink link` command MUST exit with status 2, MUST emit the error code `project.link_target_not_found`, and MUST NOT create or modify `.speclink/link.yaml`.

#### Scenario: Link to unknown project_id is rejected

- **GIVEN** a state.db that contains no `project` row with `id` equal to `U`
- **WHEN** a user runs `speclink link U --json`
- **THEN** the command MUST exit with status 2 and the JSON output MUST contain `error.code` equal to `project.link_target_not_found`

### Requirement: `speclink unlink` removes binding metadata but preserves state and artifacts

The system SHALL provide a `speclink unlink` command that, when invoked inside an initialized project, MUST delete `.speclink/link.yaml`. The command MUST NOT delete other files under `.speclink/` (including `schemas/`) and MUST NOT delete `state.db` or any other file under the state root. The command MUST exit with status 0 on success.

#### Scenario: Unlink keeps state.db and schemas

- **GIVEN** an initialized project with `.speclink/link.yaml`, `.speclink/schemas/spec.json`, and `.git/speclink/state.db`
- **WHEN** a user runs `speclink unlink --json`
- **THEN** the command MUST exit with status 0, `.speclink/link.yaml` MUST NOT exist, `.speclink/schemas/spec.json` MUST exist, and `.git/speclink/state.db` MUST exist

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
