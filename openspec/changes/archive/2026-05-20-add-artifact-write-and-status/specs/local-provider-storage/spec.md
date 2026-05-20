## MODIFIED Requirements

### Requirement: Local provider directory layout

The local filesystem provider SHALL store all state under a `.speclink/` directory rooted at the project base (the directory containing the project config, or the current working directory when no project config exists). The layout SHALL be:

```
.speclink/
  config.toml              # project-level config (optional, created by CLI when missing)
  state.db                 # SQLite runtime state
  changes/
    <change-id>/
      proposal.md          # the change proposal artifact (required, created by `propose create`)
      design.md            # the design artifact (optional, created by `artifact write design`)
      tasks.md             # the tasks artifact (optional, created by `artifact write tasks`)
      specs/               # spec artifacts directory (optional, created when first spec is written)
        <capability>/
          spec.md          # one spec file per capability
      metadata.json        # lifecycle and actor metadata
```

Subdirectories that are not listed above (`archive/`, `packs/`, `cache/`) MAY exist when populated by future capabilities; this change SHALL NOT create them eagerly. Within `changes/<change-id>/`, only files corresponding to artifacts that have been written SHALL exist — `design.md`, `tasks.md`, and `specs/` SHALL NOT be created until the corresponding `artifact write` invocation succeeds.

All file paths SHALL be constructed with `std::path::PathBuf` join operations. The CLI SHALL NOT contain hard-coded `/` or `\` path separators in source code.

#### Scenario: First-time `propose create` initializes the directory

- **GIVEN** a CWD with no `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test"`
- **THEN** the directory `.speclink/changes/demo/` exists
- **AND** the file `.speclink/changes/demo/proposal.md` exists
- **AND** the file `.speclink/changes/demo/metadata.json` exists
- **AND** the file `.speclink/state.db` exists
- **AND** `design.md`, `tasks.md`, and `specs/` are not created by this command

#### Scenario: `artifact write` creates subdirectories on demand

- **GIVEN** a change `demo` with only `proposal.md` and `metadata.json` present
- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the file `.speclink/changes/demo/specs/auth/spec.md` exists
- **AND** the directory `.speclink/changes/demo/specs/` was created by this command
- **AND** `design.md` and `tasks.md` were not created

#### Scenario: Cross-platform path separator handling

- **GIVEN** a CWD on Windows `C:\Users\user\proj`
- **WHEN** the local provider writes `.speclink/changes/demo/design.md`
- **THEN** the file is created at `C:\Users\user\proj\.speclink\changes\demo\design.md`
- **AND** the `--json` data payload's `path` field uses forward slashes: `.speclink/changes/demo/design.md`

## ADDED Requirements

### Requirement: Multi-artifact atomic write

The local provider SHALL support atomic writes for four artifact kinds: `proposal`, `design`, `tasks`, and `spec`. Each write SHALL follow the same atomic write protocol established for proposal in the bootstrap change:

1. Create the target directory (and any missing parent directories such as `specs/<capability>/`) if it does not exist
2. Write artifact content to a temporary sibling file with the `.tmp` suffix (e.g., `design.md.tmp`, `specs/auth/spec.md.tmp`)
3. Rename the `.tmp` file to its final name
4. On any failure (write error, rename error), remove all created `.tmp` files; if the change directory or its `specs/<capability>/` subdirectory was newly created by this invocation, remove it as well (do not remove pre-existing files)

Unlike `propose create`, `artifact write` for `design` / `tasks` / `spec` SHALL NOT update `metadata.json`. The `metadata.json` file is the responsibility of `propose create` (initial write) and future commands (`archive`, `finish`); `artifact write` only writes the artifact file itself.

The local provider SHALL refuse to write an artifact when:
- The change directory does not exist — SHALL return `ProviderError::ChangeNotFound`
- The target artifact file already exists — SHALL return a domain error mapped to `error.code = "artifact.already_exists"` (exit code 1)

#### Scenario: Sequential multi-artifact writes succeed

- **GIVEN** a change `demo` initialized by `propose create`
- **WHEN** the user runs in sequence:
  1. `echo "design body" | speclink artifact write design --change demo --stdin --json`
  2. `echo "tasks body" | speclink artifact write tasks --change demo --stdin --json`
  3. `echo "auth spec" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** all three commands exit with code 0
- **AND** `.speclink/changes/demo/design.md`, `tasks.md`, and `specs/auth/spec.md` all exist with their respective content
- **AND** no `.tmp` files remain in the change directory

#### Scenario: Spec write fails when capability dir cannot be created

- **GIVEN** `.speclink/changes/demo/specs/` is a regular file (not a directory) due to corruption
- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "internal.error"`
- **AND** no `.tmp` files remain anywhere under `.speclink/changes/demo/`

