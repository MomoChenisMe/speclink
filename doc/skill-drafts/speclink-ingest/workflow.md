# speclink-ingest Workflow

Update an existing SpecLink change — from a converged discussion (`Capture to: ingest <change>`), a plan file, or conversation context. Used when requirements shift mid-implementation or after artifact review surfaces gaps that require re-shaping artifacts.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` or `bindings/tool.md`. All operations are referenced by their **canonical id** (see `doc/protocol/operations.md`).

**Source priority**:

1. **Linked discussion** — a discussion with `state: converged` and `Capture to: ingest <change>` matching the target change. This is the cleanest source because the discussion already structured the change reasoning.
2. **Plan file** — if your AI tool exposes a plan file path in conversation context, use it.
3. **Conversation context** — fall back if neither is available.

---

## Input

Optionally specify the change name. Examples:

- `/speclink-ingest add-auth` — target change name
- `/speclink-ingest add-auth --from-discussion sse-vs-websocket` — use a specific discussion as source
- `/speclink-ingest` — infer everything from context

## Prerequisites

This skill requires a working SpecLink invocation surface. If any operation fails with `cli.not_found` / `sdk.not_initialized` or similar, report the error and STOP. Also verify the project is initialized — invoke `project.status`.

---

## Steps

### Step 1: Locate the source context

**1a. `--from-discussion <topic-id>` provided** → invoke `discuss.show` with `{ discussion_id }`:

- If found and `state == "converged"` → use its content as the source. Proceed to Step 2.
- If `state == "active"` → STOP. Tell the user the discussion hasn't converged yet; finish it first via `/speclink-discuss <topic-id>` then invoke `discuss.conclude`.
- If not found → report error and STOP.

**1b. No `--from-discussion`, check for matching converged discussions**:

Invoke `discuss.list` with `{ include_converged: true, linked_change_id: <change-name> }`.

- If exactly one converged discussion with `Capture to: ingest <change-name>` found → ask the user "Use this discussion as source?" (default yes).
- If multiple → ask the user to pick.
- If none → proceed to (c).

**1c. Plan file (if your AI tool supports one)**:

- Check conversation context for a plan file path. If found and accessible, use it via your AI tool's file-read facility.
- Plan file content typically contains: Title, Context, Stages/Steps, Files, Verification.
- If conversation references it but the file doesn't exist → report error and STOP.

**1d. Conversation context fallback**:

- Use conversation history to extract new context.
- If conversation context is insufficient, ask the user for more details rather than inventing content.
- Warn: "No discussion or plan file found. Using conversation context."

### Step 2: Parse the source structure (skip if conversation context)

Extract from the chosen source:

- **From a discussion**:
  - `Background` → maps to proposal Why / context
  - `Decisions Made` → maps to proposal What / design Decisions
  - `Open Questions` (resolved ones) → maps to spec scenarios or tasks
  - `Conclusion.Decision` + `Conclusion.Rationale` → maps to proposal summary or design Context
  - `linked_changes` → confirm target change matches
- **From a plan file**:
  - `plan_title` (H1) → confirm change name alignment
  - `plan_context` → maps to proposal Why
  - `plan_stages` → maps to tasks.md grouped sections
  - `plan_files` → maps to proposal Impact
  - `plan_verification` → maps to tasks.md verification group

### Step 3: Check for active changes (REQUIRED — ingest only updates existing changes)

Invoke `change.list` to surface candidates.

- If no changes exist at all → tell the user: "No active change to ingest. Use `/speclink-propose` first to create one." and STOP.
- If exactly one change → ask the user to confirm.
- If multiple → ask the user to pick.

**IMPORTANT**: Ingest never creates a new change. If the source describes work that doesn't fit any existing change, suggest `/speclink-propose` instead.

### Step 4: Select the change and check state

Invoke `change.show` with `{ change_id }`.

**State handling** (ingest is valid from multiple states, but with different implications):

- `state: "proposing"` → ingest acts like a continuation of propose work. Free to modify any artifact. State stays `proposing`; auto-validate after writes may move it to `reviewing` if all required artifacts are clean.
- `state: "reviewing"` → ingest is **unusual** at this state (artifacts are awaiting review). Warn the user: "Ingest will invalidate the pending artifact review approval. Proceed?" If yes, continue but note in summary.
- `state: "ready"` → ingest is fine; modifying artifacts may invalidate the prior artifact review approval. Warn the user.
- `state: "in_progress"` → ingest is the typical mid-implementation refresh path. **Preserve completed tasks** (Preservation Check in Step 6). Adding new tasks is fine; modifying proposal/spec/design is fine but may surface scope-change implications (mention in summary).
- `state: "code_reviewing"` → unusual; normally rejection auto-retreats to `in_progress`. If somehow ingest is needed at code_reviewing, ask user to invoke `review.reject` + re-shape via the normal feedback loop instead.
- `state: "archived"` → STOP. Cannot ingest into an archived change.

**Important — speclink does NOT auto-retreat state on ingest**. If your ingest substantially changes proposal/spec scope, the prior artifact review approval is invalidated implicitly, but state remains where it was. The reviewer should re-review and re-approve if needed. State retreat (e.g., back to `proposing`) is not in MVP scope.

**Read existing artifacts for context** before updating:

- The `change.show` result includes the `artifacts[]` summary.
- For each artifact you intend to modify, invoke `artifact.read` with `{ change_id, kind, capability? }` to get the current body + etag.

### Step 5: Update artifacts

For each artifact you intend to update, invoke `instructions.get` with `{ kind: <artifact-id>, change_id }` to get the template / rules / locale.

Use the `template` from instructions as the output structure. Apply `context` and `rules` as constraints but do NOT copy them into the file.

The instructions JSON includes `locale` — the language to write artifacts in. **Exception**: spec files MUST always be written in English regardless of locale, because they use normative language (SHALL/MUST).

#### Discussion-to-Artifact Mapping (when using a converged discussion)

| Discussion Section | Artifact | How to Map |
| --- | --- | --- |
| Background | proposal: Why (or design: Context) | Direct content transfer; merge into existing |
| Decisions Made | proposal: What / design: Decisions | Convert decision bullets to artifact prose |
| Open Questions (resolved) | spec scenarios / tasks | Each resolved question becomes a scenario or task |
| Conclusion.Decision | proposal: What / change name validation | Summarize as one-line change purpose |
| Conclusion.Rationale | design: Context / design: Decisions | Captures "why this approach" |
| Linked specs | proposal: Impact (Affected specs) | Cross-reference for capability impact |

#### Plan-to-Artifact Mapping (when using a plan file)

| Plan Section | Artifact | How to Map |
| --- | --- | --- |
| Title | Change name (validate matches existing) | Convert to kebab-case; if mismatch, STOP and ask |
| Context | proposal: Why | Direct content transfer; merge |
| Stages overview | proposal: What | Summarize all stages |
| Individual stages | tasks.md groups | One stage = one `##` heading, sub-items = `- [ ]` |
| File paths | proposal: Impact | Affected code list |
| Verification steps | tasks.md | Final verification task group |

