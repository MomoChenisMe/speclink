### Requirement: Stable JSON envelope for `--json` output

When invoked with `--json`, every SpecLink CLI command SHALL emit exactly one JSON object to stdout on a single line, conforming to the following envelope schema. The envelope provides forward compatibility: AI skills depend on these field names and SHALL NOT break when the CLI adds new fields, provided existing fields keep their meaning.

The envelope MUST contain these top-level fields:

- `ok` (boolean, required): `true` on success, `false` on failure.
- `data` (object or null, required): command-specific payload on success; `null` on failure.
- `warnings` (array, required): list of non-fatal advisories. Each element MUST contain `code` (string, dot-separated) and `message` (string). The array MUST be present even when empty.
- `error` (object or null, required): `null` on success; on failure MUST contain `code` (string, dot-separated), `message` (string), and `details` (object, MAY be empty).
- `requestId` (string, required): an opaque identifier of the form `req_<uuid_v4_hex>` that uniquely identifies the CLI invocation. The value SHALL be regenerated for every invocation unless the environment variable `SPECLINK_TEST_REQUEST_ID` is set, in which case its value SHALL be used verbatim.

The envelope MUST NOT contain any field whose key matches `token`, `access_token`, `refresh_token`, `api_key`, `password`, or `secret` (case-insensitive). Nested objects MUST also be redacted.

#### Scenario: Successful command emits success envelope

- **WHEN** a CLI command completes successfully with `--json` set
- **THEN** stdout contains exactly one line of valid JSON parseable by `serde_json::from_str`
- **AND** the parsed object has `ok = true`, `data` is a non-null object, `warnings` is an array, `error` is `null`, and `requestId` matches `^req_[0-9a-f]{32}$`

##### Example: propose create success envelope

- **GIVEN** invocation `speclink propose create --change demo --summary "test" --json`
- **WHEN** the command completes successfully
- **THEN** stdout contains a single line equivalent to:

```json
{"ok":true,"data":{"changeId":"demo","state":"proposed","artifactPath":".speclink/changes/demo/proposal.md","mode":"local"},"warnings":[],"error":null,"requestId":"req_0123456789abcdef0123456789abcdef"}
```

#### Scenario: Failed command emits failure envelope

- **WHEN** a CLI command fails with `--json` set
- **THEN** stdout contains exactly one line of valid JSON
- **AND** the parsed object has `ok = false`, `data` is `null`, `error` is a non-null object with `code` (dot-separated string), `message` (non-empty string), and `details` (object)
- **AND** the process exit code matches the failure category defined in the exit-code requirement below

#### Scenario: Secret-named field is redacted before serialization

- **GIVEN** a command implementation that internally holds a struct with field `access_token`
- **WHEN** the command serializes its data with `--json`
- **THEN** the emitted JSON MUST NOT contain the literal field name `access_token` or its value
- **AND** the JSON MUST NOT contain any field whose key matches `token`, `refresh_token`, `api_key`, `password`, or `secret` (case-insensitive)


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Stable exit-code table

Every SpecLink CLI command SHALL terminate with one of the following exit codes. The mapping from internal error to exit code SHALL be deterministic; identical input SHALL produce identical exit codes across runs and platforms.

| Code | Meaning                                             |
| ---- | --------------------------------------------------- |
| 0    | Success                                             |
| 1    | General error (filesystem, internal, unclassified)  |
| 2    | User input error (invalid argument, bad TOML, etc.) |
| 3    | Validation failed (reserved; unused in this change) |
| 4    | Analyzer found blocking issues (reserved; unused)   |
| 5    | Provider unavailable                                |
| 6    | Auth required but no fallback allowed               |
| 7    | Conflict (reserved; unused in this change)          |

Exit codes 3, 4, and 7 are reserved for future capabilities and SHALL NOT be produced by any command in this change. Any unclassified error SHALL produce exit code 1.

#### Scenario: Invalid argument produces exit code 2

- **WHEN** the user invokes a CLI command with a missing required flag or a malformed value
- **THEN** clap emits a parse error to stderr
- **AND** the process exit code is 2
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "input.invalid"`

#### Scenario: Remote provider configured but not authenticated, fallback disabled

- **GIVEN** a project config with `provider = "acme"` and `fallback = "disabled"`
- **AND** no auth token is configured for `acme`
- **WHEN** any AI workflow CLI command is invoked
- **THEN** the process exit code is 6
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "provider.not_authenticated"`

