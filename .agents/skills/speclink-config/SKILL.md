---
name: speclink-config
description: "Compose the workflow config's context and rules from the codebase, landed through an approved diff"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.15.0"
  generatedBy: "Speclink"
---

Compose the workflow config's `context` and `rules` from what the codebase structurally declares, then land them through a diff the user approves.

**Input**: Optionally a scope hint after `$speclink-config` (e.g. "rules only", "refresh the context"). If omitted, work the whole document.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**What this document is.** `context` is the shared briefing every artifact prompt carries; `rules` are per-artifact constraints. Both exist to add what the engine does NOT already know. The policy fields (`locale`, `spec_locale`, `tdd`, `audit`) are answers, not findings — never infer them.

**This skill never writes without approval.** Every change reaches the file through `speclink workflow-config ... --dry-run`, shown as a diff, and is only applied after the user says so.

---

## Step 1: Read the fixed input set

Scan ONLY the sources below. **Do NOT scan the source tree.** The goal is what the project structurally declares about itself; prose derived from implementation files ages out within a sprint and produces the churn this skill exists to prevent.

1. **Dependency manifests** — the workspace manifest (member list, shared dependency table) and each package's own dependency list. These name the components, their boundaries, and which runtimes each component is allowed to touch.
2. **README** — the project's own statement of what it is and who it serves.
3. **Documentation index** — the docs directory's entry points (index, architecture, status). Titles and one-line summaries only; do not mine the bodies.
4. **The current document** — `speclink workflow-config show --json`. What is already written is the starting point, not a blank page.
5. **Shared vocabulary** — `speclink language show`, when the project keeps one.

State which of the five you actually found. A missing source is a fact to report, not a gap to fill by guessing.

## Step 2: Draft against the four criteria

Every candidate line — for `context` and for `rules` alike — must survive all four. A line that fails any one of them is dropped, not softened.

### Criterion 1 — Never restate what the engine injects or a quality station already carries

Policy toggles and the schema's built-in artifact instructions are injected automatically; the quality stations carry their own standards inside their skills. Anything either of them already says is noise in this document.

**Injected content: disprove by payload, never from memory.** For each artifact of the active schema:

```bash
speclink instructions <artifact> --json
```

Read the returned instruction text and check your candidate line against it, one line at a time. If the payload already carries the requirement, drop the candidate. Do this for every artifact you intend to write rules for — an untested claim of "the engine doesn't say this" is exactly how duplicates got in before.

**Quality-station canon: disprove against the generated station skill, never from memory.** A station's standards live in its own generated skill file, in the same skills directory as this one — the review station's code-smell baseline sits in `speclink-review`, and no instructions payload carries it, so the payload check above cannot see it. Open whichever station skills are present in that directory and could overlap with a candidate line (which stations generate varies by tool), and read the passage each would duplicate. If a station already carries the standard, drop the candidate: the station skill is its single home, and a copy here is a second canon that drifts away from it.

### Criterion 2 — Artifact-specific content belongs in rules

If a line only bites on ONE artifact, it is a rule for that artifact, not context. `context` is for what every artifact prompt needs: what the system is, how it is partitioned, which constraints cut across all of them. When in doubt, demote to rules — an over-broad context line is paid for on every single prompt.

### Criterion 3 — Nothing that goes stale

No version numbers, no counts, no percentages, no "currently N crates", no test totals, no dates. These are wrong within weeks and nobody notices. Name the structure, not its measurements.

### Criterion 4 — Every reference must exist

`context` and `rules` may name commands, test names, file paths, and documents as the means of verification. **Verify each one exists, every run** — a rule pointing at a deleted test is worse than no rule, because it reads as satisfied. Run the command; check the path. Anything that no longer resolves is removed in this pass, even if you did not add it.

## Step 3: Ask for the policy fields — do not infer them

The four policy fields are the user's decision. Ask each one explicitly, one at a time, with the **AskUserQuestion tool** (or as plain text if unavailable), showing the current value from Step 1:

- `locale` — the language for generated prose
- `spec_locale` — the language for spec files (unset = English, `auto` = follow `locale`)
- `tdd` — whether apply enforces test-first discipline
- `audit` — whether apply enforces sharp-edges discipline

**Locale fields take locale CODES, never display names.** `locale` accepts exactly `tw`, `ja`, `en`; `spec_locale` accepts `tw`, `ja`, `en`, `auto`. Map the user's natural-language answer to its code before writing — 「繁體中文」 → `tw`, 「日本語」 → `ja`, "English" → `en` — the write verb rejects any value outside the code set, including display names.

Never derive an answer from the repo (a test directory does NOT mean `tdd: true`). Leave a field alone when the user has no opinion.

## Step 4: Show the diff, then write

Produce the diff for each change with the write verb's own `--dry-run`, so the preview is byte-identical to what would land:

```bash
speclink workflow-config set <key> <value> --dry-run
speclink workflow-config context --stdin --dry-run
speclink workflow-config rules <artifact> --stdin --dry-run
```

Never compute a diff yourself — a hand-made preview and the real serialization can disagree, and a preview that lies is worse than none.

Present the diffs and WAIT. Only after the user approves, re-run the same commands without `--dry-run`. Note for the user that a rewrite drops template comments from the file (the read-modify-write trade-off) — that is expected, not damage.

The same commands work in both local and remote mode; in remote mode the write is guarded against concurrent edits, and a refusal means someone else changed the document — re-run the command and it applies on top.

## Step 5: Verify convergence

Run this skill a second time against the same, unchanged codebase. **The second run's diffs MUST be empty.**

A non-empty second diff is not a reason to write again — it means a criterion was applied loosely (usually 3, restating something measurable, or 2, moving a line between `context` and `rules`). Go back to Step 2, find which line moved, and fix the judgment. Do NOT land the second diff.

Report at the end: which of the five sources were read, what was added, what was dropped and under which criterion, and the convergence result.

## Guardrails

- **Don't scan the source tree** — the fixed input set is the whole input.
- **Don't restate injected instructions** — disprove with `speclink instructions <artifact> --json`, per line.
- **Don't restate quality-station canon** — read the generated station skill; a copy here is a second canon that drifts.
- **Don't write anything that can go stale** — no versions, counts, or dates.
- **Don't reference what doesn't exist** — verify every command, test, and path, every run.
- **Don't infer the policy fields** — ask for all four.
- **Don't write without approval** — `--dry-run` first, always.
- **Don't land a non-empty second run** — that is a signal to re-judge, not to write.
