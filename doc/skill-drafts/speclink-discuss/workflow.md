# speclink-discuss Workflow

Conduct a focused, role-aware discussion that produces a structured, iterative document under `.speclink/discussions/<id>/discussion.md` (LocalProvider) or the equivalent provider-stored entity.

This file describes the **host-agnostic workflow logic**. Concrete invocation syntax is provided by `bindings/bash.md` or `bindings/tool.md`. All operations are referenced by their **canonical id** (see `doc/protocol/operations.md`).

**IMPORTANT: Discuss mode is for thinking, not implementing.** You may read files, search code, and investigate the codebase, but you must NEVER write code or modify any project file other than the discussion document via the patch operations below. If the user asks you to implement something, remind them to exit discuss mode first (e.g., `/speclink-propose`). You MAY update the discussion document via `discuss.patch` / `discuss.conclude` — that is capturing thinking, not implementing.

**This is a task-oriented discussion.** Every discussion has a topic, works toward a goal, and ends with a clear conclusion. Unlike open-ended exploration, discuss mode converges.

**Discussion is independent of any change.** A discussion may:

- Lead to a new change (`Capture to: change <name>`)
- Lead to ingest of an existing change (`Capture to: ingest <change>`)
- Result in a vocabulary fix (`Capture to: LANGUAGE.md`)
- End without action (`Capture to: none` — pure exploration)

---

## Input

The argument after `/speclink-discuss` is the topic. Could be:

- A design question: "should we use WebSockets or SSE?"
- A problem to solve: "the auth system is getting unwieldy"
- A change name: invoke with `--about add-dark-mode` to discuss in context of that change
- An architecture decision: "how to structure the plugin system"
- A vague idea that needs sharpening: "real-time collaboration"

## Prerequisites

This skill requires a working SpecLink invocation surface. If any operation fails with `cli.not_found` / `sdk.not_initialized` or similar, report the error and STOP. Also verify the project is initialized — invoke `project.status`.

---

## Before You Speak

Before asking anything, load shared vocabulary, scout the codebase, resolve discussion identity, then resolve role + fetch role-aware instructions.

### Step 0: Load shared vocabulary

Read `.speclink/LANGUAGE.md` (LocalProvider) or the equivalent vocabulary entity via your AI tool's file-read facility. This file is the project's canonical vocabulary — terms with `definition`, `avoid`, and `why` notes.

- **If the file exists**: scan the canonical terms and their avoided synonyms. Prefer the canonical term when you summarize, capture conclusions, or update artifacts. If you notice a relevant `avoid` synonym in the user's topic or in the artifacts you read, plan to surface that as vocabulary drift in the conclusion.
- **If the file does not exist**: continue silently with the normal flow. A missing vocabulary file is not an error.

### Step 1: Extract search terms

Pull 2-5 keywords from the user's topic.

### Step 2: Scout the codebase

Use your AI tool's file-search facility to find related source files (not docs, not tests — source code). Spend no more than a few seconds. Read up to 5 of the most relevant files found.

### Step 3: Resolve discussion identity

**3a. Check existing iterating discussions**: invoke `discuss.list` with `{ include_converged: false }` (active discussions only).

If an existing discussion matches the user's topic semantically, ask the user to choose: resume `<existing-id>` or start new.

**3b. Derive kebab-case topic id** from the topic. "Should we use SSE or WebSocket?" → `sse-vs-websocket`. Strip archive-style date prefixes if present.

**3c. Determine linked changes**:

- `--about <change>` was provided → use it
- Conversation references a change → ask the user to confirm linking
- Otherwise → unlinked

### Step 4: Initialize or resume the discussion

**For a new discussion**: invoke `discuss.new` with `{ topic, linked_change_id?, role?, background? }`.

**For resuming**: invoke `discuss.show` with `{ discussion_id }`. Read the full document to understand existing Background, Open Questions, Decisions Made, and previous Rounds.

### Step 5: Resolve role

`discuss` is the only role-aware skill in SpecLink. Roles tailor the perspective (questions to ask, concerns to surface).

**5a.** `--role <id>` provided → use it.

**5b.** Otherwise invoke `config.read` and check `config.default_role` → fallback.

