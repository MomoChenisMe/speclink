---
name: speclink-baseline
description: "Use when adopting Speclink on a codebase that already has behavior but no specs — establishes the baseline by generating the initial specs from what the code does today."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.26.0"
  generatedBy: "Speclink"
---

Establish the baseline for an existing codebase: generate the initial canonical specs from current behavior, so later changes have a spec baseline to build on.

**IMPORTANT: The baseline documents what the system does TODAY — not what it should do.** Specs written here describe observed behavior with evidence. Aspirations, fixes, and improvements belong in a change (`/speclink-propose`) AFTER the baseline is written. Because nothing is changing, baseline writes directly to `openspec/specs/` — no change folder is involved.

**Input**: Optionally a scope hint after `/speclink-baseline` (e.g., "auth and billing only"). If omitted, baseline the whole codebase.

---

## Step 1: Check the current state

```bash
speclink list --specs
```

- **No specs yet** → full baseline pass; continue below.
- **Some specs exist** → gap-filling mode: inventory what is NOT yet covered and scope the rest of this flow to those areas. Never rewrite an existing spec here — propose a change instead.

Read `openspec/config.yaml` (project context, and `spec_locale` — write spec prose in the configured language; structural markers and SHALL/MUST keywords stay in English).

## Step 2: Inventory the codebase

Build a behavioral map before writing anything:

1. Read the README, package manifests, and entry points (main/CLI/server routes).
2. Scan the source tree for domains: commands, modules, services, UI surfaces.
3. Read the test suite — tests are the best source of observable behavior with concrete values.
4. Note behaviors you can VERIFY (code + tests) versus behaviors you can only INFER.

Spend effort proportional to repo size; for large repos, sample entry points and tests first.

## Step 3: Propose the capability map — and WAIT

Draft a capability list (kebab-case names, one behavior area each — the same granularity a change's delta specs would use). For each: one-line purpose + the evidence files behind it.

Present the map with the **AskUserQuestion tool** (or as plain text if unavailable) and let the user confirm, merge, split, or drop capabilities. **Do NOT write any spec before the map is confirmed** — wrong boundaries here are expensive to undo later.

## Step 4: Write the specs

For each confirmed capability, create `openspec/specs/<capability>/spec.md`:

```markdown
# <capability> Specification

## Purpose

<1-3 sentences: what this capability does for whom>

## Requirements

### Requirement: <Name>
<Observed behavior in SHALL form.>

#### Scenario: <name>
- **WHEN** <trigger observed in code/tests>
- **THEN** <outcome observed in code/tests>
```

Rules:

- **Evidence or flag it.** Every requirement must trace to code or tests you actually read. If a behavior is inferred but unverified, ask the user or leave it out — do not guess it into the record.
- Concrete scenarios: real values from tests make the best WHEN/THEN data; add `##### Example:` blocks where tests provide exact input→output pairs.
- Behavior only — no implementation details (module names, algorithms) in requirement text.
- 4 hashes for `#### Scenario:`, SHALL/MUST keywords in English, prose in `spec_locale`.

## Step 5: Validate and report

```bash
speclink validate --specs --all --strict
```

Fix structural findings, then report: capabilities created (with requirement/scenario counts), behaviors flagged as unverified, and areas deliberately left out. Then state that specs now describe the current system and that future work goes through changes — the exits are in **Next steps** below.

## Guardrails

- **Don't invent behavior** — evidence-based only; unverified inferences are flagged or omitted.
- **Don't refactor while writing the baseline** — no code changes at all.
- **Don't rewrite existing specs** — gap-fill only; modifications go through a change.
- **Do confirm the capability map before writing** — boundaries are the expensive decision.
- **Do keep specs small** — a capability that needs 15 requirements is probably two capabilities.

## Next steps

Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.

- The requirements for the next piece of work are clear → `/speclink-propose <idea>`
- They are still fuzzy or worth debating → `/speclink-discuss <topic>`
