## Problem

切換 workspace（主視窗分頁或 tray 面板）後，desktop 要重新讀取整批 workspace 資料（changes／specs／discussions／已封存清單與文件）才能刷新頁面。此時若有 agent 正在同一個 repo 上操作（跑 CLI 動詞、git 操作、大量寫檔），整個 desktop UI 會卡死沒回應——主視窗凍結、tray 圖示點了沒反應、面板打不開，直到讀取全部跑完才恢復。

## Root Cause

Tauri 的非 async command 在 app 唯一的主執行緒上執行。寫入型 command 已依 design D2 全面 async＋spawn_blocking（apps/desktop/src-tauri/src/lib.rs 註解明寫「非 async command 會佔用主執行緒凍結整窗」），但讀取型與多數動詞 command 仍是同步 fn：list_changes、list_specs、status、document、spec_document、search_workspace、change_capabilities、change_meta、delete_change、validate、analyze、archive、archive_carry、discard_review、discard_verify、archived_changes、archived_document、archived_capabilities、list_discussions、discussion_document、promote_discussion、archive_discussion、open_project、init_project、adopt_project、project_stats、probe_instructions、watch_workspace、read_settings、write_app_tools、write_workflow_config、write_workflow_content、connection_list、connection_add、inspect_checkout、bind_checkout、remote_watch。

這些 command 不只讀檔案：清單讀取每次呼叫都會經 worktree 觀察面 spawn 一次 git（apps/desktop/core/src/query.rs 的 facts_for → crates/speclink-host/src/worktree.rs 的 git worktree list --porcelain），GUI 進程 spawn git 首抓可能秒級（macOS Gatekeeper 掃描稅）。切換 workspace 時前端一次發出整批同步讀取，全部排隊佔用主執行緒；agent 同時操作使 repo 忙碌（git 更慢），檔案變動又經監看觸發 workspace-changed 刷新，前端再發更多同步讀取——主執行緒被塞爆，原生事件迴圈（視窗事件、tray 圖示點擊、面板開閉）全部停擺，整窗凍結。tray 面板無獨立資料路徑（薄渲染層，動作以事件回流主視窗執行），凍結同時波及所有 surface。

（歸因出自討論 desktop-cli-multi-workspace-concurrency Round 3-4：檔案鎖／寫寫競態已被排除——desktop 與 CLI 之間不存在可互卡的共享鎖，POSIX 下讀取也不會被另一 process 的寫入 block。）

## Proposed Solution

凡會觸及檔案系統或 spawn 子進程（git 等）的 Tauri command，一律改為 async fn＋tauri::async_runtime::spawn_blocking 委派——與寫入側 design D2 完全同款：主執行緒不再執行任何檔案 IO 或子進程等待。改動全部落在 apps/desktop/src-tauri/src/lib.rs 薄包裝層，逐支機械式改寫；speclink-desktop-core 零改動。委派閉包沿用既有「逐呼叫收 root、無可變全域」慣例（workspace-session 決策 4）。

純記憶體或純視窗操作的 command 維持同步，不在改動範圍：startup_dir（讀行程環境）、connection_state（讀記憶體健康狀態）、toggle_tray_panel、quit_app、tray recovery 轉發。

前端契約零影響：command 名稱、參數、回傳形狀完全不變（前端 invoke 本就 await Promise，Rust 側同步改 async 對 TS 側不可見）。

## Non-Goals

- 不做讀取提速：worktree 觀察面「每次現取、不快取」（D4）維持不變。async 解「卡死」不解「慢」——agent 忙碌時清單可以晚到，但 UI 保持可回應。
- 不做 watcher 事件節流：async 化後刷新風暴只是 CPU 開銷問題，非凍結源。
- 不動 openspec/ 檔案樹的併發控制：跨 process 檔案鎖經討論刻意延後；引擎原子寫為獨立 change（atomic-file-writes）。
- 不動遠端模式的資料面語意與併發控制：remote_* 資料面已全數 async，遠端併發控制（CAS＋advisory lock＋If-Match）已完備。唯一例外是 `remote_watch`——它的 connectionId → origin 查找要讀 connections.json，屬本 change 的檔案 IO 範圍，一併 async 化（`remote_unwatch` 只動記憶體中的訂閱表，維持同步）。

## Success Criteria

- apps/desktop/src-tauri/src/lib.rs 中所有觸及檔案系統或 spawn 子進程的 command 皆為 async＋spawn_blocking；grep 該檔的同步 command 僅剩純記憶體／純視窗操作名單（startup_dir、connection_state、remote_unwatch、toggle_tray_panel、quit_app、tray recovery 轉發）。
- 切換 workspace 時（主視窗分頁與 tray 面板兩條入口），即使引擎讀取因 repo 忙碌耗時數秒，視窗與 tray 仍可互動——資料以載入中狀態呈現，不再整窗凍結。
- 既有測試全綠：speclink-desktop-core 與 speclink-desktop 的測試不因此變動語意；前端既有行為（畫面、資料形狀）不變。

## Impact

- Affected specs: desktop-app（新增 command 執行緒契約 requirement）
- Affected code:
  - Modified: apps/desktop/src-tauri/src/lib.rs
