## Why

LANGUAGE.md 自 2026-08-14 起以「品質關卡」為正式用詞、把「品質站」列為 avoid，但正式規格裡仍有 35 處「品質站」：5 份規格的 Purpose 段各 1 處，以及 9 份規格中 10 條需求的 30 處（其中 3 條需求的標題、2 個情境的名稱也帶著舊詞）。用詞守門（scripts/docs/vocabulary-guard.test.mjs）只掃 README、docs 與畫面文案，刻意不掃 openspec/specs，所以一直沒被抓到。正式規格是 AI agent 與人最常讀的文件，舊詞留在這裡會一再被學回去——討論 positioning-and-process-cost 期間 AI 的回覆與使用者的記憶檔都用了「品質站」。本變更是該討論分刀計畫的第 6 刀。

目標使用者與情境：閱讀正式規格的開發者與 AI agent（團隊情境下含 PO 與 PM），在 discuss、propose、verify 等會讀正式規格的階段。

## What Changes

- 以 MODIFIED delta 把 7 條需求內文的「品質站」改為「品質關卡」：client-protocol「變更清單的寫碼進度欄位」、config-skill「技能規定固定輸入來源與四條內容判準」、desktop-app「介面狀態語意色分層」與「看板卡片的驗證標示」、quality-skill「兩站時序的編排行為」、review-skill「審查技能的生成與正典化」、skill-routing「出口交棒由技能結尾承載」；行為不變。
- 兩個情境改名，以 REMOVED-SCENARIO 註解明示：config-skill「品質站正典不得重述進 rules」改為「品質關卡正典不得重述進 rules」，skill-routing「品質站落章後的交棒句」改為「品質關卡落章後的交棒句」。
- 三條標題帶舊詞的需求以 REMOVED＋ADDED 改名並改內文：desktop-app「品質站蓋章配色與主色分離」→「品質關卡蓋章配色與主色分離」、tray-status-menu「面板變更列的品質站章」→「面板變更列的品質關卡章」、user-documentation「品質站蓋章效果與非成員錯誤碼的文件揭露」→「品質關卡蓋章效果與非成員錯誤碼的文件揭露」；條款不變。
- 直接修改 change-diff-scope、client-protocol、desktop-app、server-verb-api、tray-status-menu 五份正式規格 Purpose 段裡的「品質站」（Purpose 不經 delta）。
- ui-copy-vocabulary 把「品質站」列為禁用詞的出現處保留。

## Non-Goals

- 不擴大用詞守門的掃描面到 openspec/specs：其他 avoid 詞（例如「正典」）在正式規格仍有大量存量，一併掃描會立刻全紅，需要另開變更處理。
- 不改「兩站」「站別」等不含「品質站」的說法，LANGUAGE.md 只把「品質站」列為 avoid。
- 不改已封存的變更與討論。
- 不改任何程式、技能或文件；畫面文案與 docs 已由先前的變更修正。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `client-protocol`: 「變更清單的寫碼進度欄位」用詞修正。
- `config-skill`: 「技能規定固定輸入來源與四條內容判準」用詞修正與情境改名。
- `desktop-app`: 「介面狀態語意色分層」「看板卡片的驗證標示」用詞修正；「品質站蓋章配色與主色分離」改名為「品質關卡蓋章配色與主色分離」。
- `quality-skill`: 「兩站時序的編排行為」用詞修正。
- `review-skill`: 「審查技能的生成與正典化」用詞修正。
- `skill-routing`: 「出口交棒由技能結尾承載」用詞修正與情境改名。
- `tray-status-menu`: 「面板變更列的品質站章」改名為「面板變更列的品質關卡章」。
- `user-documentation`: 「品質站蓋章效果與非成員錯誤碼的文件揭露」改名為「品質關卡蓋章效果與非成員錯誤碼的文件揭露」。

## Impact

- Affected specs: `client-protocol`, `config-skill`, `desktop-app`, `quality-skill`, `review-skill`, `skill-routing`, `tray-status-menu`, `user-documentation`
- Affected code:
  - Modified: openspec/specs/change-diff-scope/spec.md, openspec/specs/client-protocol/spec.md, openspec/specs/desktop-app/spec.md, openspec/specs/server-verb-api/spec.md, openspec/specs/tray-status-menu/spec.md
- 影響的 crate／app：沒有；只改正式規格文字。上列五份正式規格只改 Purpose 段，其餘改動經 delta 於封存時合併。
- 設定欄位：沒有。
- 生成的技能：沒有，不升 ASSET_VERSION。
- 相容性影響：沒有行為變動；正式規格中三條需求與兩個情境的名稱改變，引用舊名稱的已封存記錄保持原樣。
