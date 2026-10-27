## 1. 監看目標解析

- [x] 1.1 紅：apps/desktop/src-tauri/src/watch.rs 新增 resolve_watch_target 單元測試——(a) 以 tempdir 專案根下的子目錄（模擬 target/release 作為啟動 cwd）為起點，解析出該專案的 spec 目錄（<專案根>/openspec）；(b) 專案根含 `.speclink.yaml` 設定 spec_dir 為非預設名稱時，解析為該實際目錄；(c) 起點位於任何 speclink 專案之外時回傳可記錄的 Err、不 panic。驗證：cargo test -p speclink-desktop 出現上述測試的預期紅燈。
- [x] 1.2 綠：落實需求「監看根解析與專案探索一致」——在 apps/desktop/src-tauri/src/watch.rs 實作 resolve_watch_target（呼叫 speclink_desktop_core::init_core_context 自起點向上探索專案，回傳 workspace 的 spec 目錄路徑），並將 watch_openspec 改為接收已解析的監看目錄、不再以 cwd 拼接固定 openspec 名稱；既有三項 watch 測試（合併通知、範圍外不觸發、目標缺失回 Err）改用新簽名且維持原行為斷言。驗證：cargo test -p speclink-desktop 全綠（含 1.1 新測試）。

## 2. 啟動接線

- [x] 2.1 apps/desktop/src-tauri/src/lib.rs 的 setup 改以 resolve_watch_target 的結果掛載監看，AppState.root 同步採用探索出的專案根；探索不到專案時維持既有降級行為（root 退回啟動 cwd、錯誤 eprintln 記錄、app 照常提供其餘功能）。行為契約：以非專案根 cwd 啟動時，外部寫者改動 spec 目錄樹後 workspace-changed 事件照常送達前端並觸發整批 refresh。驗證：cargo build --release -p speclink-desktop 通過；npm test -w apps/desktop 全綠（前端無行為變更）。

## 3. 真實視窗驗證

- [x] 3.1 驗證 spec Scenario「自非專案根 cwd 啟動後外部開工即時反映」：關閉執行中的 speclink-desktop.exe → npm run build -w apps/desktop → cargo build --release -p speclink-desktop → 以 target\release 為工作目錄啟動 exe → 於外部終端執行 speclink in-progress add 對一個測試 change 蓋開工章 → 數秒內看板卡片自「提案中」欄移至「進行中」欄，全程無重啟、無任何 app 內操作（截圖留證）。操作前先確認使用者未使用螢幕。
- [x] 3.2 驗證降級行為：於任何 speclink 專案之外的目錄啟動 exe → app 照常開啟、無錯誤彈窗、看板為空專案狀態；監看建立失敗僅出現於日誌。驗證後清理測試用 change 的開工標記（還原 .openspec.yaml 或使用拋棄式測試專案）。
