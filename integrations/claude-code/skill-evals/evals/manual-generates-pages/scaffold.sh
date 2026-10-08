#!/usr/bin/env bash
# 封存過一個 change，openspec/specs/greeting/spec.md 是正式規格。
source "$(dirname "$0")/../fixtures.sh"
eval_init
eval_greeting_archived
