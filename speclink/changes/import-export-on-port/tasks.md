## 1. 匯出與匯入的函式

- [ ] 1.1 （Red）為 D1、D2、D3 寫 `transfer` 模組的測試，用記憶體版儲存與外包一層計數的儲存，對應規格「範圍的匯出與匯入」的函式情境：匯出一個含各類文件（進行中的變更、正式規格、討論、已封存的變更、流程設定、evidence、看板順序）的範圍再匯入另一個空的範圍，兩邊的鍵與內容相同；匯出的文件依鍵排序、摘要正確、帶範圍版本號；空的範圍匯出成沒有文件的包；匯出期間每次都有人提交時在有限次重試後回報暫時無法使用；匯出的讀取分批且每批不超過 200 個鍵；匯入是正好一次提交、每筆預期「必須不存在」、帶一筆 `scope-imported` 事件；非空的範圍被拒絕且沒有提交；確認空之後另一方寫入時以「不是空的」被拒絕且一份都沒寫入；摘要不符、鍵不合格、不屬於引擎命名規則的鍵、重複的鍵各被拒絕並指出鍵，儲存沒有被呼叫；沒有文件的包不提交；第一版的包換成第二版後匯入的結果相同，換不成鍵的文件被拒絕並指出它；用本專案的規格資料夾做一次往返（沿用成本測試的載入方式）。涉及 crates/host/speclink-host/tests/transfer.rs。驗證：`cargo test -p speclink-host --test transfer` 編譯失敗。 <!-- speclink-task:tsk_01M774QC6H8S1JZD1ESE9P7JC2 -->
- [ ] 1.2 （Green）實作 `speclink_host::transfer`：匯出、匯入、包的型別、錯誤型別，以及公開的「文件種類換成鍵」對照（從 TeamStore 的轉接搬來，轉接改呼叫它）。涉及 crates/host/speclink-host/src/transfer.rs、crates/host/speclink-host/src/lib.rs、crates/host/speclink-host/src/teamstore_port.rs、crates/host/speclink-host/src/commit.rs。驗證：1.1 的測試通過；`cargo test -p speclink-host` 全部通過。 <!-- speclink-task:tsk_01M774QC6JZZJ9P7DZ2VRSHGTH -->

## 2. 宿主套件的兩個門

- [ ] 2.1 （Red）為 D4、D5 寫宿主套件的測試，對應規格「範圍的匯出與匯入」的掛鉤與 HTTP 情境、「匯入的大小上限由宿主設定」與「以鍵標示文件的匯入包與匯出回應」：`export_scope` 與 `import_scope` 先問執行前掛鉤（`scope-export`、`scope-import`，後者的參數只有份數與位元組數），拒絕時 `refused` 且範圍不變，匯入成功後通知提交後掛鉤；`POST /import` 接受兩個版本，第一版的回應形狀不變，第二版回份數與版本號，不認得的版本回 409；`GET /export` 回第二版的包且可以原樣匯入另一個範圍；兩條路由對沒有寫入能力的人回 403；預設上限下 33 MiB 的本文回 413 且儲存沒有被呼叫，調高後 40 MiB 成功，調低到 1 MiB 後 2 MiB 回 413，匯出不受上限限制；直接呼叫時內容總量超過上限被拒絕；各種不合格的包回 400、非空回 409。回應的 JSON 斷言 camelCase 與型別。涉及 crates/host/speclink-host/tests/http_door.rs、crates/host/speclink-host/tests/transfer.rs。驗證：新增的測試失敗。 <!-- speclink-task:tsk_01M774QC6JT36AWGXF4WHJFPV3 -->
- [ ] 2.2 （Green）實作：線上第二版的包、匯出的回應與第二版匯入的回應；`HostKit::export_scope`、`HostKit::import_scope` 與 `HostKitOptions.import_limit_bytes`；兩條路由與錯誤的對映；`speclink-remote` 新增匯出的呼叫與送第二版包的呼叫。涉及 crates/protocol/speclink-protocol/src/query.rs、crates/host/speclink-host/src/kit.rs、crates/host/speclink-host/src/http/mod.rs、crates/host/speclink-host/src/http/router.rs、crates/host/speclink-host/src/http/routes.rs、crates/host/speclink-host/src/http/error.rs、crates/protocol/speclink-remote/src/client.rs。驗證：2.1 的測試通過；`cargo test -p speclink-host`、`cargo test -p speclink-protocol` 與 `cargo test -p speclink-remote` 通過。 <!-- speclink-task:tsk_01M774QC6JHEJJJXPMMTS0X4PV -->

## 3. 示範用 server

