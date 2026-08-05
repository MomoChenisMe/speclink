---
name: speclink-discuss
description: "Have a focused discussion that is recorded to a discussion document"
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.11.0"
  generatedBy: "Speclink"
---

Have a focused discussion about a topic and reach a conclusion.

**IMPORTANT: Discuss mode is for thinking, not implementing.** You may read files, search code, and investigate the codebase, but you must NEVER write code or implement features. If the user asks you to implement something, remind them to exit discuss mode first (e.g., start a change with `$speclink-propose`). You MAY create Speclink artifacts (proposals, designs, specs) if the user asks—that's capturing thinking, not implementing.

**This is a task-oriented discussion.** Every discussion has a topic, works toward a goal, and ends with a clear conclusion. Unlike open-ended exploration, discuss mode converges.

**Input**: The argument after `$speclink-discuss` is the topic. Could be:

- A design question: "should we use WebSockets or SSE?"
- A problem to solve: "the auth system is getting unwieldy"
- A change name: "add-dark-mode" (discuss in context of that change)
- An architecture decision: "how to structure the plugin system"
- A vague idea that needs sharpening: "real-time collaboration"
- A document path: `docs/plans/realtime.md` — a plan you wrote by hand, a plan-mode output, a doc under the repo, or any readable path (see "Document input" below)

**Not every topic is a discussion.** If the request is really a question — the user wants to understand how something works or whether something is feasible, and no decision hangs on the answer — answer it directly in the conversation and do not open a discussion record. A discussion exists to settle something; understanding-seeking without a verdict is ask-shaped, not discuss-shaped. When in doubt, start talking without a record: the record is only created at the first substantive round (see below), so nothing is lost by waiting.

---

## Recording the discussion (speclink)

Unlike an ephemeral chat, **every speclink discussion is persisted to a document** (`openspec/discussions/<slug>.md`) so the conversation keeps its thread across turns and sessions, and so a later `$speclink-propose --from-discussion <slug>` can seed a proposal directly from it. Drive the record through the CLI — never hand-write the file.

The document has a fixed skeleton — like the proposal template, every discussion record has the same shape:

```
## Context      ← the framing, set once when the record is created (discuss context)
## Rounds       ← ### Round N entries, appended per exchange (discuss add-round)
## Conclusion   ← the decision, written at convergence (discuss conclude)
```

**Document rules** (the record is a Socratic ledger, not a transcript):

1. **One focus per round.** Each recorded round distills exactly one question examined and what it settled — never a dump of everything said.
2. **Rounds are append-only.** Never rewrite an earlier round. When a position changes, a new round names what changed and why — the reasoning trail is the point.
3. **Record what was ruled out, with the reason.** Rejected options are the most valuable part of the record: they stop future readers (and future you) from re-litigating settled ground.
4. **Keep an open-questions ledger.** Each round ends with the questions still unresolved; the next round picks one of them up. The conclusion must resolve or explicitly defer every remaining one.
5. **Position bullets over prose.** When a Position exceeds one sentence it SHALL be bulleted — a one-sentence verdict first, then `- ` points one per line. A single-line wall-of-text Position is unreadable in every viewer. Focus / Ruled out / Open stay single-line.
6. **The rounds trace the decision tree.** The first round's Position lays out the initial decision space (an ASCII tree is welcome); each later round resolves one node; branches discovered mid-round are recorded in that round's Open. The Open ledger is thus always the exact frontier of the unexplored tree.

**At the start (before Step 0):**

1. Check for an existing open discussion on this topic:
   ```bash
   speclink discuss list --json
   ```
2. If one matches the topic and its `status` is `open`, resume it (reuse its `slug`) — the record already exists; every later `add-round`/`conclude` call uses it.
3. Otherwise **do not create the record yet.** Derive an English kebab-case slug from the topic (translate when the topic is not English — e.g. 「看板搜尋列」 → `board-search-bar`), announce it ("This discussion will be recorded as `<slug>` once it has substance."), and proceed — scout, pick a mode, present your first assumptions or question with nothing on disk. A mis-invocation or a topic answered in one exchange leaves no file behind.

