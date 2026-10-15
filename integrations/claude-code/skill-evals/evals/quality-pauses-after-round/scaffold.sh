#!/usr/bin/env bash
# add-greeting 的程式任務已勾、src/greet.js 已 commit，但有一個真的錯誤（見 fixtures.sh）。
source "$(dirname "$0")/../fixtures.sh"
eval_greeting_repo
eval_buggy_greet_committed
