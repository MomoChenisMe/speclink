
---

## Worktree wrap-up

Once the apply flow above has finished (all tasks complete, or the user stopped you at a good point), do these — still **inside the worktree**.

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

> 這個 change 已在 worktree 內完成並提交，尚未合併回主分支。要收尾請執行 `/speclink:worktree-merge <change-name>`——它會檢查主樹是否乾淨、把分支合併回去，成功後移除 worktree 並刪掉分支。

Report alongside it: the worktree path, the branch name, and the tasks completed this session.
