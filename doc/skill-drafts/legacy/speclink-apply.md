---
name: speclink-apply
description: "Implement or resume tasks from a SpecLink change"
effort: xhigh
license: MIT
compatibility: Requires speclink CLI.
speclink_version: 0.1.0
template_hash: sha256:<generated-at-build>
metadata:
  author: speclink
  version: "1.0"
  generatedBy: SpecLink
---

Implement tasks from a SpecLink change.

**Input**: Optionally specify a change name (e.g., `/speclink-apply add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Task tracking is file-based only.** The tasks file's markdown checkboxes (`- [ ]` / `- [x]`) are the single source of truth for progress. Do NOT use any external task management system, built-in task tracker, or todo tool. When a task is done, run `speclink task done` — the CLI updates the checkbox AND records lifecycle / touched-file state in `state.db`. That is the only way to record progress.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP. Also verify the project is initialized — if `speclink status` reports `project.not_initialized`, ask the user to run `speclink init <project-name>` first.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:

   - Infer from conversation context if the user mentioned a change.
   - Auto-select if exactly one active change exists (state `ready` or `in_progress`, not parked).
   - If ambiguous, run both:
     ```bash
     speclink list --changes --json
     speclink list --changes --parked --json
     ```
     Parked changes should be annotated with "(parked)" in the selection list. Ask the user to choose.

   Always announce: "Using change: `<name>`" and how to override (e.g., `/speclink-apply <other>`).

2. **Check state and parked status**

   ```bash
   speclink status --change "<name>" --json
   ```

   **If the command fails** (`change.not_found` etc.): show the error and STOP.

   **If the command succeeds**, inspect the returned JSON for:

   - `state`: must be one of `ready` or `in_progress` to proceed.
   - `parked`: boolean flag (parked is a flag, not a state — see SpecLink design §6.4).
   - `schemaName`: workflow schema in use (e.g., `spec-driven`).

   **State handling**:

   - `state: "ready"` → continue (will explicitly transition to `in_progress` in step 2b below).
   - `state: "in_progress"` → continue (already started; resume work).
   - `state: "proposing"` → STOP. Tell the user to finish proposing first (`/speclink-propose <name>` or check missing artifacts).
   - `state: "reviewing"` → STOP. Tell the user the change is awaiting artifact review approval. The reviewer should run:
     ```bash
     speclink review approve --change <name> --reviewer <id>
     ```
   - `state: "code_reviewing"` → tasks are all done; tell the user to approve code review then archive, OR re-open tasks via `/speclink-ingest <name>` if more work is needed.
   - `state: "archived"` → STOP. Cannot apply to an archived change.

   **Parked handling**:

   - If `parked: true`, inform the user the change is parked (暫存). Ask the user to choose: continue (unpark and proceed) or cancel.
   - If the user chooses to continue:
     ```bash
     speclink unpark "<name>"
     ```
     This is a silent state-flag flip in `state.db`; no file move (unlike spectra). Then re-run `speclink status --change "<name>" --json` and continue.

2b. **Explicit `apply start` (if state is `ready`)**

   If state is currently `ready`, transition to `in_progress` explicitly:

   ```bash
   speclink apply start "<name>" --json
   ```

   This:

   - Flips state `ready → in_progress`.
   - Records the actor (engine derives from environment, or you may pass `--actor <id>`).
   - Is idempotent — if state is already `in_progress`, returns success no-op.

   If `state.transition_invalid` is returned (e.g., the change was archived between step 2 and 2b by another agent), STOP and report.

   **Why explicit**: in-flight work should be visible to other agents and to the user even before the first task completes. The state `in_progress` is the public signal "someone is working on this".

   **Note**: `task done` no longer triggers `ready → in_progress` (that responsibility moved here). `task done` still triggers `in_progress → code_reviewing` (or final state) when the last task completes — that auto-transition is engine-derived from "all tasks done".

3. **Get apply instructions**

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   This returns:

   - `contextFiles`: paths to read for context (varies by schema)
   - `progress`: `{ total, complete, remaining }`
   - `tasks`: list with status
   - `instruction`: dynamic guidance based on current state and config
   - `preflight`: preflight result if applicable
   - `tddDiscipline`: TDD guide if `.speclink/config.yaml#tdd: true` (embedded; no separate CLI call needed)
   - `auditDiscipline`: audit guide if `.speclink/config.yaml#audit: true` (embedded)
   - `parallelTasks`: whether `[P]` markers are enabled

   **Handle states inside the instructions**:

   - `state: "blocked"` (missing artifacts) → show message, suggest `/speclink-propose` to create the change artifacts first.
   - `state: "all_done"` → congratulate, suggest archive (the change may already be in `code_reviewing` state).
   - Otherwise → proceed to preflight check.

3b. **Preflight check**

If the apply instructions JSON includes a `preflight` field, act on its `status`:

- **`"clean"`**: silently continue — no output needed.
- **`"warnings"`**: display a brief summary, then continue automatically:
  ```
  ⚠ Preflight warnings:
  - Drifted files (modified after change was created): <list paths>
  - Change is <N> days old
  Continuing...
  ```
  Only show the lines that are relevant (skip drifted if none, skip staleness if not stale).
- **`"critical"`**: display missing files with their source artifact, then ask the user:

  ```
  ⚠ Preflight: missing files detected
  - <path> (referenced in <source artifact>)
  - ...
  These files are referenced in the change artifacts but no longer exist on disk.
  ```

  Options: "Continue anyway" / "Stop". If the user chooses "Stop", end the workflow.

If the `preflight` field is absent (blocked or all_done states), skip this step.

3c. **Artifact quality check**

Run `speclink analyze "<name>" --json` to check cross-artifact consistency (Coverage / Consistency / Ambiguity / Gaps).

- **Zero findings**: silently continue.
- **Warning / Suggestion only**: display a one-line summary (e.g., "⚠ Artifact analysis: 2 warnings found") and continue automatically.
- **Critical findings**: display each Critical finding (summary + location + recommendation), then ask the user:
  - **Fix and continue** — fix the artifact issues inline (use `speclink new artifact <kind> --change <name> --stdin --overwrite` to rewrite), then proceed.
  - **Continue anyway** — skip fixes and start implementation.
  - **Stop** — end the workflow.

3d. **Drift dormancy check** (passive trigger for stale changes)

When the change has been dormant for more than 5 days AND the change directory has had zero commits in the past 3 days, surface a drift report before tasks begin — the change is likely out-of-sync with the current codebase.

Detect dormancy from `.speclink/changes/<name>/metadata.json` `created` and `git log -1 --format=%at -- .speclink/changes/<name>/`:

- **Both conditions met**: run `speclink drift <name> --json`, display the report, then ask the user:
  - **Continue with apply** — proceed to tasks (recommended for Light drift).
  - **Refresh first** — pause apply, run `/speclink-ingest <name>` to update artifacts, then resume.
  - **Stop** — end the workflow.
- **Either condition not met**: silently continue, no output.

The trigger is guidance only — it MUST NOT block apply from proceeding when the user chooses to continue. Hard-blocking on dormancy would punish legitimate "I came back after a long weekend" cases.

(Threshold reasoning: AI-assisted commits are daily-cadence. ≥5 days dormant + ≥3 days no commit ≈ genuine stagnation, not normal pacing.)

4. **Read context files**

   Read the files listed in `contextFiles` from the apply instructions output.

   The files depend on the schema being used:

   - **spec-driven**: proposal, specs/\*\*/\*.md, design (if exists), tasks.
   - Other schemas: follow the `contextFiles` from CLI output — never assume a hardcoded set.

   If a linked discussion exists (apply instructions may include `linkedDiscussions[]` for changes that originated from a discussion), optionally skim the discussion's Conclusion + Decisions Made for the original "why" — but do NOT modify the discussion.

5. **Check project preferences**

   The apply instructions JSON already embeds the relevant config (TDD, audit, parallel_tasks). Use those values directly:

   - **`tdd: true`** → apply TDD discipline throughout:
     - For each task, write a failing test FIRST, then implement to make it pass.
     - Follow the Red-Green-Refactor cycle.
     - For bug fixes, reproduce the bug with a failing test before fixing.
     - The discipline detail is in `tddDiscipline` from the instructions JSON.

   - **`audit: true`** → apply sharp-edges discipline throughout:
     - When designing APIs or interfaces, evaluate through adversary lenses (Scoundrel, Lazy Developer, Confused Developer).
     - When adding configuration options, verify defaults are secure and zero/empty values are safe.
     - When accepting parameters, check for type confusion and silent failures.
     - The discipline detail is in `auditDiscipline` from the instructions JSON.

   - **`parallel_tasks: true`** → check whether consecutive pending tasks have `[P]` markers (format: `- [ ] [P] Task description`). You SHALL dispatch consecutive `[P]` tasks as parallel agents. Only fall back to sequential when tasks have a data dependency (one task's output is another's input) or when tasks modify overlapping regions of the same file. Targeting the same file alone is NOT a reason to skip parallel dispatch — if the modified regions are disjoint, dispatch in parallel. If the environment does not support parallel execution, ignore `[P]` markers and execute tasks sequentially.

6. **Show current progress**

   Display:

   - State (`ready` or `in_progress`) and schema being used
   - Progress: "N/M tasks complete"
   - Remaining tasks overview
   - Dynamic instruction from CLI

7. **Implement tasks (loop until done or blocked)**

   **Reminder: Track progress by running `speclink task done` only. Do not use any built-in task tracker, todo tool, or external task management.**

   For each pending task:

   - Show which task is being worked on.
   - Re-read the sections of design and spec files that are relevant to this task's scope — do not rely on memory from earlier in the conversation, as context may have been compressed.
   - **Read the Implementation Contract for this task before editing any source file.** If `design.md` exists and contains an `## Implementation Contract` section (or contract content under another heading the design uses), read the part of it that covers this task's scope. The contract names the observable behavior, interface or data shape, failure modes, acceptance criteria, and scope boundaries you must satisfy. Treat the contract as the durable handoff — it is what the task will be measured against, regardless of who started the change.
   - **Detect unclear or path-only tasks before writing code.** A task is unclear if it:
     - only names files to edit ("edit `foo.rs`", "update `bar.svelte`") with no behavior, contract, or verification target;
     - is vague ("handle edge cases", "wire it up", "make it work");
     - conflicts with the implementation contract (asks for behavior the contract excludes, or omits behavior the contract requires).

     When this happens, pause. Either update the artifact (design or tasks via `/speclink-ingest <name>`) so the task names a concrete behavior and verification target, or report the blocker and wait for guidance. Do NOT silently guess against unclear requirements.
   - Before writing code, check:
     1. **Reuse** — search adjacent modules and shared utilities for existing implementations before writing new code.
     2. **Quality** — derive values from existing state instead of duplicating; use existing types and constants over new literals.
     3. **Efficiency** — parallelize independent async operations; avoid unnecessary awaits; match operation scope to actual need.
     4. **No Placeholders in artifacts** — if the design or spec for this task contains placeholder language (TBD, TODO, "add appropriate handling"), pause and fix the artifact first (via `/speclink-ingest`) or flag to the user. Do not implement against vague requirements.
     5. **Examples as verification** — if the spec for this task's scope includes `##### Example:` blocks, use them as concrete test cases:
        - When TDD is enabled: derive the first failing test directly from the example's GIVEN/WHEN/THEN values.
        - When TDD is not enabled: after implementing, verify the code handles the example's input→output correctly.
        - Example tables map to parameterized tests — one test per row.

        Do NOT invent additional test values beyond what the spec examples provide without reason. The examples ARE the agreed specification.
   - Make the code changes required.
   - Keep changes minimal and focused.
   - **Verify before marking done** — re-read the task description from the tasks file AND the relevant Implementation Contract content from design.md. For each requirement stated in the task description and each contract item that covers this task's scope, confirm it is addressed by your changes. Confirm the verification target named by the task (test name, CLI invocation, analyzer check, or manual assertion) actually passes. If any contract item, task requirement, or verification target is missing or failing, implement/fix it now. Do not mark the task complete until every part of the description is covered and the contract for this task is satisfied.
   - Mark task complete by running:
     ```bash
     speclink task done <task-id> --change "<name>" --json
     ```
     This command:
     - Marks the checkbox in `tasks.md` (`- [ ]` → `- [x]`)
     - Records touched files in `.speclink/touched/<name>.json` (if the Git worktree has modified or staged files)
     - On last task done (all tasks `[x]`): engine auto-transitions:
       - to `code_reviewing` if `.speclink/config.yaml#require_code_review: true` — the change is now awaiting code review approval; tell the user to ask their reviewer
       - to `archive-ready` (still `in_progress`, with `all_tasks_done: true` flag) if `false` — tell the user `/speclink-archive <name>` is now valid
     - May return drift warnings in the response — review them; if severity is critical, pause and consider `/speclink-ingest`.

   **Handling review feedback tasks**: If you see tasks prefixed with `[Review feedback by <reviewer>, <date>]` in `tasks.md`, these are auto-appended by engine when a previous code review was rejected. Treat them as normal tasks — address the feedback, then `speclink task done <id>`. Engine will auto re-enter `code_reviewing` when all tasks (including these synthetic ones) are `[x]` again.
   - Continue to next task.

   **Parallel task dispatch**: When consecutive `[P]`-marked tasks are found and `parallel_tasks: true` is configured (see Step 5), dispatch them as parallel agents in a single message. If any `[P]` task fails, pause and report.

   **Pause if**:

   - Task is unclear → ask for clarification or update artifacts via `/speclink-ingest`.
   - Implementation reveals a design issue → suggest `/speclink-discuss --about <name>` to capture structured discussion, then `/speclink-ingest <name>` to update artifacts.
   - Error or blocker encountered → report and wait for guidance.
   - User interrupts.
   - `change.locked` received (another agent is writing) → retry 1-2 sec × max 3 attempts before pausing.

---

## Rationalization Table

| What You're Thinking                                               | What You Should Do                                                                                                                            |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| "This task looks done, I'll mark it complete"                      | Re-read the task description first. Check whether your diff covers every part of it. Incomplete tasks marked done are the #1 source of rework |
| "This task is trivial, I don't need to re-read the design"         | Re-read. Context compression loses details. 30s of reading saves 30min of rework                                                              |
| "I already know how this works, skip the code search"              | Search anyway. Someone may have added a utility since you last looked                                                                         |
| "The test is obvious, I'll add it after implementation"            | If TDD is enabled, test first. If not, still write it before marking done                                                                     |
| "This is just a small refactor, no test needed"                    | Small refactors are how regressions sneak in. Write the test                                                                                  |
| "The artifact says X but Y makes more sense"                       | Pause and run `/speclink-ingest <name>` to update the artifact. Don't silently deviate                                                        |
| "I'll fix this other thing I noticed while I'm here"               | Finish current task first. Address the other thing separately                                                                                 |
| "The example values are just illustrations, I'll pick better ones" | Use the spec example values exactly. They were chosen deliberately                                                                            |
| "I'll edit `tasks.md` directly to mark this done"                  | Use `speclink task done` — direct edits skip state.db updates and touched-file tracking                                                       |

---

8. **Final check**

   After completing all tasks, re-run:

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   Confirm `state: "all_done"`. Also check the change state via `speclink status --change "<name>" --json`:

   - If state has auto-transitioned to `code_reviewing` → tell the user code review is required:
     ```bash
     speclink review approve --change <name> --reviewer <id> --phase code
     ```
     Then `/speclink-archive <name>` to finalize.
   - If state is still `in_progress` but all tasks are done (i.e., `require_code_review: false`) → tell the user to run `/speclink-archive <name>` directly.

   If not `all_done`, review remaining tasks and complete them.

9. **On completion or pause, show status**

   Display:

   - Tasks completed this session
   - Overall progress: "N/M tasks complete"
   - Current state after engine auto-transition
   - If all done + `code_reviewing` → mention reviewer step
   - If all done + still `in_progress` → suggest archive
   - If paused → explain why and wait for guidance

---

**Output During Implementation**

```
## Implementing: <change-name> (state: in_progress, schema: <schema-name>)

Working on task 3/7: <task description>
[...implementation happening...]
✓ Task complete

Working on task 4/7: <task description>
[...implementation happening...]
✓ Task complete
```

**Output On Completion (require_code_review: true)**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** code_reviewing (auto-transitioned)
**Progress:** 7/7 tasks complete ✓

### Completed This Session
- [x] Task 1
- [x] Task 2
...

All tasks complete! Code review is required before archive.
1. Reviewer runs: `speclink review approve --change <change-name> --reviewer <id> --phase code`
2. Then run `/speclink-archive <change-name>` to finalize.
```

**Output On Completion (require_code_review: false)**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** in_progress (code review disabled in config)
**Progress:** 7/7 tasks complete ✓

### Completed This Session
- [x] Task 1
- [x] Task 2
...

All tasks complete! Run `/speclink-archive <change-name>` to finalize.
```

**Output On Pause (Issue Encountered)**

```
## Implementation Paused

**Change:** <change-name>
**Schema:** <schema-name>
**State:** <state>
**Progress:** 4/7 tasks complete

### Issue Encountered
<description of the issue>

**Options:**
1. <option 1>
2. <option 2>
3. Other approach (e.g., `/speclink-discuss --about <change-name>` to think it through, then `/speclink-ingest <change-name>` to update artifacts)

What would you like to do?
```

---

## Concurrency & Errors

- `change.locked` (another agent is writing to the same change) → retry 1-2 sec × max 3 attempts. If still failing, pause and surface to user.
- `change.not_found` → suggest `speclink list --changes --json` to find the correct name.
- `state.transition_invalid` (e.g., trying to apply when state is `proposing`) → stop; tell the user what state the change is actually in and what skill to run instead.
- `artifact.validation_failed` (when fixing artifacts via `new artifact --overwrite`) → engine returns specific errors; fix content and retry.
- `config.malformed` → surface engine warnings (warnings[]) to the user; never auto-modify config.
- `task.not_found` → check task id; the tasks file may have been edited externally.
- `project.not_initialized` → stop; ask user to run `speclink init <project-name>`.

## Guardrails

- Keep going through tasks until done or blocked.
- Always read context files before starting (from the apply instructions output).
- If task is ambiguous, pause and ask before implementing — or suggest `/speclink-ingest` to update the artifact.
- If implementation reveals issues, pause and suggest `/speclink-discuss` (to think) or `/speclink-ingest` (to update artifacts). Do not silently deviate.
- Keep code changes minimal and scoped to each task.
- Update task checkbox via `speclink task done` only — never manually edit `tasks.md` to flip checkboxes.
- Pause on errors, blockers, or unclear requirements — don't guess.
- Use `contextFiles` from CLI output, don't assume specific file names.
- **No external task tracking** — do not use any built-in task management, todo list, or progress tracking tool; the tasks file (via `speclink task done`) is the only system.
- **Never auto-approve review** — `speclink review approve` requires a human reviewer id and explicit decision; the apply skill does not invoke it.
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.

## Pausing Apply

The user may explicitly pause an in-flight apply to free the change for others or to step away cleanly:

```bash
speclink apply pause "<change-name>" --json
```

This:

- Flips state `in_progress → ready` and clears the actor marker.
- **Idempotent in both directions** (see design doc §6.2): calling `apply pause` on a change already in `ready` returns `{state: "ready"}` with a no-op message — not an error. Calling on `code_reviewing` / `archived` returns the current state with a friendly message rather than `state.transition_invalid`.
- Symmetric to `apply start`: both commands ensure-actor and return current state JSON.

After `apply pause`, the change is available for any agent to `apply start` again later. The completed tasks remain marked; pause does not reset task state.

**When to use pause vs park**:

- `apply pause` — short-term "stepping away, will resume soon"; change still appears in default `speclink list`.
- `speclink park <change>` — long-term "set this aside, may come back later"; hidden from default `list` (requires `--parked`). Park can happen from any state.

## Fluid Workflow Integration

This skill supports the "actions on a change" model:

- **Can be invoked anytime** within state `ready` or `in_progress`. Before all artifacts are done (state `proposing` / `reviewing`), apply is not yet valid — finish proposing or wait for review approval first.
- **Allows artifact updates** mid-implementation: if implementation reveals design issues, pause and run `/speclink-discuss --about <change>` (to capture reasoning) and `/speclink-ingest <change>` (to update artifacts). Apply can resume afterward.
- **No phase-lock** — even after some tasks are done, you can `apply pause` then refine artifacts (via ingest), then `apply start` and continue.
- **State transitions**:
  - `ready → in_progress` is explicit via `speclink apply start` (step 2b).
  - `in_progress → ready` is explicit via `speclink apply pause` (this section).
  - `in_progress → code_reviewing` (or end-of-apply state) is engine-derived from "all tasks done".
