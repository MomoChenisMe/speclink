## MODIFIED Requirements

### Requirement: New error codes SHALL be registered with stable exit codes

The following error codes SHALL be added to the global error registry with the listed exit codes (per design §17.4):

| Code | Exit | Hint behavior |
|---|---|---|
| `config.not_found` | 2 | Suggest running `speclink init` (only raised by write paths when file missing) |
| `config.malformed` | 3 | Cite the YAML parse error line / column when available |
| `config.key_not_found` | 2 | List the closest matching key paths in the current config (best-effort). When raised from a JSONPath subset parse failure (wildcard / filter / recursive-descent), the `key` field SHALL preserve the user's original `--key` argument literally; the diagnostic reason SHALL appear in `message`, not in `key`. |
| `config.edit_mode_required` | 2 | Tell the caller that `speclink config edit` requires `--stdin`, `--editor <cmd>`, or `$EDITOR` to be set. Raised when none of the three are supplied. |

The error code `state.etag_mismatch` (exit 7) is already registered by earlier slices and SHALL apply to `config.write` without redefinition. Its human-readable `message` SHALL format `expected` / `actual` as plain etag strings (e.g., `expected=v1.abc123def456, actual=v2.000000000000`) and SHALL NOT leak Rust `Debug` formatting wrappers (no literal `Some(...)`, no `None` keyword); when `expected` is absent the message SHALL render it as `<none>`.

Audit-only codes SHALL also be added (these surface only via warnings or audit rows, never as errors):

| Code | Surface |
|---|---|
| `config.external_edit_detected` | JSON envelope `warnings`; one row in `config_change` with `mode='external_edit'` |
| `config.malformed_using_defaults` | JSON envelope `warnings`; no `config_change` row |

#### Scenario: All six codes appear in CLI error registry

- **WHEN** the CLI binary is built with A5 + polish-config-error-messages patches
- **THEN** `speclink describe-errors --json` (or the equivalent test harness query) SHALL include `config.not_found`, `config.malformed`, `config.key_not_found`, `config.edit_mode_required`, `config.external_edit_detected`, and `config.malformed_using_defaults` in the registry output

#### Scenario: `config edit` without input mode emits `config.edit_mode_required`

- **GIVEN** an initialized project where `$EDITOR` env var is unset
- **WHEN** the user runs `speclink config edit --json` (no `--stdin`, no `--editor`)
- **THEN** the command SHALL exit 2, the JSON envelope `error.code` SHALL be `config.edit_mode_required`, and `error.message` SHALL include the literal substring `--stdin` and `$EDITOR` so the caller knows the two ways to retry

#### Scenario: JSONPath parse failure preserves the user's `--key` argument in the error envelope

- **GIVEN** an initialized project with default config
- **WHEN** the user runs `speclink config show --key 'rules.*' --json`
- **THEN** the command SHALL exit 2, `error.code` SHALL be `config.key_not_found`, `error.message` SHALL include the literal substring `rules.*` (the user's original argument), and `error.message` SHALL NOT mention `wildcards not supported` as if it were the key name

#### Scenario: `state.etag_mismatch` message does not leak Rust Debug formatting

- **GIVEN** an initialized project with current config etag `v1.<sha[:12]>`
- **WHEN** the user runs `speclink config set rules.require_code_review true --expected-etag v99.bogus0000000 --json`
- **THEN** the command SHALL exit 7, `error.code` SHALL be `state.etag_mismatch`, `error.message` SHALL contain `v99.bogus0000000` and the current actual etag as plain strings, and SHALL NOT contain the literal substring `Some(` nor end with `)` as a Debug wrapper
