# speclink-drift Workflow

Detect drift between a SpecLink change and the current codebase state. Reports time dormancy, broken design anchors, task collisions with external commits, and a single recommended next command.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is in `bindings/bash.md` or `bindings/tool.md`.

All operations are referenced by their **canonical id** (e.g., `drift.run`). See `doc/protocol/operations.md` for full operation specs.

---

## Input

Optionally specify a change name (e.g., `/speclink-drift add-auth`). If omitted, infer from conversation context or auto-select if only one active change exists.

## Prerequisites

This skill requires a working SpecLink invocation surface. If any operation fails with `cli.not_found` / `sdk.not_initialized`, report and STOP.

---

## Steps

### Step 1: Determine change name

If not provided, infer from context or invoke `change.list`.

- **Zero active changes**: empty-state message + STOP.
- **One active change**: auto-select.
- **Multiple active changes** (fork context): return candidate list, ask main thread to rerun `/speclink-drift <change-name>`. Do NOT prompt inside fork.

### Step 2: Run programmatic drift analysis

Invoke `drift.run` with `{ change_id }`.

The op returns:

- `severity`: `"light"` / `"medium"` / `"heavy"`
- `total_score`: aggregate over Time / Structure / Tasks (Environment is display-only)
- `dimensions[]`: `{ kind, status, score, contributes_to_total }`
- `broken_anchors[]`: design.md references (file paths / symbols / functions / CLI flags) that no longer resolve
- `tasks_blocked_external[]`: pending tasks whose referenced files were modified by commits outside the change directory
- `tasks_maybe_resolved[]`: pending tasks whose verb+target keywords match commit subjects since `created_at`
- `primary_recommendation`: a single copy-pasteable command line

### Step 3: Present the report

Use a user-readable, **conclusion-first** format. The first substantive paragraph after the title MUST be a plain-language conclusion that says what to do next, BEFORE showing score tables / broken anchors / task collisions / severity labels.

Translate severity into action-oriented meaning:

- **Light**: the change can continue with apply.
- **Medium**: the change can continue, but the plan should be refreshed before implementation.
- **Heavy**: the old plan is likely unsuitable for direct implementation; restart or refresh first.

Recommended shape:

```markdown
## Drift Report: <change-name>

<Plain-language conclusion. Example for medium: "This change can continue, but update the plan
before implementing it. Related code has changed since the plan was written, so applying the old
tasks directly may cause rework or conflicts.">

### Why

- <1-3 plain-language reasons derived from dimensions, broken anchors, and task collisions>

### Details

| Item              | Result                                                 |
| ----------------- | ------------------------------------------------------ |
| Time              | <status>                                               |
| Design references | <broken anchor count or "No broken references">        |
| Pending tasks     | <blocked/maybe-resolved count or "No task collisions"> |
| Overall           | <light/medium/heavy, total score N>                    |

### Recommendation

Run `<primary_recommendation>`.
```

Keep technical details **below** the plain-language conclusion. List broken anchors / blocked tasks / maybe-resolved tasks only when non-empty. Omit empty technical detail sections entirely. Keep the report short enough to skim; the goal is to help the user decide, not to explain the scoring model.

### Step 4: Apply the recommendation interactively

Use AskUserQuestion to offer ONE decision based on `severity`. Use plain-language option labels while preserving the exact command in each option description. Do NOT auto-invoke `/speclink-apply`, `/speclink-ingest`, or `archive.run` — always wait for user choice.

**Light** (score 0-3, drift is minor):

- Recommended: **Directly start work** → run `/speclink-apply <name>`
- Alternate: **Pause for now** → do nothing until the user reviews manually

**Medium** (score 4-8, refresh worth doing):

- Recommended: **Refresh the plan** → run `/speclink-ingest <name>` with the broken references and task collisions as context
- Alternate: **Directly start work** → run `/speclink-apply <name>` only if the user knows the reported changes are harmless
- Alternate: **Pause for now** → do nothing until manual review

**Heavy** (score >8 or anchor decay >30%, design diverges from code):

- Recommended: **Archive and restart** → run the engine's `primary_recommendation` (typically a guided archive + new propose)
- Alternate: **Refresh the plan** → try `/speclink-ingest <name>` before restarting
- Alternate: **Pause for now** → do nothing until manual review

If AskUserQuestion is not available in the host, present the same plain-language choices as text and wait for the user's response.

---

## Passive Trigger Contract

`/speclink-drift` has TWO invocation paths:

1. **Explicit (user-driven)**: `/speclink-drift <change-name>` — runs this skill end-to-end.
2. **Implicit (inline from `/speclink-apply`)**: `speclink-apply` Step 3d, when dormancy conditions are met (change `created_at` > 5 days ago AND change directory has zero commits in past 3 days), runs `drift.run` op **inline** + presents inline AskUserQuestion. This does NOT invoke this skill — it directly calls the `drift.run` op for tighter integration with apply's flow.

Both paths use the same `drift.run` op. The skill (path 1) adds standalone presentation + interactive layer; the inline path (path 2) folds it into apply's preflight.

Threshold reasoning: AI-assisted commits are daily-cadence. ≥5 days dormant + ≥3 days no commit ≈ genuine stagnation, not normal pacing.

---

## Concurrency & Errors

- `change.not_found` → list candidates and re-prompt.
- `provider.connection_failed` → engine retried; surface to user.
- `drift.failed` → engine internal error; surface JSON and STOP.
- `drift.anchor_cap_exceeded` — drift checks cap anchors at 50 (`ANCHOR_CAP` in engine). If cap hit, report shows note but doesn't fail.

---

## Exit Criteria

The skill ends when ONE of:

- Drift report presented + user picked next action (apply / ingest / pause).
- Empty-state (no change to drift-check) → user dismissed.
- Unrecoverable error → user informed.

---

## Guardrails

- **Read-only** — NEVER modify files, artifacts, or git state based on drift findings.
- **Do NOT auto-invoke follow-up** — recommendations are user-confirmed. Even if Heavy drift, the skill does NOT automatically archive + restart.
- **Conclusion-first format** — always lead with plain-language "what to do" before any table / score / technical detail.
- **Honor anchor cap** — the engine caps anchor checks at 50 to bound runtime. If you see "anchor cap reached" in the result, present the partial findings honestly.
- **Don't run inside `proposing` state** — drift makes no sense before specs / tasks exist. Surface a hint and STOP.
- **Skip empty sections** — if broken_anchors is empty, omit the section entirely. Don't pad the report.