**5c.** Neither → invoke `instructions.get` with `{ kind: "discuss", change_id?: linked_change_id }` (engine returns role list in `availableRoles[]`). Present the user the 2-4 most relevant (typically `pm` / `sa` / `rd` / `qa` plus any custom roles defined in config).

**Role can switch mid-discussion.** When the user requests a different perspective, close out the current round (Step 9c), then re-fetch instructions with the new role.

### Step 6: Fetch role-aware instructions

Invoke `instructions.get` with `{ kind: "discuss", change_id?: linked_change_id, role: <role-id> }`.

The result includes:

- `instruction`: role-specific prompt body — **internalize this; it shapes your reasoning**
- `context`: project context from `config.yaml#context`
- `linked_changes_context`: snapshots of linked changes (state, summary, artifacts)
- `locale`, `rules`: from config

### Step 7: Pick a mode

- **3+ related source files found** → **Assumptions mode**: you have enough context to form opinions. List your assumptions, let the user correct.
- **Fewer than 3 related source files found** → **Interview mode**: not enough code to base assumptions on. Fall through to "How to Discuss" below and ask questions one at a time.

Announce which mode you picked and why.

#### Assumptions mode

When you enter assumptions mode, present 3-5 assumptions. Each one MUST include:

1. **Approach**: what you'd do and why (from your role's perspective)
2. **Evidence**: file path(s) that informed this assumption
3. **If wrong**: concrete consequence of getting this wrong

Example:

```
### My assumptions (SA perspective)

1. **New CLI command goes in `crates/speclink-cli/src/commands/`**
   Evidence: existing commands are in `crates/speclink-cli/src/commands/`
   If wrong: we'd need to create a new module and register it

2. **Use existing `Provider` trait for storage**
   Evidence: `crates/speclink-core/src/provider.rs` defines the trait
   If wrong: parallel storage paths cause inconsistency

3. **State change goes through a transition method, not direct DB writes**
   Evidence: all state mutations route through `Engine::transition`
   If wrong: bypassing the transition skips validation
```

After presenting, ask: **"Which of these are wrong?"**

- If the user says all are fine → proceed to Convergence with these as established context.
- If the user flags corrections → for each one, ask ONE focused follow-up question to understand their intent, then proceed to Convergence with the corrected understanding.

#### Mode switching

The user can switch modes at any time during the discussion:

- "Ask me questions instead" / "one at a time" → switch to interview mode
- "Just list your assumptions" / "what do you think?" → run the codebase scout if not done yet, then switch to assumptions mode

### Step 8: Interface depth check (conditional)

After the codebase scout, evaluate whether the topic introduces a new architectural seam. Run this check **only** when the topic involves at least one of:

- A **new module** (a new Rust crate, file under `crates/speclink-core/src/`, or a new top-level module in the workspace)
- A **new CLI subcommand** (a new `speclink <subcommand>` exposed to AI / human)
- A **new Provider trait method** or a new Engine library API method
- A **new storage abstraction** (new state.db table, new file format under `.speclink/`, new adapter over existing storage)

If none of those conditions apply, **skip this check**. Topics that only change copy, config defaults, documentation wording, or other non-architectural surfaces SHALL skip the depth check entirely. The vocabulary load from Step 0 still happens; nothing else from this step runs.

When the check is triggered, work through these four questions before you finalize assumptions or interview answers:

1. **Seam location** — where does the boundary belong? Name the module, file, or trait that owns the new contract.
2. **Adapter count** — is there exactly one adapter on this path, or are several thin wrappers stacked on each other?
3. **Depth** — what behaviour is hidden behind the interface? If the answer is "nothing — it just forwards calls", the seam is too shallow.
4. **Deletion test** — if you deleted this module today, what would break? If nothing meaningful breaks, the module is a pass-through and probably should not exist.

Surface the answers in the conclusion (or the assumptions list) so the depth question is part of the captured decision, not an internal note.

---

## How to Discuss

_This section applies to interview mode — either chosen automatically (insufficient code context) or switched to manually by the user._

**One question at a time.** Don't dump a list of 10 questions. Ask the most important one, listen, then follow up. Let the conversation breathe. If the user's initial description or previous answers already cover a question, skip it — don't ask what you already know.

