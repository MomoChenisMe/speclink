## ADDED Requirements

### Requirement: Catalogue SHALL expose exactly 37 operations as a compile-time const slice

The `crates/runtime/` crate SHALL expose a module `catalogue` with a `Catalogue` type providing `Catalogue::all() -> &'static [Operation]` returning exactly 37 `Operation` entries that mirror `doc/protocol/operations.md` (one entry per row of the Index table). The slice SHALL be a `const` or `static` value so the count is fixed at compile time and the binary contains no runtime parsing.

#### Scenario: Catalogue count is 37

- **WHEN** any caller invokes `Catalogue::all().len()`
- **THEN** the returned value SHALL be exactly 37

#### Scenario: Each operation has the canonical id from operations.md

- **WHEN** a caller iterates `Catalogue::all()` and collects `op.id` into a set
- **THEN** the set SHALL equal `{ "project.init", "project.link", "project.unlink", "project.status", "config.read", "config.write", "schema.list", "schema.show", "schema.fork", "schema.delete", "discuss.new", "discuss.list", "discuss.show", "discuss.patch", "discuss.conclude", "discuss.delete", "change.create", "change.list", "change.show", "change.delete", "artifact.write", "artifact.read", "apply.start", "apply.pause", "task.done", "review.approve", "review.reject", "review.history", "archive.run", "spec.list", "spec.show", "instructions.get", "analyze.run", "validate.run", "drift.run", "doctor.run", "tool.describe" }`

##### Example: spot-check three entries

| Catalogue ID | category | cli | tool_binding | mvp | destructive |
| --- | --- | --- | --- | --- | --- |
| `change.create` | `change` | `new change <name>` | `new_change` | `true` | `false` |
| `change.delete` | `change` | `delete change <id>` | `delete_change` | `true` | `true` |
| `tool.describe` | `tool` | `describe-tools` | n/a | `true` | `false` |

### Requirement: Each Operation entry SHALL carry the metadata required by downstream surfaces

Every `Operation` struct entry returned by `Catalogue::all()` SHALL populate the following fields with values consistent with `doc/protocol/operations.md`: `id` (canonical id), `category`, `cli` (CLI binding string), `tool_binding` (snake_case name, or the literal `"n/a"` when no tool binding exists), `sdk_method` (camelCase dotted), `http_endpoint` (HTTP method + path, or the literal `"n/a"` when no HTTP endpoint exists), `mvp` (boolean), `destructive` (boolean), `idempotency` (enum), `lock` (enum), `phases` (enum slice), `curated` (boolean), `description` (one-line summary), and `inputs_schema` (function pointer returning a JSON Schema as `serde_json::Value`).

#### Scenario: Every operation has non-empty id, category, cli, sdk_method, description

- **WHEN** a caller iterates `Catalogue::all()`
- **THEN** for every entry `op.id.is_empty()`, `op.category.is_empty()`, `op.cli.is_empty()`, `op.sdk_method.is_empty()`, and `op.description.is_empty()` SHALL all be `false`

#### Scenario: Tool binding string of "n/a" only for tool.describe

- **WHEN** a caller iterates `Catalogue::all()` and filters by `op.tool_binding == "n/a"`
- **THEN** the resulting set SHALL contain exactly one entry whose `op.id` is `tool.describe`

#### Scenario: Destructive flag matches the Index table

- **WHEN** a caller iterates `Catalogue::all()` and filters by `op.destructive == true`
- **THEN** the resulting set of `op.id` values SHALL equal `{ "change.delete", "discuss.delete", "schema.delete" }`

### Requirement: Catalogue SHALL provide id-keyed lookup

The `Catalogue` type SHALL expose `Catalogue::get(id: &str) -> Option<&'static Operation>`. The lookup SHALL be case-sensitive and SHALL return `Some(op)` when `id` matches an `op.id` value exactly, otherwise `None`.