#### Context-to-Artifact Mapping (when using conversation context)

| Conversation Element | Artifact | How to Map |
| --- | --- | --- |
| Goal / requirement | proposal: Why | Extract motivation from discussion |
| Discussed approach | proposal: What | Summarize agreed approach |
| Mentioned files | proposal: Impact | Affected code list |
| Discussion phases | tasks.md groups | One topic = one `##` heading |

#### When updating an existing change

- **Merge new context into existing artifacts** — do NOT replace wholesale.
- **Preserve completed tasks (`- [x]`) verbatim** — never revert progress. Add new tasks as additional rows.
- **Preserve existing `[P]` markers** on tasks that still qualify.
- **Preserve synthetic review-feedback tasks** — tasks prefixed `[Review feedback by <reviewer>, <date>]` were auto-appended by engine; if they're still pending, leave them. If they're done, they remain `- [x]`.
- **Do NOT remove existing content** without explicit reason; if removing, justify in the summary.

**Parallel task markers (`[P]`)**: When updating tasks.md, the instructions JSON exposes `config.parallel_tasks`. If `true`, add `[P]` markers to **new** tasks that qualify. If `false`, do NOT add new `[P]` markers — but **preserve any existing `[P]` markers already in the file**.

Write each updated artifact by invoking `artifact.write` with `{ change_id, kind, content, capability?, overwrite: true, expected_etag: <etag from Step 4 read> }`:

