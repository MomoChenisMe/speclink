# speclink-verify Workflow

Verify that an implementation matches the change artifacts (specs, tasks, design). This is a **QA review skill** — primarily AI-driven; engine only provides change context. Use it before approving code review or before archive.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` or `bindings/tool.md`.

All operations are referenced by their **canonical id**. See `doc/protocol/operations.md` for full operation specs.

---

## Input

Optionally specify a change name after `/speclink-verify` (e.g., `/speclink-verify add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

## Prerequisites

This skill requires:

- A working SpecLink invocation surface (CLI / SDK).
- Your AI tool's file-search facility (grep / glob / read) to inspect the codebase.

If any operation fails with `cli.not_found` / `sdk.not_initialized`, report and STOP.

---

## Steps

### Step 1: Select the change

If no change name provided, invoke `change.list` and ask the user to choose. Show changes that have a `tasks` artifact written (verifying makes no sense before tasks exist).

Mark changes with incomplete tasks as "(In Progress)" so the user knows verification will report missing work.

**IMPORTANT**: Do NOT guess or auto-select unless context unambiguously points to one change.

### Step 2: Load change context

Invoke `change.show` with `{ change_id }` to get:

- `state` (must be `in_progress` / `code_reviewing` / `archived` to make sense — earlier states have no implementation to verify)
- `schema_id`
- `artifacts[]` — which artifacts exist
- `tasks` — `{ total, done }`

If state is `proposing` / `reviewing` / `ready`: STOP. Tell the user "No implementation to verify yet. Run `/speclink-apply <name>` first."

Invoke `instructions.get` with `{ kind: "apply", change_id }` to retrieve `contextFiles[]` (artifact paths to read).

### Step 3: Initialize verification report structure

Create a three-dimension report:

- **Completeness**: are all tasks done? All requirements implemented?
- **Correctness**: does the implementation actually match what specs require?
- **Coherence**: does the implementation follow design decisions + project patterns?

Each dimension can have **CRITICAL** / **WARNING** / **SUGGESTION** issues.

### Step 4: Verify Completeness

**Task completion**:

- Invoke `artifact.read` with `{ change_id, kind: "tasks" }` to fetch tasks.md.
- Parse checkboxes: `- [ ]` (incomplete) vs `- [x]` (complete).
- Count complete vs total.
- For each incomplete task:
  - Add **CRITICAL**: "Task not done: <description>"
  - Recommendation: "Complete the task, OR mark as done if already implemented and confirmed."

**Spec coverage**:

- For each delta spec capability in `artifacts[]`, invoke `artifact.read` with `{ change_id, kind: "spec", capability }`.
- Extract requirements (marked `### Requirement:`).
- For each requirement:
  - Use your AI tool's file-search facility to search the codebase for keywords related to the requirement.
  - Assess whether implementation likely exists.
- For requirements appearing unimplemented:
  - Add **CRITICAL**: "Requirement not found: <name>"
  - Recommendation: "Implement requirement X (<description>) in <suggested file path>"

### Step 5: Verify Correctness

**Requirement-to-implementation mapping**:

- For each requirement from delta specs:
  - Search codebase for implementation evidence (function names, exports, route definitions, etc.)
  - When found, note file paths + line ranges.
  - Assess whether implementation matches the requirement's intent (not just keyword match).
- For divergences:
  - Add **WARNING**: "Implementation may diverge from spec: <details>"
  - Recommendation: "Review `<file>:<lines>` against requirement X — specifically <what to check>"

**Scenario coverage**:

- For each scenario in delta specs (marked `#### Scenario:`):
  - Check whether conditions are handled in code.
  - Check whether tests exist that cover the scenario.
- For uncovered scenarios:
  - Add **WARNING**: "Scenario not covered: <name>"
  - Recommendation: "Add test or implementation for scenario: <description>"

**Example traceability** (specs with `##### Example:` blocks):

- For each example with GIVEN/WHEN/THEN values:
  - Check whether a test uses the same input values.
  - If the example has a parameterized table, check whether tests cover all rows.
- For untested examples:
  - Add **WARNING**: "Spec example not covered by test: <name>"
  - Recommendation: "Add test using GIVEN/WHEN/THEN from example"

### Step 6: Verify Coherence

**Design adherence**:

