# Instructions: proposal

Create the proposal document that establishes **WHY** this change is needed.

## What to write

Fill in the template's sections in order:

- **Why** — 1-2 sentences on the problem or opportunity. State the concrete pain (broken behavior, missing capability, scaling limit), not vague aspirations.
- **What Changes** — bullet list of concrete deltas. Be specific about new capabilities, modifications, and removals. Mark breaking changes with **BREAKING**.
- **Non-Goals** — scope exclusions and rejected approaches. If `design.md` will be created for this change, Non-Goals MAY appear there instead; otherwise write them here so rejected approaches are recorded in a persistent artifact.
- **Capabilities** — identify which specs will be created or modified:
  - **New Capabilities** — each becomes `specs/<name>/spec.md`. Use kebab-case identifiers.
  - **Modified Capabilities** — only list when spec-level behavior changes; each needs a delta spec file.
- **Impact** — affected code paths (relative to project root), APIs, dependencies, systems.

## Rules

- Focus on the WHY, not the HOW. Implementation details belong in `design.md`.
- Keep it concise (1-2 pages typically). Long proposals indicate the change is too big.
- The Capabilities section is the contract between proposal and specs phases. The exact capability name becomes the `specs/<name>/` directory name — any capability without a matching spec file is flagged as Critical by the analyzer.
- Research existing specs (under `openspec/specs/` or your project's spec root) before listing modified capabilities — reusing the wrong name will fail downstream merge.

## What NOT to write

- Don't sketch implementations, code snippets, or task lists — those belong in `design.md` and `tasks.md`.
- Don't use placeholder language ("TBD", "TODO", "details to follow"). If the requirement is not clear yet, surface that as an Open Question instead of an empty section.

The proposal is the foundation — specs, design, and tasks all build on it. Be precise.
