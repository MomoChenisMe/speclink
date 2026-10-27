## Why

archive 在合併 delta 進正典之前有一道 fail-closed 的合併守門（archive-merge「封存合併 fail-closed 守門」）：MODIFIED／REMOVED 的目標需求在正典已不存在、ADDED 撞既有需求名、MODIFIED 少寫正典 scenario 又沒宣告移除、RENAMED 缺 TO 目標或撞名，都會被拒收。這道判斷 drift 與 bulk archive 預檢已共用，但 `speclink validate` 沒有——結果是 propose 收尾 validate 全綠、analyze 全綠，一路做到 archive 才炸（本 repo 已有實例：MODIFIED 改了 scenario 名，直到 archive 才發現要補 REMOVED-SCENARIO 宣告）。討論 analyze-rule-tuning 的 Deferred 已裁定另開此 change：讓 validate 早期抓到 archive 會拒收的 delta。

## What Changes

- `speclink validate <change>`（含無參數、`--all`、`--changes` 的逐 change 驗證，fs 與 remote 兩模式）對每個 delta capability 執行與 archive 相同的合併守門判斷，每筆違規化為一條 error，使該 change 驗證結果為 invalid。
- 已由既有結構檢查報過的項目不重複列：新開 capability 的 Purpose 不合格只報既有的 Purpose error（含範例骨架），同一 delta 內同名需求只報既有的 Duplicate／appears in both error；RENAMED 端點撞名這類只有守門才看得到的仍會列出。
- archive（單筆與 bulk）的驗證前置改呼叫「只含結構檢查」的驗證入口，archive 的拒絕訊息（Validation failed 與 merge_refusal 的聚合違規清單）逐位元不變。
- 守門 error 的文字：`specs/<capability>/spec.md: <operation> '<requirement>': <reason> (see: speclink drift <change>)`，reason 逐字沿用 archive 守門的凍結字串。
- 目標使用者：透過 AI 代理跑 SDD 的開發者；情境是 propose 收尾與 ingest 後跑 validate，以及 desktop 分析面板頂部的「結構驗證」列。

**相容性影響**：validate 的人眼輸出與 `--json` 形狀（`change`／`errors`／`valid`／`warnings` 四欄）不變，只是 errors 陣列多了守門類的字串、`valid` 在有違規時變 false，exit code 隨既有規則（任一 invalid 即非零）。既有結構 error 的文字與順序不變，守門 error 一律排在既有 error 之後。archive 的輸出不變。之前 validate 通過、但 archive 會拒收的 change，現在 validate 就會報 invalid——這是刻意變更；修法與 archive 拒收時相同（speclink drift 看漂移、/speclink-ingest 更新 delta）。CLI 指令的子指令、旗標、stdin 不變；不涉及設定欄位；不涉及生成的技能文字。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `spec-validation`: 新增需求「change 驗證納入合併守門」——validate 的逐 change 驗證 SHALL 執行合併守門並以 error 呈現，archive 的前置驗證 SHALL 維持只含結構檢查。
- `archive-merge`: 「過期判定單源共用」的共用者自三處擴為四處（加上 validate），四處對同一 delta 的過期認定一致。

## Impact

- Affected specs: `spec-validation`（ADDED 一條需求）、`archive-merge`（MODIFIED 一條需求）。
- Affected code:
  - Modified: crates/speclink-core/src/validate.rs（驗證入口拆成「結構檢查」與「結構檢查＋合併守門」兩個公開函式，守門 error 的組字與去重）、crates/speclink-core/src/archive.rs（單筆 archive 的驗證前置改呼叫結構檢查入口）、crates/speclink-cli/src/verbs/lifecycle.rs（bulk archive 的驗證前置改呼叫結構檢查入口）、crates/speclink-cli/tests/it/validate_specs.rs（新增守門 error 的 CLI 對照）
  - New: （無）
  - Removed: （無）
- 影響的 crate：speclink-core、speclink-cli。speclink-server 的 validate 端點與 apps/desktop/core 走同一個核心驗證入口，自動取得守門，不需改碼；crates/speclink-cli/tests/it/remote_verb_parity.rs 的 validate 對照只看形狀，不受影響。
- 文件與技能：docs 與 SKILL 文字未列舉 validate 的檢查項，不需改。
