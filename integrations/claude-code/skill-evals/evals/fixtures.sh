# 評測題共用的 scaffold 片段：每題的 scaffold.sh 以 source 載入，在評測工作區（目前目錄）執行。
# 只依賴 PATH 上的 speclink 與 git。
#
# EVAL_TOOLS 預設 claude：技能由評測 plugin 的 skills/ 提供，所以刪掉 init 寫出的專案技能。
# 手動抽查 Codex／Copilot 時設成 codex 或 copilot，技能留在工作區給該工具讀：
#   mkdir ws && cd ws && EVAL_TOOLS=codex bash <題目>/scaffold.sh
set -euo pipefail

eval_init() {
  local tools="${EVAL_TOOLS:-claude}"
  speclink init . --tools "$tools" >/dev/null
  if [ "$tools" = claude ]; then rm -rf .claude/skills; fi
}

# 評測的 HOME 是臨時目錄，沒有全域 git 身分，所以設在工作區內。
eval_git_init() {
  git init -q -b main .
  git config user.name "Eval User"
  git config user.email "eval@example.com"
}

eval_commit() {
  git add -A
  git commit -qm "$1"
}

# 第一個 commit 只有 README.md 的 git repo，之後才 init（init 寫出的檔案留在工作目錄、未 commit）。
eval_greeting_repo() {
  eval_git_init
  echo '# greeting' >README.md
  eval_commit 'initial commit'
  eval_init
}

# commit add-greeting 的 artifacts，並記錄 apply 基準（review／verify 的凍結範圍以它為起點）。
# 基準要在別處記錄的題目（由受測的技能記錄，或由 scaffold 在 worktree 內另行記錄）只用 eval_commit。
eval_greeting_artifacts_with_baseline() {
  eval_commit 'add-greeting: artifacts'
  speclink review prepare add-greeting >/dev/null 2>&1
}

# change add-greeting：在 repo 根目錄建立內容為 hello 的 greeting.txt。
# 參數 manual：多一個 [M] 任務（打開檔案確認內容）。
eval_greeting_change() {
  local dir=openspec/changes/add-greeting
  speclink new change add-greeting >/dev/null
  mkdir -p "$dir/specs/greeting"
  cat >"$dir/proposal.md" <<'EOF'
## Why

New contributors should see a friendly greeting when they open the repository.

## What Changes

- Add `greeting.txt` at the repository root with the content `hello`.

## Capabilities

### New Capabilities

- `greeting`: A greeting file at the repository root.

### Modified Capabilities

(None)

## Impact

- New file: `greeting.txt`
EOF
  cat >"$dir/design.md" <<'EOF'
## Context

The repository has no greeting yet.

## Decisions

### Plain text file

`greeting.txt` is a plain UTF-8 text file with the single line `hello`. No build step reads it.
EOF
  cat >"$dir/specs/greeting/spec.md" <<'EOF'
## Purpose

The greeting file that welcomes contributors at the repository root, and nothing else.

## ADDED Requirements

### Requirement: Greeting file

The repository SHALL contain a `greeting.txt` file at its root whose content is `hello`.

#### Scenario: Greeting file exists

- **WHEN** a contributor opens the repository root
- **THEN** `greeting.txt` exists and its content is `hello`
EOF
  {
    echo '## 1. Greeting'
    echo
    echo '- [ ] 1.1 Create `greeting.txt` at the repository root with the content `hello`. Verify: `cat greeting.txt` prints `hello`.'
    if [ "${1:-}" = manual ]; then
      echo '- [ ] [M] 1.2 Open `greeting.txt` in an editor and confirm that it reads `hello`.'
    fi
  } >"$dir/tasks.md"
}

# add-greeting 的程式任務已完成：建立 greeting.txt 並以 task done 記錄。
eval_greeting_done() {
  speclink in-progress add add-greeting >/dev/null
  echo hello >greeting.txt
  speclink task done --change add-greeting 1 >/dev/null
}

# 已封存的 add-greeting：openspec/specs/greeting/spec.md 成為正式規格。
eval_greeting_archived() {
  eval_greeting_change
  eval_greeting_done
  speclink archive add-greeting --yes >/dev/null
}

# 在 git 中完成 add-greeting：change 先 commit、記錄 apply 基準，再做完程式任務並 commit。
eval_greeting_committed() {
  eval_greeting_change
  eval_greeting_artifacts_with_baseline
  eval_greeting_done
  eval_commit 'add-greeting: greeting.txt'
}

# 品質關卡用的 add-greeting：src/greet.js 已實作並 commit，但有一個真的錯誤——條件式把空字串
# 指派給 name，任何呼叫都回傳 "hello, "，違反規格的兩個 scenario，也是程式上的錯。
eval_buggy_greet_committed() {
  local dir=openspec/changes/add-greeting
  speclink new change add-greeting >/dev/null
  mkdir -p "$dir/specs/greeting"
  cat >"$dir/proposal.md" <<'EOF'
## Why

The app needs one place that builds the greeting text shown to contributors.

## What Changes

- Add `greet(name)` in `src/greet.js`.

## Capabilities

### New Capabilities

- `greeting`: The greet helper that builds the greeting text.

### Modified Capabilities

(None)

## Impact

- New file: `src/greet.js`
EOF
  cat >"$dir/design.md" <<'EOF'
## Context

The greeting text is built in several places today.

## Decisions

### One helper

`greet(name)` in `src/greet.js` is the only place that builds the greeting text.
EOF
  cat >"$dir/specs/greeting/spec.md" <<'EOF'
## Purpose

The greet helper that builds the greeting text shown to contributors, and nothing else.

## ADDED Requirements

### Requirement: Greet by name

`greet(name)` in `src/greet.js` SHALL return `hello, <name>`. When `name` is the empty string it SHALL return `hello`.

#### Scenario: Named greeting

- **WHEN** `greet` is called with `Ada`
- **THEN** it returns `hello, Ada`

#### Scenario: Empty name

- **WHEN** `greet` is called with the empty string
- **THEN** it returns `hello`
EOF
  cat >"$dir/tasks.md" <<'EOF'
## 1. Greet helper

- [ ] 1.1 Add `greet(name)` in `src/greet.js` that returns `hello, <name>`, or `hello` when `name` is empty. Verify: `node -e "const {greet}=require('./src/greet'); console.log(greet('Ada'), greet(''))"` prints `hello, Ada hello`.
EOF
  eval_greeting_artifacts_with_baseline
  speclink in-progress add add-greeting >/dev/null
  mkdir -p src
  cat >src/greet.js <<'EOF'
function greet(name) {
  if (name = '') return 'hello';
  return `hello, ${name}`;
}

module.exports = { greet };
EOF
  speclink task done --change add-greeting 1 >/dev/null
  eval_commit 'add-greeting: greet helper'
}
