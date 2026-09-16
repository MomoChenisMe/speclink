## 1. 步驟 A：面板先收再喚起主視窗（設計 D1、D2）

- [x] 1.1 新增 Tauri command `hide_tray_panel`（設計 D2）：apps/desktop/src-tauri/src/lib.rs 單行委派到 apps/desktop/src-tauri/src/panel.rs 的 hide 函式；面板不存在或已隱藏時回 Ok、非 macOS 回 Ok 無事。驗證：`cargo check -p speclink-desktop` 通過，且 lib.rs 的 generate_handler 列表含 hide_tray_panel。 <!-- speclink-task:tsk_01M2MKHFMKA3TD9V7WANJBKNWN -->
- [x] 1.2 apps/desktop/src/tray.ts 的 openMainWindow 在 unminimize 之前先 invoke `hide_tray_panel`（失敗靜默、不阻斷後續 show 與 setFocus）；所有會喚起主視窗的 open-* 動作與 add-project、open-project-settings、open-settings 都走同一條 openMainWindow（設計 D2）。驗證（設計 D5 自動層）：apps/desktop/src/__tests__/tray.test.ts 新增斷言——觸發 open-change 動作時 invoke 的第一個呼叫為 hide_tray_panel、之後才是視窗 show／setFocus；`npm test -w apps/desktop` 通過。 <!-- speclink-task:tsk_01M2MKHFMK8N6QAKCB7SD0180W -->
- [x] [M] 1.3 以 scripts/desktop/desktop-install.mjs --install 安裝開發版，依規格「面板交接主視窗後滑鼠互動完整」的情境走重現路徑（設計 D1 的裁判、D5 手動層）：主視窗背景 5～10 秒→開面板→點變更列→主視窗前景、抽屜開啟時滑鼠移到抽屜內按鈕 hover 要亮→點灰底關抽屜後卡片 hover 與 tooltip 要正常；再走「開啟 Speclink」與討論列兩條路徑各一次。驗證：三條路徑 hover 皆正常即記「A 通過」於本任務後方，2 與 3 整組以「A 通過，不執行」勾銷；任一路徑仍死即記「A 未通過」並進入 2。 **實測結果：A 通過**（2026-09-16 使用者於本機安裝 worktree 建置的開發版（引擎 v1.37.0）實測，三條路徑 hover 皆正常）。 <!-- speclink-task:tsk_01M2MKHFMKCBXP0J4QP42X9E1M -->

## 2. 步驟 B：Rust 端聚焦命令取代前端 setFocus（設計 D3；僅 1.3 記「A 未通過」時執行）

- [x] 2.1 新增 Tauri command `focus_main_window`（設計 D3）：lib.rs 單行委派；實作放 panel.rs 的 macOS 視窗交接區段，順序固定為 activateIgnoringOtherApps → makeKeyAndOrderFront → 把主視窗的 WKWebView 設回 first responder；找不到主視窗回 Err 單行訊息；非 macOS 委派既有 show 加 set_focus。驗證：`cargo check -p speclink-desktop` 通過，generate_handler 含 focus_main_window。 **A 通過，不執行**（依 1.3 實測，2026-09-16）。 <!-- speclink-task:tsk_01M2MKHFMKZGZQ0D69S0QHSX8F -->
- [x] 2.2 apps/desktop/src/tray.ts 的 openMainWindow 改為 hide_tray_panel → unminimize → invoke `focus_main_window`；invoke 失敗時 console.error 並退回既有 show 加 setFocus（設計 D3）。驗證（設計 D5 自動層）：tray.test.ts 斷言 open-change 觸發後 invoke 序列為 hide_tray_panel、focus_main_window 且未呼叫 setFocus；模擬 focus_main_window 拒絕時斷言退回 show 加 setFocus；`npm test -w apps/desktop` 通過。 **A 通過，不執行**（依 1.3 實測，2026-09-16）。 <!-- speclink-task:tsk_01M2MKHFMKZN5EPJGFCGF9W1S3 -->
- [x] [M] 2.3 重新安裝開發版，走 1.3 的三條路徑（規格「面板交接主視窗後滑鼠互動完整」）。驗證：皆正常即記「B 通過」，3 整組以「B 通過，不執行」勾銷；仍死即記「B 未通過」並進入 3。 **A 通過，不執行**（依 1.3 實測，2026-09-16）。 <!-- speclink-task:tsk_01M2MKHFMK61CNZ3NPBXNW55GZ -->

## 3. 步驟 C：交接後重算主視窗追蹤區（設計 D4；僅 2.3 記「B 未通過」時執行）

- [x] 3.1 focus_main_window 尾端對主視窗 contentView 呼叫 invalidateCursorRectsForView、對 WKWebView 呼叫 updateTrackingAreas，兩者皆在主執行緒執行、無視覺副作用（設計 D4）。驗證：`cargo check -p speclink-desktop` 通過；panel.rs 的交接函式註解記載此段為追蹤區重算補丁與其依據。 **A 通過，不執行**（依 1.3 實測，2026-09-16）。 <!-- speclink-task:tsk_01M2MKHFMK03WR0RRK0MCSWP10 -->
- [x] [M] 3.2 重新安裝開發版，走 1.3 的三條路徑（規格「面板交接主視窗後滑鼠互動完整」）。驗證：皆正常即記「C 通過」；仍死即記「A／B／C 皆未通過」，本變更停在此、依設計 Risks 回 discuss 重開。 **A 通過，不執行**（依 1.3 實測，2026-09-16）。 <!-- speclink-task:tsk_01M2MKHFMKGPYE15R0278BKDV5 -->

## 4. 收尾

- [x] 4.1 依實測結果修整 proposal.md 的 Impact 與 design.md 的 Decisions（設計 D1 的「未執行步驟勾銷」規則）：未執行的步驟所對應的命令與檔案自 Impact 移除或標明未建立，並在 design.md 的 Open Questions 記下哪一步通過與依據。驗證：`speclink validate fix-desktop-hover-after-tray-panel` 通過；`speclink analyze fix-desktop-hover-after-tray-panel --json` 無 critical 與 warning。 <!-- speclink-task:tsk_01M2MKHFMKKZZ3H55AVDDKHSR3 -->
- [x] 4.2 既有面板情境回歸（規格「面板交接主視窗後滑鼠互動完整」的「面板已先收合時交接仍成功」與「Cmd-Tab 回前景行為不變」情境）：面板開啟不搶前景 app 焦點、點面板外自動收合、Cmd-Tab 回前景 hover 正常。驗證：`npm test -w apps/desktop` 通過；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。 <!-- speclink-task:tsk_01M2MKHFMKMHM0B705EAY21GVJ -->
