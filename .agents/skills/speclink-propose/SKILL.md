---
name: speclink-propose
description: "Use when a change needs planning, proposing or designing — creates the change with every required artifact; seed it from a concluded discussion with --from-discussion."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.26.0"
  generatedBy: "Speclink"
---

Create a complete Speclink change proposal — from requirement to validated artifacts — in a single workflow.

**Input**: The argument after `$speclink-propose` is the requirement description. Examples:

- `$speclink-propose add dark mode`
- `$speclink-propose fix the login page crash`
- `$speclink-propose improve search performance`

If no argument is provided, the workflow will extract requirements from conversation context or ask.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Determine the requirement source**

   Priority order: explicit requirement argument → `--from-doc` document → discussion document → plan file → conversation context.

   a. **Argument provided** (e.g., "add dark mode") → use it as the requirement description, skip to deriving the change name below. A `--from-doc <path>` argument explicitly selects the document in (b); a `--from-discussion <slug>` argument explicitly selects the discussion document in (c).

   b. **Document supplied directly** (`--from-doc <path>`, speclink enhancement):
   - This is a skill-text convention modelled on `--from-discussion`, NOT an engine flag — it changes no CLI syntax and requires nothing new from the engine.
   - When the user passes it, read that document and use it as the requirement source: its title or opening statement gives the requirement description, its content feeds Why, What Changes, Capabilities, and Impact. **No existing discussion is needed** — this is the path for building a proposal straight from a plan the user brought, without going through `$speclink-discuss` first.
   - The document is consumed, not grilled: itemized challenge of its claims belongs to `discuss`. Never edit the user's original document.
   - **Leave a provenance line.** A proposal built from `--from-doc` SHALL carry one line `Source doc: <path>` in its Why or Impact section, naming the document it came from — a `--from-discussion` proposal gets its origin recorded by the link, and this line is the `--from-doc` counterpart. Skill-text convention only; the engine records nothing for you.
   - `--from-doc` outranks (c) and (d): when it is present, do not go hunting for a discussion record or a plan file.
   - When the user did NOT pass `--from-doc`, this entry does not apply at all — requirement-source determination proceeds exactly as before, with no extra file reading.

   c. **Discussion document available** (speclink enhancement):
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
   - **Follow the `Source doc:` line.** If the discussion's `## Context` carries a line `Source doc: <path>`, read that original document as well — the record stores only the decision diff, so the underlying plan lives in that file. Synthesize the two with **overlay semantics**: the document is the base layer, the discussion is the winning layer.
     - **Decided in the discussion** → the discussion wins. It is newer and it was stress-tested against the codebase.
     - **Untouched by the discussion** → the document's content carries over into the proposal as-is.
     - **Ruled out in the discussion** → SHALL NOT reappear in the proposal in any form, even where the document still advocates it.

     Worked example: the document proposes SSE plus three-times retry; the discussion ruled out SSE in favour of WebSocket and never touched retry → the proposal is WebSocket plus three-times retry, and SSE appears nowhere in it.

     Do NOT re-grill the document here — itemized challenge of a document's claims is `discuss`'s job; propose only consumes it, synthesizing or adopting verbatim. Never edit the user's original document.

     If the Context has no `Source doc:` line, this step is skipped entirely: the from-discussion flow is exactly as before, with no extra file reading.
   - If no discussion exists or the user declines → fall through to (d).

   d. **Plan file available**:
   - Check if the conversation context mentions a plan file path (plan mode system messages include the path like `<name>.md`)
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
   - If the user picks conversation context → fall through to (e)

   e. **Conversation context** → attempt to extract requirements from conversation history
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
   5. Leave a trace of the scan result in the proposal: name the related specs you found (or state that none matched) — the "why no existing spec covers this" sentence required for each New Capability in step 5 builds on this trace

   **IMPORTANT**:
   - If related specs are found, display them but do NOT stop or ask for confirmation — continue to the next step
   - If no related specs are found, proceed without pausing — the scan outcome still gets its trace in the proposal