#### Scenario: Pre-existing artifact is not overwritten

- **GIVEN** `.speclink/changes/demo/specs/auth/spec.md` already exists with content `OLD`
- **WHEN** the user runs `echo "NEW" | speclink artifact write spec --change demo --capability auth --stdin --json`
- **THEN** the process exit code is 1
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.already_exists"`
- **AND** the existing file content remains `OLD`

### Requirement: Spec capability routing

When writing a spec artifact, the local provider SHALL route the content to `<change-dir>/specs/<capability>/spec.md` where `<capability>` is the value of the `--capability` flag. The capability name SHALL match `^[a-z][a-z0-9-]{0,63}$` (same rules as change-id).

If multiple spec artifacts are written for the same change (different capabilities), each SHALL occupy its own subdirectory under `specs/`. The provider SHALL NOT impose a limit on the number of capabilities per change.

The provider SHALL NOT interpret the spec content (e.g., it SHALL NOT parse `## ADDED Requirements` / `## MODIFIED Requirements` headings). The content is treated as opaque markdown. Delta heading parsing is the responsibility of a future archive capability.

#### Scenario: Multiple spec capabilities in one change

- **GIVEN** a change `demo` initialized by `propose create`
- **WHEN** the user runs `artifact write spec` twice with capability `auth` and `billing` respectively
- **THEN** both `.speclink/changes/demo/specs/auth/spec.md` and `.speclink/changes/demo/specs/billing/spec.md` exist
- **AND** each contains its respective stdin content

#### Scenario: Capability name validation matches change-id rules

- **WHEN** the user runs `echo "x" | speclink artifact write spec --change demo --capability Auth-Module --stdin --json`
- **THEN** the process exit code is 2
- **AND** the stdout JSON contains a failure envelope with `error.code = "artifact.invalid_capability"`

### Requirement: Change status filesystem scan

The local provider SHALL implement `Provider::get_status` by scanning the filesystem under `<change-dir>/`:

1. If `<change-dir>/metadata.json` does not exist, SHALL return `ProviderError::ChangeNotFound`
2. Read and parse `metadata.json` to obtain `changeId` and `state`; parsing failure SHALL return a domain error mapped to `error.code = "internal.error"`
3. Check existence of `<change-dir>/proposal.md`, `design.md`, `tasks.md` and produce one `ArtifactStatus` entry per kind regardless of whether the file exists (`status` field reflects presence)
4. If `<change-dir>/specs/` exists, enumerate its immediate subdirectories; for each subdirectory `<capability>/` containing a `spec.md` file, produce one `ArtifactStatus` entry with `id = "spec:<capability>"` and `status = "done"`. Subdirectories without `spec.md` SHALL be ignored.
5. The returned `ChangeStatus.artifacts` array SHALL be ordered: `proposal`, `design`, `tasks`, then spec entries sorted ascending by capability name

The scan SHALL be read-only: no files SHALL be created, modified, or deleted by `get_status`.

The scan SHALL NOT recurse beyond depth 2 under `<change-dir>/` (it MUST NOT walk arbitrary subdirectories). Unknown files or subdirectories under `<change-dir>/` SHALL be ignored.

#### Scenario: Status of partially complete change

- **GIVEN** `.speclink/changes/demo/` containing only `proposal.md`, `design.md`, `metadata.json`, and `specs/auth/spec.md`
- **WHEN** the local provider's `get_status` is invoked for change `demo`
- **THEN** the returned `ChangeStatus.artifacts` contains 4 entries in this order: `proposal` (done), `design` (done), `tasks` (missing), `spec:auth` (done)
- **AND** `ChangeStatus.state` equals the state from `metadata.json` (e.g., `"proposed"`)

#### Scenario: Empty specs dir produces no spec entries

- **GIVEN** `.speclink/changes/demo/specs/` exists but contains no subdirectories
- **WHEN** `get_status` is invoked for change `demo`
- **THEN** the returned `artifacts` array contains only `proposal`, `design`, `tasks` entries
- **AND** no `spec:*` entries are present

#### Scenario: Subdirectory under specs without spec.md is ignored

- **GIVEN** `.speclink/changes/demo/specs/auth/` exists but contains no `spec.md` (e.g., a stale empty directory)
- **WHEN** `get_status` is invoked for change `demo`
- **THEN** no `spec:auth` entry appears in the returned `artifacts` array
