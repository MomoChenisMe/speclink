## ADDED Requirements

### Requirement: `propose create` command surface

The CLI SHALL provide a subcommand `speclink propose create` with the following surface:

- **Command form**: `speclink propose create --change <change-id> --summary <summary-text> [--json] [--no-color] [--quiet]`
- **Required flags**:
  - `--change <change-id>`: a kebab-case identifier matching the change-id validation rules defined in the `local-provider-storage` capability
  - `--summary <summary-text>`: a non-empty UTF-8 string describing the change in one line. Maximum length 200 characters.
- **Optional flags**: the machine-interface flags `--json`, `--no-color`, `--quiet` as defined in the `cli-machine-interface` capability.
- **Stdin**: this command SHALL NOT accept stdin in this change. If invoked with `--stdin`, the command SHALL exit with code 2 and `error.code = "input.invalid"`.

The command SHALL belong to the AI-skill-callable command group. The command SHALL NOT prompt for input interactively; all required information SHALL be passed via flags.

#### Scenario: Missing --change flag

- **WHEN** the user runs `speclink propose create --summary "test"` without `--change`
- **THEN** the process exit code is 2
- **AND** stderr contains a clap usage message naming the missing `--change` flag
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "input.invalid"`

#### Scenario: Empty --summary value

- **WHEN** the user runs `speclink propose create --change demo --summary ""`
- **THEN** the process exit code is 2
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "input.invalid"`
- **AND** `error.message` indicates the summary is empty

#### Scenario: --summary exceeds maximum length

- **GIVEN** a summary string of 201 characters
- **WHEN** the user runs `speclink propose create --change demo --summary "<201-char string>"`
- **THEN** the process exit code is 2
- **AND** if `--json` is set, stdout contains a failure envelope with `error.code = "input.invalid"`

### Requirement: Successful proposal creation produces a defined side effect

On successful invocation, the command SHALL perform the following side effects in order. Each step SHALL succeed before the next runs; failure at any step SHALL trigger the cleanup behavior defined in the `local-provider-storage` capability.

1. Resolve the active provider per the `provider-resolution` capability
2. Verify the change id does not already exist as a directory under `.speclink/changes/`
3. Create the directory `.speclink/changes/<change-id>/`
4. Write `.speclink/changes/<change-id>/proposal.md` with content equal to `## Why\n\n<summary>\n` where `<summary>` is the `--summary` value verbatim
5. Write `.speclink/changes/<change-id>/metadata.json` containing the lifecycle metadata defined below
6. Open or create `.speclink/state.db`
7. Execute `INSERT OR REPLACE INTO in_progress_change (change_id, created_at) VALUES (?, ?)` with the change id and current UTC timestamp
8. Emit the `--json` envelope (if `--json` is set) and exit with code 0

The `metadata.json` content SHALL be:

```json
{
  "changeId": "<change-id>",
  "state": "proposed",
  "createdAt": "<ISO 8601 UTC timestamp>",
  "createdBy": {
    "type": "agent",
    "name": "<value from --created-by-name flag, or empty string when flag absent>"
  }
}
```

In this change, no `--created-by-name` flag is defined; the `createdBy.name` field SHALL be the empty string. Future changes MAY add the flag.

#### Scenario: Successful creation in empty directory

- **GIVEN** a temporary directory with no existing `.speclink/` directory
- **WHEN** the user runs `speclink propose create --change demo --summary "test summary" --json`
- **THEN** the process exit code is 0
- **AND** stdout contains exactly one JSON envelope line
- **AND** the file `.speclink/changes/demo/proposal.md` contains exactly `## Why\n\ntest summary\n`
- **AND** the file `.speclink/changes/demo/metadata.json` contains valid JSON with `state = "proposed"`
- **AND** the SQLite query `SELECT change_id FROM in_progress_change` returns `demo`

##### Example: stdout envelope on success

- **GIVEN** invocation `speclink propose create --change demo --summary "test" --json` with `SPECLINK_TEST_REQUEST_ID=req_00000000000000000000000000000000`
- **WHEN** the command succeeds
- **THEN** stdout equals the following single line followed by `\n`:

```json
{"ok":true,"data":{"changeId":"demo","state":"proposed","artifactPath":".speclink/changes/demo/proposal.md","mode":"local"},"warnings":[],"error":null,"requestId":"req_00000000000000000000000000000000"}
```

### Requirement: JSON output schema for propose create

When `--json` is set and the command succeeds, the envelope's `data` field SHALL contain the following object schema:

```json
{
  "changeId": "<string, kebab-case>",
  "state": "proposed",
  "artifactPath": "<string, POSIX-style relative path to proposal.md>",
  "mode": "local"
}
```

- `changeId` SHALL equal the value passed via `--change`
- `state` SHALL always be the literal string `"proposed"` on success
- `artifactPath` SHALL be the path to the proposal.md file relative to the project base, using forward slashes regardless of OS
- `mode` SHALL be the literal string `"local"` in this change (no other provider implementations exist)

