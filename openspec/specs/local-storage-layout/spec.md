# local-storage-layout Specification

## Purpose

TBD - created by archiving change 'add-project-bootstrap'. Update Purpose after archive.

## Requirements

### Requirement: Artifact root MUST be located at `.speclink/` in the working tree

The artifact root MUST be the directory `.speclink/` immediately under the working tree root. The artifact root MUST be tracked by git, except for the file `.speclink/link.yaml` which MUST be ignored (see the `.gitignore` policy requirement below). The artifact root MUST contain `link.yaml` (binding metadata) and `schemas/` (the schema files seeded at init time). Future capability changes MAY add subdirectories such as `changes/` under the artifact root; this requirement does not restrict their addition.

#### Scenario: Init populates the artifact root with link.yaml and schemas

- **WHEN** a user runs `speclink init` in a fresh git repository
- **THEN** the directory `.speclink/` MUST exist directly under the working tree root, the file `.speclink/link.yaml` MUST exist, and the directory `.speclink/schemas/` MUST exist


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
### Requirement: State root MUST be located under the git common directory with namespace `speclink/`

The state root MUST be located at `<git-common-dir>/speclink/`, where `<git-common-dir>` is the value returned by `git rev-parse --git-common-dir`. The state root MUST NOT be derived by string-concatenating `.git/speclink` to the working tree root. The state root MUST contain `state.db`. The state root MUST contain an empty directory `locks/` reserved for future use.

#### Scenario: Init creates state root under git common dir

- **GIVEN** a git working tree at `T` whose `git rev-parse --git-common-dir` returns `T/.git`
- **WHEN** a user runs `speclink init`
- **THEN** the directory `T/.git/speclink/` MUST exist, the file `T/.git/speclink/state.db` MUST exist, and the directory `T/.git/speclink/locks/` MUST exist


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
### Requirement: State root MUST resolve to the main git directory in a linked worktree

When SpecLink commands run inside a linked worktree (created by `git worktree add`), the state root MUST resolve to the state root under the main repository's git directory, not under the linked worktree's `.git` file. Path resolution MUST use `git rev-parse --git-common-dir` so that this behavior follows from a single algorithm rather than from worktree-specific handling.

#### Scenario: Status in a linked worktree resolves to main state root

- **GIVEN** an initialized SpecLink project at main working tree `M`, and a linked worktree `W` created by `git worktree add` from `M`
- **WHEN** a user runs `speclink status --json` from inside `W`
- **THEN** the JSON output MUST contain `data.state_root` whose absolute resolution equals the absolute resolution of `M/.git/speclink`, regardless of `W`'s own `.git` file contents


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
### Requirement: Path resolution algorithm MUST shell out to `git rev-parse --git-common-dir`

The runtime MUST resolve the state root by spawning the `git` executable with arguments `rev-parse --git-common-dir`, capturing stdout, trimming whitespace, and joining with `speclink/`. The runtime MUST NOT parse the contents of the `.git` file itself. When the `git` executable is not available or fails with non-zero exit, the runtime MUST treat this as the `project.requires_git` error condition (see the `project-bootstrap` spec).

#### Scenario: Missing git executable raises requires_git

- **GIVEN** a working tree where the `git` executable cannot be invoked (for example, `git` is not on PATH)
- **WHEN** a user runs `speclink init --json`
- **THEN** the command MUST exit with status 2 and the JSON output MUST contain `error.code` equal to `project.requires_git`


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
### Requirement: `.gitignore` policy MUST be a single line for `.speclink/link.yaml`

The `speclink init` command MUST ensure the working tree `.gitignore` contains exactly one line whose content is `.speclink/link.yaml`. The command MUST NOT add additional ignore entries for `.speclink/state.db`, `.speclink/cache`, `.speclink/locks`, or any other path. The command MUST NOT create `.gitignore` entries for paths under `.git/speclink/`, because `.git/` is already excluded from the working tree by git itself.

When `.gitignore` does not exist, the command MUST create it with that single line. When `.gitignore` exists, the command MUST append the line at the end if and only if the file does not already contain an exact match for that line. The match comparison MUST be line-based and exact (no globbing, no whitespace tolerance).

#### Scenario: Append to existing gitignore without duplication

