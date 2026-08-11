## 1. async 化清單與文件讀取 command

- [x] 1.1 apps/desktop/src-tauri/src/lib.rs：將 list_changes、list_specs、status、document、spec_document、search_workspace、change_capabilities、change_meta 改為 async fn＋tauri::async_runtime::spawn_blocking 委派，落實 spec「觸及檔案系統的 command 不佔用主執行緒」；樣板照既有寫入側（set_task_done 等，design D2）：閉包 move 收 root 與參數、錯誤訊息帶 command 語境。回傳型別與 serde 形狀不變。 <!-- speclink-task:tsk_01KZQ8M58N87551Y0QC5BHW48M -->
- [x] 1.2 apps/desktop/src-tauri/src/lib.rs：將 archived_changes、archived_document、archived_capabilities、list_discussions、discussion_document 同樣改為 async＋spawn_blocking。回傳型別與 serde 形狀不變。 <!-- speclink-task:tsk_01KZQ8M58NVZ3YM8QACDKF9EQ0 -->

## 2. async 化動詞與專案管理 command

- [x] 2.1 apps/desktop/src-tauri/src/lib.rs：將 validate、analyze、archive、archive_carry、discard_review、discard_verify、delete_change、promote_discussion、archive_discussion 改為 async＋spawn_blocking（這批含目錄搬移與 git spawn，凍結風險最高）。 <!-- speclink-task:tsk_01KZQ8M58NRCDX3RVZHQK7GZJV -->
- [x] 2.2 apps/desktop/src-tauri/src/lib.rs：將 open_project、init_project、adopt_project、project_stats、probe_instructions、read_settings、write_app_tools、write_workflow_config、write_workflow_content 改為 async＋spawn_blocking。 <!-- speclink-task:tsk_01KZQ8M58PP19N04KB15AFP57Q -->
- [x] 2.3 apps/desktop/src-tauri/src/lib.rs：將 connection_list、connection_add、inspect_checkout、bind_checkout、remote_watch 改為 async＋spawn_blocking（讀寫 appConfigDir 下 connections.json 與 checkout 檢查皆為檔案 IO；remote_watch 的 connectionId → origin 查找讀的正是同一份 registry，Non-Goals 原述「remote_* 已全數 async」對此支不成立）。 <!-- speclink-task:tsk_01KZQ8M58PN3XMYKV93K0EQ21X -->
- [x] 2.4 apps/desktop/src-tauri/src/lib.rs：watch_workspace 的監看重掛（rearm 遞迴掛載與 git 身分預熱觸發）移至 spawn_blocking——掛載大樹屬 IO；維持既有降級語意（監看不可用僅記錄、不回錯）。若 WatchSlot 內含非 Send 成員導致無法跨執行緒，改為在 command 內以既有 WatcherState Mutex 於 blocking 閉包中取鎖，行為不變。 <!-- speclink-task:tsk_01KZQ8M58PPJTKSBTTNM75BG9Y -->
- [x] 2.5 apps/desktop/src-tauri/src/lib.rs：於模組層註解記錄同步保留名單與理由——startup_dir（讀行程環境）、connection_state（讀記憶體健康狀態）、remote_unwatch（只動記憶體中的訂閱表）、toggle_tray_panel、quit_app、tray recovery 轉發（純視窗／純記憶體操作，不觸檔案系統）。 <!-- speclink-task:tsk_01KZQ8M58PRXMGXCT3JY6RZXMV -->

## 3. 驗證

- [x] 3.1 全量編譯與測試：cargo clippy -p speclink-desktop 零新警告；cargo test -p speclink-desktop-core 與 speclink-desktop 既有測試全綠（測試前依慣例確認 sidecar 與 server-web dist 已備妥）。 <!-- speclink-task:tsk_01KZQ8M58PZEZXNDF56ZJQA7Z6 -->
- [x] 3.2 前端行為驗證：npm test（desktop 面 vitest）全綠——command 名稱、參數、回傳形狀不變，前端零改動即通過。 <!-- speclink-task:tsk_01KZQ8M58PS1ZFD97QN0B8WS33 -->
- [ ] 3.3 [M] 手動驗證卡死消失：讓 agent 於同一 workspace 高頻操作（跑動詞、寫檔）時，自主視窗分頁與 tray 面板兩條入口切換 workspace——視窗與 tray 圖示全程可互動，資料延後出現但不凍結。 <!-- speclink-task:tsk_01KZQ8M58PQME1R6TPC9PR66Z5 -->
