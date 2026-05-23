# Instructions: ingest

This is a workflow phase, not an artifact-producing step. No file is written to the change directory by this kind.

Update an in-progress change from external context — new requirements discovered during apply, scope creep that needs to be formalized, or feedback from review that requires rewriting `proposal` / `spec` / `tasks` before resuming.

## When to ingest

Run ingest when one of the following happens:

- During apply, a user message or discovery reveals new requirements not captured in the change's artifacts.
- A reviewer's feedback requires changing the spec, not just adding code.
- An external doc / Slack / issue introduces context the change needs to absorb.
- You realize the proposal scope is wrong and needs reshaping.

Do NOT use ingest for trivial code-only adjustments; just continue apply.

## Workflow

### Step 1: Select the change

The change SHALL already exist and be in `proposing`, `reviewing`, `ready`, `in_progress`, or `code_reviewing` state. Ingest cannot create new changes — use propose for that.

### Step 2: Identify the cleanup scope

Determine which state the change is in, and how far back ingest must reach:

- `code_reviewing` → may need to drop review approvals if the spec materially changed.
- `in_progress` → may need to un-mark task checkboxes whose work no longer matches the new contract.
- `ready` → straightforward: rewrite artifacts, keep state.

The runtime records cascade cleanup in audit events as `change.state_changed` with `from_state` / `to_state` / `ingest_reason`.

### Step 3: Re-read artifacts and external context

Read `proposal.md`, all `specs/**/*.md`, `design.md`, `tasks.md`. Then read whatever external context drove the ingest (user message, design doc section, Slack thread). Identify exactly which sections need to change.

### Step 4: Update artifacts

Use `artifact.write` to update the affected artifacts. Be surgical:

- If only the spec needs new requirements, update spec.md, do not touch proposal / design / tasks.
- If the proposal scope expanded, update proposal first, then propagate to spec / design / tasks.
- If tasks are now wrong (because design changed), update tasks.md and surface the diff to the user.

When tasks already-marked-done no longer match the new contract, surface them to the user before touching their checkboxes. Do not silently undo completed work.

### Step 5: Resume apply

If the change was `in_progress`, invoke `apply.start` again (idempotent for the same actor) and continue from the first pending task. If the change moved back to `ready`, the user will need to start apply again.

## Rules

- Ingest is for **requirement change**, not "I'm rewriting because I got lazy with the first pass". If apply is just unclear about a detail already covered, re-read instead of ingesting.
- Every ingest SHALL emit an audit event with a non-empty reason. Empty reasons are rejected.
- Ingest holds a lock on the change while artifacts are updated. Release the lock by either completing the rewrite or aborting explicitly.
- Do NOT delete completed tasks in `tasks.md` because they are "no longer relevant". Either move them to a removed section with a reason, or accept they are part of the audit trail.

## What success looks like

After ingest, the change's artifacts reflect the new contract. Apply can resume without further ambiguity. The audit trail records what changed and why.