**Create the record at the first substantive round** — the moment an exchange actually moves the topic (your assumptions list drew confirmations or corrections, an interview answer settled something). Right before recording that first round, run:

```bash
speclink discuss new "<topic>" --slug <english-kebab-slug>
```

Always pass `--slug` with the English kebab-case slug you derived — the slug names the record file; the topic stays in the user's language verbatim. Without `--slug` the filename falls back to deriving from the topic, so non-English topics produce non-English filenames. Write the Context section (below), then `add-round`. From here on the record is live and every exchange is persisted.

**When the record is created**, fill the Context section once — the framing a future reader (or `propose`) needs before the rounds make sense:

```bash
speclink discuss context <slug> --stdin <<'CTX_EOF'
What prompted this discussion, the mode chosen (assumptions | interview) and why,
and the related changes/specs found by the codebase scout.
CTX_EOF
```

**Source doc convention** — when the topic named a document (see "Document input" below), the Context SHALL carry one line naming it:

```
Source doc: <path>
```

That line is the mechanical marker a later `$speclink-propose --from-discussion <slug>` looks for to know there is an underlying document to read. Three rules travel with it:

- **Evidence cites the document by reference, not by transcription.** When a round's Evidence points at the document, name the section heading or quote a short phrase from it.
- **The record stores the outcome of the discussion only.** It SHALL NOT embed the planning document in full — the document stays where it is, and the record holds the decision diff against it.
- **Never modify the user's original document.** Corrections live in the record; merging document and decisions is `propose`'s job, not yours.

When the topic named no document, none of this applies: the Context has no `Source doc:` line, there is no extra file-reading step, and recording proceeds exactly as it does today.

**After each round** (each Assumptions list you present, or each interview question-and-answer that moves the topic forward), persist a concise summary so the record shows how the thinking evolved:

```bash
speclink discuss add-round <slug> --mode assumptions --stdin <<'ROUND_EOF'
**Focus**: the one question this round examined
**Position**: one-sentence verdict of the direction taken, expanded as bullets:
- one point per line — the decision detail and the evidence (files, probe results) behind it
- keep bulleting until the position is fully stated; never fold it back into one long line
**Ruled out**: options eliminated this round — each with the reason it lost
**Open**: questions still unresolved, for the next round to pick up
ROUND_EOF
```

Use `--mode assumptions` or `--mode interview` to match the mode you picked in Step 3. Omit a line rather than pad it (e.g. no `**Ruled out**` when nothing was eliminated). Keep each round terse — it is a durable summary following the Document rules above, not a transcript. This is the mechanism that keeps a long discussion from drifting off-topic: each round is anchored to the record.

**At convergence**, write the conclusion into the record (see the Convergence and "Capture decisions" sections below):

```bash
speclink discuss conclude <slug> --stdin <<'CONCLUSION_EOF'
**Decision**: ...
**Rationale**: ... (the key trade-off that drove it)
**Rejected alternatives**: ... (each with why it lost)
**Deferred**: open questions intentionally left unresolved — or "none"
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md
**Next**: $speclink-propose --from-discussion <slug>
CONCLUSION_EOF
```

This flips the record's `status` to `concluded`. The step logic below (vocabulary load, codebase scout, mode selection, interface depth check, convergence, conclusion capture) is unchanged — recording sits alongside it.

**Fast path**: when the user wants to go straight from the conclusion to a change without a full propose round, offer:

```bash
speclink discuss promote <slug>            # change name defaults to the slug
speclink discuss promote <slug> --name <change-name>
```

This scaffolds the change, prefills the proposal's Why from the conclusion, and links both sides (`from_discussion` in the change metadata, `status: promoted` + `promoted_to` in the record). One discussion can fan out into several changes — promote (or `$speclink-propose --from-discussion`) again and `promoted_to` accumulates each name; the discussion is archived automatically when the last of its changes is archived. The remaining artifacts are still created via `$speclink-propose`.

**Conclusion routed to an EXISTING change**: when the conclusion's **Capture to** points at a change already in flight (the decision updates its artifacts instead of spawning a new one), run link first, then hand off to ingest:

```bash
speclink discuss link <slug> <existing-change>
```

