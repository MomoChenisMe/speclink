Sync delta specs from a change to main specs.

This is an **agent-driven** operation - you will read delta specs and directly edit main specs to apply the changes. This allows intelligent merging (e.g., adding a scenario without copying the entire requirement).

**Input**: Optionally specify a change name after `/speclink:sync` (e.g., `/speclink:sync add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select.

   Show changes that have delta specs (under `specs/` directory).

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Find delta specs**

   Look for delta spec files in `{{SPEC_DIR}}changes/<name>/specs/*/spec.md`.

   Each delta spec file contains sections like:
   - `## ADDED Requirements` - New requirements to add
   - `## MODIFIED Requirements` - Changes to existing requirements
   - `## REMOVED Requirements` - Requirements to remove
   - `## RENAMED Requirements` - Requirements to rename (FROM:/TO: format)

   If no delta specs found, inform user and stop.

3. **For each delta spec, apply changes to main specs**

   For each capability with a delta spec at `{{SPEC_DIR}}changes/<name>/specs/<capability>/spec.md`:

   a. **Read the delta spec** to understand the intended changes

   b. **Read the main spec** at `{{SPEC_DIR}}specs/<capability>/spec.md` (may not exist yet)

   c. **Apply changes intelligently**:

   **ADDED Requirements:**
   - If requirement doesn't exist in main spec → add it
   - If requirement already exists → update it to match (treat as implicit MODIFIED)

   **MODIFIED Requirements:**
   - Find the requirement in main spec
   - Apply the changes - this can be:
     - Adding new scenarios (don't need to copy existing ones)
     - Modifying existing scenarios
     - Changing the requirement description
   - Preserve scenarios/content not mentioned in the delta

   **REMOVED Requirements:**
   - Remove the entire requirement block from main spec

   **RENAMED Requirements:**
   - Find the FROM requirement, rename to TO

   d. **Create new main spec** if capability doesn't exist yet:
   - Create `{{SPEC_DIR}}specs/<capability>/spec.md`
   - Add Purpose section (can be brief, mark as TBD)
   - Add Requirements section with the ADDED requirements

4. **Normalize the delta specs**

   After the main specs are updated, rewrite each delta spec file so a later `speclink archive` accepts it (NOTE: this dormant asset predates the fail-closed merge gate — the engine now REFUSES an ADDED requirement that already exists instead of skipping it, so a delta left un-normalized fails the archive loudly):

   - Rewrite each MODIFIED requirement as the complete final state, matching the merged main spec content (excluding any `@trace` comments)
   - Convert each ADDED requirement to MODIFIED with complete final state (it now exists in the main spec)
   - Leave REMOVED and RENAMED sections as-is

5. **Show summary**

   After applying all changes, summarize:
   - Which capabilities were updated
   - What changes were made (requirements added/modified/removed/renamed)
   - Confirm the delta specs were normalized to final state (a later archive re-applies them idempotently)

**Delta Spec Format Reference**

```markdown
## ADDED Requirements

### Requirement: New Feature

The system SHALL do something new.

#### Scenario: Basic case

- **WHEN** user does X
- **THEN** system does Y

## MODIFIED Requirements

### Requirement: Existing Feature

#### Scenario: New scenario to add

- **WHEN** user does A
- **THEN** system does B

## REMOVED Requirements

### Requirement: Deprecated Feature

## RENAMED Requirements

- FROM: `### Requirement: Old Name`
- TO: `### Requirement: New Name`
```

**Key Principle: Intelligent Merging**

Unlike programmatic merging, you can apply **partial updates**:

- To add a scenario, just include that scenario under MODIFIED - don't copy existing scenarios
- The delta represents _intent_, not a wholesale replacement
- Use your judgment to merge changes sensibly

**Output On Success**

```
## Specs Synced: <change-name>

Updated main specs:

**<capability-1>**:
- Added requirement: "New Feature"
- Modified requirement: "Existing Feature" (added 1 scenario)

**<capability-2>**:
- Created new spec file
- Added requirement: "Another Feature"

Main specs are now updated and delta specs normalized to final state. The change remains active - archive when implementation is complete.
```

**Guardrails**

- Read both delta and main specs before making changes
- Preserve existing content not mentioned in delta
- If something is unclear, ask for clarification
- Show what you're changing as you go
- The operation should be idempotent - running twice should give same result
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response

