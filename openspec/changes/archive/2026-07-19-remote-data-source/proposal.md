## Why

workspace-session 立了 session 承重牆、connection-registry-keychain 給了憑證，但 remote locator 至今無法建構——沒有 RemoteDataSource，remote session 就是空殼。本刀讓 Desktop 第一次真正開出 remote workspace：看板、文件、任務、動詞與討論走 server，事件走 SSE、以 Polling＋ETag 收斂（roadmap Phase 3 gate 的兩條：capability 驅動停用、SSE 中斷以 Polling＋ETag 恢復）。

## What Changes

- speclink-remote 新增 SSE 事件消費模組：對 /events 訂閱（Authorization 三 headers 照舊）、解析 eventId（outbox seq）／scope／resource 提示與 reset 信號、支援 Last-Event-ID 續傳——目前 workspace 內沒有任何 client 端 SSE 消費者。
- src-tauri 新增 per-connection remote runtime：以刀 2 的 Keychain refresh credential 換發 access token（記憶體持有；401 時 refresh 一次重試一次，仍失敗即回報需重新認證的錯誤狀態）、逐請求建構 speclink-remote Client、binding handshake 成功後才建立 session（fail-closed）。
- src-tauri 新增 per-server event manager：同 connection 的 sessions 共用單條 SSE；收 invalidate 即向前端發帶 locator key 的事件；斷線或漏事件時以 /sync-state 的 ETag 收斂後帶 Last-Event-ID 續訂；收 reset 信號即觸發全量重載。push 只做 invalidate，重讀一律 Query。
- TS 新增 RemoteDataSource：實作 SpeclinkDataSource 介面、背後為帶 connection 與 repo 的 invoke 包裝；依覆蓋矩陣分三類——直達端點（清單、文件、任務勾選、claim、archive、討論全流程）、組合實作（setAllTasks 以逐任務寫回組合）、明確不支援（archived 瀏覽、searchWorkspace、validate/analyze、deleteChange、moveTask、reorderCard、正典 spec 內文——server 無端點）。
- capability 驅動停用：RemoteDataSource 附帶 capability 描述，UI 對不支援操作停用其affordance（按鈕停用附繁中說明、archived 頁與 spec 內文顯示 remote 不支援提示）；本地 session 全功能不變。
- remote locator 首個建構路徑：伺服器頁籤新增「開啟 workspace」極簡入口（輸入 repo 識別→handshake 驗證→開 remote 分頁）；完整 Workspace chooser 屬下一刀。
- remote 分頁以 cloud 狀態圖示與 Project/Repo 顯示（§10.5 的最小呈現面）。

## Capabilities

### New Capabilities

- `remote-workspace-data`: remote session 的資料面行為保證——handshake 後才建立、Query＋ETag 為重讀正典、SSE 只做 invalidate、斷線以 Polling＋ETag 收斂後續訂、capability 驅動停用、token 換發全程 Rust 側。

### Modified Capabilities

(none)

## Impact

- 相容性影響：本地 session 行為零改動；server 零改動（純消費端）——server 端點缺口（archived 瀏覽、search、正典 spec 內文、validate/analyze、專案清單）如實以停用呈現，補端點屬後續獨立刀。與活躍 change desktop-failure-toast 有 apps/desktop/src/store.ts 與 App.tsx 的潛在共檔，apply 前需確認平行 session 狀態。
- Affected specs: `remote-workspace-data`（新增）
- Affected code:
  - New: crates/speclink-remote/src/events.rs、crates/speclink-remote/tests/events_sse.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/src/event_manager.rs、apps/desktop/src-tauri/tests/{remote_runtime,remote_data,event_manager,common/mod}.rs、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/__tests__/{remoteDataSource,remoteOpen}.test.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - Modified: crates/speclink-remote/src/lib.rs（RemoteError 增 status 欄位——401/403 判別）、crates/speclink-remote/src/{client,device}.rs（status 欄位機械補齊）、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/session.ts、apps/desktop/src/store.ts、apps/desktop/src/main.tsx、apps/desktop/src/components/ServersPanel.tsx、apps/desktop/src/App.tsx、apps/desktop/src/components/ProjectTabs.tsx、packages/ui/src/components/{KanbanBoard,BoardSearchBar,RichDetailDrawer,TaskList}.tsx（capability 停用的附加性 optional props——既有介面與測試零修改）、Cargo.lock
  - Removed: 無