#### Scenario: Existing id returns Some

- **WHEN** a caller invokes `Catalogue::get("change.create")`
- **THEN** the result SHALL be `Some(op)` with `op.id == "change.create"`

#### Scenario: Unknown id returns None

- **WHEN** a caller invokes `Catalogue::get("no.such.op")`
- **THEN** the result SHALL be `None`

#### Scenario: Lookup is case-sensitive

- **WHEN** a caller invokes `Catalogue::get("CHANGE.CREATE")`
- **THEN** the result SHALL be `None`

### Requirement: Catalogue SHALL mark exactly 12 operations as curated for Layer 1 SDK subset

The `Operation::curated` field SHALL be `true` for exactly the 12 operation ids that constitute the Layer 1 curated subset defined in `doc/speclink-design.md` §22.2: `discuss.new`, `discuss.patch`, `discuss.conclude`, `change.create`, `artifact.write`, `artifact.read`, `apply.start`, `task.done`, `review.approve`, `review.reject`, `archive.run`, `instructions.get`. All other 25 operations SHALL have `curated == false`.

#### Scenario: Curated count is 12

- **WHEN** a caller invokes `Catalogue::all().iter().filter(|op| op.curated).count()`
- **THEN** the returned value SHALL be exactly 12

#### Scenario: Curated set matches design.md §22.2

- **WHEN** a caller collects the ids of curated operations into a set
- **THEN** the set SHALL equal `{ "discuss.new", "discuss.patch", "discuss.conclude", "change.create", "artifact.write", "artifact.read", "apply.start", "task.done", "review.approve", "review.reject", "archive.run", "instructions.get" }`

### Requirement: Catalogue SHALL provide JSON Schema for every operation via inputs_schema

The `Operation::inputs_schema` field SHALL be a `fn() -> serde_json::Value` function pointer that returns a JSON Schema Draft 2020-12 object describing the operation's request inputs. The returned value SHALL be deterministic across calls (same input always produces the same JSON), SHALL have top-level `"type": "object"`, and SHALL conform to JSON Schema syntax such that `serde_json::from_value::<serde_json::Value>(schema)` succeeds.

#### Scenario: Every operation produces a valid JSON Schema object

- **WHEN** a caller iterates `Catalogue::all()` and invokes `(op.inputs_schema)()` for each entry
- **THEN** each returned value SHALL be a `serde_json::Value::Object`, SHALL contain the key `"type"` with value `"object"`, and SHALL be parseable by a JSON Schema Draft 2020-12 validator without syntax errors

#### Scenario: Schema function is deterministic

- **WHEN** a caller invokes `(op.inputs_schema)()` twice for the same `op`
- **THEN** the two returned values SHALL be equal under `serde_json::Value::eq`

### Requirement: Catalogue SHALL stay in sync with doc/protocol/operations.md via a CI snapshot test

The crate `crates/runtime/` SHALL include an integration test at `crates/runtime/tests/catalogue_doc_sync.rs` that reads `doc/protocol/operations.md`, parses the Index table, and asserts: (a) the row count equals `Catalogue::all().len()`; (b) for every row, the `Catalogue ID`, `Category`, `CLI`, `Tool binding`, `MVP` (`✓` → `true`, `[deferred]` → `false`), and `Destructive` (`⚠` → `true`, `—` → `false`) columns equal the corresponding fields on the matching `Operation` entry. The test SHALL fail the CI build when any mismatch is detected.

#### Scenario: Test passes when catalogue matches doc

- **WHEN** the integration test runs and every Index row matches its corresponding `Operation` entry
- **THEN** the test SHALL exit with status 0 and SHALL NOT print any failure diagnostic

#### Scenario: Test fails when catalogue drifts from doc

- **WHEN** the integration test runs and the catalogue declares 36 operations while the Index lists 37
- **THEN** the test SHALL fail and the failure message SHALL identify the missing or extra operation id
