## Context

**事件的兩層。** 引擎的命令執行層對每個成功的寫入回報領域事件（`speclink-core` 的 `DomainEvent`）。宿主套件把它們轉成「名稱加內容」（`speclink-host` 的 `commit.rs`，`event_payload`），和寫入一起交給儲存的 `commit`，提交後再交給提交後掛鉤。推送給用戶端的是另一層：`commit.rs` 的對照把事件名稱歸成資源類別（變更、規格、討論、其他），串流送出的只有「哪個資源變了」，不帶事件的內容。

- 蓋章：`ReviewStamp` 與 `VerifyStamp` 命令帶著呼叫端給的 `head_commit`，引擎把它寫進 metadata。事件 `ReviewStamped`、`VerifyStamped` 只有變更名稱；蓋章的結果型別（`ReviewSubjectOutcome`）也只有變更名稱。
- 流程設定：兩個寫入命令的結果（`WorkflowConfigWriteOutcome`）有 `changed_keys` 與 `written`，但事件清單固定是空的，原始碼註記這是 workflow-config-commands 當時列的不做項目。
- 事件的內容已經不是一律只有名稱：`ChangeLanded` 帶 commit 與分支，自由欄位的兩個事件帶變動的鍵。

**Speclink Desktop 的訂閱。** `apps/desktop/src-tauri/src/lib.rs` 在開啟遠端專案時向事件管理器註冊兩個函式：訂閱（呼叫 `speclink_remote::events::subscribe`，它把 `/events` 接在專案的 API 位址後面）與狀態查詢（`sync_state`）。訂閱失敗時，事件管理器走既有的「SSE 不可用」路徑：用狀態查詢的 ETag 輪詢，並以退避重試訂閱。握手的宣告只被用來算一個布林（有輪詢或有 SSE 就算有即時更新），宣告的 url 沒有被讀取。所以連到只宣告輪詢的宿主時，Desktop 仍然會反覆嘗試那個不存在的固定路徑。

握手的宣告型別（`speclink-protocol` 的 `EventTransport`）有 `type`、`url`、`resume`。示範用 server 宣告 `/events`，輪詢宣告 `/sync-state`；兩者都是相對於專案的 API 位址。宿主套件的測試用的是完整的網址。

用戶端的斷線判定在 `speclink-remote` 的 `events.rs`：45 秒沒有收到任何位元組就視為中斷（`STALL_LIMIT`）。示範用 server 的心跳預設 15 秒。

## Goals / Non-Goals

**Goals:**

- 宿主在 `commit` 與提交後掛鉤拿到的事件足以直接記錄：蓋章是哪個 commit、流程設定改了哪些鍵。
- Speclink Desktop 連宿主宣告的串流，而且不會把憑證送到別的主機。
- 文件寫明事件的設計、推送網址的寫法與心跳的數字。

**Non-Goals:**

- 推送給用戶端的訊息不加內容。
- 不新增推送的資源類別。
- 用戶端不實作 WebSocket；不改輪詢。
- 不改其他事件的內容，不改蓋章與流程設定寫入的行為與輸出。
- 宿主套件不提供現成的串流。

## Decisions

### D1 蓋章事件的 commit 來自蓋章的結果

蓋章改用專屬的結果型別 `StampOutcome`（`change` 與 `commit: Option<String>`），`commit` 是這次蓋章實際寫進 metadata 的 commit（沒有寫就是沒有），由 `station::stamp_with_scope` 回傳。兩個放棄的結果維持只有變更名稱的 `ReviewSubjectOutcome`，所以放棄的結果不會帶一個永遠是空的 commit。`DomainEvent::ReviewStamped` 與 `VerifyStamped` 新增同名的選填欄位，由結果帶入。`event_payload` 在有值時輸出 `commit`，沒有值時不輸出這個鍵，所以沒有 commit 的蓋章事件內容與現在逐位元相同。

用結果而不是命令的 `head_commit` 當來源，因為引擎會先檢查 commit 的形狀；事件要反映的是記下來的值。

動詞的輸出（人眼與 `--json`）與 HTTP 入口的蓋章回應只讀 `change`，所以輸出不變。

### D2 流程設定的寫入在真的寫入時回報 workflow-config-changed

`DomainEvent` 新增 `WorkflowConfigChanged { keys: Vec<String>, occurred_at }`，名稱 `workflow-config-changed`。事件清單的產生處（目前對三個流程設定的結果固定回傳空清單）改成：單項編輯與整份取代的結果 `written` 為真時回傳這一筆，`keys` 由 `changed_keys` 依原順序轉成字串；`written` 為假、以及讀取（show）時仍是空清單。`event_payload` 輸出 `{ "keys": [...] }`。

