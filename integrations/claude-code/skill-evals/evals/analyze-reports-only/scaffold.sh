#!/usr/bin/env bash
# demo 建立的 change 名稱是隨機的，改成固定的 demo-change 讓提示可以指名。
source "$(dirname "$0")/../fixtures.sh"
eval_init
speclink demo >/dev/null
demo=$(ls openspec/changes | grep -v '^archive$')
mv "openspec/changes/$demo" openspec/changes/demo-change
