## ADDED Requirements

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

### Requirement: The renderer SHALL NOT alter `--json` envelope behavior

When the `--json` flag is supplied, the CLI SHALL emit the existing `Envelope` JSON structure unchanged from the bootstrap and slice-A snapshots. The pretty-printer introduced by this requirement SHALL only be invoked on the success path of human-mode output (no `--json`).

#### Scenario: `--json` flag bypasses the human pretty-printer

- **WHEN** the user runs `speclink --json <any-subcommand>` for any subcommand that previously had a `--json` snapshot
- **THEN** the byte sequence on stdout SHALL be the existing single-line JSON `Envelope`, identical to the snapshot fixed in slice A

#### Scenario: Existing JSON snapshots remain stable

- **WHEN** the workspace runs the existing `crates/cli/tests/snapshots.rs` insta tests
- **THEN** all 10 slice-A snapshots SHALL pass without modification, and `crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap` SHALL remain unchanged

### Requirement: The renderer SHALL leave stderr error and hint output untouched

When `speclink <subcommand>` fails, the CLI SHALL continue to write `error[<code>]: <message>` and optional `hint: <text>` to stderr exactly as the bootstrap and slice-A code does. This requirement explicitly EXCLUDES the failure path from the human pretty-printer.

#### Scenario: Failure stderr output is unchanged

- **WHEN** a user runs `speclink show change unknown` (without `--json`)
- **THEN** stderr SHALL contain the two lines `error[change.not_found]: change `unknown` not found in state.db` and `hint: Verify the change name via `speclink list --changes`.`, and stdout SHALL be empty
