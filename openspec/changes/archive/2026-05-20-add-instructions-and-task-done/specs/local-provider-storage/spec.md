## ADDED Requirements

### Requirement: Tasks.md task id format

Within `.speclink/changes/<change-id>/tasks.md`, individual task items SHALL follow this format:

```
## N. <Section heading>

- [ ] N.M <Task description>
```

Where:

- `N` is a positive decimal integer matching the section heading number on a line of the form `^## (\d+)\. ` (with no leading zeros)
- `M` is a positive decimal integer (no leading zeros)
- The task line matches the pattern `^- \[( |x)\] (\d+)\.(\d+) `

The task id is the concatenation `N.M` (e.g., `1.1`, `10.3`). Task ids SHALL be unique within a single `tasks.md` file. Section numbers SHALL be unique within a single `tasks.md` file.

Three-level task ids (`N.M.P`) are NOT supported in this change. Lines matching `^- \[( |x)\] \d+\.\d+\.\d+ ` SHALL cause `tasks.parse_error` when the file is parsed by `mark_task_done`.

#### Scenario: Valid task id structure

- **GIVEN** a `tasks.md` file with content:

  ```
  ## 1. Setup

  - [ ] 1.1 Install dependencies
  - [ ] 1.2 Configure env

  ## 2. Implementation

  - [x] 2.1 Write parser
  ```
- **WHEN** the parser reads this file
- **THEN** task ids `1.1`, `1.2`, and `2.1` are recognized
- **AND** their section numbers (1 for first two, 2 for third) match the `## N.` headings above them

#### Scenario: Three-level task id rejected

- **GIVEN** a `tasks.md` file containing `- [ ] 1.1.1 Subtask`
- **WHEN** the parser reads this file
- **THEN** parsing fails with `tasks.parse_error`

#### Scenario: Section number mismatch detected

- **GIVEN** a `tasks.md` file with `## 1. Setup` heading followed by `- [ ] 2.1 Task` (section number 2 inside section 1)
- **WHEN** the parser reads this file
- **THEN** parsing fails with `tasks.parse_error` reporting the mismatch

### Requirement: Atomic tasks.md update for task done

The local provider SHALL update `tasks.md` atomically when `mark_task_done` is invoked, following the same protocol as the atomic artifact write established in `Multi-artifact atomic write`:

1. Read entire `<change-dir>/tasks.md` content
2. Locate the matching task line by section-aware parsing
3. If found and currently `- [ ]`, modify only the checkbox character in memory (replace `[ ]` with `[x]` on that specific line, leaving all other bytes unchanged including the rest of the line, neighbouring lines, line endings, and trailing whitespace)
4. Write modified content to `<change-dir>/tasks.md.tmp`
5. Rename `tasks.md.tmp` to `tasks.md`
6. On any failure, remove `tasks.md.tmp` if it exists; the original `tasks.md` SHALL remain unchanged

If the task is already `- [x]`, the local provider SHALL skip steps 4–5 entirely (no `.tmp` file is created). The `mark_task_done` return value SHALL still indicate success.

If `<change-dir>/tasks.md` does not exist, the local provider SHALL return `ProviderError::ArtifactMissing { artifact_id: "tasks", change_id }` (mapped to `artifact.missing` with exit code 1).

The local provider SHALL NOT modify `metadata.json` when `mark_task_done` succeeds — task tracking is recorded only in `tasks.md` content.

#### Scenario: Idempotent update creates no tmp file

- **GIVEN** `tasks.md` contains `- [x] 1.1 Already done`
- **WHEN** `mark_task_done("1.1")` is invoked
- **THEN** the function returns success
- **AND** no `tasks.md.tmp` file is created or removed
- **AND** `tasks.md` modification time is unchanged

#### Scenario: Atomic rollback on rename failure

- **GIVEN** `tasks.md` contains `- [ ] 1.1 First task`
- **AND** the rename step (`tasks.md.tmp` → `tasks.md`) fails (e.g., simulated by readonly parent)
- **WHEN** `mark_task_done("1.1")` is invoked
- **THEN** the function returns an `internal.error`
- **AND** `tasks.md` still contains `- [ ] 1.1 First task` (unchanged)
- **AND** no `tasks.md.tmp` remains in the change directory

### Requirement: Hardcoded artifact instructions in runtime

The local provider's `get_artifact_instructions` SHALL return instructions composed from hardcoded markdown content shipped in the runtime crate. The CLI SHALL NOT read instructions from external configuration files in this change.

The runtime SHALL provide four embedded markdown files via `include_str!`:

- `crates/runtime/instructions/proposal.md`
- `crates/runtime/instructions/design.md`
- `crates/runtime/instructions/tasks.md`
- `crates/runtime/instructions/spec.md`

These files SHALL be plain markdown with three logical sections (delimited by the runtime's parsing, not by mandatory headings within the markdown itself):

- Body text used for the `instruction` field
- A template block (markdown skeleton) used for the `template` field
- A rules list used for the `rules` field

The exact format of these embedded files is internal to runtime and MAY change without a spec change, provided the resulting `ArtifactInstructions` JSON shape (defined in the `cli-instructions` capability) remains stable.

When a future change introduces remote-managed instructions (e.g., via the HTTP provider), the local provider's behavior SHALL remain hardcoded — this requirement applies only to the local provider.

#### Scenario: Each kind produces non-empty instruction content

- **GIVEN** the runtime crate is built
- **WHEN** the local provider invokes `get_artifact_instructions` for each of the four kinds (proposal, design, tasks, spec)
- **THEN** for every kind, the returned `ArtifactInstructions` has:
  - non-empty `instruction` string
  - non-empty `template` string
  - at least one entry in `rules`
  - `locale` equal to `"Traditional Chinese (繁體中文)"`
