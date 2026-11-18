## Context

把本機專案做成匯入包的程式是 `apps/desktop/src-tauri/src/remote.rs` 的 `build_import_bundle`。它經引擎的儲存介面（`Store`）讀取本機專案：進行中的變更（metadata、依產出流程列出的文件、evidence）、正式規格、進行中與已封存的討論、已封存的變更（全部文件）、流程設定、詞彙表，以及由各變更與討論的順序鍵組成的看板順序文件。它還需要工作區的資訊來解析變更的產出流程（含使用者層的自訂流程）與組出錯誤訊息裡的檔案路徑。任何一個變更的 metadata 壞掉，或變更含有不支援的文件形式，整個組裝就中止。

它的產物是第 1 版的匯入包（`ImportBundle`：每份文件以種類命名，另帶目標的專案與 repo）。Desktop 送出時先把它轉成第 2 版（每份文件以儲存的鍵命名）送去，Server 不認得第 2 版時退回送第 1 版。轉換的函式已經在 `speclink-host` 的 `transfer` 模組（`from_first_version`、`key_of_document`）。

`speclink-host` 已經依賴 `speclink-core` 與 `speclink-fs`。`speclink-node` 依賴 `speclink-host`。`apps/desktop` 的殼層與 `speclink-desktop-core` 也依賴 `speclink-host`。

Node SDK 的本機引擎由 `speclink-node` 的 `engine_from_fs` 建立，目前只有 `dispatch`。宿主（`createHost`）已經有 `exportScope` 與 `importScope`，回傳與接受第 2 版的包；JS 這一側的 `ScopeBundle` 型別的 `scopeVersion` 是選填。

## Goals / Non-Goals

**Goals:**

- 宿主的程式能用 Node SDK 把本機專案做成匯入包，再用既有的 `host.importScope` 寫進自己的儲存。
- 做匯入包的程式只有一份，Speclink Desktop 與 Node SDK 共用。
- Speclink Desktop 的遷移行為不變。

**Non-Goals:**

- CLI 的匯入與匯出動詞。
- 對外提供第 1 版的包。
- 壓縮、部分匯出、匯出到檔案（呼叫端拿到物件後自己決定怎麼存）。
- 改動匯入的規則與線上格式。

## Decisions

### D1 匯入包的組裝搬到 speclink-host 的 transfer 模組

`speclink-host` 的 `transfer` 模組新增一個函式，接受工作區、儲存介面與使用者層的設定目錄，回傳「文件種類加內容」的有序清單（也就是第 1 版的包去掉目標專案與 repo 之後的部分）。`build_import_bundle` 的主體與它用到的私有輔助函式（解析產出流程、列出變更的文件、插入文件時的檢查、看板順序文件的組裝）一起搬過去，邏輯不改。

兩個呼叫端：

- Speclink Desktop：`build_import_bundle` 留在原處但只剩幾行——檢查目標的專案與 repo、建立工作區與儲存、呼叫共用的函式、把結果包成帶目標的第 1 版包。它之後的送出流程（先送第 2 版、不認得時退回第 1 版）不動。
- Node SDK：呼叫共用的函式，再用既有的 `from_first_version` 同一套「種類轉成鍵」的規則轉成第 2 版的包。

放在 `speclink-host` 而不是 `speclink-core`，因為它要解析使用者層的產出流程與工作區，這些是應用層的事；`transfer` 模組已經是匯入匯出的家。放在 `speclink-host` 而不是 `speclink-desktop-core`，因為 `speclink-node` 不依賴桌面的 crate，也不該依賴。

回傳「種類加內容」而不是直接回傳第 2 版，因為 Desktop 的退回路徑還需要第 1 版；種類轉成鍵是單向的，反過來做不到。

替代方案：讓 Node SDK 用本機資料夾建一個儲存接口的實作，再呼叫既有的 `transfer::export`。否決：repo 裡的檔案系統儲存接口範例用的是自己的資料夾配置，不是 checkout 的規格資料夾；為了匯出再寫一個「checkout 配置的儲存接口」比搬一個函式大得多。

### D2 Node 的本機引擎新增 exportScope

- 原生端：本機引擎的類別新增方法 `export_scope`，和 `dispatch` 一樣在背景執行緒上執行、用同樣的信封回傳。它用建立引擎時的專案根目錄與規格資料夾建立工作區與檔案儲存，呼叫 D1 的函式，轉成第 2 版的包，以 JSON 文字回傳（比照宿主的 `export_scope`：大的包以一個字串過橋）。包的 `scope_version` 是沒有，序列化時不輸出。
- `index.js`：`Engine` 的包裝新增 `exportScope()`，解包並解析 JSON。
- `index.d.ts`：`Engine` 介面新增 `exportScope(): Promise<ScopeBundle>`，說明寫明它讀的是 `createEngine` 指向的本機專案、沒有 `scopeVersion`、可以直接交給 `host.importScope`。

文件依鍵的位元組順序排列：轉成第 2 版之後排序一次。Desktop 送出的第 2 版維持它既有的順序，不受這個變更影響。

錯誤的對應：共用函式的失敗（壞掉的 metadata、不支援的文件形式、轉不成鍵的名稱）在原生端轉成 `dispatch` 既有的錯誤信封，`code` 用 `error`，`message` 沿用共用函式給的文字（它已經點名檔案）。

