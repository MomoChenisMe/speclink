#!/usr/bin/env bash
# change add-greeting 以 task done 記錄 greeting.txt；另有無關的 notes.txt 改動。
source "$(dirname "$0")/../fixtures.sh"
eval_git_init
echo 'notes' >notes.txt
eval_commit 'initial commit'
eval_init
eval_greeting_change
eval_greeting_artifacts_with_baseline
eval_greeting_done
echo 'unrelated edit' >>notes.txt
