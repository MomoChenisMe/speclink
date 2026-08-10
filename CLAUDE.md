<!-- SPECLINK:START v1.19.7 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

## Use `/speclink-*` skills when:

- Requirements are fuzzy or worth debating → `/speclink-discuss` (recorded as a document; promote turns it into a change)
- User asks for improvements without naming a topic → `/speclink-improve` (user-initiated only; scans the codebase and records the candidates as a discussion)
- User wants to plan, propose, or design a change → `/speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `/speclink-onboard`
- Tasks are ready to implement → `/speclink-apply`
- Implementing several independent changes at once → `/speclink-apply-with-worktree` (one git worktree per change)
- A worktree change is committed and ready to land → `/speclink-worktree-merge` (merge back, then clean up)
- Resuming a change that sat idle → run `/speclink-drift` first
- Requirements change mid-work → `/speclink-ingest`
- Implementation is done, before archiving → optional quality stations `/speclink-review` (craft quality) ∥ `/speclink-verify` (spec compliance; user's call), then `/speclink-archive`
- Both quality stations over one change → `/speclink-quality` (both checks first without stamping, then it stops after every round for your call on what to fix and when to stamp); only one station → call `/speclink-review` or `/speclink-verify` directly
- Commit only files related to a specific change → `/speclink-commit`

## Workflow

discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive

worktree: apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive (main checkout)

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Quality stations belong inside the worktree (the Apply baseline lives there); archive runs only from the main checkout — archiving inside a linked worktree is refused by the engine
- Requirements change mid-work? Plan mode → `ingest` → resume `apply`

<!-- SPECLINK:END -->
