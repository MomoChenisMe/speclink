---
name: speclink-improve
description: "Scan the codebase for architectural improvements and record the candidates as a discussion — user-initiated only, never triggered by the model on its own"
disallowedTools: [Edit, Write]
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.20.0"
  generatedBy: "Speclink"
---

Scan the codebase for architectural improvements and turn the best ones into a recorded discussion. This is the mirror image of `/speclink-discuss`: there the user brings the topic, here the model brings the candidates. Everything downstream — rounds, conclusion, promote/link, archive — is the same discussion machinery.

**IMPORTANT: This skill is user-initiated only.** Never trigger it on your own. Do not offer to "run improve" in the middle of another task, and do not start scanning because some code looked messy while you were doing something else. It runs when the user asks for it, and only then.

**IMPORTANT: This skill never implements.** You read files, search code, run git, and record a discussion. You do NOT write application code, refactor anything, or fix what you find. Improvements reach the codebase through the normal route: conclusion → `/speclink-propose` (or `promote`) → `/speclink-apply`. If the user asks you to just fix one of the candidates, tell them to conclude the discussion first and start a change.

**Input**: Optionally a direction after `/speclink-improve` — a module, subsystem, crate, or pain point ("the store layer", "everything around auth", "the CLI feels bloated"). When given, that direction IS the scope and Step 2's inference is skipped. When omitted, Step 2 infers the scope.

**Prerequisites**: This skill requires the `speclink` CLI and `git`. If any command fails with "command not found" or similar, report the error and STOP.

**What counts as an improvement here**: structural deepening — making modules deeper, seams fewer and better placed, complexity concentrated instead of smeared. Behavioural correctness is NOT this skill's job: bugs belong to `/speclink-review`, spec compliance to `/speclink-verify`, security sharp edges to `/speclink-audit`.

---

## The six steps

### Step 0: Load shared vocabulary

Run `speclink language show`. It prints the project's canonical vocabulary — terms with `definition`, `avoid`, and `why`.

- **Succeeds**: use the canonical terms when you name candidates and write the record. A candidate whose description drifts into an `avoid` synonym reads as a different problem than it is.
- **Fails (no vocabulary document)**: continue silently. A missing vocabulary is not an error — do not announce it, do not block.

Architecture vocabulary (seam, depth, adapter, shallow module) stays in this document — it is working vocabulary for the scan, not project vocabulary for `LANGUAGE.md`.

### Step 1: Re-proposal check

Before scanning, find out what has already been decided. Proposing something the user already rejected is worse than proposing nothing.

```bash
speclink discuss list --json
speclink discuss list --archived --json
speclink list --json
```

- **Read the archived discussions' `Ruled out` lines and conclusions.** They are this project's decision record: an option that lost, with the reason it lost. Read the ones whose topic touches your scope (`speclink discuss show <slug>` falls back to the archive).
- **A previously rejected approach SHALL NOT be re-proposed as a candidate** — unless you can name the reason it was rejected AND state concretely why that reason no longer holds (the code it depended on is gone, the constraint was lifted, the trade-off inverted). Say so in the candidate itself; do not quietly re-file it.
- **Read the in-flight changes** from `speclink list --json` and their proposals. A candidate that overlaps the area an in-flight change is already rewriting SHALL NOT be proposed — the work is happening, and a discussion about it now collides with a change mid-flight.

### Step 2: Converge the scope (scope before you scan)

Never blind-scan the whole repository. Candidates from unrelated corners are not comparable, and the discussion loses its focus.

- **When the user named a direction** (a module, subsystem, or pain point), that IS the scope. Use it directly and skip the inference below entirely.
- **Otherwise, infer from git history.** Look for hotspots — the areas that change most often:

  ```bash
  git log --since="3 months ago" --name-only --pretty=format: | sort | uniq -c | sort -rn | head -40
  ```

  **Weight recent churn more heavily.** The payoff of deepening is that future changes get easier, so the code that keeps changing is where that payoff lands.

- **Local supplement**: cross-check the archived changes' touched records (`.evidence.json` under `openspec/changes/archive/<dated-name>/`, the same records `/speclink-commit` uses). Where a bare `git log` says "these files changed together", the touched records say *which intent* moved *which files* — a stronger signal for what belongs to one seam.
- **When the hotspots are diffuse** — no clear focus, churn spread evenly — **widen the net** instead of forcing one. Take a larger area (a whole crate, a whole layer) and scan it as one scope rather than picking an arbitrary hot file.

Announce the scope you settled on and why, in one or two sentences, before scanning.

### Step 3: Scan

Explore the scope organically — read the code, follow what looks strange, chase the thing that surprises you. **This is not a checklist to tick off.** The five friction signals below are what to keep an eye out for while reading, not a form to fill in.

**The five friction signals:**

1. **Understanding one concept requires jumping between several small modules.** The concept is real but its implementation is scattered; the reader reassembles it every time.
2. **A shallow module: the interface is nearly as complex as the implementation behind it.** The abstraction charges as much to learn as it saves.
3. **A pure function was extracted to make testing easy, but the bugs live at the call site.** The tests pass over the extracted piece while the actual behaviour — the wiring around it — has no locality and no coverage.
4. **Tight coupling leaks across a seam.** Two sides that are supposed to be separable know each other's internals; changing one forces changing the other.
5. **An area that is hard to test through its current interface.** The interface makes the natural test awkward — which usually means the interface is on the wrong boundary.

