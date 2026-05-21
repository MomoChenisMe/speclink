# speclink-propose Workflow

Create a complete SpecLink change — from requirement to validated artifacts — in a single workflow. The change ends in `reviewing` state (or `ready` if review is disabled), ready for human review or direct apply.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` (CLI subprocess hosts) or `bindings/tool.md` (typed Tool hosts). At deploy time, engine concatenates this workflow with the chosen binding to produce the final SKILL.md.

All operations are referenced by their **canonical id** (e.g., `change.create`). See `doc/protocol/operations.md` for full operation specs.

---

## Input

The argument after `/speclink-propose` is the requirement description. Examples:

- `/speclink-propose add dark mode`
- `/speclink-propose fix the login page crash`
- `/speclink-propose improve search performance`

If no argument is provided, the workflow will look for:

1. A converged discussion with `Capture to: change <name>` matching the topic
2. A plan file referenced by your AI tool's conversation context (e.g., a "plan mode" file path)
3. Relevant requirements from conversation context

If none of the above yields a clear requirement, ask the user.

## Prerequisites

This skill requires a working SpecLink invocation surface (CLI binary or SDK client). If any operation fails with `cli.not_found` / `sdk.not_initialized` or similar, report the error and STOP.

Also verify the project is initialized — invoke `project.status`. If it reports `project.not_initialized`, ask the user to initialize before retrying (`project.init` is not invoked from this workflow; the user must run it explicitly).

---

## Steps

### Step 1: Determine the requirement source

**1a. Argument provided** (e.g., "add dark mode") → use it as the requirement description, skip to deriving the change name below.

**1b. Linked discussion**:

- Invoke `discuss.list` with `{ include_converged: true }` to find converged discussions
- If a discussion's `Capture to:` references a change matching the topic, invoke `discuss.show` with `{ discussion_id }`
- Use the discussion's Background, Decisions Made, and Conclusion as the proposal source
- Ask the user to confirm: use this discussion, or override with explicit input

**1c. Plan file available** (if your AI tool exposes one):

- Check conversation context for a plan file path
- If found and accessible, read it via your AI tool's file-read facility and extract:
  - Title → requirement description
  - Context section → proposal Why/Motivation
  - Implementation stages → artifact creation
  - File paths mentioned → Impact section
- Ask the user to confirm using the plan file vs conversation context

**1d. Conversation context** → attempt to extract requirements from conversation history. If context is insufficient, ask the user what they want to build.

From the resolved description, derive a kebab-case change name (e.g., "add dark mode" → `add-dark-mode`). Strip archive-style date prefixes if present (e.g., `2026-05-20-foo` → `foo`).

**IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

### Step 2: Classify the change type

Based on the requirement, classify into one of three types:

| Type     | When to use                                                         |
| -------- | ------------------------------------------------------------------- |
| Feature  | New functionality, new capabilities                                 |
| Bug Fix  | Fixing existing behavior, resolving errors                          |
| Refactor | Architecture improvements, performance optimization, UI adjustments |

This determines the proposal template format in Step 5.

### Step 3: Scan existing specs and changes for relevance

Before creating the change, check if anything overlaps:

- Invoke `spec.list` to enumerate capabilities with canonical specs
- Invoke `change.list` to enumerate existing changes

For each candidate (max 3), invoke `spec.show` (with `{ capability }`) or `change.show` (with `{ change_id }`) to retrieve the Purpose section.

**IMPORTANT**:
- If related specs / changes are found, display them as an informational summary but do NOT stop or ask for confirmation — continue to the next step.
- If nothing related is found, silently proceed without mentioning the scan.

### Step 4: Create the change

Invoke `change.create` with `{ name, description, schema_id? }`.

- Default schema is the one configured in `config.yaml#schema` (typically `spec-driven`).
- Pass `schema_id` only if the user explicitly wants a non-default workflow schema.
- If a change with that name already exists (`change.duplicate_id`), suggest continuing the existing change (via `/speclink-ingest`) or picking a different kebab-case name.

The change is now in state `proposing`.

### Step 5: Write the proposal

**IMPORTANT — file path rules for the `## Impact` section:**