#### Scenario: Successful invocation produces exit code 0

- **WHEN** the command completes its intended action without error
- **THEN** the process exit code is 0
- **AND** if `--json` is set, `ok = true`


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Error code naming convention

Every error and warning emitted in the envelope SHALL use a dot-separated identifier of the form `<capability>.<short_snake_case_reason>`. The capability prefix SHALL match a spec capability name (without the `cli-` prefix where applicable). The reason segment SHALL use lowercase ASCII letters, digits, and underscores only.

This change defines the following error codes:

- `input.invalid` — user supplied an invalid argument, flag value, or stdin payload
- `change.already_exists` — a change with the given id already exists
- `change.invalid_id` — change id does not match the required kebab-case pattern
- `provider.not_authenticated` — a remote provider is configured but no valid auth token is present
- `provider.unavailable` — a configured provider cannot be reached
- `internal.error` — an unclassified internal failure

#### Scenario: New error code conforms to the naming convention

- **GIVEN** any error code emitted by a CLI command in this change
- **WHEN** the code is inspected
- **THEN** the code matches the regular expression `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`
- **AND** the capability prefix corresponds to an existing spec capability name


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Machine-interface flag semantics

Every SpecLink CLI command SHALL accept the following flags with consistent semantics:

- `--json`: emit a single envelope JSON object to stdout. Without this flag, stdout MAY contain human-readable text and the envelope SHALL NOT be emitted.
- `--no-color`: disable ANSI color codes in human-readable stderr output. The flag SHALL have no effect on stdout when `--json` is set.
- `--quiet`: suppress all human-readable progress output on stderr at tracing level INFO and below. Errors at WARN and ERROR levels SHALL still be emitted to stderr.
- `--stdin`: accept input from stdin. The accepted format SHALL be defined per command. In this change, no command accepts stdin; any command receiving `--stdin` in this change SHALL produce exit code 2 with `error.code = "input.invalid"`.

#### Scenario: `--json` and `--quiet` together

- **GIVEN** invocation `speclink propose create --change demo --summary "test" --json --quiet`
- **WHEN** the command runs successfully
- **THEN** stdout contains exactly one JSON envelope line
- **AND** stderr contains no output below WARN level

#### Scenario: Cross-platform stdout line ending

- **GIVEN** invocation with `--json`
- **WHEN** stdout is captured on Windows
- **THEN** the captured bytes end with exactly one `\n` (LF), not `\r\n`
- **AND** the same invocation on macOS and Linux produces the same byte sequence


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Secret redaction in tracing output

When the CLI emits tracing logs to stderr, any field or value matching a known secret pattern SHALL be replaced with the literal string `[REDACTED]` before emission. Known secret patterns include any field named `token`, `access_token`, `refresh_token`, `api_key`, `password`, `secret`, or values matching `Bearer <whitespace><non-whitespace>+`.

#### Scenario: Bearer token in URL is redacted in tracing

- **GIVEN** internal code that logs a struct containing `authorization: "Bearer abc123"` via tracing
- **WHEN** tracing emits the log line to stderr
- **THEN** the line contains `[REDACTED]` in place of `Bearer abc123`

## Requirements


<!-- @trace
source: bootstrap-workspace-and-propose-create
updated: 2026-05-19
code:
  - crates/runtime/src/propose.rs
  - crates/provider/src/config_discovery.rs
  - Cargo.toml
  - crates/provider-local/src/lib.rs
  - crates/cli/src/main.rs
  - crates/provider-local/Cargo.toml
  - crates/provider-local/src/error.rs
  - crates/provider/src/model.rs
  - crates/provider-local/src/state_db.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/commands/propose.rs
  - crates/cli/src/commands/mod.rs
  - crates/provider-local/src/storage.rs
  - .github/workflows/ci.yml
  - crates/runtime/src/lib.rs
  - README.md
  - crates/provider/src/resolution.rs
  - crates/provider/src/config.rs
  - crates/cli/src/tracing_layer.rs
  - crates/provider/src/lib.rs
  - crates/provider/src/error.rs
  - crates/cli/src/exit_code.rs
  - crates/runtime/Cargo.toml
  - crates/cli/Cargo.toml
  - crates/cli/src/output.rs
  - crates/cli/src/cli.rs
  - crates/provider/Cargo.toml
  - rust-toolchain.toml
  - LICENSE
