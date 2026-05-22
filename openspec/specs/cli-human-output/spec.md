# cli-human-output Specification

## Purpose

TBD - created by archiving change 'improve-human-output-pretty-print'. Update Purpose after archive.

## Requirements

### Requirement: Human-mode CLI output SHALL recursively pretty-print the `data` payload

When `speclink <subcommand>` is invoked without the `--json` flag and the operation succeeds, the CLI SHALL render the success `data` payload using the rules in this requirement and SHALL write the result to stdout followed by a single trailing newline. The `--json` envelope output (when `--json` IS supplied) SHALL be unchanged from prior slices.

The renderer SHALL accept any `serde_json::Value` and produce a deterministic UTF-8 string. The renderer SHALL be a pure function: no I/O, no panic, no ANSI escape sequences, no terminal-width awareness.

#### Scenario: Empty `data` object renders as `OK`

- **WHEN** the success `data` is the empty JSON object `{}`
- **THEN** the CLI SHALL print exactly `OK` followed by a newline, preserving the bootstrap-slice behavior used by `speclink unlink`

##### Example: empty object

GIVEN data:
```json
{}
```
WHEN rendered, the human-mode output SHALL equal:
```
OK
```

#### Scenario: Flat object renders one `key: value` per line

- **WHEN** the success `data` is a non-empty JSON object whose values are all scalars (string, number, bool, null)
- **THEN** the CLI SHALL print one `<key>: <value>` line per top-level key, sorted by key in lexicographic order (the default `serde_json::Map` iteration order, consistent with the byte-locked `--json` envelope snapshots from slice A), with no leading indentation; string values SHALL NOT be wrapped in JSON quotes; null SHALL render as the literal `null`

##### Example: flat object

GIVEN data:
```json
{ "name": "demo-billing", "version": 1, "active": true }
```
WHEN rendered, the human-mode output SHALL equal:
```
active: true
name: demo-billing
version: 1
```

#### Scenario: Nested object value renders with 2-space indentation on the next line

- **WHEN** a top-level key has a non-empty object value
- **THEN** the CLI SHALL print `<key>:` followed by a newline, then each child field on a new line indented by 2 additional spaces relative to the parent

##### Example: nested object

GIVEN data:
```json
{ "change": { "name": "demo-billing", "version": 1 } }
```
WHEN rendered, the human-mode output SHALL equal:
```
change:
  name: demo-billing
  version: 1
```

#### Scenario: Array of objects renders each element with `-` bullet and indented fields

- **WHEN** a top-level key has a non-empty array value whose elements are objects
- **THEN** the CLI SHALL print `<key>:` followed by a newline, then each element prefixed with `- ` indented by 2 spaces, with the element's fields indented by 4 spaces

##### Example: array of objects

GIVEN data:
```json
{ "artifacts": [ { "kind": "proposal", "capability": null }, { "kind": "spec", "capability": "user-auth" } ] }
```
WHEN rendered, the human-mode output SHALL equal (object fields sorted lexicographically per the flat-object scenario):
```
artifacts:
  - capability: null
    kind: proposal
  - capability: user-auth
    kind: spec
```

#### Scenario: Empty array renders as `(empty)`

- **WHEN** an array value is empty (`[]`)
- **THEN** the CLI SHALL print `<key>: (empty)` on a single line

##### Example: empty array

GIVEN data:
```json
{ "changes": [] }
```
WHEN rendered, the human-mode output SHALL equal:
```
changes: (empty)
```

#### Scenario: Array of scalars renders each element with `-` bullet

- **WHEN** a top-level key has a non-empty array value whose elements are scalars
- **THEN** the CLI SHALL print `<key>:` followed by a newline, then each element prefixed with `- ` indented by 2 spaces

##### Example: array of strings

GIVEN data:
```json
{ "capabilities": [ "rate-limiting", "user-auth" ] }
```
WHEN rendered, the human-mode output SHALL equal:
```
capabilities:
  - rate-limiting
  - user-auth
```

#### Scenario: String containing newlines preserves newlines with continuation indent

- **WHEN** a string value contains one or more `\n` characters
- **THEN** the CLI SHALL print the string with each `\n` followed by an indent matching the current nesting depth, so subsequent lines align under the value column

##### Example: string with newlines

GIVEN data:
```json
{ "content": "line one\nline two" }
```
WHEN rendered, the human-mode output SHALL equal:
```
content: line one
  line two
```


<!-- @trace
source: improve-human-output-pretty-print
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/runtime/src/ops.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/types.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/commands/show_change.rs
  - crates/provider/src/error.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - doc/speclink-design.md
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/new_artifact.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/artifact_read.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/link.rs
  - crates/cli/src/commands/status.rs
  - README.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/provider-local/src/artifact_store.rs
  - doc/protocol/operations.md
  - crates/cli/src/human.rs
  - crates/provider-local/src/paths.rs
  - crates/cli/src/commands/init.rs
tests:
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/change_crud.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
-->

