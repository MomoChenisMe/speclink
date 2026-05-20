## MODIFIED Requirements

### Requirement: Local provider directory layout

The local filesystem provider SHALL store all state under a `.speclink/` directory rooted at the project base (the directory containing the project config, or the current working directory when no project config exists). The layout SHALL be:

```
.speclink/
  config.toml              # project-level config (optional, created by CLI when missing)
  state.db                 # SQLite runtime state
  specs/                   # main spec directory (created when first archive applies deltas)
    <capability>/
      spec.md              # accumulated spec for each capability after archive merges
  changes/
    <change-id>/           # active change directory
      proposal.md          # the change proposal artifact (required, created by `propose create`)
      design.md            # the design artifact (optional, created by `artifact write design`)
      tasks.md             # the tasks artifact (optional, created by `artifact write tasks`)
      specs/               # change-local delta spec artifacts (optional)
        <capability>/
          spec.md          # one delta spec file per capability
      metadata.json        # lifecycle and actor metadata
    archive/               # archived changes directory (created on first archive)
      YYYY-MM-DD-<change-id>/   # one entry per archived change
        proposal.md
        design.md          # if it existed
        tasks.md           # if it existed
        specs/             # the change's delta spec files (preserved as historical record)
          <capability>/
            spec.md
        metadata.json      # with state="archived" and archivedAt timestamp
```

Subdirectories that are not listed above (`packs/`, `cache/`) MAY exist when populated by future capabilities. Within `changes/<change-id>/`, only files corresponding to artifacts that have been written SHALL exist. Within `.speclink/specs/`, only capabilities that have been archived at least once SHALL appear.

All file paths SHALL be constructed with `std::path::PathBuf` join operations. The CLI SHALL NOT contain hard-coded `/` or `\` path separators in source code.

The `.speclink/specs/` directory is the main spec directory: it contains the accumulated state of all capabilities after their respective delta specs have been merged in by archive operations. It SHALL NOT contain any metadata files (no `.spec_index.json`, no per-capability metadata sidecar) — each `<capability>/spec.md` is a self-contained markdown file.

#### Scenario: First-time `propose create` initializes the directory

- **GIVEN** a CWD with no `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test"`
- **THEN** the directory `.speclink/changes/demo/` exists
- **AND** the file `.speclink/changes/demo/proposal.md` exists
- **AND** the file `.speclink/changes/demo/metadata.json` exists
- **AND** the file `.speclink/state.db` exists
- **AND** `design.md`, `tasks.md`, `specs/`, `archive/`, and `.speclink/specs/` are not created by this command

#### Scenario: `archive` creates archive directory and main spec directory

- **GIVEN** an active change `demo` with `specs/auth/spec.md` containing one ADDED requirement
- **AND** neither `.speclink/changes/archive/` nor `.speclink/specs/` exists before archive
- **WHEN** the user runs `speclink archive demo --json`
- **THEN** the directory `.speclink/changes/archive/YYYY-MM-DD-demo/` exists
- **AND** the file `.speclink/specs/auth/spec.md` exists
- **AND** the file `.speclink/changes/archive/YYYY-MM-DD-demo/metadata.json` has `state: "archived"` and an `archivedAt` field

#### Scenario: Cross-platform path separator handling

- **GIVEN** a CWD on Windows `C:\Users\user\proj`
- **WHEN** the local provider writes `.speclink/specs/auth/spec.md` during archive
- **THEN** the file is created at `C:\Users\user\proj\.speclink\specs\auth\spec.md`
- **AND** the `--json` data payload's `archivePath` and `mainSpecPath` fields use forward slashes

## ADDED Requirements

### Requirement: Lifecycle state value `archived`

The `metadata.json` `state` field SHALL accept the value `"archived"` (lowercase). The `State` enum in the provider crate SHALL have the variants `Draft`, `Proposed`, and `Archived` (with serde rename_all to lowercase strings). The `State::Archived` variant SHALL be the lifecycle state assigned by the `archive` command upon successful archive.

In-progress, validated, packed, unpacked, reviewing, accepted, rejected, and cancelled states are deferred to future changes. AI skills consuming `metadata.json` SHALL be forward-compatible: an unknown state value SHALL NOT cause skills to panic; SpecLink does not introduce other state values in this change but reserves the right to add them.

#### Scenario: Archived metadata is serialized as lowercase string