這筆事件沒有變更名稱或討論 slug 可以當主體；主體是範圍本身，比照匯入的 `scope-imported`。`commit.rs` 的資源對照不把它加進任何一組，所以它落在「其他」，推送出去是 unknown。

這個決定推翻 workflow-config-commands 的不做項目。當時的理由是把「宿主怎麼得知」留給提供掛鉤的變更；實際上執行前掛鉤在提交之前，拿它當「已經改了」的通知會把被拒絕或失敗的寫入也算進去。

`keys` 可以是空陣列（只改排版或註解）：文件確實變了，顯示設定原文的用戶端需要重新讀取。

### D3 串流網址的解析是 speclink-remote 的一個純函式

`speclink-remote` 的 `events` 模組新增 `stream_url(project_base, declared) -> Option<StreamUrl>`：

- `declared` 以 `http://` 或 `https://` 開頭：解析出它的 origin 與 `project_base` 的 origin（協定、主機、埠；埠省略時用協定的預設值），相同就回傳解析後的網址，不同就回傳沒有。
- `declared` 以 `/` 開頭：把 `project_base` 去掉結尾斜線後接上 `declared`，解析並比較 origin，回傳解析後的網址。
- 其他寫法（空字串、不以 `/` 開頭的相對路徑、別的協定）：回傳沒有。

回傳的 `StreamUrl` 存的是解析結果的正規寫法（`url::Url` 的序列化：主機小寫、預設埠省略、`.` 與 `..` 化簡），不是宣告的原始字串。`subscribe` 把它交給 ureq 連線；檢查 origin 與連線用的是同一次解析的結果，所以日後 ureq 換了解析器，「檢查時看到的主機」與「實際連線的主機」也不會分岔（品質關卡第 1 輪的審查結論）。宣告的寫法已經是正規寫法時，回傳的文字與宣告的逐字相同，示範用 server 的 `/events` 不受影響。

`subscribe` 的第一個參數由「專案的 API 位址」改成「串流的完整網址」（`&StreamUrl`），不再自己接 `/events`；它的其他行為（標頭、續傳、斷線判定、錯誤翻譯）不變。`sync_state` 不變。

`StreamUrl` 是只有 `stream_url` 做得出來的型別（沒有公開的建構方式，`as_str()` 取出文字）。`subscribe` 會送出存取憑證，收普通字串的話，之後的呼叫端可以繞過同 origin 的檢查直接把宣告的網址傳進去；收 `StreamUrl` 時繞過檢查就編譯不過（apply 期的 sharp-edges 稽核結論）。

origin 的比較用 `url` 套件的解析結果（它已經由 ureq 帶進 Cargo.lock；`speclink-remote` 的 Cargo.toml 把它宣告成直接依賴），不自己切字串，避免 `https://specs.example.com@evil.example` 這類寫法被誤判成同一個主機。

解析放在 `speclink-remote` 而不是 Desktop，因為它是用戶端對線上宣告的解讀，之後別的用戶端也會用到，而且純函式可以在沒有桌面殼層的情況下測試。

### D4 Speclink Desktop 把握手宣告的第一個 SSE 網址交給訂閱

- `apps/desktop/src-tauri/src/remote.rs` 在握手之後，從宣告的 transports 取第一個 `type` 是 sse 的 url，經 `stream_url` 解析，把結果（一個選填的 `StreamUrl`）放在握手建立的把手上（`RemoteWorkspace::declared_stream`）。它不在回給前端的開啟結果 `RemoteOpenInfo` 裡：網址會收到存取憑證，只留在 Rust 端，不送給前端、也不由前端轉交回來。既有的 `live_updates` 布林的算法不變。
- 同一個檔的 `DeclaredStreams` 以 connection、project、repo 記下每個分頁的串流網址。`apps/desktop/src-tauri/src/lib.rs` 的 `remote_open` 把握手的結果記進去；`remote_watch` 註冊的訂閱函式是 `remote::subscribe_declared`，每次訂閱都重讀這張表：有網址就用它呼叫 `subscribe`；沒有網址就不發出任何請求，直接回傳「串流不可用」的錯誤。`connection_remove` 把那條連線記下的網址一起清掉。前端一定先以 `remote_open` 的結果建立分頁、再訂閱，所以兩者用的是同一次握手。事件管理器對訂閱失敗已經有完整的處理（以狀態查詢輪詢、退避重試），所以沒有網址時自然只做輪詢，也不會呈現錯誤。