- For **proposal / design / tasks** (single-instance): omit `capability`
- For **spec** (one invocation per capability — `spec` is `multi: true`): pass `capability: <capability-name>`

If the invocation fails with a validation error, fix the content and retry.

After writing each artifact, re-invoke `change.show` to refresh status.

Show progress: "✓ Updated <artifact-id>" or "✓ Created <artifact-id>" if it didn't exist before.

### Step 6: Inline Self-Review (before letting the engine auto-validate)

After updating all artifacts, scan them manually. Fix issues inline.

#### Check 1: No Placeholders

- "TBD", "TODO", "FIXME", "implement later", "details to follow"
- Vague: "Add appropriate error handling", "Handle edge cases", "Write tests for the above"
- Delegation by reference: "Similar to Task N" without repeating specifics
- Steps describing WHAT without HOW
- Empty template sections left unfilled
- Weasel quantities: "some", "various", "several" when a specific number/list is needed

#### Check 2: Internal Consistency

- Does every capability in the proposal have a corresponding spec?
- Does the design reference only capabilities from the proposal?
- Do tasks cover all design decisions, and nothing outside proposal scope?
- Are file paths consistent across proposal Impact, design, and tasks?

#### Check 3: Scope Check

- More than 15 pending tasks → consider decomposing into multiple changes.
- Any single task would take more than 1 hour → split it.
- Touches more than 3 unrelated subsystems → consider splitting.

#### Check 4: Ambiguity Check

- Are success/failure conditions testable and specific?
- Are boundary conditions defined (empty input, max limits, error cases)?
- Could "the system" refer to multiple components? Be explicit.

#### Check 5: Preservation Check (ingest-specific)

- Are all completed tasks `[x]` still present and unchanged?
- Were existing `[P]` markers preserved on tasks that still qualify?
- Were synthetic review-feedback tasks (`[Review feedback by ...]`) preserved as-is?
- Was existing content merged (not replaced)?

#### Check 6: Durable Handoff Review

Reject and fix any of the following on **incomplete** design and task content (do NOT rewrite completed `[x]` tasks):

- **File-path-only tasks**: a pending task whose entire description is "edit file X" with no behavior, contract, or verification target.
- **Line-number-coupled instructions**: design or task content that points to "line 42" / "the function on lines 80-95" as the only way to identify the work. Source line numbers drift; name the function, command, struct, or behavior instead.
- **Vague acceptance criteria**: success conditions like "works correctly", "behaves as expected", "handles edge cases" without naming the observable behavior or the verification target.
- **Missing scope boundaries on non-trivial work**: design lacking explicit "in scope" / "out of scope" lines for any change that touches more than one subsystem or introduces new behavior.

Fix every failure inline using the new source context. Update incomplete design and task content so behavior contracts, verification criteria, and scope boundaries stay current. Preserve completed tasks unchanged.

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
| "I'll rewrite the synthetic review-feedback task to be cleaner"  | Don't. Engine wrote it; reviewer reads it. Leave verbatim                     |

---

### Step 7: Analyze-Fix Loop (max 2 iterations)

Engine auto-validates each `artifact.write`. After all artifacts are updated, run cross-artifact analysis:

1. Invoke `analyze.run` with `{ change_id }`.
2. Filter findings to **Critical and Warning only** (ignore Suggestion).
3. If no Critical/Warning findings → show "Artifacts look consistent ✓" and proceed.
4. If Critical/Warning findings exist:
   - Show: "Found N issue(s), fixing... (attempt M/2)"
   - Fix each finding in the affected artifact (invoke `artifact.write` with `overwrite: true`).
   - Re-invoke `analyze.run`.
   - Repeat up to 2 total iterations.
5. After 2 attempts, if findings remain:
   - Show remaining findings as a summary.
   - Proceed normally (do NOT block).

### Step 8: Validation

Invoke `validate.run` with `{ change_id, strict: true }`.

If validation fails, fix errors and re-invoke.

### Step 9: Summary and next steps

Show:

