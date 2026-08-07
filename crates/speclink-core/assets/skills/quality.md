Run both quality stations over one change as a single pass: `/speclink:review` and `/speclink:verify` each do their checking WITHOUT stamping, the findings from both are fixed together, each station re-validates — still without stamping — and only after one full pass lands zero new edits do the two stamps land back to back, followed by archive. Use this when both stations are known up front to be in play. Running only one station does NOT go through this skill — call that station directly and let it keep its own stamp-when-clean default.

**Input**: Optionally specify a change name after `/speclink:quality` (e.g., `/speclink:quality add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous, resolve it BEFORE step 1: run `speclink list --json` and prompt with the available changes (the AskUserQuestion tool, or plain text + wait if unavailable), then pass the same name to every station call.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**What this skill owns**

The ORDER of the two stations, and nothing else. What each station checks, how it freezes its scope, how it records its ticket, how it triages findings and what its stamp means all belong to `/speclink:review` and `/speclink:verify` — this document never restates them, and when it appears to disagree with a station's own instructions, the station wins. Follow each station's skill as written; this skill only decides when each one runs and which of its exits to take.

**Why the order matters**

A station's stamp freezes the content fingerprint of the files in its scope. Every edit after that — including the fix for the OTHER station's findings — turns the stamp stale ("done, but changed since"). Holding BOTH stamps until every fix has landed and been re-validated keeps both green all the way to archive: no stamp exists yet when a fix lands, so no stamp can go stale.

**Entry condition**: the change's tasks are all complete. That is the stations' own precondition — when it is not met, each station shows its own behavior (review refuses and stops; verify reports a mid-flight check-in instead of landing a ticket): relay that outcome verbatim. This skill adds no gate of its own, never pre-checks on the stations' behalf, and never swallows a station's error.

**Steps**

1. **Review check, no stamp**

   Run `/speclink:review` for the change. At its closing question, take the **stop without stamping** exit — the ticket and its frozen snapshot stay for step 4. A clean pass takes the same exit; the station's own quality-timeline exception covers it.

2. **Verify check, no stamp**

   Run `/speclink:verify` for the same change and take the same **stop without stamping** exit, clean pass included.

3. **Fix everything, once**

   Triage both stations' findings together and fix them in one go. Fixes happen HERE in the main thread following the project's TDD discipline — the stations' checking passes never edit files. Get the project's full build and test suite green before moving on.

4. **Review re-validation, still no stamp**

   Run `/speclink:review` again. Its validation pass covers every fix made since its frozen point — including the ones the verify findings asked for. Keep fixing and re-validating until its must-fix set is empty, then take the **stop without stamping** exit again; the station's quality-timeline exception defers the stamp.

5. **Verify re-validation, still no stamp**

   Run `/speclink:verify` again the same way: re-validate until its must-fix set is empty, exit without stamping.

6. **Converge before stamping**

   If step 4 or step 5 landed ANY new fix since the previous full pass, return to step 4 — the other station must validate those fixes too. The stamps wait until one full pass through both stations lands zero new edits.

7. **Stamp both, back to back**

   Re-enter review, then verify, telling each station explicitly that this is the quality timeline's **closing stamp call** — that phrase switches off their defer-the-stamp exception for this one call. Each station's own rules then decide the mechanics: an untouched clean last round stamps directly; content that moved since gets one validation pass in the same call, and the cleared round stamps immediately. Do NOT edit anything from here to archive — with zero edits between them, both stamps stay green.

8. **Archive**

   Both stamps are green — recommend `/speclink:archive`.

**Edge cases**

- **Changed your mind after a stamp** (one station already stamped, and only then the other is wanted): do NOT redo the stamped station. Run the new station, accept that the earlier stamp shows as changed-since in the meantime, and let archive settle it back to done — archive records that a stamp exists, it does not recompute freshness. Re-running a stamped station means a fresh full discovery pass for no gain.
- **One station only, or neither**: this skill does not apply. Call the station directly; its stamp-when-done default is already correct when no other station's fixes are coming.

**Guardrails**

- Never restate or override a station's checking, ticket or stamping rules — route to the station and follow what it says
- Both checking passes finish before any fixing starts; neither stamp lands before both stations' re-validations are clean
- No edits between the two stamps, and none between them and archive
- A station's refusal or error stops this flow and is reported as-is — do not work around it
