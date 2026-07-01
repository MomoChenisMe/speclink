Create the task list that breaks down the implementation work.

**IMPORTANT: Follow the template below exactly.** The apply phase parses
checkbox format to track progress. Tasks not using `- [ ]` won't be tracked.

Guidelines:
- Group related tasks under ## numbered headings
- Each task MUST be a checkbox: `- [ ] X.Y Task description`
- Use the instructions JSON `locale` for human-readable task group headings
  and task descriptions.
- Preserve machine-readable syntax and technical tokens exactly: markdown
  headings, checkbox markers, task numbers, `[P]` markers, file paths,
  symbol names, commands, API names, and code identifiers MUST NOT be
  translated or localized.
- Tasks should be small enough to complete in one session
- Order tasks by dependency (what must be done first?)
- **Behavior + verification (REQUIRED for every non-trivial task):**
  - Each task MUST state the behavior or contract being delivered — what is
    observably true when the task is complete (user-visible behavior, generated
    artifact contract, CLI/IPC output, or tool behavior). "Edit file X" is
    NOT a behavior; it is supporting context for locating the work.
  - Each task MUST also state how completion is verified — a test name, a
    CLI invocation, an analyzer check, a manual assertion, or a content
    review on a generated artifact. A task without a verification target
    is not a valid task.
  - File paths MAY appear in a task description, but only as locator
    context. The task SHALL still state the behavior or contract on top
    of any file path it mentions.
  - File-edit-only tasks (e.g. "Update file X to handle Y") are invalid
    unless they also describe the resulting behavior and how it is
    verified.
- Cross-referencing (analyzer checks these):
  - Every `### Requirement:` name from specs MUST appear as a case-insensitive substring in at least one task description
  - If design.md exists, every `###` heading from design.md should be referenced in at least one task description

Example:
```
## 1. Setup

- [ ] 1.1 Create new module structure
- [ ] 1.2 Add dependencies to package.json

## 2. Core Implementation

- [ ] 2.1 Implement data export function
- [ ] 2.2 Add CSV formatting utilities
```

Reference specs for what needs to be built. If design.md exists, reference it for how to build it.
Each task should be verifiable - you know when it's done.
