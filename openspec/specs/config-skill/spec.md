# config-skill Specification

## Purpose

TBD - created by archiving change 'workflow-config-verb-and-skill'. Update Purpose after archive.

## Requirements

### Requirement: 內嵌 speclink-config 技能的渲染與保護

內嵌 speclink-config 技能（事實來源 crates/speclink-core/assets/skills/config.md）SHALL 經 init 與 update 渲染至各工具技能目錄（claude 與 codex），與既有內嵌技能同機制；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: init 與 update 渲染技能

- **WHEN** 執行 speclink init 或 speclink update 且工具含 claude 與 codex
- **THEN** 兩工具的技能目錄各生成 speclink-config 技能檔，內容源自 config.md 資產的渲染

#### Scenario: golden 保護渲染產物

- **WHEN** config.md 資產變更後執行 cargo test 的 render_golden 測試
- **THEN** 快照不符時測試失敗；刻意變更以快照再生落地並可審視 diff


<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: 技能規定固定輸入來源與四條內容判準

渲染產出的 speclink-config 技能檔 SHALL 規定：掃描輸入限固定的結構性來源——workspace 清單與相依 manifest（含 Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 的相依清單）、README、docs 索引、既有 openspec/config.yaml、以及 speclink language show 的共用詞彙（若有）；SHALL NOT 全 repo 掃描原始碼。技能檔 SHALL 載明四條內容判準：(1) 已由政策開關或 schema 內建 instruction 自動注入的內容，context 與 rules 皆不得重述——判定 SHALL 以 speclink instructions <artifact> --json 取得的實際 payload 逐條反證，不得憑印象；(2) 只對單一 artifact 咬合的內容歸 rules，不入 context；(3) 會過時的內容（版本號、計數、統計數字）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行皆核實。

#### Scenario: 渲染產物含固定來源清單

- **WHEN** 檢視渲染產出的 speclink-config 技能檔
- **THEN** 技能檔載明固定輸入來源清單（manifest、README、docs 索引、既有 config.yaml、language show），並明示不做全 repo 原始碼掃描

#### Scenario: 渲染產物含四條判準與反證步驟

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準段落
- **THEN** 四條判準俱在，且判準一明定以 speclink instructions <artifact> --json 的 payload 逐條反證、判準四明定引用存在性核實


<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: 技能規定 diff 先行與收斂驗收

渲染產出的 speclink-config 技能檔 SHALL 規定執行流程：整理結果 SHALL 先以 workflow-config 動詞的 --dry-run 產出 unified diff 呈現給使用者，經使用者確認後才執行實際寫入；SHALL NOT 未經確認直接寫入。政策四欄（locale、spec_locale、tdd、audit）SHALL 逐項詢問使用者、不由技能推斷。技能檔 SHALL 載明收斂驗收：對同一未變動的 codebase 連續執行兩次，第二次產出的 diff SHALL 為空——第二次仍有 diff 即為判準執行不當，SHALL 回查判準而非落檔。

#### Scenario: 渲染產物規定 dry-run 先行

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的執行流程段落
- **THEN** 流程明定先以 --dry-run 產 diff、經使用者確認後才寫入，且政策四欄逐項詢問

#### Scenario: 渲染產物載明收斂驗收

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的驗收段落
- **THEN** 載明同一未變動 codebase 連跑兩次、第二次 diff 為空的驗收條件，及第二次仍有 diff 時回查判準的處置

<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: 技能規定政策語系欄位寫入代碼

渲染後的 speclink-config 技能文件（所有 tool flavor）SHALL 於政策欄位段落明文規定：locale 僅接受語系代碼 tw、ja、en，spec_locale 僅接受 tw、ja、en、auto；SHALL 要求執行技能的 agent 把使用者的自然語言回答映射為代碼後寫入，並 SHALL 附至少一組映射示例（「繁體中文」→ tw）；SHALL 明文禁止把顯示名稱字串當作值寫入。內嵌資產與 repo 技能實例的同步一致性歸既有需求「內嵌 speclink-config 技能的渲染與保護」管轄，本需求 SHALL NOT 另立同步機制。

#### Scenario: 渲染文件含代碼指引

- **WHEN** 於啟用 claude 工具的專案渲染 speclink-config 技能
- **THEN** 產出的技能文件含 locale 與 spec_locale 的合法代碼集合、「繁體中文」→ tw 的映射示例，以及禁止寫入顯示名稱的指示

#### Scenario: 技能執行不再寫入顯示名稱

- **WHEN** agent 依更新後的技能文件執行設定流程，使用者以「繁體中文」回答語系偏好
- **THEN** agent 對 workflow-config set 寫入的值為 tw 而非「繁體中文」

<!-- @trace
source: workflow-config-locale-validation
updated: 2026-07-30
code:
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/policy_write.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->