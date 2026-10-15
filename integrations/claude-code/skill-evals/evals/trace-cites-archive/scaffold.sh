#!/usr/bin/env bash
# greeting 正式規格來自已封存的 change add-greeting。
source "$(dirname "$0")/../fixtures.sh"
eval_init
eval_greeting_archived
