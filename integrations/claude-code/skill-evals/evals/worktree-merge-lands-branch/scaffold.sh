#!/usr/bin/env bash
# 依 apply-with-worktree 的慣例建立 worktree（分支 speclink/add-greeting、
# 路徑 <repo-parent>/<repo-folder>.worktrees/add-greeting），在其中完成變更並 commit。
source "$(dirname "$0")/../fixtures.sh"
eval_greeting_repo
speclink workflow-config set worktree true >/dev/null
eval_greeting_change
eval_commit 'add-greeting: artifacts'
worktree="$(dirname "$PWD")/$(basename "$PWD").worktrees/add-greeting"
git worktree add -q -b speclink/add-greeting "$worktree"
(
  cd "$worktree"
  speclink review prepare add-greeting >/dev/null 2>&1
  eval_greeting_done
  eval_commit 'add-greeting: greeting.txt'
)
