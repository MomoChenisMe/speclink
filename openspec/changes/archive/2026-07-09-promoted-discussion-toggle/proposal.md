## Why

桌面看板「討論」欄目前把已轉出（promoted）討論以預設展開的欄底群組呈現，佔用欄體空間、與進行中的討論混雜、視覺複雜；且階段 chip 統一灰底不分狀態、欄計數把 promoted 也算入。使用者要把「討論中」與「已轉換成變更」的討論分離、降低視覺複雜度。

## What Changes

- 討論欄 header 新增「顯示已轉出」開關（↗ 圖示＋promoted 計數）：promoted 討論預設隱藏、關閉時零佔位；無任何 promoted 討論時開關缺席。
- 開關開啟時，promoted 討論於欄底「已轉出變更的討論」群組以衍生樹細列呈現（維持唯讀）。
- 討論欄計數徽章只計 active（open＋concluded）；promoted 計數移到 header 開關上。
- 階段 chip 改用看板階段配色（提案中／進行中／已就緒對應各階段欄的 teal 濃度、已封存中性色、已刪除 destructive 加刪除線），取代現行統一灰底。
- 空狀態：無 active 討論但存在 promoted 討論時，欄體不顯示「尚無討論」（由 header 開關傳達）。

## Non-Goals

見 design.md 的 Goals/Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`：「討論於看板第 0 欄兩級呈現」需求——promoted 呈現由預設展開的欄底群組改為 header 開關（預設隱藏、零佔位）、計數只算 active、chip 沿看板階段配色、無 active 但有 promoted 時空狀態調整。

## Impact

- Affected specs: desktop-app（modified）
- Affected code:
  - Modified:
    - packages/ui/src/components/DiscussionColumn.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/discussionColumn.test.tsx
  - New: (none)
  - Removed: (none)
