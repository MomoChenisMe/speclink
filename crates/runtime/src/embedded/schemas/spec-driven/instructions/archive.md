# Instructions: archive

This is a workflow phase, not an artifact-producing step. No file is written to the change directory by this kind. Archive moves the change directory and merges spec deltas into canonical specs.

## When to archive

Archive when:

- All tasks in `tasks.md` are checked.
- If `require_code_review: true`, code review has been approved.
- If `require_code_review: false`, archive runs immediately after `all_tasks_done`.

The engine records `all_tasks_done` automatically when the last `task.done` is invoked.

## Workflow

### Step 1: Pre-flight check

Invoke `change.show` to confirm:

- `state` is `code_reviewing` (review enabled) or `in_progress` with `all_tasks_done: true` (review disabled).
- All artifacts in `applyRequires` are `done`.

If any precondition fails, the engine returns `change.tasks_incomplete` and archive cannot proceed.

### Step 2: Invoke archive.run

Invoke `archive.run` for the change. The engine:

1. Reads each `specs/<capability>/spec.md` delta file in the change directory.
2. Applies delta operations (ADDED / MODIFIED / REMOVED / RENAMED) to the canonical capability spec at `openspec/specs/<capability>/spec.md` (or the project's configured spec root).
3. Renames the change directory from `changes/<name>/` to `changes/archive/<YYYY-MM-DD>-<name>/`.
4. Updates the change row's `archived_at` timestamp.
5. Emits an audit event with the spec delta summary.

When a delta cannot merge (conflict with canonical spec content), the engine returns `validation.archive_failed` with a `archive.specs_skipped` warning listing the specs that did not merge. Resolve the conflict by either updating the delta or the canonical spec, then re-run archive.

### Step 3: Optional commit sub-flow

After archive, the user may ask to commit. If so, generate a commit envelope referencing the archived change directory and the canonical spec changes. The runtime's `instructions.get kind=commit` returns the commit-flow guidance.

Do NOT commit silently. The user explicitly drives the commit step.

## Rules

- Archive is **destructive on the change directory** — the change is no longer in `changes/<name>/`. Tools that watch active changes should re-scan after archive.
- Spec delta merge is best-effort. Conflicts surface as warnings, never silent loss.
- Once archived, the change cannot be un-archived in MVP. Restore would require a separate flow (deferred).

## What success looks like

The change directory is now under `changes/archive/`. The canonical specs reflect every delta. The audit log records spec delta names, conflicts (if any), and the archive timestamp. The change row's `state` is `archived`.
