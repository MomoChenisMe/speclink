## Context

把本機專案做成匯入包的程式是 `apps/desktop/src-tauri/src/remote.rs` 的 `build_import_bundle`。它經引擎的儲存介面（`Store`）讀取本機專案：進行中的變更（metadata、依產出流程列出的文件、evidence）、正式規格、進行中與已封存的討論、已封存的變更（全部文件）、流程設定、詞彙表，以及由各變更與討論的順序鍵組成的看板順序文件。它還需要工作區的資訊來解析變更的產出流程（含使用者層的自訂流程）與組出錯誤訊息裡的檔案路徑。任何一個變更的 metadata 壞掉，或變更含有不支援的文件形式，整個組裝就中止。

它的產物是第 1 版的匯入包（`ImportBundle`：每份文件以種類命名，另帶目標的專案與 repo）。Desktop 送出時先把它轉成第 2 版（每份文件以儲存的鍵命名）送去，Server 不認得第 2 版時退回送第 1 版。轉換的函式已經在 `speclink-host` 的 `transfer` 模組（`from_first_version`、`key_of_document`）。

`speclink-host` 已經依賴 `speclink-core` 與 `speclink-fs`。`speclink-node` 依賴 `speclink-host`。`apps/desktop` 的殼層與 `speclink-desktop-core` 也依賴 `speclink-host`。

Node SDK 的本機引擎由 `speclink-node` 的 `engine_from_fs` 建立，目前只有 `dispatch`。宿主（`createHost`）已經有 `exportScope` 與 `importScope`，回傳與接受第 2 版的包；JS 這一側的 `ScopeBundle` 型別的 `scopeVersion` 是選填。

## Goals / Non-Goals

**Goals:**

- 宿主的程式能用 Node SDK 把本機專案做成匯入包，再用既有的 `host.importScope` 寫進自己的儲存。
- 做匯入包的程式只有一份，Speclink Desktop 與 Node SDK 共用。
- Speclink Desktop 的遷移行為不變，唯一的例外是規格資料夾含有符號連結的專案（D5）。
- 匯出與遷移都不經符號連結讀檔，連結指向的內容不會進到任何包裡（D5）。
- 匯出只讀專案根目錄裡面的規格資料夾（D6）。D5 與 D6 合起來，兩個入口都不讀專案外的檔案。

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

組裝在讀取任何文件之前，先檢查規格資料夾在專案根目錄裡面（D6），再檢查規格資料夾裡沒有符號連結（D5）。

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

### D5 規格資料夾裡的符號連結讓匯出與遷移中止

實作期的稽核實測：`speclink/specs/leak/spec.md` 是一個連到專案外檔案的符號連結時，D1 的共用函式經檔案儲存讀到連結指向的內容，那個外部檔案的全文就進到包裡。Desktop 的遷移用同一份程式，所以今天就有同樣的行為。

- 落點：D1 的共用函式（`speclink-host` 的 `transfer` 模組）在經儲存讀取任何文件之前，從專案根目錄下的規格資料夾（工作區的根目錄加規格資料夾名稱）開始，用不跟隨連結的讀法（`symlink_metadata`）逐一看每個項目：通往規格資料夾路上的每一層資料夾（不含專案根目錄本身，避開根目錄經過連結的情況，例如 macOS 的暫存資料夾）、規格資料夾本身、它底下的每個檔案與資料夾。舊位置的 evidence 資料夾（工作區的 `touched_dir()`，也就是 `.speclink/touched/`）照同樣的規則檢查：變更資料夾裡沒有 evidence 時，檔案儲存會退回讀這裡。看到第一個符號連結就回傳錯誤，不再往下讀。只往實體資料夾裡走，不進入符號連結的資料夾。同一層的項目依名稱排序，所以同一個專案每次點名同一個項目。這些位置有列不出內容的資料夾（例如只有執行權限）時也回傳錯誤：`cannot list <路徑> to check it for symbolic links: <原因>`，因為儲存仍可能依已知檔名讀到裡面經連結的檔案。
- 錯誤訊息：`symbolic link in the spec folder: <路徑> — replace it with the file itself or remove it`。`<路徑>` 是該項目相對專案根目錄的路徑，一律用正斜線（例如 `speclink/specs/leak/spec.md`）。訊息不含連結指向的位置，也不含任何檔案內容。
- 規格資料夾不存在：不算錯，照舊回傳空的清單（沿用 `dispatch` 對這種資料夾的既有行為）。
- 兩個入口：都經同一個函式，所以中止條件相同。Desktop 照既有流程把錯誤顯示在遷移的畫面，送出之前就停下，不做備份也不轉換；Node 以 `error` 這個 code 拒絕 Promise。
- 查整個資料夾，而不是只查會進包的檔案：儲存介面不提供文件的檔案路徑；逐種文件推回路徑，要在這裡重複一份檔案佈局的知識（新舊資料檔名、封存的日期資料夾）。查整個資料夾的規則簡單、不會漏。代價是規格資料夾裡與文件無關的符號連結（例如專案自訂流程的資料夾）也會讓匯出中止；訊息點名它，使用者移掉或換成實體檔案即可。
- 只讀：只用讀取資料夾與讀取項目資訊的呼叫，不寫入。
- 範圍：只有這兩個入口。CLI 與其他動詞讀規格資料夾的方式不變；匯入端（`host.importScope`、HTTP 入口的匯入）收到的是包，不讀本機檔案，不在範圍內。任務 3.2 的搜尋另外命中兩個測試檔：crates/host/speclink-host/tests/transfer.rs 與 crates/host/speclink-server/tests/it/api/import_api.rs。兩者都在匯入端用記憶體裡組好的包做測試，不讀本機檔案，不修。

