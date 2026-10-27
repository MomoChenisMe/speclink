## Why

封存合併引擎目前對過期 delta 靜默跳過（ADDED 撞名不套用、MODIFIED／REMOVED／RENAMED 缺目標無事發生），逐 capability 邊驗邊寫使多 capability 封存可能半完成，且 MODIFIED 整段替換會無聲吃掉 delta 漏抄的 scenario——這些都是真實的規格資料遺失路徑（OpenSpec 同型事故記錄於其 parallel-merge-plan：後封存的 change 無警告地蓋掉先前 change 已合入的 scenario）。守門目前只存在於 drift 的提醒與 bulk archive 的預檢，單筆 archive 引擎本身不拒絕。討論 post-archive-spec-value 裁決：合併引擎改為 fail-closed。

目標使用者是透過 AI 代理跑 SDD 的開發者，情境對應 archive 工作流階段（`/speclink-archive` 技能與 speclink archive CLI 動詞，含單筆與 bulk 兩型）。

## What Changes

- 合併引擎改硬性守門，以下情形一律拒絕封存（correctness 級，無任何旁路旗標）：
  - ADDED 的需求名已存在於正典
  - MODIFIED／REMOVED／RENAMED 的來源需求名不存在於正典
  - 同一需求名出現在同一 delta 的多個操作區段
  - RENAMED 的目標名已存在於正典
  - MODIFIED 區塊缺少正典目標需求既有的任何 scenario，且未以明示標記聲明刪除（scenario superset check）
  - 正典尚不存在的新 capability 出現 ADDED 以外的操作（現行「MODIFIED 也物化成新規格」的行為移除）
- 套用順序改為兩階段：先讀取並驗證全部 capability 的 delta、產生完整合併計畫，全數通過後才開始寫入（含 snapshot 備份與正典寫回）；任何一條失敗則整個封存不落地、change 留在原位
- 新 capability 的正典 Purpose 自 delta 的 `## Purpose` 區段帶入；delta 未提供才落 TBD 骨架（既有 capability 的 Purpose 不受 delta 影響，維持現行）
- 拒絕訊息逐條列明不符項（capability、操作、需求名、原因），並指引補救動線：先跑 speclink drift，再以 ingest 更新 delta
- speclink archive 的 `--no-validate` 語意不變：仍只略過文件驗證，不解鎖上述合併錯誤；`--skip-specs` 仍可整段跳過規格套用（既有逃生口，不新增旗標）
- drift 的 spec assumption reason 文案由「archive would skip it」改為拒絕語意，與新引擎行為一致
- archive 技能文字的背景敘述同步更新：移除「引擎靜默跳過、封存前先把既存 ADDED 轉 MODIFIED 以觸發重注入」的補救教學，改為「引擎直接拒絕、依錯誤清單修 delta」

**相容性影響**：

- 行為面 **BREAKING**：先前可完成（但靜默丟資料）的封存改為以非零 exit code 拒絕；受影響者為帶過期 delta 的 in-flight change，補救動線是既有的 drift → ingest，無資料遷移
- 人眼與 `--json` 輸出：archive 新增拒絕錯誤輸出；drift 的 assumption reason 字串變更——兩者皆屬刻意變更，golden（render_golden）與 CLI 整合測試同批更新
- drift 判定收斂單源後，`spec_assumptions` 的回報範圍隨守門擴大（scenario 漏抄、畸形註解、孤兒 RENAMED 等新違規類別），`operation` 欄於多區段互撞時為逗號串接的多值；JSON 欄位結構不變，但 Specs 維度分數與 primary recommendation 可能隨之變動——皆屬刻意變更
- bulk archive 的 readiness 預檢保留（提前過濾的 UX 較好），引擎守門成為第二道；兩者判定一致
- 已封存的 125 份歷史 change 與既有正典規格完全不受影響

## Non-Goals

- 不新增 sync 動詞——archive 維持唯一物化點，也不移植 OpenSpec 的 early-sync no-op 例外（內容相同視為已同步）
- 不動 @trace 內容、evidence 記錄位置與 evidence gate——屬第二刀 change（evidence 隨行與 trace 瘦身）
- 不做 requirement 基準 fingerprint 與 CAS 衝突判定（後置，OpenSpec parallel-merge-plan Phase 0 為現成參考）
- 不動桌面 UI 與 server 路由——拒絕錯誤沿既有 Refusal 通道呈現
- 不改 validate 的文件品質檢查範圍

## Capabilities

### New Capabilities

- `archive-merge`: 封存時 delta 併入正典規格的合併引擎語意——fail-closed 守門清單、兩階段合併計畫、新 capability 的 Purpose 帶入

### Modified Capabilities

(none)

## Impact

- Affected specs: archive-merge（new）
- Affected code:
  - Modified: crates/speclink-core/src/archive.rs（合併守門、兩階段套用、Purpose 帶入；既有「靜默跳過」凍結測試翻轉為拒絕測試）
  - Modified: crates/speclink-core/src/drift.rs（spec assumption reason 文案對齊拒絕語意）
  - Modified: crates/speclink-core/src/model.rs（delta 解析補 scenario 名清單與 Purpose 區段擷取，若現有解析不足）
  - Modified: crates/speclink-cli/src/commands.rs（單筆與 bulk 的錯誤呈現對齊）
  - Modified: crates/speclink-core/assets/skills/archive.md（背景敘述與補救動線改寫）
  - Modified: crates/speclink-core/tests/render_golden.rs 及 crates/speclink-cli/tests/ 相關整合測試（golden 同批更新）
  - New: (none)
  - Removed: (none)
