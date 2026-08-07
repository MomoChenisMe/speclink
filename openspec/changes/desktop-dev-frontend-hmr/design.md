## Context

`apps/desktop` 的 Tauri 殼目前只設定了 frontendDist，未設定 devUrl，因此 dev 與 release 共用同一條前端供應路徑：由 Tauri 的 context 產生巨集在編譯 `apps/desktop/src-tauri` 時，將 `apps/desktop/dist` 整包嵌入產出的 binary。同時，build script 宣告的 Cargo 重編觸發清單不含前端產物目錄，於是純前端變更不會使 Cargo 判定需要重編，dev 視窗載入的是上一次重編當下嵌入的舊快照。

`scripts/dev.mjs` 目前在啟動長時間 child process 之前，同步執行兩個前置步驟：建置當前 checkout 的 speclink-cli，以及建置 desktop 前端。後者在上述路徑下對 dev 視窗毫無作用。

本次變更完全不觸及任何 Rust 原始碼，僅調整建置設定與開發編排腳本。

相關既有契約：`apps/desktop` 為多頁入口（主視窗與系統匣面板各一 HTML 進入點），面板視窗以相對路徑字串建立 webview；`scripts/desktop-install.mjs` 的 release 流程是先手動建置前端再呼叫 bundle。

## Goals / Non-Goals

**Goals:**

- 開發者修改 `apps/desktop` 前端原始碼後，變更能在 dev 視窗生效，且不需重編 Rust binary。
- 保住 dev-harness 既有的三項編排契約：CLI 建置失敗即拒絕啟動、兩個長時間 child process 的輸出直通終端、Ctrl+C 同時收束兩者。
- release 與 bundle 路徑零改動。

**Non-Goals:**

- 不處理 dev 與安裝版共用 Tauri identifier 所導致的 app 設定目錄互相覆寫，也不處理指向安裝版的 CLI 符號連結。
- 不新增 beforeBuildCommand，不改動 release 建置順序。
- 不觸碰 `apps/server-web` 的前端產物過期問題（其根因是 server 於 debug 模式動態讀取磁碟目錄，與編譯期嵌入無關）。
- 不改動 Rust 端的 build script 或重編觸發清單。

## Decisions

### 決策一：dev 模式改由 Vite dev server 供應前端

在 `apps/desktop/src-tauri/tauri.conf.json` 的 build 區塊新增 devUrl 與 beforeDevCommand。devUrl 指向本機 Vite dev server；beforeDevCommand 由 Tauri 在啟動 dev 前代為執行，內容為 `apps/desktop` 既有的 vite 啟動腳本。frontendDist 原樣保留，release 仍讀靜態產物。

設定 devUrl 後，Tauri 在 dev 模式改以該網址載入 webview，前端不再經編譯期嵌入，因此 Cargo 是否重編與前端變更徹底脫鉤——這正是根因的直接解除，而非繞道。

面板視窗以相對路徑字串建立 webview，Tauri 在 dev 模式會自動以 devUrl 為基底解析，Vite dev server 亦直接供應根目錄下的第二個 HTML 進入點，故兩個入口皆無需改碼。此點須於驗收時實際確認，不得僅依推論。

替代方案與否決理由：於 build script 宣告前端產物目錄為重編觸發來源，可使前端變更觸發重編，但每次前端改動都要重編逾 80 MB 的 debug binary，迴圈時間以分鐘計，且未解除「前端被嵌進 binary」這個結構成因；由編排腳本主動觸碰 Rust 原始檔以強制重編，同樣要付整趟重編成本，並以偽造時間戳操縱建置系統，維護性差。

### 決策二：固定 Vite dev server 連接埠並開啟嚴格模式

`apps/desktop/vite.config.ts` 目前沒有 server 區塊，Vite 會採用預設連接埠，且在該埠被占用時自動遞增。tauri.conf.json 的 devUrl 是寫死的字串，一旦 Vite 換埠，webview 會載入失敗。因此固定連接埠為 1420（Tauri 官方範本慣例，與 Vite 預設埠錯開，可避免與其他 Vite 專案相撞），並開啟 strictPort。

開啟 strictPort 是刻意選擇的失敗模式：連接埠被占用時，Vite 直接以錯誤結束，而非靜默換埠導致 webview 載入空白頁。明確失敗優於難以診斷的空白視窗。

### 決策三：從 dev.mjs 前置步驟移除 desktop 前端建置

前端建置改由 beforeDevCommand 承擔，`scripts/dev.mjs` 的前置步驟保留前端建置只會造成每次啟動多一次無用的完整建置。移除後，前置步驟僅剩 CLI 建置，且該建置僅適用於完整模式（單獨啟動 desktop 的模式不需要 CLI 建置，單獨啟動 server 的模式本來就不含前置步驟）。

連帶的死碼處理：前端建置步驟是 devPrerequisites 之 isWindows 參數的唯一消費者（CLI 建置以真實 binary 執行，不走 shell），移除後該參數與 startDevEnvironment 對應的可注入選項一併孤兒化，應於本次清除。模組層級的 Windows 平台常數仍被 child process 生成函式與訊號處理使用，須保留。

