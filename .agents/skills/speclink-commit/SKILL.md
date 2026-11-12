---
name: speclink-commit
description: "Use when only the files belonging to one specific change should be committed — selects that change's files and writes the commit."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.58.0"
  generatedBy: "Speclink"
---

**Conversation language**: write to the user in the language they use with you — progress updates, questions, and summaries. This skill is written in English, and its example messages show structure, not language: translate their prose, and keep commands, file paths, code, and fixed tokens (severity labels, status words) as they are. When a step sets the language of its own output — a document, a report, or the way it presents results and asks about them (for example through `locale`) — that step decides for that output. When you edit a document that already exists and no step sets its language, keep the language it is written in. When you run in a forked context that cannot see the conversation, use the language of the request you were given.

Commit files related to a specific Speclink change.

This is a **utility skill** (not a workflow step). It reads source file tracking data and artifact changes to stage and commit only the files belonging to one change — useful when multiple changes are in progress simultaneously.

**Input**: Optionally specify a change name after `$speclink-commit` (e.g., `$speclink-commit add-auth`). If omitted, check if it can be inferred from conversation context; if it is still vague or ambiguous, step 1 asks the user.

**Prerequisites**: This skill requires `git`. Run `git --version`. If git is not available (command not found or similar error), inform the user to install git and STOP.

**Asking the user**: ask with the **AskUserQuestion tool**; when that tool is not available, ask the same question with the same options as plain text and wait for the user's response. The display-first rule of step 7 applies to the plain-text question too.

**Write-back: changing an artifact that already exists.** Every fix and update this skill makes to an artifact that already exists is a write-back. Read the artifact's full text with `speclink artifact cat <artifact-id> --change "<name>"` — a delta spec is `specs/<capability>`, which the `deltaSpecs` field of `speclink show <name> --json` lists as `<capability>/spec.md` — make the change on that full text, and write the whole result back with `--stdin --force`:

```bash
speclink new artifact <artifact-id> --change "<name>" --stdin --force <<'ARTIFACT_EOF'
<full updated content>
ARTIFACT_EOF
```

A delta spec is written back with `speclink new artifact spec <capability> --change "<name>" --stdin --force`. The same verbs work when the documents live in a remote store. When `new artifact` refuses a write-back with an error (for example, a task list whose tasks are all checked), stop and tell the user the message — do not edit the file instead.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select, and wait for the answer

   Always announce: "Committing for change: <name>"

2. **Read the evidence record**

   Run `speclink evidence show "<change-name>" --json` — the change's completion evidence,
   written by `speclink task done`. `entries` holds one entry per completed task, which
   gives the source files grouped by task; `files` is every touched file, listed once.

   Expected format:

   ```json
   {
     "change": "<change-name>",
     "entries": [
       {
         "taskId": "tsk_…",
         "taskDesc": "Task description",
         "touchedFiles": ["src/file1.ts", "src/file2.ts"]
       }
     ],
     "files": ["src/file1.ts", "src/file2.ts"]
   }
   ```

   If `files` is empty, check the pre-move location `.speclink/touched/<change-name>.json`
   (read-only compatibility for records written before the move — a state file of this
   checkout, not a spec document). If neither gives a file, proceed without source file data.

3. **Collect artifact files**

   Run `git status --porcelain` and filter the output to files under `speclink/changes/<name>/`. These are the change's artifact files (proposal, design, tasks, specs, etc.) — including `.evidence.json` itself, which is version-controlled and belongs to this change's commit.

4. **Identify unrelated dirty files**

   From the full `git status --porcelain` output, any dirty files that are neither in the artifact set nor in the evidence record are "unrelated changes."

5. **Generate commit message**

   If there are no artifact files AND no tracked source files, inform the user that there is nothing to commit and STOP.

   Run `speclink artifact cat proposal --change "<name>"`. Extract the first sentence from the Why section (or Problem/Summary section if Why is absent).

   Generate a message in this format:

   ```
   speclink(<change-name>): <summary>

   Change: <change-name>
   Tasks: <completed>/<total> complete
   ```

   Task progress is the change's `completedTasks` and `totalTasks` in `speclink list --json`.

   If the archive sub-flow runs later (step 7a), the commit message gains an `Archived: yes` line — see step 7a.

