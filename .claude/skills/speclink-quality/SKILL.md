---
name: speclink-quality
description: "Run both quality stations over one change, pausing after every round for the user's call: both checks first without stamping, then both stations' findings are reported together and the skill stops — nothing is fixed, stamped or archived without the user's answer"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.18.0"
  generatedBy: "Speclink"
---

Run both quality stations over one change as a single pass that pauses after every round: `/speclink-review` and `/speclink-verify` each do their checking WITHOUT stamping, then this skill reports both stations' findings together and STOPS for the user's call on what to fix, when to stamp, and whether to archive. Nothing is fixed, stamped or archived without their answer. Use this when both stations are known up front to be in play. Running only one station does NOT go through this skill — call that station directly and let it keep its own stamp-when-clean default.

**Input**: Optionally specify a change name after `/speclink-quality` (e.g., `/speclink-quality add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous, resolve it BEFORE step 1: run `speclink list --json` and prompt with the available changes (the AskUserQuestion tool, or plain text + wait if unavailable), then pass the same name to every station call.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**What this skill owns**

The ORDER of the two stations and the pause that ends every round, and nothing else. What each station checks, how it freezes its scope, how it records its ticket, how it triages findings and what its stamp means all belong to `/speclink-review` and `/speclink-verify` — this document never restates them, and when it appears to disagree with a station's own instructions, the station wins. Follow each station's skill as written; this skill only decides when each one runs, which of its exits to take, and when to hand the decision back to the user.

**Why the order matters**

A station's stamp freezes the content fingerprint of the files in its scope. Every edit after that — including the fix for the OTHER station's findings — turns the stamp stale ("done, but changed since"). Holding BOTH stamps until every fix has landed and been re-validated keeps both green all the way to archive: no stamp exists yet when a fix lands, so no stamp can go stale.

**Why every round pauses**

Which findings are worth fixing, and whether the change is ready to stamp, are the user's calls — not this skill's. So every round ends the same way: report both stations, then stop and ask. Never fix a finding, stamp a station or recommend your way past a decision on your own initiative. A clean round is not an exception: "both stations green" is something to report and wait on, like everything else.

**Entry condition**: the change's tasks are all complete. That is the stations' own precondition — when it is not met, each station shows its own behavior (review refuses and stops; verify reports a mid-flight check-in instead of landing a ticket): relay that outcome verbatim. This skill adds no gate of its own, never pre-checks on the stations' behalf, and never swallows a station's error.

**Steps**

1. **Review check, no stamp**

   Run `/speclink-review` for the change. At its closing question, take the **stop without stamping** exit — the ticket and its frozen snapshot stay for the rounds that follow. A clean pass takes the same exit; the station's own quality-timeline exception covers it.

2. **Verify check, no stamp**

   Run `/speclink-verify` for the same change and take the same **stop without stamping** exit, clean pass included.

3. **Stop and ask — the round's pause**

   Both stations have reported. Summarize their findings TOGETHER — grouped by station, must-fix separated from the rest — and then STOP: put the next step to the user with the AskUserQuestion tool (no such tool: ask in plain text and wait for an answer). Make no edits before the answer arrives.

   The options:

   - **Fix everything** — every finding from both stations.
   - **Fix a selection** — the user names which ones; the rest stay in the tickets, unfixed.
   - **Fix nothing and stop** — end the pass right here. Both stations already left through their **stop without stamping** exit, so both tickets and their frozen snapshots stay on disk, no stamp lands, and nothing is archived.
   - **Go to the closing stamps** — offer this option ONLY when both stations' must-fix sets are empty. While any must-fix is outstanding it is not on the menu: must-fix-cleared-before-the-stamp is the stations' rule and this skill does not route around it.

   Options with nothing to act on simply do not appear — a round where both stations found nothing offers the last two.

   A **fix everything** or **fix a selection** answer continues to step 4; **fix nothing and stop** ends here; **go to the closing stamps** jumps to step 6.

4. **Fix exactly what was chosen**

   Fix the findings the user picked, and only those. Fixes happen HERE in the main thread following the project's TDD discipline — the stations' checking passes never edit files. Get the project's full build and test suite green before moving on.

5. **Another round, still no stamp**

   Run `/speclink-review` again, then `/speclink-verify` again, each taking the **stop without stamping** exit as before. Their validation passes cover every fix made since their frozen points — including the ones the other station's findings asked for. Then go back to step 3: the round ends in the same pause whatever it found. A clean round pauses too — report that both stations are green and let the user decide whether to close out.

6. **Closing stamps, back to back**

   Only on the user's say-so. Re-enter review, then verify, telling each station explicitly that this is the quality timeline's **closing stamp call** — that phrase switches off their defer-the-stamp exception for this one call. Each station's own rules then decide the mechanics: an untouched clean last round stamps directly; content that moved since gets one validation pass in the same call, and the cleared round stamps immediately. Do NOT edit anything from here to archive — with zero edits between them, both stamps stay green.

7. **Archive — a recommendation**

   Both stamps are green: recommend `/speclink-archive` and leave the run to the user.

**Edge cases**

- **Changed your mind after a stamp** (one station already stamped, and only then the other is wanted): do NOT redo the stamped station. Run the new station, accept that the earlier stamp shows as changed-since in the meantime, and let archive settle it back to done — archive records that a stamp exists, it does not recompute freshness. Re-running a stamped station means a fresh full discovery pass for no gain.
- **One station only, or neither**: this skill does not apply. Call the station directly; its stamp-when-done default is already correct when no other station's fixes are coming.

**Guardrails**

- Never restate or override a station's checking, ticket or stamping rules — route to the station and follow what it says
- Both checking passes finish before any fixing starts; neither stamp lands before both stations' re-validations are clean
- Every round ends with step 3's pause, a clean round included — never fix, stamp or archive without the user's answer
- Fix only what the user picked; the findings they passed on stay in their tickets, unfixed and unargued
- No edits between the two stamps, and none between them and archive
- A station's refusal or error stops this flow and is reported as-is — do not work around it
