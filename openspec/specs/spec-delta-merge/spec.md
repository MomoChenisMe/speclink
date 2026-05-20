# spec-delta-merge Specification

## Purpose

TBD - created by archiving change 'add-local-archive'. Update Purpose after archive.

## Requirements

### Requirement: Delta heading recognition

The spec delta merge SHALL recognize exactly four top-level (`## `) heading variants in a delta spec file:

```
## ADDED Requirements
## MODIFIED Requirements
## REMOVED Requirements
## RENAMED Requirements
```

Heading recognition SHALL be:

- Case-sensitive (exact match required)
- Whitespace-tolerant only for trailing whitespace on the same line (leading whitespace before `##` SHALL cause the line to be treated as content, not a heading)
- Order-independent: the four headings MAY appear in any order
- Optional: a delta spec MAY contain only a subset of these four headings; absent headings SHALL produce zero counts
- Unique: each heading SHALL appear at most once per delta file; a second occurrence SHALL produce a `spec.delta_parse_error`

Lines that begin with `## ` but do not exactly match one of the four variants SHALL produce a `spec.delta_parse_error`.

#### Scenario: All four headings present

- **GIVEN** a delta spec containing `## ADDED Requirements`, `## MODIFIED Requirements`, `## REMOVED Requirements`, and `## RENAMED Requirements` in that order
- **WHEN** the merger parses the file
- **THEN** all four sections are recognized and their requirement blocks are categorized accordingly

#### Scenario: Only ADDED heading

- **GIVEN** a delta spec containing only `## ADDED Requirements` followed by two `### Requirement:` blocks
- **WHEN** the merger parses the file
- **THEN** `addedCount = 2`, `modifiedCount = 0`, `removedCount = 0`, `renamedCount = 0`

#### Scenario: Unrecognized top-level heading rejected

- **GIVEN** a delta spec containing `## ADDED Requirements` followed by `## DEPRECATED Requirements`
- **WHEN** the merger parses the file
- **THEN** parsing fails with error code `spec.delta_parse_error`
- **AND** the error message names the unrecognized heading


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: Requirement block delimitation

Within each delta section, each `### Requirement: <name>` heading SHALL begin a new requirement block. The block content SHALL extend from the heading line (inclusive) until one of the following:

- The next `### Requirement: ` heading (same level, same section)
- The next `## ` heading (new section)
- End of file