- **GIVEN** a working tree whose `.gitignore` already contains a single line `node_modules`
- **WHEN** a user runs `speclink init`
- **THEN** the resulting `.gitignore` MUST contain two lines in this order: `node_modules`, `.speclink/link.yaml`

#### Scenario: Create gitignore when missing

- **GIVEN** a working tree without a `.gitignore` file
- **WHEN** a user runs `speclink init`
- **THEN** the resulting `.gitignore` MUST exist and MUST contain exactly one line `.speclink/link.yaml`

#### Scenario: Force re-init does not duplicate the gitignore line

- **GIVEN** an initialized project where `.gitignore` already contains `.speclink/link.yaml`
- **WHEN** a user runs `speclink init --force`
- **THEN** the resulting `.gitignore` MUST contain exactly one occurrence of `.speclink/link.yaml`


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
### Requirement: `link.yaml` MUST follow a versioned YAML schema

The file `.speclink/link.yaml` MUST be valid YAML and MUST contain the top-level keys `version`, `project_id`, `instance_id`, `provider`, `created_at`, and `working_dir_fingerprint`. The key `version` MUST equal the integer `1` for the schema defined by this requirement. The key `project_id` MUST be a UUID v4 string. The key `instance_id` MUST be a UUID v4 string. The key `provider` MUST be the literal string `local`. The key `created_at` MUST be an RFC 3339 timestamp. The key `working_dir_fingerprint` MUST be a SHA-256 hex digest of the canonicalized absolute path of the working tree root.

#### Scenario: Init writes link.yaml in the version 1 schema

- **WHEN** a user runs `speclink init` in a fresh git repository
- **THEN** the file `.speclink/link.yaml` MUST be valid YAML, `version` MUST equal `1`, `project_id` MUST match the UUID v4 pattern, `instance_id` MUST match the UUID v4 pattern, `provider` MUST equal `local`, `created_at` MUST match the RFC 3339 pattern, and `working_dir_fingerprint` MUST be a 64-character lowercase hex string


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
### Requirement: `state.db` MUST be initialized at schema version 1 with the prescribed tables

The file `state.db` MUST be a SQLite database opened in WAL journal mode. After initialization, the database MUST contain the tables `_migrations` and `project`. The table `_migrations` MUST contain exactly one row with `version` equal to `1`. The table `project` MUST contain exactly one row whose `id` equals the `project_id` written into `link.yaml`. The schema MUST follow the column definitions below.

`_migrations`:

| Column      | Type    | Constraints   |
| ----------- | ------- | ------------- |
| version     | INTEGER | PRIMARY KEY   |
| applied_at  | TEXT    | NOT NULL      |

`project`:

| Column       | Type | Constraints   |
| ------------ | ---- | ------------- |
| id           | TEXT | PRIMARY KEY   |
| instance_id  | TEXT | NOT NULL      |
| working_dir  | TEXT | NOT NULL      |
| created_at   | TEXT | NOT NULL      |

#### Scenario: Initialized state.db contains v1 schema and one project row

- **WHEN** a user runs `speclink init` in a fresh git repository
- **THEN** opening `state.db` via SQLite MUST report `journal_mode` equal to `wal`, querying `SELECT version FROM _migrations` MUST return a single row with value `1`, and querying `SELECT count(*) FROM project` MUST return `1`


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
### Requirement: `speclink init` inside a linked worktree MUST share the main repo's `project_id`

When `speclink init` runs inside a working directory whose state root already contains a `state.db` with exactly one `project` row (a "preexisting state"), the command MUST adopt that row's `id` as the new `link.yaml`'s `project_id` rather than generating a fresh UUID. The command MUST NOT insert an additional `project` row into the preexisting `state.db`. The command MUST still rotate `instance_id` and refresh `created_at` for the new working directory's `link.yaml`. This requirement ensures that a main worktree and any linked worktree created via `git worktree add` remain bound to the same SpecLink project, since both share the same git common dir and therefore the same `state.db`.

When the preexisting `state.db` contains zero or more than one `project` row, the behavior is unspecified by this change (deferred to a future multi-project capability).

#### Scenario: Init in a linked worktree adopts main repo's project_id

