## 1. 共用的匯入包組裝（D1、D3）

- [ ] 1.1 先寫失敗的測試，寫在 crates/host/speclink-host/src/transfer.rs 的測試區塊：把 apps/desktop/src-tauri/src/remote.rs 既有的匯入包測試（本機專案的每一種文件都在包裡且內容相同、進行中與已封存變更的 evidence 在包裡、壞掉的變更 metadata 中止並點名檔案、名稱轉不成鍵的文件中止並點名）改寫成對共用函式的斷言；另加 D3 兩個入口的對照測試固定「同一份實作」的鍵對照——共用函式的結果經 `from_first_version` 轉換後，每一項的鍵等於對該文件種類呼叫 `key_of_document` 的結果，沒有轉不成鍵的種類。驗證：`cargo test -p speclink-host transfer` 這時因共用函式不存在而編譯失敗。 <!-- speclink-task:tsk_01M7PP718YWZK98M623C77VCR5 -->
- [ ] 1.2 實作 D1 匯入包的組裝搬到 speclink-host 的 transfer 模組：把 `build_import_bundle` 的主體與它的私有輔助函式（解析產出流程、列出變更的文件、插入文件時的檢查、看板順序文件的組裝）搬到 crates/host/speclink-host/src/transfer.rs 的一個公開函式，輸入是工作區、`&dyn Store` 與使用者層的設定目錄，邏輯不改。驗證：1.1 的測試通過；`cargo test -p speclink-host` 全部通過。 <!-- speclink-task:tsk_01M7PP718Y6W9R70EWZDJSZ6KB -->
- [ ] 1.3 Speclink Desktop 改用共用的版本（需求「本機專案的匯出與遷移產生相同的文件集合」）：apps/desktop/src-tauri/src/remote.rs 的 `build_import_bundle` 只保留目標專案與 repo 的檢查、建立工作區與儲存、呼叫共用函式、包成帶目標的第 1 版包；已搬走的測試從這個檔案移除，保留針對目標檢查與包成第 1 版的測試；送出流程不動。驗證：先補齊桌面測試需要的 sidecar 與 server-web dist，`cargo test -p speclink-desktop` 全部通過；跑完後執行 `cargo build -p speclink-cli` 讓 CLI 的 binary 回到最新。 <!-- speclink-task:tsk_01M7PP718YRF2K8Z1RH7MJ7720 -->

## 2. Node 的本機引擎匯出（D2）

- [ ] 2.1 先寫失敗的測試，涵蓋需求「本機引擎的 exportScope 方法」的五個情境，寫在新檔 crates/adapters/speclink-node/__test__/export-local.spec.ts：匯出含進行中變更、正式規格與討論的 fixture 專案，包的 formatVersion 是 2、沒有 scopeVersion、documents 依鍵的位元組順序且鍵不重複、每一項的 digest 是 content 的 SHA-256；把包交給 JS 記憶體儲存的 `host.importScope` 後，`host.dispatch` 與 `engine.dispatch` 對 `['list', '--json']`、`['show', 'auth', '--item-type', 'spec']`、`['discuss', 'list', '--json']` 的結果相同（本機才有的欄位除外）；匯出前後規格資料夾每個檔案的內容與修改時間不變；有一個 metadata 壞掉的變更時 Promise 以 Error 拒絕、message 點名該檔、沒有回傳包；剛初始化的專案匯出成功；已封存的變更與討論在匯入後的宿主讀得到。驗證：`npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`，新檔的測試因 `exportScope` 不存在而失敗。 <!-- speclink-task:tsk_01M7PP718YWHZNNTG81EA6F4SC -->
- [ ] 2.2 實作 D2 Node 的本機引擎新增 exportScope：crates/adapters/speclink-node/src/lib.rs 的本機引擎新增在背景執行緒執行的匯出方法——用建立引擎時的專案根目錄與規格資料夾建立工作區與檔案儲存、呼叫共用函式、轉成第 2 版並依鍵排序、以 JSON 文字經既有的信封回傳，失敗時用 `error` 這個 code 與共用函式的訊息；crates/adapters/speclink-node/index.js 的 `Engine` 包裝新增 `exportScope()`；crates/adapters/speclink-node/index.d.ts 的 `Engine` 介面新增 `exportScope(): Promise<ScopeBundle>` 與說明。驗證：build 後 `npm --prefix crates/adapters/speclink-node test` 全部通過，包含 2.1 的五個情境與既有的 engine、host、transfer 測試。 <!-- speclink-task:tsk_01M7PP718Y6APSQFYX55SEDC9B -->

## 3. 稽核

- [ ] 3.1 對新增的方法與公開函式套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得），把結論寫在 crates/host/speclink-host/src/transfer.rs 共用函式的說明註解：匯出只讀不寫；鍵由既有的鍵建立函式產生，不從檔案系統的路徑拼出來，所以專案裡的檔名不能造出範圍外的鍵；任何失敗都不回傳部分的包；包裡有流程設定與討論的全文，文件已提醒整合者由自己決定誰可以執行匯出；匯入端的大小上限與匯出的大小之間的關係已寫進文件。驗證：`cargo test -p speclink-host transfer` 與 `npm --prefix crates/adapters/speclink-node test` 全部通過。 <!-- speclink-task:tsk_01M7PP718YPNA6G3D4S8TFW0E2 -->

## 4. 手動驗證

- [ ] [M] 4.1 用 root 的 `npm run dev:desktop` 開啟 Speclink Desktop，對一個本機專案執行「遷移到 Server」到本機示範用 server 的一個空專案。確認：遷移成功、遠端專案的看板列出同樣的變更與討論、本機照既有流程留下備份並轉換成遠端連線。 <!-- speclink-task:tsk_01M7PP718YBHC56Q8BBSMZJCBH -->

## 5. 文件與收尾（D4）

- [ ] 5.1 依 D4 使用者文件更新，中英兩版同步：docs/sdk-node.md 與 docs/sdk-node.zh-TW.md 的「匯出與匯入」一節新增本機專案的匯出與批次搬遷的範例（對每個專案資料夾 createEngine、exportScope、host.importScope 到各自的空範圍），說明失敗時沒有不完整的包、匯入的範圍必須是空的、大小上限、自訂產出流程的位置，並改寫該節結尾關於搬專案入口的那一句；docs/product-status.md 與 docs/product-status.zh-TW.md 的已知缺口「CLI 沒有匯入與匯出的動詞」那一列加上新的替代做法，Node SDK 那一列提到本機專案的匯出。驗證：文件的範例程式碼使用的方法名稱與回傳欄位和 2.1 的測試一致；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M7PP718YHD2Z9TTRTFQBAK0H -->
