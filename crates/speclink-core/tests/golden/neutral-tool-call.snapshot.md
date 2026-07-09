=== WAD.md ===
<!-- SPECLINK:START v1.2.0 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

Speclink verbs are executed by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]).

## Use the `speclink-*` skills when:

- Requirements are fuzzy or worth debating → `speclink-discuss` (recorded as a document; promote turns it into a change)
- User wants to plan, propose, or design a change → `speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `speclink-onboard`
- Tasks are ready to implement → `speclink-apply`
- Resuming a change that sat idle → run `speclink-drift` first
- Requirements change mid-work → `speclink-ingest`
- Implementation is done → `speclink-archive`
- Commit only files related to a specific change → `speclink-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → archive

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Requirements change mid-work? `ingest` → resume `apply`

<!-- SPECLINK:END -->

=== .wad/skills/speclink-apply/SKILL.md ===
---
name: speclink-apply
description: "Implement or resume tasks from a Speclink change"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Implement tasks from a Speclink change.

**Input**: Optionally specify a change name (e.g., `speclink apply add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Task tracking is file-based only.** The tasks file's markdown checkboxes (`- [ ]` / `- [x]`) are the single source of truth for progress. Do NOT use any external task management system, built-in task tracker, or todo tool. When a task is done, edit the checkbox in the tasks file — that is the only way to record progress.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `speclink list --json` to get all available changes. Use the **AskUserQuestion tool** to let the user select

   Always announce: "Using change: <name>" and how to override (e.g., `speclink apply <other>`).

2. **Check status to understand the schema**

   ```bash
   speclink status --change "<name>" --json
   ```

   **If the command fails**: show the error and STOP.

   **If the command succeeds**, mark the change as in-progress and proceed:

   ```bash
   speclink in-progress add "<name>"
   ```

   This is a silent operation — do not show the output to the user.

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
   - If `state: "blocked"` (missing artifacts): show message, suggest using `speclink propose` to create the change artifacts first
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
  - **Refresh first** — pause apply, run `speclink-ingest <change-name>` to update artifacts, then resume
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
   - Continue to next task

   **Pause if:**
   - Task is unclear → ask for clarification
   - Implementation reveals a design issue → suggest updating artifacts
   - Error or blocker encountered → report and wait for guidance
   - User interrupts

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

All tasks complete! You can archive this change with `speclink archive`.
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

=== .wad/skills/speclink-archive/SKILL.md ===
---
name: speclink-archive
description: "Archive a completed change"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Archive a completed change.

**Input**: Optionally specify a change name after `speclink archive` (e.g., `speclink archive add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select.

   Show only active changes (not already archived).
   Include the schema used for each change if available.

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check artifact completion status**

   Run `speclink status --change "<name>" --json` to check artifact completion.

   Parse the JSON to understand:
   - `schemaName`: The workflow being used
   - `artifacts`: List of artifacts with their status (`done` or other)

   **If any artifacts are not `done`:**
   - Display warning listing incomplete artifacts
   - Prompt user for confirmation to continue
   - Proceed if user confirms

3. **Check task completion status**

   Read the tasks file (typically `tasks.md`) to check for incomplete tasks.

   Count tasks marked with `- [ ]` (incomplete) vs `- [x]` (complete).

   **If incomplete tasks found:**
   - Display warning showing count of incomplete tasks
   - Prompt user for confirmation to continue
   - Proceed if user confirms

   **If no tasks file exists:** Proceed without task-related warning.

4. **Assess delta spec completeness**

   Check for delta specs at `openspec/changes/<name>/specs/`. If none exist, skip this step.

   Background: the archive CLI applies deltas mechanically — a MODIFIED requirement **wholesale-replaces** the canonical requirement block with the delta's content, and an ADDED requirement that already exists in the canonical spec is skipped (leaving it without a `@trace` marker). A delta is therefore safe to archive only when every MODIFIED requirement contains the complete final text (including scenarios that should survive unchanged).

   **For each delta spec, compare against `openspec/specs/<capability>/spec.md`:**
   - MODIFIED requirements: does the canonical requirement contain scenarios or content the delta omits but that should survive?
   - ADDED requirements: does the requirement already exist in the canonical spec (e.g., from an earlier mid-flight sync)?

   **If every delta is already complete final-state and no ADDED requirement pre-exists:** proceed directly to step 5 — no prompt needed.

   **Otherwise**, show a summary of what would be lost or skipped, then use the **AskUserQuestion tool**:
   - "Normalize delta then archive (recommended)": rewrite the delta spec file(s) in place —
     - merge the omitted canonical content into each MODIFIED requirement so it reads as the complete final state
     - convert each pre-existing ADDED requirement to MODIFIED (complete final state) so the CLI re-applies it and injects `@trace`
     - do NOT edit the main specs — only the delta files change
   - "Archive as-is": proceed, accepting that omitted canonical content will be lost on merge
   - "Cancel"

   After normalizing, show a brief diff summary of the rewritten delta files, then continue.

5. **Clean up tracking file**

   Delete `.speclink/touched/<change-name>.json` if it exists. This file contains implementation tracking data that is not needed after archiving.

   ```bash
   rm -f .speclink/touched/<change-name>.json
   ```

   If the file does not exist, silently continue.

6. **Perform the archive**

   Use the `speclink archive` CLI command which handles the full archive workflow
   (spec snapshot, delta application, @trace injection, identity recording):

   ```bash
   speclink archive <name>
   ```

   **Optional flags:**
   - `--skip-specs` — skip delta spec application (for tooling/doc-only changes)
   - `--mark-tasks-complete` — mark all incomplete tasks as complete before archiving
   - `--no-validate` — skip delta spec validation

   **If archive fails** with "already exists" error, suggest renaming existing archive.

7. **Display summary**

   Show archive completion summary including:
   - Change name
   - Schema that was used
   - Archive location
   - Spec sync status (synced / sync skipped / no delta specs)
   - Note about any warnings (incomplete artifacts/tasks)

**Output On Success**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** ✓ Synced to main specs

All artifacts complete. All tasks complete.
```

**Output On Success (No Delta Specs)**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** No delta specs

All artifacts complete. All tasks complete.
```

**Output On Success With Warnings**

```
## Archive Complete (with warnings)

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** Sync skipped (user chose to skip)

**Warnings:**
- Archived with 2 incomplete artifacts
- Archived with 3 incomplete tasks
- Delta spec sync was skipped (user chose to skip)

Review the archive if this was not intentional.
```

**Output On Error (Archive Exists)**

```
## Archive Failed

**Change:** <change-name>
**Target:** openspec/changes/archive/YYYY-MM-DD-<name>/

Target archive directory already exists.

**Options:**
1. Rename the existing archive
2. Delete the existing archive if it's a duplicate
3. Wait until a different date to archive
```

**Guardrails**

- Always prompt for change selection if not provided
- Use artifact graph (speclink status --json) for completion checking
- Don't block archive on warnings - just inform and confirm
- Preserve .openspec.yaml when moving to archive (it moves with the directory)
- Show clear summary of what happened
- Normalization rewrites delta files only — NEVER edit main specs directly; delta application is the archive CLI's job
- If delta specs exist, always run the completeness assessment; only prompt when normalization is actually needed
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response


## RENAMED is actually executed (speclink-specific)

Unlike Spectra — which documents `## RENAMED Requirements` but never applies a rename in
any syntax and always reports `renamed: 0` — speclink executes renames at archive time.
Both documented forms work:

```markdown
## RENAMED Requirements

- FROM: `### Requirement: Old Name`
- TO: `### Requirement: New Name`
```

or the header form (`### Requirement: Old Name` followed by a `TO: New Name` line). The
canonical requirement header is rewritten, the summary counts it under `renamed:`, and a
rename-only delta is a valid change (validates and archives). `speclink drift` checks
RENAMED targets in `spec_assumptions`, so a stale FROM target routes to ingest before
archiving instead of silently no-opping.

## Bulk archive (speclink-specific)

When several changes are finished, archive them in one pass:

```bash
speclink archive <name-a> <name-b>     # explicit set
speclink archive --all                 # every ready change
```

Semantics (the CLI enforces all three):

1. **Clean work tree required** — the dirty code-file set is the @trace source and would be
   injected into EVERY archived change's canonical specs. Commit first; the command refuses
   otherwise and lists the offending files.
2. **Skip, never silently** — a change is archived only when it is ready: tasks complete
   (or `--mark-tasks-complete`), validation passes (or `--no-validate`), and no stale delta
   assumptions (a MODIFIED/REMOVED target another change already rewrote — run
   `speclink drift <name>` and reconcile via ingest). Skipped changes are reported with the
   reason and a `Bulk archive: N archived, M skipped` summary.
3. **Fail-fast** — archives apply in created-date order; on the first hard error the run
   stops and reports archived / failed / untouched (already-archived changes cannot be
   rolled back automatically).

Each archived change still gets the full single-archive treatment: delta application with
@trace, snapshot for unarchive, `.started` cleanup, and its linked discussion archived
alongside. Delete each change's `.speclink/touched/<name>.json` afterwards as in step above.

=== .wad/skills/speclink-audit/SKILL.md ===
---
name: speclink-audit
description: "Audit changed code for security sharp edges — dangerous defaults, type confusion, and silent failures"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Audit changed code for security sharp edges — API design traps, dangerous defaults, and interfaces that make it easy to do the wrong thing.

Good APIs don't require developers to "be careful" to stay secure. If the correct usage requires reading docs, remembering rules, or understanding cryptography, the API has failed.

**Core principle:** Security should be the path of least resistance. Insecure usage should be harder than secure usage.

## Two Modes

This skill operates in two modes depending on how it's invoked:

- **Standalone** (`speclink audit`): Full 3-agent parallel analysis on current git diff. See [Standalone Mode](#standalone-mode).
- **Discipline** (via `speclink apply` when `audit: true`): Condensed checklist applied during implementation. See [Discipline Mode](#discipline-mode).

Both modes share the same [Core Framework](#core-framework).

---

## Standalone Mode

When invoked directly as `speclink audit`:

### Phase 1: Gather Changes

Run `git diff HEAD` to get the full diff of current modifications.

If there are no changes, report "No changes to audit" and stop.

### Phase 2: Parallel 3-Agent Analysis

Launch 3 agents in parallel (one message, 3 tool calls). Each agent receives the full diff and analyzes it through one adversary lens.

**Agent 1 — The Scoundrel (壞蛋)**

A malicious developer or attacker deliberately manipulating configuration.

Search the diff for:

- Config options that can disable security mechanisms
- Algorithm parameters that accept downgrades (e.g., `"none"`, `"md5"`)
- Values that can be injected to bypass validation
- Dangerous config combinations (e.g., `auth_required: true` + `bypass_auth_for_health: true` + `health_check_path: "/"`)
- String concatenation in security-critical paths (permissions, queries, paths)

**Agent 2 — The Lazy Developer (懶惰的開發者)**

A developer who copy-pastes examples and skips documentation.

Search the diff for:

- Unsafe defaults: `verify: false`, `timeout: 0`, empty strings as keys
- Zero/nil/empty behavior: what does `timeout=0`, `max_attempts=0`, `key=""` mean?
- Error messages that don't guide toward secure usage
- The "first example found" test: is the most obvious usage secure?
- Path of least resistance: does the simplest way to use this API produce secure results?

**Agent 3 — The Confused Developer (搞混的開發者)**

A developer who misunderstands API usage.

Search the diff for:

- Parameters that can be swapped without type errors (e.g., `encrypt(msg, key, nonce)` — key and nonce are both strings)
- Silent failures: security checks that return true/false where the return value can be ignored
- Raw primitives where semantic types should exist (strings for keys, bytes for nonces)
- Configuration cliffs: one wrong value = catastrophe with no warning (e.g., `verify_ssl: fasle`)
- Stringly-typed security: permissions as comma-separated strings instead of enums

### Phase 3: Consolidate and Fix

Merge findings from all 3 agents. For each finding:

- If fixable: apply the fix directly
- If false positive or not worth changing: skip without debate
- Classify severity: Critical / High / Medium / Low

End with a brief summary of what was fixed (or confirm the code is clean).

---

## Discipline Mode

When referenced by `speclink apply` (via `speclink instructions --skill audit`), do NOT launch the 3-agent workflow above. Instead, apply this condensed checklist continuously during implementation.

### Quick 3-Role Check

Before finalizing any code that involves APIs, configuration, parameters, or security-related logic, ask:

1. **Scoundrel**: Can this be abused? Can config disable security? Can values be injected?
2. **Lazy Developer**: Is the default safe? Will copy-paste usage be secure? Does the error message guide correctly?
3. **Confused Developer**: Can params be swapped? Will wrong usage fail loudly? Are types distinct enough?

### Red Flags During Implementation

Stop and fix immediately if you notice:

- Adding a string parameter for security-related logic → use enum or newtype
- Adding a config option that defaults to `false` → is the "off" state safe?
- `if value == 0` or `if key.nil?` → what does zero/nil MEAN in this context?
- Security check returns true/false → can the return value be ignored?
- Accepting algorithm/mode as a parameter → can it be hardcoded to the safe choice?
- Adding a config option without validation → what happens with invalid/malicious values?

### When to Engage

Not every line of code needs audit scrutiny. Focus on:

- New function signatures and public APIs
- Configuration options and their defaults
- Authentication, authorization, encryption interfaces
- Input validation and error handling at system boundaries
- Anywhere a developer makes a security-relevant choice

---

## Core Framework

### Three Adversaries

| Role                   | Mindset                                   | Key Questions                                                                     |
| ---------------------- | ----------------------------------------- | --------------------------------------------------------------------------------- |
| **Scoundrel**          | Malicious, deliberate exploitation        | Can I disable security via config? Downgrade algorithms? Inject values?           |
| **Lazy Developer**     | Copy-paste, skips docs, deadline pressure | Is the first example safe? Is the default secure? Do errors guide me right?       |
| **Confused Developer** | Misunderstands usage                      | Can I swap params silently? Will mistakes fail loudly? Are types distinguishable? |

### Six Trap Categories

#### 1. Algorithm Choice Traps

Letting developers choose algorithms = inviting them to choose wrong.

```ruby
# Dangerous: accepts arbitrary algorithm
OpenSSL::Digest.new(algorithm).hexdigest(password)  # algorithm = "md5"?

# Safe: no choice
BCrypt::Password.create(password)  # can't pick wrong
```

#### 2. Dangerous Defaults

Defaults that are insecure, or zero/empty values that disable security.

```ruby
# What does timeout=0 mean? Never expire? Expire immediately?
def verify_token(token, timeout: 300)
  return true if timeout == 0  # 0 = skip verification?!
end
```

**Key question:** What do `timeout=0`, `max_attempts=0`, `key=""`, `nil` each mean?

#### 3. Raw Primitives vs Semantic Types

Using raw bytes/strings instead of meaningful types invites type confusion.

```ruby
# Dangerous: both params are strings, swappable
encrypt(message, key, nonce)

# Safe: types protect against swapping
encrypt(message, Key.new(k), Nonce.new(n))
```

#### 4. Configuration Cliffs

One wrong config value = disaster, with no warning.

```yaml
# A typo = security mechanism disappears
verify_ssl: fasle # not "false", might be treated as truthy?

# Dangerous combination
auth_required: true
bypass_auth_for_health: true
health_check_path: "/" # oops, entire site bypasses auth
```

#### 5. Silent Failures

Security errors that don't surface, or "success" masking failure.

```ruby
# Silent bypass
def verify_signature(sig, data, key)
  return true if key.nil?  # no key = skip verification?!
end

# Return value ignored
result = crypto.verify(data, sig)  # returns false but nobody checks
```

#### 6. Stringly-Typed Security

Security-critical values as plain strings = open door for injection and confusion.

```ruby
# Dangerous: string concatenation
permissions = "read,write"
permissions += ",admin"   # too easy to escalate

# Safe: use enums
permissions = Set[Permission::READ, Permission::WRITE]
```

### Severity Classification

| Severity | Condition                                 | Example                                             |
| -------- | ----------------------------------------- | --------------------------------------------------- |
| Critical | Default or most obvious usage is insecure | `verify: false` is default, empty password accepted |
| High     | Easy misconfiguration breaks security     | Algorithm param accepts `"none"`                    |
| Medium   | Uncommon but possible misconfiguration    | Negative timeout has unexpected behavior            |
| Low      | Requires deliberate misuse                | Obscure parameter combination                       |

### Rationalization Table

| Excuse                                | Why It's Wrong                             | What To Do                                             |
| ------------------------------------- | ------------------------------------------ | ------------------------------------------------------ |
| "Docs explain it"                     | Devs skip docs under deadlines             | Make the safe option the default or only option        |
| "Advanced users need flexibility"     | Flexibility = foot-gun opportunity         | Provide safe high-level API, hide low-level primitives |
| "It's the developer's responsibility" | You designed the trap                      | Remove the trap or make it impossible to misuse        |
| "Nobody would do that"                | Devs under pressure do everything          | Assume maximum developer chaos                         |
| "It's just a config option"           | Config is code; wrong config ships to prod | Validate config, reject dangerous combinations         |
| "Backwards compatibility"             | Insecure defaults can't be grandfathered   | Deprecate loudly, force migration                      |

=== .wad/skills/speclink-commit/SKILL.md ===
---
name: speclink-commit
description: "Commit files related to a specific Speclink change"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Commit files related to a specific Speclink change.

This is a **utility skill** (not a workflow step). It reads source file tracking data and artifact changes to stage and commit only the files belonging to one change — useful when multiple changes are in progress simultaneously.

**Input**: Optionally specify a change name after `speclink commit` (e.g., `speclink commit add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires `git`. Run `git --version`. If git is not available (command not found or similar error), inform the user to install git and STOP.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select

   Always announce: "Committing for change: <name>"

2. **Read tracking file**

   Check for `.speclink/touched/<change-name>.json`. If it exists, parse it to get source files grouped by task.

   Expected format:

   ```json
   {
     "change": "<change-name>",
     "touched": [
       {
         "task_id": "1",
         "task_desc": "Task description",
         "files": ["src/file1.ts", "src/file2.ts"]
       }
     ]
   }
   ```

   If the file does not exist, proceed without source file data — only artifact files will be included.

3. **Collect artifact files**

   Run `git status --porcelain` and filter the output to files under `openspec/changes/<name>/`. These are the change's artifact files (proposal, design, tasks, specs, etc.).

4. **Identify unrelated dirty files**

   From the full `git status --porcelain` output, any dirty files NOT in the artifact set and NOT in the tracking file are "unrelated changes."

5. **Display commit plan**

   Show the file list grouped into sections:

   ```
   ## Commit Plan: <change-name>

   ### Change Artifacts
   - M  openspec/changes/<name>/proposal.md
   - M  openspec/changes/<name>/tasks.md

   ### Source Files
   **Task 1: <task description>**
   - M  src/lib/components/search.svelte
   - A  src/lib/stores/search.ts

   **Task 3: <task description>**
   - M  src/routes/+page.svelte

   ### Unrelated Changes (not included)
   - M  src/lib/utils/format.ts
   - ??  tmp/scratch.js
   ```

   If no tracking file was found, show a warning instead of the Source Files section:

   ```
   ### Source Files
   ⚠ No source file tracking data found.
   Only artifact files will be committed. Use `speclink task done` during apply to enable source file tracking.
   ```

   If there are no artifact files AND no tracked source files, inform the user that there is nothing to commit and STOP.

6. **User confirmation**

   Use the **AskUserQuestion tool** to ask the user how to proceed.

   Options:
   - **Commit as shown**: Proceed with the displayed artifact + source files
   - **Include all dirty files**: Add all unrelated files to the commit as well
   - **Customize**: Let the user add or remove specific files from the commit set
   - **Archive first, then commit together**: Run archive before committing — archive file moves will be included in this commit

   If the user selects "Customize":
   - Show a numbered list of all dirty files (included and excluded)
   - Ask which files to add or remove
   - Re-display the updated commit plan for confirmation

   If the user selects "Archive first, then commit together":
   - Proceed to step 6a (Archive sub-flow) before continuing to step 7

6a. **Archive sub-flow** (only when the user selected "Archive first, then commit together")

    This sub-flow executes three checks in sequence before returning to the main commit flow.

    **6a-i. Incomplete task handling**

    Run `speclink artifact cat tasks --change "<name>"` and count `- [x]` (complete) and `- [ ]` (incomplete) checkboxes in the output.

    - If **all tasks are complete**: skip to 6a-ii.
    - If **incomplete tasks exist**:
      - Display the list of incomplete tasks
      - Use the **AskUserQuestion tool** to ask: "These tasks are still incomplete. Mark all as complete before archiving?"
        - **Yes**: set a flag to pass `--mark-tasks-complete` to `speclink archive`
        - **No**: proceed without the flag (archive will continue with a warning)

      If **AskUserQuestion tool** is not available, ask the same question as plain text and wait for the user's response.

    **6a-ii. Delta spec completeness check**

    Check whether delta specs exist at `openspec/changes/<name>/specs/`.

    - If **no delta specs exist** (directory is empty or absent): skip to 6a-iii.
    - If **delta specs exist**: compare each delta against `openspec/specs/<capability>/spec.md`. The archive CLI wholesale-replaces a MODIFIED requirement with the delta's content and skips an ADDED requirement that already exists in the canonical spec — so a partial delta (one that omits canonical content that should survive) loses that content on archive.
      - If every delta is complete final-state and no ADDED requirement pre-exists: skip to 6a-iii.
      - Otherwise use the **AskUserQuestion tool** to ask: "Delta specs are not complete final-state. Normalize before archiving?"
        - **Yes**: rewrite the delta files in place — merge the omitted canonical content into MODIFIED requirements, convert pre-existing ADDED requirements to MODIFIED — then proceed. Do NOT edit main specs.
        - **No**: proceed as-is (omitted canonical content will be lost on merge)

      If **AskUserQuestion tool** is not available, ask the same question as plain text and wait for the user's response.

    **6a-iii. Archive execution and file collection**

    Execute the archive:

    ```bash
    speclink archive <name>          # without --mark-tasks-complete
    speclink archive <name> --mark-tasks-complete  # if user chose to mark tasks complete in 6a-i
    ```

    After archive completes successfully:

    1. Re-run `git status --porcelain` to capture all file changes produced by the archive (deletions from `openspec/changes/<name>/`, additions in `openspec/archived/`)
    2. Add these archive-related file changes to the commit set
    3. Display an **updated commit plan** showing all sections:

    ```
    ## Updated Commit Plan: <change-name> (with archive)

    ### Change Artifacts (archived)
    - D  openspec/changes/<name>/proposal.md
    - D  openspec/changes/<name>/tasks.md
    - ...

    ### Archived Files
    - A  openspec/changes/archive/<name>/proposal.md
    - A  openspec/changes/archive/<name>/tasks.md
    - ...

    ### Source Files
    (same as before)

    ### Main Spec Updates (from archive's delta application)
    - M  openspec/specs/<spec-name>/spec.md
    - ...
    ```

    Then continue to step 7.

7. **Generate commit message**

   Run `speclink artifact cat proposal --change "<name>"`. Extract the first sentence from the Why section (or Problem/Summary section if Why is absent).

   Generate a message in this format:

   ```
   speclink(<change-name>): <summary>

   Change: <change-name>
   Tasks: <completed>/<total> complete
   ```

   If the archive sub-flow was executed (user selected "Archive first, then commit together"), add `Archived: yes` to the message body:

   ```
   speclink(<change-name>): <summary>

   Change: <change-name>
   Tasks: <completed>/<total> complete
   Archived: yes
   ```

   Task progress comes from reading the tasks file and counting `- [x]` vs `- [ ]` checkboxes.

   Show the generated message to the user and allow editing before proceeding.

8. **Selective staging**

   Stage each confirmed file individually:

   ```bash
   git add <file1>
   git add <file2>
   ...
   ```

   **NEVER use `git add .` or `git add -A`.** Each file must be staged explicitly.

9. **Commit**

   ```bash
   git commit -m "<message>"
   ```

10. **Show result**

    ```bash
    git log --oneline -1
    ```

    Display the commit hash and message to confirm.

**Output On Success**

```
## Committed: <change-name>

**Commit:** <short-hash> speclink(<change-name>): <summary>
**Files:** <N> files committed (<A> artifacts, <S> source files)
**Tasks:** <completed>/<total> complete
```

**Output On Nothing To Commit**

```
## Nothing to Commit

**Change:** <change-name>

No dirty files found for this change (no modified artifacts, no tracked source files).
```

**Guardrails**

- **NEVER use `git add .` or `git add -A`** — every file must be staged individually with `git add <file>`
- **NEVER commit files the user hasn't confirmed** — always show the file list and get explicit confirmation first
- **Always show the full file list before committing** — no silent staging
- If the tracking file is missing, warn but don't block — artifact-only commits are valid
- The "Unrelated Changes" section is informational only — these files are excluded by default
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

=== .wad/skills/speclink-discuss/SKILL.md ===
---
name: speclink-discuss
description: "Have a focused discussion that is recorded to a discussion document"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Have a focused discussion about a topic and reach a conclusion.

**IMPORTANT: Discuss mode is for thinking, not implementing.** You may read files, search code, and investigate the codebase, but you must NEVER write code or implement features. If the user asks you to implement something, remind them to exit discuss mode first (e.g., start a change with `speclink propose`). You MAY create Speclink artifacts (proposals, designs, specs) if the user asks—that's capturing thinking, not implementing.

**This is a task-oriented discussion.** Every discussion has a topic, works toward a goal, and ends with a clear conclusion. Unlike open-ended exploration, discuss mode converges.

**Input**: The argument after `speclink discuss` is the topic. Could be:

- A design question: "should we use WebSockets or SSE?"
- A problem to solve: "the auth system is getting unwieldy"
- A change name: "add-dark-mode" (discuss in context of that change)
- An architecture decision: "how to structure the plugin system"
- A vague idea that needs sharpening: "real-time collaboration"

**Not every topic is a discussion.** If the request is really a question — the user wants to understand how something works or whether something is feasible, and no decision hangs on the answer — answer it directly in the conversation and do not open a discussion record. A discussion exists to settle something; understanding-seeking without a verdict is ask-shaped, not discuss-shaped. When in doubt, start talking without a record: the record is only created at the first substantive round (see below), so nothing is lost by waiting.

---

## Recording the discussion (speclink)

Unlike an ephemeral chat, **every speclink discussion is persisted to a document** (`openspec/discussions/<slug>.md`) so the conversation keeps its thread across turns and sessions, and so a later `speclink propose --from-discussion <slug>` can seed a proposal directly from it. Drive the record through the CLI — never hand-write the file.

The document has a fixed skeleton — like the proposal template, every discussion record has the same shape:

```
## Context      ← the framing, set once when the record is created (discuss context)
## Rounds       ← ### Round N entries, appended per exchange (discuss add-round)
## Conclusion   ← the decision, written at convergence (discuss conclude)
```

**Document rules** (the record is a Socratic ledger, not a transcript):

1. **One focus per round.** Each recorded round distills exactly one question examined and what it settled — never a dump of everything said.
2. **Rounds are append-only.** Never rewrite an earlier round. When a position changes, a new round names what changed and why — the reasoning trail is the point.
3. **Record what was ruled out, with the reason.** Rejected options are the most valuable part of the record: they stop future readers (and future you) from re-litigating settled ground.
4. **Keep an open-questions ledger.** Each round ends with the questions still unresolved; the next round picks one of them up. The conclusion must resolve or explicitly defer every remaining one.
5. **Position bullets over prose.** When a Position exceeds one sentence it SHALL be bulleted — a one-sentence verdict first, then `- ` points one per line. A single-line wall-of-text Position is unreadable in every viewer. Focus / Ruled out / Open stay single-line.

**At the start (before Step 0):**

1. Check for an existing open discussion on this topic:
   ```bash
   speclink discuss list --json
   ```
2. If one matches the topic and its `status` is `open`, resume it (reuse its `slug`) — the record already exists; every later `add-round`/`conclude` call uses it.
3. Otherwise **do not create the record yet.** Derive an English kebab-case slug from the topic (translate when the topic is not English — e.g. 「看板搜尋列」 → `board-search-bar`), announce it ("This discussion will be recorded as `<slug>` once it has substance."), and proceed — scout, pick a mode, present your first assumptions or question with nothing on disk. A mis-invocation or a topic answered in one exchange leaves no file behind.

**Create the record at the first substantive round** — the moment an exchange actually moves the topic (your assumptions list drew confirmations or corrections, an interview answer settled something). Right before recording that first round, run:

```bash
speclink discuss new "<topic>" --slug <english-kebab-slug>
```

Always pass `--slug` with the English kebab-case slug you derived — the slug names the record file; the topic stays in the user's language verbatim. Without `--slug` the filename falls back to deriving from the topic, so non-English topics produce non-English filenames. Write the Context section (below), then `add-round`. From here on the record is live and every exchange is persisted.

**When the record is created**, fill the Context section once — the framing a future reader (or `propose`) needs before the rounds make sense:

```bash
speclink discuss context <slug> --stdin <<'CTX_EOF'
What prompted this discussion, the mode chosen (assumptions | interview) and why,
and the related changes/specs found by the codebase scout.
CTX_EOF
```

**After each round** (each Assumptions list you present, or each interview question-and-answer that moves the topic forward), persist a concise summary so the record shows how the thinking evolved:

```bash
speclink discuss add-round <slug> --mode assumptions --stdin <<'ROUND_EOF'
**Focus**: the one question this round examined
**Position**: one-sentence verdict of the direction taken, expanded as bullets:
- one point per line — the decision detail and the evidence (files, probe results) behind it
- keep bulleting until the position is fully stated; never fold it back into one long line
**Ruled out**: options eliminated this round — each with the reason it lost
**Open**: questions still unresolved, for the next round to pick up
ROUND_EOF
```

Use `--mode assumptions` or `--mode interview` to match the mode you picked in Step 3. Omit a line rather than pad it (e.g. no `**Ruled out**` when nothing was eliminated). Keep each round terse — it is a durable summary following the Document rules above, not a transcript. This is the mechanism that keeps a long discussion from drifting off-topic: each round is anchored to the record.

**At convergence**, write the conclusion into the record (see the Convergence and "Capture decisions" sections below):

```bash
speclink discuss conclude <slug> --stdin <<'CONCLUSION_EOF'
**Decision**: ...
**Rationale**: ... (the key trade-off that drove it)
**Rejected alternatives**: ... (each with why it lost)
**Deferred**: open questions intentionally left unresolved — or "none"
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md
**Next**: speclink propose --from-discussion <slug>
CONCLUSION_EOF
```

This flips the record's `status` to `concluded`. The step logic below (vocabulary load, codebase scout, mode selection, interface depth check, convergence, conclusion capture) is unchanged — recording sits alongside it.

**Fast path**: when the user wants to go straight from the conclusion to a change without a full propose round, offer:

```bash
speclink discuss promote <slug>            # change name defaults to the slug
speclink discuss promote <slug> --name <change-name>
```

This scaffolds the change, prefills the proposal's Why from the conclusion, and links both sides (`from_discussion` in the change metadata, `status: promoted` + `promoted_to` in the record). One discussion can fan out into several changes — promote (or `speclink propose --from-discussion`) again and `promoted_to` accumulates each name; the discussion is archived automatically when the last of its changes is archived. The remaining artifacts are still created via `speclink propose`.

**Conclusion routed to an EXISTING change**: when the conclusion's **Capture to** points at a change already in flight (the decision updates its artifacts instead of spawning a new one), run link first, then hand off to ingest:

```bash
speclink discuss link <slug> <existing-change>
```

`link` forges the change-side chain (`from_discussion` in the change metadata) without scaffolding anything, so drawer links and auto-archive engage — the discussion is archived automatically when the last linked change is archived. Unlike promote, `link` does NOT mark the discussion 已轉出 (`promoted`): that reflection is sealed by `speclink ingest`, which folds the decision into the change's artifacts and then runs `speclink discuss seal` — so the discussion flips to promoted only once its content has actually landed, never at link time. Then run `speclink ingest <existing-change>` to fold the decision in and seal. Without the link, a concluded-then-ingested discussion sits on the board forever with nothing to archive it.

**Lifecycle**: a discussion that concluded without spawning a change (an explicit "don't do this" is a valid outcome) should be closed out with:

```bash
speclink discuss archive <slug>       # → discussions/archive/<created>-<slug>.md
```

Archived discussions stay readable — `speclink discuss show <slug>` falls back to the archive, and `speclink discuss list --archived` lists them. The slug becomes free for a future discussion.

A discussion that turns out not to be needed at all — the user abandons it mid-way, or the topic proved ask-shaped after all — is **discarded**, not archived:

```bash
speclink discuss discard <slug>            # refuses once rounds exist
speclink discuss discard <slug> --force    # delete despite recorded rounds
```

`discard` deletes the live record (archived records are never touched). Once rounds exist it refuses without `--force` — a discussion that examined real trade-offs should keep its reasoning through `conclude` + `archive`, even when the conclusion is "don't do this". Use `discard` freely for records that settled nothing; never leave an abandoned discussion sitting `open`.

---

## Before You Speak

Before asking anything, load the shared vocabulary, then do a quick codebase scout to decide how to run this discussion.

### Step 0: Load shared vocabulary

Run `speclink language show`. It prints the project's canonical vocabulary — terms with `definition`, `avoid`, and `why` notes, plus principles for when legacy terminology may remain.

- **If the command succeeds**: scan the canonical terms and their avoided synonyms. Prefer the canonical term when you summarize, capture conclusions, or update artifacts. If you notice a relevant `avoid` synonym in the user's topic or in the artifacts you read, plan to surface that as vocabulary drift in the conclusion.
- **If the command fails (no vocabulary document)**: continue silently with the normal flow. A missing vocabulary is not an error; do not announce it, do not block, and do not stop to ask the user to create it.

This step runs before the codebase scout, the assumptions list, the interview questions, and the conclusion capture.

### Step 1: Extract search terms

Pull 2-5 keywords from the user's topic. For "search should support fuzzy matching", that's `search`, `fuzzy`, `match`. For "should we add a plugin system", that's `plugin`, `extension`, `module`.

### Step 2: Scout the codebase

Use Grep and Glob to find related source files (not docs, not tests — source code). Spend no more than a few seconds on this. Read up to 5 of the most relevant files found.

### Step 3: Pick a mode

- **3+ related source files found** → **Assumptions mode**: you have enough context to form opinions. List your assumptions, let the user correct.
- **Fewer than 3 related source files found** → **Interview mode**: not enough code to base assumptions on. Fall through to "How to Discuss" below and ask questions one at a time.

Announce which mode you picked and why: "Found `search.rs`, `SearchPanel.svelte`, `search-store.ts` — I have enough context to list my assumptions." or "Didn't find much related code — I'll ask questions instead."

### Assumptions mode

When you enter assumptions mode, present 3-5 assumptions. Each one MUST include:

1. **Approach**: what you'd do and why
2. **Evidence**: file path(s) that informed this assumption
3. **If wrong**: concrete consequence of getting this wrong

Example:

```
### My assumptions

1. **New IPC command goes in `commands/search.rs`**
   Evidence: existing search commands are in `src-tauri/src/commands/search.rs`
   If wrong: we'd need to create a new module and register it

2. **Use the existing `SearchStore` for state**
   Evidence: `src/lib/stores/search-store.ts` already manages search state
   If wrong: parallel state would cause sync bugs

3. **Fuzzy matching runs in Rust, not frontend**
   Evidence: current search scoring is in `search.rs:calculate_score()`
   If wrong: moving to frontend means rewriting the scoring logic in TypeScript
```

After presenting, ask: **"Which of these are wrong?"**

- If the user says all are fine → proceed to Convergence with these as established context.
- If the user flags corrections → for each one, ask ONE focused follow-up question to understand their intent, then proceed to Convergence with the corrected understanding.

### Mode switching

The user can switch modes at any time during the discussion:

- **"Ask me questions instead"** / **"one at a time"** → switch to interview mode (the "How to Discuss" section below)
- **"Just list your assumptions"** / **"what do you think?"** → run the codebase scout if not done yet, then switch to assumptions mode

### Step 4: Interface depth check (conditional)

After the codebase scout, evaluate whether the topic introduces a new architectural seam. Run this check **only** when the topic involves at least one of:

- A **new module** (a new Rust crate, file under `src-tauri/src/commands/`, or a new top-level Svelte module).
- A **new IPC command** (a new `#[tauri::command]` exposed to the frontend, or a new front-to-back message shape).
- A **cross-layer Rust ↔ Tauri ↔ Svelte flow** that did not exist before.
- A **new storage abstraction** (new on-disk format, new database table, new file-system layout, new adapter over existing storage).

If none of those conditions apply, **skip this check**. Topics that only change static UI copy, visual styling, documentation wording, or other non-architectural surfaces SHALL skip the depth check entirely. The vocabulary load from Step 0 still happens; nothing else from this step runs.

When the check is triggered, work through these four questions before you finalize assumptions or interview answers:

1. **Seam location** — where does the boundary belong? Name the module, file, or store that owns the new contract.
2. **Adapter count** — is there exactly one adapter on this path, or are several thin wrappers stacked on each other?
3. **Depth** — what behaviour is hidden behind the interface? If the answer is "nothing — it just forwards calls", the seam is too shallow.
4. **Deletion test** — if you deleted this module today, what would break? If nothing meaningful breaks, the module is a pass-through and probably should not exist.

Surface the answers in the conclusion (or the assumptions list, if you are in assumptions mode) so the depth question is part of the captured decision, not an internal note.

---

## How to Discuss

_This section applies to interview mode — either chosen automatically (insufficient code context) or switched to manually by the user._

**One question at a time.** Don't dump a list of 10 questions. Ask the most important one, listen, then follow up. Let the conversation breathe. If the user's initial description or previous answers already cover a question, skip it — don't ask what you already know.

**Propose concrete options.** When exploring approaches, present 2-3 specific options with trade-offs — not abstract possibilities. Use comparison tables when helpful:

```
| Approach      | Pros              | Cons              |
|---------------|-------------------|-------------------|
| WebSockets    | Real-time, bidir  | Complex, stateful |
| SSE           | Simple, HTTP      | One-way only      |
| Polling       | Simplest          | Latency, waste    |
```

**Ground in reality.** Investigate the actual codebase when relevant. Map existing architecture, find integration points, surface hidden complexity. Don't just theorize.

**Visualize freely.** Use ASCII diagrams when they clarify thinking:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Client  │────▶│  Server  │────▶│    DB    │
└──────────┘     └──────────┘     └──────────┘
```

System diagrams, state machines, data flows, dependency graphs — whatever helps.

**Challenge assumptions.** Including the user's and your own. Ask "do we actually need this?" Apply YAGNI — the simplest solution that works is often the best.

**Be direct.** If you have a recommendation, say it. Don't hedge endlessly. "I'd go with option B because..." is more useful than "all options have merit."

**No empty validation.** Never pad responses with hollow affirmations. These add nothing and erode trust:

- ~~"That's an interesting approach"~~ → State what specifically is interesting and why
- ~~"There are many ways to think about this"~~ → Name the 2-3 concrete ways and their trade-offs
- ~~"That could work"~~ → Explain why it would or wouldn't work, and under what conditions
- ~~"Great question"~~ → Just answer the question
- ~~"You raise a good point"~~ → Engage with the point directly

If you agree, say why. If you disagree, say why. Empty agreement is worse than honest pushback.

**Push for specifics.** When the user gives a vague answer, don't accept it — dig deeper. The goal is to reach decisions concrete enough to implement.

Bad vs. good:

```
User: "We should make it more modular"
Bad:  "That sounds good. How would you like to proceed?"
Good: "What would you split out? Are we talking separate crates,
       feature flags, or a plugin interface? Each has very different
       cost."
```

```
User: "Performance might be an issue"
Bad:  "Good point, we should keep performance in mind."
Good: "What's the threshold? Are we talking sub-100ms response time,
       handling 1000 concurrent users, or keeping memory under a
       budget? The answer changes the architecture."
```

```
User: "We need better error handling"
Bad:  "Agreed, error handling is important."
Good: "Which errors are causing problems now? Are users seeing
       crashes, silent failures, or unhelpful messages? Let's look
       at the actual error paths."
```

---

## Convergence

Discussions must converge. As the conversation progresses:

1. **Narrow the options** — eliminate approaches that don't fit
2. **Surface the key trade-off** — most decisions come down to one fundamental tension
3. **Make a recommendation** — or help the user make one
4. **State the conclusion clearly** — what was decided, and why

The conclusion should be one of:

- **Design decision**: "We'll use SSE because one-way is sufficient and it's simpler"
- **Direction consensus**: "The auth refactor should split into gateway + provider"
- **Next-step recommendation**: "We need to spike the plugin API first to validate the approach"
- **Explicit deferral**: "We don't have enough info yet. Specifically, we need to know X before deciding"

**Example elicitation**: When the discussion converges on a specific requirement or behavior, propose a concrete example before capturing the decision. Instead of concluding "search should sort by relevance", propose: "So if we have items scored 0.9, 0.3, 0.7, the result order would be 0.9, 0.7, 0.3 — is that right?" This naturally produces `##### Example:` content for the spec and confirms shared understanding with real values.

**If the user wants to move faster.** Sometimes the user signals impatience — "let's just go with X", "I don't want to overthink this", "can we move on?". Respect their pace:

1. **First time**: Briefly flag if there's an important unresolved question — one sentence, not a lecture. "Before we commit to X, worth noting that Y could affect Z. Want to address it or move forward?"
2. **If they push again**: Respect it. Skip remaining questions, go straight to convergence with the best conclusion you can form from what's been discussed. Don't push back a second time.

The goal is thoroughness, not interrogation. One nudge maximum.

---

## Speclink Awareness

You have full context of the Speclink system. Use it naturally.

### Check for context

At the start, quickly check what exists:

```bash
speclink list --json
```

If the user mentioned a specific change name, read its artifacts for context.

### Capture decisions

When the discussion converges, **proactively present a conclusion summary**. Don't wait to be asked — propose it, and let the user opt out.

Summary format:

```
## Conclusion

**Decision**: [What was decided]
**Rationale**: [Why — the key trade-off that drove this]
**Rejected alternatives**: [What lost, and why]
**Deferred**: [Open questions intentionally left unresolved — or "none"]
**Capture to**: [Where this should be recorded]
```

Where to capture:

| Insight Type               | Where to Capture             |
| -------------------------- | ---------------------------- |
| New requirement discovered | `specs/<capability>/spec.md` |
| Design decision made       | `design.md`                  |
| Scope changed              | `proposal.md`                |
| New work identified        | `tasks.md`                   |
| Vocabulary drift           | `openspec/LANGUAGE.md`    |

**Vocabulary drift** means the discussion surfaced a recurring concept that is missing, ambiguous, or pulling away from the shared vocabulary loaded in Step 0. Examples: the topic uses a term that the vocabulary lists as an `avoid` synonym, or the discussion repeatedly names a concept that has no entry yet. When this happens, name it as vocabulary drift in the conclusion summary and direct the capture to `openspec/LANGUAGE.md`. The conclusion summary SHALL preserve this contract — do not silently rewrite the term in the artifacts without recording the drift.

Present the summary and say something like "I'll capture this to design.md unless you'd rather not." Default to capturing — the user can decline.

### Transition to action

When the discussion converges on building something:

- First record the conclusion in the discussion document: `speclink discuss conclude <slug> --stdin` (see "Recording the discussion" above). This flips its status to `concluded`.
- Then: "Ready to formalize this? `speclink propose --from-discussion <slug>`" — propose will seed the proposal from the recorded Decision and rounds.
- Or capture the decision in existing artifacts and continue

---

## Guardrails

- **Do record the discussion** — Announce the intended English kebab-case slug at the start, open the record at the first substantive round (`speclink discuss new` with `--slug`), append a round after each exchange, and `conclude` at the end. The document is the durable thread; keep it current. If the discussion is abandoned instead, `speclink discuss discard` the record — never leave it sitting `open`.
- **Don't implement** — Never write code or implement features. Creating Speclink artifacts and discussion records is fine, writing application code is not.
- **Don't leave without a conclusion** — If the user tries to end without a conclusion, summarize where things stand and state what's unresolved.
- **Don't fake understanding** — If something is unclear, dig deeper.
- **Don't overwhelm** — One question at a time, not a barrage.
- **Don't over-engineer** — Challenge complexity. Prefer simpler solutions.
- **Do visualize** — A good diagram is worth many paragraphs.
- **Do explore the codebase** — Ground discussions in reality.
- **Do be opinionated** — Have a recommendation. The user can disagree.

=== .wad/skills/speclink-drift/SKILL.md ===
---
name: speclink-drift
description: "Detect drift between a Speclink change and the current codebase state"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Detect drift between a Speclink change and the current codebase state. Reports time dormancy, broken design anchors, task collisions with external commits, and a single recommended next command.

**Input**: Optionally specify a change name (e.g., `speclink-drift add-auth`). If omitted, infer from conversation context or auto-select if only one active change exists.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Determine change name**

   If not provided, infer from context or run `speclink list --json` to auto-select. If multiple active changes exist and no name is given, list candidates and ask the user to rerun with an explicit name.

2. **Run programmatic drift analysis**

   ```bash
   speclink drift <change-name> --json
   ```

   The JSON contains:
   - `severity`: `"light"` / `"medium"` / `"heavy"`
   - `total_score`: aggregate over Time / Structure / Tasks / Specs (Environment is display-only)
   - `dimensions`: array of `{ kind, status, score, contributes_to_total }`
   - `broken_anchors`: design.md references that no longer resolve. Only code-like tokens anchor (camelCase / snake_case / multi-hump PascalCase, plus backticked expressions); prose capitalized words never do. Backticked file paths (`hr/index.html`) are existence-checked and report as category `File`. The change's own directory is excluded from the symbol search, so a committed design does not satisfy its own anchors.
   - `spec_assumptions`: delta-spec operations whose canonical target has drifted — a MODIFIED/REMOVED/RENAMED requirement that no longer exists, or an ADDED requirement that now already exists. **Archiving would silently skip these**; any entry here routes the recommendation to ingest.
   - `tasks_blocked_external`: pending tasks referencing a file that was touched by commits since `created` and no longer exists
   - `tasks_maybe_resolved`: pending tasks referencing a file that was committed since `created` and exists — the work may already be done
   - `commits_since_created` and the Environment status: commits since the created date (midnight-anchored), with a count of how many touch files this change references (touched record + task references)
   - `primary_recommendation`: a single copy-pasteable command line

3. **Present the report**

   **Report language**: run `speclink instructions apply --change "<name>" --json` and use its `locale` field (e.g., "Traditional Chinese (繁體中文)") — write the report in that language, prose, headings, and table labels included. Keep severity labels (light/medium/heavy), command lines, and code references in English. If the field is absent or the call fails, write in English.

   Use a user-readable, conclusion-first format. The first substantive paragraph after the title MUST be a plain-language conclusion that says what to do next before showing score tables, broken anchors, task collisions, or severity labels.

   Translate severity into action-oriented meaning:
   - **Light**: the change can continue with apply.
   - **Medium**: the change can continue, but the plan should be refreshed before implementation.
   - **Heavy**: the old plan is likely unsuitable for direct implementation; restart or refresh first.

   Recommended shape:

   ```markdown
   ## Drift Report: <change-name>

   <Plain-language conclusion. Example for medium: "This change can continue, but update the plan before implementing it. Related code has changed since the plan was written, so applying the old tasks directly may cause rework or conflicts.">

   ### Why

   - <1-3 plain-language reasons derived from dimensions, broken anchors, and task collisions>

   ### Details

   | Item              | Result                                                    |
   | ----------------- | --------------------------------------------------------- |
   | Time              | <status>                                                  |
   | Design references | <broken anchor count or "No broken references">           |
   | Delta assumptions | <stale assumption count or "All delta targets still hold"> |
   | Pending tasks     | <blocked/maybe-resolved count or "No task collisions">    |
   | Environment       | <N commits, M touching this change's files>               |
   | Overall           | <light/medium/heavy, total score N>                       |

   ### Recommendation

   Run `<primary_recommendation>`.
   ```

   Keep technical details below the plain-language conclusion. List broken anchors, stale delta assumptions, blocked tasks, and maybe-resolved tasks only when non-empty. Omit empty technical detail sections entirely. Keep the report short enough to skim; the goal is to help the user decide, not to explain the scoring model.

   **Stale delta assumptions take priority in the conclusion**: they mean another change has already rewritten the canonical requirement this delta targets, and archiving as-is would silently drop or misapply the delta. Say so explicitly and lead with the ingest recommendation.

4. **Apply the recommendation interactively**

   Use the **AskUserQuestion tool** to offer one decision based on `severity`. Use plain-language option labels (in the report language) while preserving the exact command in each option description. Do NOT auto-invoke `speclink-apply`, `speclink-ingest`, or `speclink archive`; always wait for the user's choice.
   - **Light** (score 0-3, drift is minor):
     - Recommended label: "Directly start work"
       - Description: run `speclink-apply <name>`
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually
   - **Medium** (score 4-8, refresh worth doing):
     - Recommended label: "Refresh the plan"
       - Description: run `speclink-ingest <name>` with the broken references and task collisions as context
     - Alternate label: "Directly start work"
       - Description: run `speclink-apply <name>` only if the user knows the reported changes are harmless
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually
   - **Heavy** (score >8 or anchor decay >30%, design diverges from code):
     - Recommended label: "Archive and restart"
       - Description: run `<primary_recommendation>`
     - Alternate label: "Refresh the plan"
       - Description: try `speclink-ingest <name>` before restarting
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually

   If the **AskUserQuestion tool** is not available, present the same plain-language choices as text and wait for the user's response.

**Passive Trigger**

When `speclink-apply` is invoked on a change whose `.openspec.yaml created` date is more than 5 days ago AND no commits have touched the change directory in the past 3 days, the apply skill SHOULD run drift analysis first and surface findings before tasks begin. The trigger is guidance only and MUST NOT block apply from proceeding.

(Threshold reasoning: AI-assisted commits are daily-cadence, not weekly. A change sitting ≥5 days with ≥3 days of no commits is almost always genuine stagnation rather than normal pacing.)

**Guardrails**

- Read-only: NEVER modify files, artifacts, or git state based on drift findings
- The CLI caps anchor checks at 50 via `ANCHOR_CAP` in `speclink_core::drift` to bound run-time
- If `speclink drift` returns a non-zero exit code (e.g., older binary without the drift subcommand), report the error and stop
- Do NOT auto-invoke any follow-up command — recommendations are user-confirmed
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

=== .wad/skills/speclink-ingest/SKILL.md ===
---
name: speclink-ingest
description: "Update an existing Speclink change from external context"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Update an existing Speclink change — from a plan file or conversation context.

**Plan file support** is available when the tool has a plan directory (``). Otherwise, use conversation context to update artifacts.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Input**: Optionally specify a plan file path or name.

- `speclink ingest agile-discovering-rocket.md`
- `speclink ingest agile-discovering-rocket`
- `speclink ingest` (use conversation context or auto-detect plan file)

**Steps**

1. **Locate the requirement source**

   a. **Argument provided** → treat as plan file reference (prepend `` and append `.md` if needed)
   - If the file exists → use it as the plan file source, proceed to Step 2
   - If the file does NOT exist → report the error and **stop**

   b. **No argument, plan file detectable**:
   - If found and the file exists → use the **AskUserQuestion tool** to ask:
     - Option 1: Use the plan file
     - Option 2: Use conversation context
   - If the user picks plan file → proceed to Step 2
   - If the user picks conversation context → skip Step 2, go to Step 3

   c. **No argument, no plan file detectable**:
   - Check `` for recent files
   - If recent files exist → list 5 most recent with the **AskUserQuestion tool**, include "Use conversation context" as an additional option
   - If the user picks a file → proceed to Step 2
   - If the user picks conversation context → skip Step 2, go to Step 3

   d. **Conversation context fallback** (no plan files found at all):
   - Use conversation context to update artifacts
   - If conversation context is insufficient, use the **AskUserQuestion tool** to get more details
   - Warn: "No plan file found. Using conversation context."

   **Source is a discussion conclusion?** When the update being folded in comes from a concluded discussion — the discuss skill routed its **Capture to** at this existing change, or the change already carries a `from_discussion` link — treat that discussion's conclusion as a first-class source, not just the conversation context:

   1. Discover the linked discussion(s): `speclink show <change> --json` exposes `fromDiscussions` (the change's `from_discussion` chain).
   2. Read each linked discussion's conclusion and fold it into the artifacts alongside the conversation/plan context (merge, do not replace what you already have):

      ```bash
      speclink discuss show <slug> --json
      ```

   3. Confirm the lifecycle chain is forged before touching artifacts:

      ```bash
      speclink discuss link <slug> <change>
      ```

   `link` is idempotent (a no-op when the discuss step already ran it) and forges ONLY the change-side chain — it does NOT mark the discussion promoted. Marking it 已轉出 is sealed at the END of this workflow (see the final step), once the artifacts actually carry the discussion's content. Without the link, the discussion never archives with the change it fed.

   **Change flagged stale (`restaleFrom`)?** When `speclink show <change> --json` exposes a non-empty `restaleFrom`, those discussions were re-concluded *after* this change last sealed them — its artifacts are now stale against the newer conclusions (`speclink analyze <change>` surfaces the same as an informational finding, and the desktop board shows a "待重新反映" badge). This is exactly a re-ingest: for each slug in `restaleFrom`, read the discussion's current conclusion (`speclink discuss show <slug> --json`), fold the revised decision into the artifacts, and the seal at the END of this workflow clears that slug from `restaleFrom` — marking the reflection honest again.

2. **Parse the plan structure** (skip if using conversation context)

   Claude Code plan files typically contain:
   - **Title** (`# ...`) — the high-level goal
   - **Context** section — background, motivation, current state
   - **Stages/Steps** — numbered implementation stages with goals and file lists
   - **Files involved** — list of files to modify/create
   - **Verification** section — how to test the changes

   Extract:
   - `plan_title`: from the H1 heading
   - `plan_context`: from the Context section
   - `plan_stages`: each numbered stage with its goal and file list
   - `plan_files`: all file paths mentioned
   - `plan_verification`: verification steps

3. **Check for active changes** (REQUIRED — ingest only updates existing changes)

   ```bash
   speclink list --json
   ```

   Parse the JSON output to get the full list of changes.
   - If one change exists → use the **AskUserQuestion tool** to confirm updating it
   - If multiple changes exist → use the **AskUserQuestion tool** to let user pick which one to update
   - If no changes at all → tell the user: "No active change found. Use `speclink propose` first to create one." and **stop**

4. **Select the change**

   Read existing artifacts for context before updating.

5. **Update artifacts**

   For each artifact, get instructions first:

   ```bash
   speclink instructions <artifact-id> --change "<name>" --json
   ```

   Use the `template` from instructions as the output structure. Apply `context` and `rules` as constraints but do NOT copy them into the file.

   The instructions JSON includes `locale` — the language to write artifacts in. If present, you MUST write the artifact content in that language. Spec files (specs/\*/\*.md) default to English instead — unless the project sets `spec_locale` in `.speclink.yaml` or `openspec/config.yaml` (a locale code, or `auto` to follow `locale`), in which case write spec prose in that language. Structural markers (`### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST) always stay in English.

   **Plan-to-Artifact Mapping** (when using a plan file):

   | Plan Section       | Artifact         | How to Map                                        |
   | ------------------ | ---------------- | ------------------------------------------------- |
   | Title              | Change name      | Convert to kebab-case                             |
   | Context            | proposal: Why    | Direct content transfer                           |
   | Stages overview    | proposal: What   | Summarize all stages                              |
   | Individual stages  | tasks.md groups  | One stage = one `##` heading, sub-items = `- [ ]` |
   | File paths         | proposal: Impact | Affected code list                                |
   | Verification steps | tasks.md         | Final verification task group                     |

   **Context-to-Artifact Mapping** (when using conversation context):

   | Conversation Element | Artifact         | How to Map                         |
   | -------------------- | ---------------- | ---------------------------------- |
   | Goal / requirement   | proposal: Why    | Extract motivation from discussion |
   | Discussed approach   | proposal: What   | Summarize agreed approach          |
   | Mentioned files      | proposal: Impact | Affected code list                 |
   | Discussion phases    | tasks.md groups  | One topic = one `##` heading       |

   **When updating an existing change:**
   - Merge new context into existing proposal (don't replace)
   - Add new tasks from plan stages or conversation, **preserve completed `[x]` items**
   - Do NOT remove existing content

   After creating each artifact, re-check status:

   ```bash
   speclink status --change "<name>" --json
   ```

   Continue until all `applyRequires` artifacts are complete. Show progress: "✓ Created <artifact-id>"

6. **Inline Self-Review** (before CLI analysis)

   After updating all artifacts, scan them manually. Fix issues inline, then proceed to the CLI analyzer.

   **Check 1: No Placeholders**

   These patterns are artifact failures — fix each one before proceeding:
   - "TBD", "TODO", "FIXME", "implement later", "details to follow"
   - Vague instructions: "Add appropriate error handling", "Handle edge cases", "Write tests for the above"
   - Delegation by reference: "Similar to Task N" without repeating specifics
   - Steps describing WHAT without HOW: "Implement the authentication flow" (what flow? what steps?)
   - Empty template sections left unfilled
   - Weasel quantities: "some", "various", "several" when a specific number or list is needed

   **Check 2: Internal Consistency**
   - Does every capability in the proposal have a corresponding spec?
   - Does the design reference only capabilities from the proposal?
   - Do tasks cover all design decisions, and nothing outside proposal scope?
   - Are file paths consistent across proposal Impact, design, and tasks?

   **Check 3: Scope Check**
   - More than 15 pending tasks → consider decomposing into multiple changes
   - Any single task would take more than 1 hour → split it
   - Touches more than 3 unrelated subsystems → consider splitting

   **Check 4: Ambiguity Check**
   - Are success/failure conditions testable and specific?
   - Are boundary conditions defined (empty input, max limits, error cases)?
   - Could "the system" refer to multiple components? Be explicit.

   **Check 5: Preservation Check** (ingest-specific)
   - Are all completed tasks `[x]` still present and unchanged?
   - Was existing content merged (not replaced)?

   **Check 6: Durable Handoff Review** (run BEFORE the CLI analyzer)

   The updated change has to survive being handed to another agent. Reject and fix any of the following on **incomplete** design and task content (do not rewrite completed `[x]` tasks):
   - **File-path-only tasks**: a pending task whose entire description is "edit file X" with no behavior, contract, or verification target. File paths are locator context — the task SHALL still describe what is observably true when complete.
   - **Line-number-coupled instructions**: design or task content that points to "line 42" / "the function on lines 80-95" as the only way to identify the work. Source line numbers drift; name the function, command, struct, or behavior instead.
   - **Vague acceptance criteria**: success conditions like "works correctly", "behaves as expected", "handles edge cases" without naming the observable behavior or the verification target (test name, CLI invocation, analyzer rule, manual assertion).
   - **Missing scope boundaries on non-trivial work**: design lacking explicit "in scope" / "out of scope" lines for any change that touches more than one subsystem or introduces new behavior. Trivial artifact-only edits MAY skip this; runtime, build, or tooling effects MUST NOT.

   Fix every failure inline using the existing context and the new plan/conversation source before running the CLI analyzer. Update incomplete design and task content so behavior contracts, verification criteria, and scope boundaries stay current with the new context. Preserve completed tasks unchanged.

---

## Rationalization Table

| What You're Thinking                                             | What You Should Do                                                            |
| ---------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| "The existing artifacts are close enough, just adjust the tasks" | Read the new context carefully. "Close enough" means you're missing something |
| "The proposal doesn't need updating, the change is the same"     | If new context exists, the proposal likely needs updates. At minimum, check   |
| "I can merge these tasks, they're basically the same"            | Keep tasks granular. Merged tasks are harder to track                         |
| "The completed tasks still apply, no need to review"             | Verify they're still relevant to updated scope. Don't blindly keep stale work |
| "This spec change is minor, skip the scenario update"            | If the requirement changed, the scenario must change                          |
| "The conversation didn't discuss this artifact, so skip it"      | Absence of discussion doesn't mean absence of impact. Check                   |

---

7. **Analyze-Fix Loop** (max 2 iterations)

   ```bash
   speclink analyze <name> --json
   ```

   1. Filter findings to **Critical and Warning only** (ignore Suggestion)
   2. If no Critical/Warning findings → show "Artifacts look consistent ✓" and proceed
   3. If Critical/Warning findings exist:
      a. Show: "Found N issue(s), fixing... (attempt M/2)"
      b. Fix each finding in the affected artifact
      c. Re-run `speclink analyze <name> --json`
      d. Repeat up to 2 total iterations
   4. After 2 attempts, if findings remain:
      - Show remaining findings as a summary
      - Proceed normally (do NOT block)

8. **Validation**

   ```bash
   speclink validate "<name>"
   ```

   If validation fails, fix errors and re-validate.

9. **Seal the reflection** (discussion-sourced ingests only)

   If this ingest folded a linked discussion's conclusion into the change, mark the reflection now that the content has landed:

   ```bash
   speclink discuss seal <slug> <change>
   ```

   `seal` flips the discussion to promoted (已轉出) and is idempotent — run it once per linked `from_discussion` slug (from `fromDiscussions` in `speclink show <change> --json`). This is what keeps "已轉出" honest: the discussion is marked reflected only after ingest actually carried its content in, never at link time. `seal` also clears that slug from the change's `restaleFrom` flag, so a re-ingest (triggered by a re-concluded discussion) closes the loop and the "待重新反映" marker disappears. Skip this step when no discussion fed the change.

10. **Summary and next steps**

   Show:
   - Source used: plan file (`<path>`) or conversation context
   - Change name and location
   - Artifacts created/updated
   - Validation result

   Use **AskUserQuestion tool** to confirm the workflow is complete. This ensures the workflow stops even when auto-accept is enabled. Provide exactly these options:
   - **First option (will be auto-selected)**: "Done" — End the ingest workflow. Inform the user they can run `speclink apply <change-name>` when ready.
   - **Second option**: "Apply" — Invoke `speclink apply <change-name>` to start implementation.

   If **AskUserQuestion tool** is not available, display the summary and inform the user to run `speclink apply <change-name>` when ready. Then STOP — do not continue.

   **After the user responds**, if they chose "Done", the workflow is OVER. If they chose "Apply", invoke `speclink apply <change-name>` to begin implementation.

**Guardrails**

- **NEVER** modify the original plan file in ``
- **NEVER** write application code — this skill only creates/updates Speclink artifacts
- **NEVER** create new changes — ingest only updates existing changes. If no active change exists, direct user to `speclink propose`
- When updating existing changes, **preserve all completed tasks** (`[x]`) — never revert progress
- If the source content is too brief to fill all artifact sections, use the **AskUserQuestion tool** to get more details rather than inventing content
- If `speclink` CLI is not available, report the error and stop
- Verify each artifact file exists after writing before proceeding to next
- **NEVER** skip the artifact workflow to write code directly
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

=== .wad/skills/speclink-onboard/SKILL.md ===
---
name: speclink-onboard
description: "Adopt Speclink on an existing codebase by generating initial specs from current behavior"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Adopt Speclink on an existing codebase by generating the initial canonical specs from current behavior.

**IMPORTANT: Onboarding documents what the system does TODAY — not what it should do.** Specs written here describe observed behavior with evidence. Aspirations, fixes, and improvements belong in a change (`speclink propose`) AFTER onboarding. Because nothing is changing, onboard writes directly to `openspec/specs/` — no change folder is involved.

**Input**: Optionally a scope hint after `speclink onboard` (e.g., "auth and billing only"). If omitted, onboard the whole codebase.

---

## Step 1: Check the current state

```bash
speclink list --specs
```

- **No specs yet** → full onboarding; continue below.
- **Some specs exist** → gap-filling mode: inventory what is NOT yet covered and scope the rest of this flow to those areas. Never rewrite an existing spec here — propose a change instead.

Read `openspec/config.yaml` (project context) and `.speclink.yaml` (`spec_locale` — write spec prose in the configured language; structural markers and SHALL/MUST keywords stay in English).

## Step 2: Inventory the codebase

Build a behavioral map before writing anything:

1. Read the README, package manifests, and entry points (main/CLI/server routes).
2. Scan the source tree for domains: commands, modules, services, UI surfaces.
3. Read the test suite — tests are the best source of observable behavior with concrete values.
4. Note behaviors you can VERIFY (code + tests) versus behaviors you can only INFER.

Spend effort proportional to repo size; for large repos, sample entry points and tests first.

## Step 3: Propose the capability map — and WAIT

Draft a capability list (kebab-case names, one behavior area each — the same granularity a change's delta specs would use). For each: one-line purpose + the evidence files behind it.

Present the map with the **AskUserQuestion tool** (or as plain text if unavailable) and let the user confirm, merge, split, or drop capabilities. **Do NOT write any spec before the map is confirmed** — wrong boundaries here are expensive to undo later.

## Step 4: Write the specs

For each confirmed capability, create `openspec/specs/<capability>/spec.md`:

```markdown
# <capability> Specification

## Purpose

<1-3 sentences: what this capability does for whom>

## Requirements

### Requirement: <Name>
<Observed behavior in SHALL form.>

#### Scenario: <name>
- **WHEN** <trigger observed in code/tests>
- **THEN** <outcome observed in code/tests>
```

Rules:

- **Evidence or flag it.** Every requirement must trace to code or tests you actually read. If a behavior is inferred but unverified, ask the user or leave it out — do not guess it into the record.
- Concrete scenarios: real values from tests make the best WHEN/THEN data; add `##### Example:` blocks where tests provide exact input→output pairs.
- Behavior only — no implementation details (module names, algorithms) in requirement text.
- 4 hashes for `#### Scenario:`, SHALL/MUST keywords in English, prose in `spec_locale`.

## Step 5: Validate and report

```bash
speclink validate --specs --all --strict
```

Fix structural findings, then report: capabilities created (with requirement/scenario counts), behaviors flagged as unverified, and areas deliberately left out. Suggest the natural next step:

> Specs now describe the current system. Future work goes through changes: `speclink propose <idea>`.

## Guardrails

- **Don't invent behavior** — evidence-based only; unverified inferences are flagged or omitted.
- **Don't refactor while onboarding** — no code changes at all.
- **Don't rewrite existing specs** — gap-fill only; modifications go through a change.
- **Do confirm the capability map before writing** — boundaries are the expensive decision.
- **Do keep specs small** — a capability that needs 15 requirements is probably two capabilities.

=== .wad/skills/speclink-propose/SKILL.md ===
---
name: speclink-propose
description: "Create a change proposal with all required artifacts"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "1.0"
  generatedBy: "Speclink"
---

## Invocation

This harness executes speclink verbs by calling the speclink tool with an argv array (e.g. ["apply", "add-auth"]). Wherever this document says `speclink <verb> [arguments]`, it means calling the speclink tool with those arguments as argv.

---

Create a complete Speclink change proposal — from requirement to validated artifacts — in a single workflow.

**Input**: The argument after `speclink propose` is the requirement description. Examples:

- `speclink propose add dark mode`
- `speclink propose fix the login page crash`
- `speclink propose improve search performance`

If no argument is provided, the workflow will extract requirements from conversation context or ask.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Determine the requirement source**

   Priority order: explicit argument → discussion document → plan file → conversation context.

   a. **Argument provided** (e.g., "add dark mode") → use it as the requirement description, skip to deriving the change name below. A `--from-discussion <slug>` argument explicitly selects the discussion document in (b).

   b. **Discussion document available** (speclink enhancement):
   - List recorded discussions:
     ```bash
     speclink discuss list --json
     ```
   - If the user passed `--from-discussion <slug>`, or a discussion with `status: concluded` exists that matches the topic, read its full record:
     ```bash
     speclink discuss show <slug> --json
     ```
   - If a candidate discussion exists, use the **AskUserQuestion tool** to confirm using it (Option 1: this discussion, Option 2: another source). When confirmed, extract from the discussion document (`discussions/<slug>.md`):
     - `## Conclusion` → **Decision** becomes the requirement description; **Rationale** becomes the proposal Why; **Capture to** routing guides which artifacts to emphasize.
     - The `## Context` section and the accumulated `### Round N` entries → context for What Changes, Capabilities, and Impact (the rounds' **Ruled out** lines name the alternatives already eliminated — don't re-propose them).
   - If no discussion exists or the user declines → fall through to (c).

   c. **Plan file available**:
   - If found, check if the file exists at ``
   - If a plan file is found, use the **AskUserQuestion tool** to ask:
     - Option 1: Use the plan file
     - Option 2: Use conversation context
   - If conversation context has no relevant discussion, mention this when presenting the choice
   - If the user picks the plan file → read it and extract:
     - `plan_title` (H1 heading) → use as requirement description
     - `plan_context` (Context section) → use as proposal Why/Motivation content
     - `plan_stages` (numbered implementation stages) → use for artifact creation
     - `plan_files` (all file paths mentioned) → use for Impact section
   - If the user picks conversation context → fall through to (d)

   d. **Conversation context** → attempt to extract requirements from conversation history
   - If context is insufficient, use the **AskUserQuestion tool** to ask what they want to build

   From the resolved description, derive a kebab-case change name (e.g., "add dark mode" → `add-dark-mode`).
   Do not keep archive-style date prefixes in active change names. If the source name starts with `YYYY-MM-DD-`, strip that date prefix before running `speclink new change`; archived change names and directories are historical references, not active names to reuse.

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

2. **Classify the change type**

   Based on the requirement, classify the change into one of three types:

   | Type     | When to use                                                         |
   | -------- | ------------------------------------------------------------------- |
   | Feature  | New functionality, new capabilities                                 |
   | Bug Fix  | Fixing existing behavior, resolving errors                          |
   | Refactor | Architecture improvements, performance optimization, UI adjustments |

   This determines the proposal template format in step 5.

3. **Scan existing specs for relevance**

   Before creating the change, check if any existing specs overlap:
   1. Run `speclink list --specs --json` to get the spec identifier list
   2. Compare against the user's description to identify related specs (max 5 candidates)
   3. For each candidate (max 3), run `speclink show <spec-id>` and read the Purpose section at the top of the output
   4. If related specs are found, display them as an informational summary

   **IMPORTANT**:
   - If related specs are found, display them but do NOT stop or ask for confirmation — continue to the next step
   - If no related specs are found, silently proceed without mentioning the scan

4. **Create the change directory**

   ```bash
   speclink new change "<name>" --agent wad-harness
   ```

   When the proposal is sourced from a discussion document (path (b) in step 1), pass the link so the change records its origin and the discussion is marked `promoted` (it will be archived together with the change later):

   ```bash
   speclink new change "<name>" --agent wad-harness --from-discussion <slug>
   ```

   If a change with that name already exists, suggest continuing the existing change instead of creating a new one.

5. **Write the proposal**

   **IMPORTANT — file path rules for the `## Impact` section:**
   - All file paths SHALL be written relative to the project root (e.g., `src/lib/foo.ts`, `src-tauri/crates/core/src/bar.rs`, `docs/specs/specs/auth/spec.md`).
   - Do NOT use relative fragments (e.g., `parser/mod.rs`, `core/mod.rs`) — preflight rejects them as non-anchored paths.
   - Do NOT wrap shell commands in backticks inside artifact text (e.g., `` `git mv a.rs b.rs` ``) — preflight's backtick extractor will otherwise mis-parse the command as a file reference.
   - When referring to a file without naming its concrete path, use descriptive prose (e.g., "Parser 入口檔") rather than a backticked path fragment.

   Get instructions:

   ```bash
   speclink instructions proposal --change "<name>" --json
   ```

   Generate the proposal content based on change type (see formats below), then write it via CLI:

   ```bash
   speclink new artifact proposal --change "<name>" --stdin <<'ARTIFACT_EOF'
   <proposal content>
   ARTIFACT_EOF
   ```

   If the command fails with a validation error, fix the content and retry.

   Use the following format based on change type:

   ### Feature

   ```markdown
   ## Why

   <!-- Why this functionality is needed -->

   ## What Changes

   <!-- What will be different -->

   ## Non-Goals (optional)

   <!-- Scope exclusions and rejected approaches. Required when design.md is skipped. -->

   ## Capabilities

   ### New Capabilities

   - `<capability-name>`: <brief description>

   ### Modified Capabilities

   (none)

   ## Impact

   - Affected specs: <new or modified capabilities>
   - Affected code:
     - New: <paths to be created, relative to project root>
     - Modified: <paths that already exist>
     - Removed: <paths to be deleted>
   ```

   ### Bug Fix

   ```markdown
   ## Problem

   <!-- Current broken behavior -->

   ## Root Cause

   <!-- Why it happens -->

   ## Proposed Solution

   <!-- How to fix -->

   ## Non-Goals (optional)

   <!-- Scope exclusions and rejected approaches. Required when design.md is skipped. -->

   ## Success Criteria

   <!-- Expected behavior after fix, verifiable conditions -->

   ## Impact

   - Affected code:
     - Modified: <paths that already exist>
     - New: <paths to be created, relative to project root>
     - Removed: <paths to be deleted>
   ```

   ### Refactor / Enhancement

   ```markdown
   ## Summary

   <!-- One sentence description -->

   ## Motivation

   <!-- Why this is needed -->

   ## Proposed Solution

   <!-- How to do it -->

   ## Non-Goals (optional)

   <!-- Scope exclusions and rejected approaches. Required when design.md is skipped. -->

   ## Alternatives Considered (optional)

   <!-- Other approaches considered and why not -->

   ## Impact

   - Affected specs: <affected capabilities>
   - Affected code:
     - Modified: <paths that already exist>
     - New: <paths to be created, relative to project root>
     - Removed: <paths to be deleted>
   ```

6. **Get the artifact build order**

   ```bash
   speclink status --change "<name>" --json
   ```

   Parse the JSON to get:
   - `applyRequires`: array of artifact IDs needed before implementation
   - `artifacts`: list of all artifacts with their status and dependencies

7. **Create remaining artifacts in sequence**

   Loop through artifacts in dependency order (skip proposal since it's already done):

   a. **For each artifact that is `ready` (dependencies satisfied)**:
   - **Check if the artifact is optional**: If the artifact is NOT in the dependency chain of any `applyRequires` artifact (i.e., removing it would not block reaching apply), it is optional. Get its instructions and read the `instruction` field. If the instruction contains conditional criteria (e.g., "create only if any apply"), evaluate whether any criteria apply to this change based on the proposal content. If none apply, skip the artifact and show: "⊘ Skipped <artifact-id> (not needed for this change)". Then continue to the next artifact.
   - Get instructions:
     ```bash
     speclink instructions <artifact-id> --change "<name>" --json
     ```
   - The instructions JSON includes:
     - `context`: Project background (constraints for you - do NOT include in output)
     - `rules`: Artifact-specific rules (constraints for you - do NOT include in output)
     - `template`: The structure to use for your output file
     - `instruction`: Schema-specific guidance
     - `outputPath`: Where to write the artifact
     - `dependencies`: Completed artifacts to read for context
     - `locale`: The language to write the artifact in (e.g., "Japanese (日本語)"). If present, you MUST write the artifact content in this language. Spec files (specs/\*_/_.md) default to English instead — unless the project sets `spec_locale` in `.speclink.yaml` or `openspec/config.yaml` (a locale code, or `auto` to follow `locale`), in which case write spec prose in that language. Structural markers (`### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST) always stay in English.
   - Read each completed dependency for context via `speclink artifact cat <artifact-id> --change "<name>"` (never open artifact files by path — the documents may live in a remote store)
   - Generate the artifact content using `template` as the structure
   - Apply `context` and `rules` as constraints - but do NOT copy them into the file
   - Write the artifact via CLI (the CLI handles directory creation and format validation):

     For **design** or **tasks**:

     ```bash
     speclink new artifact <artifact-id> --change "<name>" --stdin <<'ARTIFACT_EOF'
     <content>
     ARTIFACT_EOF
     ```

     For **specs** (one command per capability):

     ```bash
     speclink new artifact spec <capability-name> --change "<name>" --stdin <<'ARTIFACT_EOF'
     <delta spec content>
     ARTIFACT_EOF
     ```

     If the command fails with a validation error, fix the content and retry.

   - Show brief progress: "✓ Created <artifact-id>"

   b. **Continue until all `applyRequires` artifacts are complete**
   - After creating each artifact, re-run `speclink status --change "<name>" --json`
   - Check if every artifact ID in `applyRequires` has `status: "done"`
   - Stop when all `applyRequires` artifacts are done

   c. **If an artifact requires user input** (unclear context):
   - Use **AskUserQuestion tool** to clarify
   - Then continue with creation

8. **Inline Self-Review** (before CLI analysis)

   After creating all artifacts, scan them manually. Fix issues inline, then proceed to the CLI analyzer.

   **Check 1: No Placeholders**

   These patterns are artifact failures — fix each one before proceeding:
   - "TBD", "TODO", "FIXME", "implement later", "details to follow"
   - Vague instructions: "Add appropriate error handling", "Handle edge cases", "Write tests for the above"
   - Delegation by reference: "Similar to Task N" without repeating specifics
   - Steps describing WHAT without HOW: "Implement the authentication flow" (what flow? what steps?)
   - Empty template sections left unfilled
   - Weasel quantities: "some", "various", "several" when a specific number or list is needed

   **Check 2: Internal Consistency**
   - Does every capability in the proposal have a corresponding spec?
   - Does the design reference only capabilities from the proposal?
   - Do tasks cover all design decisions, and nothing outside proposal scope?
   - Are file paths consistent across proposal Impact, design, and tasks?

   **Check 3: Scope Check**
   - More than 15 pending tasks → consider decomposing into multiple changes
   - Any single task would take more than 1 hour → split it
   - Touches more than 3 unrelated subsystems → consider splitting

   **Check 4: Ambiguity Check**
   - Are success/failure conditions testable and specific?
   - Are boundary conditions defined (empty input, max limits, error cases)?
   - Could "the system" refer to multiple components? Be explicit.

   **Check 5: Durable Handoff Review** (run BEFORE the CLI analyzer)

   This change has to survive being handed to another agent. Reject and fix any of the following:
   - **File-path-only tasks**: a task whose entire description is "edit file X" with no behavior, contract, or verification target. File paths are locator context — the task SHALL still describe what is observably true when complete.
   - **Line-number-coupled instructions**: design or tasks content that points to "line 42" / "the function on lines 80-95" as the only way to identify the work. Source line numbers drift; name the function, command, struct, or behavior instead.
   - **Vague acceptance criteria**: success conditions like "works correctly", "behaves as expected", "handles edge cases" without naming the observable behavior or the verification target (test name, CLI invocation, analyzer rule, manual assertion).
   - **Missing scope boundaries on non-trivial work**: design lacking explicit "in scope" / "out of scope" lines for any change that touches more than one subsystem or introduces new behavior. Trivial artifact-only edits MAY skip this; runtime, build, or tooling effects MUST NOT.

   Fix every failure inline using the existing context before running the CLI analyzer. If a failure cannot be fixed without new input from the user, surface it explicitly rather than papering over it.

---

## Rationalization Table

| What You're Thinking                                          | What You Should Do                                                                    |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| "The requirements are clear enough, no need for discuss"      | Fine if true — but check you're not skipping because you're lazy                      |
| "This artifact isn't needed for this change"                  | Check `applyRequires` — if it's in the dependency chain, create it                    |
| "The spec doesn't need scenarios, the requirement is obvious" | Obvious to you now. Write scenarios for the implementer who doesn't have your context |
| "I'll keep the design brief, code will be self-explanatory"   | Design exists so implementers don't reverse-engineer intent. Be specific              |
| "This is a small change, skip the scope check"                | Small changes touching 5 subsystems aren't small. Check                               |
| "The placeholder is fine for now, I'll fill it in later"      | There is no "later" — implementation is next. Fill it in now                          |

---

9. **Analyze-Fix Loop** (max 2 iterations)
   1. Run `speclink analyze <change-name> --json`
   2. Filter findings to **Critical and Warning only** (ignore Suggestion)
   3. If no Critical/Warning findings → show "Artifacts look consistent ✓" and proceed
   4. If Critical/Warning findings exist:
      a. Show: "Found N issue(s), fixing... (attempt M/2)"
      b. Fix each finding in the affected artifact
      c. Re-run `speclink analyze <change-name> --json`
      d. Repeat up to 2 total iterations
   5. After 2 attempts, if findings remain:
      - Show remaining findings as a summary
      - Proceed normally (do NOT block)

10. **Validation**

    ```bash
    speclink validate "<name>"
    ```

    If validation fails, fix errors and re-validate.

11. **End the workflow**

    Show summary:
    - Change name and location
    - List of artifacts created
    - Validation result

    Inform the user that the change is ready and that running `speclink apply <change-name>` when ready will start implementation.


    The propose workflow ENDS here. Do NOT invoke `speclink apply`. Do NOT call **AskUserQuestion** to ask whether to apply. This behavior is identical across Auto Mode, interactive mode, and any other agent mode.

**Artifact Creation Guidelines**

- Follow the `instruction` field from `speclink instructions` for each artifact type
- Read dependency artifacts for context before creating new ones
- Use `template` as the structure for your output file - fill in its sections
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact
  - These guide what you write, but should never appear in the output

**Guardrails**

- Create all artifacts needed for implementation. Optional artifacts (those not in the `applyRequires` dependency chain) may be skipped if their inclusion criteria don't apply.
- Always read dependency artifacts before creating a new one
- If context is critically unclear, ask the user - but prefer making reasonable decisions to keep momentum
- If a change with that name already exists, suggest continuing that change instead
- Verify each artifact file exists after writing before proceeding to next
- **NEVER** write application code or implement features during this workflow
- **NEVER** skip the artifact workflow to write code directly
- **NEVER** reinterpret requirements by ignoring the proposal file
- **NEVER** invoke `speclink apply` — this workflow ends after artifact creation. The user decides when to start implementation
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response
