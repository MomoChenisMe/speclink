# Spectra Discuss / Propose 服務化拆分分析

日期：2026-05-19

本文分析如果要把 `$spectra-discuss` 與 `$spectra-propose` 拆到自有服務上執行，而把 `$spectra-apply` 之後的實作流程保留給工程師在本機執行，需要承接哪些 CLI 能力、資料模型與流程邊界。

目標邊界：

- 服務端負責：需求討論、決策收斂、proposal/design/spec/tasks 產生、分析、驗證、停放。
- 工程師本機負責：取回或還原 change，執行 `$spectra-apply`、修改程式碼、`task done`、`archive`、commit 等後續工作。

## 總結

若只服務化 `discuss` 與 `propose`，最小 CLI 能力不是整個 Spectra CLI，而是這幾組：

```text
spectra list --json
spectra show <change-or-spec> --json
spectra new change <name> --agent <agent> [--description ...] [--schema ...]
spectra instructions <artifact> --change <name> --json
spectra status --change <name> --json
spectra new artifact <artifact> --change <name> --stdin [--json]
spectra new artifact spec <capability> --change <name> --stdin [--json]
spectra analyze <change> --json
spectra validate <change> [--json]
spectra park <change>
spectra schema which <name> --json --all
spectra schemas --json
spectra templates --json
```

`discuss` 本身幾乎不是 CLI 工作流，它主要需要「讀取專案上下文」與「記錄結論」。`propose` 才需要完整 Artifact DAG 與 CLI 驗證能力。

## Discuss 需要的功能

`$spectra-discuss` 是討論與收斂流程，不負責實作，也不一定建立 change。若放到服務端，核心能力應拆成以下模組。

### 1. 專案上下文讀取

用途：

- 讀 `openspec/LANGUAGE.md`，取得專案詞彙。
- 搜尋與 topic 相關的原始碼。
- 讀取既有 specs 或 change artifacts。
- 判斷要走 assumptions mode 或 interview mode。

對應 CLI：

```text
spectra list --json
spectra show <change-or-spec> --json
```

服務端還需要自己的 repo scanner：

```text
rg <keywords>
rg --files
read file
```

這部分不一定要包成 Spectra CLI，因為 skill 原本就是用 grep/glob/read file 做 scout。

### 2. 討論狀態機

服務端需要保存討論狀態：

```text
topic
mode: assumptions | interview
keywords
related_files
assumptions
user_corrections
open_questions
decisions
recommended_capture_target
```

Discuss 的重要行為：

- 一次只問一個問題。
- 如果找到足夠程式碼脈絡，列出 3-5 個 assumptions。
- 每個 assumption 必須有 approach、evidence、if wrong。
- 最後要有 conclusion，不應無限發散。

這些不是 CLI 功能，而是服務端對話邏輯。

### 3. 結論輸出與可選 artifact capture

Discuss 收斂後應輸出：

```markdown
## Conclusion

**Decision**: ...
**Rationale**: ...
**Capture to**: proposal.md / design.md / specs/<capability>/spec.md / tasks.md / openspec/LANGUAGE.md
```

如果使用者要求把討論結果變成 Spectra artifact，才會進入 propose 或 artifact 寫入流程。

## Propose 需要的功能

`$spectra-propose` 是主要需要服務化的部分。它的本質是「建立一個可交給 apply 的完整 Spectra change」。

### 1. Requirement intake

服務端需要接收需求來源：

- 使用者輸入的一句需求。
- discuss 的 conclusion。
- plan file 或外部設計文件。
- 既有 change 的補充上下文。

輸出：

```text
change_name: kebab-case
change_type: feature | bugfix | refactor
requirement_summary
known_constraints
related_specs
related_files
```

對應 CLI：

```text
spectra list --json
spectra show <spec> --json
```

用途是找既有 specs，避免重複建立 capability。

### 2. Change 建立

對應 CLI：

```text
spectra new change "<name>" --agent codex --description "<summary>"
```

可選：

```text
spectra new change "<name>" --agent codex --schema "<schema>"
```

服務端需要處理：

- change name kebab-case。
- active change 已存在時，回報並改走「延續既有 change」。
- 不沿用 archive 目錄的 `YYYY-MM-DD-` 前綴。
- 記錄 `.openspec.yaml` metadata。

### 3. Schema / artifact DAG resolution

