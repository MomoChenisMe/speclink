### Requirement: Five-level provider resolution priority

The SpecLink CLI SHALL resolve the active provider for every AI workflow command by evaluating the following sources in order and selecting the first non-empty match:

1. The `--provider` command-line flag (if the command accepts it)
2. The `provider` field of the project config (`.speclink/config.toml` found by walking up from CWD)
3. The provider of the global active profile (`<config_dir>/speclink/config.toml`, field `active_profile` to `[profiles.<name>].provider`)
4. The `SPECLINK_PROVIDER` environment variable
5. The local filesystem provider (always available, never fails to instantiate)

Each lower-priority source SHALL be consulted only when all higher-priority sources are empty or unset. If multiple sources name the same provider, the highest-priority one SHALL be considered authoritative for warning attribution.

#### Scenario: Project config selects a provider

- **GIVEN** a project config containing `provider = "acme"`
- **AND** a global config that does not name `acme` under `[providers]`
- **WHEN** any AI workflow command runs without `--provider`
- **THEN** the CLI selects the provider named `acme` from the project config
- **AND** if `acme` is type `http` and no auth token is present, fallback rules apply

#### Scenario: All sources empty falls through to local

- **GIVEN** no command-line flag, no project config, no global config, and no `SPECLINK_PROVIDER` environment variable
- **WHEN** any AI workflow command runs
- **THEN** the CLI selects the local filesystem provider
- **AND** the resolved `mode` field in any `--json` data payload is `"local"`

##### Example: priority interaction table

| Flag        | Project config | Global active profile | Env var      | Selected provider          |
| ----------- | -------------- | --------------------- | ------------ | -------------------------- |
| `--provider acme`  | `provider = "billing"`  | `acme`                | unset        | `acme` (flag wins)        |
| unset       | `provider = "billing"`  | `acme`                | unset        | `billing` (project wins)  |
| unset       | unset                   | `acme`                | unset        | `acme` (global wins)      |
| unset       | unset                   | unset                 | `acme`       | `acme` (env wins)         |
| unset       | unset                   | unset                 | unset        | local fallback             |


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

### Requirement: Local fallback is always available unless explicitly disabled

The local filesystem provider SHALL be instantiable at any time without external dependencies. When a higher-priority remote provider is selected but cannot be used (because it is unauthenticated or unreachable), the CLI SHALL fall back to the local provider unless the active fallback policy is `disabled`.

The fallback policy SHALL be read from the project config field `fallback` and SHALL accept the values `"local"` (default) and `"disabled"`. Any other value SHALL be treated as a user input error.

#### Scenario: Remote provider unauthenticated, fallback enabled

- **GIVEN** a project config with `provider = "acme"`, `fallback = "local"` (or `fallback` unset)
- **AND** the `acme` provider has type `http` and no auth token is configured
- **WHEN** an AI workflow command runs
- **THEN** the CLI falls back to the local filesystem provider
- **AND** the `--json` output contains a warning with `code = "provider.not_authenticated"`
- **AND** the process exit code is 0 (or the command-specific success code)

#### Scenario: Remote provider unauthenticated, fallback disabled

- **GIVEN** a project config with `provider = "acme"` and `fallback = "disabled"`
- **AND** the `acme` provider has type `http` and no auth token is configured
- **WHEN** an AI workflow command runs
- **THEN** the CLI does not fall back
- **AND** the process exit code is 6
- **AND** the `--json` output contains a failure envelope with `error.code = "provider.not_authenticated"`

#### Scenario: Invalid fallback value

