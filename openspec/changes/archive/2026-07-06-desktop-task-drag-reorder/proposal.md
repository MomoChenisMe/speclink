## Why

桌面詳情抽屜的任務分頁目前以上下箭頭按鈕逐格搬移任務，且搬移後任務文字裡的編號前綴（1.1、1.2、2.1…）維持舊值——順序與編號立即脫節，使用者必須自己心算「第 3 列其實是 1.5」。Spectra 桌面版的同一互動是拖放：每列左側有 ⠿ 把手、拖曳到目標位置放開即完成排序。本刀把任務排序改為拖放手勢，並在搬移寫回時自動重寫受影響任務的編號前綴，讓 tasks.md 的文字編號永遠與實際順序一致。目標使用者：在桌面 app 內整理任務順序的開發者與 PO/PM。

**修訂（2026-07-06，首輪實作後使用者實測回報）**：(1) 拖曳中的讓位動畫只位移任務列、群組標題靜態不動——把 1.6 拖向群組 2 時，2.1 的讓位視覺會穿越「## 2」標題跑進群組 1 區域，預覽與放開後的實際結果不一致，使用者誤判為交換；(2) ordinal 落點模型表達不出「放到群組開頭」——組界兩義槽位（群組 1 尾 vs 群組 2 首）無法區分。本修訂把群組標題納入讓位序列並使其可作為「組首」落點，moveTask 增加可選側別參數消解兩義。

## What Changes

- **拖放排序**：任務列左側新增 ⠿ 拖曳把手（僅把手可觸發拖曳，點擊核取方塊與文字不受影響），拖曳中以 DragOverlay 呈現浮起列與落點指示，放開後回寫 tasks.md；上下箭頭按鈕移除。封存唯讀檢視不提供把手（維持不可互動）。
- **自動重編號**：搬移寫回時重算任務編號前綴——文字以「數字.數字」開頭的任務行，前綴重寫為「所屬群組編號.組內序」；不符該樣式的任務文字與群組標題（## N. …）原樣保留。跨群組搬移的任務取得新群組的編號。
- **組界修正（修訂新增）**：群組標題以不可拖項加入讓位序列——拖曳中任務的讓位視覺不再穿越群組標題；把手放到群組標題上＝該任務成為該群組的第一個任務。
- **介面收斂（修訂更新）**：TaskList 的逐格 onMove（上移/下移）回呼改為一次到位的 onReorder(from, to, before?)；SpeclinkDataSource.moveTask 增加**可選**第四參數 before（省略時維持既有方向推斷行為——既有呼叫端零改動、向後相容），引擎與 CLI 零變更。

## Non-Goals

- 群組（## 標題）本身的拖放排序與重編號——僅任務列可拖。
- 拖入空群組（標題下沒有任何任務的群組）的落點支援——已知限制，屆時另案。
- 跨 change 的任務搬移；看板卡片拖曳行為（既有功能，不動）。
- 引擎/CLI 的任務動詞變更——speclink task done 以 ordinal 定址，與文字編號無關。
- 無「數字.數字」前綴的任務自動補編號（樣式猜測風險，不做）。
- 行動裝置觸控最佳化。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增一項需求——任務清單拖放排序與自動重編號（把手觸發拖曳、寫回時重算編號前綴、群組標題可作組首落點且讓位不穿越標題、唯讀檢視無把手）。既有需求不變。

## Impact

- Affected specs: 修改 `desktop-app`（ADDED 一條 Requirement）。
- Affected code:
  - Modified: packages/ui/src/components/TaskList.tsx（chevron 改 ⠿ 把手＋dnd-kit sortable；修訂：標題入讓位序列與組首落點）、packages/ui/src/components/RichDetailDrawer.tsx（onMove 逐格改 onReorder 一次到位＋側別轉發）、packages/ui/src/adapter.ts（moveTask 可選 before 參數）、packages/ui/package.json（加 @dnd-kit/sortable）、apps/desktop/core/src/manage.rs（move_task_at 搬移後重編號＋可選側別）、apps/desktop/src-tauri/src/lib.rs（move_task command 可選參數）、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/App.tsx（側別傳遞）、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、apps/desktop/src/__tests__/App.test.tsx（如受牽動）
  - New: （無）
  - Removed: （無——上下箭頭按鈕於 TaskList.tsx 檔內移除，不刪檔）
- 相依：@dnd-kit/sortable（與既有 @dnd-kit/core 同族；KanbanBoard 已用 core）。
- 相容性：tasks.md 僅在使用者主動拖放時被重寫編號；SpeclinkDataSource.moveTask 僅**新增可選參數**（既有簽名呼叫不受影響），web/remote 端零影響。
