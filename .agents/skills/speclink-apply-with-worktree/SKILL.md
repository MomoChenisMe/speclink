---
name: speclink-apply-with-worktree
description: "Implement tasks from a Speclink change inside an isolated git worktree, for parallel work"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.18.1"
  generatedBy: "Speclink"
---

Implement tasks from a Speclink change in an isolated git worktree, so several changes can be applied in parallel without stepping on each other.

**Input**: Optionally specify a change name (e.g., `$speclink-apply-with-worktree add-auth`). Everything the plain apply skill accepts applies here too.

**Prerequisites**: This skill requires the `speclink` CLI and `git`. If any command fails with "command not found" or similar, report the error and STOP.

---

## Worktree preflight

Complete these steps **before** any of the apply flow below. Each one can stop the run.

### P0. One change per run

This skill takes **exactly one** change. Parallel work means one session per change, each in its own worktree — not one session cycling through several.

If the input names more than one change (e.g. `$speclink-apply-with-worktree add-auth add-billing add-search`), STOP and use the **AskUserQuestion tool** to have the user pick the one to run here. Then print the recipe for the rest, naming them:

> 平行做法是一個 change 一個 session：另外開視窗，各自執行 `$speclink-apply-with-worktree <change-name>`。主資料夾的看板會同時顯示每個 worktree 的進度。

Do **NOT** run them one after another in this session. A single session working through several changes serializes what the user asked to parallelize, and its context is spent on the wrong change by the time the second one starts.

If there is no AskUserQuestion tool available, list the names as plain text, ask which one to run, and wait for the answer.

### P1. Check the worktree policy

Read the EFFECTIVE value, the same way the CLI resolves it — the env layer wins over the file:

1. If the environment variable `SPECLINK_WORKTREE` is set to `true` or `false` (case-insensitive), that IS the effective value — do not consult the file.
2. Otherwise read the canonical value:

   ```bash
   speclink workflow-config show --json
   ```

   and use its `worktree` field.

- **effective value `true`** — continue to P2.
- **anything else (`false` or absent)** — STOP. Tell the user, in these terms:

  > 本專案未啟用 worktree 流程。要啟用請執行：`speclink workflow-config set worktree true`

  Do **NOT** fall back to running the apply flow in the main folder. Enabling the policy is the user's decision, not yours — offer to run `speclink workflow-config set worktree true` and wait for their answer.

### P2. Confirm the change exists and is not archived

```bash
speclink list --json
```

The change must appear among the active changes. If it does not (unknown name, or already archived), STOP and report which change names are available.

### P3. Get the change's artifacts into HEAD

A worktree is materialized from HEAD. If the change's artifacts (`openspec/changes/<change-name>/`) are not committed yet — which is the normal state right after `$speclink-propose` — the new worktree simply will not contain the change, and every later step dead-ends.

Check:

```bash
git status --porcelain -- "openspec/changes/<change-name>/"
```

- **Output empty** (artifacts already committed, unchanged) — continue to P4.
- **Output non-empty** — commit exactly that directory, nothing else:

  ```bash
  git add "openspec/changes/<change-name>"
  git commit -m "<a conventional-commit message for the change's spec artifacts, in the project's language>"
  ```

  Never sweep other dirty files into this commit. If the directory cannot be committed cleanly (e.g. merge conflict markers), STOP and report.

### P3.5. Check whether progress and code have come apart

P3 commits the change's **artifacts**. Its **source changes** are a separate matter: when this change was already applied in the main folder, its task checkboxes went into HEAD just now while the code that satisfies them may still be sitting dirty in the main tree. The worktree would then carry tasks marked done with no implementation behind them.

Read the change's evidence record and check those files against the main tree:

```bash
cat "openspec/changes/<change-name>/.evidence.json"
git status --porcelain -- <each touched path from that record>
```

- **No record, or it lists no touched files** — nothing was implemented here yet. Continue to P4 silently.
- **Every listed path is clean** — the code is already in HEAD and travels with the worktree. Continue to P4 silently.
- **Any listed path is dirty** — STOP and show the user the dirty paths, then use the **AskUserQuestion tool**:
  - **先收程式碼再開 worktree**（recommended） — run `$speclink-commit <change-name>` to get this change's source into HEAD, then come back and re-run this skill.
  - **照樣繼續** — create the worktree knowing it will not contain those edits; the tasks they belong to will read as done with nothing behind them.
  - **停止** — end the run and leave everything as it is.

