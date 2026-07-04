<!-- SPECTRA:START v1.0.2 -->

# Spectra Instructions

This project uses Spectra for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`.

## Use `/spectra-*` skills when:

- A discussion needs structure before coding → `/spectra-discuss`
- User wants to plan, propose, or design a change → `/spectra-propose`
- Tasks are ready to implement → `/spectra-apply`
- There's an in-progress change to continue → `/spectra-ingest`
- User asks about specs or how something works → `/spectra-ask`
- Implementation is done → `/spectra-archive`
- Commit only files related to a specific change → `/spectra-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → archive

- `discuss` is optional — skip if requirements are clear
- Requirements change mid-work? Plan mode → `ingest` → resume `apply`

## Parked Changes

Changes can be parked（暫存）— temporarily moved out of `openspec/changes/`. Parked changes won't appear in `spectra list` but can be found with `spectra list --parked`. To restore: `spectra unpark <name>`. The `/spectra-apply` and `/spectra-ingest` skills handle parked changes automatically.

<!-- SPECTRA:END -->

<!-- SPECLINK:START v1.2.0 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

## Use `/speclink-*` skills when:

- Requirements are fuzzy or worth debating → `/speclink-discuss` (recorded as a document; promote turns it into a change)
- User wants to plan, propose, or design a change → `/speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `/speclink-onboard`
- Tasks are ready to implement → `/speclink-apply`
- Resuming a change that sat idle → run `/speclink-drift` first
- Requirements change mid-work → `/speclink-ingest`
- Implementation is done → `/speclink-verify`, then `/speclink-archive`
- Commit only files related to a specific change → `/speclink-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → verify? → archive

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Requirements change mid-work? Plan mode → `ingest` → resume `apply`

<!-- SPECLINK:END -->
