# speclink-archive Workflow

Archive a completed change. Apply pending spec deltas to canonical specs, move the change to archived state, and optionally orchestrate a git commit for the implementation.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` (CLI subprocess hosts) or `bindings/tool.md` (typed Tool hosts). All operations are referenced by their **canonical id** (see `doc/protocol/operations.md`).

---

## Input

Optionally specify a change name after `/speclink-archive` (e.g., `/speclink-archive add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous, you MUST prompt for available changes.

## Prerequisites

This skill requires a working SpecLink invocation surface. If any operation fails with `cli.not_found` / `sdk.not_initialized` or similar, report the error and STOP. Also verify the project is initialized — invoke `project.status`. If it reports `project.not_initialized`, ask the user to run `project.init` first.

---

## Steps

### Step 1: If no change name provided, prompt for selection

Invoke `change.list` with `{ include_archived: false }`.

Ask the user to choose. Show only active (non-archived) changes, with their current state and schema annotated.

**IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

### Step 2: Check state and code-review status

Archive is only valid from specific states. Invoke `change.show` with `{ change_id }`.

Parse the result for `state`, `schema_id`, and `review_status`.

**State handling**:

- `state: "code_reviewing"` → **code review is required**. Check `review_status.code_approved`:
  - If approved → continue.
  - If not approved → STOP. Tell the user the change is awaiting code review approval. The reviewer should invoke `review.approve` with `{ change_id, reviewer, phase: "code" }`. Then re-run `/speclink-archive <name>`.
  - **If the reviewer rejects** via `review.reject` with `{ phase: "code", reason }`, engine auto-retreats state to `in_progress` AND auto-appends a synthetic task to tasks.md (format: `- [ ] [Review feedback by <reviewer>, YYYY-MM-DD] <reason>`). In that case, do NOT run `/speclink-archive` — run `/speclink-apply` to address the feedback task(s). When all tasks (including synthetic ones) are `[x]` again, engine auto re-enters `code_reviewing` and `/speclink-archive` becomes valid again.
- `state: "in_progress"` AND all tasks are `[x]` (engine signals `tasks.done == tasks.total`) AND `require_code_review: false` → continue (code review is disabled in config; ready to archive directly).
- `state: "in_progress"` AND not all tasks done → STOP. Tell the user to finish remaining tasks first (`/speclink-apply <name>`).
- `state: "ready"` → STOP. Tell the user the change hasn't started implementation; either pick a different change or run `/speclink-apply <name>` first.
- `state: "proposing"` / `"reviewing"` → STOP. The change isn't ready for apply, much less archive.
- `state: "archived"` → STOP. The change is already archived.

### Step 3: Check artifact completion status

The `change.show` result already includes `artifacts[]` with each artifact's status.

**If any required artifacts are not `done`** (engine flags this via `apply_requires_missing[]`):

- Display warning listing incomplete artifacts.
- Ask the user to confirm continuing.
- Proceed if the user confirms (engine will warn again during `archive.run` if `no_validate` is not set).

### Step 4: Check task completion status

Read the tasks file via `artifact.read` with `{ change_id, kind: "tasks" }` and count tasks marked `- [ ]` (incomplete) vs `- [x]` (complete).

**If incomplete tasks found**:

- Display warning showing count of incomplete tasks.
- Offer the user three options:
  - **Cancel archive** — return to apply
  - **Mark complete via archive flag** — let `archive.run` with `mark_tasks_complete: true` flip remaining `- [ ]` to `- [x]` before archiving (audit-logged)
  - **Archive as-is** — leaves `- [ ]` rows in archived tasks.md (recorded but not retroactively completed)

**If no tasks file exists**: proceed without task-related warning.

### Step 5: Assess delta spec sync state

Check for delta specs in the change (artifacts of kind `spec`, capability-scoped). If none exist, proceed without sync prompt.

**If delta specs exist**:

- For each delta spec capability, invoke `spec.show` with `{ capability }` to read the current canonical spec.
- Invoke `artifact.read` with `{ change_id, kind: "spec", capability }` to read the delta.
- Determine what changes would be applied (ADDED / MODIFIED / REMOVED / RENAMED requirements).
- Show a combined summary before prompting.

**Prompt options**:

- If changes needed: "Sync as part of archive (recommended)" / "Archive without syncing (`skip_specs: true`)"
- If already synced: "Archive now" / "Re-sync anyway" / "Cancel"

**Important — speclink does NOT have a separate sync sub-skill**. `archive.run` applies delta specs automatically as part of its workflow (unless `skip_specs: true` is passed).

### Step 6: Perform the archive

Invoke `archive.run` with `{ change_id, skip_specs?, no_validate?, mark_tasks_complete?, yes? }`.

**Optional flags**:

- `skip_specs: true` — skip delta spec application (for tooling / doc-only changes). Engine logs `archive.specs_skipped` audit event.
- `mark_tasks_complete: true` — flip remaining `- [ ]` to `- [x]` before archiving (audit-logged).
- `no_validate: true` — skip delta spec validation. Use only when you've already manually verified.
- `yes: true` — bypass interactive confirmation for protective flags.

**Default behavior** (no protective flags):

- Engine validates delta specs (equivalent to `validate.run` with `strict: true`).
- Engine applies delta specs to canonical specs (`.speclink/specs/` for LocalProvider).
- Engine moves change to archived state, records `archived_at`.
- Engine flips state to `archived` (terminal).
- Engine updates `linked_changes[]` in any linked discussion documents (metadata frontmatter only; document body untouched).

**If archive fails**:

- `archive.target_exists` (date+name collision) → suggest renaming existing archive or waiting a day.
- `validation.archive_failed` → fix delta specs and retry; or pass `no_validate: true` if intentional.
- `state.transition_invalid` → re-invoke `change.show` to inspect state.
- `lock.not_acquired` → engine handles jittered backoff retry.

### Step 7: Commit sub-flow (optional)

After archive succeeds, ask the user whether to commit the implementation work to git:

```
Archive complete. Would you like to commit the changes to git?
1. Yes — generate a commit message and commit the touched files
2. Skip — handle git manually
```

If the user chooses to commit:

**7a. Fetch commit instructions**

Invoke `instructions.get` with `{ kind: "commit", change_id }`. The result includes:

- `messageTemplate`: suggested commit message structure
- `touchedFiles[]`: files the implementation modified
- `completedTasks[]`: list of `{ id, desc }` for tasks done in this change
- `rules`: any user-defined commit message rules from config
- `context`: project context (e.g., conventional commits format if applicable)

**7b. Compose commit message** using the template + completed tasks + change summary. Apply rules (e.g., conventional commits format).

**7c. Verify touched files exist and are tracked**

Use your AI tool's git facility to run `git status --short`. Cross-check with `touchedFiles[]`. If files reported as touched are no longer modified (e.g., user already committed manually), skip those.

**7d. Stage and commit**

Use your AI tool's git facility to run:

```
git add <touched-file-1> <touched-file-2> ...
git commit -m "<composed message>"
```

Show the commit hash in the summary.

**If commit fails** (pre-commit hooks, merge conflicts, etc.):

- Report the git error directly.
- The archive is already complete; only the git commit is incomplete.
- User can re-attempt manually.

**If the user chooses Skip**:

- Do not run any git command.
- Mention that touched files are recorded in the audit log for reference.

### Step 8: Display summary

Show archive completion summary:

- Change name
- Schema that was used
- Archive location (`.speclink/archive/<YYYY-MM-DD>-<name>/` for LocalProvider)
- Spec sync status (synced / sync skipped / no delta specs)
- Linked discussion ids (if any) — preserved as `converged`
- Code review status (approved by `<reviewer>` if applicable)
- Commit status (hash if committed, "skipped" if not)
- Note about any warnings (incomplete artifacts/tasks, protective flags used)

---

## Output On Success (Full)

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived (terminal)
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** ✓ Synced to canonical specs (3 added, 1 modified)
**Code review:** ✓ Approved by <reviewer-id>
**Linked discussions:** <topic-1>, <topic-2> (preserved as converged)
**Commit:** abc123 — "feat(orders): add order export endpoint"

All artifacts complete. All tasks complete.
```

## Output On Success (No Delta Specs)

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** No delta specs
**Commit:** skipped

All artifacts complete. All tasks complete.
```

## Output On Success With Warnings

```
## Archive Complete (with warnings)

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** Sync skipped (user chose skip_specs)
**Commit:** skipped

**Warnings:**
- Archived with 2 incomplete artifacts
- Archived with 3 incomplete tasks (marked complete via mark_tasks_complete)
- Delta spec sync was skipped (user chose skip_specs)

Review the archive if this was not intentional.
```

## Output On Error (Code Review Required)

```
## Archive Blocked

**Change:** <change-name>
**State:** code_reviewing
**Code review:** ✗ Not yet approved

The project requires code review before archive (`require_code_review: true`).

Reviewer invokes `review.approve` with phase=code.

Then re-run `/speclink-archive <change-name>`.
```

---

## Concurrency & Errors

- `lock.not_acquired` → engine handles jittered backoff retry.
- `change.not_found` → suggest `change.list`.
- `state.transition_invalid` → state is not `code_reviewing` or `in_progress` (with all tasks done); follow the state-handling table in Step 2.
- `change.code_review_pending` → see Output On Error above.
- `archive.target_exists` (date+name collision) → suggest different timing or rename existing archive.
- `validation.archive_failed` → engine validate found delta spec errors; fix and retry, or use `no_validate: true` if intentional.
- `config.malformed` → surface warnings; never auto-modify config.
- `project.not_initialized` → stop; ask user to invoke `project.init`.

---

## Guardrails

- Always prompt for change selection if not provided.
- Use `change.show` (which includes `artifacts[]`) for completion checking.
- Don't block archive on warnings — inform and let the user confirm.
- Engine handles spec delta merge inline; **never** invoke a separate sync sub-skill (it does not exist).
- **Never** modify or delete linked discussions; engine only updates their `linked_changes[]` metadata.
- **Never** auto-approve code review — the skill REQUIRES a separate `review.approve` invocation by the reviewer.
- Commit sub-flow is **opt-in** — never auto-commit without user confirmation.
- **NEVER** pass `force: true` flags casually. `archive.run` does not have a `force` flag (it has `skip_specs` / `no_validate` / `mark_tasks_complete` / `yes` instead); all of these are intentional escape hatches and require explicit user choice.
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.

---

## Fluid Workflow Integration

This skill is **terminal in the change lifecycle** — after a successful archive, the change is in state `archived` and cannot return to active development without explicit unarchive (a future feature, not in MVP).

- **Can be invoked anytime** the state is `code_reviewing` (with code-review approved) or `in_progress` (with all tasks done and `require_code_review: false`).
- **Other states are rejected** with guidance pointing to the correct skill.
- **Commit sub-flow is a convenience** — archiving and committing are decoupled. Users who use a different VCS workflow can skip the commit step entirely without losing audit data (touched files are recorded in the audit log).
- **Linked discussions persist** — discussions are independent entities; archiving a change does not delete or modify any discussion. Invoke `discuss.delete` with `{ force: true }` explicitly if a discussion is no longer wanted.
