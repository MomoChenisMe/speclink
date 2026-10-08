#!/usr/bin/env bash
# change add-greeting 兩個任務：1.1 建立 greeting.txt、[M] 1.2 打開檔案確認內容。
source "$(dirname "$0")/../fixtures.sh"
eval_git_init
eval_init
eval_greeting_change manual
eval_commit 'add-greeting: artifacts'
