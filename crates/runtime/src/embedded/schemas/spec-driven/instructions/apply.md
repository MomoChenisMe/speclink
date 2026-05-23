# Instructions: apply

This is a workflow phase, not an artifact-producing step. No file is written to the change directory by this kind.

Implement tasks from a completed change. Read the change's `tasks.md`, work through pending tasks one at a time, mark each complete as you go.

## Prerequisites

Before starting:

- The change SHALL have `proposal.md`, all required specs, and `tasks.md` written (i.e., `applyRequires` artifacts must be `done`). If artifacts are missing, run the propose workflow first.
- The project SHALL be initialized. If `project.status` reports not initialized, ask the user to run init first.

## Workflow

### Step 1: Select the change

If a change name is provided, use it. Otherwise:

- Infer from conversation context if the user mentioned one.
- Auto-select if exactly one change exists in `ready` or `in_progress` state.
- If ambiguous, list active and parked changes and ask the user to choose.

If the selected change is parked, ask whether to unpark before continuing.

### Step 2: Enter in_progress

Invoke `apply.start` for the change. The engine assigns an actor record and transitions state from `ready` to `in_progress`. If the change is already `in_progress` for the same actor, this is idempotent.

### Step 3: Read context

Read the change's `proposal.md`, all `specs/**/*.md`, `design.md` (if present), and `tasks.md`. Do NOT skip reading — context compression earlier in the conversation may have lost details.

For each task, re-read the Implementation Contract section of `design.md` that covers the task's scope. The contract is what the task is measured against.

### Step 4: Implement tasks in order

For each pending task in `tasks.md`:

1. Show which task is being worked on.
2. **Detect unclear tasks before writing code.** A task is unclear if it only names files to edit, uses vague verbs ("handle edge cases", "make it work"), or conflicts with the Implementation Contract. When this happens, pause and either update the artifact or ask the user — do NOT silently guess.
3. Before writing code, check for existing utilities (reuse over reinvent), derive values from existing state (no duplication), and confirm spec example values are used verbatim in tests.
4. Make the code changes required for the task. Keep changes minimal and scoped.
5. **Verify before marking done.** Re-read the task description AND the relevant Implementation Contract content. Confirm every requirement stated in the task and every contract item is satisfied. Confirm the named verification target (test, CLI invocation, analyzer check) actually passes.
6. Mark the task complete by invoking `task.done` with the task identifier. The engine updates the checkbox in `tasks.md` AND records which files this task touched.
7. Continue to the next task.

### Step 5: Pause if blocked

Pause and ask the user when:

- A task is genuinely unclear and the artifact lacks the answer.
- Implementation reveals a design issue that requires updating an artifact.
- An error or environment problem blocks progress and root cause is unclear.
- The user interrupts.

## Rules

- **Task tracking is file-based only.** Do NOT use any external task manager or built-in todo tool. The checkboxes in `tasks.md` (maintained via `task.done`) are the single source of truth for progress.
- TDD: when the project enables TDD, write the failing test first for each task, then implement to pass. Use spec example values verbatim in tests where examples exist.
- Do not skip tasks. If a task is blocked, pause — do not silently mark complete.
- Do not bundle "while I'm here" fixes into the current task. Finish the current task; address other findings separately.

## What success looks like

When `task.done` has been invoked for every checkbox, the engine sets `all_tasks_done=1` on the change. Under walking-skeleton mode (`require_code_review=false`, current default), `state` stays `in_progress` and the next step is an explicit `archive.run` call to transition `in_progress → archived`. When `require_code_review=true` is configured (Phase 2), the engine auto-transitions `in_progress → code_reviewing` and waits for reviewer approval before archive becomes eligible. Either way, the next phase is archive or review — not more apply.
