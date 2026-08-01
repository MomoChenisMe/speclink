Review a change's implementation for craft quality: two parallel read-only axes — **Standards** (repo conventions + a fixed code-smell baseline) and **Correctness** (bug hunting) — recorded round by round to a review ticket, closed by a stamp. Spec compliance is NOT this skill's job — that is `/speclink:verify`; the two quality stations run independently and either, both, or neither may be used per change.

**Input**: Optionally specify a change name after `/speclink:review` (e.g., `/speclink:review add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

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

   Read `progress`. If `remaining > 0`, STOP and explain: the review station requires every task complete before reviewing — finish `/speclink:apply` first. Do NOT spawn sub-agents and do NOT write the ticket.

   Keep the payload: `contextFiles` feeds step 4, and `locale` is the resolved language for the whole output chain — the sub-agent reports, the round presentation, the questions, and the ticket record alike (steps 5, 6 and 8).

3. **Scope this round**

   ```bash
   speclink review show "<name>" --json
   ```

   - **Command succeeds** (a ticket exists) → this is a follow-up round: the scope is the union of `lastRound.findings[].path` plus any files modified while fixing them. Do NOT rescan the whole change. If the last round carries zero findings (e.g. a refused stamp left a clean round behind), take the union of every round's `scope` from the same payload instead — the already-reviewed surface, not a full rescan.
   - **Command fails** (no ticket) → this is the first round: read the touched record `.speclink/touched/<name>.json` and collect the union of its recorded files. If the record is missing or empty (fresh clone, work done on another machine), use the **AskUserQuestion tool** (plain text + wait if unavailable) to ask for a git comparison base (e.g. `main`, a commit SHA), then scope with `git diff --name-only <base>...HEAD`.

   Keep source and test files; drop files that no longer exist. Artifact documents under `{{SPEC_DIR}}` are judging context (step 4), not review targets.

4. **Read the change artifacts as judging context**

   Read `contextFiles` (proposal, design, specs, tasks). They tell the reviewers what the code intends — pass the relevant intent into both briefs. Two hard rules:

   - Do NOT issue spec-compliance verdicts here — that is `/speclink:verify`'s dimension.
   - When artifacts are thin, judge only from the code and tests. Never invent requirements.

   **Remote mode**: when the workspace is connected to a remote store, `contextFiles` points into the read-only Context Projection (`.speclink/context/`). Read it freely, but NEVER edit projection files; spec changes go through speclink verbs.

5. **Spawn the two review axes in parallel**

   Send ONE message with TWO parallel read-only sub-agent calls (e.g. the Agent tool with a read-only agent). Sub-agents MUST NOT modify any file. If your harness cannot spawn sub-agents, run the two axes yourself sequentially and keep their analyses strictly separate.

   Both briefs carry: the scope file list, the relevant artifact intent, and the reporting contract — **under 400 words**, each finding on its own line as `- [SEVERITY] path — description`, SEVERITY ∈ CRITICAL / WARNING / SUGGESTION.

   Both briefs also carry the resolved `locale` (step 2): finding descriptions are written in that language; severity labels, the `Standards:` / `Correctness:` axis prefixes, file paths, and command lines stay in English. If `locale` is absent, everything is English.

   **Follow-up rounds**: when accepted findings exist — adjudicated this session (step 9) or carried in the ticket's last round with the `(accepted)` token — both briefs also carry that list with a hard instruction: do NOT re-report these items or near-variants of them — they are already adjudicated.

   **Standards axis brief** — first gather what the repo documents (CLAUDE.md / AGENTS.md, CONTRIBUTING, style docs, lint configs) and check the scope against it, citing the document for each violation. On top of whatever the repo documents, the Standards axis always carries the smell baseline below — a fixed set of Fowler code smells (Refactoring, ch.3) that applies even when a repo documents nothing. Two rules bind it:

   - **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
   - **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

   Each smell reads what it is → how to fix; match it against the diff:

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

   **Correctness axis brief** — hunt bugs in the scope: logic errors, boundary and edge cases, error-handling gaps, resource leaks, concurrency hazards, invariants broken between the changed files and their callers. Use the artifact intent only to understand what the code is for; report bugs, not compliance. CRITICAL = wrong behavior or data loss on a realistic path; WARNING = likely bug or fragile pattern; SUGGESTION = hardening opportunity. Quote the suspect hunk.

6. **Present both reports side by side**

   Render the two reports verbatim under `## Standards` and `## Correctness` headings — do NOT merge them, do NOT re-rank across axes. The reports already arrive in the `locale` language (bound in step 5) — never translate them. Close with exactly one summary line in that same language: the findings count per axis and the worst severity within each (never across).

7. **Triage every finding**

   After the reports, classify each finding into one of two buckets and show the result as its own list. The triage drives the recommendation in step 9 — it never changes the ticket format:

   - **Must-fix** — CRITICAL findings; Correctness findings with a realistic trigger path (WARNING included); unambiguous violations of a documented repo standard.
   - **Discretionary** — "possible X" smell judgements and SUGGESTION-level items. Give each one line: the cost of fixing weighed against the benefit.

8. **Record the round**

   ```bash
   speclink review add-round "<name>" --stdin
   ```

   Feed via stdin:
   - one `**Scope**:` line — this round's reviewed files, comma-separated repo-root relative paths;
   - zero or more findings lines `- [SEVERITY] path — description`, carrying both axes; start each description with its axis (`Standards:` / `Correctness:`).

   Findings descriptions go in exactly as the sub-agents reported them — same language, never translated by the main thread; severity labels and axis prefixes stay in English.

   **Follow-up rounds**: append every accepted, still-unfixed finding verbatim to this round's findings lines, ending each with the structural token `(accepted)` — the token stays English like the severity labels; a later session rebuilds the no-re-report list from the lines carrying it. The last round must reflect all outstanding reservations — that is what keeps an `--accept` stamp honest.

   NEVER hand-write or edit `{{SPEC_DIR}}changes/<name>/review.md` — the ticket is verb-owned; a malformed round is rejected by the verb, fix the stdin content and retry.

9. **Branch on findings**

   - **Zero findings this round** → stamp and report:

     ```bash
     speclink review stamp "<name>" --agent {{TOOL}}
     ```

     If the stamp refuses (e.g. tasks regressed meanwhile), report the reason and stop.

   - **Findings exist** → derive the recommendation from the triage: any must-fix outstanding → recommend option 1 and list the must-fix items; only discretionary items left → recommend option 2, noting the reservations stay on record via `--accept`. Then use the **AskUserQuestion tool** (plain text + wait if unavailable) with three options, the recommended one first and labelled "(Recommended)":
     1. **Fix and re-review** — fixes happen HERE in the main thread, following the project's TDD discipline; sub-agents never edit. Fix the must-fix list; discretionary items only when the user asks — anything left unfixed is accepted and carried (steps 5 and 8). **Verification gate**: after the fixes, run the project's full build and test suite and get it green BEFORE looping back to step 3 — a fix-introduced regression must never flow into the next round. The next round's scope is the last round's findings files plus whatever the fixes touched.
     2. **Accept as-is and stamp** — `speclink review stamp "<name>" --accept --agent {{TOOL}}` (stamps with reservations; the round's findings stay on record in the change history).
     3. **Stop without stamping** — end the session; the ticket stays for a later session or another reviewer to pick up (`speclink review show <name> --json` hands them the last round).

**Guardrails**

- The review station judges craft; `/speclink:verify` judges spec compliance — never issue compliance verdicts here
- Sub-agents are read-only; every fix returns to the main thread
- The ticket is verb-owned: create, append, and close it only through `speclink review` verbs
- Follow-up rounds review only the last round's findings files plus fix-touched files — no full rescans (a zero-findings last round widens to the union of the ticket's round scopes, still never the whole change)
- Triage drives the recommendation: must-fix outstanding → fix; only discretionary left → accept — never loop for taste alone
- The verification gate is hard: no next round starts on a failing build or test suite
- Accepted findings are carried, never re-reported: sub-agents get the no-re-report list, the round record keeps the items
- Thin artifacts: judge from code and tests, never invent requirements
- Stop on errors and report — don't guess past a failing verb
