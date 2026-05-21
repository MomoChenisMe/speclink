---
name: speclink-propose
description: "Create a SDD change with all required artifacts and transition out of proposing state"
effort: xhigh
license: MIT
compatibility: Requires speclink CLI.
speclink_version: 0.1.0
template_hash: sha256:<generated-at-build>
metadata:
  author: speclink
  version: "1.0"
  generatedBy: SpecLink
---

Create a complete SpecLink change — from requirement to validated artifacts — in a single workflow. The change ends in `reviewing` state (or `ready` if review is disabled), ready for human review or direct apply.

**Input**: The argument after `/speclink-propose` is the requirement description. Examples:

- `/speclink-propose add dark mode`
- `/speclink-propose fix the login page crash`
- `/speclink-propose improve search performance`

If no argument is provided, the workflow will look for:

1. A converged discussion with `Capture to: change <name>` matching the topic
2. A plan file referenced by your AI tool's conversation context (e.g., a "plan mode" file path)
3. Relevant requirements from conversation context

If none of the above yields a clear requirement, ask the user.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP. Also verify the project is initialized — if `speclink status` reports `project.not_initialized`, ask the user to run `speclink init <project-name>` first.

**Steps**

1. **Determine the requirement source**

   a. **Argument provided** (e.g., "add dark mode") → use it as the requirement description, skip to deriving the change name below.

   b. **Linked discussion**:
   - Check converged discussions whose `Capture to:` references a change matching the topic:
     ```bash
     speclink discuss list --status converged --json
     ```
   - If a discussion is found, read it:
     ```bash
     speclink discuss show <topic-id> --json
     ```
   - Use the discussion's Background, Decisions Made, and Conclusion as the proposal source.
   - Ask the user to confirm: use this discussion, or override with explicit input.

   c. **Plan file available** (if your AI tool exposes one):
   - Check conversation context for a plan file path
   - If found and accessible, read it and extract:
     - Title → use as requirement description
     - Context section → use as proposal Why/Motivation
     - Implementation stages → use for artifact creation
     - File paths mentioned → use for Impact section
   - Ask the user to confirm using the plan file vs conversation context.

   d. **Conversation context** → attempt to extract requirements from conversation history.
   - If context is insufficient, ask the user what they want to build.

   From the resolved description, derive a kebab-case change name (e.g., "add dark mode" → `add-dark-mode`).
   Strip archive-style date prefixes if present (e.g., `2026-05-20-foo` → `foo`).

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

2. **Classify the change type**

   Based on the requirement, classify into one of three types:

   | Type     | When to use                                                         |
   | -------- | ------------------------------------------------------------------- |
   | Feature  | New functionality, new capabilities                                 |
   | Bug Fix  | Fixing existing behavior, resolving errors                          |
   | Refactor | Architecture improvements, performance optimization, UI adjustments |

   This determines the proposal template format in step 5.

3. **Scan existing specs and changes for relevance**

   Before creating the change, check if anything overlaps:

   ```bash
   speclink list --specs --json
   speclink list --changes --json
   ```

   - Compare returned items against the user's description to identify related entries (max 5 candidates).
   - For each candidate (max 3), inspect the first lines to retrieve the Purpose section:
     ```bash
     speclink show spec <capability> --json
     # or
     speclink show change <change-id> --json
     ```
   - If related items are found, display them as an informational summary.

   **IMPORTANT**:
   - If related specs / changes are found, display them but do NOT stop or ask for confirmation — continue to the next step.
   - If nothing related is found, silently proceed without mentioning the scan.

4. **Create the change**

   ```bash
   speclink new change "<name>" [--schema <schema-id>] --json
   ```

   - Default schema is the one configured in `.speclink/config.yaml#schema` (typically `spec-driven`).
   - Pass `--schema <id>` only if the user explicitly wants a non-default workflow schema.
   - If a change with that name already exists (`change.already_exists`), suggest continuing the existing change or picking a different kebab-case name.

   The change is now in state `proposing`.

5. **Write the proposal**

   **IMPORTANT — file path rules for the `## Impact` section:**
   - All file paths SHALL be written relative to the project root (e.g., `crates/speclink-core/src/foo.rs`, `doc/skill-drafts/speclink-propose.md`).
   - Do NOT use relative fragments (e.g., `core/mod.rs`, `parser/lib.rs`) — preflight rejects them as non-anchored paths.
   - Do NOT wrap shell commands in backticks inside artifact text (e.g., `` `git mv a.rs b.rs` ``) — the backtick extractor may otherwise mis-parse the command as a file reference.
   - When referring to a file without naming its concrete path, use descriptive prose ("the engine entry module") rather than a backticked path fragment.

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

   **If sourcing from a linked discussion**: incorporate the Conclusion's Decision / Rationale into the proposal's Why / Motivation. Optionally reference the discussion id in proposal frontmatter or a footer line (e.g., "Discussion: `sse-vs-websocket`").

