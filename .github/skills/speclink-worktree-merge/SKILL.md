---
name: speclink-worktree-merge
description: "Use when a worktree change is committed and ready to land — merges the branch back into the main branch, then cleans the worktree up."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.50.5"
  generatedBy: "Speclink"
---

**Conversation language**: write to the user in the language they use with you — progress updates, questions, and summaries. This skill is written in English, and its example messages show structure, not language: translate their prose, and keep commands, file paths, code, and fixed tokens (severity labels, status words) as they are. When a step sets the language of its own output — a document, a report, or the way it presents results and asks about them (for example through `locale`) — that step decides for that output. When you edit a document that already exists and no step sets its language, keep the language it is written in. When you run in a forked context that cannot see the conversation, use the language of the request you were given.

Merge a finished Speclink worktree branch back into the main branch, then clean up.

This is the wrap-up half of `/speclink-apply-with-worktree`. That skill stops right after committing inside the worktree; this one takes it from there. It is **human-triggered**: merging is a decision, and a conflict is the user's call.

**Input**: Optionally specify a change name (e.g., `/speclink-worktree-merge add-auth`). If omitted, run `git worktree list --porcelain` and offer the `speclink/*` branches found. If more than one is a candidate you MUST ask which one and wait for the answer — never guess.

**Prerequisites**: This skill requires `git` and the `speclink` CLI. Run `git --version`. If git is not available, report it and STOP. Every step below is driven from the **main checkout**; the steps that act on the worktree — its status check and the rebase — reach it with `git -C <worktree-path>` rather than moving you there.

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

   Before anything moves, record the main checkout's HEAD — the **pre-merge HEAD**. Step 5 gives it to the quality stations as their starting point in the main checkout:

   ```bash
   git -C <main-checkout> rev-parse HEAD
   ```

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

5. **Carry the quality baseline over, then clean up**

   After a successful merge — fast-forward or merge node alike — and before the worktree is removed, run this from the main checkout:

   ```bash
   speclink review prepare "<change-name>" --base "<pre-merge-HEAD>" --from "<worktree-path>"
   ```

   It writes the quality stations' Apply baseline in the main checkout and carries over both stations' snapshots from the worktree, so `/speclink-review`, `/speclink-verify` and `/speclink-quality` work in the main checkout with no `--base` from the user. From the pre-merge HEAD to the merged HEAD is exactly what this change brought in, conflict resolutions included. If the command fails, report its error and stop here: do not remove the worktree — the snapshots still live there. The merge has already landed, so the way back is not this skill: once the cause is cleared (an open ticket stamped or discarded, a failed copy retried), run that same `speclink review prepare` line again with the pre-merge HEAD recorded in step 3, then run the cleanup below. A second run of this skill would record the merged HEAD as the pre-merge HEAD, and the baseline would then hide every change this merge brought in.

   Then remove the worktree and delete the branch:

   ```bash
   git -C <main-checkout> worktree remove "<worktree-path>"
   git -C <main-checkout> branch -d "speclink/<change-name>"
   ```

   If `worktree remove` refuses because the worktree is dirty, do NOT force it — report what is uncommitted there and STOP. If `branch -d` refuses, report it and STOP: an unmerged commit means step 3 did not carry everything.

   After removal the main checkout's `speclink list` no longer shows the `[worktree]` marker for this change, and its task counts read from the main copy again.

6. **Confirm and hand off**

   Tell the user the wrap-up is done: the branch is merged, the quality baseline is carried over, the worktree is removed, and the branch is deleted. Then hand off by the change's state. Read the manual tasks from `speclink instructions apply --change "<change-name>" --json` (`remaining` above `codeRemaining` means unchecked `[M]` tasks); ask the user when you cannot tell whether the quality stations already ran.

   - **The quality stations already ran in the worktree** (their stamps came along with the merge):

     > 這個 change 已合併回主分支，品質關卡也已在 worktree 內跑完。下一步是在主 checkout `/speclink-archive` 封存。

   - **The quality stations have not run, and no `[M]` task is open**:

     > 這個 change 已合併回主分支，品質關卡的起點已承接到主 checkout。要跑的話直接在這裡執行 `/speclink-review`（工藝品質）∥ `/speclink-verify`（規格符合度），或 `/speclink-quality`（兩站合跑），是否要跑由你決定；之後 `/speclink-archive` 封存。

   - **Unchecked `[M]` manual tasks remain**:

     > 這個 change 已合併回主分支，品質關卡的起點已承接到主 checkout。在這裡執行 `/speclink-review` ∥ `/speclink-verify`，或 `/speclink-quality`，再把手動任務做完並勾選，之後 `/speclink-archive` 封存。

**Output On Success**

```
## Worktree merged

**Change:** <change-name>
**Branch:** speclink/<change-name> → <main-branch> ✓
**Landed as:** fast-forward (straight line) — or "merge node (rebase fell back)" when step 3 took the fallback
**Quality baseline:** carried over to the main checkout ✓
**Worktree:** <path> (removed)
**Branch deleted:** ✓

接下來：<step 6 的交棒——品質關卡已跑完→主 checkout /speclink-archive；未跑→直接在主 checkout 跑品質關卡再封存；有未勾手動任務→主 checkout 跑品質關卡、完成手動任務後封存>
```

**Output On Stop**

```
## Merge stopped

**Change:** <change-name>
**Reason:** <preflight failure, conflict, or baseline carry failed after the merge landed>

<dirty file list, conflicting file list, or the prepare error verbatim>

**What you need to do:**
<the specific action — commit/stash in the main tree, commit in the worktree, resolve the conflict, or clear the prepare error and rerun that one prepare line with the recorded pre-merge HEAD <sha>, then the cleanup>
```

**Guardrails**

Check these before you report:

- [ ] Never stash or commit on the user's behalf — in the main checkout or in the worktree (step 2)
- [ ] Never resolve merge conflicts yourself; abort the merge and report (step 4)
- [ ] Never resolve rebase conflicts yourself; `rebase --abort` and fall back to the plain merge (step 3)
- [ ] Never leave a half-finished merge state behind (step 4)
- [ ] Never force-remove a worktree that still has uncommitted work (step 5)
- [ ] The pre-merge HEAD was recorded before the rebase, and the worktree was removed only after `speclink review prepare --base --from` succeeded (steps 3 and 5)
- [ ] Never merge a change whose worktree you did not verify is fully committed (step 2)

## Next steps

Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.

- The quality stations already ran in the worktree → `/speclink-archive <change-name>` **from the main checkout** — archiving inside a linked worktree is refused by the engine
- The quality stations have not run → run them in the main checkout, where step 5 carried their baseline over: `/speclink-review` ∥ `/speclink-verify`, or `/speclink-quality` for both; then `/speclink-archive <change-name>`
- Unchecked `[M]` manual tasks remain → the quality stations in the main checkout, then the manual tasks, then `/speclink-archive <change-name>`