Tauri 殼層只做取值與轉交；判斷都在 `stream_url`。

### D5 文件寫明事件的設計、網址的寫法與心跳

- 動詞契約文件（中英）的事件一節：開頭加一段說明事件只指出哪個資源變了、收到後重新讀取，這是刻意的；補上宣告 url 的兩種寫法與同 origin 的限制；補上「示範用 server 預設每 15 秒送一次心跳、用戶端 45 秒沒有收到資料就重連、自己做串流的宿主要在 45 秒內送心跳」；事件對照表下方的說明加上 `review-stamped`、`verify-stamped` 的 `commit` 與 `workflow-config-changed` 的 `keys`，並說明後者歸在「其他」。
- Node SDK 文件（中英）：`pushTransports` 的段落寫明 url 的兩種寫法、同 origin 的限制與心跳的要求；「一個動詞的四個步驟」提到流程設定的寫入現在有事件；`index.d.ts` 的 `HostPushTransport` 說明同步。
- 專案能力狀態（中英）：推送事件那一列的說明更新（Speclink Desktop 使用宣告的網址）。

### D6 45 秒的斷線時限只有一個來源，示範用 server 拒絕不小於它的心跳

45 秒同時是用戶端的斷線判定與提供串流的一方的心跳上限。這個數字放在兩邊都依賴的 `speclink-protocol`：`crates/protocol/speclink-protocol/src/events.rs` 新增 `pub const STALL_LIMIT_SECS: u64 = 45`。

- `speclink-remote` 的 `STALL_LIMIT` 改成由它算出（`Duration::from_secs(STALL_LIMIT_SECS)`），行為不變。
- `crates/host/speclink-server/src/config.rs` 讀設定檔時，`events.heartbeat_secs` 大於或等於 `STALL_LIMIT_SECS` 就回 `ConfigError::InvalidValue`（欄位 `events.heartbeat_secs`、規則 `must be less than 45`，數字取自常數）。這和既有的 `events.buffer` 檢查是同一種錯誤，server 既有的啟動路徑會印出訊息並在綁定任何埠之前以非零 exit code 結束。
- 沒有設定時的預設值（15 秒）與 44 以下的值不變。只檢查設定檔這一個入口；程式內直接建構 `EventSettings` 的地方（測試用毫秒級的間隔）不經過這項檢查。
- serde 結構不變：`heartbeat_secs` 仍是選填的整數，既有的設定檔只要值小於 45 就照常讀取。

為什麼拒絕而不是只寫文件：設成 45 以上時每個用戶端都會每 45 秒斷線重連一次，這是設定錯誤，啟動時就指出來比事後查連線問題容易。

### D7 稽核結論各放在它講的程式旁邊

`stream_url` 的說明註解只留網址解析與憑證去向的結論，另加一句指向 `event_payload`。「事件內容不含設定的值、token 或文件內容，推送給用戶端的訊息不含事件的內容」這一點搬到 `crates/host/speclink-host/src/commit.rs` 的 `event_payload` 說明註解：它講的是事件的內容，內容一改就該在那裡更新。這修訂任務 5.1 當時寫的位置。

## Implementation Contract

**行為**

- 蓋章有記下 commit 時，`review-stamped`、`verify-stamped` 的內容多一個 `commit`。
- 流程設定真的寫入時多一筆 `workflow-config-changed`，內容 `{ keys }`。
- 兩者都出現在儲存的 `commit` 收到的事件與提交後掛鉤收到的事件。
- 推送給用戶端的訊息格式不變；流程設定的寫入現在會讓用戶端收到一筆 unknown 的提示。
- Speclink Desktop 訂閱握手宣告的 SSE 網址；沒有可用的網址時只輪詢。連線用的是解析後的正規寫法。
- 示範用 server 的設定檔 `events.heartbeat_secs` 是 45 以上時不啟動。

**介面與資料形狀**

- 事件內容：`review-stamped`／`verify-stamped` → `{ "change": string, "commit"?: string }`；`workflow-config-changed` → `{ "keys": string[] }`。
- `speclink_remote::events::stream_url(project_base: &str, declared: &str) -> Option<StreamUrl>`；`subscribe(stream_url: &StreamUrl, token, repo, last_event_id)`。
- `StreamUrl::as_str()` 是解析後的正規寫法。
- `speclink_protocol::events::STALL_LIMIT_SECS: u64 = 45`。
- 示範用 server 的設定錯誤訊息：`config file '<path>': 'events.heartbeat_secs' must be less than 45`。
- 握手的型別、輪詢、線上端點都不變。

