## ADDED Requirements

### Requirement: `speclink instructions <kind>` SHALL return an 11-field envelope for supported artifact and workflow phase kinds

The system SHALL provide a `speclink instructions <kind> [--change <change-id>] [--json] [--role <role-id>] [--discussion <discussion-id>]` command. When `<kind>` is one of `proposal`, `spec`, `design`, `tasks`, `apply`, `ingest`, `archive`, or `commit`, the command SHALL exit with status 0 and produce a JSON envelope whose `data` object SHALL contain exactly these fields: `kind`, `schema_id`, `instruction`, `template`, `context`, `rules`, `dependencies`, `output_path`, `locale`, `available_roles`, `linked_changes_context`.

The op SHALL be registered as catalogue entry 32 (`instructions.get`) of category `meta`, idempotency `idempotent`, lock `none`.

For artifact kinds (`proposal`, `spec`, `design`, `tasks`), `template` SHALL be a non-null string read from the embedded `spec-driven` schema bundle file `templates/<kind>.md`, and `output_path` SHALL be a non-null relative path (`proposal.md`, `spec.md`, `design.md`, `tasks.md`).

For workflow phase kinds (`apply`, `ingest`, `archive`, `commit`), `template` SHALL be `null` and `output_path` SHALL be `null`.

The fields `available_roles` and `linked_changes_context` SHALL always be `null` (reserved for the Phase 2 `add-discuss-ops` slice).

#### Scenario: Get proposal instructions (artifact kind)

- **WHEN** the user runs `speclink instructions proposal --json`
- **THEN** the command SHALL exit 0 and the JSON envelope `data` SHALL contain `kind: "proposal"`, `schema_id: "spec-driven"`, a non-empty string `instruction`, a non-empty string `template`, `output_path: "proposal.md"`, `dependencies: []`, `available_roles: null`, `linked_changes_context: null`

#### Scenario: Get apply instructions (workflow phase kind)

- **WHEN** the user runs `speclink instructions apply --json`
- **THEN** the command SHALL exit 0 and the JSON envelope `data` SHALL contain `kind: "apply"`, `schema_id: "spec-driven"`, a non-empty string `instruction`, `template: null`, `output_path: null`, `dependencies` containing three entries with kinds `proposal`, `spec`, `tasks`, `available_roles: null`, `linked_changes_context: null`

##### Example: Field shape per kind family

| Kind family | `template` | `output_path` | `dependencies` length |
| ----------- | ---------- | ------------- | --------------------- |
| `proposal` | non-null string | `"proposal.md"` | 0 |
| `spec` | non-null string | `"spec.md"` | 1 |
| `design` | non-null string | `"design.md"` | 2 |
| `tasks` | non-null string | `"tasks.md"` | 3 |
| `apply` | `null` | `null` | 3 |
| `ingest` | `null` | `null` | 3 |
| `archive` | `null` | `null` | 2 |
| `commit` | `null` | `null` | 0 |

---

### Requirement: `instructions.get` SHALL reject unknown kinds with `instructions.unknown_kind` and exit 2

The command SHALL exit with status 2 and emit error code `instructions.unknown_kind` when `<kind>` is not in the supported set `{proposal, spec, design, tasks, apply, ingest, archive, commit}`. This SHALL include `kind=discuss` (reserved for Phase 2), any typo, and any other unsupported string. The error envelope `hint` field SHALL list the supported set.

The clap CLI surface SHALL accept `<kind>` as a free `String` (not `ValueEnum`), so runtime rejection emits the canonical `instructions.unknown_kind` envelope rather than a clap parse error.

#### Scenario: Reject `discuss` kind

- **WHEN** the user runs `speclink instructions discuss --json`
- **THEN** the command SHALL exit 2, the JSON envelope `ok` SHALL be `false`, the `error.code` SHALL be `instructions.unknown_kind`, the `error.message` SHALL include the literal string `discuss`, and the `error.hint` SHALL list the eight supported kinds

#### Scenario: Reject arbitrary string

- **WHEN** the user runs `speclink instructions random_kind_xyz --json`
- **THEN** the command SHALL exit 2 and the `error.code` SHALL be `instructions.unknown_kind`

---

### Requirement: `instructions.get` SHALL derive `dependencies[]` from a static artifact DAG table

The `dependencies[]` array in the response SHALL be derived from a compile-time static table that mirrors the `spec-driven` schema artifact DAG. Each dependency entry SHALL be an object with three fields:

- `kind`: string, one of `proposal`/`spec`/`design`/`tasks`
- `capability`: always `null` (multi-instance spec capability resolution is reserved for the Phase 2 `add-spec-canonical-read` slice)
- `path`: string, the dependency artifact's `output_path` (e.g., `"proposal.md"`)

The static table SHALL be:

| Kind | `dependencies[].kind` values (in order) |
| ---- | --------------------------------------- |
| `proposal` | (empty) |
| `spec` | `proposal` |
| `design` | `proposal`, `spec` |
| `tasks` | `proposal`, `spec`, `design` |
| `apply` | `proposal`, `spec`, `tasks` |
| `ingest` | `proposal`, `spec`, `tasks` |
| `archive` | `spec`, `tasks` |
| `commit` | (empty) |

