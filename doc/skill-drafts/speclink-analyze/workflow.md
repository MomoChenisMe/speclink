# speclink-analyze Workflow

Analyze artifact consistency for a SpecLink change. Combines engine programmatic analysis (`analyze.run` op) with AI semantic checks. Can be invoked directly or triggered automatically when all artifacts are complete.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` (CLI subprocess hosts) or `bindings/tool.md` (typed Tool hosts).

All operations are referenced by their **canonical id** (e.g., `analyze.run`). See `doc/protocol/operations.md` for full operation specs.

---

## Input

Optionally specify a change name (e.g., `/speclink-analyze add-auth`). If omitted, infer from conversation context or auto-select if only one active change exists.

## Prerequisites

This skill requires a working SpecLink invocation surface. If any operation fails with `cli.not_found` / `sdk.not_initialized`, report the error and STOP.

---

## Steps

### Step 1: Determine change name

If not provided, infer from context or invoke `change.list` to auto-select.

- **Zero active changes** (none in `proposing` / `reviewing` / `ready` / `in_progress` / `code_reviewing`): show empty-state message + STOP.
- **One active change**: auto-select.
- **Multiple active changes** (fork context): return candidate list, ask the main thread to rerun `/speclink-analyze <change-name>`. Do NOT prompt for selection inside the fork.

### Step 2: Run programmatic analysis

Invoke `analyze.run` with `{ change_id }`.

The op returns structured JSON:

- `dimensions[]`: `{ dimension, status, finding_count }` for Coverage, Consistency, Ambiguity, Gaps
- `findings[]`: `{ id, dimension, severity, location, summary, recommendation }`
- `artifacts_analyzed[]` / `artifacts_missing[]`: which artifacts were available

### Step 3: Present programmatic results

Format the JSON output as a readable summary:

```
## Artifact Analysis: <change-name>

| Dimension     | Status                   |
|---------------|--------------------------|
| Coverage      | <status>                 |
| Consistency   | <status>                 |
| Ambiguity     | <status>                 |
| Gaps          | <status>                 |
```

Group findings by severity (Critical > Warning > Suggestion) with locations and recommendations.

### Step 4: Supplement with AI semantic analysis

The programmatic analyzer (Step 2) catches structural issues (Coverage / Consistency / Ambiguity / Gaps based on artifact-DAG cross-references). For deeper semantic analysis, ALSO read the artifacts and check for:

- **Design vs spec contradiction**: design decisions that contradict spec requirements
- **Scope drift**: tasks referencing work outside proposal scope
- **Risk gap**: design lists risks that have no corresponding spec coverage / mitigation
- **Logical inconsistency**: requirements / scenarios that contradict each other
- **Spec example untested**: spec `##### Example:` rows with GIVEN/WHEN/THEN values but no corresponding task that creates a test using those values

Use `artifact.read` with `{ change_id, kind }` to fetch each artifact body. For spec artifacts, optionally also invoke `spec.show` to compare delta against canonical (when applicable).

Add any additional semantic findings to the report under their own subsection:

```
### AI Semantic Findings (in addition to programmatic)

- **CRITICAL** — design.md decides X, but spec requires Y (contradiction)
  - Location: design.md "Approach" section ↔ specs/<cap>/spec.md requirement "Z"
  - Recommendation: align design with spec, OR update spec to reflect design intent
```

### Step 5: Recommend next step

- **Critical findings exist** (programmatic OR semantic): "Found N critical issue(s) worth addressing. Want to fix these before implementing?"
- **Only warnings / suggestions**: note them briefly, then recommend proceeding with `/speclink-apply <name>`.
- **Clean (no critical / warning)**: "Artifacts look consistent ✓" and recommend `/speclink-apply <name>`.

---

## Passive Trigger Contract

When the orchestrating AI host observes:

```
change.state ∈ {reviewing, ready}  AND  all required artifacts written
```

(equivalent to spectra's `spectra status` reporting `isComplete: true`), the host SHOULD invoke `/speclink-analyze <change-name>` automatically before recommending `/speclink-apply`. This complements the inline `analyze.run` step that `speclink-propose` runs internally — the inline step catches programmatic issues, this skill adds the AI semantic layer.

The contract is **advisory**, not enforced — no hard wiring binds another skill to invoke this one. SpecLink engine surfaces the `isComplete` signal via `change.show.value.is_complete` (derived from artifact DAG state) so the host can detect the trigger condition.

---

## Concurrency & Errors

- `change.not_found` → list candidates via `change.list` and ask user to clarify.
- `provider.connection_failed` → engine has retried with backoff; surface error to user.
- `state.transition_invalid` → unlikely (analyze is read-only) but surface if happens.
- `analyze.failed` → engine internal error; surface JSON to user and STOP.

---

## Exit Criteria

The skill ends when ONE of:

- Analysis complete (programmatic + AI semantic) + report shown to user.
- Empty-state (no change to analyze) → user dismissed.
- Unrecoverable error → user informed.

---

## Guardrails

- **Read-only** — NEVER modify artifacts. Fix recommendations are **suggestions** for user / next skill. Use `disallowedTools: [Edit, Write]` to enforce.
- **Don't prompt inside fork** — if running in `context: fork`, do NOT use AskUserQuestion. Return findings to the main thread for the user to decide.
- **Don't auto-invoke `/speclink-apply`** — recommendations are user-confirmed.
- **Keep output concise** — this skill runs inline / as a fork; don't bloat the conversation context with raw JSON dumps. Summarize.
- **Don't fake the AI semantic layer** — if artifacts are too thin to analyze semantically, say so explicitly rather than inventing findings.
