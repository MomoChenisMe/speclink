# Instructions: commit

This is a workflow phase, not an artifact-producing step. No file is written to the change directory by this kind. Commit produces a commit envelope (subject + body + file list) for the change being archived; the user runs the actual `git commit`.

## When to invoke commit

Commit is a sub-flow of archive. Invoke it after `archive.run` succeeds, when the user explicitly asks to commit. Do NOT auto-commit.

## What to produce

Generate a commit envelope structured as:

- **Subject** — Conventional Commits format (e.g., `feat:`, `fix:`, `refactor:`, `chore:`). Keep under 72 characters. Reflect the change's proposal `Why` section in one sentence.
- **Body** — short paragraph (3-5 lines) summarizing what changed and why. Reference the archived change directory path and the canonical specs that were updated.
- **Files** — list of paths to stage. Include:
  - `openspec/changes/archive/<YYYY-MM-DD>-<name>/**` (the archived change directory)
  - `openspec/specs/<capability>/spec.md` for each capability whose canonical spec was updated by the spec delta merge
  - Any source files modified during apply (the engine recorded these per task — fetch them via `change.show` or by reading `tasks.md` annotations)

## Workflow

### Step 1: Fetch context

Invoke `change.show` for the archived change. The response includes the archive timestamp, the spec delta summary, and the per-task touched-file list (recorded by `task.done`).

### Step 2: Compose envelope

Use the change's proposal `Why` text as the seed for the subject line. Compress to one sentence in Conventional Commits format. The body should reference the canonical specs that were merged (so future readers can find the contract change).

### Step 3: Surface to user

Output the commit envelope as a structured block (subject / body / files). The user reviews, edits as needed, and runs `git commit` themselves.

## Rules

- Do NOT execute `git commit` automatically. Surface the envelope; let the user run it.
- Do NOT include unrelated files in the file list. Stick to what the change actually touched, per the engine's per-task records.
- The subject SHALL follow Conventional Commits. Pick the type that best fits: `feat` for new capability, `fix` for bug repair, `refactor` for internal restructure with no behavior change, `chore` for tooling / config / version updates.
- If multiple capabilities changed, list them in the body, not the subject. Subject names the dominant theme.

## What NOT to produce

- Don't write a multi-paragraph essay. The body is a short paragraph, not a design document.
- Don't list files that were not actually modified — the engine knows. Trust the record.
- Don't include diff hunks in the commit message. Subject + body + file list only.
