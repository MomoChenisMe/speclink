## Why

桌面 app 的看板有兩個 Spectra parity 缺口與一個狀態架構缺陷：(1) 清單資料只在啟動與 app 內操作後刷新，外部寫者（CLI、agent、人手改 tasks.md——SDD 的常態）的變更不反映，造成任務數長期顯示 0/0；(2) 已封存變更只有名稱列，無法像 Spectra 一樣展開檢視文件；(3) 引擎的 in-progress 標記存於 .git/speclink-app/ 下的 SQLite——host-local、不隨 repo 走（換機即失聯，Spectra 既有缺陷）、且全 repo 零讀取者，看板「進行中」欄只能以任務數近似，剛 propose 完的 change 被錯置為進行中。目標使用者：透過桌面 GUI 追蹤 SDD 進度的開發者與 PO/PM，情境涵蓋 workflow 全階段的看板觀察與封存回顧。本案來源：討論「桌面即時刷新與封存瀏覽」（2026-07-06 結論，第一刀）。

## What Changes

- **A 即時刷新**：Tauri 層以檔案監看覆蓋整個 openspec/ 樹（specs、changes 含 archive、discussions），事件經 debounce 後通知前端整批 refresh；雙向收斂——外部寫者與 app 內操作都反映到畫面，既有的操作後主動 refresh 保留。監看的 wiring 留在宿主層（apps/desktop），SpeclinkDataSource 介面不加訂閱方法。
- **B 封存瀏覽**：儲存介面新增封存 artifact 讀取（既有封存 meta 讀取的對稱擴充）；已封存頁的列可展開為唯讀檢視（提案／設計／任務／規格分頁），列上顯示任務數徽章；封存清單的 SQLite 衍生快取升版（1→2）加入任務計數欄位，舊版快取自動重建。
- **C in-progress 標記真相遷入 change meta**：change 目錄的 .openspec.yaml 新增 started_at、started_by、started_with 三欄位，補齊 created → started → archived 三站生命週期歸屬；speclink in-progress add 改為寫入 meta（stdout 與 exit code 形狀不變），重複標記冪等（保留首次開工蓋章）；.git/speclink-app/ 的 SQLite 寫入端退役、不做既有標記遷移（該庫從未被任何讀取者消費）；歸檔時 started_* 欄位隨 meta 保留不剝除。
- **D 看板欄位標記驅動**：無 started＝提案中（就算任務已就位也留在此欄）、有 started 且未全完成＝進行中、任務全完成＝已就緒；詳情抽屜顯示「誰於何時開工」。桌面清單所需的標記欄位由桌面 core 疊加提供，speclink list --json 的 CLI 輸出維持位元級不變（parity 紅線）。

## Non-Goals

- 封存列的復原（unarchive）與刪除操作（Spectra 有，此次不做）。
- 討論看板（本討論扇出的第二刀 desktop-discussion-board，另案）。
- 側欄「規格」「備忘」內容頁；備忘的內容定位。
- 逐任務歸屬、狀態變更歷史、event log——fs 模式的完整稽核由 git history 承擔，meta 只記三站里程碑。
- 任何 web／remote 端實作（僅在既有骨架落現況註記）；SpeclinkDataSource 的訂閱介面。
- CLI 人眼與 --json 輸出的任何欄位變更。

## Capabilities

### New Capabilities

- `change-lifecycle`: change 的三站生命週期歸屬（created／started／archived）以 change meta 為真相——in-progress 標記的寫入行為、冪等性、歸檔保留，與 CLI 指令輸出的不變性。

### Modified Capabilities

- `desktop-app`: 新增三項需求——外部變更即時反映、已封存變更展開檢視、看板欄位由生命週期標記驅動。既有需求（直嵌引擎、啟動語境與空狀態）不變。

## Impact

- Affected specs: 新增 `change-lifecycle`；修改 `desktop-app`；`store-abstraction` 的既有 parity 需求為約束但不變更（CLI 行為位元級不變）。
- Affected crates:
  - `speclink-core`：inprogress 模組真相改寫（meta 而非 SQLite）、Store trait 封存 artifact 讀取擴充。
  - `speclink-fs`：FsStore 實作新 trait 方法。
  - `speclink-desktop-core`（apps/desktop/core）：清單疊加標記欄位、封存文件查詢、快取升版。
  - `speclink-desktop`（apps/desktop/src-tauri）：檔案監看與事件發送、新查詢 command。
  - `speclink-cli`：僅 in-progress add 的呼叫點跟隨 core 函式簽名調整（改傳儲存介面），輸出與行為零變更。
- Affected code:
  - Modified: crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/store.rs、crates/speclink-core/src/model.rs、crates/speclink-core/src/archive.rs、crates/speclink-fs/src/lib.rs、crates/speclink-cli/src/commands.rs、apps/desktop/core/src/query.rs、apps/desktop/core/src/cache.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/Cargo.toml、apps/desktop/src/App.tsx、apps/desktop/src/adapter/tauriDataSource.ts、packages/ui/src/adapter.ts、packages/ui/src/stage.ts、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/RichDetailDrawer.tsx
  - New: apps/desktop/src-tauri/src/watch.rs
  - Removed: （無——inprogress.rs 的 SQLite 機制於檔內退役，不刪檔）
- 相容性影響：speclink in-progress add 的 stdout、stderr 與 exit code 不變；**檔案效果變更**——不再建立 .git/speclink-app/（speclink.db、.migrate.lock、.migrated），改寫 change 目錄的 .openspec.yaml；既有機器上的舊 speclink.db 留置無害、無遷移（從未有讀取者）。speclink list --json 與其他指令輸出位元級不變，parity／color 對照不受影響。
- 設定欄位：不涉及 .speclink.yaml 與 openspec/config.yaml；新增的是 change meta（.openspec.yaml）的 started_at／started_by／started_with 三欄位，無預設值（未開工即缺席）。
- 技能與注入區塊：無影響。
