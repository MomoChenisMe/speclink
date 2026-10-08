#!/usr/bin/env bash
# change add-greeting 進行中，任務都還沒做。
source "$(dirname "$0")/../fixtures.sh"
eval_init
eval_greeting_change
speclink in-progress add add-greeting >/dev/null