`link` forges the change-side chain (`from_discussion` in the change metadata) without scaffolding anything, so drawer links and auto-archive engage — the discussion is archived automatically when the last linked change is archived. Unlike promote, `link` does NOT mark the discussion 已轉出 (`promoted`): that reflection is sealed by `$speclink-ingest`, which folds the decision into the change's artifacts and then runs `speclink discuss seal` — so the discussion flips to promoted only once its content has actually landed, never at link time. Then run `$speclink-ingest <existing-change>` to fold the decision in and seal. Without the link, a concluded-then-ingested discussion sits on the board forever with nothing to archive it.

**Lifecycle**: a discussion that concluded without spawning a change (an explicit "don't do this" is a valid outcome) should be closed out with:

```bash
speclink discuss archive <slug>       # → discussions/archive/<created>-<slug>.md
```

Archived discussions stay readable — `speclink discuss show <slug>` falls back to the archive, and `speclink discuss list --archived` lists them. The slug becomes free for a future discussion.

A discussion that turns out not to be needed at all — the user abandons it mid-way, or the topic proved ask-shaped after all — is **discarded**, not archived:

```bash
speclink discuss discard <slug>            # refuses once rounds exist
speclink discuss discard <slug> --force    # delete despite recorded rounds
```

`discard` deletes the live record (archived records are never touched). Once rounds exist it refuses without `--force` — a discussion that examined real trade-offs should keep its reasoning through `conclude` + `archive`, even when the conclusion is "don't do this". Use `discard` freely for records that settled nothing; never leave an abandoned discussion sitting `open`.

---

## Before You Speak

Before asking anything, load the shared vocabulary, then do a quick codebase scout to decide how to run this discussion.

### Step 0: Load shared vocabulary

Run `speclink language show`. It prints the project's canonical vocabulary — terms with `definition`, `avoid`, and `why` notes, plus principles for when legacy terminology may remain.

- **If the command succeeds**: scan the canonical terms and their avoided synonyms. Prefer the canonical term when you summarize, capture conclusions, or update artifacts. If you notice a relevant `avoid` synonym in the user's topic or in the artifacts you read, plan to surface that as vocabulary drift in the conclusion.
- **If the command fails (no vocabulary document)**: continue silently with the normal flow. A missing vocabulary is not an error; do not announce it, do not block, and do not stop to ask the user to create it.

This step runs before the codebase scout, the assumptions list, the interview questions, and the conclusion capture.

### Step 1: Extract search terms

Pull 2-5 keywords from the user's topic. For "search should support fuzzy matching", that's `search`, `fuzzy`, `match`. For "should we add a plugin system", that's `plugin`, `extension`, `module`.

### Step 2: Scout the codebase

Use Grep and Glob to find related source files (not docs, not tests — source code). Spend no more than a few seconds on this. Read up to 5 of the most relevant files found.

This scout exists only to pick the mode (Step 3) — it is not the investigation. Deeper verification happens later, node by node along the decision tree (see "How to Discuss").

### Step 3: Pick a mode

- **3+ related source files found** → **Assumptions mode**: you have enough context to form opinions. List your assumptions, let the user correct.
- **Fewer than 3 related source files found** → **Interview mode**: not enough code to base assumptions on. Fall through to "How to Discuss" below and ask questions one at a time.
- **The topic is a document path** → **Document input** (below): the document supplies the tree already filled in, so skip the "list 3-5 assumptions" opening and triage its claims instead. The scout still runs — it is what you triage the claims against.

Announce which mode you picked and why: "Found `search.rs`, `SearchPanel.svelte`, `search-store.ts` — I have enough context to list my assumptions." or "Didn't find much related code — I'll ask questions instead."

### Assumptions mode

When you enter assumptions mode, present 3-5 assumptions. Each one MUST include:

1. **Approach**: what you'd do and why
2. **Evidence**: file path(s) that informed this assumption
3. **If wrong**: concrete consequence of getting this wrong

Example:

```
### My assumptions

1. **New IPC command goes in `commands/search.rs`**
   Evidence: existing search commands are in `src-tauri/src/commands/search.rs`
   If wrong: we'd need to create a new module and register it

2. **Use the existing `SearchStore` for state**
   Evidence: `src/lib/stores/search-store.ts` already manages search state
   If wrong: parallel state would cause sync bugs

3. **Fuzzy matching runs in Rust, not frontend**
   Evidence: current search scoring is in `search.rs:calculate_score()`
   If wrong: moving to frontend means rewriting the scoring logic in TypeScript
```

After presenting, ask: **"Which of these are wrong?"**

- If the user says all are fine → proceed to Convergence with these as established context.
- If the user flags corrections → for each one, ask ONE focused follow-up question to understand their intent, then proceed to Convergence with the corrected understanding.

### Document input

When the topic names a **file path** rather than a sentence — a plan the user wrote by hand, a plan-mode output, a doc under the repo, or any readable path — read that file and treat it as **someone else's assumptions list**: a decision tree that arrives pre-filled and now has to be stress-tested. The document SHALL NOT be read once as background material and then set aside; it is not colour for opinions you form independently.

Extract every claim the document makes as a tree node, then triage each claim against the codebase:

| Triage            | Meaning                                  | What you do with it                                                                    |
| ----------------- | ---------------------------------------- | -------------------------------------------------------------------------------------- |
| **Confirmed**     | The codebase backs the claim             | Record it with the code evidence — file path, symbol, or probe result                   |
| **Contradicted**  | The document says X, the code does Y     | Name the difference for that claim — doc says X, code does Y — with the evidence for Y   |
| **Real decision** | Nothing in the environment can settle it | Send it to the user, carrying your proposed answer and its Evidence                     |

- **Contradictions are itemized, one claim at a time.** For each contradicted claim, state what the document asserts, what the code actually does, and the evidence for the latter. Summarizing the document, or a blanket "parts of this are out of date", is not triage.
- **Confirmed nodes do not need a user round.** They are settled facts — say so and move on.
- **Only real decisions reach the user**, one at a time, in dependency order (see "How to Discuss").
- The triage IS the first round's Position, and it replaces the "list 3-5 assumptions" opening — the document already listed them.

Recording follows the **Source doc convention** above: the Context carries `Source doc: <path>`, Evidence cites the document by section heading or short phrase, and the original document is never modified.

### Mode switching

The user can switch modes at any time during the discussion:

- **"Ask me questions instead"** / **"one at a time"** → switch to interview mode (the "How to Discuss" section below)
- **"Just list your assumptions"** / **"what do you think?"** → run the codebase scout if not done yet, then switch to assumptions mode

### Step 4: Interface depth check (conditional)

After the codebase scout, evaluate whether the topic introduces a new architectural seam. Run this check **only** when the topic involves at least one of:

- A **new module** (a new Rust crate, file under `src-tauri/src/commands/`, or a new top-level Svelte module).
- A **new IPC command** (a new `#[tauri::command]` exposed to the frontend, or a new front-to-back message shape).
- A **cross-layer Rust ↔ Tauri ↔ Svelte flow** that did not exist before.
- A **new storage abstraction** (new on-disk format, new database table, new file-system layout, new adapter over existing storage).

If none of those conditions apply, **skip this check**. Topics that only change static UI copy, visual styling, documentation wording, or other non-architectural surfaces SHALL skip the depth check entirely. The vocabulary load from Step 0 still happens; nothing else from this step runs.

When the check is triggered, work through these four questions before you finalize assumptions or interview answers:

1. **Seam location** — where does the boundary belong? Name the module, file, or store that owns the new contract.
2. **Adapter count** — is there exactly one adapter on this path, or are several thin wrappers stacked on each other?
3. **Depth** — what behaviour is hidden behind the interface? If the answer is "nothing — it just forwards calls", the seam is too shallow.
4. **Deletion test** — if you deleted this module today, what would break? If nothing meaningful breaks, the module is a pass-through and probably should not exist.

Surface the answers in the conclusion (or the assumptions list, if you are in assumptions mode) so the depth question is part of the captured decision, not an internal note.

---

## How to Discuss

_This section applies to interview mode — either chosen automatically (insufficient code context) or switched to manually by the user._

