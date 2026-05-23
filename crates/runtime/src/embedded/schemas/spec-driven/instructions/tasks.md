# Instructions: tasks

Create the task list that breaks down the implementation work.

## What to write

Follow the template format exactly — the apply phase parses checkboxes for progress tracking, and non-checkbox tasks are NOT tracked.

- Group related tasks under `## <number>. <group name>` headings.
- Each task is a checkbox: `- [ ] <task-id> <task description>`.
- Use 1.1 / 1.2 / 2.1 / 2.2 style identifiers — they are stable references for `task.done`.
- Order tasks by dependency. What must be done first appears first.

### Behavior + verification (REQUIRED for every non-trivial task)

Each task description SHALL state:

1. **Behavior or contract delivered** — what is observably true when the task is complete. "Edit file X" is NOT a behavior; it is locator context.
2. **Verification target** — how completion is proved: a test name, a CLI invocation, an analyzer check, a manual assertion, or a content review on a generated artifact. A task without a verification target is not a valid task.

File paths MAY appear in task descriptions, but only as locator context — never as the task itself. File-edit-only tasks (e.g., "Update `foo.rs` to handle Y") are invalid unless they also describe the resulting behavior and how it is verified.

### Parallel markers

If the project's config enables `parallel_tasks`, mark independent tasks with `[P]`: `- [ ] 1.1 [P] <description>`. Two tasks may both be `[P]` when they target different files (or disjoint regions of the same file) and have no data dependency on each other. When in doubt, leave `[P]` off.

## Rules

- Cross-reference: every `### Requirement: <name>` in specs SHOULD appear as a case-insensitive substring in at least one task description. The analyzer flags requirements with no covering task.
- Cross-reference: every `### <heading>` in `design.md` SHOULD appear (case-insensitive substring) in at least one task description. The analyzer flags decision headings with no covering task.
- Task granularity: aim for tasks completable in one session (~30 min – 2 hours). Tasks taking more than 1 hour SHOULD be split.
- TDD discipline: when the project enables TDD (`.spectra.yaml` / `.speclink/config.yaml`), follow Red → Green → Refactor. Pair test tasks with implementation tasks (e.g., `1.1 Write failing test ...` → `1.2 Implement to pass test ...`).

## What NOT to write

- Don't list "write tests" as a separate trailing group. Tests are paired with each implementation task.
- Don't use vague verbs: "wire it up", "handle edge cases", "polish". Name the behavior.
- Don't reference line numbers — they drift. Name the function, struct, or behavior instead.
- Don't write tasks that only restate the requirement without naming the work. The task is the unit of work; the requirement is the contract.