- **Source used**: discussion (`<topic-id>`), plan file (`<path>`), or conversation context
- **Change name** and current state (after ingest, state has not changed; mention if review approvals are now stale)
- **Artifacts created / updated** (with which ones were modified)
- **Tasks added** (count) and **tasks preserved** (count of `[x]` kept)
- **Validation result**
- **Stale review approvals** (if state was `reviewing` / `ready` / `code_reviewing` and proposal/spec changed substantially): "Prior <phase> review approval is now stale; re-request when ready"

Ask the user to confirm completion. Provide:

- **Done** — End the ingest workflow. Inform the user of next-step options based on state.
- **Apply** — Invoke `/speclink-apply <change-name>` to continue implementation.

Suggest next based on state:

- `state: "proposing"` → "Continue building artifacts via `/speclink-propose <change-name>` or re-run ingest if more context needed."
- `state: "reviewing"` → "Reviewer should re-approve via `review.approve` with `phase: artifact`."
- `state: "ready"` → "Run `/speclink-apply <name>` when ready."
- `state: "in_progress"` → "Run `/speclink-apply <name>` to continue implementation (address new / synthetic tasks)."

After the user responds, if "Done", workflow is OVER. If "Apply", invoke `/speclink-apply <change-name>`.

---

## Concurrency & Errors

- `lock.not_acquired` (another agent is writing) → engine handles jittered backoff retry.
- `change.not_found` → suggest `change.list` to surface candidates.
- `discussion.not_found` (when `--from-discussion` doesn't match) → suggest `discuss.list` with `include_converged: true`.
- `discussion.active` (when the linked discussion hasn't converged) → STOP; suggest finishing the discussion first.
- `state.transition_invalid` → ingest is not valid at this state (e.g., archived); follow Step 4 state table.
- `validation.*` → engine returns specific errors; fix content and retry.
- `state.etag_mismatch` → re-invoke `artifact.read` to get latest etag, merge changes, re-invoke `artifact.write`. Engine guides via `read-then-retry`.
- `config.malformed` → surface engine warnings to user; never auto-modify config.
- `project.not_initialized` → stop; ask user to invoke `project.init`.

---

## Guardrails

- **NEVER** create new changes — ingest only updates existing changes. If no active change exists or source doesn't fit any, direct user to `/speclink-propose`.
- **NEVER** modify or delete the original plan file or discussion document.
- **NEVER** write application code — this skill only creates/updates SpecLink artifacts.
- **NEVER** remove completed tasks (`[x]`) — preserve progress verbatim.
- **NEVER** rewrite synthetic review-feedback tasks (`[Review feedback by ...]`) — engine wrote them, reviewer reads them; leave verbatim.
- **NEVER** auto-retreat state (e.g., back to `proposing`) — state retreat is not in MVP scope; if scope changes substantially, surface the implication in summary and let the user decide.
- **NEVER** pass `force: true` to destructive operations.
- If source content is too brief to fill artifact sections, ask the user for more details rather than inventing content.
- If `speclink` CLI / SDK is not available, report the error and stop.
- Verify each artifact was written successfully (invocation returned `ok: true`) before proceeding to next.
- **NEVER** skip the artifact workflow to write code directly.
- If a structured-question facility is not available, ask the same questions as plain text and wait for the user's response.

---

## Fluid Workflow Integration

This skill is the **state-preserving alternative to re-propose** — when requirements shift mid-flight, ingest keeps the existing change alive instead of forcing a brand new one. It works at most states (`proposing` / `ready` / `in_progress`) and is the typical follow-up after `/speclink-discuss --about <change>`.

- **Typical entry points**:
  - From a converged discussion with `Capture to: ingest <change>` → ingest reads discussion as source.
  - Mid-apply when discovery shifts approach → ingest re-shapes artifacts.
  - After artifact review rejection → not typical (review reject doesn't usually need full ingest; the reviewer's feedback comment may suffice for direct artifact edits). Use ingest only if scope changes.
- **Does NOT**:
  - Change the change's state (no auto-retreat in MVP).
  - Touch linked discussion documents (engine updates `linked_changes` metadata only when applicable).
  - Remove completed tasks or synthetic feedback tasks.
- **DOES**:
  - Update artifacts in place with `overwrite: true`.
  - Surface stale review approvals (if any) in summary.
  - Run engine validate + analyze automatically.
