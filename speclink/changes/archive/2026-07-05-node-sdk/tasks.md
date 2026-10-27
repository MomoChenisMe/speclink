## 1. 綁定骨架與 fs 形式

- [x] 1.1 撰寫 Node 對照測試：於 `crates/speclink-node/__test__/engine.spec.ts` 以 fixture 專案斷言 createEngine fs 形式的 dispatch(['list','--json'])、dispatch(['status','--change',…,'--json']) 回傳物件與 CLI 同專案輸出逐欄位一致——紅燈（骨架未建）
- [x] 1.2 實作使測試轉綠：新增 `crates/speclink-node/Cargo.toml`（napi、napi-derive；`Cargo.toml` workspace members 加入）、`crates/speclink-node/src/lib.rs`（createEngine fs 形式：內部建 speclink-fs 實作；dispatch 以背景工作執行、回傳 Promise、輸出走既有 --json 序列化路徑）、`crates/speclink-node/package.json`（napi-rs 標準佈局）
- [x] 1.3 驗證：npm test 綠；cargo test 全 workspace 綠（CLI 無回歸）

## 2. JS Store 橋接

- [x] 2.1 撰寫橋接測試：JS Store 物件（方法回傳 Promise）支撐 dispatch 讀路徑；缺方法時 createEngine 同步拋錯並列出方法名；Store 方法 reject 時 dispatch 以 Error 傳遞（message 含 store 方法名前綴）——紅燈
- [x] 2.2 實作：`crates/speclink-node/src/store_bridge.rs` 以 ThreadsafeFunction 將 JS Store 方法橋接為 core 同步 Store 介面（工作執行緒經 channel 等待 Promise 解析）；建構時驗證方法集完整性——綠燈
- [x] 2.3 重構與壓力驗證：連續與並發各一百次 dispatch 的無死結測試；cargo clippy 無新警告（覆蓋需求：createEngine 的雙形式儲存建構）

## 3. 寫路徑與 stdin 參數

- [x] 3.1 撰寫測試：dispatch(['new','artifact',…,'--stdin'], { stdin }) 觸發 Store 寫入；claim 衝突以 Error（message 語義化、code 反映類別）拒絕——紅燈
- [x] 3.2 實作：dispatch 第二參數（stdin 內容）注入既有動詞的 stdin 路徑；錯誤轉換層（exit code 與 reason → Error.code）——綠燈
- [x] 3.3 驗證：npm test 全綠（覆蓋需求：dispatch 的輸入輸出契約）

## 4. 渲染 API

- [x] 4.1 撰寫測試：skills.list() 清單內容；skills.render 與 instructions.render 於（neutral × tool-call × remote）組合的措辭斷言；（claude × cli × fs）輸出與 CLI init 生成的 SKILL.md 一致——紅燈
- [x] 4.2 實作：`crates/speclink-node/src/render.rs` 直通 core 渲染矩陣（與 CLI 共用同一渲染程式碼）——綠燈
- [x] 4.3 驗證：npm test 全綠，一致性對照通過

## 5. 型別、發佈與文件

- [x] 5.1 撰寫 `crates/speclink-node/index.d.ts`：Engine、Store（方法回傳 T | Promise<T>）、RenderOptions、DispatchOptions 型別；npm 套件中繼資料（名稱 @speclink/engine、平台 optionalDependencies）
- [x] 5.2 建立 CI 預編譯管線：`.github/workflows/` 新增五目標（win-x64、darwin-x64、darwin-arm64、linux-x64-gnu、linux-arm64-gnu）建置與打包工作流程；驗證：五目標建置全綠
- [x] 5.3 撰寫 `docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md`：安裝與平台注意（native module、npm install 即用、無系統依賴）、createEngine 兩形式、Store 介面實作指南（逐方法說明與 wadpilot 式資料庫映射示例）、dispatch 契約與逐動詞 payload 連結（verb-contract 文件）、渲染 API、Copilot SDK defineTool("speclink") 完整整合範例與 skillDirectories 餵入流程、勿在 Store 方法內同步回呼 engine 的警告；`README.md` Documentation 章節增列雙語連結
- [x] 5.4 驗證：README 引用路徑存在；npm test 與 cargo test 全綠；npm pack 產出套件內容清單正確