#### Scenario: Schema fields present and correctly typed

- **WHEN** `propose create` succeeds with `--json`
- **THEN** parsing `data` as the schema above produces no extra or missing fields
- **AND** `data.changeId` is a string
- **AND** `data.state` is the literal `"proposed"`
- **AND** `data.artifactPath` starts with `.speclink/changes/`
- **AND** `data.mode` is the literal `"local"`

### Requirement: Failure mapping

The command SHALL map internal failures to exit codes and error codes as follows:

| Failure Condition                                              | Exit Code | Error Code                  |
| -------------------------------------------------------------- | --------- | --------------------------- |
| Change id is invalid (does not match kebab-case rules)         | 2         | `change.invalid_id`         |
| Summary is empty or exceeds 200 characters                     | 2         | `input.invalid`             |
| Change directory `.speclink/changes/<change-id>/` already exists | 1         | `change.already_exists`     |
| Remote provider configured but unauthenticated + fallback off  | 6         | `provider.not_authenticated` |
| Project config is malformed TOML                               | 2         | `input.invalid`             |
| Filesystem write fails (permissions, disk full, etc.)          | 1         | `internal.error`            |
| SQLite operation fails                                         | 1         | `internal.error`            |
| Any unclassified internal error                                | 1         | `internal.error`            |

A successful run SHALL NOT emit any of these error codes.

#### Scenario: Change already exists

- **GIVEN** `.speclink/changes/demo/` already exists
- **WHEN** the user runs `speclink propose create --change demo --summary "x" --json`
- **THEN** the process exit code is 1
- **AND** stdout contains a failure envelope with `error.code = "change.already_exists"`
- **AND** the existing `.speclink/changes/demo/` directory is unmodified

#### Scenario: Remote configured but no auth with fallback disabled

- **GIVEN** a project config with `provider = "acme"`, `fallback = "disabled"`, no auth token
- **WHEN** the user runs `speclink propose create --change demo --summary "x" --json`
- **THEN** the process exit code is 6
- **AND** stdout contains a failure envelope with `error.code = "provider.not_authenticated"`
- **AND** the directory `.speclink/changes/demo/` does not exist after the failed invocation
- **AND** the SQLite `in_progress_change` table is unchanged

### Requirement: Warning emission on remote-to-local fallback

When the user has configured a remote provider but the CLI falls back to local (because the remote is unauthenticated and `fallback = "local"`), the success envelope SHALL include a warning. The warning SHALL have `code = "provider.not_authenticated"` and a `message` field describing which configured provider was bypassed.

#### Scenario: Fallback succeeds with warning

- **GIVEN** a project config with `provider = "acme"`, `fallback = "local"` (or fallback unset), no auth token
- **WHEN** the user runs `speclink propose create --change demo --summary "x" --json`
- **THEN** the process exit code is 0
- **AND** stdout contains a success envelope with `ok = true`
- **AND** `warnings` contains exactly one entry with `code = "provider.not_authenticated"`
- **AND** the entry's `message` mentions the provider name `acme`
- **AND** `data.mode = "local"`

##### Example: warnings array content

- **GIVEN** the fallback scenario above
- **WHEN** the command succeeds
- **THEN** `warnings` equals:

```json
[
  {
    "code": "provider.not_authenticated",
    "message": "Provider 'acme' is configured but not authenticated. Using local provider fallback."
  }
]
```

### Requirement: Lifecycle transition `draft` to `proposed`

The `propose create` command SHALL be the sole entry point in this change for transitioning a change from non-existence to lifecycle state `proposed`. The implied state machine for this change is:

```
(no change) ---propose create---> proposed
```

The metadata field `state` SHALL equal `"proposed"` immediately after successful invocation. No other lifecycle states are produced by this change; future changes will add `discussed`, `validated`, `packed`, etc.

#### Scenario: Resulting metadata state is proposed

- **WHEN** `propose create` succeeds
- **THEN** `.speclink/changes/<change-id>/metadata.json` contains `"state": "proposed"`
- **AND** no other state value appears in the file

### Requirement: Secret-free output guarantee

The `propose create` command SHALL NOT include any token, refresh token, API key, or password in stdout, stderr, or any file written to disk. Provider credentials (which do not exist in this change but will in future changes) SHALL never appear in `metadata.json`, `proposal.md`, the JSON envelope, or any tracing log.

#### Scenario: No secret keys in JSON output

- **GIVEN** any invocation of `propose create` with `--json`
- **WHEN** the JSON envelope is captured
- **THEN** the JSON does not contain any key whose name matches `token`, `access_token`, `refresh_token`, `api_key`, `password`, or `secret` (case-insensitive)
- **AND** the JSON does not contain any value matching the pattern `Bearer\s+\S+`
