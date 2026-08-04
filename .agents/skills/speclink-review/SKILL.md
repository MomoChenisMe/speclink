---
name: speclink-review
description: "Review a change's implementation for craft quality — parallel standards and correctness axes, recorded to a review ticket"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.9.0"
  generatedBy: "Speclink"
---

Review a change's implementation for craft quality: two parallel read-only axes — **Standards** (repo conventions + a fixed code-smell baseline) and **Correctness** (bug hunting) — run ONCE against a frozen change patch, then validated round by round to a review ticket, closed by a stamp. Round 1 is the only discovery pass; every later round only validates remediation. Spec compliance is NOT this skill's job — that is `$speclink-verify`; the two quality stations run independently and either, both, or neither may be used per change.

**Input**: Optionally specify a change name after `$speclink-review` (e.g., `$speclink-review add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select (if this tool is not available, ask as plain text and wait for the user's response).

   Show changes that have implementation tasks (tasks artifact exists).

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Gate: all tasks must be complete**

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   Read `progress`. If `remaining > 0`, STOP and explain: the review station requires every task complete before reviewing — finish `$speclink-apply` first. Do NOT spawn sub-agents and do NOT write the ticket.

   Keep the payload: `contextFiles` feeds step 4, and `locale` is the resolved language for the whole output chain — the sub-agent reports, the round presentation, the questions, and the ticket record alike (steps 5, 6 and 8).

3. **Check the ticket, then resolve the frozen scope**

   ```bash
   speclink review show "<name>" --json
   ```

   - **Ticket exists and `lastRound.findings` is empty** (e.g. a refused stamp left a clean round behind) → do NOT re-review: once the external gate recovers, retry the stamp directly — `speclink review stamp "<name>" --agent codex` — and report the outcome. No new discovery, no new validation.
   - **Otherwise** (no ticket, or the last round carries findings) → resolve the frozen scope:

   ```bash
   speclink review scope "<name>" --json
   ```

   - **State `resolved`** → keep the payload. `phase` names the pass (`discovery` on a ticketless change, `validation` on a follow-up), `patchHash` is the frozen patch identity, and `patch` / `files` carry the exact hunks under review. Every later step judges THIS frozen patch — the file list is never the review surface.
   - **State `needsInput` (non-zero exit)** → the scope is ambiguous (files dirty before Apply started, an overlapping active change, a missing or late baseline, or empty touched records). Relay the reported reasons and wait for the user to resolve it explicitly by one of: a trusted `--base <rev>`; a hash-pinned hunk selection (`--candidate-hash <sha256>` plus repeated `--include-hunk <id>`, ids from the needsInput payload); or redoing the work in an isolated worktree. Do NOT substitute the touched file list and do NOT widen to the whole worktree — commit-graph diffs and file lists both miss what the frozen patch pins.
   - **Command fails** (legacy ticket without a snapshot, drifted candidate, missing baseline for a follow-up) → report the error verbatim and stop; the explicit way out is the user's call: keep the ticket for later, or `speclink review discard "<name>"` and re-run discovery with an explicit trusted base. NEVER fall back to re-reviewing whole files.

4. **Read the change artifacts as judging context**

   Read `contextFiles` (proposal, design, specs, tasks). They tell the reviewers what the code intends — pass the relevant intent into both briefs. Two hard rules:

   - Do NOT issue spec-compliance verdicts here — that is `$speclink-verify`'s dimension.
   - When artifacts are thin, judge only from the code and tests. Never invent requirements.

   **Remote mode**: when the workspace is connected to a remote store, `contextFiles` points into the read-only Context Projection (`.speclink/context/`). Read it freely, but NEVER edit projection files; spec changes go through speclink verbs.

5. **Branch on `phase`**

   **Discovery (`phase: discovery`) — the one and only exploration pass.**

   Send ONE message with TWO parallel read-only sub-agent calls (e.g. the Agent tool with a read-only agent). Sub-agents MUST NOT modify any file. If your harness cannot spawn sub-agents, run the two axes yourself sequentially and keep their analyses strictly separate.

   Both briefs carry the same frozen patch — step 3's `patch` text with its hunk ranges — plus the relevant artifact intent and the reporting contract: **under 400 words**, each finding on its own line as `- [SEVERITY] path — description`, SEVERITY ∈ CRITICAL / WARNING / SUGGESTION. Both axes judge only the change hunks and the callers/tests needed to judge their direct impact — the unchanged remainder of a touched file is context, not review surface.

   Both briefs also carry the resolved `locale` (step 2): finding descriptions are written in that language; severity labels, the `Standards:` / `Correctness:` axis prefixes, file paths, and command lines stay in English. If `locale` is absent, everything is English.

   When accepted findings exist in the ticket's last round (the `(accepted)` token), both briefs also carry that list with a hard instruction: do NOT re-report these items or near-variants of them — they are already adjudicated.

   **Standards axis brief** — first gather what the repo documents (CLAUDE.md / AGENTS.md, CONTRIBUTING, style docs, lint configs) and check the frozen hunks against it, citing the document for each violation. On top of whatever the repo documents, the Standards axis always carries the smell baseline below — a fixed set of Fowler code smells (Refactoring, ch.3) that applies even when a repo documents nothing. Two rules bind it:

   - **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
   - **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

   Each smell reads what it is → how to fix; match it against the frozen hunks:

   - **Mysterious Name** — a function, variable, or type whose name doesn't reveal what it does or holds. → rename it; if no honest name comes, the design's murky.
   - **Duplicated Code** — the same logic shape appears in more than one hunk or file in the change. → extract the shared shape, call it from both.
   - **Feature Envy** — a method that reaches into another object's data more than its own. → move the method onto the data it envies.
   - **Data Clumps** — the same few fields or params keep travelling together (a type wanting to be born). → bundle them into one type, pass that.
   - **Primitive Obsession** — a primitive or string standing in for a domain concept that deserves its own type. → give the concept its own small type.
   - **Repeated Switches** — the same switch/if-cascade on the same type recurs across the change. → replace with polymorphism, or one map both sites share.
   - **Shotgun Surgery** — one logical change forces scattered edits across many files in the diff. → gather what changes together into one module.
   - **Divergent Change** — one file or module is edited for several unrelated reasons. → split so each module changes for one reason.
   - **Speculative Generality** — abstraction, parameters, or hooks added for needs the spec doesn't have. → delete it; inline back until a real need shows.
   - **Message Chains** — long a.b().c().d() navigation the caller shouldn't depend on. → hide the walk behind one method on the first object.
   - **Middle Man** — a class or function that mostly just delegates onward. → cut it, call the real target direct.
   - **Refused Bequest** — a subclass or implementer that ignores or overrides most of what it inherits. → drop the inheritance, use composition.

   (Baseline source: Matt Pocock's code-review skill, MIT.)

   Severity mapping for this axis: smells are judgement calls — report them as WARNING or SUGGESTION with "possible X" phrasing and the offending hunk quoted; CRITICAL is reserved for unambiguous violations of a documented repo standard.

   **Correctness axis brief** — hunt bugs in the frozen hunks: logic errors, boundary and edge cases, error-handling gaps, resource leaks, concurrency hazards, invariants broken between the changed hunks and their callers. Use the artifact intent only to understand what the code is for; report bugs, not compliance. CRITICAL = wrong behavior or data loss on a realistic path; WARNING = likely bug or fragile pattern; SUGGESTION = hardening opportunity. Quote the suspect hunk.

   **Validation (`phase: validation`) — remediation validation, never re-discovery.**

   Send the same two parallel read-only axes, but each brief carries ONLY: the last round's unresolved findings (verbatim), the accepted list, the remediation patch (step 3's frozen validation patch), and the necessary adjacent callers/tests plus artifact intent. Each axis judges, per original finding, resolved or unresolved — and reports only regressions the remediation patch directly introduces. It must NOT report new smells, SUGGESTIONs, or pre-existing issues in unchanged areas. The locale binding and the reporting contract are the same as in discovery.

   **Unrelated late findings during validation**: something new that the remediation patch did not cause must NOT be added to the current round and must NOT reopen discovery. Only when it carries evidence — a realistic trigger path plus one of a reproduction, a failing test, or a clear invariant violation — AND it affects security, data loss, or wrong behavior, end this station as **scope changed / failed**: keep the ticket, do not stamp, and recommend a separate discovery or a spun-off change. Anything below that bar is a note for later, never a blocker.

6. **Present both reports side by side**

   Render the two reports verbatim under `## Standards` and `## Correctness` headings — do NOT merge them, do NOT re-rank across axes. The reports already arrive in the `locale` language (bound in step 5) — never translate them. Close with exactly one summary line in that same language: the findings count per axis and the worst severity within each (never across).

7. **Triage every finding**

   After the reports, classify each finding into one of two buckets and show the result as its own list. The triage drives step 9 — it never changes the ticket format:

   - **Must-fix** — CRITICAL findings; Correctness findings with a realistic trigger path (WARNING included); unambiguous violations of a documented repo standard.
   - **Discretionary** — "possible X" smell judgements and SUGGESTION-level items. Give each one line: the cost of fixing weighed against the benefit.

   The **blocking set** of a round is its must-fix findings the user has not accepted — step 9's loop rule runs on its size.

8. **Record the round**

   ```bash
   speclink review add-round "<name>" --stdin
   ```

   Feed via stdin, in order:
   - one `**Phase**:` line — step 3's `phase` (`discovery` or `validation`);
   - one `**Patch**:` line — step 3's `patchHash` (`sha256:<hex>`);
   - one `**Scope**:` line — this round's reviewed files, comma-separated repo-root relative paths (the frozen patch's `paths`);
   - zero or more findings lines `- [SEVERITY] path — description`, carrying both axes; start each description with its axis (`Standards:` / `Correctness:`).

   Findings descriptions go in exactly as the sub-agents reported them — same language, never translated by the main thread; severity labels and axis prefixes stay in English.

   **Validation rounds**: every unresolved original finding is carried into the new round verbatim — never reworded; a reworded line fakes the shrinking the loop rule depends on. Regressions the remediation patch directly introduced enter as new findings lines. Every accepted, still-unfixed finding is appended verbatim ending with the structural token `(accepted)` — the token stays English like the severity labels. The last round must reflect all outstanding reservations — that is what keeps an `--accept` stamp honest.

   NEVER hand-write or edit `openspec/changes/<name>/review.md` — the ticket is verb-owned; a malformed round is rejected by the verb, fix the stdin content and retry.

