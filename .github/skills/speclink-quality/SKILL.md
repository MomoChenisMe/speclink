---
name: speclink-quality
description: "Use when both quality stations should run over one change, pausing after every round for the user's call: both check without stamping, their findings are reported together and the skill stops — for only one station, call review or verify directly."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.50.5"
  generatedBy: "Speclink"
---

**Conversation language**: write to the user in the language they use with you — progress updates, questions, and summaries. This skill is written in English, and its example messages show structure, not language: translate their prose, and keep commands, file paths, code, and fixed tokens (severity labels, status words) as they are. When a step sets the language of its own output — a document, a report, or the way it presents results and asks about them (for example through `locale`) — that step decides for that output. When you edit a document that already exists and no step sets its language, keep the language it is written in. When you run in a forked context that cannot see the conversation, use the language of the request you were given.

Run both quality stations over one change as a single pass that pauses after every round: `/speclink-review` and `/speclink-verify` each do their checking WITHOUT stamping, then this skill reports both stations' findings together and STOPS for the user's call on what to fix, when to stamp, and whether to archive. Nothing is fixed, stamped or archived without their answer. Use this when both stations are known up front to be in play. Running only one station does not go through this skill — call that station directly and let it keep its own stamp-when-clean default and its own way of sending sub-agents.