- **GIVEN** a project config with `fallback = "remote"` (an unrecognized value)
- **WHEN** any AI workflow command runs
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "input.invalid"`


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

### Requirement: Configuration file locations and discovery

The CLI SHALL discover configuration files using the following rules:

- **Project config**: the CLI SHALL search upward from the current working directory for a `.speclink/config.toml` file, stopping at the first match, at the filesystem root, or at a directory containing a `.git` directory (whichever comes first). If no project config is found, the CLI SHALL proceed as if the file does not exist.
- **Global config**: the CLI SHALL read `<config_home>/speclink/config.toml`, where `<config_home>` is:
  - The value of the `SPECLINK_CONFIG_HOME` environment variable, if set and non-empty
  - Otherwise, the platform-specific user config directory: `%APPDATA%` on Windows, `~/.config` on Linux, `~/Library/Application Support` on macOS
- **Missing files**: a missing project or global config file SHALL NOT be treated as an error.
- **Unreadable or malformed files**: a TOML parse error or filesystem read error SHALL produce exit code 2 with `error.code = "input.invalid"`.

#### Scenario: Project config search stops at git root

- **GIVEN** a directory tree where `.speclink/config.toml` exists at `/home/user/project/.speclink/config.toml`
- **AND** the current working directory is `/home/user/project/src/foo`
- **AND** `/home/user/project/.git` exists
- **WHEN** the CLI searches for project config
- **THEN** the CLI finds and loads `/home/user/project/.speclink/config.toml`
- **AND** the CLI does not search above `/home/user/project`

#### Scenario: SPECLINK_CONFIG_HOME overrides global config location

- **GIVEN** the environment variable `SPECLINK_CONFIG_HOME=/tmp/test-config` is set
- **AND** a file exists at `/tmp/test-config/speclink/config.toml`
- **WHEN** the CLI loads global config
- **THEN** the CLI reads from `/tmp/test-config/speclink/config.toml`
- **AND** the CLI does not read from `%APPDATA%`, `~/.config`, or `~/Library/Application Support`

#### Scenario: Malformed TOML produces input.invalid

- **GIVEN** a project config file containing `provider = ` (invalid TOML)
- **WHEN** any AI workflow command runs
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "input.invalid"`
- **AND** `error.details` contains a `file` field naming the malformed file


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

### Requirement: Resolution result reports the local-fallback reason

When the CLI emits a `--json` data payload that includes the resolved provider mode, the payload SHALL distinguish how the local provider was selected. The `mode` field SHALL be one of:

- `"local"` when the local provider was selected for any reason in this change's scope

In this change, the `--json` data payload's `mode` field SHALL always be `"local"` because no HTTP provider implementation exists. The reason for the local selection SHALL be recorded in the warnings array when applicable (`provider.not_authenticated` warning indicates fallback from an unauthenticated remote).

#### Scenario: No remote configured, mode is local with no warnings

- **GIVEN** no project config, no global config, and no environment variables
- **WHEN** a CLI command produces a `--json` data payload
- **THEN** `data.mode = "local"`
- **AND** `warnings` is an empty array

#### Scenario: Remote configured but no auth, mode is local with warning

- **GIVEN** a project config with `provider = "acme"`, `fallback = "local"`, no auth token
- **WHEN** a CLI command produces a `--json` data payload
- **THEN** `data.mode = "local"`
- **AND** `warnings` contains exactly one warning with `code = "provider.not_authenticated"`


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

### Requirement: Provider resolution is testable independently of I/O

The provider resolution logic SHALL be exposed as a pure function that takes loaded config values, flag values, and environment values as inputs and returns the resolved provider together with any warnings. Filesystem and environment lookup SHALL happen outside the resolution function so that unit tests can supply synthetic inputs covering each of the five priority levels.

#### Scenario: Unit test supplies inputs synthetically

- **GIVEN** a unit test that constructs `ResolutionInputs` with `flag_provider = None`, `project_config = Some(ProjectConfig { provider: Some("acme"), fallback: FallbackPolicy::Local })`, `global_config = None`, `env_provider = None`
- **WHEN** the test calls `resolve(inputs)`
- **THEN** the function returns a `ResolvedProvider::Local { reason: LocalReason::FallbackFromUnauthenticated }` and a warning vector containing `provider.not_authenticated`
- **AND** no filesystem or environment access occurs during the call

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

### Requirement: Five-level provider resolution priority