9. **Branch on the blocking set**

   Let Bn be this round's blocking set (step 7). Compare its size with the previous round's Bn-1 (a first round has nothing to compare against):

   - **Bn is empty and no accepted findings remain** → stamp and report **passed clean**:

     ```bash
     speclink review stamp "<name>" --agent codex
     ```

     If the stamp refuses (e.g. tasks regressed meanwhile), report the reason and stop — the next session retries the stamp through step 3.

   - **Bn is empty but accepted findings remain** → recommend the user explicitly stamp with reservations — `speclink review stamp "<name>" --accept --agent codex` — and report **passed with reservations**. Never run `--accept` unprompted.

   - **Bn is strictly smaller than Bn-1** (or this is the first round with findings) → use the **AskUserQuestion tool** (plain text + wait if unavailable) with three options, the recommended one first and labelled "(Recommended)": any must-fix outstanding → recommend option 1; only discretionary left → recommend option 2.
     1. **Fix and re-validate** — fixes happen HERE in the main thread, following the project's TDD discipline; sub-agents never edit. Fix the must-fix list; discretionary items only when the user asks — anything left unfixed is accepted and carried (step 8). **Verification gate**: after the fixes, run the project's full build and test suite and get it green BEFORE looping back to step 3 — a fix-introduced regression must never flow into the next round. Step 3 then freezes the validation patch for the next round.
     2. **Accept as-is and stamp** — `speclink review stamp "<name>" --accept --agent codex` (stamps with reservations; the round's findings stay on record in the change history).
     3. **Stop without stamping** — end the session; the ticket and its frozen snapshot stay for a later session or another reviewer (`speclink review show <name> --json` hands them the last round).

   - **Bn is not strictly smaller than Bn-1** (equal or larger) → the round is already recorded; report **failed** immediately: keep the ticket, do NOT stamp, do NOT start another round automatically. The user decides what happens next (more work outside this loop, `--accept`, or discard).

   The shrinking blocking set only decides whether the automatic loop may continue — it is never a quality score and never described as "passed". There is no fixed maximum round count; every automatic continuation must strictly shrink the blocking set.

**Guardrails**

- The review station judges craft; `$speclink-verify` judges spec compliance — never issue compliance verdicts here
- Round 1 is the only discovery pass; validation rounds judge the original findings and the remediation patch's direct regressions — nothing else
- The frozen patch from `speclink review scope` is the review surface; touched file lists and worktree state never substitute for it
- needsInput and scope failures wait for an explicit disposal (trusted `--base`, hash-pinned selection, isolated worktree, or discard) — never guess past them
- Sub-agents are read-only; every fix returns to the main thread
- The ticket is verb-owned: create, append, and close it only through `speclink review` verbs
- Unresolved findings travel verbatim between rounds — rewording fakes progress
- The verification gate is hard: no next round starts on a failing build or test suite
- Accepted findings are carried, never re-reported: sub-agents get the no-re-report list, the round record keeps the items
- Thin artifacts: judge from code and tests, never invent requirements
- Stop on errors and report — don't guess past a failing verb
