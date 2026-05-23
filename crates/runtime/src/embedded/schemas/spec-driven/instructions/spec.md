# Instructions: spec

Create specification files that define **WHAT** the system shall do.

Specs use normative SHALL/MUST language and concrete WHEN/THEN scenarios. They are testable: every scenario is a potential test case.

## What to write

For each capability listed in the proposal's Capabilities section, create one delta spec file:

- **New capabilities** — use the exact kebab-case name from the proposal. Path: `specs/<capability>/spec.md` (relative to the change directory).
- **Modified capabilities** — use the existing canonical spec folder name. Same path under the change directory; the archive step merges your delta into the canonical spec.

### Delta operations (use `##` headers)

- **ADDED Requirements** — new requirements introduced by this change.
- **MODIFIED Requirements** — changed behavior. MUST include the FULL updated requirement block (header through scenarios); partial blocks lose detail at archive time.
- **REMOVED Requirements** — deprecated requirements. MUST include `**Reason**:` and `**Migration**:` lines.
- **RENAMED Requirements** — name changes only. Use `FROM:` / `TO:` format.

### Requirement structure

- Header: `### Requirement: <name>` followed by 1-2 sentences using SHALL / MUST.
- At least one scenario per requirement.
- Each scenario: `#### Scenario: <name>` (exactly 4 hashtags — `###` or bullets are silently ignored by the parser) with `**WHEN**` / `**THEN**` lines.

### Concrete examples (SBE — Specification by Example)

When a scenario involves data transformation, ordering, filtering, scoring, or state transitions, add an `##### Example:` block (5 hashtags) with concrete GIVEN/WHEN/THEN values. For multiple cases, use a markdown table inside the example block. Examples are optional but strongly recommended for non-obvious behavior; the analyzer suggests adding them for abstract scenarios.

## Rules

- **Spec files SHALL be written in English** regardless of the project locale. Normative language (SHALL / MUST / WHEN / THEN) is the contract surface and must remain canonical.
- Use forbidden-word filter: replace `should`, `may`, `might`, `consider`, `possibly`, `TBD`, `TODO`, `???`, `TKTK` with SHALL / SHALL NOT / MUST / MUST NOT. The analyzer flags these.
- One requirement → one observable behavior. If a requirement bundles three unrelated behaviors, split it.
- Cover the happy path AND the error path. Boundary conditions (empty input, max limits, unknown values) deserve their own scenarios.

## What NOT to write

- Don't describe HOW the system implements the requirement — that belongs in `design.md`.
- Don't reference internal module names, function signatures, or file paths inside requirement text. The spec describes external observable behavior, not implementation structure.
- Don't write scenarios so abstract they cannot be turned into a test. If you cannot imagine the assertion, the scenario is too vague.