Do NOT create the worktree before the user has chosen. If there is no AskUserQuestion tool available, present the same three options as plain text and wait for the answer.

### P4. Create or reuse the worktree

The convention is fixed — do not invent paths or branch names:

- **Branch**: `speclink/<change-name>`
- **Location**: a sibling nest beside the repo, `<repo-folder-name>.worktrees/<change-name>/`

  For a repo at `/work/speclink` and change `add-auth`, that is `/work/speclink.worktrees/add-auth/`. The nest sits *outside* the repo so it is never picked up by the repo's own tooling.

Check what already exists:

```bash
git worktree list --porcelain
git branch --list "speclink/<change-name>"
```

- **Worktree already present at that path** — reuse it and continue there. Do NOT create a second one, and do NOT remove and recreate it: it may hold work in progress.
- **Branch exists but no worktree** — attach a worktree to the existing branch:

  ```bash
  git worktree add "<repo-parent>/<repo-folder-name>.worktrees/<change-name>" "speclink/<change-name>"
  ```

- **Neither exists** — create both:

  ```bash
  git worktree add -b "speclink/<change-name>" "<repo-parent>/<repo-folder-name>.worktrees/<change-name>"
  ```

If `git worktree add` fails, report the error verbatim and STOP.

### P5. Tell the user what this costs

Print a short note before starting work:

> worktree 是一份完整的原始碼副本。相依套件與建置產物不會跟著複製過去——第一次在裡面跑測試或建置，要自己重新安裝相依並重新建置，會花上一段時間。

### P6. Work inside the worktree from here on

Every step of the apply flow below runs **inside the worktree folder**, not the main checkout:

- `cd` into the worktree, or pass it explicitly to every command.
- File reads and edits target the worktree copy.
- `speclink` verbs run with the worktree as the working directory, so task checkboxes and stamps land in that copy.

The main checkout stays untouched. Its `speclink list` will show this change with a `[worktree]` marker and reflect the worktree's task progress live — that is how the user watches parallel work from one place.

---

以下為 apply 本體流程，於上述 worktree 資料夾內執行。

---

Implement tasks from a Speclink change.

