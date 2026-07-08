## Why

變更抽屜的任務分頁有兩個互動缺陷：其一，缺少批次操作——對照 Spectra 的任務工具列（全部已完成／下一個未完成／重置任務），收尾或重跑一輪任務時只能逐一點勾；其二，手動勾選有可感知的卡頓——受控 checkbox 要等單發寫回、全量清單 refresh、抽屜五份文件重載整條鏈完成後才反映勾選，期間整個清單被指標鎖定。目標使用者是在桌面 app 追蹤與勾核 apply 進度的開發者／PO／PM；使用情境是變更抽屜任務分頁的日常勾選與批次收尾。源自討論 desktop-reading-and-tasks-ux（七項問題的任務互動刀）。

## What Changes

- 任務工具列：任務清單頂部新增三鍵——「全部已完成」「下一個未完成」（含 n 快捷鍵，捲動至第一個未完成任務並短暫高亮）「重置任務」（全部取消勾選）；全部任務已完成時前兩鍵不可用；唯讀封存檢視不顯示工具列。
- 批次寫回走新 IPC 動詞：桌面嵌入引擎新增批次設定全部任務完成狀態的函式與對應 Tauri 指令，一次讀檔一次寫回；開工章與 touched 語意沿用單發勾選（未開工變更首次完成任務時蓋章一次；重置不蓋章、不記 touched）；同狀態重跑冪等不改檔。前端 SpeclinkDataSource 介面同步擴充。
- 勾選樂觀更新：勾選瞬間 UI 立即翻轉、清單保持可互動（移除 busy 期間的整列指標鎖定），寫回失敗時回滾至磁碟現況並顯示單行錯誤；「重載統一由 refresh 世代驅動」的單一資料流與互動讓路機制保留。
- 無 CLI 影響：speclink-core／speclink-cli 兩個 crate 不動，CLI 人眼與 --json 輸出逐位元不變；變更落在 apps/desktop/core（嵌入引擎包裝）、apps/desktop/src-tauri（指令層）與 packages/ui。

## Non-Goals

- markdown 渲染與字體（desktop-reading-experience 刀）；規格 nav 頁（desktop-specs-view 刀）。
- CLI 端批次任務指令——GUI 專屬需求，CLI 使用者以編輯器直接改 tasks.md。
- 前端迴圈呼叫單發勾選實作批次——N 次寫檔＋N 次檔案監看 refresh，討論已否決。
- 任務拖放排序與自動重編號行為不動（既有規格照舊）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增任務互動需求——任務分頁批次操作工具列（單次寫回、開工章語意沿用）；勾選任務即時回饋（樂觀更新、失敗回滾）。

## Impact

- Affected specs: desktop-app
- Affected code:
  - Modified: apps/desktop/core/src/manage.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/App.tsx、packages/ui/src/adapter.ts、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/src/__tests__/App.test.tsx
  - New: （無）
  - Removed: （無）
- 新增依賴：（無）
