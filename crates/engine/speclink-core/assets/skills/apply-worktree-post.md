
---

## Worktree wrap-up

Once the apply flow above has finished (all tasks complete, or the user stopped you at a good point), do these — still **inside the worktree**.

The apply body's own **Next steps** section does not apply here — this worktree flow replaces it with the edges in **Next steps** at the end of this document.

### W1. Commit the change in the worktree

Follow the `/speclink:commit` skill's attribution convention: stage only the files belonging to this change (its artifacts under `{{SPEC_DIR}}changes/<change-name>/` plus the source files recorded in the change's evidence record), leave unrelated dirty files alone, and write the commit message in the project's language.

The commit lands on branch `speclink/<change-name>` inside the worktree. Nothing reaches the main branch yet.

### W2. Catch up with the main branch, then reset the quality starting point

Rebasing now means a stamp landed in this worktree covers the content as it will be after the merge, and the later merge is a fast-forward that keeps the stamp fresh. Do it only while neither quality station has a ticket — a ticket means the stations are already running, and a rebase in the middle of their loop would move the ground under them:

```bash
speclink review show "<change-name>" --json
speclink verify show "<change-name>" --json
```

Both report that no ticket exists → continue. Either one shows a ticket → skip this whole step and say so.

1. Find the main checkout's path in `git worktree list --porcelain` (its first entry), then the branch it is on:

   ```bash
   git -C <main-checkout> branch --show-current
   ```

   A `speclink/*` branch or empty output (detached HEAD) → skip the rest of this step and tell the user why.

2. Record that branch's current commit, then replay this worktree's branch onto it:

   ```bash
   git -C <main-checkout> rev-parse HEAD
   git rebase "<recorded-commit>"
   ```

   - **Conflicts** → do not resolve them. Run `git rebase --abort` (the branch returns exactly as it was), report the conflicting files, and skip the reset below: the branch did not change, so its old starting point still holds.
   - **Git refuses for another reason** (for example, uncommitted files) → report git's message and skip the reset below.

3. The rebase succeeded → move the quality stations' starting point to the recorded commit:

   ```bash
   speclink review prepare "<change-name>" --base "<recorded-commit>"
   ```

A stamp can still go stale if the main branch changes the same files again while the stations run.

### W3. Stop here — do not merge, do not remove the worktree

This skill's job ends at the commit. Explicitly:

- **Do NOT** merge `speclink/<change-name>` back into the main branch.
- **Do NOT** run `git worktree remove`, and do NOT delete the branch.
- **Do NOT** switch the main checkout to another branch or touch it in any way.

Merging is a separate, human-triggered step: it needs a clean main tree, and a conflict there is the user's call, not yours. Leaving the worktree in place also means the main checkout's `speclink list` keeps showing this change with its `[worktree]` marker until the merge happens.

### W4. Hand off

Pick the path by the change's state: unchecked `[M]` manual tasks show in `speclink instructions apply --change "<change-name>" --json` as `remaining` above `codeRemaining`.

- **No unchecked `[M]` task** — tell the user, plainly:

  > 這個 change 已在 worktree 內完成並提交，尚未合併回主分支。
  >
  > 建議在這個 worktree 內跑品質關卡——`/speclink:review`（工藝品質）∥ `/speclink:verify`（規格符合度），或 `/speclink:quality`（兩站合跑），是否要跑由你決定。蓋章會寫進 change 的 meta，記得補一次提交，再執行 `/speclink:worktree-merge <change-name>` 收尾。
  >
  > 想在蓋章前先看到 CI 結果的話，改走另一條路：先執行 `/speclink:worktree-merge <change-name>` 合併，再到主 checkout 跑品質關卡——worktree 分支只在本機、從不 push，CI 看不到它；合併時品質關卡的起點會一起承接到主 checkout。

- **Unchecked `[M]` tasks remain** — tell the user, plainly:

  > 這個 change 已在 worktree 內完成並提交，尚未合併回主分支，還有手動任務沒做。
  >
  > 建議先執行 `/speclink:worktree-merge <change-name>`——它會檢查主樹是否乾淨、把分支合併回去、把品質關卡的起點承接到主 checkout，成功後移除 worktree 並刪掉分支。之後在主 checkout 跑品質關卡——`/speclink:review`（工藝品質）∥ `/speclink:verify`（規格符合度），或 `/speclink:quality`（兩站合跑），是否要跑由你決定——做完手動任務再封存。

Report alongside it: the worktree path, the branch name, the tasks completed this session, and whether W2 rebased or why it skipped.

### Wrap-up check

Check these before you hand off:

- [ ] The commit in the worktree holds only this change's artifacts and evidence-recorded source files (W1).
- [ ] W2 rebased only while neither station had a ticket; a conflict was aborted, never resolved; a successful rebase was followed by `speclink review prepare --base` (W2).
- [ ] Nothing was merged, no worktree or branch was removed, and the main checkout was not touched (W3).
- [ ] The hand-off names the worktree path, the branch and the tasks completed, takes the path the manual tasks call for, and lists review, verify and quality as the entries next to `/speclink:worktree-merge` (W4).

## Next steps

{{NEXT_STEPS_LEAD}}

- No unchecked `[M]` task → the quality stations here in the worktree: `/speclink:review` ∥ `/speclink:verify`, or `/speclink:quality` for both; commit the stamps, then `/speclink:worktree-merge <change-name>`
- Unchecked `[M]` tasks remain, or the user wants CI results before stamping (the worktree branch is local only, so CI never sees it) → `/speclink:worktree-merge <change-name>` first, then the quality stations and the manual tasks in the main checkout, then archive
- Requirements changed mid-work → `/speclink:ingest <change-name>` inside this worktree, then resume apply here