6. **Display commit plan and message**

   Output the commit plan AND the full commit message together as one visible message — both must actually appear in the conversation before any confirmation question is asked. Show the file list grouped into sections, then the message:

   ```
   ## Commit Plan: <change-name>

   ### Change Artifacts
   - M  speclink/changes/<name>/proposal.md
   - M  speclink/changes/<name>/tasks.md

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

   If no evidence record was found, show a warning instead of the Source Files section:

   ```
   ### Source Files
   ⚠ No source file tracking data found.
   Only artifact files will be committed. Use `speclink task done` during apply to enable source file tracking.
   ```

7. **User confirmation**

   Only after the commit plan and commit message from step 6 are visible in the conversation, use the **AskUserQuestion tool** to ask the user how to proceed.

   Options:
   - **Commit as shown**: Proceed with the displayed files and the displayed commit message
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

    This sub-flow runs its steps in sequence before returning to the main commit flow.

    **7a-i. Incomplete task handling**

    Compare the change's `completedTasks` with its `totalTasks` in `speclink list --json`.

    - If **all tasks are complete**: skip to 7a-ii.
    - If **incomplete tasks exist**:
      - Display the list of incomplete tasks — the `- [ ]` lines of `speclink artifact cat tasks --change "<name>"`
      - Use the **AskUserQuestion tool** to ask: "These tasks are still incomplete. Mark all as complete before archiving?"
        - **Yes**: set a flag to pass `--mark-tasks-complete` to `speclink archive`
        - **No**: proceed without the flag (archive will continue with a warning)

    **7a-ii. Delta spec completeness check**

    The change's `deltaCapabilities` in `speclink list --json` names every capability it carries a delta spec for.

    - If **`deltaCapabilities` is absent or empty**: skip to 7a-ii-b.
    - If **it names capabilities**: for each one, set the delta (`speclink artifact cat specs/<capability> --change "<name>"`) beside the canonical spec (`speclink show <capability> --item-type spec`; a capability this change creates has none yet). The archive merge engine is fail-closed: a MODIFIED requirement wholesale-replaces the canonical block, and the engine refuses the whole archive with zero file effect when an ADDED requirement already exists in the canon, a MODIFIED/REMOVED/RENAMED target is missing, or a MODIFIED block drops a canonical scenario without a `<!-- REMOVED-SCENARIO: <name> -->` declaration.
      - If every delta is complete final-state and no ADDED requirement pre-exists: skip to 7a-ii-b.
      - Otherwise use the **AskUserQuestion tool** to ask: "Delta specs would be refused by the archive merge gate. Fix them before archiving?"
        - **Yes**: fix each delta with a write-back (see **Write-back** above the steps) — merge the omitted canonical content into MODIFIED requirements (or declare deliberate drops with `<!-- REMOVED-SCENARIO: … -->`), drop or retarget pre-existing ADDED requirements — then proceed. Do NOT edit main specs.
        - **No**: skip the archive (commit without it) and route the delta repair through `speclink drift <name>` → `$speclink-ingest <name>` — archiving as-is would exit non-zero

    **7a-ii-b. Plan order hint**

    Run `speclink plan --json` and find the target change in its `changes` array. Tell the user each of these that applies:

    - `archiveAfter` is non-empty:

      > plan 建議先封存 <archiveAfter 的名稱>；它們封存後，重讀本 change 對同名 requirement 的 MODIFIED／REMOVED／RENAMED 區塊、對照正式規格重寫（走 `$speclink-ingest`）再封存 <name>。重疊的 requirement：<requirementOverlap 裡對應那些 change 的 capability › requirement>。

    - `requirementOverlap` holds an entry whose `conflict` is true:

      > <name> 與 <change> 都新增（或改名成）同名的 requirement <requirement>，或都移除（或改名掉）它；後封存的那一個會封存失敗（名字重複，或名字已不存在），先改掉其中一邊。

    - `blockedBy` is non-empty:

      > 宣告前置 <blockedBy 的名稱> 尚未封存。

    Do not offer `speclink drift` as the remedy for an overlap: drift only checks that the requirement names still exist, and cannot see content that an earlier archive overwrote.

    Also note every other entry of `changes` that lists `<name>` in its `archiveAfter` — even when none of the hints above applies: those changes archive after `<name>` and touch the same requirements. Once `<name>` is archived, the plan no longer names it, so the reminder after the archive is the last point where their rewrite gets named — keep the list for it. The note asks nothing now.

    Then use the **AskUserQuestion tool** to ask which way to go — archive `<name>` now anyway, or stop here and handle those first (plain text + wait if the tool is unavailable). This is a suggestion only. It does NOT block the archive and relies on no engine gate: if the user confirms, archive as usual. An empty `archiveAfter` and `blockedBy` with no conflict, a target missing from the plan, or a `plan` failure (a dependency cycle) → say nothing about ordering and continue. When the user stops instead, skip the archive (commit without it): the changes named above get archived first, or one side of the conflict gets changed first.

    **7a-iii. Archive execution, re-display, and re-confirmation**

    Execute the archive:

    ```bash
    speclink archive <name>          # without --mark-tasks-complete
    speclink archive <name> --mark-tasks-complete  # if user chose to mark tasks complete in 7a-i
    ```

    After archive completes successfully:

    1. Re-run `git status --porcelain` to capture all file changes produced by the archive (deletions from `speclink/changes/<name>/`, additions in `speclink/changes/archive/`)
    2. Add these archive-related file changes to the commit set
    3. Regenerate the commit message with an `Archived: yes` line appended to the body
    4. Display an **updated commit plan and message** as one visible message, showing all sections:

    ```
    ## Updated Commit Plan: <change-name> (with archive)

    ### Change Artifacts (archived)
    - D  speclink/changes/<name>/proposal.md
    - D  speclink/changes/<name>/tasks.md
    - ...

    ### Archived Files
    - A  speclink/changes/archive/<name>/proposal.md
    - A  speclink/changes/archive/<name>/tasks.md
    - ...

    ### Source Files
    (same as before)

    ### Main Spec Updates (from archive's delta application)
    - M  speclink/specs/<spec-name>/spec.md
    - ...

    ### Commit Message
    speclink(<change-name>): <summary>

    Change: <change-name>
    Tasks: <completed>/<total> complete
    Archived: yes
    ```

    5. Use the **AskUserQuestion tool** again to confirm the updated plan and message (the archive option is no longer offered). Only continue to step 8 after this re-confirmation.

    6. Close the sub-flow with the reminders below. Print them once, wherever the flow ends: after the step 10 result, or right where the user stops at the re-confirmation above — the archive has already run either way, and after a stop its file moves are still uncommitted, so also remind the user to commit them with a plain git commit:

       - When the workspace has a `speclink/manual/` directory, add one line: the manual may be stale now, and `$speclink-manual` will report which pages this archive's spec changes outdated. The condition is the directory's existence only — do not work out which specs this archive touched, and do not judge whether the manual is actually stale; that is the manual skill's report. This is a reminder only — never run `$speclink-manual` yourself.
       - When the plan order hint noted changes that list `<name>` in their `archiveAfter`, add one more line:

         > <那些 change 的名稱> 要在 <name> 之後封存，並動到同名的 requirement：先重讀它們對同名 requirement 的 MODIFIED／REMOVED／RENAMED 區塊、對照正式規格重寫（走 `$speclink-ingest`），再封存它們。

         This too is a reminder only — never run the ingest yourself.
       - Run `speclink plan --json` and hand the user the next change to start. When `next` is non-null, add one more line:

         > plan 的下一個可開工：<next>，執行 `$speclink-apply <next>`。

         When the effective worktree policy is on (`speclink workflow-config show --json` → `worktree`; a `SPECLINK_WORKTREE` env override wins) and `changes` holds two or more entries with `wave` 1 and `stage` `proposed` (wave 1 waits on nothing, so their `blockedBy` is empty), also list them as parallel-safe:

         > 第 1 波可並行：<name-a>、<name-b>，各開一個 session 走 `$speclink-apply-with-worktree <name>`。

         Policy off, or only one such change → name `next` alone. A null `next`, or a `plan` failure (a dependency cycle) → say nothing about ordering. This too is a reminder only — never run apply yourself.

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

9a. **Report the landing**

    Do this only after `git commit` succeeded. The commit is where this change's code landed; record it. Take the commit from `git rev-parse HEAD` and pass it, so the report names it wherever the verb runs:

    ```bash
    speclink change landed "<change-name>" --commit "<commit>"
    ```

    Skip this step in these two cases, and say which one applied:

    - The current branch (`git branch --show-current`) is `speclink/<change-name>`, the change's branch of a linked worktree. That commit is not the landing yet: `$speclink-worktree-merge` reports it after the merge.
    - The archive sub-flow ran (step 7a). The change is archived now, and a landing report needs an active change.

    On any other branch, report. The engine does not know which branch is the main one; a host that needs to know checks the recorded commit and branch itself.

    When the verb is refused (for example, a Server that has no such route), tell the user its message and continue with step 10. Leave the commit as it is: do not undo it, and do not make it again.

    The report is recorded in the change's own data file, so that file shows as modified after this step. Say so in the result: it goes into this change's next commit.

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
**Landing:** reported ✓ — or "left to the worktree merge", "not reported: the change is archived", or the verb's refusal message
```

**Output On Nothing To Commit**

```
## Nothing to Commit

**Change:** <change-name>

No dirty files found for this change (no modified artifacts, no tracked source files).
```

**Guardrails**

Check these before you commit:

- [ ] Every file was staged individually with `git add <file>` — no `git add .` or `git add -A` (step 8).
- [ ] The user confirmed the exact file list; nothing unconfirmed was committed, and nothing was staged silently (step 7).
- [ ] **NEVER ask for confirmation before the commit plan and the full commit message have been output as visible message text** — the confirmation question must not reference content that was never displayed in the conversation (e.g., "the plan above" when no plan was shown). This applies equally to the plain-text fallback: display first, then ask.
- [ ] A missing evidence record produced a warning, not a block — artifact-only commits are valid (step 6).
- [ ] The "Unrelated Changes" section stayed informational; those files were excluded by default (step 6).
