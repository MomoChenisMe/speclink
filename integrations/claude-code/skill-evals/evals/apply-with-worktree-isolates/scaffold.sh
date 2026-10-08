#!/usr/bin/env bash
# worktree 政策開啟；change add-greeting 已 commit。
source "$(dirname "$0")/../fixtures.sh"
eval_greeting_repo
speclink workflow-config set worktree true >/dev/null
eval_greeting_change
eval_commit 'add-greeting: artifacts'
