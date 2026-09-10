# improve-skill Specification

## Purpose

/speclink-improve 技能的內容：以六步骨架渲染到 claude 與 codex 兩種工具、掃描精髓段在渲染時逐字保留，以及掃描出的改進候選以討論記錄承載。本 capability 保證改進掃描的結果落成可繼續討論的文件而非一次性輸出，且兩種工具拿到的指引內容一致。

## Requirements

### Requirement: improve 技能以六步骨架渲染至兩工具

內嵌 speclink-improve 技能(事實來源 crates/engine/speclink-core/assets/skills/improve.md,經 init 與 update 渲染至 claude 與 codex 工具技能目錄)SHALL 以六步骨架規定流程:載入詞彙(speclink language show)、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂;並 SHALL 標示技能僅由使用者發起、模型 SHALL NOT 自行觸發、SHALL NOT 於流程中實作程式碼。防重提檢查 SHALL 規定:開場以範圍關鍵字執行 speclink discuss search(--json)取得在途與封存討論中 Ruled out 與結論的命中,閱讀順序 SHALL 將 kind 為 improve 且同範圍的舊記錄排前;已否決方案 SHALL NOT 再列為 candidate,除非敘明當時否決理由已失效;並讀取 speclink list 的 in-flight changes,與其重疊區域的 candidate SHALL NOT 提出。技能檔 SHALL NOT 再指示以 speclink discuss list --archived 後逐筆 show 作為防重提的讀取方式。本能力屬 Speclink 自身延伸;渲染產物內容由 speclink-core 的 render_golden 測試(cargo test)保護,golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含六步骨架

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-improve 技能檔 SHALL 依序含載入詞彙、防重提檢查、範圍收斂、掃描、建記錄呈現 candidates、grilling 收斂六步,且含「僅使用者發起」與「不得實作」的限定

#### Scenario: 防重提檢查涵蓋已封存討論與 in-flight changes

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的防重提段落
- **THEN** 內容 SHALL 規定以 speclink discuss search 取得已封存與在途討論的 Ruled out 與結論命中以排除已否決方案、同範圍舊 improve 記錄排前閱讀,並規定避開 in-flight changes 的重疊區域;SHALL NOT 含 speclink discuss list --archived 後逐筆 show 的指示


<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->

---
### Requirement: 掃描精髓段逐字保留

渲染產出的 speclink-improve 技能檔 SHALL 完整保留源自 improve-codebase-architecture 的掃描精髓,SHALL NOT 濃縮:(1) 範圍收斂——使用者點名方向(模組/子系統/痛點)時 SHALL 直接採用並跳過推斷;否則 SHALL 以 git log 熱點推斷並加權近期常變區域,輔以 openspec/changes/archive 的 touched 記錄;熱點分散無焦點時 SHALL 放寬網。(2) 掃描 SHALL 列出全部六條 friction 訊號:概念理解需跳多個小模組、interface 複雜度逼近實作的 shallow module、為測試抽純函式但 bug 藏在呼叫端、緊耦合跨 seam 洩漏、難以透過現行 interface 測試的區域、以及分組只靠檔名猜(同一概念的檔案散在平鋪目錄、讀者靠前綴或字母序重拼分組,或目錄與既有設計文件的分層對不上);並 SHALL 保留「有機探索、不逐條打勾」的精神敘述。(3) deletion test SHALL 為前五條訊號的 candidate 准入判準:刪除後複雜度集中才成立,僅搬家不成立。(4) 掃描機制 SHALL 規定 inline 為預設,僅於未指定方向或範圍跨 crate 時派 Explore subagent,且數量硬上限為 2。(5) 第六條訊號 SHALL 帶自己的准入測試,緊接 deletion test 段落敘明:改問讀者預測測試(第一次打開該目錄的人能否不靠搜尋預測功能住哪、哪些檔案是一組),且 candidate 須同時滿足三條件——分組有客觀依據(模組引用方向、既有設計文件的分層、對外連結)、對呼叫者不可見(既有路徑經根層 re-export 維持,無法 re-export 的面以守門測試斷言無殘留舊路徑)、搬移本身即整個變更(需要每個呼叫端改路徑的只是搬家,SHALL NOT 列為 candidate);candidate 同時命中前五條訊號與第六條時 SHALL 先以 deletion test 裁決,刪除後複雜度集中者屬程式碼 candidate、第六訊號只涵蓋原樣搬移者。

