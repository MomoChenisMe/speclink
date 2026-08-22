---
name: speclink-worktree-merge
description: "Merge a finished Speclink worktree branch back into the main branch, then clean up"
disallowedTools: [Edit, Write]
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.19.19"
  generatedBy: "Speclink"
---

Merge a finished Speclink worktree branch back into the main branch, then clean up.

This is the wrap-up half of `/speclink-apply-with-worktree`. That skill stops right after committing inside the worktree; this one takes it from there. It is **human-triggered**: merging is a decision, and a conflict is the user's call.

**Input**: Optionally specify a change name (e.g., `/speclink-worktree-merge add-auth`). If omitted, run `git worktree list --porcelain` and offer the `speclink/*` branches found. If more than one is a candidate you MUST ask which one — never guess.

**Prerequisites**: This skill requires `git`. Run `git --version`. If git is not available, report it and STOP. Every step below is driven from the **main checkout**; the steps that act on the worktree — its status check and the rebase — reach it with `git -C <worktree-path>` rather than moving you there.

**Steps**

1. **Identify the worktree**

   ```bash
   git worktree list --porcelain
   ```

   Find the entry whose branch is `speclink/<change-name>`. If there is none, STOP and tell the user no worktree exists for that change — nothing to merge.

   Announce: "Merging worktree for change: <name>" plus the worktree path and branch.

2. **Preflight — all three conditions must hold**

   Check which branch the main checkout is on:

   ```bash
   git -C <main-checkout> branch --show-current
   ```

   - **A `speclink/*` branch, or empty output (detached HEAD)** — STOP. The main checkout is parked somewhere a merge must never land; tell the user to switch it back to the main branch first. Do **NOT** switch branches on their behalf.
   - **Anything else** — that branch is the merge target. Announce it (it appears again in step 3 and in the success output), so a wrong destination is visible BEFORE the merge, not after.

   Check the main checkout's working tree:

   ```bash
   git -C <main-checkout> status --porcelain
   ```

   Check the worktree's:

   ```bash
   git -C <worktree-path> status --porcelain
   ```

   - **Main tree not clean** (any uncommitted change) — STOP. List the dirty files and tell the user to commit or stash them first. Do **NOT** stash on their behalf. Do **NOT** commit their unrelated work for them.
   - **Worktree not fully committed** (any uncommitted change) — STOP. List the dirty files and tell the user the change's work must be committed inside the worktree first (`/speclink-apply-with-worktree` does this at its wrap-up). Do **NOT** commit on their behalf.

   Only when all three hold, continue.

3. **Merge into the main branch — rebase first, fast-forward second**

   `speclink/*` branches are local and never pushed, so rewriting their history costs nothing. Replaying the branch onto the target first lets the merge be a fast-forward, which keeps the main branch a straight line instead of collecting one merge node per parallel change.

   In the worktree, replay the branch onto the target branch verified in step 2:

   ```bash
   git -C <worktree-path> rebase "<target-branch>"
   ```

   - **Rebase succeeds** — in the main checkout, land it without a merge node:

     ```bash
     git -C <main-checkout> merge --ff-only "speclink/<change-name>"
     ```

     If the fast-forward is refused, the target branch moved on between the rebase and the merge — another worktree landed first. Take the same exit as a rebase conflict below: fall back to a plain merge, and tell the user this one lands as a merge node.

   - **Rebase reports conflicts** — do **NOT** resolve them. Abort, which restores the branch exactly as it was:

     ```bash
     git -C <worktree-path> rebase --abort
     ```

     Then fall back to a plain merge in the main checkout, and tell the user this one lands as a merge node:

     ```bash
     git -C <main-checkout> merge "speclink/<change-name>"
     ```

4. **Conflict — stop immediately**

   If the fallback merge reports conflicts:

   - Abort so nothing half-merged is left behind:

     ```bash
     git -C <main-checkout> merge --abort
     ```

   - Report the conflicting file list to the user, verbatim.
   - Do **NOT** edit conflict markers, do **NOT** pick a side, do **NOT** commit a partial merge.
   - STOP and wait for the user to decide how to resolve it.

   The main checkout must end up exactly as it was before step 3.

5. **Clean up after a successful merge**

   ```bash
   git -C <main-checkout> worktree remove "<worktree-path>"
   git -C <main-checkout> branch -d "speclink/<change-name>"
   ```

   If `worktree remove` refuses because the worktree is dirty, do NOT force it — report what is uncommitted there and STOP. If `branch -d` refuses, report it and STOP: an unmerged commit means step 3 did not carry everything.

   After removal the main checkout's `speclink list` no longer shows the `[worktree]` marker for this change, and its task counts read from the main copy again.

6. **Confirm and hand off**

   Tell the user the wrap-up is done: the branch is merged, the worktree is removed, and the branch is deleted. Then point at what comes next in canonical order — the quality stations belong inside the worktree, so by this point they are either done or deliberately skipped, and the next step is archiving from the main checkout:

   > 這個 change 已合併回主分支。品質關卡建議在 worktree 內就跑完；跑過或使用者決定略過的話，下一步是在主 checkout `/speclink-archive` 封存。
   >
   > 還沒跑品質關卡的話，仍可在主 checkout 補跑 `/speclink-review`、`/speclink-verify` —— 但那是降級路徑：主 checkout 沒有 Apply baseline，審查凍結面會退回較粗的判定。

**Output On Success**

```
## Worktree merged

**Change:** <change-name>
**Branch:** speclink/<change-name> → <main-branch> ✓
**Landed as:** fast-forward (straight line) — or "merge node (rebase fell back)" when step 3 took the fallback
**Worktree:** <path> (removed)
**Branch deleted:** ✓

接下來：主 checkout 封存 /speclink-archive（品質關卡建議已在 worktree 內完成；未跑則主 checkout 補跑屬降級路徑）
```

**Output On Stop**

```
## Merge stopped

**Change:** <change-name>
**Reason:** <preflight failure or conflict>

<dirty file list, or conflicting file list>

**What you need to do:**
<the specific action — commit/stash in the main tree, commit in the worktree, or resolve the conflict>
```

**Guardrails**

- Never stash or commit on the user's behalf — in the main checkout or in the worktree
- Never resolve merge conflicts yourself; abort the merge and report
- Never resolve rebase conflicts yourself; `rebase --abort` and fall back to the plain merge
- Never leave a half-finished merge state behind
- Never force-remove a worktree that still has uncommitted work
- Never merge a change whose worktree you did not verify is fully committed
