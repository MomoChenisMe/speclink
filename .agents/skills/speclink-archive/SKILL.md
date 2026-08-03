---
name: speclink-archive
description: "Archive a completed change"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.6.0"
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

   Background: the merge engine is fail-closed. It validates every capability's delta
   against the canonical spec first and writes nothing until all of them pass, so a stale
   or self-contradicting delta refuses the archive with zero file effect instead of
   silently dropping content. It refuses when:
   - an ADDED requirement name already exists in the canonical spec
   - a MODIFIED/REMOVED/RENAMED source requirement no longer exists
   - one requirement name appears under more than one operation section of the delta
   - a RENAMED target name is already taken in the canonical spec
   - a MODIFIED block drops a scenario the canonical requirement has, without declaring it
   - a capability with no canonical spec yet carries anything other than ADDED

   A MODIFIED requirement **wholesale-replaces** the canonical requirement block, so it must
   contain the complete final text. To drop a canonical scenario on purpose, declare it
   inside the MODIFIED block — one per line, stripped before the text reaches the canon:

   ```markdown
   ### Requirement: Retry policy

   <!-- REMOVED-SCENARIO: Offline queue -->
   ```

   A delta for a capability being created may also carry a `## Purpose` section; it becomes
   the new canonical spec's Purpose (without one, a TBD placeholder is written).

   **For each delta spec, compare against `openspec/specs/<capability>/spec.md`:**
   - MODIFIED requirements: does the canonical requirement contain scenarios or content the delta omits but that should survive?
   - ADDED requirements: does the requirement already exist in the canonical spec (e.g., from an earlier mid-flight sync)?

   **If every delta is already complete final-state and no ADDED requirement pre-exists:** proceed directly to the archive (step 5) — no prompt needed.

   **Otherwise**, show a summary of what the engine would refuse, then use the **AskUserQuestion tool**:
   - "Fix the delta then archive (recommended)": rewrite the delta spec file(s) in place —
     - merge the omitted canonical content into each MODIFIED requirement so it reads as the complete final state, or declare the drop with `<!-- REMOVED-SCENARIO: … -->`
     - drop or retarget each pre-existing ADDED requirement (a requirement the canon already carries is edited via MODIFIED, not re-added)
     - do NOT edit the main specs — only the delta files change
   - "Refresh from the codebase": run `/speclink-drift <name>` to see what moved, then `/speclink-ingest <name>` to update the delta — the route the refusal message itself points at
   - "Cancel"

   After fixing, show a brief diff summary of the rewritten delta files, then continue.

5. **Perform the archive**

   Use the `speclink archive` CLI command which handles the full archive workflow
   (spec snapshot, delta application, @trace injection, identity recording):

   ```bash
   speclink archive <name>
   ```

   **Optional flags:**
   - `--skip-specs` — skip delta spec application (for tooling/doc-only changes)
   - `--mark-tasks-complete` — mark all incomplete tasks as complete before archiving
   - `--no-validate` — skip delta spec validation

   **If archive fails** because the archived name already exists, suggest renaming existing archive.

   **If the merge gate refuses**, the error lists every offending operation
   (capability / operation / requirement / reason) at once. Fix them in one round on the
   delta files — `speclink drift <name>` shows what moved, `/speclink-ingest <name>`
   updates the delta — then re-run the archive. `--no-validate` does not unlock the gate;
   `--skip-specs` skips spec application entirely.

6. **Display summary**

   Show archive completion summary including:
   - Change name
   - Schema that was used
   - Archive location
   - Spec sync status (synced / sync skipped / no delta specs)
   - Note about any warnings (incomplete artifacts/tasks)

7. **Clean up tracking file** — only after the archive has been committed

   `.speclink/touched/<change-name>.json` holds the change's per-task evidence. Delete it once
   the archive is committed, never before:

   ```bash
   rm -f .speclink/touched/<change-name>.json
   ```

   Two reasons the deletion trails the archive and the commit:

   - **It is the `@trace` source.** `speclink archive` builds the trace file list from this
     record; with the record gone the list falls back to scanning the work tree's dirty files,
     sweeping unrelated edits — a parallel session's work, an unfinished experiment — into the
     canonical specs.
   - **It is the commit skill's file-attribution source.** `$speclink-commit` reads it to decide
     which files belong to this change, so deleting before the commit lands leaves it with no
     file list.

   If the file does not exist, silently continue.

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
- Fixing a delta rewrites delta files only — NEVER edit main specs directly; delta application is the archive CLI's job
- If delta specs exist, always run the completeness assessment; only prompt when a fix is actually needed
- Never work around the merge gate — it protects the canonical specs from silent data loss; fix the delta instead
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response


## RENAMED is actually executed (speclink-specific)

speclink executes renames at archive time — `## RENAMED Requirements` is applied to the
canonical spec, not merely recorded. Both documented forms work:

```markdown
## RENAMED Requirements

- FROM: `### Requirement: Old Name`
- TO: `### Requirement: New Name`
```

or the header form (`### Requirement: Old Name` followed by a `TO: New Name` line). The
canonical requirement header is rewritten, the summary counts it under `renamed:`, and a
rename-only delta is a valid change (validates and archives). A stale FROM target — or a TO
name the canon already carries — refuses the archive; `speclink drift` surfaces the same
verdict in `spec_assumptions` so it routes to ingest before archiving.

## Bulk archive (speclink-specific)

When several changes are finished, archive them in one pass:

```bash
speclink archive <name-a> <name-b>     # explicit set
speclink archive --all                 # every ready change
```

Semantics (the CLI enforces all three):

1. **Clean work tree required** — a change's @trace file list is aggregated from its evidence
   record (`.speclink/touched/<name>.json`) when one exists, and falls back to the work tree's
   dirty code files only when it does not. That fallback would inject unrelated dirty files into
   the archived change's canonical specs. Commit first; the command refuses otherwise and lists
   the offending files.
2. **Skip, never silently** — a change is archived only when it is ready: tasks complete
   (or `--mark-tasks-complete`), validation passes (or `--no-validate`), and no delta
   operation the merge gate would refuse (a MODIFIED/REMOVED target another change already
   rewrote — run `speclink drift <name>` and reconcile via ingest). The pre-check reads the
   engine's own verdict, so it never disagrees with it; it only reports the change as
   skipped with the reason and a `Bulk archive: N archived, M skipped` summary instead of
   aborting the run.
3. **Fail-fast** — archives apply in created-date order; on the first hard error the run
   stops and reports archived / failed / untouched (already-archived changes cannot be
   rolled back automatically).

Each archived change still gets the full single-archive treatment: delta application with
@trace, snapshot for unarchive, `.started` cleanup, and its linked discussion archived
alongside. Delete each change's `.speclink/touched/<name>.json` only after the archive is
committed, exactly as in the single-archive cleanup step — never before archiving.
