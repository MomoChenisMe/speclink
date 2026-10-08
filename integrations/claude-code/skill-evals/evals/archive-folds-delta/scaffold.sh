#!/usr/bin/env bash
# change add-greeting 任務全勾，含 capability greeting 的 ADDED delta。
source "$(dirname "$0")/../fixtures.sh"
eval_greeting_repo
eval_greeting_committed
