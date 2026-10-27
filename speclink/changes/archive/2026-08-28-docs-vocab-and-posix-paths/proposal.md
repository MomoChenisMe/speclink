## Why

2026-08-28 推送後 CI 紅燈,兩個真問題:(1) ubuntu 與 windows 的詞彙守門測試(scripts/vocabulary-guard.test.mjs)抓到使用者文件 7 處 `openspec/LANGUAGE.md` 避免詞——避免詞正典在 8/14 與 8/22 落地,違規文案的 commit 在其後,但期間未推送,這次一起上去才在 CI 首次引爆;(2) windows 的 3 個路徑推導測試(scripts/docs-screenshots.test.mjs 的狀態目錄推導、工作路徑推導、備份計畫)自 8/14 推送起就失敗——實作用 path.join,在 Windows 把測試注入的 posix 假 home(/fake/home)轉成反斜線,違反跨平台正典「邏輯路徑用正斜線」。兩者都是實作偏離既有正典,把實作修回來即可,規格不動。

目標使用者:讀繁中使用者文件的開發者/PO/PM,以及仰賴 CI 綠燈守門的維護者。使用情境:對應品質守門(ui-copy-vocabulary 的詞彙守門)與 user-documentation 的截圖腳本條款。

## What Changes

- 使用者文件 7 處避免詞依正典改詞:
  - 「抽屜」→「詳情面板」共 5 處:docs/product-status.zh-TW.md(2 處)、docs/roadmap.zh-TW.md(2 處)、docs/verb-contract.zh-TW.md(1 處)。
  - 「追溯」→「溯源」共 2 處:docs/workflow.zh-TW.md。
  - 改詞後句子保持通順,語意不變;必要時微調前後字詞,但不重寫段落。
- scripts/docs-screenshots.mjs 的純路徑推導函式(stateDirsFor、pathsFor、manifestPathIn、backupPlanFor)改用 path.posix.join 組邏輯路徑。這些函式只推導路徑字串、不碰真實檔案系統;腳本實際執行僅在 macOS(搬 Library 下的狀態目錄),posix join 在 macOS 行為不變,Windows CI 上的測試則不再被反斜線弄炸。

## Non-Goals

- 不改 `openspec/LANGUAGE.md` 的詞彙正典,不放寬或跳過守門測試。
- 不改 scripts/vocabulary-guard.test.mjs 與 scripts/docs-screenshots.test.mjs 的既有斷言——測試是對的,錯的是實作與文案。
- 不更名程式碼識別符(RichDetailDrawer、SpecDrawer 等)與 CSS 類名——正典明文排除。
- 不動 docs-screenshots.mjs 的真實搬移邏輯、demo workspace 建置與 CLI 呼叫路徑。
- 不處理 macOS job 的 index.crates.io DNS 解析失敗——GitHub runner 基建飄移,重跑即復原,無程式碼可修。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none)——step-3 規格掃描結果:最接近的是 ui-copy-vocabulary(定義使用者可見文案面與避免詞守門)與 user-documentation(涵蓋截圖腳本條款),兩者的需求本身都正確且不變;dev-harness 與本變更無關。本變更是讓文案與實作回到這兩份既有規格的要求,需求層級零變更,故無 delta。

## Impact

- Affected specs: 無(不新增、不修改任何 spec)。
- Affected code:
  - Modified: docs/product-status.zh-TW.md、docs/roadmap.zh-TW.md、docs/verb-contract.zh-TW.md、docs/workflow.zh-TW.md、scripts/docs-screenshots.mjs
  - New: 無
  - Removed: 無
- 影響的 crate 或 app:無——只動 docs 與 scripts,不碰任何 Rust crate、Node SDK 或前端 app。
- 相容性影響:無 CLI 指令、人眼輸出或 `--json` shape 變更;golden 不受影響。既有測試 scripts/vocabulary-guard.test.mjs 與 scripts/docs-screenshots.test.mjs 由紅轉綠即為驗收。