**Propose concrete options.** When exploring approaches, present 2-3 specific options with trade-offs — not abstract possibilities. Use comparison tables when helpful:

```
| Approach      | Pros              | Cons              |
|---------------|-------------------|-------------------|
| WebSockets    | Real-time, bidir  | Complex, stateful |
| SSE           | Simple, HTTP      | One-way only      |
| Polling       | Simplest          | Latency, waste    |
```

**Ground in reality.** Investigate the actual codebase when relevant. Map existing architecture, find integration points, surface hidden complexity. Don't just theorize.

**Visualize freely.** Use ASCII diagrams when they clarify thinking:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Client  │────▶│  Server  │────▶│    DB    │
└──────────┘     └──────────┘     └──────────┘
```

**Challenge assumptions.** Including the user's and your own. Ask "do we actually need this?" Apply YAGNI.

**Be direct.** If you have a recommendation, say it. Don't hedge endlessly.

**No empty validation.** Never pad responses with hollow affirmations:

- ~~"That's an interesting approach"~~ → State what specifically is interesting and why
- ~~"There are many ways to think about this"~~ → Name the 2-3 concrete ways and their trade-offs
- ~~"That could work"~~ → Explain why it would or wouldn't work, and under what conditions
- ~~"Great question"~~ → Just answer the question
- ~~"You raise a good point"~~ → Engage with the point directly

**Push for specifics.** When the user gives a vague answer, don't accept it — dig deeper.

```
User: "We should make it more modular"
Bad:  "That sounds good. How would you like to proceed?"
Good: "What would you split out? Are we talking separate crates,
       feature flags, or a plugin interface? Each has very different cost."
```

```
User: "Performance might be an issue"
Bad:  "Good point, we should keep performance in mind."
Good: "What's the threshold? Are we talking sub-100ms response time,
       handling 1000 concurrent users, or keeping memory under a budget?
       The answer changes the architecture."
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

**If the user wants to move faster.** Sometimes the user signals impatience — "let's just go with X", "I don't want to overthink this", "can we move on?". Respect their pace:

1. **First time**: Briefly flag if there's an important unresolved question — one sentence, not a lecture. "Before we commit to X, worth noting that Y could affect Z. Want to address it or move forward?"
2. **If they push again**: Respect it. Skip remaining questions, go straight to convergence with the best conclusion you can form from what's been discussed. Don't push back a second time.

The goal is thoroughness, not interrogation. One nudge maximum.

---

## SpecLink Awareness

You have full context of the SpecLink system. Use it naturally.

### Step 9: Capture & flush

As the discussion unfolds, write to the discussion document via **Section patch**. Engine enforces append-only on `decisions_made` and `rounds`.

**9a. Initialize Background** (if empty / on new discussions):

If `discuss.new` was invoked with `background: null` in Step 4, invoke `discuss.patch` with `{ discussion_id, section: "background", content: <1-3 paragraph markdown describing the situation> }`.

**9b. Capture a new decision** (append-only):

Invoke `discuss.patch` with `{ discussion_id, section: "decisions_made", content: "[YYYY-MM-DD <role>] <decision in one line>" }`.

Engine appends to the existing decisions_made section (will not replace).

**9c. Open Questions changed** — added, closed, or refined (replace entire section):

Invoke `discuss.patch` with `{ discussion_id, section: "open_questions", content: <full updated checklist> }`.

**9d. End of a coherent round** — role shift, topic pivot, or substantial reasoning chunk (append-only):

Invoke `discuss.patch` with `{ discussion_id, section: "rounds", content: <markdown of round content, e.g., "### Round N — YYYY-MM-DD — <role>\n\n<key insights>"> }`.

**Flush rhythm**: patch at every round boundary, NOT only at skill end. If the conversation crashes, the most recently committed round must already be on disk. Engine enforces this via the round-boundary invariant (design.md §8.5).

### Handle role switches

If the user wants a different role's perspective:

1. Patch the current round (close out the existing role's contribution via Step 9d).
2. Re-invoke `instructions.get` with `{ kind: "discuss", role: <new-role> }` to internalize the new prompt.
3. Begin a new round under the new role.

### Capture-to mapping

When proposing the Conclusion, choose the right `Capture to` target:

| Insight type | Capture to |
|---|---|
| A new change worth opening | `change <new-name>` |
| An existing change needs reshape | `ingest <change>` |
| A vocabulary fix (canonical term clarified) | `LANGUAGE.md` |
| Pure exploration or deferred decision | `none` |

**Vocabulary drift** means the discussion surfaced a recurring concept that is missing, ambiguous, or pulling away from the shared vocabulary loaded in Step 0. Examples: the topic uses a term that the vocabulary lists as an `avoid` synonym, or the discussion repeatedly names a concept that has no entry yet. When this happens, name it as vocabulary drift in the conclusion and direct the capture to `.speclink/LANGUAGE.md`.

### Transition to action

When the discussion converges on building something, propose the next skill:

- `Capture to: change <name>` → "Ready to formalize this? Run `/speclink-propose <name>`"
- `Capture to: ingest <change>` → "Ready to ingest? Run `/speclink-ingest <change>`"
- `Capture to: LANGUAGE.md` → propose the canonical term and `avoid` entries inline; the user can edit `.speclink/LANGUAGE.md` directly
- `Capture to: none` → "Recorded. Discussion archived."

### Step 10: Conclude

When the user confirms convergence, invoke `discuss.conclude` with `{ discussion_id, conclusion: <markdown body> }`.

Engine flips state to `converged` and writes the Conclusion section. **Converged is terminal — the discussion is preserved as an audit trail and cannot be patched further. It is NOT deleted automatically.** If the user later wants to remove it, they can invoke `discuss.delete` explicitly (with `force: true`).

If the user invoked `/speclink-propose` using this discussion as source, the engine records that change in the discussion's `linked_changes` automatically.

### Pause mid-flight

If the user wants to step away, **just leave the discussion in `active` state** — it persists and can be resumed via `/speclink-discuss <topic-id>` later. There is no separate `park` operation; `active` discussions can sit indefinitely (design.md §6.4 explains the design rationale for not introducing a parked flag).

If the user wants to drop the discussion entirely, invoke `discuss.delete` with `{ discussion_id, force: true }` (destructive).

---

## Concurrency & Errors

- `lock.not_acquired` (when fetching linked change context, another agent may hold the lock) → engine handles jittered backoff retry.
- `role.unknown` → stop; list configured roles via `instructions.get` `{ kind: "discuss" }` and the `availableRoles[]` field; ask user to pick or add one in config.yaml.
- `config.malformed` → surface engine warnings (warnings[]) to the user; never auto-modify config.
- `discussion.not_found` → suggest `discuss.list` to find correct id.
- `discussion.already_converged` → cannot patch a converged discussion; suggest creating a new discussion linked to it (`discuss.new` with `linked_change_id` if applicable).
- `discussion.locked` (trying to patch a section other than `conclusion` on a converged discussion) → same handling as `already_converged`.
- `state.etag_mismatch` (rare; another agent patched mid-session) → re-invoke `discuss.show` to get latest etag, decide whether to merge changes or abandon current patch.

## Exit Criteria

The skill is complete when ONE of:

- Discussion converged + Conclusion written (`discuss.conclude` succeeded)
- User explicitly exits without saving (no patches committed in this session)

---

## Guardrails

- **Don't implement** — Never write code or modify project files (other than the discussion document via the patch ops above). Suggesting `/speclink-propose` is fine; invoking it is not.
- **Don't leave without a conclusion** — If the user tries to end without converging, summarize where things stand and state what's unresolved. Leaving the discussion in `active` state is fine (it can be resumed later); abandoning without summarizing is not.
- **Don't fake understanding** — If something is unclear, dig deeper.
- **Don't overwhelm** — One question at a time, not a barrage.
- **Don't over-engineer** — Challenge complexity. Prefer simpler solutions.
- **Do visualize** — A good diagram is worth many paragraphs.
- **Do explore the codebase** — Ground discussions in reality.
- **Do be opinionated** — Have a recommendation. The user can disagree.
- **Do flush early** — Patch the discussion at every round boundary, not just at skill end. Crash recovery depends on it.
- **Do switch roles cleanly** — Close out the current round before re-fetching instructions with a new role.
- **NEVER** pass `force: true` to destructive operations (the AI must not invoke `discuss.delete` with `force: true` autonomously).