The SpecLink CLI SHALL resolve the active provider for every AI workflow command by evaluating the following sources in order and selecting the first non-empty match:

1. The `--provider` command-line flag (if the command accepts it)
2. The `provider` field of the project config (`.speclink/config.toml` found by walking up from CWD)
3. The provider of the global active profile (`<config_dir>/speclink/config.toml`, field `active_profile` to `[profiles.<name>].provider`)
4. The `SPECLINK_PROVIDER` environment variable
5. The local filesystem provider (always available, never fails to instantiate)

Each lower-priority source SHALL be consulted only when all higher-priority sources are empty or unset. If multiple sources name the same provider, the highest-priority one SHALL be considered authoritative for warning attribution.

#### Scenario: Project config selects a provider

- **GIVEN** a project config containing `provider = "acme"`
- **AND** a global config that does not name `acme` under `[providers]`
- **WHEN** any AI workflow command runs without `--provider`
- **THEN** the CLI selects the provider named `acme` from the project config
- **AND** if `acme` is type `http` and no auth token is present, fallback rules apply

#### Scenario: All sources empty falls through to local

- **GIVEN** no command-line flag, no project config, no global config, and no `SPECLINK_PROVIDER` environment variable
- **WHEN** any AI workflow command runs
- **THEN** the CLI selects the local filesystem provider
- **AND** the resolved `mode` field in any `--json` data payload is `"local"`

##### Example: priority interaction table

| Flag        | Project config | Global active profile | Env var      | Selected provider          |
| ----------- | -------------- | --------------------- | ------------ | -------------------------- |
| `--provider acme`  | `provider = "billing"`  | `acme`                | unset        | `acme` (flag wins)        |
| unset       | `provider = "billing"`  | `acme`                | unset        | `billing` (project wins)  |
| unset       | unset                   | `acme`                | unset        | `acme` (global wins)      |
| unset       | unset                   | unset                 | `acme`       | `acme` (env wins)         |
| unset       | unset                   | unset                 | unset        | local fallback             |

---
### Requirement: Local fallback is always available unless explicitly disabled

The local filesystem provider SHALL be instantiable at any time without external dependencies. When a higher-priority remote provider is selected but cannot be used (because it is unauthenticated or unreachable), the CLI SHALL fall back to the local provider unless the active fallback policy is `disabled`.

The fallback policy SHALL be read from the project config field `fallback` and SHALL accept the values `"local"` (default) and `"disabled"`. Any other value SHALL be treated as a user input error.

#### Scenario: Remote provider unauthenticated, fallback enabled

- **GIVEN** a project config with `provider = "acme"`, `fallback = "local"` (or `fallback` unset)
- **AND** the `acme` provider has type `http` and no auth token is configured
- **WHEN** an AI workflow command runs
- **THEN** the CLI falls back to the local filesystem provider
- **AND** the `--json` output contains a warning with `code = "provider.not_authenticated"`
- **AND** the process exit code is 0 (or the command-specific success code)

#### Scenario: Remote provider unauthenticated, fallback disabled

- **GIVEN** a project config with `provider = "acme"` and `fallback = "disabled"`
- **AND** the `acme` provider has type `http` and no auth token is configured
- **WHEN** an AI workflow command runs
- **THEN** the CLI does not fall back
- **AND** the process exit code is 6
- **AND** the `--json` output contains a failure envelope with `error.code = "provider.not_authenticated"`

#### Scenario: Invalid fallback value