### D3 兩個入口的對照測試固定「同一份實作」

- `speclink-host` 的 `transfer` 測試：把 Desktop 既有的匯入包測試搬到共用函式的旁邊（內容涵蓋、evidence、壞 metadata 的中止、轉不成鍵的名稱）。Desktop 那邊保留針對「目標專案與 repo 的檢查」與「包成第 1 版」的測試。
- Node 的測試：匯出一個 fixture 專案，斷言包的形狀；匯入 JS 的記憶體儲存後，用 `host.dispatch` 與 `engine.dispatch` 對照讀到的內容。
- 兩個入口的文件逐項相同，由「都呼叫同一個函式」保證；再加一個 `speclink-host` 的測試直接斷言：共用函式的結果經 `from_first_version` 轉換後的鍵集合，等於逐一對每個種類呼叫 `key_of_document` 的結果，沒有轉不成鍵的種類。

### D4 使用者文件

- Node SDK 文件（中英）的「匯出與匯入」一節：新增本機專案的匯出，並給一段批次搬遷的範例——對每個專案資料夾 `createEngine`、`exportScope`、`host.importScope` 到各自的空範圍；說明失敗時不會有不完整的包、匯入的範圍必須是空的。該節結尾「CLI 沒有匯入匯出的動詞。搬專案的入口是 Speclink Desktop 與你自己的程式」改寫成指出三個入口：Speclink Desktop、`engine.exportScope()` 加 `host.importScope`、HTTP 入口。
- 專案能力狀態（中英）：已知缺口「CLI 沒有匯入與匯出的動詞」那一列保留，替代做法加上「宿主的程式：用 `createEngine` 的 `exportScope()` 把本機專案做成包」；Node SDK 那一列的能力說明提到本機專案的匯出。

## Implementation Contract

**行為**

- `await createEngine({ store: { type: 'fs', root } }).exportScope()` 回傳第 2 版的包，內容是該本機專案的全部規格文件，可以直接交給 `host.importScope`。
- 匯出只讀不寫。
- Speclink Desktop 的「遷移到 Server」行為與送出的內容不變。

**介面與資料形狀**

- Node：`Engine.exportScope(): Promise<ScopeBundle>`；回傳值 `{ formatVersion: 2, documents: [{ key, content, digest }] }`，`documents` 依鍵的位元組順序，沒有 `scopeVersion`。
- Rust：`speclink-host` 的 `transfer` 模組新增一個公開函式，輸入是工作區、`&dyn Store` 與使用者層的設定目錄，輸出是第 1 版包的文件清單或錯誤；它不寫入、不連線。

**失敗模式**

- 變更（進行中或已封存）的 metadata 無法解析：整個匯出失敗，訊息點名檔案。
- 變更含有不支援的文件形式，或文件的名稱轉不成鍵：整個匯出失敗，訊息點名文件。
- `createEngine` 指向的資料夾不是本機專案：沿用 `createEngine` 與 `dispatch` 既有的行為。
- 任何失敗都不回傳部分的包。

**驗收**

- `cargo test -p speclink-host transfer`：共用函式的測試（由 Desktop 搬來的內容涵蓋、evidence、中止條件）與 D3 的鍵對照測試。
- `cargo test -p speclink-desktop`（先補齊 sidecar 與 server-web dist）：遷移相關的既有測試全部通過，包的內容不變。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`：node-sdk 規格 delta 的五個情境。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍**

- 範圍內：共用函式與它的測試、Desktop 改用它、Node 的新方法與型別、文件。
- 範圍外：CLI、線上格式、匯入的規則、宿主的 `exportScope`、Desktop 的畫面與送出流程。

## Risks / Trade-offs

- [搬動程式時不小心改了 Desktop 遷移的內容] → 搬動不改邏輯；Desktop 既有的包內容測試跟著搬到共用函式旁邊，原樣通過才算完成；另跑 `cargo test -p speclink-desktop`。
- [回歸對照] → CLI 的輸出與引擎的 golden 都不涉及；Node SDK 既有的測試不改。
- [變更 claim-release-and-output 與 event-payloads-and-push-url 也會改 Desktop 殼層的同一個檔案] → 它們改的是線上型別的字面值與事件訂閱，這個變更搬走的是匯入包的組裝，區塊不同；為了避免合併時互相干擾，把這個變更排在 event-payloads-and-push-url 之後。
- [大的專案匯出時整包放在記憶體] → 和 Desktop 的遷移、宿主的匯出相同；參考量是 165 個已封存變更、31 MB，在可以接受的範圍。匯入端另有大小上限（預設 32 MiB），文件提醒整合者視需要調高。
- [跨平台] → 鍵一律用正斜線，由既有的鍵建立函式產生，不從檔案系統的路徑拼出來；Windows 上的 Node 測試在 CI 跑同一組斷言。
- [使用者層的自訂產出流程會影響變更的文件清單] → 行為和 Desktop 的遷移相同（讀同一個使用者層設定目錄）；文件說明在宿主機器上執行匯出時，自訂流程要放在執行者的使用者層設定目錄，或改用專案內的流程。
