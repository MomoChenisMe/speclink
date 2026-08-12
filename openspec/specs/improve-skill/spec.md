# improve-skill Specification

## Purpose

/speclink-improve 技能的內容：以六步骨架渲染到 claude 與 codex 兩種工具、掃描精髓段在渲染時逐字保留，以及掃描出的改進候選以討論記錄承載。本 capability 保證改進掃描的結果落成可繼續討論的文件而非一次性輸出，且兩種工具拿到的指引內容一致。

## Requirements

### Requirement: improve 技能以六步骨架渲染至兩工具

內嵌 speclink-improve 技能(事實來源 crates/speclink-core/assets/skills/improve.md,經 init 與 update 渲染至 claude 與 codex 工具技能目錄)SHALL 以六步骨架規定流程:載入詞彙(speclink language show)、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂;並 SHALL 標示技能僅由使用者發起、模型 SHALL NOT 自行觸發、SHALL NOT 於流程中實作程式碼。防重提檢查 SHALL 規定:開場讀取開放與已封存討論(speclink discuss list 含 --archived)的 Ruled out 與結論,已否決方案 SHALL NOT 再列為 candidate,除非敘明當時否決理由已失效;並讀取 speclink list 的 in-flight changes,與其重疊區域的 candidate SHALL NOT 提出。本能力屬 Speclink 自身延伸;渲染產物內容由 speclink-core 的 render_golden 測試(cargo test)保護,golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含六步骨架

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-improve 技能檔 SHALL 依序含載入詞彙、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂六步,且含「僅使用者發起」與「不得實作」的限定

#### Scenario: 防重提檢查涵蓋已封存討論與 in-flight changes

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的防重提段落
- **THEN** 內容 SHALL 規定讀取已封存討論的 Ruled out 與結論以排除已否決方案,並規定避開 in-flight changes 的重疊區域


<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: 掃描精髓段逐字保留

渲染產出的 speclink-improve 技能檔 SHALL 完整保留源自 improve-codebase-architecture 的掃描精髓,SHALL NOT 濃縮:(1) 範圍收斂——使用者點名方向(模組/子系統/痛點)時 SHALL 直接採用並跳過推斷;否則 SHALL 以 git log 熱點推斷並加權近期常變區域,輔以 openspec/changes/archive 的 touched 記錄;熱點分散無焦點時 SHALL 放寬網。(2) 掃描 SHALL 列出全部五條 friction 訊號:概念理解需跳多個小模組、interface 複雜度逼近實作的 shallow module、為測試抽純函式但 bug 藏在呼叫端、緊耦合跨 seam 洩漏、難以透過現行 interface 測試的區域;並 SHALL 保留「有機探索、不逐條打勾」的精神敘述。(3) deletion test SHALL 為 candidate 准入判準:刪除後複雜度集中才成立,僅搬家不成立。(4) 掃描機制 SHALL 規定 inline 為預設,僅於未指定方向或範圍跨 crate 時派 Explore subagent,且數量硬上限為 2。

#### Scenario: 五條 friction 訊號逐條在列

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描段落
- **THEN** 五條 friction 訊號 SHALL 逐條出現,且 deletion test 以「複雜度集中才算、搬家不算」的判準敘述

#### Scenario: subagent 上限與觸發判準

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描機制段落
- **THEN** 內容 SHALL 規定 inline 為預設、Explore subagent 僅於未指定方向或跨 crate 時派出、硬上限 2


<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: candidates 以討論記錄承載

渲染產出的 speclink-improve 技能檔 SHALL 規定:掃描完成後以 speclink discuss new 帶 --kind improve 與 --slug(慣例 improve-<範圍>)建立討論記錄,candidates 以 Round 1(mode 標籤 scan)記錄,每個 candidate SHALL 含 Files、Problem、Solution、Wins、建議強度(強烈建議/值得探索/尚屬臆測三級)五欄位,結尾 SHALL 含首選建議並詢問使用者深入哪一個。grilling SHALL 沿用一次一題、提案帶證據的紀律,interface depth check SHALL 對每個被挑中的 candidate 無條件執行。收斂 SHALL 走 conclude,經 promote 或 link 扇出變更;使用者全數否決時 SHALL 仍以 conclude(記明不做與理由)加 archive 收尾,SHALL NOT discard。

#### Scenario: candidates 記錄形式

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的建記錄段落
- **THEN** 內容 SHALL 規定 discuss new 帶 --kind improve、Round 1 以 mode 標籤 scan 記錄、candidate 五欄位與三級建議強度、結尾首選建議

#### Scenario: 全數否決仍留記錄

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的收斂段落
- **THEN** 內容 SHALL 規定全數否決時走 conclude 加 archive,並 SHALL 含禁止 discard 的敘述

<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->