## 1. locator 與持久化（TDD）

- [x] 1.1 紅（規格「分頁身分為 WorkspaceLocator 而非 root 路徑」與「分頁持久化 v2 與 v1 靜默遷移」；design「決策 1：locator 與 key——分頁身分的單一來源」「決策 2：persisted v2 與 v1 靜默遷移」）：於 apps/desktop/src/__tests__/session.test.ts 與既有 tabs 測試新增案例——locatorKey 的 local:{root} 格式與去重、v2 讀寫往返、v1（root＋name＋activeRoot）靜默遷移為 local locator 與 activeKey、v1 條目 root 非字串即丟棄、壞 JSON 歸零分頁。執行 npm test -w apps/desktop 確認新案例全紅。 <!-- speclink-task:tsk_01KXNVBRP1M1SNRGHWDQR1FZ9F -->
- [x] 1.2 綠：新增 apps/desktop/src/session.ts（WorkspaceLocator 含 remote 變體型別、locatorKey、WorkspaceSession 型別）並改 apps/desktop/src/tabs.ts——分頁條目身分改 locator、persistTabs/readPersistedTabs 升 v2 含 v1 遷移。1.1 案例全綠。 <!-- speclink-task:tsk_01KXNVBRP1XB2PQKWFC2V8AGW3 -->

## 2. session 工廠與 adapter root 化（TDD）

- [x] 2.1 紅（規格「每個 session 自帶 dataSource 且 Rust 側無 current-root 全域」；design「決策 3：session 物件與 createLocalSession 工廠」）：session.test.ts 新增案例——createLocalSession 以注入的假 invoke 建立後，逐支 dataSource 方法與 settings 方法呼叫皆攜帶綁定的 root 參數；事件來源收到 payload root 不等於自身 root 時不觸發回呼、相等時觸發。確認全紅。 <!-- speclink-task:tsk_01KXNVBRP1QMZZ3V34685W7E9S -->
- [x] 2.2 綠：createTauriDataSource 改收 root 並於每支 invoke 帶 root；apps/desktop/src/adapter/workspace.ts 的設定與專案操作面 root 參數化（型別名 WorkspaceSettingsProvider 對齊 design 決策 3）；實作 createLocalSession(root, deps) 組合三者。2.1 案例全綠。 <!-- speclink-task:tsk_01KXNVBRP1A1T1WRTDBQG8PATS -->

## 3. Rust 側單例消滅

- [x] 3.1（design「決策 4：Rust command 逐呼叫收 root、單例消滅」）apps/desktop/src-tauri/src/lib.rs：全部讀寫 command 簽名加 root 參數直通 desktop-core 的帶路徑函式；刪除 AppState 的 root Mutex 與 current_project 命令；open_project 改純探測、同路徑重複呼叫冪等無副作用。驗證：cargo build -p speclink-desktop 通過、無殘留 state.root() 呼叫。 <!-- speclink-task:tsk_01KXNVBRP1RGW70983B495A73Q -->
- [x] 3.2（design「決策 5：watcher 顯式重掛、事件帶 root」；規格「watcher 顯式跟隨活躍 session 且事件攜帶 root」）：新增 watch_workspace(root) command 負責重掛單一 watcher；workspace-changed 事件 emit 攜帶被監看 root；監看不可用僅失去刷新的既有語意保留（apps/desktop/src-tauri/src/watch.rs 與 lib.rs 掛載處）。驗證：cargo build 通過，手動以 CLI 改活躍專案 openspec/ 後看板數秒內更新。 <!-- speclink-task:tsk_01KXNVBRP1Q1F75VZ9V7MPE2R5 -->

## 4. store 與 UI 接線

- [x] 4.1（design「決策 6：store 收 session 工廠、單活躍載入語意不變」）apps/desktop/src/store.ts：createStore 改收 session 工廠與探測面；持 sessions（locatorKey 為鍵）與 activeKey；reload、詳情、動詞、任務操作一律經活躍 session 的 dataSource；openProjectAt＝純探測→upsert session 與分頁→設 activeKey→watch_workspace→reload；首啟活躍專案改由持久化 activeKey 決定。既有 store.test 斷言語意不變地調整注入方式後全綠。 <!-- speclink-task:tsk_01KXNVBRP16ABT1E39V38ND1WT -->
- [x] 4.2 apps/desktop/src/App.tsx 與 apps/desktop/src/main.tsx：移除全域 dataSource 注入、改注入 createLocalSession 工廠；apps/desktop/src/tray.ts 與 apps/desktop/src/panel/TrayPanel.tsx 的分頁項識別改 locatorKey（顯示文字與行為不變）。App、tray、trayPanel、settingsView、workspace 各測試套件調整注入後全綠。 <!-- speclink-task:tsk_01KXNVBRP17MRC0TK581BT0HTK -->

## 5. 凍結驗證

- [x] 5.1 回歸（規格「重構行為凍結」）：npm test -w apps/desktop 與 npm test -w packages/ui 全綠；git status 確認 packages/ui 零 diff；cargo build --release -p speclink-desktop 成功（重建前先關閉執行中的 exe）。 <!-- speclink-task:tsk_01KXNVBRP1P6CJJS0RTTRS533Q -->
- [x] 5.2 GUI 真實視窗手動驗證（規格「重構行為凍結」；操作前確認使用者未在使用螢幕）：兩專案開分頁互切並確認看板內容各自正確、設定頁對活躍專案讀寫且另一專案 config.yaml 不變、外部改檔秒級反映、先以 v1 格式預置 localStorage 再啟動驗證分頁與活躍分頁完整遷移且重啟後為 v2、tray panel 切換專案、重啟恢復分頁列。 <!-- speclink-task:tsk_01KXNVBRP1NW60JE7XSN1KBGE9 -->