- **GIVEN** a project config with `fallback = "remote"` (an unrecognized value)
- **WHEN** any AI workflow command runs
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "input.invalid"`

---
### Requirement: Configuration file locations and discovery

The CLI SHALL discover configuration files using the following rules:

- **Project config**: the CLI SHALL search upward from the current working directory for a `.speclink/config.toml` file, stopping at the first match, at the filesystem root, or at a directory containing a `.git` directory (whichever comes first). If no project config is found, the CLI SHALL proceed as if the file does not exist.
- **Global config**: the CLI SHALL read `<config_home>/speclink/config.toml`, where `<config_home>` is:
  - The value of the `SPECLINK_CONFIG_HOME` environment variable, if set and non-empty
  - Otherwise, the platform-specific user config directory: `%APPDATA%` on Windows, `~/.config` on Linux, `~/Library/Application Support` on macOS
- **Missing files**: a missing project or global config file SHALL NOT be treated as an error.
- **Unreadable or malformed files**: a TOML parse error or filesystem read error SHALL produce exit code 2 with `error.code = "input.invalid"`.

#### Scenario: Project config search stops at git root

- **GIVEN** a directory tree where `.speclink/config.toml` exists at `/home/user/project/.speclink/config.toml`
- **AND** the current working directory is `/home/user/project/src/foo`
- **AND** `/home/user/project/.git` exists
- **WHEN** the CLI searches for project config
- **THEN** the CLI finds and loads `/home/user/project/.speclink/config.toml`
- **AND** the CLI does not search above `/home/user/project`

#### Scenario: SPECLINK_CONFIG_HOME overrides global config location

- **GIVEN** the environment variable `SPECLINK_CONFIG_HOME=/tmp/test-config` is set
- **AND** a file exists at `/tmp/test-config/speclink/config.toml`
- **WHEN** the CLI loads global config
- **THEN** the CLI reads from `/tmp/test-config/speclink/config.toml`
- **AND** the CLI does not read from `%APPDATA%`, `~/.config`, or `~/Library/Application Support`

#### Scenario: Malformed TOML produces input.invalid

- **GIVEN** a project config file containing `provider = ` (invalid TOML)
- **WHEN** any AI workflow command runs
- **THEN** the process exit code is 2
- **AND** the `--json` output contains a failure envelope with `error.code = "input.invalid"`
- **AND** `error.details` contains a `file` field naming the malformed file

---
### Requirement: Resolution result reports the local-fallback reason

When the CLI emits a `--json` data payload that includes the resolved provider mode, the payload SHALL distinguish how the local provider was selected. The `mode` field SHALL be one of:

- `"local"` when the local provider was selected for any reason in this change's scope

In this change, the `--json` data payload's `mode` field SHALL always be `"local"` because no HTTP provider implementation exists. The reason for the local selection SHALL be recorded in the warnings array when applicable (`provider.not_authenticated` warning indicates fallback from an unauthenticated remote).

#### Scenario: No remote configured, mode is local with no warnings

- **GIVEN** no project config, no global config, and no environment variables
- **WHEN** a CLI command produces a `--json` data payload
- **THEN** `data.mode = "local"`
- **AND** `warnings` is an empty array

#### Scenario: Remote configured but no auth, mode is local with warning

- **GIVEN** a project config with `provider = "acme"`, `fallback = "local"`, no auth token
- **WHEN** a CLI command produces a `--json` data payload
- **THEN** `data.mode = "local"`
- **AND** `warnings` contains exactly one warning with `code = "provider.not_authenticated"`

---
### Requirement: Provider resolution is testable independently of I/O

The provider resolution logic SHALL be exposed as a pure function that takes loaded config values, flag values, and environment values as inputs and returns the resolved provider together with any warnings. Filesystem and environment lookup SHALL happen outside the resolution function so that unit tests can supply synthetic inputs covering each of the five priority levels.

#### Scenario: Unit test supplies inputs synthetically

- **GIVEN** a unit test that constructs `ResolutionInputs` with `flag_provider = None`, `project_config = Some(ProjectConfig { provider: Some("acme"), fallback: FallbackPolicy::Local })`, `global_config = None`, `env_provider = None`
- **WHEN** the test calls `resolve(inputs)`
- **THEN** the function returns a `ResolvedProvider::Local { reason: LocalReason::FallbackFromUnauthenticated }` and a warning vector containing `provider.not_authenticated`
- **AND** no filesystem or environment access occurs during the call