替代方案：跟著連結讀，但拒絕指到專案外的連結。否決：要解析每個連結的真實路徑並比對專案根目錄，還要處理多層連結與根目錄本身經過連結（例如 macOS 的暫存資料夾）的情況；規則複雜，錯了就是洩漏。一律拒絕，使用者的改法也簡單。

### D6 規格資料夾必須在專案根目錄裡面

實作期的稽核實測：checkout 的 `.speclink.yaml` 寫 `spec_dir: ../other/speclink`，或寫成一個絕對路徑時，`engine.exportScope()` 讀的是那個專案外的資料夾，它的內容就進到包裡。規格資料夾名稱今天沒有任何檢查：工作區把專案根目錄接上這個名稱，絕對路徑會整個取代根目錄，`..` 會往上走出專案。

- 落點：D1 的共用函式，在 D5 的符號連結檢查之前、經儲存讀取任何文件之前。
- 規則：規格資料夾名稱拆成路徑段後，每一段 SHALL 是一般名稱。下列情形一律拒絕：絕對路徑（Unix 的 `/…`、Windows 的磁碟代號或網路路徑）、任何 `..` 段、任何 `.` 段、空的名稱。`docs/specs` 這種專案裡的多層名稱照常接受。
- 名稱的來源不分：checkout 的 `.speclink.yaml` 的 `spec_dir`，或 `createEngine` 明確傳入的 `specDir`，都經同一個檢查。宿主要匯出其他位置的規格資料夾，就把 `createEngine` 的 `root` 指到它的上一層。
- 錯誤訊息：`spec folder outside the project: "<名稱>" — spec_dir must name a folder under the project root`。`<名稱>` 是設定裡原樣的名稱，放在雙引號裡。訊息不含任何檔案內容。
- 只比對名稱，不解析檔案系統的真實路徑：名稱本身就決定了儲存讀哪裡；名稱在專案裡面之後，再由 D5 擋下指到外面的符號連結。
- 兩個入口：Desktop 的遷移在組包之前就以既有的檢查拒絕預設以外的規格資料夾（`migratable_spec_dir`），所以 D6 對 Desktop 沒有可觀察的變化；它對 Node 的入口生效。
- 同類位置：送出本機規格內容的出口只有 D1 的共用函式（任務 3.2 的搜尋結果），D6 落在同一個函式裡，不另做排查。
- 範圍：只有匯出。CLI 與其他動詞讀 `spec_dir` 的方式不變：它們讀的是使用者自己的 checkout，不把內容送到別的儲存。

替代方案：解析規格資料夾的真實路徑，確認它在專案根目錄的真實路徑底下。否決：要處理根目錄本身經過連結的情況（例如 macOS 的暫存資料夾），而且 D5 已經拒絕規格資料夾裡的連結；只看名稱的規則簡單、可預期。

## Implementation Contract

**行為**

- `await createEngine({ store: { type: 'fs', root } }).exportScope()` 回傳第 2 版的包，內容是該本機專案的全部規格文件，可以直接交給 `host.importScope`。
- 匯出只讀不寫。
- Speclink Desktop 的「遷移到 Server」行為與送出的內容不變；規格資料夾含有符號連結的專案例外，遷移在送出前中止（D5）。
- 兩個入口都不經符號連結讀檔，連結指向的內容不出現在任何包、回傳值或訊息裡。
- 匯出只讀專案根目錄裡面的規格資料夾（D6）。

**介面與資料形狀**

- Node：`Engine.exportScope(): Promise<ScopeBundle>`；回傳值 `{ formatVersion: 2, documents: [{ key, content, digest }] }`，`documents` 依鍵的位元組順序，沒有 `scopeVersion`。
- Rust：`speclink-host` 的 `transfer` 模組新增一個公開函式，輸入是工作區、`&dyn Store` 與使用者層的設定目錄，輸出是第 1 版包的文件清單或錯誤；它不寫入、不連線。

**失敗模式**

- 變更（進行中或已封存）的 metadata 無法解析：整個匯出失敗，訊息點名檔案。
- 變更含有不支援的文件形式，或文件的名稱轉不成鍵：整個匯出失敗，訊息點名文件。
- 規格資料夾名稱是絕對路徑、含 `..` 或 `.` 段、或是空的：整個匯出在讀取任何檔案之前失敗，訊息以雙引號原樣點名那個名稱（D6）。
- 通往規格資料夾路上的資料夾、規格資料夾本身或它底下任何一個檔案或資料夾是符號連結（舊位置的 evidence 資料夾亦同），或這些位置有列不出內容的資料夾：整個匯出失敗，訊息以相對專案根目錄、正斜線的路徑點名第一個找到的項目（同一層依名稱排序）；Desktop 的遷移在送出前以同一個訊息中止。
- `createEngine` 指向的資料夾不是本機專案：沿用 `createEngine` 與 `dispatch` 既有的行為。
- 任何失敗都不回傳部分的包。

