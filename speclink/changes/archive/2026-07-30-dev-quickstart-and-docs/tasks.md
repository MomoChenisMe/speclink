## 1. dev harness 單獨模式

- [x] 1.1 撰寫 dev.mjs 模式分流的單元測試（scripts 目錄，node --test）：server-only 模式的啟動計畫不含 CLI 建置、前端建置與 desktop；desktop-only 模式的啟動計畫為前端建置後啟動 tauri dev、不含 server；兩模式沿用既有設定驗證（postgres 缺 URL 即 throw）。驗證：node --test 紅燈確認測試有效。 <!-- speclink-task:tsk_01KYS5Q058YNB9DX4DY5TQ9DWA -->
- [x] 1.2 實作 scripts/dev.mjs 模式分流並於 package.json 新增 dev:server 與 dev:desktop scripts，使 1.1 轉綠，落實需求「單獨啟動 server」與「單獨啟動 desktop」。驗證：node --test 全綠；實跑 npm run dev:server 於全新環境印出 /setup?token= 連結且無 desktop 視窗、Ctrl+C 無殘留 process；實跑 npm run dev:desktop 先完成前端建置再開啟本地模式視窗且未啟動 server。 <!-- speclink-task:tsk_01KYS5Q05871K44E6X0HE0C34K -->

## 2. npm run cli 自動建置

- [x] 2.1 撰寫 cli.mjs 自動建置的單元測試（scripts 目錄，node --test，注入 runSync 斷言）：binary 不存在時先以 checkout root 為工作目錄觸發 cargo build -p speclink-cli 再執行 debug binary、絕不執行 PATH 中的 speclink；建置失敗時 stderr 顯示原因、非零收場、不執行任何 CLI；建置進度不寫入 stdout。驗證：node --test 紅燈確認測試有效。 <!-- speclink-task:tsk_01KYS5Q0587HKXPJ4533ZQ4ZV1 -->
- [x] 2.2 實作 scripts/cli.mjs 自動建置使 2.1 轉綠，落實修改後的需求「checkout 內 CLI 測試入口」。驗證：node --test 全綠；手動刪除 target/debug/speclink 後執行 npm run cli -- --version 自動建置並輸出版本；npm run --silent cli -- list --json 的 stdout 僅含 CLI 的 JSON payload（camelCase 欄位維持既有契約）。 <!-- speclink-task:tsk_01KYS5Q05890JT577212PVZ3HW -->

## 3. 開發者文件

- [x] 3.1 撰寫 docs/development.zh-TW.md 與 docs/development.md 雙語對，落實需求「開發者入口文件雙語對」：npm run dev、dev:server、dev:desktop、dev:reset、cli 五個入口各一節（用途、前置條件、預期可觀察結果），加上未簽章安裝檔放行教學（macOS 系統設定放行、Windows SmartScreen 仍要執行），放行節產物名稱對齊 desktop-release 規格。驗證：文件中每條指令逐一實跑可執行且輸出與敘述相符；兩語言版章節骨架一一對應。 <!-- speclink-task:tsk_01KYS5Q058SKFQ9NK088428J4G -->
- [x] 3.2 README.md 加指向 docs/development.zh-TW.md 的導流連結、README.en.md 加指向 docs/development.md 的導流連結。驗證：連結目標檔案存在、內容審閱一次通過。 <!-- speclink-task:tsk_01KYS5Q058FWB5T7QCBDP536N9 -->

## 4. 回歸驗證

- [x] 4.1 全量回歸：npm run test:all 的 scripts 段全綠；npm run dev 仍為整套編排（CLI 建置→前端建置→server＋desktop 同起）、npm run dev:reset 僅重置不建置；未動任何 CLI 子指令輸出，cargo test -p speclink-core --test render_golden 維持綠燈。驗證：上述指令逐一執行並記錄結果。 <!-- speclink-task:tsk_01KYS5Q058NT4TKW4XYCJT7QBY -->
