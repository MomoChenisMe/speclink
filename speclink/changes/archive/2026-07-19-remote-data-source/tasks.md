## 1. SSE 消費模組（TDD）

- [x] 1.1 紅（design「決策 3：SSE 消費落 speclink-remote、event manager 落 src-tauri」的 client 半邊）：新增 crates/speclink-remote/tests/events_sse.rs——以 in-process speclink-server 覆蓋：訂閱後收到 invalidate 事件（id＝outbox seq、scope、resource）、Last-Event-ID 續傳補收錯過事件、server 端保留期外續傳收到 reset 信號、可中止的阻塞讀取正常收束。cargo test -p speclink-remote 確認全紅。 <!-- speclink-task:tsk_01KXQQ9BPWNQKJ7TGV85JW40CQ -->
- [x] 1.2 綠：新增 crates/speclink-remote/src/events.rs（SSE 逐行解析、typed 事件、Last-Event-ID 請求頭、中止把手）並於 crates/speclink-remote/src/lib.rs 匯出。1.1 全綠。 <!-- speclink-task:tsk_01KXQQ9BPWQW66R0C12GTFMWRH -->

## 2. remote runtime 與 token（TDD）

- [x] 2.1 紅（design「決策 4：token 生命週期與 401 語意」；規格「token 換發全程 Rust 側且 401 語意固定」）：src-tauri 測試（in-process server＋in-memory CredentialStore）——請求前自動換發、401→refresh 一次→重試一次成功、rotation 新 refresh credential 回寫、refresh 亦失效→needs-reauth 狀態且後續操作回拒絕錯誤。確認全紅。 <!-- speclink-task:tsk_01KXQQ9BPWEVEPRGGYJFW6YCTX -->
- [x] 2.2 綠：新增 apps/desktop/src-tauri/src/remote.rs——per-connection TokenManager 與逐請求建構 speclink-remote Client；needs-reauth 狀態暴露為 TS 可查的連線狀態。2.1 全綠。 <!-- speclink-task:tsk_01KXQQ9BPWCCE2MH5DJYJKSMDR -->

## 3. handshake 與資料面命令（TDD）

- [x] 3.1 紅（規格「handshake 成功後才建立 remote session」；design「決策 1：RemoteDataSource 的三類覆蓋矩陣」）：src-tauri 測試——remote_open 以 repo 識別 handshake：成功回 project/repo 顯示名與 capability 描述、403/404 原樣回錯不建 runtime；直達類逐方法對 in-process server 回真值（清單、get_change、artifact 內文、task done/undone、claim、archive、討論清單/內文/promote/archive）；組合類 setAllTasks 中途失敗中止並回報筆數；不支援類回拒絕錯誤。確認全紅。 <!-- speclink-task:tsk_01KXQQ9BPWD20TZSQXTGVMTJ29 -->
- [x] 3.2 綠：remote_* 命令實作三類矩陣（capability 描述物件隨 remote_open 回傳，內容依 design「決策 2：capability 描述隨 session 建立產生」）。3.1 全綠。 <!-- speclink-task:tsk_01KXQQ9BPW26FSD6EHWF1ZM8N8 -->

## 4. event manager 與收斂（TDD）

- [x] 4.1 紅（design「決策 5：斷線收斂程序（§9.2 的桌面實體）」；規格「斷線以 Polling 加 ETag 收斂後續訂」「Query 加 ETag 為重讀正典且 push 只做 invalidate」）：src-tauri 測試——invalidate 到達即發 remote-workspace-changed（payload＝locator key）；同 connection 兩 session 共用單一訂閱（計數斷言）；強制斷流後 /sync-state ETag 相異即發重載通知、以注入退避序列重連並 Last-Event-ID 續傳；reset 信號發全量重載通知後自新位點續訂。確認全紅。 <!-- speclink-task:tsk_01KXQQ9BPW4YYR8D44J0A87HMA -->
- [x] 4.2 綠：新增 apps/desktop/src-tauri/src/event_manager.rs（per-connection 訂閱執行緒、session 註冊分發、收斂程序、退避注入點）。4.1 全綠。 <!-- speclink-task:tsk_01KXQQ9BPWE2ZHSY48W5GDR87T -->

## 5. TS 接線與開啟入口

- [x] 5.1（design「決策 7：TS RemoteDataSource＝薄 invoke 包裝」）：新增 apps/desktop/src/adapter/remoteDataSource.ts（SpeclinkDataSource 全方法對 remote_* invoke 的映射、不支援方法回拒絕）與 apps/desktop/src/session.ts 的 createRemoteSession 工廠；新增 apps/desktop/src/__tests__/remoteDataSource.test.ts 斷言逐方法 invoke 參數映射（connectionId＋repo）與拒絕語意。全綠。 <!-- speclink-task:tsk_01KXQQ9BPWVTPCMZ48VSZ348R7 -->
- [x] 5.2（design「決策 6：極簡開啟入口與 handshake fail-closed」；規格「handshake 成功後才建立 remote session」）：apps/desktop/src/components/ServersPanel.tsx 已登入條目加「開啟 workspace」對話（repo 識別輸入，用自建 Input）；apps/desktop/src/store.ts 接 createRemoteSession 開分頁；apps/desktop/src/components/ProjectTabs.tsx 的 remote 分頁呈現 cloud 圖示與 Project/Repo 名；重啟恢復走 handshake、失敗呈現狀態不消失。vitest 假 adapter 覆蓋開啟成功/失敗兩路徑。全綠。 <!-- speclink-task:tsk_01KXQQ9BPWYPSVS0WWX1VQHTRX -->
- [x] 5.3（規格「capability 驅動停用且不偽造缺口」）：UI 消費 capability 描述——remote 分頁停用刪除/拖排/搜尋/validate/analyze 附繁中 tooltip，archived 頁與 spec 內文顯示尚未提供提示卡；本地分頁全功能不變（迴歸斷言）。vitest 假 adapter 覆蓋停用與本地不受影響。全綠。 <!-- speclink-task:tsk_01KXQQ9BPWV1FV3AJ1DTEX4GBK -->

## 6. 驗收

- [x] 6.1 GUI 鐵律手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev → 已登入連線開啟 remote workspace → 看板呈現 server 資料 → 以 CLI 對同 repo 建 change 數秒內反映 → 手動重啟 server 驗證收斂與續訂 → 不支援操作停用/提示如實 → 重啟 app remote 分頁恢復、本地分頁不受影響。 <!-- speclink-task:tsk_01KXQQ9BPWJQ4AHM30EPE69JMN -->
- [x] 6.2 回歸：cargo test -p speclink-remote、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠（重建前關閉執行中 exe）；apply 前確認 desktop-failure-toast 平行 session 的 store.ts／App.tsx 改動狀態，共檔依提交衛生拆分。 <!-- speclink-task:tsk_01KXQQ9BPW7FH4VBW04YPDB59H -->
