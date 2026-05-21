---
name: speclink-archive
description: "Archive a completed change with optional commit sub-flow"
effort: low
license: MIT
compatibility: Requires speclink CLI.
speclink_version: 0.1.0
template_hash: sha256:<generated-at-build>
metadata:
  author: speclink
  version: "1.0"
  generatedBy: SpecLink
---

Archive a completed change. Apply pending spec deltas to main specs, move the change directory to `.speclink/archive/`, and optionally orchestrate a git commit for the implementation.

**Input**: Optionally specify a change name after `/speclink-archive` (e.g., `/speclink-archive add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP. Also verify the project is initialized — if `speclink status` reports `project.not_initialized`, ask the user to run `speclink init <project-name>` first.

**Steps**

1. **If no change name provided, prompt for selection**

   Run:

   ```bash
   speclink list --changes --json
   ```

   Ask the user to choose. Show only active (non-archived) changes, with their current state and schema annotated.

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check state and code-review status**

   Archive is only valid from specific states. Run:

   ```bash
   speclink status --change "<name>" --json
   ```

   Parse the JSON for `state`, `schemaName`, `parked`, and `reviewApprovals`.

   **State handling**:

   - `state: "code_reviewing"` → **code review is required**. Check `reviewApprovals.code` in the JSON:
     - If approved → continue.
     - If not approved → STOP. Tell the user the change is awaiting code review approval:
       ```bash
       speclink review approve --change <name> --reviewer <id> --phase code
       ```
       Then re-run `/speclink-archive <name>`.
     - **If the reviewer rejects** (`speclink review reject --change <name> --reviewer <id> --phase code --reason "..."`), engine auto-retreats state to `in_progress` AND auto-appends a synthetic task to `tasks.md`:
       ```
       - [ ] [Review feedback by <reviewer>, YYYY-MM-DD] <reason>
       ```
       In that case, do NOT run `/speclink-archive` — run `/speclink-apply` to address the feedback task(s). When all tasks (including synthetic ones) are `[x]` again, engine auto re-enters `code_reviewing` and `/speclink-archive` becomes valid again.
   - `state: "in_progress"` AND all tasks are `[x]` (engine signals `apply_complete: true`) AND `require_code_review: false` → continue (code review is disabled in config; ready to archive directly).
   - `state: "in_progress"` AND not all tasks done → STOP. Tell the user to finish remaining tasks first (`/speclink-apply <name>`).
   - `state: "ready"` → STOP. Tell the user the change hasn't started implementation; either pick a different change or run `/speclink-apply <name>` first.
   - `state: "proposing"` / `"reviewing"` → STOP. The change isn't ready for apply, much less archive.
   - `state: "archived"` → STOP. The change is already archived.

   **Parked handling**:

   - If `parked: true`, archiving is unusual but allowed. Inform the user the change is currently parked and ask whether to:
     - Unpark first then archive (cleaner audit trail)
     - Archive directly (will remain in the parked flag's last-known state at archive time)

3. **Check artifact completion status**

   The status JSON already includes `artifacts[]` with each artifact's status.

   **If any required artifacts are not `done`** (engine flags this in `applyRequiresMissing[]`):

   - Display warning listing incomplete artifacts.
   - Ask the user to confirm continuing.
   - Proceed if the user confirms (engine will warn again during `speclink archive` if `--no-validate` is not set).

4. **Check task completion status**

   Read the tasks file (`.speclink/changes/<name>/tasks.md`) and count tasks marked `- [ ]` (incomplete) vs `- [x]` (complete).

   **If incomplete tasks found**:

   - Display warning showing count of incomplete tasks.
   - Offer the user three options:
     - Cancel archive — return to apply
     - Use `--mark-tasks-complete` — let the archive CLI flip remaining `- [ ]` to `- [x]` before archiving (audit-logged)
     - Archive as-is (leaves `- [ ]` rows in archived tasks.md — recorded but not retroactively completed)

   **If no tasks file exists**: proceed without task-related warning.

5. **Assess delta spec sync state**

   Check for delta specs at `.speclink/changes/<name>/specs/`. If none exist, proceed without sync prompt.

   **If delta specs exist**:

   - Compare each delta spec with its corresponding main spec at `.speclink/specs/<capability>/spec.md`.
   - Determine what changes would be applied (ADDED / MODIFIED / REMOVED / RENAMED requirements).
   - Show a combined summary before prompting.

   **Prompt options**:

   - If changes needed: "Sync as part of archive (recommended)" / "Archive without syncing (`--skip-specs`)"
   - If already synced: "Archive now" / "Re-sync anyway" / "Cancel"

   **Important — speclink does NOT have a separate sync sub-skill**. The `speclink archive` CLI applies delta specs automatically as part of its workflow (unless `--skip-specs` is passed). No external `speclink-sync-specs` subagent invocation needed.

6. **Clean up tracking file**

   Delete `.speclink/touched/<change-name>.json` if it exists — implementation tracking data not needed after archive.

   ```bash
   rm -f .speclink/touched/<change-name>.json
   ```

   If the file does not exist, silently continue.

   **Important**: Do NOT touch linked discussion files. Discussions are independent entities (see design doc §6.1) and are preserved as audit trails. Engine will record the archive event in each linked discussion's `linked_changes[]` metadata automatically; the discussion document itself is not modified.

7. **Perform the archive**

   Use the `speclink archive` CLI which handles the full archive workflow (spec delta application, snapshot creation, state transition to `archived`):

   ```bash
   speclink archive "<name>" --json
   ```

   **Optional flags**:

   - `--skip-specs` — skip delta spec application (for tooling / doc-only changes). Engine logs this in the archive audit.
   - `--mark-tasks-complete` — flip remaining `- [ ]` to `- [x]` before archiving (audit-logged).
   - `--no-validate` — skip delta spec validation. Use only when you've already manually verified.
   - `--yes` — bypass interactive confirmation for protective flags (`--skip-specs`, `--no-validate`).

   **Default behavior** (no protective flags):

   - Engine validates delta specs (`speclink validate <name> --strict` equivalent).
   - Engine applies delta specs to `.speclink/specs/`.
   - Engine creates snapshot for unarchive support (under `.speclink/archive/<YYYY-MM-DD>-<name>/.snapshot/`).
   - Engine moves change directory to `.speclink/archive/<YYYY-MM-DD>-<name>/`.
   - Engine flips state to `archived` (terminal).
   - Engine updates `linked_changes[]` in any linked discussion documents (metadata frontmatter only; document body untouched).

   **If archive fails**:

   - `archive.target_exists` (date+name collision) → suggest renaming existing archive or waiting a day.
   - `archive.validation_failed` → fix delta specs and retry; or pass `--no-validate` if intentional.
   - `state.transition_invalid` → re-check state via `speclink status`.
   - `change.locked` → retry 1-2 sec × max 3.

8. **Commit sub-flow (optional)**

   After archive succeeds, ask the user whether to commit the implementation work to git:

   ```
   Archive complete. Would you like to commit the changes to git?
   1. Yes — generate a commit message and commit the touched files
   2. Skip — handle git manually
   ```

   If the user chooses to commit:

   a. **Fetch commit instructions**:

   ```bash
   speclink instructions commit --change "<name>" --json
   ```

   Engine returns:

   - `messageTemplate`: suggested commit message structure
   - `touchedFiles[]`: files the implementation modified (from `.speclink/touched/<name>.json` before cleanup OR derived from state.db)
   - `completedTasks[]`: list of `{ id, desc }` for tasks done in this change
   - `rules`: any user-defined commit message rules from config
   - `context`: project context (e.g., conventional commits format if applicable)

   b. **Compose commit message** using the template + completed tasks + change summary. Apply rules (e.g., conventional commits format).

   c. **Verify touched files exist and are tracked**:

   ```bash
   git status --short
   ```

   Cross-check with `touchedFiles[]`. If files reported as touched are no longer modified (e.g., user already committed manually), skip those.

   d. **Stage and commit**:

   ```bash
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
   - Mention that touched files are listed in `.speclink/audit/<name>.log` for reference.

9. **Display summary**

   Show archive completion summary:

   - Change name
   - Schema that was used
   - Archive location (`.speclink/archive/<YYYY-MM-DD>-<name>/`)
   - Spec sync status (synced / sync skipped / no delta specs)
   - Linked discussion ids (if any) — preserved as `converged`
   - Code review status (approved by `<reviewer>` if applicable)
   - Commit status (hash if committed, "skipped" if not)
   - Note about any warnings (incomplete artifacts/tasks, protective flags used)

---

**Output On Success (Full)**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived (terminal)
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** ✓ Synced to main specs (3 added, 1 modified)
**Code review:** ✓ Approved by <reviewer-id>
**Linked discussions:** <topic-1>, <topic-2> (preserved as converged)
**Commit:** abc123 — "feat(orders): add order export endpoint"

All artifacts complete. All tasks complete.
```

**Output On Success (No Delta Specs)**

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

**Output On Success (Commit Skipped)**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** ✓ Synced
**Commit:** skipped (you can commit manually; touched files in .speclink/audit/<name>.log)
```

**Output On Success With Warnings**

```
## Archive Complete (with warnings)

**Change:** <change-name>
**Schema:** <schema-name>
**State:** archived
**Archived to:** .speclink/archive/YYYY-MM-DD-<name>/
**Specs:** Sync skipped (user chose --skip-specs)
**Commit:** skipped

**Warnings:**
- Archived with 2 incomplete artifacts
- Archived with 3 incomplete tasks (marked complete via --mark-tasks-complete)
- Delta spec sync was skipped (user chose --skip-specs)

Review the archive if this was not intentional.
```

**Output On Error (Target Exists)**

```
## Archive Failed

**Change:** <change-name>
**Target:** .speclink/archive/YYYY-MM-DD-<name>/

Target archive directory already exists.

**Options:**
1. Rename the existing archive directory
2. Delete the existing archive if it's a duplicate
3. Wait until a different date to archive
```

**Output On Error (Code Review Required)**

```
## Archive Blocked

**Change:** <change-name>
**State:** code_reviewing
**Code review:** ✗ Not yet approved

The project requires code review before archive (`require_code_review: true`).

Run as the reviewer:
  speclink review approve --change <change-name> --reviewer <your-id> --phase code

Then re-run `/speclink-archive <change-name>`.
```

---

## Concurrency & Errors

- `change.locked` → retry 1-2 sec × max 3.
- `change.not_found` → suggest `speclink list --changes --json`.
- `state.transition_invalid` → state is not `code_reviewing` or `in_progress` (with all tasks done); follow the state-handling table in Step 2.
- `review.unauthorized` (when phase=code is required but missing) → see "Output On Error (Code Review Required)" above.
- `archive.target_exists` → see "Output On Error (Target Exists)" above.
- `archive.validation_failed` → engine validate found delta spec errors; fix and retry, or use `--no-validate` if intentional.
- `config.malformed` → surface warnings; never auto-modify config.
- `project.not_initialized` → stop; ask user to run `speclink init <project-name>`.

## Guardrails

- Always prompt for change selection if not provided.
- Use artifact graph (`speclink status --json`) for completion checking.
- Don't block archive on warnings — inform and let the user confirm.
- Preserve `metadata.json` when moving to archive (engine handles this — directory move is atomic).
- Show clear summary of what happened, including commit hash if committed.
- Engine handles spec delta merge inline; **never** invoke a separate `speclink-sync-specs` sub-skill (it does not exist).
- **Never** modify or delete linked discussions; engine only updates their `linked_changes[]` metadata.
- **Never** auto-approve code review — the skill REQUIRES a separate `speclink review approve --phase code` invocation by the reviewer.
- Commit sub-flow is **opt-in** — never auto-commit without user confirmation.
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.

## Fluid Workflow Integration

This skill is **terminal in the change lifecycle** — after a successful archive, the change is in state `archived` and cannot return to active development without explicit unarchive (a future feature, not in MVP).

- **Can be invoked anytime** the state is `code_reviewing` (with code-review approved) or `in_progress` (with all tasks done and `require_code_review: false`).
- **Other states are rejected** with guidance pointing to the correct skill.
- **Commit sub-flow is a convenience** — archiving and committing are decoupled. Users who use a different VCS workflow can skip the commit step entirely without losing audit data (`touched files` are logged in `.speclink/audit/<name>.log`).
- **Linked discussions persist** — discussions are independent entities; archiving a change does not delete or modify any discussion. Use `speclink discuss delete <id>` explicitly if a discussion is no longer wanted.
