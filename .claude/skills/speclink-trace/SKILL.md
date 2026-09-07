---
name: speclink-trace
description: "Use when someone asks how a capability came to be or why it works this way — walks its provenance chain across archived changes, source discussions, evidence and live code."
context: fork
agent: Explore
disallowedTools: [Edit, Write]
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.30.0"
  generatedBy: "Speclink"
---

Answer "how did this come to be / why is it designed this way" for a feature: map the question to a capability, walk its provenance chain (archived changes → source discussions → evidence → live code), and reply with one sourced narrative.

**Input**: A natural-language question after `/speclink-trace` (e.g., `/speclink-trace why does archiving stamp @trace blocks?`). The question names or implies one feature or behavior.

**Prerequisites**: This skill requires the `speclink` CLI and `git`. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Canon pass — map the question to a capability**

   Run `speclink list --specs --json` and match the question against the canonical capability names; when a name alone is not conclusive, check its scope with `speclink show <name> --item-type spec`.

   - One capability clearly matches → continue with step 2.
   - No capability plausibly covers the question → skip to step 6 and answer from the codebase. Do NOT reply "no spec found" and stop.

2. **Fetch the provenance chain**

   Run `speclink trace <capability> --json`. The payload carries:

   - `changes`: the archived changes that touched the capability, oldest first — each with `archivedDir`, `fromDiscussion` (slug or null) and `evidence` (per-task file lists, or null)
   - `discussions`: each source discussion with its full `promotedTo` fan-out — the sibling changes it spawned and the capabilities they touched
   - `requirements`: which change each canonical requirement currently comes from

   If the command fails with name suggestions, redo step 1 against those exact names.

3. **Read the story behind each link**

   - For every discussion slug: `speclink discuss show <slug>` — pull the conclusion (the decision and its reasons) and the rounds where alternatives were weighed and rejected.
   - For every change in the chain: its `proposal.md` sits inside the archived change directory named by `archivedDir` — pull the motivation ("Why") section.
   - Note the sibling changes from `promotedTo` and the capabilities that evolved alongside.

4. **Locate the touched code**

   - `evidence` non-null → the per-task `files` lists name what the change touched; use them as the file set.
   - `evidence` null → look the change up in version-control history instead: `git log --oneline --name-only` and match commit subjects carrying the change name (the commit-scope convention). Treat the hits as best-effort leads, not guarantees. Nothing found → answer from the discussion and proposal content; never stop, and never ask the user to supply a missing record.

5. **Let live code have the last word**

   Any statement about CURRENT behavior must come from the current code, freshly consulted in this session. Evidence lists and commits are historical snapshots — cite them for "how it got here", never for "how it is".

6. **No spec matched — answer from the codebase**

   Search the code for the behavior by keyword; use `git log -S` and `git blame` to find the commits that introduced it; read the commit messages as decision leads. Then answer in exactly the format of step 7.

7. **Compose the answer**

   One narrative, containing:

   - the decision and its reasons
   - alternatives that were considered and rejected (when the records show them)
   - related capabilities that evolved together
   - a source path for every claim — spec path, discussion slug, archived change directory, commit hash, or code file

**Output**

```
## <capability or topic>: how it came to be

<narrative: decision → reasons → rejected alternatives → current state>

**Sources**
- <claim> — <path / slug / commit>
```

**Guardrails**

- Every claim carries a source path; a claim you cannot source gets qualified or dropped.
- The answer NEVER contains internal pipeline words — "degraded", "fallback", "old era", "incomplete data", "no evidence recorded" and the like. All sources (discussions, proposals, commits, code) are woven in as natural citations of one story.
- Missing records change where you look, never what the reader sees: the answer format is identical whether it was assembled from the chain, from git history, or from code archaeology.
- git commit-message conventions are best-effort leads; when they yield nothing, discussions and proposals carry the answer.
- Do not edit any file — this skill only reads and answers.
