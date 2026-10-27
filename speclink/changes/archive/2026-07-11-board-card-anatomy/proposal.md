## Why

看板上的變更卡與討論卡各自演化、無共用骨架：變更卡標題為粗體 sans 而討論卡為等寬 mono（兩者同為 kebab-case 可複製把手）；討論卡有 topic 描述而變更卡沒有任何內容摘要；討論卡建立者以全名 email 直出佔一整行、變更卡卻是頭像圓點；複製鈕在兩種卡上都被推至列右緣，與「緊跟標題文字」的位置規則（已於 desktop-ux-polish 落實於已轉出細列與討論抽屜標題）不一致。本變更出自已結論討論 spec-archive-drawer-ux 第 4 輪的統一卡片解剖學決議，目標使用者為透過桌面 app 看板檢視 SDD 工作狀態的開發者／PO／PM。

## What Changes

- 定一套看板全尺寸卡的三列骨架並套用於變更卡與討論卡：識別列（等寬標題＋緊跟標題文字的複製鈕＋右端狀態 chip 與建立者圓點）、描述列（一行截斷，無內容時缺席）、meta 列（變更卡＝進度條與完成數；討論卡＝輪數與建立時間）。
- 變更卡標題由粗體 sans 改等寬字型（與討論卡 slug 一致——同為 CLI 動詞把手）；複製鈕自列右緣移至標題文字正後方。
- 變更卡新增描述列：proposal 的 Why 首句一行截斷；資料由桌面 core 變更清單 payload 新增 whyExcerpt 欄位帶出，proposal 缺席或 Why 為空時描述列缺席。
- 討論卡建立者由全名直出收斂為頭像圓點＋hover tooltip 全名（與變更卡同款）；「N 輪」自卡身挪至卡底 meta 列、與建立時間並排；複製 slug 鈕改行內尾隨（slug 為多行折行，按鈕直接跟在最後字元後流動）。
- 明訂狀態 chip 規則：僅在所在位置無法表達狀態時出現——討論卡（討論欄一欄兩態）保留 chip，變更卡（所在欄即階段）維持無 chip。

相容性影響：桌面 app 內部變更清單 payload 屬 app 自有介面，whyExcerpt 為向後相容之新增欄位。speclink-core／speclink-cli 的 CLI 人眼與 --json 輸出一律不動，回歸對照不受影響。

## Non-Goals

- 已轉出細列與討論抽屜標題的複製鈕位置——desktop-ux-polish 已落實（其任務 4.4），不重做。
- 規格卡／封存卡的資訊與版面——active 變更 spec-archive-drawer 的範圍。
- 看板欄位判定、拖排順序（board-card-order）、封存流程與搜尋行為——一律不動。
- 變更卡不加建立時間與狀態 chip（討論未決定的不做）；描述列不做多行展開。
- 不新增引擎動詞、CLI 子指令或設定欄位；不動 crates/speclink-core 與 crates/speclink-cli。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增「看板卡片統一解剖學」requirement（三列骨架、等寬標題、複製鈕緊跟標題、變更卡描述列、建立者圓點、chip 規則）；「討論於看板第 0 欄兩級呈現」的全卡呈現細節同步改寫（建立者圓點化、輪數入 meta 列、複製鈕行內尾隨）。本 delta 的 MODIFIED 基底為 desktop-ux-polish 落地後的正典版本——封存順序上本變更 SHALL 排在 desktop-ux-polish 之後。

## Impact

- Affected specs: `desktop-app`
- Affected crate: speclink-desktop-core（apps/desktop/core，變更清單 payload 新增 whyExcerpt）；speclink-core 與 speclink-cli 不動
- Affected code:
  - Modified:
    - packages/ui/src/components/ChangeCard.tsx
    - packages/ui/src/components/DiscussionColumn.tsx
    - packages/ui/src/adapter.ts
    - packages/ui/src/__tests__/kanban.test.tsx
    - packages/ui/src/__tests__/discussionColumn.test.tsx
    - apps/desktop/core/src/query.rs
  - New: (none)
  - Removed: (none)