- **GIVEN** a main working tree `M` initialized as a SpecLink project with `project_id` equal to `P`, and a linked worktree `W` created via `git worktree add` from `M`
- **WHEN** a user runs `speclink init` from inside `W`
- **THEN** the command MUST exit with status 0, the file `W/.speclink/link.yaml` MUST contain `project_id: P`, the file `M/.git/speclink/state.db` MUST still contain exactly one `project` row whose `id` equals `P`, and `W/.speclink/link.yaml`'s `instance_id` MUST differ from `M/.speclink/link.yaml`'s `instance_id`


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
### Requirement: `state_root` field in command output MUST be a clean path with no leading double slash

When SpecLink CLI commands report the `state_root` field (in `init` success, `status`, or `link` success envelopes), the field's string value MUST NOT start with the substring `//`. When the state root lies inside the working directory the field MUST use a working-directory-relative path with POSIX-style `/` separators. When the state root lies outside the working directory (such as in a linked worktree pointing to the main repo's git common dir), the field MUST use the canonical absolute path with exactly one leading `/` on POSIX or a drive-letter prefix on Windows.

#### Scenario: state_root in linked worktree has no leading double slash

- **GIVEN** a main working tree `M` initialized as a SpecLink project, and a linked worktree `W` created via `git worktree add`
- **WHEN** a user runs `speclink init --json` from inside `W`
- **THEN** the JSON output's `data.state_root` MUST be a non-empty string, MUST NOT start with the substring `//`, and MUST contain the absolute path to `M/.git/speclink` (in canonical form)


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
### Requirement: Init MUST commit artifact and state changes only after every prepare step succeeds

The `speclink init` command MUST perform schema seeding, state.db migration, and `link.yaml` generation in a staging area (such as a temporary directory or a transactional database connection) before any of these are made visible at their final locations. When any prepare step fails, the command MUST clean up the staging area and MUST NOT leave partial files at `.speclink/link.yaml`, `.speclink/schemas/`, or the state root.

#### Scenario: Failure during prepare leaves no partial files

- **GIVEN** a fresh git working tree and an injected failure during state.db migration (for example, a simulated I/O error)
- **WHEN** a user runs `speclink init`
- **THEN** the command MUST exit with non-zero status, the file `.speclink/link.yaml` MUST NOT exist, the directory `.speclink/schemas/` MUST NOT exist, and the file `state.db` under the state root MUST NOT exist

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
### Requirement: `state.db` schema MUST be upgraded to version 5 with the `config_state` and `config_change` tables

The state.db migration sequence SHALL include a v5 step that creates two tables (`config_state` singleton row keyed on `id=1`, and `config_change` autoincrement audit table) and inserts a row into `_migrations` with `version=5`. The v5 schema SHALL be additive: the `project`, `change`, and `state_transition` tables defined by earlier migrations SHALL NOT be altered, dropped, or renamed by v5.

The complete column-level schema is normative and is defined in the `config-rw` capability spec under the requirement "state.db SHALL be upgraded to version 5 with `config_state` and `config_change` tables". This requirement in the `local-storage-layout` capability SHALL stand as a sibling pointer ensuring the layout-level contract (which tables exist after a v5 migration) is also captured in the storage spec, paralleling how v3 / v4 migrations were anchored.

Migration v5 SHALL be forward-only; no downgrade SHALL be supported.

When a state.db at schema version 4 is opened by a v5-capable binary, the migration runner SHALL apply v5 atomically: either both tables exist and `_migrations` records `5`, or none of the above changes have been applied.

#### Scenario: v5 migration produces additive schema

- **GIVEN** a state.db at schema version 4 with at least one `project` row, one `change` row, and one `state_transition` row
- **WHEN** the engine opens the db
- **THEN** the migration runner SHALL apply v5, the `_migrations` table SHALL contain a row with `version=5`, all pre-existing `project`, `change`, and `state_transition` rows SHALL remain byte-identical, the `config_state` table SHALL exist with exactly one row (`id=1`, populated by the migration's `INSERT OR IGNORE` step), and the `config_change` table SHALL exist (empty)

#### Scenario: Idempotent re-open after v5 migration

- **GIVEN** a state.db that has already had v5 applied
- **WHEN** the engine re-opens the db
- **THEN** no migration SHALL re-run, `_migrations` SHALL still contain exactly one row per applied version (1, 2, 3, 4, 5), and the `config_state` row SHALL NOT be re-inserted or modified

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