- All file paths SHALL be written relative to the project root (e.g., `crates/speclink-core/src/foo.rs`, `doc/skill-drafts/speclink-propose/workflow.md`).
- Do NOT use relative fragments (e.g., `core/mod.rs`, `parser/lib.rs`) — preflight rejects them as non-anchored paths.
- Do NOT wrap shell commands in backticks inside artifact text — the backtick extractor may otherwise mis-parse the command as a file reference.
- When referring to a file without naming its concrete path, use descriptive prose ("the engine entry module") rather than a backticked path fragment.

Invoke `instructions.get` with `{ kind: "proposal", change_id }`. The returned content describes the proposal template and rules.

Generate the proposal content based on change type (see formats below), then invoke `artifact.write` with `{ change_id, kind: "proposal", content }`.

If `artifact.write` fails with a validation error, fix the content and retry.

#### Feature

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

#### Bug Fix

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

#### Refactor / Enhancement

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

### Step 6: Get the artifact build order

Invoke `change.show` with `{ change_id }` and parse the result for:

- `apply_requires`: array of artifact ids needed before the change can leave `proposing` state (varies by schema)
- `artifacts`: list of all artifacts (from the change's schema) with their status and dependencies

> **Note**: `change.show` Output schema may extend to include `apply_requires`; if absent, derive from `schema.show` of the active schema. This is an MVP gap to reconcile.

**Note**: The artifact list is **schema-driven**. The default `spec-driven` schema produces `proposal` / `spec` (multi) / `design` (optional) / `tasks`. A custom schema may differ — never assume a hardcoded artifact set.

### Step 7: Create remaining artifacts in sequence

Loop through artifacts in dependency order (skip proposal since it's already done):

**7a. For each artifact that is `ready` (dependencies satisfied)**:

- **Check if the artifact is optional**: If the artifact is NOT in the dependency chain of any `apply_requires` artifact (i.e., removing it would not block the state transition out of `proposing`), it is optional. Invoke `instructions.get` for the artifact and read the `instruction` field. If the instruction contains conditional criteria ("create only if..."), evaluate whether any criteria apply to this change based on the proposal content. If none apply, skip and show: "⊘ Skipped <artifact-id> (not needed for this change)". Continue to next artifact.

- Invoke `instructions.get` with `{ kind: <artifact-id>, change_id }`. The result includes:
  - `context`: Project background from `config.yaml#context` (constraints for you — do NOT include in output)
  - `rules`: Artifact-specific rules from `config.yaml#rules.<artifact>` (constraints for you — do NOT include in output)
  - `template`: The structure to use for your output file
  - `instruction`: Schema-specific guidance
  - `output_path`: Where the artifact will be written
  - `dependencies`: Completed artifacts you should read for context
  - `locale`: Language for the artifact (e.g., "Traditional Chinese (繁體中文)"). If present, write the artifact body in this language. **Exception**: spec files MUST always be written in English regardless of locale, because they use normative language (SHALL/MUST).

- Read any completed dependency files for context — invoke `artifact.read` with `{ change_id, kind: <dep-id> }` for each dependency.

- Generate the artifact content using `template` as the structure.

- Apply `context` and `rules` as constraints — but do NOT copy them into the file.

- Invoke `artifact.write` with `{ change_id, kind: <artifact-id>, content, capability? }`:
  - For **design** or **tasks** (single-instance artifacts): no `capability` parameter
  - For **spec** (one invocation per capability — `spec` is `multi: true`): pass `capability: <capability-name>`

  If the invocation fails with a validation error (engine auto-validates each write), fix the content and retry.

- Show brief progress: "✓ Created <artifact-id>"

**7b. Continue until all `apply_requires` artifacts are complete**:

- After creating each artifact, invoke `change.show` and check if every artifact id in `apply_requires` has `status: "done"`.
- Stop when all `apply_requires` artifacts are done.

**7c. If an artifact requires user input** (unclear context):

- Ask the user to clarify the specific question.
- Then continue with creation.

### Step 8: Inline Self-Review (before letting the engine auto-validate)

After creating all artifacts, scan them manually. Fix issues inline.

#### Check 1: No Placeholders

These patterns are artifact failures — fix each one before proceeding:

- "TBD", "TODO", "FIXME", "implement later", "details to follow"
- Vague instructions: "Add appropriate error handling", "Handle edge cases", "Write tests for the above"
- Delegation by reference: "Similar to Task N" without repeating specifics
- Steps describing WHAT without HOW: "Implement the authentication flow" (what flow? what steps?)
- Empty template sections left unfilled
- Weasel quantities: "some", "various", "several" when a specific number or list is needed

#### Check 2: Internal Consistency

- Does every capability in the proposal have a corresponding spec?
- Does the design reference only capabilities from the proposal?
- Do tasks cover all design decisions, and nothing outside proposal scope?
- Are file paths consistent across proposal Impact, design, and tasks?

#### Check 3: Scope Check

- More than 15 pending tasks → consider decomposing into multiple changes
- Any single task would take more than 1 hour → split it
- Touches more than 3 unrelated subsystems → consider splitting

#### Check 4: Ambiguity Check

- Are success/failure conditions testable and specific?
- Are boundary conditions defined (empty input, max limits, error cases)?
- Could "the system" refer to multiple components? Be explicit.

#### Check 5: Durable Handoff Review

This change must survive being handed to another agent / role / week. Reject and fix:

- **File-path-only tasks**: a task whose entire description is "edit file X" with no behavior, contract, or verification target. File paths are locator context — the task SHALL still describe what is observably true when complete.
- **Line-number-coupled instructions**: design or tasks content that points to "line 42" / "the function on lines 80-95" as the only way to identify the work. Source line numbers drift; name the function, command, struct, or behavior instead.
- **Vague acceptance criteria**: success conditions like "works correctly", "behaves as expected", "handles edge cases" without naming the observable behavior or the verification target (test name, CLI invocation, analyzer rule, manual assertion).
- **Missing scope boundaries on non-trivial work**: design lacking explicit "in scope" / "out of scope" lines for any change that touches more than one subsystem or introduces new behavior. Trivial artifact-only edits MAY skip this; runtime, build, or tooling effects MUST NOT.

Fix every failure inline using the existing context. If a failure cannot be fixed without new input from the user, surface it explicitly rather than papering over it.

### Step 9: Analyze-Fix Loop (max 2 iterations)

Engine auto-validates each `artifact.write` for structural correctness. **This step always runs** for cross-artifact consistency analysis (Coverage / Consistency / Ambiguity / Gaps):

1. Invoke `analyze.run` with `{ change_id }`
2. Filter findings to **Critical and Warning only** (ignore Suggestion)
3. If no Critical/Warning findings → show "Artifacts look consistent ✓" and proceed
4. If Critical/Warning findings exist:
   - Show: "Found N issue(s), fixing... (attempt M/2)"
   - Fix each finding in the affected artifact (re-invoke `artifact.write` with `overwrite: true` if needed)
   - Re-invoke `analyze.run`
   - Repeat up to 2 total iterations
5. After 2 attempts, if findings remain:
   - Show remaining findings as a summary
   - Proceed normally (do NOT block — Step 10 validation still gates state transition)

**Note**: this in-skill analyze is **programmatic only** (uses `analyze.run` engine op). The deeper AI semantic analysis (design vs spec contradictions, scope drift, risk gaps) is the responsibility of `/speclink-analyze` skill, triggered separately by the **Passive Trigger Contract** at end of this workflow.

### Step 10: Validation and state transition

Invoke `validate.run` with `{ change_id, strict: true }`.

If validation fails, fix errors and re-invoke.

Once validation passes, the engine automatically transitions the change out of `proposing`:

- If `config.yaml#require_artifact_review: true` → state becomes `reviewing`
- If `false` → state becomes `ready`

Invoke `change.show` to confirm the resulting state.

**Show summary**:

- Change name and location (`.speclink/changes/<name>/` for LocalProvider)
- List of artifacts created
- Validation result
- Current state (`reviewing` or `ready`)
- Linked discussion id (if sourced from a discussion) — note: discussion is preserved as converged; engine added this change to its `linked_changes`. Do NOT delete the discussion as part of this workflow. If the user later wants to remove it, they can invoke `discuss.delete` explicitly.

**Tell the user the next step based on state**:

- State = `reviewing` → "Change is awaiting review. When ready, a reviewer approves via `review.approve` with `phase: 'artifact'`. Then run `/speclink-apply <name>` to begin implementation."
- State = `ready` → "Change is ready for implementation. Run `/speclink-apply <name>` to begin."

The propose workflow ENDS here. Do NOT invoke `/speclink-apply`. Do NOT auto-approve review. The user decides when to start implementation and (if required) when to approve.

---

## Passive Trigger Contract

After this workflow completes successfully (`change.show.value.is_complete == true` AND state ∈ `{reviewing, ready}`), the orchestrating AI host SHOULD invoke `/speclink-analyze <name>` automatically before recommending `/speclink-apply`. This:

- Adds the AI semantic layer (design vs spec contradictions, scope drift, risk gaps) on top of the programmatic analysis done in Step 9
- Mirrors spectra-propose's passive trigger pattern where `/spectra-analyze` runs after artifacts are complete

The contract is **advisory** — `/speclink-analyze` is independent and read-only. Skipping it is acceptable; running it is recommended.

---

## Rationalization Table

| What You're Thinking                                          | What You Should Do                                                                    |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| "The requirements are clear enough, no need for discuss"      | Fine if true — but check you're not skipping because you're lazy                      |
| "This artifact isn't needed for this change"                  | Check `apply_requires` — if it's in the dependency chain, create it                   |
| "The spec doesn't need scenarios, the requirement is obvious" | Obvious to you now. Write scenarios for the implementer who doesn't have your context |
| "I'll keep the design brief, code will be self-explanatory"   | Design exists so implementers don't reverse-engineer intent. Be specific              |
| "This is a small change, skip the scope check"                | Small changes touching 5 subsystems aren't small. Check                               |
| "The placeholder is fine for now, I'll fill it in later"      | There is no "later" — implementation is next. Fill it in now                          |

---

## Artifact Creation Guidelines

- Follow the `instruction` field from `instructions.get` for each artifact type.
- Read dependency artifacts for context before creating new ones (via `artifact.read`).
- Use `template` as the structure for your output file — fill in its sections.
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file.
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact.
  - These guide what you write, but should never appear in the output.
- **Parallel task markers (`[P]`)**: When creating the **tasks** artifact, first invoke `config.read` and check `config.parallel_tasks`. If `true`, add `[P]` markers to tasks that can be executed in parallel. Format: `- [ ] [P] Task description`. A task qualifies for `[P]` if it targets different files from other pending tasks AND has no dependency on incomplete tasks in the same group. When `parallel_tasks` is not enabled, do NOT add `[P]` markers.
- **TDD task ordering**: When `config.tdd: true`, format tasks in the TDD order: write test (red) → implement (green) → refactor. Use sub-numbering like `1.1`, `1.2`, `1.3` for the red/green/refactor triplet.

---

## Concurrency & Errors

- `lock.not_acquired` (another agent is writing to the same change) → engine handles jittered backoff retry (~4 attempts, ~7s worst case). If still failing, surface to user.
- `change.duplicate_id` → suggest continuing the existing change (`/speclink-ingest <name>` if state is past proposing) or pick a different kebab-case name.
- `validation.*` (any validation error from `artifact.write` auto-validate) → engine returns specific errors in the JSON envelope; fix content and retry.
- `artifact.already_exists` → pass `overwrite: true` to rewrite the existing artifact body intentionally, or pick a different artifact kind.
- `state.transition_invalid` → state machine refused; invoke `change.show` to inspect current state and follow the correct skill.
- `schema.not_found` → if `schema_id` was passed, the schema is not installed; invoke `schema.list` to enumerate available.
- `config.malformed` → surface engine warnings (warnings[]) to the user; never auto-modify config.
- `project.not_initialized` → stop; ask user to invoke `project.init` with their project name.
- `state.etag_mismatch` (concurrent edit to artifact) → re-invoke `artifact.read` to get latest content, merge changes, re-invoke `artifact.write` with the new etag. Engine guides via `read-then-retry` semantics (design.md §12.5.4).

---

## Guardrails

- Create all artifacts needed to leave `proposing` state. Optional artifacts (not in `apply_requires` dependency chain) may be skipped if their criteria don't apply.
- Always read dependency artifacts before creating a new one.
- If context is critically unclear, ask the user — but prefer making reasonable decisions to keep momentum.
- If a change with that name already exists, suggest continuing that change instead.
- Verify each artifact was written successfully (invocation returned `ok: true`) before proceeding to the next.
- **NEVER** write application code or implement features during this workflow.
- **NEVER** skip the artifact workflow to write code directly.
- **NEVER** reinterpret requirements by ignoring the proposal file.
- **NEVER** invoke `/speclink-apply` — this workflow ends after artifact creation and state transition. The user decides when to start implementation.
- **NEVER** auto-approve review — `review.approve` requires a human reviewer id and explicit decision.
- **NEVER** pass `force: true` to destructive operations (the AI must not bypass safety checks).
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.