**驗收**

- `cargo test -p speclink-host transfer`：共用函式的測試（由 Desktop 搬來的內容涵蓋、evidence、中止條件）、D3 的鍵對照測試，以及 D5 的三種符號連結（規格資料夾底下的檔案、底下的資料夾、規格資料夾本身）各讓匯出失敗並點名該項目；D6 的規格資料夾名稱：`../other/speclink` 與一個絕對路徑失敗並點名名稱，`docs/specs` 照常匯出。
- `cargo test -p speclink-desktop`（先補齊 sidecar 與 server-web dist）：遷移相關的既有測試全部通過，包的內容不變；D5 的 Desktop 測試：含符號連結的專案組不出遷移的包，訊息點名同一個項目。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`：node-sdk 規格 delta 的情境，包含符號連結讓匯出失敗的情境，以及 `.speclink.yaml` 的 `spec_dir` 指到專案外讓匯出失敗的情境。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍**

- 範圍內：共用函式與它的測試、Desktop 改用它、Node 的新方法與型別、兩個入口的符號連結檢查（D5）、規格資料夾名稱的檢查（D6）、文件。
- 範圍外：CLI、線上格式、匯入的規則、宿主的 `exportScope`、Desktop 的畫面與送出流程、其他動詞讀規格資料夾時對符號連結與 `spec_dir` 的處理。

## Risks / Trade-offs

- [檔案存在但讀不出來（沒有權限、內容不是 UTF-8）] → 沿用 Desktop 既有行為：儲存讀不到內容，包裡就沒有那份文件，不算失敗。程式註解、`index.d.ts` 與 SDK 文件寫明「不給部分結果」只針對 Implementation Contract 列的失敗。
- [匯出期間有人寫入同一個專案] → 本機專案沒有範圍版本號可以比對，匯出逐一讀檔、不做快照；`index.d.ts` 與 SDK 文件提醒整合者匯出期間不要寫入（經同一個引擎、CLI 或其他程式）。不加鎖：鎖只擋得住同一個實例，還會改變既有 `dispatch` 的並行行為。
- [搬動程式時不小心改了 Desktop 遷移的內容] → 搬動不改邏輯；Desktop 既有的包內容測試跟著搬到共用函式旁邊，原樣通過才算完成；另跑 `cargo test -p speclink-desktop`。
- [回歸對照] → CLI 的輸出與引擎的 golden 都不涉及；Node SDK 既有的測試不改。
- [變更 claim-release-and-output 與 event-payloads-and-push-url 也會改 Desktop 殼層的同一個檔案] → 它們改的是線上型別的字面值與事件訂閱，這個變更搬走的是匯入包的組裝，區塊不同；為了避免合併時互相干擾，把這個變更排在 event-payloads-and-push-url 之後。
- [大的專案匯出時整包放在記憶體] → 和 Desktop 的遷移、宿主的匯出相同；參考量是 165 個已封存變更、31 MB，在可以接受的範圍。匯入端另有大小上限（預設 32 MiB），文件提醒整合者視需要調高。
- [跨平台] → 鍵一律用正斜線，由既有的鍵建立函式產生，不從檔案系統的路徑拼出來；Windows 上的 Node 測試在 CI 跑同一組斷言。
- [含符號連結的專案以前遷移得了，現在會中止（D5）] → 這是刻意的行為改變；訊息點名項目並說明改法。本 repo 的規格資料夾沒有任何符號連結。Desktop 的既有遷移測試的 fixture 都沒有連結，不受影響。
- [跨平台：符號連結的測試] → 在 Windows 上建立符號連結要特別權限，所以 Rust 的符號連結測試只在 Unix 編譯（`#[cfg(unix)]`），Node 的符號連結測試在 Windows 上略過；檢查本身用標準函式庫的跨平台讀法，Windows 上同樣認得符號連結。訊息裡的路徑一律轉成正斜線。
- [宿主明確傳入絕對路徑的 `specDir`，以前 `exportScope` 還沒有，這次就不能匯出（D6）] → `exportScope` 是新方法，沒有既有使用者；`dispatch` 不受影響。文件說明改法：把 `root` 指到規格資料夾的上一層。
- [跨平台：絕對路徑的寫法] → Windows 的磁碟代號（`C:\…`、`C:foo`）與網路路徑（`\\server\share`）拆成路徑段時都有不是一般名稱的段，一律拒絕。Rust 的測試用暫存資料夾的絕對路徑，在三個平台都成立。
- [使用者層的自訂產出流程會影響變更的文件清單] → 行為和 Desktop 的遷移相同（讀同一個使用者層設定目錄）；文件說明在宿主機器上執行匯出時，自訂流程要放在執行者的使用者層設定目錄，或改用專案內的流程。