**失敗模式**

- 宣告的 url 是別的 origin、別的協定或無法解析：`stream_url` 回傳沒有；Desktop 不發請求、不呈現錯誤、改用輪詢。
- 訂閱到宣告的網址但對方不是 SSE 或回錯誤：沿用既有的訂閱失敗處理。
- 流程設定的寫入失敗、被拒絕或沒有改變內容：沒有事件。
- 設定檔的 `events.heartbeat_secs` 是 45 以上：示範用 server 印出指出欄位與規則的訊息，不綁定埠，exit code 非零。

**驗收**

- `cargo test -p speclink-core`：蓋章結果與事件帶 commit 的測試（有與沒有兩種）；流程設定事件的測試（規格 delta 的範例表六列）。
- `cargo test -p speclink-host`：`event_payload` 的三種輸出；宿主套件的測試確認儲存的 `commit` 與提交後掛鉤都收到新的事件內容；HTTP 入口的測試確認推送的對照把 `workflow-config-changed` 歸為 unknown；既有斷言「流程設定寫入的事件是空的」的地方刻意更新。
- `cargo test -p speclink-remote`：`stream_url` 的表格測試（規格 delta 範例表的七列，加上使用者名稱夾帶主機的寫法與預設埠的寫法）；`events_sse` 的既有測試改用完整的串流網址後全部通過。
- `cargo test -p speclink-desktop`（先補齊 sidecar 與 server-web dist）：握手宣告轉成選填網址的測試。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`：JS 的 `commit` 函式與 `afterCommit` 收到新的事件內容。
- `cargo test -p speclink-remote`（修訂後）：`stream_url` 回傳正規寫法的列（主機大小寫、預設埠、`..`）；`STALL_LIMIT` 等於協定常數。
- `cargo test -p speclink-server --lib config::`：規格 delta「各種 heartbeat_secs 的結果」五列。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。
- 手動：Speclink Desktop 連到示範用 server，即時更新照常運作。

**範圍**

- 範圍內：兩種事件的內容與發出條件、事件內容的輸出、串流網址的解析、Desktop 的訂閱、示範用 server 對 `events.heartbeat_secs` 的上限檢查、文件。
- 範圍外：推送訊息的格式、資源對照的類別、WebSocket、輪詢、其他事件、宿主套件的握手內容。

## Risks / Trade-offs

- [流程設定的寫入現在有事件，會讓「提交後掛鉤收到的事件是空的」這類既有斷言失敗] → 搜尋 `speclink-host` 與 `speclink-node` 的測試中對流程設定寫入事件的斷言，同批更新；套件尚未發版，沒有外部使用者要遷移。
- [回歸對照] → 動詞的輸出不變，引擎的 golden 與 CLI 測試不需要更新；事件內容只新增欄位，沒有 commit 的蓋章事件逐位元不變，由 `commit.rs` 既有的內容測試固定。
- [連到示範用 server 的 Desktop 行為] → `/events` 經 `stream_url` 解析出的網址和原本寫死的相同，用一個測試固定；另有手動驗證。
- [把憑證送到別的主機] → 只接受同 origin；origin 用解析後的協定、主機、埠比較；測試涵蓋使用者名稱夾帶主機、大小寫不同的主機、預設埠寫與不寫三種寫法。
- [流程設定每次寫入都讓用戶端重新讀取全部] → 流程設定的寫入很少發生；不為它新增資源類別，等真的有成本問題再處理。
- [跨平台] → 沒有檔案路徑或 git 的行為；網址解析不依賴平台。
- [既有的示範用 server 部署把 `events.heartbeat_secs` 設成 45 以上，升級後不啟動] → 訊息指出欄位與規則，改成小於 45 即可；部署文件與動詞契約文件寫明上限。這種設定原本就讓每個用戶端每 45 秒重連一次。
- [連線改用正規寫法後，宿主收到的請求網址與它宣告的字面不同] → 只有主機大小寫、預設埠與 `.`／`..` 的差別，HTTP 上是同一個資源；宣告的寫法已是正規寫法時逐字相同，測試固定這兩種情況。
- [連到只宣告輪詢的宿主時，Desktop 不再嘗試固定路徑] → 這是修正：少掉一直失敗的請求。事件管理器的輪詢路徑已有測試。