對應 CLI：

```text
spectra schema which <schema> --json --all
spectra schemas --json
spectra templates --json
spectra status --change "<name>" --json
```

服務端需要知道：

- 當前 change 使用哪個 schema。
- 哪些 artifacts 必填。
- 哪些 artifacts blocked / ready / done。
- `applyRequires` 是哪些 artifact。
- 是否可以跳過 optional artifact。

最小資料模型：

```json
{
  "changeName": "add-foo",
  "schemaName": "spec-driven",
  "applyRequires": ["tasks"],
  "artifacts": [
    {
      "id": "proposal",
      "outputPath": "proposal.md",
      "status": "ready|blocked|done",
      "missingDeps": []
    }
  ]
}
```

### 4. Artifact instructions

對應 CLI：

```text
spectra instructions proposal --change "<name>" --json
spectra instructions specs --change "<name>" --json
spectra instructions design --change "<name>" --json
spectra instructions tasks --change "<name>" --json
```

服務端必須保留這個能力，因為 instructions 會回傳 schema-specific 的產物規則。

重要欄位：

```text
changeName
artifactId
schemaName
changeDir
outputPath
description
instruction
locale
template
dependencies
unlocks
context
rules
```

服務端產生 artifact 時：

- `template` 是輸出結構。
- `instruction` 是 artifact-specific guidance。
- `context` / `rules` 是生成約束，不應原樣寫進檔案。
- `dependencies` 指出要讀哪些已完成 artifacts。
- spec 檔即使專案 locale 是中文，也應維持英文規範語句，因為會用 SHALL / MUST。

### 5. Artifact 寫入

對應 CLI：

```text
spectra new artifact proposal --change "<name>" --stdin --json
spectra new artifact design --change "<name>" --stdin --json
spectra new artifact tasks --change "<name>" --stdin --json
spectra new artifact spec <capability> --change "<name>" --stdin --json
```

需要支援：

- stdin content 寫入。
- `--json` 結果解析。
- validation error retry。
- `--force` 覆寫策略，但建議服務端預設不要自動 force。

CLI 已驗證的基本 validation：

```text
proposal: 需要 ## Why、## Problem 或 ## Summary
design:   需要 ## Context
tasks:    需要至少一個 checkbox (- [ ])
spec:     需要可解析為 delta spec
```

服務端應在呼叫 CLI 前先做一次自檢，避免反覆被 CLI 擋。

### 6. Proposal 生成規則

Feature 建議格式：

```markdown
## Why

## What Changes

## Non-Goals (optional)

## Capabilities

### New Capabilities

- `<capability-name>`: ...

### Modified Capabilities

(none)

## Impact

- Affected specs: ...
- Affected code:
  - New: ...
  - Modified: ...
  - Removed: ...
```

Bug fix 建議格式：

```markdown
## Problem

## Root Cause

## Proposed Solution

## Non-Goals (optional)

## Success Criteria

## Impact
```

Refactor / enhancement 建議格式：

```markdown
## Summary

## Motivation

## Proposed Solution

## Non-Goals (optional)

## Alternatives Considered (optional)

## Impact
```

重要：`Impact` 的路徑要相對於專案根目錄，例如：

```text
src/lib/foo.ts
src-tauri/crates/core/src/bar.rs
```

不要寫：

```text
parser/mod.rs
core/mod.rs
```

原因：Spectra preflight / drift 會把 backtick path 當 anchor，太片段的路徑可能被判斷為 non-anchored path。

### 7. Spec 生成規則

服務端要能從 proposal 的 Capabilities 拆出：

```text
New Capabilities -> ADDED Requirements
Modified Capabilities -> MODIFIED Requirements
Removed behavior -> REMOVED Requirements
Renamed behavior -> RENAMED Requirements
```

每個 capability 一個檔案：

```text
openspec/changes/<change>/specs/<capability>/spec.md
```

寫入 CLI：

```text
spectra new artifact spec <capability> --change "<name>" --stdin --json
```

每個 requirement 至少應有：

```markdown
### Requirement: <name>
The system SHALL ...

#### Scenario: <scenario name>
- **WHEN** ...
- **THEN** ...
```

建議補 `##### Example:`，因為 analyzer 可能對抽象 scenario 給 suggestion。

### 8. Design 生成規則