The requirement name SHALL be the substring after `### Requirement: ` (with leading/trailing whitespace trimmed) on the heading line. Names MAY contain any printable characters including backticks (e.g., ``### Requirement: `propose create` command surface``).

The merger SHALL NOT interpret nested `#### Scenario:` or `##### Example:` content — these are part of the requirement block and SHALL be carried verbatim into the main spec.

#### Scenario: Requirement block includes nested scenarios

- **GIVEN** a delta containing `### Requirement: A` followed by 3 `#### Scenario:` blocks and then `### Requirement: B`
- **WHEN** the merger extracts requirement `A`
- **THEN** the extracted content includes the heading line, all 3 scenarios, and stops before `### Requirement: B`

#### Scenario: Requirement name with backticks

- **GIVEN** a delta containing ``### Requirement: `artifact write` command surface``
- **WHEN** the merger parses the requirement
- **THEN** the requirement name is recorded as ``` `artifact write` command surface ``` (with backticks preserved)


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: ADDED requirements append semantics

For each `### Requirement: <name>` block under `## ADDED Requirements`:

- If the main spec does NOT contain a `### Requirement: <name>` heading with matching name, the entire block (heading + body) SHALL be appended to the main spec
- If the main spec already contains a matching requirement, the merger SHALL fail with error code `spec.delta_conflict` and the error message SHALL include the conflicting requirement name

Append order SHALL preserve the order in the delta file. Block separation SHALL be a single blank line between the existing main spec content and each appended block (unless the main spec already ends with a blank line, in which case no extra separator is added).

When the main spec file does not exist at all, ADDED requirements SHALL create the main spec file with no header content (the file begins directly with the first `### Requirement:` heading).

#### Scenario: ADDED to nonexistent main spec

- **GIVEN** `.speclink/specs/auth/spec.md` does not exist
- **AND** the delta contains `## ADDED Requirements` with `### Requirement: User login` (with body)
- **WHEN** the merger applies the delta
- **THEN** the file `.speclink/specs/auth/spec.md` is created
- **AND** the file content begins with `### Requirement: User login`
- **AND** `createdMainSpec` in the summary equals `true`

#### Scenario: ADDED with existing requirement conflicts

- **GIVEN** `.speclink/specs/auth/spec.md` contains `### Requirement: User login`
- **AND** the delta contains `## ADDED Requirements` with `### Requirement: User login`
- **WHEN** the merger applies the delta
- **THEN** the merge fails with error code `spec.delta_conflict`
- **AND** the error message includes `User login`


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: MODIFIED requirements replace semantics

For each `### Requirement: <name>` block under `## MODIFIED Requirements`:

- The merger SHALL locate the existing `### Requirement: <name>` block in the main spec (matching the exact name string)
- The entire existing block (from `### Requirement:` line up to but excluding the next `### Requirement:`, next `## ` heading, or end of file) SHALL be replaced by the delta block
- If the existing block cannot be located, the merger SHALL fail with error code `spec.delta_conflict`

The merger SHALL NOT compute or compare content hashes — it trusts the delta content verbatim.

#### Scenario: MODIFIED replaces full block

- **GIVEN** `.speclink/specs/auth/spec.md` contains `### Requirement: Token rotation` with 1 scenario
- **AND** the delta contains `## MODIFIED Requirements` with `### Requirement: Token rotation` with 3 scenarios
- **WHEN** the merger applies the delta
- **THEN** the main spec contains `### Requirement: Token rotation` with 3 scenarios (the new content)
- **AND** the original 1-scenario version is gone

#### Scenario: MODIFIED not found

- **GIVEN** `.speclink/specs/auth/spec.md` exists but contains no `### Requirement: Missing Req`
- **AND** the delta contains `## MODIFIED Requirements` with `### Requirement: Missing Req`
- **WHEN** the merger applies the delta
- **THEN** the merge fails with error code `spec.delta_conflict`
- **AND** the error message includes `Missing Req`


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: REMOVED requirements delete semantics

For each `### Requirement: <name>` block under `## REMOVED Requirements`:

- The merger SHALL locate the existing `### Requirement: <name>` block in the main spec
- The entire block SHALL be removed from the main spec, including any trailing blank lines between this block and the next heading
- If the block cannot be located, the merger SHALL fail with error code `spec.delta_conflict`

The body of REMOVED requirement blocks MAY contain `**Reason**:` and `**Migration**:` metadata lines; the merger SHALL treat these as documentation only and SHALL NOT use them to alter merge behavior.

#### Scenario: REMOVED deletes from main spec

- **GIVEN** `.speclink/specs/auth/spec.md` contains 3 requirements: `A`, `B`, `C`
- **AND** the delta contains `## REMOVED Requirements` with `### Requirement: B`
- **WHEN** the merger applies the delta
- **THEN** the main spec contains only `### Requirement: A` and `### Requirement: C`
- **AND** no blank line gap larger than one line remains between A and C

#### Scenario: REMOVED metadata is documentation only

- **GIVEN** a delta whose `## REMOVED Requirements` section contains a single requirement named `Old Token Flow`
- **AND** that requirement block additionally contains `**Reason**: Replaced by new flow` and `**Migration**: Update clients to use the new endpoint` lines under the heading
- **AND** the main spec contains a requirement with the same name `Old Token Flow`
- **WHEN** the merger applies the delta
- **THEN** the matched requirement is removed from the main spec
- **AND** the `Reason` and `Migration` text from the delta does not appear in the resulting main spec


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: RENAMED requirements rename-only semantics

For each `### Requirement: <name>` block under `## RENAMED Requirements`:

- The block body MUST contain two lines: `**FROM:** <old-name>` and `**TO:** <new-name>` (in any order). Both lines SHALL be plain text without backtick wrapping of the name (the name text MAY contain backticks as part of the name itself).
- The merger SHALL locate `### Requirement: <old-name>` in the main spec and replace only the heading line, changing `<old-name>` to `<new-name>`. The remainder of the requirement body SHALL remain unchanged.
- If `<old-name>` cannot be located in the main spec, the merger SHALL fail with error code `spec.delta_conflict`
- If the delta block lacks either `**FROM:**` or `**TO:**` line, the merger SHALL fail with error code `spec.delta_parse_error`

RENAMED SHALL only change the name. To change name AND content, the author SHALL provide both a RENAMED entry and a MODIFIED entry (applied in order: RENAMED first, then MODIFIED matching the new name).

#### Scenario: RENAMED changes only the heading

- **GIVEN** `.speclink/specs/auth/spec.md` contains `### Requirement: User login` with a `#### Scenario: Email login` body
- **AND** the delta contains:

  ```
  ## RENAMED Requirements

  ### Requirement: Sign-in
  **FROM:** User login
  **TO:** Sign-in
  ```
- **WHEN** the merger applies the delta
- **THEN** the main spec contains `### Requirement: Sign-in`
- **AND** the body (including `#### Scenario: Email login`) is preserved unchanged

#### Scenario: RENAMED with missing FROM line

- **GIVEN** the delta contains a RENAMED entry with only `**TO:** New name` (no `**FROM:**`)
- **WHEN** the merger parses the delta
- **THEN** parsing fails with error code `spec.delta_parse_error`


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: Apply order across heading sections

When a single delta file contains multiple heading sections, the merger SHALL apply them in this fixed order regardless of their order in the source file:

1. `## RENAMED Requirements` (rename existing blocks first so subsequent MODIFIED can target new names)
2. `## REMOVED Requirements`
3. `## MODIFIED Requirements`
4. `## ADDED Requirements`

This order avoids ambiguity: a delta SHALL NOT contain a RENAMED to name `X` and an ADDED with name `X` in the same file. If such a conflict exists, ADDED runs after RENAMED and the existing `X` (now named X via rename) collides with the ADDED — the merger SHALL fail with `spec.delta_conflict`.

#### Scenario: RENAMED before MODIFIED with same new name

- **GIVEN** the delta contains both `## RENAMED Requirements` (`FROM: A`, `TO: B`) and `## MODIFIED Requirements` (`### Requirement: B` with new content)
- **WHEN** the merger applies the delta
- **THEN** the main spec first has `A` renamed to `B`
- **AND** then `B` is replaced with the modified content
- **AND** the final main spec contains `### Requirement: B` with the modified content


<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->

---
### Requirement: Apply summary output

After a successful apply, the merger SHALL return a structured summary containing the counts of each operation performed:

- `added_count`: integer ≥ 0
- `modified_count`: integer ≥ 0
- `removed_count`: integer ≥ 0
- `renamed_count`: integer ≥ 0
- `created_main_spec`: boolean (true if the main spec file was created during this apply)

When apply fails (any error returned), no summary SHALL be returned and no partial state SHALL be visible to callers (the merger operates on in-memory strings; the caller is responsible for not writing on failure).

#### Scenario: Summary counts match delta

- **GIVEN** a delta with 2 ADDED, 1 MODIFIED, 1 REMOVED, 0 RENAMED requirements applied successfully
- **WHEN** the merger returns the summary
- **THEN** `added_count = 2`, `modified_count = 1`, `removed_count = 1`, `renamed_count = 0`

<!-- @trace
source: add-local-archive
updated: 2026-05-20
code:
  - crates/cli/src/exit_code.rs
  - crates/runtime/src/spec_delta.rs
  - crates/cli/src/main.rs
  - crates/cli/Cargo.toml
  - crates/provider/src/model.rs
  - crates/cli/src/commands/artifact.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/state_db.rs
  - crates/provider/src/error.rs
  - crates/runtime/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/lib.rs
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/artifact.rs
  - crates/provider-local/src/archive.rs
  - crates/runtime/src/lib.rs
  - crates/provider-local/src/lib.rs
  - crates/provider-local/src/storage.rs
  - crates/runtime/src/propose.rs
  - crates/cli/src/commands/archive.rs
  - README.md
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/runtime/src/status.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/archive.rs
  - crates/cli/src/commands/status.rs
tests:
  - crates/cli/tests/archive.rs
  - crates/cli/tests/snapshots/status_snapshots__status_only_proposal.snap
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_spec_success.snap
  - crates/provider-local/tests/archive_integration.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_design_success.snap
  - crates/cli/tests/status.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_dry_run_success.snap
  - crates/cli/tests/archive_snapshots.rs
  - crates/cli/tests/snapshots/archive_snapshots__archive_success.snap
  - crates/cli/tests/artifact_write.rs
  - crates/cli/tests/snapshots/artifact_write_snapshots__artifact_write_already_exists.snap
  - crates/cli/tests/snapshots/archive_snapshots__archive_delta_conflict.snap
  - crates/cli/tests/status_snapshots.rs
  - crates/cli/tests/artifact_write_snapshots.rs
  - crates/cli/tests/snapshots/status_snapshots__status_change_not_found.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/status_snapshots__status_with_design_and_spec.snap
  - crates/provider-local/tests/multi_artifact_integration.rs
-->