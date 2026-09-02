---
name: speclink-ingest
description: "Use when requirements change mid-work, including after a separate planning session — folds the new context into an existing change's artifacts so apply can resume."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.26.1"
  generatedBy: "Speclink"
---

Update an existing Speclink change — from a plan file or conversation context.

**Plan file support** is available when the tool has a plan directory (``). Otherwise, use conversation context to update artifacts.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Input**: Optionally specify a plan file path or name.

- `$speclink-ingest agile-discovering-rocket.md`
- `$speclink-ingest agile-discovering-rocket`
- `$speclink-ingest` (use conversation context or auto-detect plan file)

**Steps**

1. **Locate the requirement source**

   a. **Argument provided** → treat as plan file reference (prepend `` and append `.md` if needed)
   - If the file exists → use it as the plan file source, proceed to Step 2
   - If the file does NOT exist → report the error and **stop**

   b. **No argument, plan file detectable**:
   - Check conversation context for plan file path (plan mode system messages include the path like `<name>.md`)
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
   - If no changes at all → tell the user: "No active change found. Use `$speclink-propose` first to create one." and **stop**

4. **Select the change**

   Read existing artifacts for context before updating.

5. **Update artifacts**

   For each artifact, get instructions first:

   ```bash
   speclink instructions <artifact-id> --change "<name>" --json
   ```

   Use the `template` from instructions as the output structure. Apply `context` and `rules` as constraints but do NOT copy them into the file.

   The instructions JSON includes `locale` — the language to write artifacts in. If present, you MUST write the artifact content in that language. Spec files (specs/\*/\*.md) default to English instead — unless the project sets `spec_locale` in `openspec/config.yaml` (a locale code, or `auto` to follow `locale`), in which case write spec prose in that language. Structural markers (`### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST) always stay in English.

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

   **Before adding a new delta capability**, compare its name against the existing ones — the canonical specs (`speclink list --specs --json`) and the delta capabilities of other in-flight changes. If an existing name means the same capability, reuse that exact name instead of opening a near-duplicate; `speclink validate` warns on near-named new capabilities.

   **Mark manual tasks with `[M]`**: a task the agent cannot do itself — the user has to do it by hand, whether that is operating the product and accepting the result, creating an account on an external service, or placing a key — carries an `[M]` marker, so the quality stations can judge "the code is finished" separately from "a human did their part". Anything the agent can do itself, including code and automated tests, never carries it.

   **The marker goes right after the checkbox, separated by exactly one space; the task number comes after the marker, never before it.** Putting the number first reads naturally and is the easy mistake — the engine does not accept the marker there.

   ```
   Write:  - [ ] [M] 3.2 Open the imported document and confirm the list stays one list
   Not:    - [ ] 3.2 [M] Open the imported document and confirm the list stays one list
   Not:    - [ ]  [M] 3.2 Open the imported document …   (two spaces after the checkbox)
   ```

   A misplaced marker is read as ordinary description text: the task silently counts as code work, "code tasks all complete" never becomes true, and apply stalls on a task no agent may check off. `speclink validate` reports it as an error.

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

   Then state the suggestion from **Next steps** and STOP. Never invoke `$speclink-apply` yourself — starting implementation is the user's call, and this workflow is over once the summary is out.

**Guardrails**

- **NEVER** modify the original plan file in ``
- **NEVER** write application code — this skill only creates/updates Speclink artifacts
- **NEVER** create new changes — ingest only updates existing changes. If no active change exists, direct user to `$speclink-propose`
- When updating existing changes, **preserve all completed tasks** (`[x]`) — never revert progress
- If the source content is too brief to fill all artifact sections, use the **AskUserQuestion tool** to get more details rather than inventing content
- If `speclink` CLI is not available, report the error and stop
- Verify each artifact file exists after writing before proceeding to next
- **NEVER** skip the artifact workflow to write code directly
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

## Next steps

Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.

- The artifacts are updated and validated → `$speclink-apply <change-name>` to resume implementation
- A linked discussion fed this change → `speclink discuss seal <slug>` first (step 9), then the same suggestion applies