Design 是讓工程師理解「為什麼這樣做」的交接文件，不應只列檔名。

建議至少包含：

```markdown
## Context

## Goals / Non-Goals

## Decisions

### <Decision Heading>
...

## Risks / Trade-offs
```

注意：analyzer 會把 `###` decision heading 正規化後檢查是否出現在 `tasks.md`。所以如果 design 有：

```markdown
### ProbeAdapter
```

tasks 需要提到 `ProbeAdapter`，否則會出：

```text
CON-1 / conDesignNotInTasks
Design topic 'probeadapter' not referenced in tasks
```

### 9. Tasks 生成規則

Tasks 是工程師本機 apply 的主要輸入，必須可執行、可驗證、可交接。

基本格式：

```markdown
## Implementation Tasks

- [ ] 1. Implement ...
- [ ] 2. Add tests for ...
- [ ] 3. Verify ...
```

避免：

- 只有檔案路徑，沒有行為描述。
- 依賴不穩定行號，例如「改第 42 行」。
- 「處理 edge cases」但不列出 edge cases。
- 「參考 Task 1」但不重述具體工作。
- 超過 15 個 pending tasks 仍不拆 change。

如果 `.spectra.yaml` 設 `parallel_tasks: true`，服務端可在互不依賴且不同檔案的 tasks 加 `[P]`：

```markdown
- [ ] [P] 2. Add parser tests for ...
```

### 10. Analyze / fix loop

對應 CLI：

```text
spectra analyze "<name>" --json
```

服務端至少要處理 Critical / Warning：

```text
covMissingSpec
covMissingTask
ambNoScenario
ambWeakLanguage
conDesignNotInTasks
gapNoMainSpec
gapModifiedNotFound
```

建議流程：

1. 跑 `spectra analyze <change> --json`
2. 只處理 Critical / Warning，Suggestion 可列為改善項。
3. 修 artifact。
4. 最多重跑 2 次。
5. 若仍有 Warning，保留摘要但不一定阻塞。

### 11. Validate

對應 CLI：

```text
spectra validate "<name>" --json
```

服務端應把 validation error 視為阻塞，因為工程師本機 apply 前 artifact 應該可解析。

常見輸出：

```json
[
  {
    "change": "add-foo",
    "valid": true,
    "errors": [],
    "warnings": []
  }
]
```

### 12. Park / handoff

Propose 結束時應 park，讓工程師本機 apply 時自動 unpark 或手動 unpark。

對應 CLI：

```text
spectra park "<name>"
```

服務端輸出給工程師：

```text
Change: <name>
Artifacts: proposal.md, design.md, tasks.md, specs/<cap>/spec.md
Status: validated and parked
Next local command: $spectra-apply <name>
```

若服務端與工程師本機不是同一 filesystem，不能只依賴 `.git/spectra-app`。需要額外設計同步方式。

## 服務端需要的 API 拆分

建議服務端 API：

```text
POST /discussions
POST /discussions/{id}/message
POST /discussions/{id}/conclude

POST /changes/propose
GET  /changes/{name}/status
GET  /changes/{name}/artifacts
POST /changes/{name}/artifacts
POST /changes/{name}/analyze
POST /changes/{name}/validate
POST /changes/{name}/park

GET  /schemas
GET  /schemas/{name}/resolution
GET  /changes
GET  /specs
```

內部可分成：

| 模組 | 責任 | 對應 CLI |
| --- | --- | --- |
| Discussion Engine | 問答、assumptions、conclusion | 無直接 CLI，讀 repo/spec 輔助 |
| Repository Scout | 搜尋程式碼與 specs | `rg`, file read, `spectra list/show` |
| Change Manager | 建 change、查狀態 | `new change`, `status` |
| Schema Resolver | schema/template/instructions | `schema which`, `schemas`, `templates`, `instructions` |
| Artifact Writer | 寫 proposal/design/spec/tasks | `new artifact` |
| Analyzer | 一致性檢查與修復建議 | `analyze` |
| Validator | 最終 validation | `validate` |
| Handoff Manager | park / 匯出交接包 | `park` |

## 服務端與本機交接策略

### 同一個 repo / shared filesystem

最簡單：

1. 服務端直接在 repo 建 change。
2. 服務端 `spectra park <change>`。
3. 工程師本機執行：

```text
$spectra-apply <change>
```

