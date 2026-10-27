## Why

看板 active 卡片已達 9 張（變更 6＋討論 3）且會持續成長，靠掃視定位特定卡片開始吃力；已封存頁已有搜尋過濾，看板缺同等能力。此需求源自討論「專案選擇對齊-spectra」結論的延後項目（註明基建現成、優先）。目標使用者：透過桌面 GUI 追蹤 SDD 流程的開發者與 PO/PM；使用情境：discuss／propose／apply 全程的看板掃視與卡片定位。

## What Changes

- 看板上方新增搜尋列：輸入即時過濾看板全部欄位的卡片——變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；比對規則與已封存頁一致（去頭尾空白、不分大小寫、子字串）。
- 各欄欄頭計數隨過濾結果更新；清空輸入即還原全量呈現。
- 搜尋字串存於桌面 app 的 UI 狀態（與已封存頁的搜尋字串各自獨立），不持久化、不跨啟動保留。

## Non-Goals

- 進階搜尋語法（欄位限定、regex、模糊比對）——子字串比對已滿足十數張卡的定位需求。
- 已封存頁的搜尋（已存在，不動）與詳情抽屜內的內容搜尋。
- 搜尋歷史、持久化或跨啟動保留。
- 既有 UI 字串的 i18n 補抽——desktop-config-multiproject 已完成全面抽 key；本刀新增的搜尋 placeholder 直接走既有 i18n 字典（kanban.searchPlaceholder，zh-TW／en），不另擴大 i18n 範圍。
- CLI 與引擎行為的任何變更——純前端過濾，speclink-core 與 speclink-cli 不動，人眼與 --json 輸出皆不變，回歸對照不受影響。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 看板新增「搜尋過濾卡片」需求——輸入過濾變更卡與討論卡、欄頭計數隨過濾更新、清空即還原。

## Impact

- Affected specs: `desktop-app`（MODIFIED——新增看板搜尋需求）
- Affected crates: 無——speclink-core／speclink-cli 不動，變更僅及 packages/ui 與 apps/desktop 前端。
- Affected code:
  - Modified: packages/ui/src/components/KanbanBoard.tsx（搜尋輸入與過濾）、packages/ui/src/components/ArchivedList.tsx（改用共用比對函式，行為不變）、apps/desktop/src/store.ts（看板搜尋字串狀態）、apps/desktop/src/App.tsx（接線）、packages/ui/src/__tests__/kanban.test.tsx、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/App.test.tsx
  - New: （無）
  - Removed: （無）
- 相容性影響：無——CLI 人眼與 --json 輸出不變；桌面既有互動（卡片點擊、拖曳封存）不受搜尋列影響。