---
### Requirement: The renderer SHALL NOT alter `--json` envelope behavior

When the `--json` flag is supplied, the CLI SHALL emit the existing `Envelope` JSON structure unchanged from the bootstrap and slice-A snapshots. The pretty-printer introduced by this requirement SHALL only be invoked on the success path of human-mode output (no `--json`).

#### Scenario: `--json` flag bypasses the human pretty-printer

- **WHEN** the user runs `speclink --json <any-subcommand>` for any subcommand that previously had a `--json` snapshot
- **THEN** the byte sequence on stdout SHALL be the existing single-line JSON `Envelope`, identical to the snapshot fixed in slice A

#### Scenario: Existing JSON snapshots remain stable

- **WHEN** the workspace runs the existing `crates/cli/tests/snapshots.rs` insta tests
- **THEN** all 10 slice-A snapshots SHALL pass without modification, and `crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap` SHALL remain unchanged


<!-- @trace
source: improve-human-output-pretty-print
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/runtime/src/ops.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/types.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/commands/show_change.rs
  - crates/provider/src/error.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - doc/speclink-design.md
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/new_artifact.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/artifact_read.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/link.rs
  - crates/cli/src/commands/status.rs
  - README.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/provider-local/src/artifact_store.rs
  - doc/protocol/operations.md
  - crates/cli/src/human.rs
  - crates/provider-local/src/paths.rs
  - crates/cli/src/commands/init.rs
tests:
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/change_crud.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
-->

---
### Requirement: The renderer SHALL leave stderr error and hint output untouched

When `speclink <subcommand>` fails, the CLI SHALL continue to write `error[<code>]: <message>` and optional `hint: <text>` to stderr exactly as the bootstrap and slice-A code does. This requirement explicitly EXCLUDES the failure path from the human pretty-printer.

#### Scenario: Failure stderr output is unchanged

- **WHEN** a user runs `speclink show change unknown` (without `--json`)
- **THEN** stderr SHALL contain the two lines `error[change.not_found]: change `unknown` not found in state.db` and `hint: Verify the change name via `speclink list --changes`.`, and stdout SHALL be empty

<!-- @trace
source: improve-human-output-pretty-print
updated: 2026-05-22
code:
  - crates/cli/src/commands/unlink.rs
  - crates/runtime/src/ops.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/types.rs
  - crates/cli/src/main.rs
  - crates/provider-local/src/lib.rs
  - crates/runtime/src/change_ops.rs
  - crates/cli/src/commands/list_specs.rs
  - crates/cli/src/commands/show_change.rs
  - crates/provider/src/error.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__flat_object.snap
  - crates/provider-local/Cargo.toml
  - crates/runtime/src/bootstrap.rs
  - crates/cli/src/output.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_array.snap
  - doc/speclink-design.md
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/new_artifact.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/new_change.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__empty_object.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__string_with_newlines.snap
  - crates/provider-local/src/state_db.rs
  - crates/runtime/src/lib.rs
  - crates/cli/src/commands/delete_change.rs
  - crates/cli/src/commands/artifact_read.rs
  - crates/cli/src/commands/list_changes.rs
  - crates/runtime/src/artifact_ops.rs
  - crates/provider/Cargo.toml
  - crates/runtime/src/error.rs
  - crates/cli/src/commands/link.rs
  - crates/cli/src/commands/status.rs
  - README.md
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_objects.snap
  - crates/cli/src/snapshots/speclink_cli__human__tests__array_of_scalars.snap
  - crates/provider-local/src/store.rs
  - crates/provider-local/src/change_store.rs
  - crates/cli/src/snapshots/speclink_cli__human__tests__nested_object.snap
  - crates/provider-local/src/artifact_store.rs
  - doc/protocol/operations.md
  - crates/cli/src/human.rs
  - crates/provider-local/src/paths.rs
  - crates/cli/src/commands/init.rs
tests:
  - crates/cli/tests/snapshots/snapshots__list_specs_two_caps.snap
  - crates/cli/tests/snapshots/snapshots__new_change_duplicate_error.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_version_conflict_error.snap
  - crates/cli/tests/snapshots/snapshots__show_change_not_found_error.snap
  - crates/cli/tests/cli.rs
  - crates/cli/tests/snapshots/snapshots__show_change_empty.snap
  - crates/cli/tests/snapshots.rs
  - crates/cli/tests/snapshots/snapshots__delete_change_success.snap
  - crates/runtime/tests/change_ops.rs
  - crates/cli/tests/snapshots/snapshots__artifact_read_proposal_hello.snap
  - crates/cli/tests/snapshots/snapshots__new_change_success.snap
  - crates/cli/tests/artifact_io.rs
  - crates/runtime/tests/artifact_ops.rs
  - crates/cli/tests/change_crud.rs
  - crates/cli/tests/etag_concurrency.rs
  - crates/cli/tests/snapshots/snapshots__list_changes_one.snap
  - crates/cli/tests/snapshots/snapshots__new_artifact_proposal_hello.snap
-->