**Open by laying out the decision space.** Before asking anything, map the decision tree: the root node is "what is this topic actually deciding?", expanded into its sub-decisions with the dependency edges between them. Present the map up front (an ASCII tree works well) so the user sees the shape of the whole problem before the first question.

**Traverse in dependency order — one question at a time.** Don't dump a list of 10 questions. Ask exactly one question per exchange, picked by dependency order: resolve upstream decisions first, because the shape of downstream questions depends on the upstream answers. Listen, then move to the next node. If the user's initial description or previous answers already settle a node, mark it resolved and skip it — don't ask what you already know.

**Every question MUST come with a proposed answer, and the proposal MUST cite Evidence.** This is a hard rule, not a suggestion: each question you ask carries your recommended answer, backed by Evidence — file path(s) or probe results. The user only needs to agree or correct. Never hand the user a bare open question that Evidence could have grounded first. (This is the same Evidence convention assumptions mode already uses, applied per question.)

**Triage every node: fact or decision.** Before resolving a node, classify it. A **fact** is anything the environment can answer — code, file system, tool output; a **decision** is a judgment call only the user can make. Facts MUST be verified yourself with Grep/Read at the node where they arise — never ask the user for a fact, and never answer one from memory. Only genuine decisions go to the user. Verification depth follows the tree: spend deep reads on branches you will actually traverse, and don't pre-read branches that get pruned.

**Propose concrete options.** When exploring approaches, present 2-3 specific options with trade-offs — not abstract possibilities. Use comparison tables when helpful:

```
| Approach      | Pros              | Cons              |
|---------------|-------------------|-------------------|
| WebSockets    | Real-time, bidir  | Complex, stateful |
| SSE           | Simple, HTTP      | One-way only      |
| Polling       | Simplest          | Latency, waste    |
```

**Visualize freely.** Use ASCII diagrams when they clarify thinking:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Client  │────▶│  Server  │────▶│    DB    │
└──────────┘     └──────────┘     └──────────┘
```

System diagrams, state machines, data flows, dependency graphs — whatever helps.

**Challenge assumptions.** Including the user's and your own. Ask "do we actually need this?" Apply YAGNI — the simplest solution that works is often the best.

**Be direct.** If you have a recommendation, say it. Don't hedge endlessly. "I'd go with option B because..." is more useful than "all options have merit."

**No empty validation.** Never pad responses with hollow affirmations. These add nothing and erode trust:

- ~~"That's an interesting approach"~~ → State what specifically is interesting and why
- ~~"There are many ways to think about this"~~ → Name the 2-3 concrete ways and their trade-offs
- ~~"That could work"~~ → Explain why it would or wouldn't work, and under what conditions
- ~~"Great question"~~ → Just answer the question
- ~~"You raise a good point"~~ → Engage with the point directly

If you agree, say why. If you disagree, say why. Empty agreement is worse than honest pushback.

**Push for specifics.** When the user gives a vague answer, don't accept it — dig deeper. The goal is to reach decisions concrete enough to implement.

Bad vs. good:

```
User: "We should make it more modular"
Bad:  "That sounds good. How would you like to proceed?"
Good: "What would you split out? Are we talking separate crates,
       feature flags, or a plugin interface? Each has very different
       cost."
```

```
User: "Performance might be an issue"
Bad:  "Good point, we should keep performance in mind."
Good: "What's the threshold? Are we talking sub-100ms response time,
       handling 1000 concurrent users, or keeping memory under a
       budget? The answer changes the architecture."
```

```
User: "We need better error handling"
Bad:  "Agreed, error handling is important."
Good: "Which errors are causing problems now? Are users seeing
       crashes, silent failures, or unhelpful messages? Let's look
       at the actual error paths."
```

---

## Convergence

Discussions must converge. As the conversation progresses:

1. **Narrow the options** — eliminate approaches that don't fit
2. **Surface the key trade-off** — most decisions come down to one fundamental tension
3. **Make a recommendation** — or help the user make one
4. **State the conclusion clearly** — what was decided, and why

The conclusion should be one of:

- **Design decision**: "We'll use SSE because one-way is sufficient and it's simpler"
- **Direction consensus**: "The auth refactor should split into gateway + provider"
- **Next-step recommendation**: "We need to spike the plugin API first to validate the approach"
- **Explicit deferral**: "We don't have enough info yet. Specifically, we need to know X before deciding"

**Example elicitation**: When the discussion converges on a specific requirement or behavior, propose a concrete example before capturing the decision. Instead of concluding "search should sort by relevance", propose: "So if we have items scored 0.9, 0.3, 0.7, the result order would be 0.9, 0.7, 0.3 — is that right?" This naturally produces `##### Example:` content for the spec and confirms shared understanding with real values.

