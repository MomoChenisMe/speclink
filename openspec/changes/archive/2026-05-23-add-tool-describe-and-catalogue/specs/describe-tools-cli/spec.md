## ADDED Requirements

### Requirement: `speclink describe-tools` SHALL emit catalogue subsets in three supported formats

The CLI command `speclink describe-tools [--format <json|text|copilot-sdk>] [--filter <op-id>,...] [--categories <c>,...] [--phases <p>,...] [--full] [--json]` SHALL read the catalogue from `Catalogue::all()`, apply the filters described below, serialize the result in the requested format, and exit with status 0 on success. The default `--format` SHALL be `json`. When `--json` is supplied at the global level the command SHALL emit a stable JSON envelope; without `--json` the command SHALL emit the rendered `content` payload directly to stdout followed by a single newline.

#### Scenario: Default invocation emits curated subset as JSON

- **WHEN** the user runs `speclink describe-tools --json` in any working directory
- **THEN** the command SHALL exit with status 0 and the JSON envelope `data.format` SHALL equal `"json"`, `data.content` SHALL be a JSON array of length 12, and every array element SHALL be an object with the keys `id`, `name`, `description`, and `parameters`

##### Example: envelope shape

```json
{
  "ok": true,
  "data": {
    "format": "json",
    "content": [
      {
        "id": "change.create",
        "name": "new_change",
        "description": "Create a new change in proposing state ...",
        "parameters": { "type": "object", "properties": { "name": { "type": "string" } } }
      }
    ]
  }
}
```

#### Scenario: --full switches to the 37-op full set

- **WHEN** the user runs `speclink describe-tools --full --json`
- **THEN** the command SHALL exit with status 0 and `data.content` SHALL be a JSON array of length 37

#### Scenario: --format text emits a markdown table

- **WHEN** the user runs `speclink describe-tools --format text --json`
- **THEN** the command SHALL exit with status 0, `data.format` SHALL equal `"text"`, and `data.content` SHALL be a string whose first non-blank line begins with the pipe character `|` (markdown table header) and which contains exactly 12 data rows plus 1 header row plus 1 separator row when no filter is applied

#### Scenario: --format copilot-sdk emits defineTool descriptors

- **WHEN** the user runs `speclink describe-tools --format copilot-sdk --json`
- **THEN** the command SHALL exit with status 0, `data.format` SHALL equal `"copilot-sdk"`, and `data.content` SHALL be a JSON array where every element is an object with exactly the keys `name`, `description`, and `parameters` and no extra keys

### Requirement: Filter flags SHALL apply as AND intersection

When more than one of `--filter`, `--categories`, `--phases` is supplied, the command SHALL apply them as an AND intersection: the output set SHALL contain only operations that satisfy every supplied filter. The initial set before filtering SHALL be `Catalogue::all()` when `--full` is supplied, otherwise the curated subset. Filter values SHALL be parsed as comma-separated lists when supplied as a single argument and SHALL accumulate when the flag is repeated.

#### Scenario: Categories and filter combine via AND

- **WHEN** the user runs `speclink describe-tools --full --categories change --filter change.delete --json`
- **THEN** the command SHALL exit with status 0 and `data.content` SHALL be a JSON array of length 1 whose single element has `id == "change.delete"`

#### Scenario: Empty intersection returns empty array

- **WHEN** the user runs `speclink describe-tools --full --categories change --filter discuss.new --json`
- **THEN** the command SHALL exit with status 0 and `data.content` SHALL be the empty JSON array `[]`

#### Scenario: Phases filter limits to operations used by a skill phase

- **WHEN** the user runs `speclink describe-tools --full --phases discuss --json`
- **THEN** the command SHALL exit with status 0 and `data.content` SHALL be a JSON array whose elements all have `op.id` starting with `"discuss."`

### Requirement: Unsupported formats SHALL fail fast with `tool.format_not_supported`

The `--format` flag SHALL accept the literal values `json`, `text`, `copilot-sdk`, `copilotkit`, `openai`, `langchain`, `mcp`, and `claude` at parse time. When the resolved format is one of `copilotkit`, `openai`, `langchain`, `mcp`, or `claude`, the command SHALL emit error code `tool.format_not_supported`, SHALL exit with status 2, and SHALL NOT write any catalogue content to stdout. Any value outside the eight literals above SHALL be rejected by the clap parser with the standard "invalid value" message and exit status 2.

#### Scenario: Format mcp is rejected with format_not_supported

- **WHEN** the user runs `speclink describe-tools --format mcp --json`
- **THEN** the command SHALL exit with status 2, the JSON envelope `error.code` SHALL equal `"tool.format_not_supported"`, and `error.hint` SHALL mention that the format is deferred to a post-MVP slice

#### Scenario: Format banana is rejected by clap

- **WHEN** the user runs `speclink describe-tools --format banana`
- **THEN** the command SHALL exit with status 2 and SHALL emit a clap "invalid value" diagnostic to stderr that lists the accepted format values

### Requirement: Unknown filter values SHALL be rejected with category-specific error codes

When `--filter` contains an id that does not match any `op.id` in `Catalogue::all()`, the command SHALL emit error code `tool.unknown_op`, SHALL exit with status 2, and SHALL include the offending id in the error envelope. When `--categories` contains a value that does not match any `op.category` in `Catalogue::all()`, the command SHALL emit error code `tool.unknown_category`, SHALL exit with status 2, and SHALL include the offending category in the error envelope. Validation SHALL run before any rendering so no partial output is written.

#### Scenario: Unknown filter id is rejected

- **WHEN** the user runs `speclink describe-tools --filter no.such.op --json`
- **THEN** the command SHALL exit with status 2 and the JSON envelope `error.code` SHALL equal `"tool.unknown_op"` and `error.message` SHALL contain the string `"no.such.op"`

#### Scenario: Unknown category is rejected

- **WHEN** the user runs `speclink describe-tools --categories bogus --json`
- **THEN** the command SHALL exit with status 2 and the JSON envelope `error.code` SHALL equal `"tool.unknown_category"` and `error.message` SHALL contain the string `"bogus"`

### Requirement: `describe-tools` SHALL be read-only and require no project context

The command SHALL execute successfully outside any SpecLink project (i.e. when `.speclink/` does not exist, when the working directory is not a git working tree, or when `state.db` is missing). The command SHALL NOT read or write `.speclink/`, SHALL NOT read or write `state.db`, SHALL NOT acquire any lock, and SHALL NOT emit any audit event.

#### Scenario: Runs outside a SpecLink project

- **WHEN** the user runs `speclink describe-tools --json` in a directory that contains neither `.git/` nor `.speclink/`
- **THEN** the command SHALL exit with status 0 and SHALL produce the same `data.content` it would produce inside an initialized project with the same flags

#### Scenario: Filesystem is untouched

- **GIVEN** the user records a recursive directory listing of the working tree before running the command
- **WHEN** the user runs `speclink describe-tools --full --json` and then records a second recursive directory listing
- **THEN** the two listings SHALL be identical (no files created, modified, or deleted)