tests:
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_success.snap
  - crates/cli/tests/propose_create_snapshots.rs
  - crates/cli/tests/propose_create.rs
  - crates/provider-local/tests/local_provider_integration.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_fallback_warning.snap
  - crates/provider/tests/dyn_provider_compile.rs
  - crates/cli/tests/snapshots/propose_create_snapshots__propose_create_change_already_exists.snap
-->

### Requirement: Stable JSON envelope for `--json` output

When invoked with `--json`, every SpecLink CLI command SHALL emit exactly one JSON object to stdout on a single line, conforming to the following envelope schema. The envelope provides forward compatibility: AI skills depend on these field names and SHALL NOT break when the CLI adds new fields, provided existing fields keep their meaning.

The envelope MUST contain these top-level fields:

- `ok` (boolean, required): `true` on success, `false` on failure.
- `data` (object or null, required): command-specific payload on success; `null` on failure.
- `warnings` (array, required): list of non-fatal advisories. Each element MUST contain `code` (string, dot-separated) and `message` (string). The array MUST be present even when empty.
- `error` (object or null, required): `null` on success; on failure MUST contain `code` (string, dot-separated), `message` (string), and `details` (object, MAY be empty).
- `requestId` (string, required): an opaque identifier of the form `req_<uuid_v4_hex>` that uniquely identifies the CLI invocation. The value SHALL be regenerated for every invocation unless the environment variable `SPECLINK_TEST_REQUEST_ID` is set, in which case its value SHALL be used verbatim.

The envelope MUST NOT contain any field whose key matches `token`, `access_token`, `refresh_token`, `api_key`, `password`, or `secret` (case-insensitive). Nested objects MUST also be redacted.

#### Scenario: Successful command emits success envelope

- **WHEN** a CLI command completes successfully with `--json` set
- **THEN** stdout contains exactly one line of valid JSON parseable by `serde_json::from_str`
- **AND** the parsed object has `ok = true`, `data` is a non-null object, `warnings` is an array, `error` is `null`, and `requestId` matches `^req_[0-9a-f]{32}$`

##### Example: propose create success envelope

- **GIVEN** invocation `speclink propose create --change demo --summary "test" --json`
- **WHEN** the command completes successfully
- **THEN** stdout contains a single line equivalent to:

```json
{"ok":true,"data":{"changeId":"demo","state":"proposed","artifactPath":".speclink/changes/demo/proposal.md","mode":"local"},"warnings":[],"error":null,"requestId":"req_0123456789abcdef0123456789abcdef"}
```

#### Scenario: Failed command emits failure envelope

- **WHEN** a CLI command fails with `--json` set
- **THEN** stdout contains exactly one line of valid JSON
- **AND** the parsed object has `ok = false`, `data` is `null`, `error` is a non-null object with `code` (dot-separated string), `message` (non-empty string), and `details` (object)
- **AND** the process exit code matches the failure category defined in the exit-code requirement below

#### Scenario: Secret-named field is redacted before serialization

- **GIVEN** a command implementation that internally holds a struct with field `access_token`
- **WHEN** the command serializes its data with `--json`
- **THEN** the emitted JSON MUST NOT contain the literal field name `access_token` or its value
- **AND** the JSON MUST NOT contain any field whose key matches `token`, `refresh_token`, `api_key`, `password`, or `secret` (case-insensitive)

---
### Requirement: Stable exit-code table

Every SpecLink CLI command SHALL terminate with one of the following exit codes. The mapping from internal error to exit code SHALL be deterministic; identical input SHALL produce identical exit codes across runs and platforms.

