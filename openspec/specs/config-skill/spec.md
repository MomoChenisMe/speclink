# config-skill Specification

## Purpose

/speclink-config 技能的內容：技能檔的渲染與保護、撰寫工作流 context 與 rules 時的固定輸入來源與四條內容判準、diff 先行再收斂驗收的落地方式、政策語系欄位的寫入代碼，以及任務驗證測試範圍的第五問。本 capability 保證設定內容由程式碼實況推導而非憑印象撰寫，且任何寫入都先讓使用者過一次 diff 才落地。

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

渲染產出的 speclink-config 技能檔 SHALL 規定：掃描輸入限固定的結構性來源——workspace 清單與相依 manifest（含 Cargo workspace 成員與 workspace 相依、關鍵邊界相依、各 package 的相依清單）、README、docs 索引、既有 openspec/config.yaml、以及 speclink language show 的共用詞彙（若有）；SHALL NOT 全 repo 掃描原始碼。技能檔 SHALL 載明四條內容判準：(1) 已由政策開關或 schema 內建 instruction 自動注入的內容，以及品質站技能已承載的正典標準（如審查站的 smell baseline），context 與 rules 皆不得重述——引擎注入內容的判定 SHALL 以 speclink instructions <artifact> --json 取得的實際 payload 逐條反證，品質站正典的判定 SHALL 對照生成的品質站技能檔內容，皆不得憑印象；(2) 只對單一 artifact 咬合的內容歸 rules，不入 context；(3) 會過時的內容（版本號、計數、統計數字）不寫；(4) context 與 rules 引用的驗證手段（指令、測試名、路徑）必須實際存在於 repo，每次執行皆核實——核實 SHALL 以靜態便宜手段進行（路徑查檔案系統、測試名以文字搜尋命中原始碼、npm script 查 package.json 宣告、CLI 子指令對照 --help 輸出），SHALL NOT 執行被引用的測試或建置指令；判準一的 speclink instructions <artifact> --json payload 探測不受此限。

技能檔 SHALL 載明刪除理由限定：一條既有 rule 只因不過四條判準、或使用者本人於政策詢問中明確撤回而被刪除，SHALL NOT 因「無法自固定輸入來源導出」而被刪除——使用者裁決後落地的 rule（如討論結論轉入者）因此不需任何標記即受保護，且落地裁決與撤回裁決同源。

技能檔 SHALL 載明 scope hint 的收窄語意：呼叫帶範圍提示時，判準一至三的全面重審收窄至範圍內的 artifacts；判準四的引用核實恆為全文件掃描；未帶範圍提示時維持全文件重審。

#### Scenario: 渲染產物含固定來源清單

- **WHEN** 檢視渲染產出的 speclink-config 技能檔
- **THEN** 技能檔載明固定輸入來源清單（manifest、README、docs 索引、既有 config.yaml、language show），並明示不做全 repo 原始碼掃描

#### Scenario: 渲染產物含四條判準與反證步驟

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準段落
- **THEN** 四條判準俱在，且判準一明定以 speclink instructions <artifact> --json 的 payload 逐條反證引擎注入內容、明定品質站技能承載的正典標準對照生成技能檔反證、判準四明定引用存在性以靜態手段核實

#### Scenario: 品質站正典不得重述進 rules

- **WHEN** 候選 rules 條目與品質站技能檔內嵌的正典標準同義（如要求避免特定 code smell）
- **THEN** 技能檔的判準指引該條目被淘汰，正典標準維持品質站技能檔單一落點

#### Scenario: 判準四核實不執行引用指令

- **WHEN** 檢視渲染產出的 speclink-config 技能檔的判準四段落
- **THEN** 段落明定驗證引用以靜態手段（檔案系統、文字搜尋、package.json 宣告、--help 對照）進行、明文禁止執行被引用的測試或建置指令，並明示判準一的 payload 探測不在禁令範圍

#### Scenario: 使用者裁決型 rule 不因來源被刪

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於既有 rule 汰留的段落
- **THEN** 段落明定 rule 只因不過四條判準或使用者明確撤回而被刪，「無法自固定輸入來源導出」不構成刪除理由

#### Scenario: scope hint 收窄語意

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於範圍提示的段落
- **THEN** 段落明定範圍提示收窄判準一至三的重審至範圍內 artifacts、判準四恆為全文件掃描、未帶提示時全文件重審


<!-- @trace
source: config-skill-rule-alignment
updated: 2026-08-07
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

---
### Requirement: 技能規定任務驗證測試範圍的第五問

渲染產出的 speclink-config 技能檔 SHALL 於政策詢問流程增列第五問：任務清單（tasks）的驗證步驟要包含全量測試，或只跑受影響面的測試——與政策四欄同性質，SHALL 逐項詢問使用者、不得自 repo 推斷；現行文件已有測試範圍相關 rule 時，提問 SHALL 帶出現值供確認。使用者答「只跑受影響面」時，技能檔 SHALL 指引自已讀取的 dependency manifests 組出該專案客製的對應規則（按專案的組件型態對應其測試指令）寫入 rules 的 tasks 段，並沿既有 dry-run 核准流程落地；使用者答「全量」時 SHALL NOT 寫入任何測試範圍規則——現行文件已有測試範圍 rule 時，該答案即為使用者對其之明確撤回，技能檔 SHALL 指引沿同一 dry-run 核准流程移除之，無既有 rule 時現行文件維持原樣。

#### Scenario: 渲染產物含第五問

- **WHEN** 檢視渲染產出的 speclink-config 技能檔（claude 與 codex 兩 flavor）的政策詢問段落
- **THEN** 段落載明任務驗證測試範圍的第五問、與四欄同樣不得推斷、以及現行文件已有測試範圍 rule 時帶現值確認的指引

#### Scenario: 答受影響面時組出客製規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定自已讀取的 dependency manifests 組出專案客製的測試對應規則、寫入 rules 的 tasks 段、經 dry-run 核准後落地

#### Scenario: 答全量時不寫規則

- **WHEN** 檢視渲染產出的 speclink-config 技能檔關於第五問後續處理的段落
- **THEN** 段落明定使用者選擇全量時不寫入任何測試範圍規則；現行文件已有測試範圍 rule 時視為明確撤回、經 dry-run 核准移除該 rule，無既有 rule 時現行文件維持原樣

<!-- @trace
source: config-skill-rule-alignment
updated: 2026-08-07
-->