- **GIVEN** a change has been archived
- **WHEN** the local provider reads `metadata.json` from the archive directory
- **THEN** the `state` field is the string `"archived"`
- **AND** the `State` enum parses this string as `State::Archived`

### Requirement: `archivedAt` metadata field

When the local provider archives a change, it SHALL update `metadata.json` to add a new field `archivedAt` (string, ISO 8601 UTC with second precision, e.g., `2026-05-19T12:34:56Z`). The existing fields `changeId`, `state`, `createdAt`, and `createdBy` SHALL be preserved with their existing values, except that `state` SHALL be set to `"archived"`.

The `archivedAt` field SHALL only appear in metadata after archive succeeds. Before archive, `metadata.json` SHALL NOT contain this field.

The CLI SHALL NOT impose backward compatibility for `metadata.json` lacking `archivedAt` on already-archived changes from earlier versions — `archivedAt` is only written by this change going forward.

#### Scenario: Archived metadata contains archivedAt

- **GIVEN** a change `demo` has been archived at time T (in UTC)
- **WHEN** the local provider reads `.speclink/changes/archive/YYYY-MM-DD-demo/metadata.json`
- **THEN** the JSON object contains `state: "archived"`
- **AND** the JSON object contains `archivedAt` whose value parses as a valid ISO 8601 UTC timestamp
- **AND** the JSON object preserves `createdAt`, `createdBy`, and `changeId` from before archive

### Requirement: Archive directory naming and uniqueness

The local provider SHALL place each archived change under `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/`. The date prefix SHALL be the local-timezone date passed in via `ArchiveOptions::archive_date`, formatted `%Y-%m-%d`.

If the target archive directory `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/` already exists at the moment of the rename step, the local provider SHALL return an error mapped to `error.code = "archive.change_not_archivable"` (exit code 1). The local provider SHALL NOT attempt to merge, overwrite, or suffix-disambiguate to a different directory name.

#### Scenario: Same-day re-archive rejected

- **GIVEN** `.speclink/changes/archive/2026-05-19-demo/` already exists
- **AND** the user attempts to archive a fresh change with id `demo` on the same local date
- **WHEN** the archive command runs
- **THEN** the operation fails with `archive.change_not_archivable`
- **AND** the new change's active directory remains untouched

### Requirement: Archive rollback safeguards

The local provider SHALL implement archive as a multi-step operation with explicit rollback safeguards. The required sequence:

1. Compute delta merge results in memory for every capability spec in the change; on any conflict or parse failure, abort with the appropriate error code and no filesystem side effect
2. For each existing main spec under `.speclink/specs/<capability>/spec.md` that will be modified or replaced, create a `.bak` sibling file before writing the new content
3. Write new main spec content to `.tmp` sibling files
4. Write updated `<change>/metadata.json` to a `.tmp` sibling
5. Rename the metadata `.tmp` to its final name
6. Rename each main spec `.tmp` to its final name
7. Rename the change directory `<change>/` to `archive/<YYYY-MM-DD>-<id>/`
8. Delete the row from `in_progress_change` matching the change id (no-op if absent)
9. On all successful steps, delete the `.bak` files created in step 2

If any step from 5–7 fails, the provider SHALL attempt to:

- Remove all `.tmp` files
- Restore any main spec files that were renamed in step 6 from their `.bak` sibling
- Leave `<change>/` in its pre-archive state

If rollback succeeds, the operation SHALL return the original error (e.g., `internal.error`). If rollback itself fails (rare; typically disk full or permission), the operation SHALL return `internal.error` with a message indicating manual recovery is required and listing the leftover `.bak` and `.tmp` files.

#### Scenario: Failed final rename rolls back main spec

- **GIVEN** an archive operation that has successfully written `.speclink/specs/auth/spec.md` (with `.bak` preserved)
- **AND** the rename of `<change>/` to archive fails (e.g., target directory created concurrently)
- **WHEN** the rollback path runs
- **THEN** the file `.speclink/specs/auth/spec.md` is restored to its pre-archive content from `.bak`
- **AND** the directory `<change>/` still exists at its original active location
- **AND** the operation returns `error.code = "internal.error"`

#### Scenario: Idempotent SQLite cleanup

- **GIVEN** an archive operation has completed steps 1–7 but step 8 (SQLite delete) finds no matching row
- **WHEN** the operation finishes
- **THEN** the archive is still considered successful (the SQLite delete is idempotent)
- **AND** the JSON envelope reports `ok: true`
