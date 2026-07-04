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