- [ ] 3.1 依 D7 把示範用 server 的匯入改由宿主套件處理，對應規格「不經引擎命令的文件讀寫只用儲存接口」修改後的內容與「import 端點僅限 CreateNew 且原子」新增的情境：先新增測試——匯入成功時 TeamStore 的匯入操作沒有被呼叫（外包一層計數的 TeamStore）；訂閱著事件串流的用戶端在匯入後收到一筆 invalidation；`GET /export` 對 editor 回包、對 reader 回 403。再改接：移除 server 的匯入處理函式與它對 TeamStore 匯入的呼叫，轉交的處理函式不再把 `/import` 留給 server，建立宿主套件時傳入上限。依 design 的 Open Question 量測參考量在三個舊 driver 上一次提交的結果，寫進 design.md；任何一個 driver 不能一次寫入時停下來向使用者回報。涉及 crates/host/speclink-server/src/app.rs、crates/host/speclink-server/src/state.rs、crates/host/speclink-server/src/api/routes.rs、crates/host/speclink-server/src/api/forward.rs、crates/host/speclink-server/tests/it/api/import_api.rs、speclink/changes/import-export-on-port/design.md。驗證：`cargo test -p speclink-server` 全部通過，`import_api` 既有的測試不改斷言。 <!-- speclink-task:tsk_01M774QC6JBHHGKS8WAFAWHK4Y -->

## 4. Speclink Desktop

- [ ] 4.1 （Red→Green）依 D6 補齊組包並改送第二版，對應規格「Bundle 組裝涵蓋本地 workspace 全集」修改後的內容：先寫測試——有 evidence 的變更，包裡有它的 evidence 文件；已封存變更的 evidence 在包裡；各變更與討論有順序鍵時，包裡有看板順序文件且順序與本機相同，沒有任何順序鍵時不含這份文件；送出的是第二版的包；Server 以「格式版本不認得」拒絕時改送第一版並成功；任一來源檔解析失敗時仍然中止並點名該檔。再實作。涉及 apps/desktop/src-tauri/src/remote.rs。驗證：`cargo test -p speclink-desktop` 通過，既有的遷移整條鏈測試不改斷言，新增「遷移後遠端有 evidence 與看板順序」的斷言。 <!-- speclink-task:tsk_01M774QC6JCAT62TJ6QB0AYJB8 -->

## 5. Node SDK

- [ ] 5.1 （Red→Green）對應規格「宿主的匯出與匯入方法」的六個情境：先寫測試——匯出再匯入後另一個範圍列出同一個變更且 commit 一次；匯入非空的範圍是 `refused` 且沒有 commit；摘要不符是 `invalid_argv` 且訊息含鍵；`beforeCommand` 對 `scope-import` 拒絕時是 `refused` 與宿主的原因；`limits.importBytes` 經 `handleRequest` 生效（413）；`importBytes` 不是正整數時 `createHost` 同步拋出錯誤；兩個方法不阻塞事件迴圈。再實作並更新型別宣告，在註解寫明宿主框架的本文上限要不小於這個值、壓縮由路由層做。涉及 crates/adapters/speclink-node/__test__/transfer.spec.ts、crates/adapters/speclink-node/src/host.rs、crates/adapters/speclink-node/index.js、crates/adapters/speclink-node/index.d.ts。驗證：先 `cargo build -p speclink-cli`，再 `npm --prefix crates/adapters/speclink-node run build` 與 `npm --prefix crates/adapters/speclink-node test` 通過。 <!-- speclink-task:tsk_01M774QC6JW7HRNR1R9RZF6ZYY -->

## 6. 實際走一遍

- [ ] [M] 6.1 用 Speclink Desktop（`npm run dev:desktop`）把一個有進行中變更、evidence、看板排序與已封存變更的本機專案搬到本機的示範 server：確認搬移成功，遠端的看板順序和本機相同，用 CLI 執行 `speclink evidence show <change>`（遠端）讀得到原本的記錄。 <!-- speclink-task:tsk_01M774QC6JNR9HZR74KNJSKAXY -->

## 7. 收尾

- [ ] 7.1 對新增的介面套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得）：確認匯入的鍵只能落在請求的範圍裡，包裡的鍵不能帶出別的範圍或別類文件（含 `..`、開頭的 `/`、反斜線）；第一版的包裡的範圍與請求不符時被拒絕；上限在解析之前生效，不能用很小的本文加很深的 JSON 巢狀耗盡記憶體；匯出的路由要求寫入的能力；掛鉤的參數與錯誤訊息不帶文件內容；匯入失敗的每一條路徑都沒有任何寫入；匯出的重試有次數上限；回答 design 的第二個 Open Question（每批的鍵數）並寫進 design.md。找到的問題在這個任務修正並補測試，結論寫進 design.md 新增的「稽核結論」一節。涉及 crates/host/speclink-host/src/transfer.rs、crates/host/speclink-host/src/http/routes.rs、speclink/changes/import-export-on-port/design.md。驗證：`cargo test -p speclink-host` 通過；design.md 有「稽核結論」一節。 <!-- speclink-task:tsk_01M774QC6JHZCCKQ9PY81DTSX0 -->
- [ ] 7.2 最後驗證 design 的 Implementation Contract。涉及 crates/host/speclink-host、crates/protocol/speclink-protocol、crates/protocol/speclink-remote、crates/host/speclink-server、crates/adapters/speclink-node、apps/desktop/src-tauri。驗證：`cargo test -p speclink-host`、`cargo test -p speclink-protocol`、`cargo test -p speclink-remote`、`cargo test -p speclink-server`、`cargo test -p speclink-cli`、`cargo test -p speclink-desktop` 通過；Node 的建置與測試通過；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過；`speclink validate import-export-on-port` 通過。 <!-- speclink-task:tsk_01M774QC6JYN9P4Q5JE5V2VYT2 -->