**The deletion test is the admission criterion for a candidate.** For anything you are tempted to propose, ask: if this module/abstraction were deleted and its callers absorbed the work, what would happen? A candidate only qualifies when deleting it **concentrates** complexity — the same behaviour ends up in one place that can be understood as a whole. If deleting it merely **moves** complexity somewhere else, that is not a signal; drop it and keep reading.

**Scanning mechanism**: **inline is the default** — read and search the scope yourself. Dispatch an `Explore` subagent only when the user named no direction, or when the scope genuinely spans several crates. **The hard limit is 2 subagents.** Never spawn a third; if two are not enough, the scope was too wide — go back to Step 2 and narrow it.

Aim for 3-6 candidates. Fewer is fine when the scope is clean; a list of twelve is a sign the deletion test was not applied.

### Step 4: Record the candidates

Create the discussion record and write the scan into it as Round 1:

```bash
speclink discuss new "<topic>" --slug improve-<scope> --kind improve
```

`--kind improve` marks the record as an improvement discussion (it is what the board and the discussion drawer show a badge for). The slug convention is `improve-<scope>` — `improve-store-layer`, `improve-cli-commands`. Write the Context once (what prompted the scan, the scope you settled on in Step 2 and why, what Step 1 excluded), then record the candidates as Round 1 with the mode label `scan`:

```bash
speclink discuss context improve-<scope> --stdin <<'CTX_EOF'
...
CTX_EOF

speclink discuss add-round improve-<scope> --mode scan --stdin <<'ROUND_EOF'
**Focus**: which structural improvements this scope offers
**Position**: <one-sentence read of the scope>, with these candidates:
...
ROUND_EOF
```

**Every candidate carries exactly five fields:**

| Field          | What goes in it                                                                       |
| -------------- | ------------------------------------------------------------------------------------- |
| Files          | The concrete paths involved — the reader must be able to open them                    |
| Problem        | The friction, named and evidenced. Which signal, seen where                            |
| Solution       | The structural change proposed — where the seam moves, what absorbs what               |
| Wins           | What gets easier afterwards, concretely. "Cleaner" is not a win                        |
| Recommendation | One of three strengths: **strongly recommended** / **worth exploring** / **speculative** |

The three recommendation strengths are not decoration — they tell the user where to spend their attention. `strongly recommended`: the friction is evidenced and the deletion test is clearly passed. `worth exploring`: the friction is real but the right shape of the fix is not obvious. `speculative`: you suspect something is off but the evidence is thin.

**End the round with your own pick.** Say which candidate you would take first and why, then ask the user which one to dig into. Do not grill anything until they answer — the pick is theirs.

### Step 5: Grill the chosen candidate to a conclusion

Once the user picks, this becomes an ordinary speclink discussion, run with the discuss skill's question discipline:

- **One question per exchange.** Never a barrage. Resolve upstream questions first — the shape of the downstream ones depends on the answers.
- **Every question carries your proposed answer, and the proposal cites evidence** — file paths, symbols, probe results. The user agrees or corrects; they never get a bare open question that a Grep could have grounded.
- **Facts are yours to verify, decisions are theirs to make.** Anything the code can answer, answer yourself. Only genuine judgment calls go to the user.
- **The interface depth check runs unconditionally on every chosen candidate** — no exemptions, this skill is about seams by definition. Work through all four: (1) **Seam location** — where does the boundary belong? (2) **Adapter count** — one adapter on this path, or several thin wrappers stacked? (3) **Depth** — what behaviour hides behind the interface? "It just forwards calls" means too shallow. (4) **Deletion test** — delete it today: what actually breaks? Surface the four answers in the round or the conclusion, not as private notes.
- **Append a round per exchange** with `speclink discuss add-round <slug> --mode interview --stdin` — `**Focus**` / `**Position**` / `**Ruled out**` / `**Open**`. Rejected candidates go under `**Ruled out**` with the reason they lost; that is what Step 1 of the next scan reads.

**Converging:**

```bash
speclink discuss conclude improve-<scope> --stdin <<'CONCLUSION_EOF'
**Decision**: ...
**Rationale**: ... (the trade-off that drove it)
**Rejected alternatives**: ... (each candidate that lost, with why)
**Deferred**: ... — or "none"
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-<scope>
CONCLUSION_EOF
```

Then fan out: `speclink discuss promote <slug>` (or `/speclink-propose --from-discussion <slug>`) for a new change, or `speclink discuss link <slug> <existing-change>` when the improvement belongs to a change already in flight. One scan can fan out into several changes — the record accumulates each name and is archived automatically when the last of them is archived.

**When the user rejects every candidate, the scan still concluded something.** Write the conclusion — that nothing here is worth doing, and why each candidate lost — and archive the record:

```bash
speclink discuss conclude improve-<scope> --stdin <<'CONCLUSION_EOF'
**Decision**: no change — none of this round's candidates are worth doing
**Rationale**: ...
**Rejected alternatives**: ... (every candidate, with the reason it lost)
**Deferred**: ...
CONCLUSION_EOF

speclink discuss archive improve-<scope>
```

**Never `discard` an improvement discussion.** The rejections ARE the value: they are what stops the next scan from proposing the same thing again. A discarded record takes that memory with it.

---

## Guardrails

- **User-initiated only** — never start a scan on your own initiative
- **Never implement** — the output is a discussion record, not a diff
- **Scope before you scan** — no blind whole-repo sweeps
- **Deletion test gates every candidate** — concentrating complexity counts, moving it does not
- **At most 2 Explore subagents** — inline is the default
- **Check the archive first** — a settled rejection is not a candidate
- **Conclude and archive, never discard** — even when the answer is "do nothing"
