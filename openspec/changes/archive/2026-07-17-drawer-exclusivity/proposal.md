## Why

桌面 app 的四種 detail 抽屜（變更詳情、規格、封存、討論）各由獨立狀態欄位驅動，開啟時互不相斥——先從系統匣開啟討論抽屜、再開啟變更詳情抽屜時，兩層抽屜疊加顯示。使用者期望同時只能開著一個抽屜。既有的抽屜內跳轉已手動「先關再開」，證明互斥本是預期語意，但系統匣選單與面板等入口直呼開啟動作、繞過了這層逐點補丁。（源自討論 drawer-exclusivity 的結論，2026-07-17。）

目標使用者：透過桌面 app 檢視看板、規格、封存與討論的開發者。使用情境：任何開啟 detail 抽屜的入口——看板卡片、規格清單、封存頁、系統匣選單與面板、抽屜內互相跳轉。

## What Changes

- 桌面 app 狀態層建立「同時僅一個 detail 抽屜開啟」不變量：四個開啟動作（`openDetail`、`openDiscussion`、`openSpec`、`openArchived`）設定自身欄位時同時清除另外三個 detail 欄位——後開者取代先開者，而非拒開。
- `openDetail` 既有的動詞結果清理（`drawerVerb` 歸零）保留不變，互斥清理疊加其上；取代變更詳情抽屜的另外三個開啟動作亦一併清 `drawerVerb`——比照關閉變更詳情抽屜，不留上一個 change 的動詞結果殘餘。
- App 殼層兩處抽屜內跳轉的手動「先關再開」移除——不變量生效後成為冗餘，留著會誤導未來入口照抄。
- 無 CLI 指令、輸出、設定欄位或技能變動。

## Non-Goals

- 不逐入口（系統匣、看板、面板）補關閉呼叫——per-callsite 補丁已被現況證偽：抽屜內跳轉補了、系統匣漏了，未來新入口仍會漏。
- 不只修「討論＋變更」的組合——規格與封存抽屜病因相同，僅修一組之後會再報。
- 不動抽屜元件本身——packages/ui 的四個 Drawer 元件介面與呈現不變，互斥完全由狀態層保證。
- 不引入「拒開」語意——第二個抽屜開啟時不是被擋下，而是取代先開者。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增 detail 抽屜互斥 requirement——任一 detail 抽屜開啟中再開啟另一種抽屜時，先開者關閉、同時僅一個抽屜可見，涵蓋所有開啟入口。

## Impact

- Affected specs: `desktop-app`（新增抽屜互斥 requirement 的 delta）
- Affected code:
  - Modified: `apps/desktop/src/store.ts`（四個開啟動作加入互斥清理）、`apps/desktop/src/App.tsx`（移除兩處手動先關再開）、`apps/desktop/src/__tests__/store.test.ts`（互斥不變量測試）
  - New: (none)
  - Removed: (none)
- 影響的 crate：無——`speclink-core` 與 `speclink-cli` 不動，變更侷限於桌面 app 前端狀態層。
- 相容性影響：無——CLI 人眼與 `--json` 輸出皆不變，回歸對照不受影響。