The runtime SHALL NOT query change state or filesystem to compute `dependencies[]`. The table SHALL be co-located in the `Kind` enum impl in `crates/runtime/src/instructions_ops.rs`.

#### Scenario: Tasks dependencies include all three predecessors

- **WHEN** the user runs `speclink instructions tasks --json`
- **THEN** `data.dependencies` SHALL have length 3, the entries SHALL have `kind` values `["proposal", "spec", "design"]` in that order, each entry's `capability` SHALL be `null`, and `path` SHALL be `"proposal.md"`, `"spec.md"`, `"design.md"` respectively

#### Scenario: Proposal has empty dependencies

- **WHEN** the user runs `speclink instructions proposal --json`
- **THEN** `data.dependencies` SHALL be an empty array `[]`

---

### Requirement: `instructions.get` SHALL hydrate `context`, `rules`, and `locale` from config with best-effort fallback to null

The op SHALL invoke `ConfigStore::read_config` (provided by the `config-rw` capability) and read three fields from the returned `Config` struct to populate the response:

- `context` (string) ← `Config.context: Option<String>`
- `rules` (array of strings) ← `Config.instructions.get(<kind>)` where `<kind>` is the requested kind
- `locale` (string) ← `Config.locale: Option<String>`

The `Config` struct SHALL expose these three fields as Optional additive fields (introduced by this slice on top of A5's `config-rw` capability). Existing A5 `Rules` struct (`require_*_review` booleans) and A5 round-trip behavior SHALL NOT be affected.

Each of the three fields SHALL be populated independently:

- If `Config.context` is `None`: the response `context` field SHALL be `null`.
- If `Config.locale` is `None`: the response `locale` field SHALL be `null`.
- If `Config.instructions` does not contain a key matching the requested kind: the response `rules` field SHALL be `null`.
- A5 `LocalConfigStore::read_config` already handles malformed YAML and missing files by returning `Ok(Versioned { value: Config::default(), .. })` with a `config.malformed_using_defaults` warning (per the `config-rw` capability spec). The op SHALL propagate any such warnings on the response envelope `warnings[]` array without remapping. The op SHALL NOT raise `config.malformed` itself; A5 owns that error semantics and read path does not surface it.

The `rules` field's null-vs-empty-array distinction SHALL be preserved: `null` means the requested kind has no entry in `Config.instructions`, while `[]` means the entry exists but is explicitly empty.

#### Scenario: Config file does not exist

- **GIVEN** the project has no `.speclink/config.yaml`
- **WHEN** the user runs `speclink instructions proposal --json`
- **THEN** the command SHALL exit 0, `data.context` SHALL be `null`, `data.rules` SHALL be `null`, `data.locale` SHALL be `null`, and the `warnings[]` array MAY contain a `config.malformed_using_defaults` entry (forwarded from A5)

#### Scenario: Config exists with partial keys

- **GIVEN** `.speclink/config.yaml` contains `locale: "Traditional Chinese (繁體中文)"` but no `context` field and no `instructions.proposal` entry
- **WHEN** the user runs `speclink instructions proposal --json`
- **THEN** the command SHALL exit 0 and `data.locale` SHALL be `"Traditional Chinese (繁體中文)"`, `data.context` SHALL be `null`, `data.rules` SHALL be `null`

#### Scenario: Config malformed forwards A5 warning

- **GIVEN** `.speclink/config.yaml` contains invalid YAML such that A5 `LocalConfigStore::read_config` falls back to `Config::default()` with a `config.malformed_using_defaults` warning
- **WHEN** the user runs `speclink instructions proposal --json`
- **THEN** the command SHALL exit 0 (NOT 7), `data.context` SHALL be `null`, `data.rules` SHALL be `null`, `data.locale` SHALL be `null`, and the `warnings[]` array SHALL contain at least one entry with `code: "config.malformed_using_defaults"`

##### Example: Field independence under partial config

| Config state | `context` | `rules` (for kind=proposal) | `locale` |
| ------------ | --------- | --------------------------- | -------- |
| File missing | `null` | `null` | `null` |
| File exists, all keys present | `"<value>"` | `["<rule1>", ...]` | `"<value>"` |
| File exists, only `locale` set | `null` | `null` | `"<value>"` |
| File exists, `instructions.proposal: []` explicit | `null` | `[]` | `null` |
| File exists, `instructions: {}` (no `proposal` sub-key) | `null` | `null` | `null` |
| File malformed YAML | `null` (A5 fallback to defaults + `config.malformed_using_defaults` warning forwarded) | `null` | `null` |

---

### Requirement: `instructions.get` SHALL verify change existence when `--change <id>` is provided and reject missing changes with `change.not_found`

When the `--change <id>` flag is provided, the op SHALL invoke `ChangeStore::get_change(id)`. If the change does not exist, the op SHALL exit with status 2 and emit error code `change.not_found`. If the change exists, the op SHALL read the change's `schema_id` column (currently always `spec-driven`) and reflect it in the response `data.schema_id` field.

When `--change` is not provided, the op SHALL NOT perform any change-store lookup, and `data.schema_id` SHALL be the active schema (currently always `"spec-driven"`).

The op SHALL NOT alter `instruction`, `template`, `dependencies`, `output_path`, or any other field based on change context in this slice. Capability-list injection and prior-state insertion are reserved for the Phase 2 `add-spec-canonical-read` slice.

#### Scenario: Change exists

- **GIVEN** a change named `my-feature` exists in the project
- **WHEN** the user runs `speclink instructions proposal --change my-feature --json`
- **THEN** the command SHALL exit 0 and `data.schema_id` SHALL be `"spec-driven"`

#### Scenario: Change does not exist

- **GIVEN** no change named `nonexistent-change` exists
- **WHEN** the user runs `speclink instructions proposal --change nonexistent-change --json`
- **THEN** the command SHALL exit 2, `ok` SHALL be `false`, `error.code` SHALL be `change.not_found`, and `error.message` SHALL include the literal string `nonexistent-change`

#### Scenario: No --change flag does not invoke ChangeStore

- **WHEN** the user runs `speclink instructions proposal --json` without `--change`
- **THEN** the command SHALL exit 0 and the implementation SHALL NOT call `ChangeStore::get_change` (verifiable via test double counting invocations)

---

### Requirement: `instructions.get` SHALL load `template` and `instruction` bodies from an embedded `spec-driven` schema bundle compiled into the binary

The runtime SHALL compile the `spec-driven` schema bundle into the binary via `include_str!`. The bundle SHALL reside at `crates/runtime/src/embedded/schemas/spec-driven/` and contain:

- `schema.yaml` — schema descriptor (informational for this slice; runtime SHALL NOT parse it for dispatch)
- `templates/proposal.md`, `templates/spec.md`, `templates/design.md`, `templates/tasks.md` — non-empty markdown skeletons returned as the `template` field for artifact kinds
- `instructions/proposal.md`, `instructions/spec.md`, `instructions/design.md`, `instructions/tasks.md`, `instructions/apply.md`, `instructions/ingest.md`, `instructions/archive.md`, `instructions/commit.md` — non-empty markdown bodies returned as the `instruction` field

The runtime SHALL NOT perform filesystem lookups under `.speclink/schemas/` for this slice. User-overridable schema fork is reserved for the Phase 2 `add-schema-ops` slice.

The hardcoded `Kind::dependencies()` table SHALL match the artifact DAG declared in `schema.yaml` (verified by a Rust test that parses `schema.yaml` and compares dependency edges).

#### Scenario: Templates are non-empty for all artifact kinds

- **WHEN** the user runs `speclink instructions <kind> --json` for each kind in `{proposal, spec, design, tasks}`
- **THEN** the command SHALL exit 0 and `data.template` SHALL be a non-empty string

#### Scenario: Instructions are non-empty for all 8 supported kinds

- **WHEN** the user runs `speclink instructions <kind> --json` for each kind in `{proposal, spec, design, tasks, apply, ingest, archive, commit}`
- **THEN** the command SHALL exit 0 and `data.instruction` SHALL be a non-empty string

#### Scenario: Hardcoded dependency table matches schema.yaml DAG

- **WHEN** the test suite parses `crates/runtime/src/embedded/schemas/spec-driven/schema.yaml` and reads the artifact DAG declaration
- **THEN** for each kind in `{proposal, spec, design, tasks, apply, ingest, archive, commit}`, the kind's dependency edges from `schema.yaml` SHALL equal the result of `Kind::dependencies()` in the runtime

---

### Requirement: `--role` and `--discussion` flags SHALL be accepted by the CLI surface but ignored by the dispatcher

The CLI SHALL accept `--role <role-id>` and `--discussion <discussion-id>` as optional flags to preserve forward-compatible surface for the Phase 2 `add-discuss-ops` slice. The dispatcher SHALL ignore both values in this slice. The CLI help text for each flag SHALL include the literal substring `(reserved for Phase 2)`.

Passing `--role` or `--discussion` SHALL NOT alter `available_roles` or `linked_changes_context` in the response (both remain `null`), and SHALL NOT emit a warning, error, or audit event.

#### Scenario: --role is accepted but ignored

- **WHEN** the user runs `speclink instructions proposal --role pm --json`
- **THEN** the command SHALL exit 0, the response `data.available_roles` SHALL be `null`, and no warning SHALL appear in the `warnings` array

#### Scenario: --discussion is accepted but ignored

- **WHEN** the user runs `speclink instructions discuss --discussion abc-123 --json`
- **THEN** the command SHALL exit 2 with `error.code: "instructions.unknown_kind"` (the `discuss` kind rejection takes precedence over flag handling)

#### Scenario: --role help text mentions Phase 2

- **WHEN** the user runs `speclink instructions --help`
- **THEN** the output SHALL contain the literal substring `(reserved for Phase 2)` adjacent to both `--role` and `--discussion`

