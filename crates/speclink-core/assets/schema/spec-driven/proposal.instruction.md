Create the proposal document that establishes WHY this change is needed.

Sections:
- **Why**: 1-2 sentences on the problem or opportunity. What problem does this solve? Why now?
- **What Changes**: Bullet list of changes. Be specific about new capabilities, modifications, or removals. Mark breaking changes with **BREAKING**.
- **Non-Goals** (optional): Scope exclusions and rejected approaches. If design.md will be created, Non-Goals belong there (in Goals/Non-Goals section) — leave this section empty or omit it. If design.md will be skipped, write Non-Goals here so rejected approaches and scope exclusions are recorded in a persistent artifact.
- **Capabilities**: Identify which specs will be created or modified:
  - **New Capabilities**: List capabilities being introduced. Each becomes a new `specs/<name>/spec.md`. Use kebab-case names (e.g., `user-auth`, `data-export`).
  - **Modified Capabilities**: List existing capabilities whose REQUIREMENTS are changing. Only include if spec-level behavior changes (not just implementation details). Each needs a delta spec file. Check `openspec/specs/` for existing spec names. Leave empty if no requirement changes.
- **Impact**: Affected code, APIs, dependencies, or systems.

IMPORTANT: The Capabilities section is critical. It creates the contract between
proposal and specs phases. Research existing specs before filling this in.
Each capability listed here will need a corresponding spec file.
The exact capability name becomes the `specs/<name>/` directory name —
the analyzer flags any capability without a matching spec file as Critical.

Keep it concise (1-2 pages). Focus on the "why" not the "how" -
implementation details belong in design.md.

This is the foundation - specs, design, and tasks all build on this.