**Input**: Optionally specify a change name after `/speclink-quality` (e.g., `/speclink-quality add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous, resolve it BEFORE step 1: run `speclink list --json` and prompt with the available changes, then pass the same name to every station call.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Asking the user**: ask with the **AskUserQuestion tool**; when that tool is not available, ask the same question with the same options as plain text and wait for the user's response.

**What this skill owns**

The ORDER of the two stations, the moment their sub-agents go out, and the pause that ends every round — nothing else. What each station checks, how it freezes its scope, what its briefs say, how it records its ticket, how it triages findings and what its stamp means all belong to `/speclink-review` and `/speclink-verify` — this document never restates them, and when it appears to disagree with a station's own instructions, the station wins. Follow each station's skill as written; this skill only decides when each step of a station runs, when the sub-agents are sent, which exit a station takes, and when to hand the decision back to the user.

**Why the order matters**

A station's stamp freezes the content fingerprint of the files in its scope. Every edit after that — including the fix for the OTHER station's findings — turns the stamp stale ("done, but changed since"). Holding BOTH stamps until every fix has landed and been re-validated keeps both green all the way to archive: no stamp exists yet when a fix lands, so no stamp can go stale. The tickets are written review first, verify second; the checks themselves do not depend on each other, so their sub-agents run at the same time.

**Why every round pauses**

Which findings are worth fixing, and whether the change is ready to stamp, are the user's calls — not this skill's. So every round ends the same way: report both stations, then stop and ask. Never fix a finding, stamp a station or recommend your way past a decision on your own initiative. A clean round is not an exception: "both stations green" is something to report and wait on, like everything else.

**Entry condition**: the change's code tasks are all complete (`codeRemaining` is 0 in the apply payload; `[M]` manual tasks are excluded and may still be open — say so in the report when they are). That is the stations' own precondition — when it is not met, each station shows its own behavior (review refuses and stops; verify reports a mid-flight check-in instead of landing a ticket): relay that outcome verbatim. This skill adds no gate of its own, never pre-checks on the stations' behalf, and never swallows a station's error.

**Steps**

1. **Prepare both stations**

   For the same change, walk `/speclink-review` steps 1–4 (choose the change, the code-task gate and the project review rules, the ticket check and the frozen scope, the artifacts), then `/speclink-verify` steps 1–5 (choose the change, the status, the artifacts and task progress, the branch on the call, the frozen scope), each exactly as its skill writes them. When a station stops during its preparation — a refusal, a mid-flight check-in, `needsInput`, a scope failure — relay its outcome verbatim and follow what that station says; send no sub-agent for either station — a round checks both stations or neither, and the resume rule for an empty validation patch is the one exception.

2. **Send the three checks in one message**

   Compose the review station's Standards and Correctness briefs (review step 5) and the verify station's checking brief (verify step 6), each by its own station's rules, and send all three read-only sub-agents in ONE message — do not wait for the review ticket before sending the verify check. If your harness cannot spawn sub-agents, or cannot send three at once, fall back the way the stations do: run the three checks yourself one after another and keep each report separate.

3. **Close both stations, review first, no stamp**

   Once all three have reported, finish `/speclink-review` steps 6–9 (present, triage, record the round), then `/speclink-verify` steps 10–13 (report, triage, record the round): the review ticket is written first, the verify ticket second. At each station's closing question, take the **stop without stamping** exit — the tickets and their frozen snapshots stay for the rounds that follow. A clean pass takes the same exit; each station's own quality-timeline exception covers it.

   Do not ask the user anything between the two stations: this skill's only pause is step 4, and it takes priority over the stations' own closing questions.

4. **Stop and ask — the round's pause**

   Both stations have reported. Summarize their findings TOGETHER — grouped by station, must-fix separated from the rest — and keep the review station's opening line that names the review rules it applied. List on their own, ahead of everything else, the findings whose fix would touch an artifact (specs, design, tasks): the user decides those first, and the stations' verification gates say how such a fix is handled. Then STOP: put the next step to the user (see **Asking the user**). Make no edits before the answer arrives.

   The options:

   - **Fix everything** — every finding from both stations.
   - **Fix a selection** — the user names which ones; the rest stay in the tickets, unfixed.
   - **Fix nothing and stop** — end the pass right here. Both stations already left through their **stop without stamping** exit, so both tickets and their frozen snapshots stay on disk, no stamp lands, and nothing is archived.
   - **Go to the closing stamps** — offer this option ONLY when both stations' must-fix sets are empty. While any must-fix is outstanding it is not on the menu: must-fix-cleared-before-the-stamp is the stations' rule and this skill does not route around it. When must-fix findings the user passed on still sit in a ticket, the option must say so up front: that station's stamp lands **with reservations** (the station's `--accept`), and choosing the option here IS the explicit say-so the stations require — never presume it from anything less. Leftover SUGGESTION-level findings are not reservations: a station whose ticket carries only SUGGESTIONs stamps clean, no authorization needed.

   Options with nothing to act on simply do not appear — a round where both stations found nothing offers the last two.

   **Top-severity findings left unfixed are confirmed here, once.** When the answer — **go to the closing stamps**, or **fix a selection** — leaves must-fix findings of the top severity unfixed, list each one verbatim and ask once whether to accept them (see **Asking the user**). Only a yes goes on; a no returns to the options above. Every later `/speclink-review` call in this run then states that the top-severity findings were listed verbatim and confirmed one by one, and names them — that hand-over is the confirmation the review station would otherwise ask for.

   A **fix everything** or **fix a selection** answer continues to step 5; **fix nothing and stop** ends here; **go to the closing stamps** jumps to step 7.

5. **Fix exactly what was chosen**

   Fix the findings the user picked, and only those. Fixes happen HERE in the main thread following the project's TDD discipline — the stations' checking passes never edit files. Then pass both stations' **Verification gate** before moving on — the gate decides which tests run, what the report names and how an artifact edit is handled; when its intent-or-scope case applies, this run ends as **scope changed** in both stations.

6. **Another round, still no stamp**

   Repeat steps 1–3: prepare both stations (their scopes now freeze validation patches that cover every fix made since their frozen points — including the ones the other station's findings asked for), send the three checks in one message, then close the review station and the verify station, each through the **stop without stamping** exit. Then go back to step 4: the round ends in the same pause whatever it found. A clean round pauses too — report that both stations are green and let the user decide whether to close out.

7. **Closing stamps, back to back**

   Only on the user's say-so. Re-enter review, then verify, telling each station explicitly that this is the quality timeline's **closing stamp call** — that phrase switches off their defer-the-stamp exception for this one call. Where a ticket still carries must-fix findings the user chose to leave, relay their step-4 answer as the explicit authorization to stamp **with reservations** (the station's `--accept`) — and, for the review station, the step-4 hand-over of the top-severity findings; a reservation stamp never lands unannounced. A ticket left with only SUGGESTION-level findings stamps clean without `--accept`. Each station's own rules then decide the mechanics: an untouched last round whose must-fix set is empty stamps directly; content that moved since gets one validation pass in the same call, and the cleared round stamps immediately. Do NOT edit anything from here to archive — with zero edits between them, both stamps stay green.

8. **Archive — a recommendation**

   Both stamps are green: recommend `/speclink-archive` and leave the run to the user. When the change still carries unchecked `[M]` manual tasks, the recommendation says so: the manual work has to be completed and those tasks checked off before archive will let the change through.

**Resuming after an interrupted session**

A session closed in the middle of a fix round resumes by running `/speclink-quality` again. The tickets live in the change folder and the frozen snapshots in `.speclink/review-scopes/<change>/`, so each station's scope sees its open ticket and freezes a validation patch that holds every fix landed before the close. A resumed run adds three rules to the steps above:

- Pass both stations' **Verification gate** before step 1 — the earlier session's test state is unknown, and a fix the gate still needs must land before the scopes freeze.
- A station whose validation patch has no files while its last round still carries must-fix findings may still have an artifact fix to judge, because the patch leaves the spec directory out. List the change folder's uncommitted and untracked files with `git status --porcelain -- "openspec/changes/<change-name>/"`, the two tickets left out. Git cannot tell whether those edits came before or after the last round, so:
  - Nothing listed → that station sends no sub-agent and records no round this time: a round with nothing to judge would count as "no progress" and fail. The summary says that station had no fix to judge. When both stations are in that state, go straight to step 4 with the last rounds' findings.
  - Files listed → show the list and ask the user (see **Asking the user**) whether the earlier session fixed findings by editing those artifacts. A yes runs the station's validation pass, its brief carrying the edited passages verbatim as the gate's artifact case says; a no takes the nothing-listed path above.
- Tell the user not to commit fixes between the close and the resume: a validation patch picks up fixes by content movement and the dirty state, and a commit can drop a new file the snapshots never captured out of it.

The earlier session's fix-or-skip answers are not kept: step 4 lists only the findings still unresolved and asks again.

**Edge cases**

- **Changed your mind after a stamp** (one station already stamped, and only then the other is wanted): run the new station. If its rounds end without touching any code, the earlier stamp still holds and archive proceeds. But every fix that lands turns the earlier stamp stale, and archive's stale-stamp gate then refuses until that station is re-run — a fresh discovery pass and a new stamp. That re-run is the cost of editing after a stamp; running both stations through this skill's hold-both-stamps timeline is what avoids it.
- **One station only, or neither**: this skill does not apply. Call the station directly; its stamp-when-done default is already correct when no other station's fixes are coming.

**Guardrails**

Check these before you report:

- [ ] Never restate or override a station's checking, brief, ticket or stamping rules — route to the station and follow what it says
- [ ] Every checking round prepares both stations, sends the three sub-agents in one message, and writes the review ticket before the verify ticket
- [ ] Both checking passes finish before any fixing starts; neither stamp lands before both stations' re-validations are clean
- [ ] Every round ends with step 4's pause, a clean round included — never fix, stamp or archive without the user's answer
- [ ] Findings whose fix touches an artifact are flagged at the pause; top-severity findings left unfixed are confirmed there once and handed to the review station
- [ ] Fix only what the user picked; the findings they passed on stay in their tickets, unfixed and unargued
- [ ] No round starts before both stations' verification gates pass — on a resumed run too
- [ ] No edits between the two stamps, and none between them and archive
- [ ] A station's refusal or error stops this flow and is reported as-is — do not work around it

## Next steps

Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.

- Both stamps landed and the work is in the main checkout → `/speclink-archive <change-name>`
- Both stamps landed inside a worktree → `/speclink-worktree-merge <change-name>` first; archive runs only from the main checkout
- The round ended without stamping → nothing downstream is suggested; the round's pause is where this skill stops