### 決策四：release 與 bundle 路徑維持不變

不新增 beforeBuildCommand。`scripts/desktop-install.mjs` 現行流程是先手動建置前端再呼叫 bundle，新增該欄位會造成同一份前端被建置兩次。release 路徑因此零改動，devUrl 僅影響 dev 模式。

## Implementation Contract

**行為**：開發者執行 npm run dev 或 npm run dev:desktop 後，修改 `apps/desktop` 下的前端原始碼並存檔，dev 視窗中對應的畫面隨即更新，全程不需重編 Rust binary、不需重啟任何 process。主視窗與系統匣面板兩個入口皆適用。

**介面與資料形狀**：

- `apps/desktop/src-tauri/tauri.conf.json` 的 build 物件新增兩個字串欄位：devUrl 指向本機 1420 埠，beforeDevCommand 為啟動 `apps/desktop` vite 的 npm 腳本呼叫。frontendDist 欄位值不變。
- `apps/desktop/vite.config.ts` 的匯出設定新增 server 物件，含 port 為 1420 與 strictPort 為 true 兩個欄位。既有的 base、build.outDir、build.emptyOutDir 與多頁入口設定皆不變。
- `scripts/dev.mjs` 的 devPrerequisites 函式：回傳陣列在完整模式下僅含 CLI 建置步驟，在單獨 desktop 與單獨 server 模式下皆為空陣列。函式簽名移除 isWindows 參數；startDevEnvironment 的選項物件移除同名可注入項。parseDevMode 與 startDevEnvironment 的其餘行為不變。

**失敗模式**：

- 1420 埠被占用時，Vite 因 strictPort 而以非零狀態結束，錯誤訊息直通終端（beforeDevCommand 的輸出由 Tauri 轉送）。這是刻意選擇的明確失敗，取代靜默換埠後 webview 載入空白頁的難診斷情境。
- CLI 建置失敗時，維持既有守門：npm run dev 以非零 exit code 結束，兩個長時間 process 皆不啟動。此行為不得因本次改動而弱化。

**驗收標準**：

- `scripts/dev.test.mjs`：既有兩處斷言前置步驟含前端建置指令者，改為斷言前置步驟不含該指令；完整模式斷言前置步驟僅含 CLI 建置；單獨 desktop 模式斷言前置步驟為空。CLI 建置失敗即拒絕啟動、Ctrl+C 同殺兩個 child 的既有測試須維持通過。
- 移除 isWindows 參數後，`scripts/dev.test.mjs` 中任何注入該選項的測試須一併更新，且全檔測試通過。
- 手動驗收（本變更的核心行為無法由現有自動化測試覆蓋，須實際執行）：啟動 npm run dev，修改前端一處可見文字並存檔，確認 dev 視窗更新且終端未出現 Rust 重編；再確認系統匣面板視窗能正常開啟並顯示內容。
- release 未回歸：執行 desktop 的 tauri build 流程仍能產出 bundle，證明 frontendDist 路徑未受影響。

**範圍邊界**：

- 在範圍內：上述四個檔案的修改，以及因移除前端建置步驟而孤兒化的參數清除。
- 不在範圍內：任何 Rust 原始碼、build script、`apps/server-web`、`scripts/desktop-install.mjs`、Tauri identifier 與 app 設定目錄隔離、CLI 符號連結指向問題。實作時若發現上述領域的缺陷，記錄之，不順手修改。

## Risks / Trade-offs

- [回歸對照風險：golden 與 CLI 測試] → 本次不修改任何 Rust 原始碼，speclink-core 的 render golden 與 CLI 測試的輸入輸出均不經過本變更觸及的檔案，故預期零影響。實作後仍須執行 cargo test 全 workspace 與 scripts 測試確認，不以推論代替驗證。
- [跨平台風險：beforeDevCommand 的 shell 行為] → 該指令由 Tauri 自身以 shell 執行，三平台語意一致，不經過 dev.mjs 既有的 Windows npm 走 shell 特例。移除 devPrerequisites 的 isWindows 參數不影響 child process 生成路徑，該處使用的是模組層級常數。Windows 與 Linux 上的實際驗證依賴 CI，本機僅能驗 macOS。
- [跨平台風險：連接埠占用機率] → 1420 為 Tauri 範本慣例埠，與 Vite 預設埠錯開，降低與其他前端專案相撞的機率；真的相撞時由 strictPort 明確報錯。
- [開發體驗權衡：多一個常駐 process] → dev 模式下 Vite dev server 常駐，終端輸出比原本多。換得的是前端迭代不再需要重編 Rust，對頻繁改前端的情境淨收益明顯。
- [驗收依賴人工] → 「改前端即時生效」屬於跨 process 的互動行為，現有測試框架無法自動涵蓋。緩解方式是把手動驗收步驟明確寫進 tasks，避免被略過而讓缺陷復發。
