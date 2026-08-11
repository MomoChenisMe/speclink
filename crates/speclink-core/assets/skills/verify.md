Verify that an implementation matches the change artifacts (specs, tasks, design).

**Input**: Optionally specify a change name after `/speclink:verify` (e.g., `/speclink:verify add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `speclink list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select (if this tool is not available, ask as plain text and wait for the user's response).

   Show changes that have implementation tasks (tasks artifact exists).
   Include the schema used for each change if available.
   Mark changes with incomplete tasks as "(In Progress)".

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check status to understand the schema**

   ```bash
   speclink status --change "<name>" --json
   ```

   Parse the JSON to understand:
   - `schemaName`: The workflow being used (e.g., "spec-driven")
   - Which artifacts exist for this change

3. **Load the artifacts and read the task progress**

   ```bash
   speclink instructions apply --change "<name>" --json
   ```

   This returns the change directory and context files. Read all available artifacts from `contextFiles`, and note `progress` (complete vs total tasks).

   **Remote mode**: when the workspace is connected to a remote store, `contextFiles` points into the read-only Context Projection (`.speclink/context/`) — a local snapshot of the remote canon. Read, search, and grep it freely, but NEVER edit projection files: a direct edit is not a remote write and the next command will reject the projection as modified. Any spec or artifact change goes through speclink verbs. If a `STALE` marker file exists at the projection root or a command reports the projection as modified, re-run `speclink instructions apply` to refresh it.

   The payload also carries `locale` — the resolved language for AI output (e.g., "Traditional Chinese (繁體中文)"). Remember it: the verification report is written in this language (see Output Format).

4. **Branch on the call: mid-flight check-in, closing stamp re-entry, or finished-work verification**

   **Not every code task is done (`codeRemaining > 0`) → mid-flight progress check-in.** Run the three dimensions as a conversation report only (steps 6–9 below, reading whatever artifacts and code you need). Do NOT run `speclink verify scope`, do NOT run `speclink verify add-round`, and do NOT stamp. The verify ticket records the verification of finished work — a check-in round landing in it would make "open ticket" stop meaning "the product's verification is unfinished" and would trip the archive gate for nothing. Report and STOP after step 9.

   **The `/speclink:quality` timeline's closing stamp call, and the ticket's last round's must-fix set is empty** (`speclink verify show "<name>" --json` — `lastRound.findings` has no CRITICAL/WARNING entries; SUGGESTION-only counts as empty) → branch at the entry, do not walk the full flow: run `speclink verify scope "<name>" --json`. An empty movement patch (nothing moved since that round) → skip the checking pass entirely, run `speclink verify stamp "<name>" --agent {{TOOL}}` directly and report — do NOT record another empty round. A non-empty patch → continue from step 6 as a normal validation pass; on this call step 13's defer exception is off, so the cleared round stamps immediately. `needsInput` or a scope failure here follows step 5's disposals unchanged — never guess past them.

   **Every code task is done (`codeRemaining` is 0) → finished-work verification.** Continue to step 5. `[M]` manual-verification tasks do not hold this back — they are human acceptance work the stamp deliberately does not wait for. When `remaining` is still above 0, name the open manual tasks in the report: the verification covers the code, and archive is what waits for the manual runs.

5. **Freeze the verification scope**

   ```bash
   speclink verify scope "<name>" --json
   ```

   The payload's `phase` decides this round:

   - `state: "resolved"` → use `phase`, `patchHash`, `paths`, `files` (with their old/new hunk ranges) and `patch`. This frozen patch is the code evidence for this round; the unchanged remainder of a touched file is context, never verification surface.
   - `state: "needsInput"` → the scope is ambiguous and nothing was frozen. Read the stderr disposals and pick one WITH the user: a trusted `--base <rev>`, a hash-pinned selection (`--candidate-hash <sha256>` plus repeatable `--include-hunk <id>`), or an isolated worktree. Never guess past it.
   - The command fails because a referenced snapshot is gone (a legacy ticket, or a cleared sidecar) → STOP. Keep the ticket, say the remediation delta cannot be rebuilt precisely, and wait for the user to explicitly `speclink verify discard "<name>"` and re-run discovery. Never fall back to re-verifying touched whole files.

6. **Branch on `phase`**

   Run the checking pass (steps 7–10) in ONE read-only sub-agent call (e.g. the Agent tool with a read-only agent) so its exploration stays out of the main thread's context. The sub-agent MUST NOT modify any file. If your harness cannot spawn sub-agents, run the checking pass yourself and keep its report separate from the triage that follows. The brief carries the resolved `locale` (step 3): finding descriptions are written in that language; severity labels, dimension prefixes, file paths, and command lines stay in English. When accepted findings exist in the ticket's last round (the `(accepted)` token), the brief also carries that list with a hard instruction: do NOT re-report these items or near-variants of them — they are already adjudicated.

   **Discovery (`phase: discovery`) — the one and only exploration pass.** Run the full three dimensions (steps 7–9) against every change artifact, using the frozen patch and the callers/tests needed to judge its direct impact as the code evidence.

   **Validation (`phase: validation`) — remediation validation, never re-discovery.** Take ONLY the last round's unresolved findings (verbatim), the accepted list, the remediation patch frozen in step 5, and the adjacent callers/tests needed to judge it. Decide per original finding: resolved or unresolved. Report only regressions the remediation patch directly introduces. Do NOT re-scan the whole change, the finding's whole file, or any unmodified area; do NOT raise new SUGGESTIONs or pre-existing issues in unchanged areas.

   **Segments marked `attribution: "adjacent"`** are files the remediation moved that no finding named — a caller, a test, a regenerated artifact, or a parallel session's edit leaking in. Confirm segment by segment that each genuinely belongs to THIS remediation, and report anything that does not as a regression. Never adopt an adjacent segment silently.

   **Unrelated late findings during validation**: something new that the remediation patch did not cause must NOT be added to the current round and must NOT reopen discovery. Only when it carries evidence — a realistic trigger path plus one of a reproduction, a failing test, or a clear invariant violation — AND it affects security, data loss, or wrong behavior, end this station as **scope changed / failed**: keep the ticket, do not stamp, and recommend a separate discovery or a spun-off change. Anything below that bar is a note for later, never a blocker.

7. **Verify Completeness**

   **Task Completion**:
   - If tasks.md exists in contextFiles, read it
   - Parse checkboxes: `- [ ]` (incomplete) vs `- [x]` (complete)
   - Count complete vs total tasks
   - If incomplete tasks exist:
     - Add CRITICAL issue for each incomplete task
     - Recommendation: "Complete task: <description>" or "Mark as done if already implemented"

   **Spec Coverage**:
   - If delta specs exist in `{{SPEC_DIR}}changes/<name>/specs/`:
     - Extract all requirements (marked with "### Requirement:")
     - For each requirement:
       - Search codebase for keywords related to the requirement
       - Assess if implementation likely exists
     - If requirements appear unimplemented:
       - Add CRITICAL issue: "Requirement not found: <requirement name>"
       - Recommendation: "Implement requirement X: <description>"

8. **Verify Correctness**

   **Requirement Implementation Mapping**:
   - For each requirement from delta specs:
     - Search codebase for implementation evidence
     - If found, note file paths and line ranges
     - Assess if implementation matches requirement intent
     - If divergence detected:
       - Add WARNING: "Implementation may diverge from spec: <details>"
       - Recommendation: "Review <file>:<lines> against requirement X"

   **Scenario Coverage**:
   - For each scenario in delta specs (marked with "#### Scenario:"):
     - Check if conditions are handled in code
     - Check if tests exist covering the scenario
     - If scenario appears uncovered:
       - Add WARNING: "Scenario not covered: <scenario name>"
       - Recommendation: "Add test or implementation for scenario: <description>"

   **Example Traceability**:
   - For each `##### Example:` in delta specs:
     - Check if a test exists that uses the same input values from the example's GIVEN/WHEN/THEN
     - If the example has a table, check if parameterized tests cover all rows
     - If examples appear untested, add WARNING: "Spec example not covered by test: <example name>" with recommendation to add a test using the GIVEN/WHEN/THEN from the example

9. **Verify Coherence**

   **Design Adherence**:
   - If design.md exists in contextFiles:
     - Extract key decisions (look for sections like "Decision:", "Approach:", "Architecture:")
     - Verify implementation follows those decisions
     - If contradiction detected:
       - Add WARNING: "Design decision not followed: <decision>"
       - Recommendation: "Update implementation or revise design.md to match reality"
   - If no design.md: Skip design adherence check, note "No design.md to verify against"

   **Code Pattern Consistency**:
   - Review new code for consistency with project patterns
   - Check file naming, directory structure, coding style
   - If significant deviations found:
     - Add SUGGESTION: "Code pattern deviation: <details>"
     - Recommendation: "Consider following project pattern: <example>"

10. **Generate the verification report**

    **Summary Scorecard**:

    ```
    ## Verification Report: <change-name>

    ### Summary
    | Dimension    | Status           |
    |--------------|------------------|
    | Completeness | X/Y tasks, N reqs|
    | Correctness  | M/N reqs covered |
    | Coherence    | Followed/Issues  |
    ```

    **Issues by Priority**:
    1. **CRITICAL** (Must fix before archive):
       - Incomplete tasks
       - Missing requirement implementations
       - Each with specific, actionable recommendation

    2. **WARNING** (Must fix — blocks the stamp):
       - Spec/design divergences
       - Missing scenario coverage
       - Each with specific recommendation

    3. **SUGGESTION** (Nice to fix — never blocks the stamp):
       - Pattern inconsistencies
       - Minor improvements
       - Each with specific recommendation

    **Final Assessment**:
    - If CRITICAL issues: "X critical issue(s) found. Fix before archiving."
    - If only warnings: "No critical issues. Y warning(s) to consider. Ready for archive (with noted improvements)."
    - If all clear: "All checks passed. Ready for archive."

11. **Triage every finding**

    Classify each finding into one of two buckets and show the result as its own list. The triage drives step 13 — it never changes the ticket format:

    - **Must-fix** — CRITICAL findings; Correctness findings with a realistic trigger path (WARNING included); requirements or scenarios with no implementation at all.
    - **Discretionary** — pattern-consistency observations and other nice-to-fix items. Give each one line: the cost of fixing weighed against the benefit.

    Severity IS the blocking boundary: must-fix findings are recorded as CRITICAL or WARNING; discretionary findings are ALWAYS recorded as SUGGESTION — never WARNING. SUGGESTION-level findings do not block the stamp, need nobody's approval, and never enter the acceptance mechanism.

    The **blocking set** of a round is its must-fix findings the user has not accepted — step 13's loop rule runs on its size.

12. **Record the round**

    ```bash
    speclink verify add-round "<name>" --stdin
    ```

    Feed via stdin, in order:
    - one `**Phase**:` line — step 5's `phase` (`discovery` or `validation`);
    - one `**Patch**:` line — step 5's `patchHash` (`sha256:<hex>`);
    - one `**Scope**:` line — this round's verified files, comma-separated repo-root relative paths (the frozen patch's `paths`);
    - zero or more findings lines `- [SEVERITY] path — description`; start each description with its dimension (`Completeness:` / `Correctness:` / `Coherence:`).

    Findings descriptions go in as reported — same language, never translated by the main thread; severity labels and dimension prefixes stay in English.

    **Validation rounds**: every unresolved original finding is carried into the new round verbatim — never reworded; a reworded line fakes the shrinking the loop rule depends on. Resolved findings are dropped. Regressions the remediation patch directly introduced enter as new findings lines. Every accepted, still-unfixed **must-fix** finding is appended verbatim ending with the structural token `(accepted)` — the token stays English like the severity labels. Unfixed SUGGESTIONs carry forward as plain unresolved lines without the token (the step 11 boundary). The last round must reflect all outstanding reservations — that is what keeps an `--accept` stamp honest.

    NEVER hand-write or edit `{{SPEC_DIR}}changes/<name>/verify.md` — the ticket is verb-owned; a malformed round is rejected by the verb, fix the stdin content and retry.

13. **Branch on the blocking set**

    Let Bn be this round's blocking set (step 11). Compare its size with the previous round's Bn-1 (a first round has nothing to compare against):

    - **Bn is empty and no accepted must-fix findings remain** → stamp and report **passed clean** (leftover SUGGESTIONs stay recorded — list them in the report):

      ```bash
      speclink verify stamp "<name>" --agent {{TOOL}}
      ```

      If the stamp refuses (e.g. tasks regressed meanwhile), report the reason and stop — the next session retries the stamp through step 5.

      **Exception — inside the `/speclink:quality` timeline, before its closing stamp call**: when this station runs as a checking or re-validation step of `/speclink:quality`, do NOT stamp on a cleared round — neither a DISCOVERY round with no must-fix findings nor a VALIDATION round whose blocking set has just cleared. The round is already recorded (step 12); take the **stop without stamping** ending (the same exit as option 3 below: the verification ticket and its snapshot stay). The stamp lands at that skill's **closing stamp call**, which enters through step 4's closing-stamp branch — and on that call this exception is OFF: an untouched round whose must-fix set is empty stamps directly without a new round, a moved one clears its validation pass and stamps immediately. Called directly as a single station, a cleared round still stamps on the spot; this exception is only about the two-station ordering.

    - **Bn is empty but accepted must-fix findings remain** → recommend the user explicitly stamp with reservations — `speclink verify stamp "<name>" --accept --agent {{TOOL}}` — and report **passed with reservations**. Never run `--accept` unprompted.

    - **Bn is strictly smaller than Bn-1** (or this is the first round with must-fix findings) → use the **AskUserQuestion tool** (plain text + wait if unavailable) with three options, the recommended one first and labelled "(Recommended)": recommend option 1 — outstanding must-fix findings are what brought the loop here. SUGGESTION-only rounds never reach this menu: they stamp directly through the first bullet.
      1. **Fix and re-verify** — fixes happen HERE in the main thread, following the project's TDD discipline; the checking pass never edits files. Fix the must-fix list; discretionary items only when the user asks. A must-fix finding the user chooses not to fix is accepted and carried with the `(accepted)` token (step 12); unfixed SUGGESTIONs just carry forward (step 12). **Verification gate**: after the fixes, run the project's full build and test suite and get it green BEFORE looping back to step 5 — a fix-introduced regression must never flow into the next round. Step 5 then freezes the validation patch for the next round.
      2. **Accept as-is and stamp** — `speclink verify stamp "<name>" --accept --agent {{TOOL}}` (stamps with reservations; the round's findings stay on record in the change history).
      3. **Stop without stamping** — end the session; the ticket and its frozen snapshot stay for a later session or another verifier (`speclink verify show <name> --json` hands them the last round).

    - **Bn is not strictly smaller than Bn-1** (equal or larger) → the round is already recorded; report **failed** immediately: keep the ticket, do NOT stamp, do NOT start another round automatically. The user decides what happens next (more work outside this loop, `--accept`, or discard).

    The shrinking blocking set only decides whether the automatic loop may continue — it is never a quality score and never described as "passed". There is no fixed maximum round count; every automatic continuation must strictly shrink the blocking set.

**Verification Heuristics**

- **Completeness**: Focus on objective checklist items (checkboxes, requirements list)
- **Correctness**: Use keyword search, file path analysis, reasonable inference - don't require perfect certainty
- **Coherence**: Look for glaring inconsistencies, don't nitpick style
- **False Positives**: When uncertain, prefer SUGGESTION over WARNING, WARNING over CRITICAL
- **Actionability**: Every issue must have a specific recommendation with file/line references where applicable

**Graceful Degradation**

- If only tasks.md exists: verify task completion only, skip spec/design checks
- If tasks + specs exist: verify completeness and correctness, skip design
- If full artifacts: verify all three dimensions
- Always note which checks were skipped and why

**Output Format**

Use clear markdown with:

- Write the report in the `locale` language from step 3's payload — prose, headings, and table labels included. Keep severity labels (CRITICAL/WARNING/SUGGESTION), structural spec markers, command lines, and code references in English. If `locale` is absent, write in English.
- Table for summary scorecard
- Grouped lists for issues (CRITICAL/WARNING/SUGGESTION)
- Code references in format: `file.ts:123`
- Specific, actionable recommendations
- No vague suggestions like "consider reviewing"

**Guardrails**

- `/speclink:verify` judges spec compliance; the review station judges craft — never issue craft verdicts here
- The mid-flight check-in never touches the ticket: no `verify scope`, no `verify add-round`, no stamp
- Round 1 is the only discovery pass; validation rounds judge the original findings and the remediation patch's direct regressions — nothing else
- The frozen patch from `speclink verify scope` is the code evidence; touched file lists and worktree state never substitute for it
- needsInput and scope failures wait for an explicit disposal (trusted `--base`, hash-pinned selection, isolated worktree, or discard) — never guess past them
- The checking pass is read-only; every fix returns to the main thread
- The ticket is verb-owned: create, append, and close it only through `speclink verify` verbs
- Unresolved findings travel verbatim between rounds — rewording fakes progress
- The verification gate is hard: no next round starts on a failing build or test suite
- Accepted findings are carried, never re-reported
- Thin artifacts: verify what exists, never invent requirements
- Stop on errors and report — don't guess past a failing verb