**Input**: Optionally specify a change name (e.g., `$speclink-apply add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Task tracking is file-based only.** The tasks file's markdown checkboxes (`- [ ]` / `- [x]`) are the single source of truth for progress. Do NOT use any external task management system, built-in task tracker, or todo tool. When a task is done, edit the checkbox in the tasks file — that is the only way to record progress.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `speclink list --json` to get all available changes. Use the **AskUserQuestion tool** to let the user select

   Always announce: "Using change: <name>" and how to override (e.g., `$speclink-apply <other>`).

2. **Check status to understand the schema**

   ```bash
   speclink status --change "<name>" --json
   ```

   **If the command fails**: show the error and STOP.

   **If the command succeeds**, capture the review baseline, then mark the change as in-progress:

   ```bash
   speclink review prepare "<name>"
   speclink in-progress add "<name>"
   ```

   `review prepare` records the host-local Apply baseline (HEAD, dirty files at start) that the review station later resolves its frozen change scope against. Both are silent operations — do not show their output to the user. A stderr warning from `review prepare` (late or unavailable baseline) is fine — continue. If `speclink review prepare` fails, report the error and STOP — do NOT run `speclink in-progress add`.

   Parse the JSON to understand:
   - `schemaName`: The workflow being used (e.g., "spec-driven")
   - Which artifact contains the tasks (typically "tasks" for spec-driven, check status for others)

3. **Get apply instructions**

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   This returns:
   - Context file paths (varies by schema)
   - Progress (total, complete, remaining)
   - Task list with status
   - Dynamic instruction based on current state

   **Handle states:**
   - If `state: "blocked"` (missing artifacts): show message, suggest using `$speclink-propose` to create the change artifacts first
   - If `state: "all_done"`: congratulate, suggest archive
   - Otherwise: proceed to implementation

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
- **`"critical"`**: display missing files with their source artifact, then use the **AskUserQuestion tool** to ask the user:

  ```
  ⚠ Preflight: missing files detected
  - <path> (referenced in <source artifact>)
  - ...
  These files are referenced in the change artifacts but no longer exist on disk.
  ```

  Options: "Continue anyway" / "Stop"
  If the user chooses "Stop", end the workflow.

  If there is no AskUserQuestion tool available:
  Display the same information as plain text and ask whether to continue or stop.
  Wait for the user's response.

If the `preflight` field is absent (blocked or all_done states), skip this step.

3c. **Artifact quality check**

Run `speclink analyze <change-name> --json` to check cross-artifact consistency (Coverage, Consistency, Ambiguity, Gaps).

- **Zero findings**: silently continue.
- **Warning/Suggestion only**: display a one-line summary (e.g., "⚠ Artifact analysis: 2 warnings found") and continue automatically.
- **Critical findings**: display each Critical finding (summary + location + recommendation), then use the **AskUserQuestion tool**:
  - **Fix and continue** — fix the artifact issues inline, then proceed
  - **Continue anyway** — skip fixes and start implementation
  - **Stop** — end the workflow

  If there is no AskUserQuestion tool available, present options as plain text and wait for the user's response.

3d. **Drift dormancy check** (passive trigger for stale changes)

When the change has been dormant for more than 5 days AND the change directory has had zero commits in the past 3 days, surface a drift report before tasks begin — the change is likely out-of-sync with the current codebase.

Detect dormancy from `.openspec.yaml` `created` and `git log -1 --format=%at -- openspec/changes/<name>/`:

- **Both conditions met**: run `speclink drift <change-name>`, display the report, then use the **AskUserQuestion tool**:
  - **Continue with apply** — proceed to tasks (recommended for Light drift)
  - **Refresh first** — pause apply, run `/speclink-ingest <change-name>` to update artifacts, then resume
  - **Stop** — end the workflow
- **Either condition not met**: silently continue, no output.

The trigger is guidance only — it MUST NOT block apply from proceeding when the user chooses to continue. Hard-blocking on dormancy would punish legitimate "I came back after a long weekend" cases.

(Threshold reasoning: AI-assisted commits are daily-cadence. ≥5 days dormant + ≥3 days no commit ≈ genuine stagnation, not normal pacing.)

If there is no AskUserQuestion tool available, present options as plain text and wait for the user's response.

4. **Read context files**

   Read the files listed in `contextFiles` from the apply instructions output.
   The files depend on the schema being used:
   - **spec-driven**: proposal, specs, design, tasks
   - Other schemas: follow the contextFiles from CLI output

   **Remote mode**: when the workspace is connected to a remote store, `contextFiles` points into the read-only Context Projection (`.speclink/context/`) — a local snapshot of the remote canon. Read, search, and grep it freely, but NEVER edit projection files: a direct edit is not a remote write and the next command will reject the projection as modified. Any spec or artifact change goes through speclink verbs. If a `STALE` marker file exists at the projection root or a command reports the projection as modified, re-run `speclink instructions apply` to refresh it.

5. **Check project preferences**

   Read `.speclink.yaml` in the project root.
   If `tdd: true` is set, apply TDD discipline throughout implementation:
   - For each task, write a failing test FIRST, then implement to make it pass
   - Fetch TDD instructions by running `speclink instructions --skill tdd`, then follow the Red-Green-Refactor cycle
   - For bug fixes, reproduce the bug with a failing test before fixing

   If `audit: true` is set, apply sharp-edges discipline throughout implementation:
   - When designing APIs or interfaces, evaluate through 3 adversary lenses (Scoundrel, Lazy Developer, Confused Developer)
   - When adding configuration options, verify defaults are secure and zero/empty values are safe
   - When accepting parameters, check for type confusion and silent failures
   - Fetch audit instructions by running `speclink instructions --skill audit`, follow the discipline checklist (not the standalone 3-agent workflow)

6. **Show current progress**

   Display:
   - Schema being used
   - Progress: "N/M tasks complete"
   - Remaining tasks overview
   - Dynamic instruction from CLI

7. **Implement tasks (loop until done or blocked)**

   **Reminder: Track progress by editing checkboxes in the tasks file only. Do not use any built-in task tracker.**

   For each pending task:
   - Show which task is being worked on
   - Re-read the sections of design and spec files that are relevant to this task's scope — do not rely on memory from earlier in the conversation, as context may have been compressed
   - **Read the Implementation Contract for this task before editing any source file.** If `design.md` exists and contains an `## Implementation Contract` section (or contract content under another heading the design uses), read the part of it that covers this task's scope. The contract names the observable behavior, interface or data shape, failure modes, acceptance criteria, and scope boundaries you must satisfy. Treat the contract as the durable handoff — it is what the task will be measured against, regardless of who started the change.
   - **Detect unclear or path-only tasks before writing code.** A task is unclear if it:
     - only names files to edit ("edit `foo.rs`", "update `bar.svelte`") with no behavior, contract, or verification target;
     - is vague ("handle edge cases", "wire it up", "make it work");
     - conflicts with the implementation contract (asks for behavior the contract excludes, or omits behavior the contract requires).
       When this happens, pause. Either update the artifact (design or tasks) so the task names a concrete behavior and verification target, or report the blocker and wait for guidance. Do NOT silently guess against unclear requirements.
   - Before writing code, check:
     1. **Reuse** — search adjacent modules and shared utilities for existing implementations before writing new code
     2. **Quality** — derive values from existing state instead of duplicating; use existing types and constants over new literals
     3. **Efficiency** — parallelize independent async operations; avoid unnecessary awaits; match operation scope to actual need
     4. **No Placeholders in artifacts** — if the design or spec for this task contains placeholder language (TBD, TODO, "add appropriate handling"), pause and fix the artifact first or flag to the user. Do not implement against vague requirements.
     5. **Examples as verification** — if the spec for this task's scope includes `##### Example:` blocks, use them as concrete test cases:
        - When TDD is enabled: derive the first failing test directly from the example's GIVEN/WHEN/THEN values
        - When TDD is not enabled: after implementing, verify the code handles the example's input→output correctly
        - Example tables map to parameterized tests — one test per row
          Do NOT invent additional test values beyond what the spec examples provide without reason. The examples ARE the agreed specification.
   - Make the code changes required
   - Keep changes minimal and focused
   - **Verify before marking done** — re-read the task description from the tasks file AND the relevant Implementation Contract content from design.md. For each requirement stated in the task description and each contract item that covers this task's scope, confirm it is addressed by your changes. Confirm the verification target named by the task (test name, CLI invocation, analyzer check, or manual assertion) actually passes. If any contract item, task requirement, or verification target is missing or failing, implement/fix it now. Do not mark the task complete until every part of the description is covered and the contract for this task is satisfied.
   - Mark task complete by running: `speclink task done --change "<name>" <task-id>`
     This command marks the checkbox in tasks.md AND records which files were modified for this task.
   - If a task was checked by mistake or its implementation is rolled back, run: `speclink task undone --change "<name>" <task-id>`
     Do NOT edit tasks.md directly to uncheck a task.
   - Continue to next task

   **Pause if:**
   - Task is unclear → ask for clarification
   - Implementation reveals a design issue → suggest updating artifacts
   - Error or blocker encountered → report and wait for guidance
   - User interrupts

   **Started the wrong change?**

   If apply was run against the wrong change (or a change was marked
   in-progress by mistake), revert it to proposed:

   ```bash
   speclink in-progress remove "<name>"
   ```

   The verb succeeds only when the change carries zero work traces; with
   traces present it refuses and lists the evidence. Two ways out:

   - Checked tasks: uncheck them with `speclink task undone`, then retry
   - Touched records: the listed files may mix content from other changes —
     judge and clean them up case by case (there is no force flag and no
     mechanical cleanup), then ask the user how to proceed

   Unlike `in-progress add`, an unknown change name errors loudly — check the
   name with `speclink list` if it reports not found.

---

## Rationalization Table

| What You're Thinking                                               | What You Should Do                                                                                                                            |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| "This task looks done, I'll mark it complete"                      | Re-read the task description first. Check whether your diff covers every part of it. Incomplete tasks marked done are the #1 source of rework |
| "This task is trivial, I don't need to re-read the design"         | Re-read. Context compression loses details. 30s of reading saves 30min of rework                                                              |
| "I already know how this works, skip the code search"              | Search anyway. Someone may have added a utility since you last looked                                                                         |
| "The test is obvious, I'll add it after implementation"            | If TDD is enabled, test first. If not, still write it before marking done                                                                     |
| "This is just a small refactor, no test needed"                    | Small refactors are how regressions sneak in. Write the test                                                                                  |
| "The artifact says X but Y makes more sense"                       | Pause and suggest updating the artifact. Don't silently deviate                                                                               |
| "I'll fix this other thing I noticed while I'm here"               | Finish current task first. Address the other thing separately                                                                                 |
| "The example values are just illustrations, I'll pick better ones" | Use the spec example values exactly. They were chosen deliberately                                                                            |

---

8. **Final check**

   After completing all tasks, re-run:

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   Confirm `state: "all_done"`. If not, review remaining tasks and complete them.

9. **On completion or pause, show status**

   Display:
   - Tasks completed this session
   - Overall progress: "N/M tasks complete"
   - If all done: suggest archive
   - If paused: explain why and wait for guidance

**Output During Implementation**

```
## Implementing: <change-name> (schema: <schema-name>)

Working on task 3/7: <task description>
[...implementation happening...]
✓ Task complete

Working on task 4/7: <task description>
[...implementation happening...]
✓ Task complete
```

**Output On Completion**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Progress:** 7/7 tasks complete ✓

### Completed This Session
- [x] Task 1
- [x] Task 2
...

All tasks complete! You can archive this change with `$speclink-archive`.
```

**Output On Pause (Issue Encountered)**

```
## Implementation Paused

**Change:** <change-name>
**Schema:** <schema-name>
**Progress:** 4/7 tasks complete

### Issue Encountered
<description of the issue>

**Options:**
1. <option 1>
2. <option 2>
3. Other approach

What would you like to do?
```

**Guardrails**

- Keep going through tasks until done or blocked
- Always read context files before starting (from the apply instructions output)
- If task is ambiguous, pause and ask before implementing
- If implementation reveals issues, pause and suggest artifact updates
- Keep code changes minimal and scoped to each task
- Update task checkbox immediately after completing each task
- Pause on errors, blockers, or unclear requirements - don't guess
- Use contextFiles from CLI output, don't assume specific file names
- **No external task tracking** — do not use any built-in task management, todo list, or progress tracking tool; the tasks file is the only system
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

**Fluid Workflow Integration**

This skill supports the "actions on a change" model:

- **Can be invoked anytime**: Before all artifacts are done (if tasks exist), after partial implementation, interleaved with other actions
- **Allows artifact updates**: If implementation reveals design issues, suggest updating artifacts - not phase-locked, work fluidly


---

## Worktree wrap-up

Once the apply flow above has finished (all tasks complete, or the user stopped you at a good point), do these — still **inside the worktree**.

### W1. Commit the change in the worktree

Follow the `$speclink-commit` skill's attribution convention: stage only the files belonging to this change (its artifacts under `openspec/changes/<change-name>/` plus the source files recorded in the change's evidence record), leave unrelated dirty files alone, and write the commit message in the project's language.

The commit lands on branch `speclink/<change-name>` inside the worktree. Nothing reaches the main branch yet.

### W2. Stop here — do not merge, do not remove the worktree

This skill's job ends at the commit. Explicitly:

- **Do NOT** merge `speclink/<change-name>` back into the main branch.
- **Do NOT** run `git worktree remove`, and do NOT delete the branch.
- **Do NOT** switch the main checkout to another branch or touch it in any way.

Merging is a separate, human-triggered step: it needs a clean main tree, and a conflict there is the user's call, not yours. Leaving the worktree in place also means the main checkout's `speclink list` keeps showing this change with its `[worktree]` marker until the merge happens.

### W3. Hand off

Tell the user, plainly:

> 這個 change 已在 worktree 內完成並提交，尚未合併回主分支。
>
> 建議先在這個 worktree 內跑品質站——`$speclink-review`（工藝品質）∥ `$speclink-verify`（規格符合度），是否要跑由你決定。品質站的 Apply baseline 就在這個 worktree 裡，離開就沒有了。蓋章會寫進 change 的 meta，記得補一次提交。
>
> 品質站跑完（或決定略過）後執行 `$speclink-worktree-merge <change-name>` 收尾——它會檢查主樹是否乾淨、把分支合併回去，成功後移除 worktree 並刪掉分支。

Report alongside it: the worktree path, the branch name, and the tasks completed this session.
