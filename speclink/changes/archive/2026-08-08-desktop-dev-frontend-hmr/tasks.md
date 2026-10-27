## 1. 連接埠一致性守門與 dev server 設定

- [x] 1.1 [Red] 於 scripts/dev.test.mjs 新增失敗測試：讀取 apps/desktop/src-tauri/tauri.conf.json 與 apps/desktop/vite.config.ts，斷言 build.devUrl 的埠號與 vite server.port 為同一個值，且 strictPort 為 true。此測試守的是「兩份設定各改一邊」這個真實回歸風險——埠號不一致時 dev 視窗會載入失敗而非明確報錯。驗證：node --test scripts/dev.test.mjs 於設定尚未加入時失敗，且失敗訊息指出缺少 devUrl。 <!-- speclink-task:tsk_01KZEADWWJ2TDFPN8NGBV7J162 -->
- [x] 1.2 [Green] 落實「決策二：固定 Vite dev server 連接埠並開啟嚴格模式」：於 apps/desktop/vite.config.ts 的匯出設定新增 server 區塊（port 為 1420、strictPort 為 true），既有 base、build.outDir、build.emptyOutDir 與多頁入口設定不變。行為契約：連接埠被占用時 Vite 以非零狀態結束並輸出錯誤，不靜默改用其他埠。驗證：node --test scripts/dev.test.mjs 中 1.1 的埠號一致性斷言轉為通過。 <!-- speclink-task:tsk_01KZEADWWJH3WPSAKCQPBBK8RE -->
- [x] 1.3 [Green] 落實「決策一：dev 模式改由 Vite dev server 供應前端」：於 apps/desktop/src-tauri/tauri.conf.json 的 build 區塊新增 devUrl（指向 1420 埠）與 beforeDevCommand（呼叫 apps/desktop 既有的 vite 啟動腳本），frontendDist 值維持不變。行為契約：tauri dev 改以 devUrl 載入 webview，前端不再經編譯期嵌入 binary。驗證：node --test scripts/dev.test.mjs 全數通過；npm test -w apps/desktop 通過，確認獨立的 vitest.config.ts 未受 vite.config.ts 變動影響。 <!-- speclink-task:tsk_01KZEADWWJ9Y87N99MS93AM9PB -->

## 2. 移除 dev 編排中的前端建置

- [x] 2.1 [Red] 更新 scripts/dev.test.mjs 中現有兩處斷言前置步驟包含 desktop 前端建置指令的測試，改為斷言「一鍵啟動 remote 開發環境」的完整模式前置步驟僅含 speclink-cli 建置、「單獨啟動 desktop」模式的前置步驟為空陣列。同時確認 CLI build 失敗即拒絕啟動、Ctrl+C 同殺兩個 child 的既有測試維持不變。驗證：node --test scripts/dev.test.mjs 於 dev.mjs 尚未修改時，新斷言失敗而既有守門測試仍通過。 <!-- speclink-task:tsk_01KZEADWWJJV0YXBMNSXKSZZ6B -->
- [x] 2.2 [Green] 落實「決策三：從 dev.mjs 前置步驟移除 desktop 前端建置」：修改 scripts/dev.mjs 的 devPrerequisites，使其在完整模式僅回傳 CLI 建置步驟、單獨 desktop 與單獨 server 模式皆回傳空陣列。行為契約：npm run dev 啟動至視窗開啟之間的終端輸出不再含編排 script 發起的前端建置，前端改由 tauri dev 啟動的 dev server 供應；CLI 建置失敗即拒絕啟動的守門不得弱化。驗證：node --test scripts/dev.test.mjs 全數通過。 <!-- speclink-task:tsk_01KZEADWWJX8EV8BAJ6S8FJD7C -->
- [x] 2.3 [Refactor] 清除因 2.2 而孤兒化的死碼：devPrerequisites 的 isWindows 參數（其唯一消費者為已移除的前端建置步驟）與 startDevEnvironment 對應的可注入選項一併移除；模組層級的 Windows 平台常數須保留，因其仍被 child process 生成函式與訊號處理使用。同步更新 scripts/dev.test.mjs 中任何注入該選項的測試。驗證：node --test scripts/dev.test.mjs 全數通過；於 scripts/dev.mjs 搜尋該參數名已無殘留引用。 <!-- speclink-task:tsk_01KZEADWWJ5N5DKR5YWXR27H2B -->

## 3. 行為驗收與回歸確認

- [x] 3.1 手動驗收「dev 模式前端由 dev server 供應且變更免重編」的核心行為（此為跨 process 的互動行為，現有測試框架無法自動涵蓋）：執行 npm run dev，修改 apps/desktop 前端一處可見文字並存檔，確認 dev 視窗顯示修改後文字、終端未出現 Rust 重編輸出、tauri dev process 未重啟。驗證：以上三項逐項目視確認並記錄結果。 <!-- speclink-task:tsk_01KZEADWWJG6K0BYV2DAQHZZYA -->
- [x] 3.2 手動驗收系統匣面板入口：於 3.1 的 dev 視窗開啟系統匣面板，確認面板正常顯示內容。此步驟驗證的是面板以相對路徑建立 webview 時，Tauri 確實以 devUrl 為基底解析、且 Vite dev server 供應第二個 HTML 進入點——design 將此列為須實測而不得僅依推論的項目。驗證：面板視窗顯示內容且無載入失敗。 <!-- speclink-task:tsk_01KZEADWWJGAXJGPNRTDAT1SRB -->
- [x] 3.3 驗收連接埠占用的失敗模式：先占用 1420 埠再執行 npm run dev:desktop，確認指令以非零 exit code 結束且終端出現連接埠被占用的錯誤訊息，未開出載入失敗的空白視窗。驗證：以 echo 檢查 exit code 非零，並目視錯誤訊息。 <!-- speclink-task:tsk_01KZEADWWJ2PAG4BQ8E4A87TCJ -->
- [x] 3.4 確認「決策四：release 與 bundle 路徑維持不變」未回歸：執行 desktop 的 release bundle 建置（不需安裝），確認仍能產出 bundle 且其前端取自靜態產物目錄，並確認未新增 beforeBuildCommand 而造成前端重複建置。驗證：bundle 建置成功結束，且 scripts/desktop-install.mjs 內容未被本次變更修改。 <!-- speclink-task:tsk_01KZEADWWJGW2AK997E5XYG603 -->
- [x] 3.5 套用 sharp-edges audit 檢查清單於本次新增的四個設定選項（devUrl、beforeDevCommand、server.port、server.strictPort），重點檢視預設值是否安全、失敗是否靜默：確認 strictPort 為 true 使埠衝突明確失敗而非靜默降級，且 devUrl 僅影響 dev 模式而不洩入 release 路徑。驗證：逐項對照清單並記錄結論；以 speclink instructions --skill audit 取得清單。 <!-- speclink-task:tsk_01KZEADWWJNRK8T5H88Y6JFB1F -->
- [x] 3.6 收尾回歸：本 change 橫跨 scripts/ 與 apps/desktop 兩面，於收尾執行 node --test "scripts/**/*.test.mjs" 與 npm test -w apps/desktop 各一次，確認全數通過。因本次未修改任何 Rust 原始碼，speclink-core 的 render golden 與 CLI 測試預期零影響，仍執行 cargo test --workspace 一次以驗證而非推論。 <!-- speclink-task:tsk_01KZEADWWJ1GXJWQ1T8D5DD9WK -->