- If `artifacts[]` includes `design`, invoke `artifact.read` with `{ change_id, kind: "design" }`.
- Extract key decisions (sections like "Decision:", "Approach:", "Architecture:").
- Verify implementation follows those decisions.
- For contradictions:
  - Add **WARNING**: "Design decision not followed: <decision>"
  - Recommendation: "Update implementation OR revise design.md to match reality (then re-ingest via `/speclink-ingest`)"
- If no design.md: skip this check, note "No design.md to verify against."

**Code pattern consistency**:

- Review new code for consistency with project conventions (file naming, directory structure, coding style).
- For significant deviations:
  - Add **SUGGESTION**: "Code pattern deviation: <details>"
  - Recommendation: "Consider following project pattern: <example>"

### Step 7: Generate verification report

**Summary scorecard**:

```
## Verification Report: <change-name>

### Summary
| Dimension    | Result                       |
|--------------|------------------------------|
| Completeness | X/Y tasks done, N reqs found |
| Correctness  | M/N reqs covered             |
| Coherence    | Followed / Issues            |
```

**Issues grouped by severity**:

1. **CRITICAL** (must fix before code review approve / archive)
2. **WARNING** (should fix)
3. **SUGGESTION** (nice to fix)

Each issue must have:

- Specific location (file path + line range when applicable)
- Actionable recommendation (no "consider reviewing")

**Final assessment**:

- **CRITICAL issues exist**: "X critical issue(s) found. Fix before approving code review or archiving."
- **Only warnings**: "No critical issues. Y warning(s) to consider. Ready for review approval (with noted improvements)."
- **All clear**: "All checks passed. Ready for code review approval / archive."

### Step 8: Recommend next step

Based on assessment:

- **Critical issues** → "Run `/speclink-apply <name>` to address critical findings (or `/speclink-ingest` if specs need updating)."
- **No critical issues, current state `in_progress`** → "All apply tasks complete and verified. When ready, finalize tasks → engine auto-transitions to `code_reviewing`."
- **No critical issues, current state `code_reviewing`** → "Reviewer can invoke `review.approve` with `phase: 'code'`. Then run `/speclink-archive <name>`."
- **No critical issues, current state `archived`** → "Verification post-hoc; no action needed."

---

## Verification Heuristics

- **Completeness**: focus on objective checklist items (task checkboxes, requirements list). Less room for interpretation.
- **Correctness**: use keyword search + file-path analysis + reasonable inference. Don't require perfect certainty.
- **Coherence**: look for glaring inconsistencies; don't nitpick style.
- **False positive avoidance**: when uncertain, prefer SUGGESTION over WARNING, WARNING over CRITICAL.
- **Actionability**: every issue MUST have a specific recommendation with file/line references where applicable.

## Graceful Degradation

- **Only tasks.md exists**: verify task completion only; skip spec / design checks. Note "Limited verification — no specs to check against."
- **tasks + specs exist (no design)**: verify completeness and correctness; skip coherence. Note "No design.md — coherence check skipped."
- **Full artifacts**: verify all three dimensions.
- **No artifacts at all (proposing state)**: STOP at Step 2.

---

## Concurrency & Errors

- `change.not_found` → list candidates and re-prompt.
- `artifact.not_found` (specific kind) → record as "missing artifact" finding and continue with available ones.
- `provider.connection_failed` → engine retried with backoff; surface to user.
- `state.transition_invalid` → unlikely; verify is read-only.

---

## Exit Criteria

The skill ends when ONE of:

- Verification report generated + user informed.
- Empty-state (no change in verifiable state) → user dismissed.
- Unrecoverable error → user informed.

---

## Guardrails

- **Read-only** — NEVER modify artifacts or code. Fixes go to `/speclink-apply` or `/speclink-ingest`.
- **Don't auto-approve** — this skill does NOT invoke `review.approve`. Reviewer must explicitly approve.
- **Don't fake findings** — if codebase search is inconclusive, surface that uncertainty as a SUGGESTION, not a fabricated CRITICAL.
- **Don't run inside `proposing` / `reviewing` / `ready` states** — no implementation to verify yet.
- **Concise output** — group findings; don't dump raw search results.

---

## Output Format

Use clear markdown with:

- Table for summary scorecard
- Grouped lists for issues (CRITICAL / WARNING / SUGGESTION)
- Code references in format `file.ts:123` or `file.ts:120-140`
- Specific, actionable recommendations
- No vague suggestions like "consider reviewing"
