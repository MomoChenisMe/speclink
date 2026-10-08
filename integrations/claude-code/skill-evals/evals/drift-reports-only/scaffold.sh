#!/usr/bin/env bash
# change 的 design 提到 src/old.js；之後 git mv 成 src/new.js 並 commit。
source "$(dirname "$0")/../fixtures.sh"
eval_git_init
eval_init
mkdir -p src
echo 'module.exports = () => "hello";' >src/old.js
eval_greeting_change
printf '\n### Helper\n\nThe helper in `src/old.js` returns the greeting text.\n' >>openspec/changes/add-greeting/design.md
eval_commit 'add helper and change'
git mv src/old.js src/new.js
eval_commit 'rename helper'