優點：最接近原始 Spectra 工作流。

風險：服務端需要 repo 寫入權限，且要避免多人同時操作同一 working tree。

### 不同機器 / SaaS 服務

建議交接包：

```text
change-name/
  .openspec.yaml
  proposal.md
  design.md
  tasks.md
  specs/<capability>/spec.md
  manifest.json
```

`manifest.json` 建議包含：

```json
{
  "change": "add-foo",
  "schema": "spec-driven",
  "createdBy": "service",
  "spectraVersion": "2.3.1",
  "artifacts": [
    "proposal.md",
    "design.md",
    "tasks.md",
    "specs/foo/spec.md"
  ],
  "validation": {
    "valid": true,
    "warnings": []
  }
}
```

工程師本機匯入方式可以是：

```text
copy change-name -> openspec/changes/<change-name>
spectra validate <change-name>
$spectra-apply <change-name>
```

如果仍想使用 parked 狀態，服務端需要能產生或同步 `.git/spectra-app/changes/<name>` 與 SQLite metadata；這比較脆弱，不建議作為跨機器交換格式。跨機器建議用 active change directory。

## 最小可行實作範圍

第一版建議只做這些：

```text
discuss:
  - topic session
  - repo/spec scout
  - assumptions/interview
  - conclusion summary

propose:
  - new change
  - instructions
  - status DAG
  - new artifact proposal/spec/design/tasks
  - analyze-fix loop
  - validate
  - export or park
```

可以延後：

```text
schema init/fork UI
custom schema editor
archive/unarchive
task done
drift
commit integration
vector search
apply implementation workflow
```

## 不建議服務端承接的部分

這些應留在工程師本機：

```text
$spectra-apply
spectra task done
spectra archive
spectra drift during implementation
git commit / PR creation
actual code edits
test execution requiring local services/secrets
```

原因：

- apply 需要改程式碼，通常依賴本機環境、測試服務、IDE、憑證或 branch 狀態。
- task done 的 touched-file tracking 依賴 Git worktree modified/staged files。
- archive 會把 delta spec merge 到 `openspec/specs`，最好在工程師確認實作完成後才做。

## 風險與注意事項

### 1. 不要把 CLI 當任意 shell

服務端應用 allowlist 呼叫 Spectra：

```text
spectra list
spectra show
spectra new change
spectra instructions
spectra status
spectra new artifact
spectra analyze
spectra validate
spectra park
spectra schema which
spectra schemas
spectra templates
```

不要提供任意 command passthrough。

### 2. Artifact 內容要經過路徑約束

尤其 proposal/design/tasks 內的 backtick path 會影響 drift/preflight。服務端應避免產生：

```text
`mod.rs`
`core.rs`
`git mv a b`
```

應產生：

```text
`src-tauri/crates/core/src/parser/mod.rs`
Parser entry module
```

### 3. 服務端要保留 analyzer 結果

每次 propose 完應保存：

```text
analyze raw JSON
validate raw JSON
artifact generation attempts
final artifact hashes
```

這能讓工程師知道 artifact 是否可信，也方便日後除錯。

### 4. Park 不一定適合跨機器

`spectra park` 會使用 `.git/spectra-app/changes/<name>` 與 SQLite。這很適合同機器工作流，但不適合 SaaS 到工程師本機的交換格式。

跨機器時，建議服務端交付 active change directory 或 PR，而不是直接交付 parked SQLite 狀態。

## 建議實作順序

1. 先做 `propose` 最小閉環：`new change -> instructions -> artifacts -> analyze -> validate -> export`。
2. 再做 `discuss` session：conclusion 能直接轉成 propose input。
3. 加入 schema resolution 與 custom schema 支援。
4. 加入 artifact diff / review UI。
5. 最後才考慮 park 同步、多人協作鎖、或與 Git provider 整合。

## 對工程師的本機流程

服務端完成 propose 後，工程師本機只需要：

```text
# 若服務端交付 active change directory
spectra validate <change-name>
$spectra-apply <change-name>

# 實作過程
spectra task done <task-id> --change <change-name>

# 完成後
spectra archive <change-name>
```

如果服務端與本機共享同一 repo 且已 park：

```text
$spectra-apply <change-name>
```

`$spectra-apply` / `$spectra-ingest` skill 會處理 parked change 的還原。