| Code | Meaning                                             |
| ---- | --------------------------------------------------- |
| 0    | Success                                             |
| 1    | General error (filesystem, internal, unclassified)  |
| 2    | User input error (invalid argument, bad TOML, etc.) |
| 3    | Validation failed (reserved; unused in this change) |
| 4    | Analyzer found blocking issues (reserved; unused)   |
| 5    | Provider unavailable                                |
| 6    | Auth required but no fallback allowed               |
| 7    | Conflict (reserved; unused in this change)          |

Exit codes 3, 4, and 7 are reserved for future capabilities and SHALL NOT be produced by any command in this change. Any unclassified error SHALL produce exit code 1.

#### Scenario: Invalid argument produces exit code 2

- **WHEN** the user invokes a CLI command with a missing required flag or a malformed value
- **THEN** clap emits a parse error to stderr
- **AND** the process exit code is 2
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "input.invalid"`

#### Scenario: Remote provider configured but not authenticated, fallback disabled

- **GIVEN** a project config with `provider = "acme"` and `fallback = "disabled"`
- **AND** no auth token is configured for `acme`
- **WHEN** any AI workflow CLI command is invoked
- **THEN** the process exit code is 6
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "provider.not_authenticated"`

#### Scenario: Successful invocation produces exit code 0

- **WHEN** the command completes its intended action without error
- **THEN** the process exit code is 0
- **AND** if `--json` is set, `ok = true`

---
### Requirement: Error code naming convention

Every error and warning emitted in the envelope SHALL use a dot-separated identifier of the form `<capability>.<short_snake_case_reason>`. The capability prefix SHALL match a spec capability name (without the `cli-` prefix where applicable). The reason segment SHALL use lowercase ASCII letters, digits, and underscores only.

This change defines the following error codes:

- `input.invalid` — user supplied an invalid argument, flag value, or stdin payload
- `change.already_exists` — a change with the given id already exists
- `change.invalid_id` — change id does not match the required kebab-case pattern
- `provider.not_authenticated` — a remote provider is configured but no valid auth token is present
- `provider.unavailable` — a configured provider cannot be reached
- `internal.error` — an unclassified internal failure

#### Scenario: New error code conforms to the naming convention

- **GIVEN** any error code emitted by a CLI command in this change
- **WHEN** the code is inspected
- **THEN** the code matches the regular expression `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`
- **AND** the capability prefix corresponds to an existing spec capability name

---
### Requirement: Machine-interface flag semantics

Every SpecLink CLI command SHALL accept the following flags with consistent semantics:

- `--json`: emit a single envelope JSON object to stdout. Without this flag, stdout MAY contain human-readable text and the envelope SHALL NOT be emitted.
- `--no-color`: disable ANSI color codes in human-readable stderr output. The flag SHALL have no effect on stdout when `--json` is set.
- `--quiet`: suppress all human-readable progress output on stderr at tracing level INFO and below. Errors at WARN and ERROR levels SHALL still be emitted to stderr.
- `--stdin`: accept input from stdin. The accepted format SHALL be defined per command. In this change, no command accepts stdin; any command receiving `--stdin` in this change SHALL produce exit code 2 with `error.code = "input.invalid"`.

#### Scenario: `--json` and `--quiet` together

- **GIVEN** invocation `speclink propose create --change demo --summary "test" --json --quiet`
- **WHEN** the command runs successfully
- **THEN** stdout contains exactly one JSON envelope line
- **AND** stderr contains no output below WARN level

#### Scenario: Cross-platform stdout line ending

- **GIVEN** invocation with `--json`
- **WHEN** stdout is captured on Windows
- **THEN** the captured bytes end with exactly one `\n` (LF), not `\r\n`
- **AND** the same invocation on macOS and Linux produces the same byte sequence

---
### Requirement: Secret redaction in tracing output

When the CLI emits tracing logs to stderr, any field or value matching a known secret pattern SHALL be replaced with the literal string `[REDACTED]` before emission. Known secret patterns include any field named `token`, `access_token`, `refresh_token`, `api_key`, `password`, `secret`, or values matching `Bearer <whitespace><non-whitespace>+`.

#### Scenario: Bearer token in URL is redacted in tracing

- **GIVEN** internal code that logs a struct containing `authorization: "Bearer abc123"` via tracing
- **WHEN** tracing emits the log line to stderr
- **THEN** the line contains `[REDACTED]` in place of `Bearer abc123`