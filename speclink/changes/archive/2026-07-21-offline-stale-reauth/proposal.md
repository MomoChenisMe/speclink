## Why

remote workspace 的好天氣路徑已全部落地，但壞天氣仍是毛邊：server 不可達時查詢報錯成 toast、看板可能清空；needs-reauth 只是 runtime 裡的狀態字串，使用者沒有重新登入的入口、更沒有原地恢復；event manager 的退避 worker 沒有任何 connection 狀態信號發往前端。架構 §10.5／§13.4 與 roadmap Phase 3 gate「stale/offline snapshot 只能讀，不可產生隱性 local write queue」都是本刀的直接依據——這是 Phase 3 收官（migration＋e2e）前最後一塊功能面。

## What Changes

- per-connection 離線狀態機（Rust 側單一真相）：查詢連續失敗達閾值、或 SSE worker 持續退避且 sync-state 亦失敗 → offline；worker 恢復成功 → online。狀態轉換以 Tauri 事件廣播（connectionId、狀態、訊息），TS 據此翻 session 呈現。
- remote 分頁的明確狀態呈現：offline 與 needs-reauth 各自的分頁層級橫幅與 cloud 狀態圖示；最後成功載入的看板／文件內容保留可讀並標示 stale——reload 失敗 SHALL 不清空既有內容（顯式保障，取代現行可能清空的路徑）。
- 寫入即拒、絕不排隊：offline 或 needs-reauth 期間全部寫入操作（任務勾選、動詞、artifact 寫回、policy 儲存）UI 停用（重用 capability 停用管線疊加 offline mask）且 Rust 端命令立即拒絕；恢復後 server 端不得存在任何離線期間的寫入痕跡。
- 恢復編排：worker 收斂成功後發 online 與全量失效通知 → store 全量重查、清除 stale 標示——重用既有 Polling＋ETag 收斂管線，不新造機制。
- 重新認證 UX：橫幅帶「重新登入」入口 → 走既有 device login／PAT 流程 → install_token 復原 needs-reauth → 自動 re-handshake、全量重查、event worker 重啟——session 與分頁原地恢復，全程不消失、不退回 local mode。
- destructive 一致化檢核：remote 分頁的 archive 確認對話沿用同一路徑，措辭補充影響 server scope；deleteChange 維持停用不變。

## Capabilities

### New Capabilities

- `remote-resilience`: remote workspace 壞天氣行為保證——離線狀態機與明確呈現、最後 snapshot 唯讀與 stale 標示、寫入即拒無隱性佇列、恢復自動收斂清 stale、重新認證原地復活不退 local。

### Modified Capabilities

(none)

## Impact

- 相容性影響：好天氣路徑行為不變（狀態機只在失敗達閾值後介入）；本地 session 完全不受影響；server 零改動。
- Affected specs: `remote-resilience`（新增）
- Affected code:
  - New: apps/desktop/src/__tests__/remoteResilience.test.tsx
  - Modified: apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/src/event_manager.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/tests/event_manager.rs、apps/desktop/src-tauri/tests/remote_runtime.rs、apps/desktop/src/adapter/connections.ts、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/session.ts、apps/desktop/src/store.ts、apps/desktop/src/App.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx、apps/desktop/src/__tests__/store.test.ts
  - Removed: 無