4. **Create the change directory**

   ```bash
   speclink new change "<name>" --agent codex
   ```

   When the proposal is sourced from a discussion document (path (c) in step 1), pass the link so the change records its origin and the discussion is marked `promoted` (it will be archived together with the change later):

   ```bash
   speclink new change "<name>" --agent codex --from-discussion <slug>
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

   - `<capability-name>`: <brief description>. <one sentence on why no existing spec covers this — name the nearest specs the step-3 scan surfaced and where they fall short>

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
     - `locale`: The language to write the artifact in (e.g., "Japanese (日本語)"). If present, you MUST write the artifact content in this language. Spec files (specs/\*_/_.md) default to English instead — unless the project sets `spec_locale` in `openspec/config.yaml` (a locale code, or `auto` to follow `locale`), in which case write spec prose in that language. Structural markers (`### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST) always stay in English.
   - Read each completed dependency for context via `speclink artifact cat <artifact-id> --change "<name>"` (never open artifact files by path — the documents may live in a remote store)
   - Generate the artifact content using `template` as the structure
   - **Mark manual tasks with `[M]`** (tasks artifact only): a task the agent cannot do itself — the user has to do it by hand, whether that is operating the product and accepting the result, creating an account on an external service, or placing a key — carries an `[M]` marker. Anything the agent can do itself, including code and automated tests, never carries it. The marker is what lets the quality stations judge "the code is finished" separately from "a human did their part": they run once every non-`[M]` task is checked, while archive still waits for all of them.

     **The marker goes right after the checkbox, separated by exactly one space; the task number comes after the marker, never before it.** Putting the number first reads naturally and is the easy mistake — the engine does not accept the marker there.

     ```
     Write:  - [ ] [M] 3.2 Open the imported document and confirm the list stays one list
     Not:    - [ ] 3.2 [M] Open the imported document and confirm the list stays one list
     Not:    - [ ]  [M] 3.2 Open the imported document …   (two spaces after the checkbox)
     ```

     A misplaced marker is read as ordinary description text: the task silently counts as code work, "code tasks all complete" never becomes true, and apply stalls on a task no agent may check off. `speclink validate` reports it as an error.
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

     **The `--new` flag declares a new capability.** A capability the canonical specs do not carry yet is refused by default, and the error lists up to three similar existing names with their Purpose lines. Always run the command WITHOUT `--new` first; only when it refuses AND you have confirmed the suggestion list holds no synonym of your capability, re-run the same command with `--new` appended to declare it as genuinely new. If a suggested name IS the same capability, reuse that exact name instead of declaring a new one.

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

    Inform the user that the change is ready and that running `$speclink-apply <change-name>` when ready will start implementation.

    If you are currently in Codex Plan Mode, also remind the user to switch the session to normal mode before running `$speclink-apply <change-name>`. This is only a reminder: do NOT try to use ExitPlanMode or EnterPlanMode, do NOT ask whether to switch modes, and do NOT invoke apply.

    The propose workflow ENDS here. Do NOT invoke `$speclink-apply`. Do NOT call **AskUserQuestion** to ask whether to apply. This behavior is identical across Auto Mode, interactive mode, and any other agent mode.

    After the summary, run the **Pending-change landscape check** below before presenting Next steps.

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
- **NEVER** invoke `$speclink-apply` — this workflow ends after artifact creation. The user decides when to start implementation
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

## Pending-change landscape check

Run this check after the summary, right before presenting the Next steps below.

1. Run `speclink list --json` for the change names, then judge the stage from each change's own metadata: a change whose `openspec/changes/<name>/.openspec.yaml` carries none of the `started_*` lines (`started_at:` / `started_by:` / `started_with:`) is still at the proposal stage. The `list` payload alone cannot tell a started change from an unstarted one — do not judge from its `status` or task counts. The change just created counts.
2. **Only one proposal-stage change** (the one just created) → skip the rest of this check; the Next steps edges below already cover it.
3. **Two or more** → work out an execution order before any apply suggestion:
   - **Hard signal — delta capability overlap**: two changes that both carry a delta for the same capability (the same directory name under `openspec/changes/<name>/specs/`) must run sequentially — their deltas rewrite the same canonical spec, and archiving them out of order can trip the merge gate.
   - **Soft signal — likely code overlap or dependency**: read each proposal's Impact and tasks; changes that touch the same code areas, or where one builds on another's outcome, are safer run in sequence.
4. Present the result according to the project's effective worktree policy (`speclink workflow-config show --json` → `worktree`; a `SPECLINK_WORKTREE` env override wins):
   - **Policy on** → two groups: "parallel-safe — run each change in its own session via `$speclink-apply-with-worktree` (the multi-session recipe)" and "sequential — run in this order, one at a time".
   - **Policy off** → one recommended order covering all of them.
5. The check is suggestions only — report the grouping or order and stop; never invoke any skill automatically.

## Next steps

Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.

- Artifacts are complete → `$speclink-apply <change-name>` when the user is ready to implement (with two or more proposal-stage changes pending, the landscape check above sets the order first)
- Several independent changes will be implemented at once, and the project's worktree policy is on → `$speclink-apply-with-worktree <change-name>` (one git worktree per change)
- The requirements turned out to be fuzzier than they looked → `$speclink-discuss` before implementing