**If the user wants to move faster.** Sometimes the user signals impatience — "let's just go with X", "I don't want to overthink this", "can we move on?". The user owns the stopping point: the decision tree is a map, not a contract, and convergence never requires every branch to be resolved. Respect their pace:

1. **First time**: Briefly flag if there's an important unresolved question — one sentence, not a lecture. "Before we commit to X, worth noting that Y could affect Z. Want to address it or move forward?"
2. **If they push again**: Respect it. Skip remaining questions, go straight to convergence with the best conclusion you can form from what's been discussed, and record the branches left untraversed under **Deferred** in the conclusion. Don't push back a second time.

The goal is thoroughness, not interrogation. One nudge maximum.

---

## Speclink Awareness

You have full context of the Speclink system. Use it naturally.

### Check for context

At the start, quickly check what exists:

```bash
speclink list --json
```

If the user mentioned a specific change name, read its artifacts for context.

### Capture decisions

When the discussion converges, **proactively present a conclusion summary**. Don't wait to be asked — propose it, and let the user opt out.

Summary format:

```
## Conclusion

**Decision**: [What was decided]
**Rationale**: [Why — the key trade-off that drove this]
**Rejected alternatives**: [What lost, and why]
**Deferred**: [Open questions intentionally left unresolved — or "none"]
**Capture to**: [Where this should be recorded]
```

Where to capture:

| Insight Type               | Where to Capture             |
| -------------------------- | ---------------------------- |
| New requirement discovered | `specs/<capability>/spec.md` |
| Design decision made       | `design.md`                  |
| Scope changed              | `proposal.md`                |
| New work identified        | `tasks.md`                   |
| Vocabulary drift           | `openspec/LANGUAGE.md`    |

**Vocabulary drift** means the discussion surfaced a recurring concept that is missing, ambiguous, or pulling away from the shared vocabulary loaded in Step 0. Examples: the topic uses a term that the vocabulary lists as an `avoid` synonym, or the discussion repeatedly names a concept that has no entry yet. When this happens, name it as vocabulary drift in the conclusion summary and direct the capture to `openspec/LANGUAGE.md`. The conclusion summary SHALL preserve this contract — do not silently rewrite the term in the artifacts without recording the drift.

Present the summary and say something like "I'll capture this to design.md unless you'd rather not." Default to capturing — the user can decline.

### Transition to action

When the discussion converges on building something:

- First record the conclusion in the discussion document: `speclink discuss conclude <slug> --stdin` (see "Recording the discussion" above). This flips its status to `concluded`.
- Then: "Ready to formalize this? `$speclink-propose --from-discussion <slug>`" — propose will seed the proposal from the recorded Decision and rounds.
- Or capture the decision in existing artifacts and continue

---

## Guardrails

- **Do record the discussion** — Announce the intended English kebab-case slug at the start, open the record at the first substantive round (`speclink discuss new` with `--slug`), append a round after each exchange, and `conclude` at the end. The document is the durable thread; keep it current. If the discussion is abandoned instead, `speclink discuss discard` the record — never leave it sitting `open`.
- **Don't implement** — Never write code or implement features. Creating Speclink artifacts and discussion records is fine, writing application code is not.
- **Don't leave without a conclusion** — If the user tries to end without a conclusion, summarize where things stand and state what's unresolved.
- **Don't fake understanding** — If something is unclear, dig deeper.
- **Don't overwhelm** — One question at a time, not a barrage.
- **Don't over-engineer** — Challenge complexity. Prefer simpler solutions.
- **Do visualize** — A good diagram is worth many paragraphs.
- **Do explore the codebase** — Ground discussions in reality.
- **Do be opinionated** — Have a recommendation. The user can disagree.
