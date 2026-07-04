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

Archive a completed change.

**Input**: Optionally specify a change name after `$speclink-archive` (e.g., `$speclink-archive add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

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

4. **Assess delta spec sync state**

   Check for delta specs at `openspec/changes/<name>/specs/`. If none exist, proceed without sync prompt.

   **If delta specs exist:**
   - Compare each delta spec with its corresponding main spec at `openspec/specs/<capability>/spec.md`
   - Determine what changes would be applied (adds, modifications, removals, renames)
   - Show a combined summary before prompting

   **Prompt options:**
   - If changes needed: "Sync now (recommended)", "Archive without syncing"
   - If already synced: "Archive now", "Sync anyway", "Cancel"

   If user chooses sync, use Task tool (subagent_type: "general-purpose", prompt: "Run `speclink instructions --skill sync` to fetch the sync instructions, then follow them for change '<name>'. Delta spec analysis: <include the analyzed delta spec summary>"). Proceed to archive regardless of choice.

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
- If sync is requested, fetch the instructions via `speclink instructions --skill sync` and follow them (agent-driven)
- If delta specs exist, always run the sync assessment and show the combined summary before prompting
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
