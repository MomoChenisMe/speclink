Create specification files that define WHAT the system should do.

Create one spec file per capability listed in the proposal's Capabilities section.
- New capabilities: use the exact kebab-case name from the proposal (specs/<capability>/spec.md).
- Modified capabilities: use the existing spec folder name from openspec/specs/<capability>/ when creating the delta spec at specs/<capability>/spec.md.

Delta operations (use ## headers):
- **ADDED Requirements**: New capabilities
- **MODIFIED Requirements**: Changed behavior - MUST include full updated content
- **REMOVED Requirements**: Deprecated features - MUST include **Reason** and **Migration**
- **RENAMED Requirements**: Name changes only - use FROM:/TO: format

Format requirements:
- Each requirement: `### Requirement: <name>` followed by description
- Use SHALL/MUST for normative requirements. Forbidden words (analyzer flags these): should, may, might, consider, possibly, TBD, TODO, ???, TKTK — replace with SHALL/SHALL NOT/MUST/MUST NOT.
- Each scenario: `#### Scenario: <name>` with WHEN/THEN format
- **CRITICAL**: Scenarios MUST use exactly 4 hashtags (`####`). Using 3 hashtags or bullets will fail silently.
- Every requirement MUST have at least one scenario.

MODIFIED requirements workflow:
1. Locate the existing requirement in openspec/specs/<capability>/spec.md
2. Copy the ENTIRE requirement block (from `### Requirement:` through all scenarios)
3. Paste under `## MODIFIED Requirements` and edit to reflect new behavior
4. Ensure header text matches exactly (whitespace-insensitive)

Common pitfall: Using MODIFIED with partial content loses detail at archive time.
If adding new concerns without changing existing behavior, use ADDED instead.

Example:
```
## ADDED Requirements

### Requirement: User can export data
The system SHALL allow users to export their data in CSV format.

#### Scenario: Successful export
- **WHEN** user clicks "Export" button
- **THEN** system downloads a CSV file with all user data

## REMOVED Requirements

### Requirement: Legacy export
**Reason**: Replaced by new export system
**Migration**: Use new export endpoint at /api/v2/export
```

Specs should be testable - each scenario is a potential test case.

Concrete examples (SBE — Specification by Example):

Scenarios can include `##### Example: <name>` blocks (5 hashtags) with concrete
GIVEN/WHEN/THEN data that illustrates the scenario with real values:

    #### Scenario: sort by relevance
    - **WHEN** user searches
    - **THEN** results appear sorted by score

    ##### Example: three items sorted
    - **GIVEN** items: A(score=0.9), B(score=0.3), C(score=0.7)
    - **WHEN** user searches for "test"
    - **THEN** results appear in order: A, C, B

For multiple test cases, use a table inside the example block:

    ##### Example: boundary cases
    | Input | Expected | Notes |
    |-------|----------|-------|
    | "" | error: empty query | empty string |
    | "a" | minimum 2 chars warning | too short |
    | "valid query" | results returned | normal case |

When to add examples:
- The scenario involves computed output (sorting, filtering, scoring, ranking)
- The scenario involves state transitions or data transformation
- The boundary behavior is non-obvious
- A table can replace 3+ separate scenarios

When to skip examples:
- Simple UI navigation flows (click button, see page)
- Straightforward CRUD with no computed logic
- The WHEN/THEN already contains concrete values

Examples are optional — the analyzer will suggest adding them for abstract scenarios
but will not block specs without examples.

Spec files MUST always be written in English regardless of project locale settings,
because they use normative language (SHALL/MUST/WHEN/THEN).
