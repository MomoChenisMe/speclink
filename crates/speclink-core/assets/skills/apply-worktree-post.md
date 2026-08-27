
---

## Worktree wrap-up

Once the apply flow above has finished (all tasks complete, or the user stopped you at a good point), do these — still **inside the worktree**.

The apply body's own **Next steps** section does not apply here — this worktree flow replaces it with the edges in **Next steps** at the end of this document.

### W1. Commit the change in the worktree

Follow the `/speclink:commit` skill's attribution convention: stage only the files belonging to this change (its artifacts under `{{SPEC_DIR}}changes/<change-name>/` plus the source files recorded in the change's evidence record), leave unrelated dirty files alone, and write the commit message in the project's language.

The commit lands on branch `speclink/<change-name>` inside the worktree. Nothing reaches the main branch yet.

### W2. Stop here — do not merge, do not remove the worktree

This skill's job ends at the commit. Explicitly:

- **Do NOT** merge `speclink/<change-name>` back into the main branch.
- **Do NOT** run `git worktree remove`, and do NOT delete the branch.
- **Do NOT** switch the main checkout to another branch or touch it in any way.

Merging is a separate, human-triggered step: it needs a clean main tree, and a conflict there is the user's call, not yours. Leaving the worktree in place also means the main checkout's `speclink list` keeps showing this change with its `[worktree]` marker until the merge happens.

### W3. Hand off

Tell the user, plainly:

> 這個 change 已在 worktree 內完成並提交，尚未合併回主分支。
>
> 建議先在這個 worktree 內跑品質關卡——`/speclink:review`（工藝品質）∥ `/speclink:verify`（規格符合度），或 `/speclink:quality`（兩站合跑），是否要跑由你決定。品質關卡的 Apply baseline 就在這個 worktree 裡，離開就沒有了。蓋章會寫進 change 的 meta，記得補一次提交。
>
> 品質關卡跑完（或決定略過）後執行 `/speclink:worktree-merge <change-name>` 收尾——它會檢查主樹是否乾淨、把分支合併回去，成功後移除 worktree 並刪掉分支。

Report alongside it: the worktree path, the branch name, and the tasks completed this session.

## Next steps

{{NEXT_STEPS_LEAD}}

- The change is committed inside the worktree → the quality stations belong **here**, while the Apply baseline still exists: `/speclink:review` ∥ `/speclink:verify`, or `/speclink:quality` for both
- The stations are done or the user chose to skip them → `/speclink:worktree-merge <change-name>`
- Requirements changed mid-work → `/speclink:ingest <change-name>` inside this worktree, then resume apply here
