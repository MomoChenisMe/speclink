---
name: speclink-drift
description: "Detect drift between a Speclink change and the current codebase state"
context: fork
agent: Explore
disallowedTools: [Edit, Write]
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.16.1"
  generatedBy: "Speclink"
---

## Claude fork context

This generated Claude Code skill runs with `context: fork`. The rules in this section take precedence over the shared `drift` body below.

When no change name is provided, run `speclink list --json`. Auto-select only when there is exactly one active change. If there are zero active changes or more than one active change, return the candidate list or empty-state message and ask the main thread to rerun `/speclink-drift <change-name>`. Do NOT ask an interactive selection question inside the fork.

---

Detect drift between a Speclink change and the current codebase state. Reports time dormancy, broken design anchors, task collisions with external commits, and a single recommended next command.

**Input**: Optionally specify a change name (e.g., `/speclink-drift add-auth`). If omitted, infer from conversation context or auto-select if only one active change exists.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Determine change name**

   If not provided, infer from context or run `speclink list --json` to auto-select. If multiple active changes exist and no name is given, list candidates and ask the user to rerun with an explicit name.

2. **Run programmatic drift analysis**

   ```bash
   speclink drift <change-name> --json
   ```

   The JSON contains:
   - `severity`: `"light"` / `"medium"` / `"heavy"`
   - `total_score`: aggregate over Time / Structure / Tasks / Specs (Environment is display-only)
   - `dimensions`: array of `{ kind, status, score, contributes_to_total }`
   - `broken_anchors`: design.md references that no longer resolve. Only code-like tokens anchor (camelCase / snake_case / multi-hump PascalCase, plus backticked expressions); prose capitalized words never do. Backticked file paths (`hr/index.html`) are existence-checked and report as category `File`. The change's own directory is excluded from the symbol search, so a committed design does not satisfy its own anchors.
   - `spec_assumptions`: delta-spec operations whose canonical target has drifted — a MODIFIED/REMOVED/RENAMED requirement that no longer exists, or an ADDED requirement that now already exists. **The archive merge gate refuses these**; any entry here routes the recommendation to ingest.
   - `tasks_blocked_external`: pending tasks referencing a file that was touched by commits since `created` and no longer exists
   - `tasks_maybe_resolved`: pending tasks referencing a file that was committed since `created` and exists — the work may already be done
   - `commits_since_created` and the Environment status: commits since the created date (midnight-anchored), with a count of how many touch files this change references (touched record + task references)
   - `primary_recommendation`: a single copy-pasteable command line

3. **Present the report**

   **Report language**: run `speclink instructions apply --change "<name>" --json` and use its `locale` field (e.g., "Traditional Chinese (繁體中文)") — write the report in that language, prose, headings, and table labels included. Keep severity labels (light/medium/heavy), command lines, and code references in English. If the field is absent or the call fails, write in English.

   Use a user-readable, conclusion-first format. The first substantive paragraph after the title MUST be a plain-language conclusion that says what to do next before showing score tables, broken anchors, task collisions, or severity labels.

   Translate severity into action-oriented meaning:
   - **Light**: the change can continue with apply.
   - **Medium**: the change can continue, but the plan should be refreshed before implementation.
   - **Heavy**: the old plan is likely unsuitable for direct implementation; restart or refresh first.

   Recommended shape:

   ```markdown
   ## Drift Report: <change-name>

   <Plain-language conclusion. Example for medium: "This change can continue, but update the plan before implementing it. Related code has changed since the plan was written, so applying the old tasks directly may cause rework or conflicts.">

   ### Why

   - <1-3 plain-language reasons derived from dimensions, broken anchors, and task collisions>

   ### Details

   | Item              | Result                                                    |
   | ----------------- | --------------------------------------------------------- |
   | Time              | <status>                                                  |
   | Design references | <broken anchor count or "No broken references">           |
   | Delta assumptions | <stale assumption count or "All delta targets still hold"> |
   | Pending tasks     | <blocked/maybe-resolved count or "No task collisions">    |
   | Environment       | <N commits, M touching this change's files>               |
   | Overall           | <light/medium/heavy, total score N>                       |

   ### Recommendation

   Run `<primary_recommendation>`.
   ```

   Keep technical details below the plain-language conclusion. List broken anchors, stale delta assumptions, blocked tasks, and maybe-resolved tasks only when non-empty. Omit empty technical detail sections entirely. Keep the report short enough to skim; the goal is to help the user decide, not to explain the scoring model.

   **Stale delta assumptions take priority in the conclusion**: they mean another change has already rewritten the canonical requirement this delta targets, and the archive merge gate will refuse the change as-is. Say so explicitly and lead with the ingest recommendation.

4. **Apply the recommendation interactively**

   Use the **AskUserQuestion tool** to offer one decision based on `severity`. Use plain-language option labels (in the report language) while preserving the exact command in each option description. Do NOT auto-invoke `/speclink-apply`, `/speclink-ingest`, or `speclink archive`; always wait for the user's choice.
   - **Light** (score 0-3, drift is minor):
     - Recommended label: "Directly start work"
       - Description: run `/speclink-apply <name>`
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually
   - **Medium** (score 4-8, refresh worth doing):
     - Recommended label: "Refresh the plan"
       - Description: run `/speclink-ingest <name>` with the broken references and task collisions as context
     - Alternate label: "Directly start work"
       - Description: run `/speclink-apply <name>` only if the user knows the reported changes are harmless
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually
   - **Heavy** (score >8 or anchor decay >30%, design diverges from code):
     - Recommended label: "Archive and restart"
       - Description: run `<primary_recommendation>`
     - Alternate label: "Refresh the plan"
       - Description: try `/speclink-ingest <name>` before restarting
     - Alternate label: "Pause for now"
       - Description: do nothing until the user reviews manually

   If the **AskUserQuestion tool** is not available, present the same plain-language choices as text and wait for the user's response.

**Passive Trigger**

When `/speclink-apply` is invoked on a change whose `.openspec.yaml created` date is more than 5 days ago AND no commits have touched the change directory in the past 3 days, the apply skill SHOULD run drift analysis first and surface findings before tasks begin. The trigger is guidance only and MUST NOT block apply from proceeding.

(Threshold reasoning: AI-assisted commits are daily-cadence, not weekly. A change sitting ≥5 days with ≥3 days of no commits is almost always genuine stagnation rather than normal pacing.)

**Guardrails**

- Read-only: NEVER modify files, artifacts, or git state based on drift findings
- The CLI caps anchor checks at 50 via `ANCHOR_CAP` in `speclink_core::drift` to bound run-time
- If `speclink drift` returns a non-zero exit code (e.g., older binary without the drift subcommand), report the error and stop
- Do NOT auto-invoke any follow-up command — recommendations are user-confirmed
- If **AskUserQuestion tool** is not available, ask the same questions as plain text and wait for the user's response
