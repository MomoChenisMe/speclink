<!-- SPECLINK:START v1.16.0 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs, change proposals, and discussion records live in the team system's spec store — always access them through `speclink` verbs; never read or write spec documents as local files.

## Use `/speclink-*` skills when:

- Requirements are fuzzy or worth debating → `/speclink-discuss` (recorded as a document; promote turns it into a change)
- User wants to plan, propose, or design a change → `/speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `/speclink-onboard`
- Tasks are ready to implement → `/speclink-apply`
- Resuming a change that sat idle → run `/speclink-drift` first
- Requirements change mid-work → `/speclink-ingest`
- Implementation is done, before archiving → optional quality stations `/speclink-review` (craft quality) ∥ `/speclink-verify` (spec compliance; user's call), then `/speclink-archive`
- Commit only files related to a specific change → `/speclink-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Requirements change mid-work? Plan mode → `ingest` → resume `apply`

<!-- SPECLINK:END -->
