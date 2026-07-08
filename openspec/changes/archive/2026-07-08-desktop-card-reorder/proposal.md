## Why

桌面看板的卡片順序目前是計算值（變更卡依最後修改時間、討論卡依 slug），使用者無法表達「哪張卡優先」——任何一次編輯都會讓卡片跳位，優先序無處安放。目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM：他們在桌面看板上綜覽 discuss → propose → apply → archive 全流程的卡片，需要以拖放整理各欄的工作優先序，且這份順序要隨 repo 跨機器同步（源自已結論討論「本地看板變更卡手動排序」）。

## What Changes

- 看板四欄（討論／提案中／進行中／已就緒）各自支援**欄內**手動拖排。
- 順序以**稀疏 rank**（fractional ranking）存於卡片自身的 meta：變更卡寫入 change 目錄的 .openspec.yaml、討論卡寫入討論記錄的 frontmatter——文字檔進 repo，git 可合併、跨機同步。
- 一次拖排只改被拖那張卡的檔案（rank＝前後鄰居中點），兩機各拖不同卡時 git 自動合併零衝突。
- 無 rank 的卡（新建或既有存量）落於**欄頂**，彼此間維持現行排序作回退。
- 變更卡**跨欄放開彈回**：欄位（提案中／進行中／已就緒）由任務完成度推導，拖曳不改變階段；拖到封存落點的既有行為原樣保留。討論卡僅於討論欄內排序。
- 桌面新增一支 reorder command：前端拖放結束 → 計算中點 rank → 寫回該卡 meta → 刷新。
- speclink-core 的 meta 讀寫對 rank 欄位**讀取並原樣保留**（沿 change-lifecycle「既有欄位原樣保留」語意）；影響 crate：`speclink-core`（rank 欄位讀取與寫回保留）與桌面內嵌 core（apps/desktop/core 的排序與寫回）。`speclink-cli` 不變。

**相容性影響**：CLI 人眼與 `--json` 輸出皆不變——rank 不進任何 CLI 輸出、`speclink list` 排序不變，回歸對照（parity/color/twin）不受影響。桌面 list payload 僅項目**順序**改為 rank 優先（缺值回退修改時間序），欄位形狀不變。既有 meta 寫入路徑（開工標記、轉為變更等）遇 rank 欄位原樣保留，無遷移需求——舊 repo 無 rank 即全數回退現行排序，行為不變。

## Non-Goals

- 跨欄拖曳改變變更階段——階段是任務完成度的推導值，本變更不引入手動階段覆寫。
- 排序模式選單（名稱／建立／修改時間切換）——使用者要的是手動序，非排序規則替換。
- SQLite 或任何資料庫作為順序真相、db 檔進 repo、app 本機持久化——討論中已明文否決（二進位不可合併／跨機不同步）。
- 儲存層整體重構——本變更只加一個 rank 欄位，不動 Store 抽象。
- CLI 新增 reorder 子指令或改動任何既有 CLI 輸出。
- web 變體（web-role-views／web-agent-channel／web-server-postgres）的看板排序——server 端另案處理。
- 任務列拖排——已存在（move_task），非本題。

## Capabilities

### New Capabilities

- `board-card-order`: 看板卡片手動排序——rank 欄位的儲存契約（稀疏 rank、中點計算、缺值落欄頂回退、寫回保留）與四欄欄內拖排、跨欄彈回的互動行為。

### Modified Capabilities

(none)

## Impact

- Affected specs: 新增 `board-card-order`；`desktop-app`／`change-lifecycle`／`discussion-docs` 既有需求不變（rank 保留由 change-lifecycle 既有「既有欄位原樣保留」語意涵蓋）。
- Affected code:
  - New:
    - apps/desktop/core/src/rank.rs（字串 fractional key 的中點與批次派發演算，design D1／D4 定案）
  - Modified:
    - crates/speclink-core/src/model.rs（change meta 的 rank 欄位讀取與寫回保留）
    - crates/speclink-core/src/discuss.rs（討論 frontmatter 的 rank 讀取）
    - apps/desktop/core/src/query.rs（變更與討論清單依 rank 排序、缺值回退現行序）
    - apps/desktop/core/src/manage.rs（reorder 寫回與整欄補章）
    - apps/desktop/src-tauri/src/lib.rs（reorder command 註冊）
    - apps/desktop/src/adapter/tauriDataSource.ts（data source 新方法）
    - apps/desktop/src/store.ts（reorder 動作）
    - packages/ui/src/adapter.ts（SpeclinkDataSource 介面擴充）
    - packages/ui/src/components/KanbanBoard.tsx（三欄變更卡欄內 sortable）
    - packages/ui/src/components/DiscussionColumn.tsx（討論欄 sortable）
    - packages/ui/src/__tests__/kanban.test.tsx（欄內拖排測試）
    - packages/ui/src/__tests__/discussionColumn.test.tsx（討論欄拖排測試）
  - Removed: 無