#### Scenario: 六條 friction 訊號逐條在列

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描段落
- **THEN** 六條 friction 訊號 SHALL 逐條出現,且 deletion test 以「複雜度集中才算、搬家不算」的判準敘述並限定於前五條

#### Scenario: 第六訊號帶讀者預測測試

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔 deletion test 段落之後的內容
- **THEN** 內容 SHALL 敘明第六訊號不適用 deletion test、改問讀者預測測試,並列出分組有客觀依據、對呼叫者不可見、搬移即整個變更三條件;claude 與 codex 兩工具的技能實例 SHALL 同步含此段

#### Scenario: subagent 上限與觸發判準

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的掃描機制段落
- **THEN** 內容 SHALL 規定 inline 為預設、Explore subagent 僅於未指定方向或跨 crate 時派出、硬上限 2


<!-- @trace
source: improve-layout-signal
updated: 2026-09-10T21:20:30+08:00
-->

---
### Requirement: candidates 以討論記錄承載

渲染產出的 speclink-improve 技能檔 SHALL 規定:掃描完成後以 speclink discuss new 帶 --kind improve 與 --slug(慣例 improve-<範圍>)建立討論記錄,candidates 以 Round 1(mode 標籤 scan)記錄,每個 candidate SHALL 含 Files、Problem、Solution、Wins、建議強度(強烈建議/值得探索/尚屬臆測三級)五欄位,結尾 SHALL 含首選建議並詢問使用者深入哪一個。grilling SHALL 沿用一次一題、提案帶證據的紀律,interface depth check SHALL 對每個被挑中的 candidate 無條件執行;對第六訊號(分組只靠檔名猜)的 candidate,四問 SHALL 改為分組邊界的客觀來源、既有路徑靠什麼維持不變、搬移前盤點的路徑風險(內嵌路徑的公開網址、tag 觸發或依路徑篩選的 CI workflow、以相對路徑寫的建置期檔案引入、跨套件 path 相依)、以及回到平鋪讀者失去什麼,並 SHALL 附帶兩條做法:測試鏡射原始碼分組搬移(內嵌測試逾行數門檻時搬至同名子檔)、切刀依路徑相依排序(先動目錄本身的刀、再動目錄內分組的刀,SHALL NOT 以 worktree 平行)。收斂 SHALL 走 conclude,經 promote 或 link 扇出變更;結論規劃分期立案(先立一刀、封存後再回同一記錄轉出下一刀)時 conclude SHALL 帶 --hold 一次,hold 不因轉出而清除、只由不帶 --hold 的 conclude 或 speclink discuss archive 解除,最後一刀封存後 SHALL 教執行 speclink discuss archive <slug> 收尾;未帶旗標的記錄在最後一個轉出變更封存時隨行封存、之後的刀 SHALL 走新討論;使用者全數否決時 SHALL 仍以 conclude(記明不做與理由)加 archive 收尾,SHALL NOT discard。

#### Scenario: candidates 記錄形式

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的建記錄段落
- **THEN** 內容 SHALL 規定 discuss new 帶 --kind improve、Round 1 以 mode 標籤 scan 記錄、candidate 五欄位與三級建議強度、結尾首選建議

#### Scenario: 全數否決仍留記錄

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的收斂段落
- **THEN** 內容 SHALL 規定全數否決時走 conclude 加 archive,並 SHALL 含禁止 discard 的敘述

#### Scenario: 分期立案帶 --hold

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的扇出段落
- **THEN** 內容 SHALL 規定結論規劃分期立案時 conclude 帶 --hold 一次、hold 只由不帶 --hold 的 conclude 或 discuss archive 解除、最後一刀封存後執行 discuss archive 收尾、未帶旗標時後續刀走新討論;SHALL NOT 含「旗標由下一次轉出清除」;claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映

#### Scenario: 第六訊號的深度四問對應版

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔 grilling 段落的 interface depth check 條目
- **THEN** 內容 SHALL 於既有四問之後規定第六訊號的對應四問(分組來源、路徑維持、路徑風險盤點、讀者預測測試)取代既有四問,並含測試鏡射搬移與切刀依路徑相依排序、不以 worktree 平行的敘述;Guardrails 的 deletion test 一條 SHALL 註明第六訊號改答讀者預測測試


<!-- @trace
source: improve-layout-signal
updated: 2026-09-10T21:20:30+08:00
-->