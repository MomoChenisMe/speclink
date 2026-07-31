---
name: speclink-commit
description: "Commit files related to a specific Speclink change"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.3.0"
  generatedBy: "Speclink"
---

Commit files related to a specific Speclink change.

This is a **utility skill** (not a workflow step). It reads source file tracking data and artifact changes to stage and commit only the files belonging to one change — useful when multiple changes are in progress simultaneously.

**Input**: Optionally specify a change name after `/speclink-commit` (e.g., `/speclink-commit add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

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

5. **Generate commit message**

   If there are no artifact files AND no tracked source files, inform the user that there is nothing to commit and STOP.

   Run `speclink artifact cat proposal --change "<name>"`. Extract the first sentence from the Why section (or Problem/Summary section if Why is absent).

   Generate a message in this format:

   ```
   speclink(<change-name>): <summary>

   Change: <change-name>
   Tasks: <completed>/<total> complete
   ```

   Task progress comes from reading the tasks file and counting `- [x]` vs `- [ ]` checkboxes.

   If the archive sub-flow runs later (step 7a), the commit message gains an `Archived: yes` line — see step 7a.

6. **Display commit plan and message**

   Output the commit plan AND the full commit message together as one visible message — both must actually appear in the conversation before any confirmation question is asked. Show the file list grouped into sections, then the message:

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

   ### Commit Message
   speclink(<change-name>): <summary>

   Change: <change-name>
   Tasks: <completed>/<total> complete
   ```

   If no tracking file was found, show a warning instead of the Source Files section:

   ```
   ### Source Files
   ⚠ No source file tracking data found.
   Only artifact files will be committed. Use `speclink task done` during apply to enable source file tracking.
   ```

7. **User confirmation**

   Only after the commit plan and commit message from step 6 are visible in the conversation, use the **AskUserQuestion tool** to ask the user how to proceed.

   Options:
   - **Commit as shown**: Proceed with the displayed artifact + source files and the displayed commit message
   - **Include all dirty files**: Add all unrelated files to the commit as well
   - **Customize**: Let the user add or remove specific files from the commit set, or edit the commit message
   - **Archive first, then commit together**: Run archive before committing — archive file moves will be included in this commit

   If the user selects "Customize":
   - Show a numbered list of all dirty files (included and excluded)
   - Ask which files to add or remove, and whether to adjust the commit message
   - Re-display the updated commit plan and message (step 6), then ask for confirmation again

   If the user selects "Archive first, then commit together":
   - Proceed to step 7a (Archive sub-flow) before continuing to step 8

7a. **Archive sub-flow** (only when the user selected "Archive first, then commit together")

    This sub-flow executes three checks in sequence before returning to the main commit flow.

    **7a-i. Incomplete task handling**

    Run `speclink artifact cat tasks --change "<name>"` and count `- [x]` (complete) and `- [ ]` (incomplete) checkboxes in the output.

    - If **all tasks are complete**: skip to 7a-ii.
    - If **incomplete tasks exist**:
      - Display the list of incomplete tasks
      - Use the **AskUserQuestion tool** to ask: "These tasks are still incomplete. Mark all as complete before archiving?"
        - **Yes**: set a flag to pass `--mark-tasks-complete` to `speclink archive`
        - **No**: proceed without the flag (archive will continue with a warning)

      If **AskUserQuestion tool** is not available, ask the same question as plain text and wait for the user's response.

    **7a-ii. Delta spec completeness check**

    Check whether delta specs exist at `openspec/changes/<name>/specs/`.

    - If **no delta specs exist** (directory is empty or absent): skip to 7a-iii.
    - If **delta specs exist**: compare each delta against `openspec/specs/<capability>/spec.md`. The archive CLI wholesale-replaces a MODIFIED requirement with the delta's content and skips an ADDED requirement that already exists in the canonical spec — so a partial delta (one that omits canonical content that should survive) loses that content on archive.
      - If every delta is complete final-state and no ADDED requirement pre-exists: skip to 7a-iii.
      - Otherwise use the **AskUserQuestion tool** to ask: "Delta specs are not complete final-state. Normalize before archiving?"
        - **Yes**: rewrite the delta files in place — merge the omitted canonical content into MODIFIED requirements, convert pre-existing ADDED requirements to MODIFIED — then proceed. Do NOT edit main specs.
        - **No**: proceed as-is (omitted canonical content will be lost on merge)

      If **AskUserQuestion tool** is not available, ask the same question as plain text and wait for the user's response.

    **7a-iii. Archive execution, re-display, and re-confirmation**

    Execute the archive:

    ```bash
    speclink archive <name>          # without --mark-tasks-complete
    speclink archive <name> --mark-tasks-complete  # if user chose to mark tasks complete in 7a-i
    ```

    After archive completes successfully:

    1. Re-run `git status --porcelain` to capture all file changes produced by the archive (deletions from `openspec/changes/<name>/`, additions in `openspec/archived/`)
    2. Add these archive-related file changes to the commit set
    3. Regenerate the commit message with an `Archived: yes` line appended to the body
    4. Display an **updated commit plan and message** as one visible message, showing all sections:

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

    ### Commit Message
    speclink(<change-name>): <summary>

    Change: <change-name>
    Tasks: <completed>/<total> complete
    Archived: yes
    ```

    5. Use the **AskUserQuestion tool** again to confirm the updated plan and message (the archive option is no longer offered). Only continue to step 8 after this re-confirmation.

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
- **NEVER ask for confirmation before the commit plan and the full commit message have been output as visible message text** — the confirmation question must not reference content that was never displayed in the conversation (e.g., "the plan above" when no plan was shown). This applies equally to the plain-text fallback: display first, then ask
- If the tracking file is missing, warn but don't block — artifact-only commits are valid
- The "Unrelated Changes" section is informational only — these files are excluded by default
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response