6. **Get the artifact build order**

   ```bash
   speclink status --change "<name>" --json
   ```

   Parse the JSON to get:

   - `applyRequires`: array of artifact IDs needed before the change can leave `proposing` state (varies by schema)
   - `artifacts`: list of all artifacts (from the change's schema) with their status and dependencies

   **Note**: The artifact list is **schema-driven**. The default `spec-driven` schema produces `proposal` / `spec` (multi) / `design` (optional) / `tasks`. A custom schema may differ — never assume a hardcoded artifact set.

7. **Create remaining artifacts in sequence**

   Loop through artifacts in dependency order (skip proposal since it's already done):

   a. **For each artifact that is `ready` (dependencies satisfied)**:
   - **Check if the artifact is optional**: If the artifact is NOT in the dependency chain of any `applyRequires` artifact (i.e., removing it would not block the state transition out of `proposing`), it is optional. Get its instructions and read the `instruction` field. If the instruction contains conditional criteria ("create only if..."), evaluate whether any criteria apply to this change based on the proposal content. If none apply, skip and show: "⊘ Skipped <artifact-id> (not needed for this change)". Continue to next artifact.
   - Get instructions:
     ```bash
     speclink instructions <artifact-id> --change "<name>" --json
     ```
   - The instructions JSON includes:
     - `context`: Project background from `.speclink/config.yaml#context` (constraints for you - do NOT include in output)
     - `rules`: Artifact-specific rules from `.speclink/config.yaml#rules.<artifact>` (constraints for you - do NOT include in output)
     - `template`: The structure to use for your output file
     - `instruction`: Schema-specific guidance
     - `outputPath`: Where the artifact will be written
     - `dependencies`: Completed artifacts you should read for context
     - `locale`: Language for the artifact (e.g., "Traditional Chinese (繁體中文)"). If present, write the artifact body in this language. **Exception**: spec files (`.speclink/changes/<name>/specs/**/*.md`) MUST always be written in English regardless of locale, because they use normative language (SHALL/MUST).
   - Read any completed dependency files for context.
   - Generate the artifact content using `template` as the structure.
   - Apply `context` and `rules` as constraints — but do NOT copy them into the file.
   - Write via CLI (CLI handles directory creation and format validation):

     For **design** or **tasks** (single-instance artifacts):

     ```bash
     speclink new artifact <artifact-id> --change "<name>" --stdin <<'ARTIFACT_EOF'
     <content>
     ARTIFACT_EOF
     ```

     For **spec** (one command per capability — `spec` is `multi: true`):

     ```bash
     speclink new artifact spec --change "<name>" --capability <capability-name> --stdin <<'ARTIFACT_EOF'
     <delta spec content>
     ARTIFACT_EOF
     ```

     If the command fails with a validation error (engine auto-validates each write), fix the content and retry.

   - Show brief progress: "✓ Created <artifact-id>"

   b. **Continue until all `applyRequires` artifacts are complete**
   - After creating each artifact, re-run `speclink status --change "<name>" --json`.
   - Check if every artifact ID in `applyRequires` has `status: "done"`.
   - Stop when all `applyRequires` artifacts are done.

   c. **If an artifact requires user input** (unclear context):
   - Ask the user to clarify the specific question.
   - Then continue with creation.

8. **Inline Self-Review** (before letting the engine auto-validate)

   After creating all artifacts, scan them manually. Fix issues inline.

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

   **Check 5: Durable Handoff Review**

   This change must survive being handed to another agent / role / week. Reject and fix:

   - **File-path-only tasks**: a task whose entire description is "edit file X" with no behavior, contract, or verification target. File paths are locator context — the task SHALL still describe what is observably true when complete.
   - **Line-number-coupled instructions**: design or tasks content that points to "line 42" / "the function on lines 80-95" as the only way to identify the work. Source line numbers drift; name the function, command, struct, or behavior instead.
   - **Vague acceptance criteria**: success conditions like "works correctly", "behaves as expected", "handles edge cases" without naming the observable behavior or the verification target (test name, CLI invocation, analyzer rule, manual assertion).
   - **Missing scope boundaries on non-trivial work**: design lacking explicit "in scope" / "out of scope" lines for any change that touches more than one subsystem or introduces new behavior. Trivial artifact-only edits MAY skip this; runtime, build, or tooling effects MUST NOT.

   Fix every failure inline using the existing context. If a failure cannot be fixed without new input from the user, surface it explicitly rather than papering over it.

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

   Engine auto-validates each `new artifact` write. After all artifacts are written, optionally run cross-artifact analysis for self-review:

   1. Run `speclink analyze "<name>" --json`
   2. Filter findings to **Critical and Warning only** (ignore Suggestion)
   3. If no Critical/Warning findings → show "Artifacts look consistent ✓" and proceed
   4. If Critical/Warning findings exist:
      a. Show: "Found N issue(s), fixing... (attempt M/2)"
      b. Fix each finding in the affected artifact (re-write with `new artifact ... --overwrite` if needed)
      c. Re-run `speclink analyze "<name>" --json`
      d. Repeat up to 2 total iterations
   5. After 2 attempts, if findings remain:
      - Show remaining findings as a summary
      - Proceed normally (do NOT block)

10. **Validation and state transition**

    Run a final validation:

    ```bash
    speclink validate "<name>" --strict --json
    ```

    If validation fails, fix errors and re-validate.

    Once validation passes, the engine automatically transitions the change out of `proposing`:

    - If `.speclink/config.yaml#require_artifact_review: true` → state becomes `reviewing`
    - If `false` → state becomes `ready`

    Check the resulting state:

    ```bash
    speclink status --change "<name>" --json
    ```

    **Show summary**:

    - Change name and location (`.speclink/changes/<name>/`)
    - List of artifacts created
    - Validation result
    - Current state (`reviewing` or `ready`)
    - Linked discussion id (if sourced from a discussion) — note: discussion is preserved as converged; engine added this change to its `linked_changes`. Do NOT delete the discussion as part of this workflow. If the user later wants to remove it, they can run `speclink discuss delete <topic-id>` explicitly.

    **Tell the user the next step based on state**:

    - State = `reviewing` → "Change is awaiting review. When ready, approve with:
      ```bash
      speclink review approve --change <name> --reviewer <your-id>
      ```
      Then run `/speclink-apply <name>` to begin implementation."

    - State = `ready` → "Change is ready for implementation. Run `/speclink-apply <name>` to begin."

    The propose workflow ENDS here. Do NOT invoke `/speclink-apply`. Do NOT auto-approve review. The user decides when to start implementation and (if required) when to approve.

---

## Artifact Creation Guidelines

- Follow the `instruction` field from `speclink instructions` for each artifact type.
- Read dependency artifacts for context before creating new ones.
- Use `template` as the structure for your output file — fill in its sections.
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file.
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact.
  - These guide what you write, but should never appear in the output.
- **Parallel task markers (`[P]`)**: When creating the **tasks** artifact, first read `.speclink/config.yaml#parallel_tasks`. If `true`, add `[P]` markers to tasks that can be executed in parallel. Format: `- [ ] [P] Task description`. A task qualifies for `[P]` if it targets different files from other pending tasks AND has no dependency on incomplete tasks in the same group. When `parallel_tasks` is not enabled, do NOT add `[P]` markers.
- **TDD task ordering**: When `.speclink/config.yaml#tdd: true`, format tasks in the TDD order: write test (red) → implement (green) → refactor. Use sub-numbering like `1.1`, `1.2`, `1.3` for the red/green/refactor triplet.

---

## Concurrency & Errors

- `change.locked` (another agent is writing to the same change) → retry 1-2 sec × max 3 attempts. If still failing, surface to user.
- `change.already_exists` → suggest continuing the existing change (`/speclink-ingest <name>` if state is past proposing) or pick a different kebab-case name.
- `artifact.validation_failed` → engine returns specific errors in JSON; fix content and retry.
- `artifact.already_exists` → use `--overwrite` to rewrite the existing artifact body intentional, or pick a different name.
- `state.transition_invalid` → state machine refused; check current state via `speclink status` and follow the correct skill.
- `schema.not_found` → if `--schema <id>` was passed, the schema is not installed; list available via `speclink schemas --json`.
- `config.malformed` → surface engine warnings (warnings[]) to the user; never auto-modify config.
- `project.not_initialized` → stop; ask user to run `speclink init <project-name>`.

---

## Guardrails

- Create all artifacts needed to leave `proposing` state. Optional artifacts (not in `applyRequires` dependency chain) may be skipped if their criteria don't apply.
- Always read dependency artifacts before creating a new one.
- If context is critically unclear, ask the user — but prefer making reasonable decisions to keep momentum.
- If a change with that name already exists, suggest continuing that change instead.
- Verify each artifact file exists after writing before proceeding to the next.
- **NEVER** write application code or implement features during this workflow.
- **NEVER** skip the artifact workflow to write code directly.
- **NEVER** reinterpret requirements by ignoring the proposal file.
- **NEVER** invoke `/speclink-apply` — this workflow ends after artifact creation and state transition. The user decides when to start implementation.
- **NEVER** auto-approve review — `speclink review approve` requires a human reviewer id and explicit decision.
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.
