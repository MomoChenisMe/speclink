## Context

儲存契約在 `crates/store/speclink-store-port`：一個同步、可跨執行緒共用的介面，三個操作（列出、讀取、提交），加上合格測試（16 項檢查，經一個測試用的接口接上任何實作）與記憶體版的參考實作。

舊的儲存層在 `crates/store/speclink-store`（TeamStore 介面）與三個 driver（`speclink-store-sqlite`、`speclink-store-postgres`、`speclink-store-fs`，各約 1300 到 1600 行）。它們解決過這個變更會再遇到的問題：SQLite 的忙碌逾時、PostgreSQL 的斷線重連與密碼來源、檔案系統的原子發布與檔案鎖、沒有資料庫時測試要回報略過。這些做法可以參考，但程式不能沿用：舊 driver 的每個操作都以 TeamStore 的型別為輸入輸出。

三個舊 driver 都把識別標記放在同一個位置：SQLite 與 PostgreSQL 是 meta 資料表的 `format` 鍵（值是 `speclink-team-store`），檔案系統是 meta.json 的 `format` 欄位（值是 `speclink-team-store-fs`）。

這個變更只新增三個 crate。沒有任何既有程式會呼叫它們；下一個變更（server-on-port-examples）才讓示範用 server 改用。

## Goals / Non-Goals

**Goals:**

- 三個範例各自通過合格測試，文件帶真實的最後修改時間，每個範圍有自己的版本號。
- 三個範例示範「事件和寫入同一次交易」，並提供讀事件、清事件、檢查可用性的方法，足夠示範用 server 之後拿來做推送事件的串流與健康檢查。
- 範例拒絕開啟它不認得的資料，並回報讀到的標記，讓呼叫端能給使用者明確的說明。
- 每個範例能被整個複製走：只相依儲存契約與它自己的資料庫函式庫。

**Non-Goals:**

- 不接線：不改 `speclink-server`、`speclink-host` 與任何測試以外的呼叫端。
- 不讀、不轉換舊 driver 的資料。
- 不做連線池、TLS、配置版本之間的遷移、保存每個版本的內容。
- 不改儲存契約與合格測試。

範圍內：三個新 crate 與它們的測試、workspace 的成員清單、CI 的 PostgreSQL job。範圍外：其餘全部。

## Decisions

### D1: 三個 crate 的位置、名稱與相依

- 位置：`crates/store/speclink-store-port-sqlite`、`crates/store/speclink-store-port-postgres`、`crates/store/speclink-store-port-fs`。名稱接在 `speclink-store-port` 後面，表示「儲存契約在某種儲存上的實作」。
- 三個都設 `publish = false`。每個 crate 的說明註解第一段寫明：這是範例；它示範什麼；它不做什麼（連線池、TLS、遷移、多實例的事件傳遞）。
- 相依：`speclink-store-port`、`serde_json`，加上各自的函式庫——rusqlite 0.40.1（bundled）、postgres 0.19.14、fs4 1.1.0。版本與舊 driver 相同，所以 Cargo.lock 不會多出新的第三方套件。
- 三個 crate 互不相依，也不相依 `speclink-store`。三份程式之間的重複（錯誤的型別、標記的檢查、事件方法的簽章）是刻意的：使用者會整個複製其中一個，不該被迫帶走一個共用的小 crate。

替代方案：放在最上層的 examples 資料夾。不採用——workspace 的 crate 都在 crates 底下依層分類，守門與建置腳本依這個配置找 crate；示範用 server 之後要把它們當一般相依使用。

### D2: 範例共同的公開介面

每個 crate 匯出一個儲存的型別（`SqlitePort`、`PostgresPort`、`FsPort`）與一個開啟失敗的型別（`OpenError`）。

- 開啟：`SqlitePort::open(path)`、`PostgresPort::connect(url)`、`FsPort::open(dir)`，回傳 `Result<Self, OpenError>`。
- 契約：實作 `StorePort`。
- 契約之外的四個方法（三個 crate 簽章相同）：
  - `events_after(&self, scope, after: u64, limit: usize) -> Result<Vec<(u64, RecordedEvent)>, PortError>`：序號大於 `after` 的事件，由小到大，最多 `limit` 筆。`RecordedEvent` 沿用儲存契約 crate 既有的型別（事件名稱、內容、範圍版本號）。
  - `event_floor(&self, scope) -> Result<u64, PortError>`
  - `trim_events(&self, scope, through: u64) -> Result<(), PortError>`
  - `probe(&self) -> Result<(), PortError>`
- `OpenError` 有三種：`UnknownLayout { format: Option<String>, layout_version: Option<String> }`（讀到的值原樣帶出）、`InUse`（只有檔案系統會回）、`Store(PortError)`。
- PostgreSQL 另外匯出 `PASSWORD_VAR`（值是 `SPECLINK_POSTGRES_PASSWORD`）與 `url_embeds_password(url) -> bool`，名稱與舊 driver 相同，之後示範用 server 換相依時不用改呼叫處。

這四個方法不放進一個共用的 trait：儲存契約維持三個函式；需要把三個範例當同一種東西用的是示範用 server，那個抽象由它在下一個變更自己定義。

### D3: 識別標記與配置版本

- 標記是 `speclink-store-port`，配置版本是 `1`。位置沿用舊 driver 的位置（meta 的 `format`），所以範例開到舊資料時讀得到舊標記，能原樣回報。這不需要範例知道任何舊配置的內容。
- 開啟的判斷順序：先看位置是不是空的；不是空的就讀標記；標記與版本都對才開啟。規則的完整內容在規格「開啟時只認自己的配置」。
- 初始化的原子性：SQLite 與 PostgreSQL 把建表與寫入標記放在同一個交易裡（兩者的建表都能回滾）。檔案系統先建好其他資料夾，最後才以「寫暫存檔、同步、改名」寫出 meta.json；沒有 meta.json 的半成品在下一次開啟時視情況處理——資料夾裡只有範例自己會建的空資料夾與鎖檔時當成空的重新初始化，有其他內容時拒絕。

### D4: SQLite 的配置與提交

四個資料表：

- `meta(key, value)`
- `scopes(scope, version, event_seq, event_floor)`，主鍵 scope
- `documents(scope, key, content, version, modified_at)`，主鍵 (scope, key)
- `events(scope, seq, version, name, payload)`，主鍵 (scope, seq)

連線：一個連線加一把互斥鎖；開啟時設 WAL 與 5 秒的忙碌逾時。

提交的步驟，全部在一個 `BEGIN IMMEDIATE` 交易裡：

1. 讀這個範圍的版本號與事件序號（沒有列就當成 0）。
2. 對每個操作檢查預期：讀該鍵目前的版本，和預期比。任何一個不符就回滾並回報衝突（帶該鍵、預期與實際）。
3. 新的版本號是舊的加 1（到上限時照契約拒絕）。套用寫入與刪除；寫入的文件記下新的版本號與提交帶的時間。
4. 依序寫入事件，序號從舊的事件序號加 1 開始。
5. 更新 scopes 的版本號與事件序號，提交交易。

列出用鍵的範圍比較找前綴（鍵大於等於前綴、小於前綴的上界），不用 LIKE 或 GLOB，避免鍵裡的萬用字元。鍵的欄位用預設的位元組比較，排序就是契約要的位元組順序。讀取與列出各在一個讀取交易裡完成，所以一次呼叫看到的是同一次提交之後的狀態。

SQLite 回報忙碌或鎖定時對映成「暫時無法使用」；其他錯誤對映成儲存的失敗。

### D5: PostgreSQL 的配置與提交

資料表與 SQLite 相同的四個，建在連線目前的 schema。差異：

- 鍵與範圍的欄位用 `text COLLATE "C"`，排序才是位元組順序（資料庫預設的排序規則依地區設定而不同）。
- 內容用 `bytea`：PostgreSQL 的 text 不能存 NUL 字元，而契約對內容沒有這個限制。
- 事件的內容存成 text（序列化後的 JSON 字串），不用 jsonb：jsonb 會改寫數字與鍵的順序。

一個範圍的提交之間用列鎖排隊：交易開頭先「不存在就插入」這個範圍的列，再以 `SELECT … FOR UPDATE` 鎖住它，之後的步驟與 D4 的 2 到 5 相同。不同範圍的提交互不等待。這取代舊 driver 的 advisory lock，而且對多個連線、多個程序都成立。

列出與讀取各在一個 REPEATABLE READ 的唯讀交易裡，確保版本號與文件來自同一個時點。

連線：一個連線加一把互斥鎖。呼叫遇到連線層的錯誤時，重新連線一次並重做該次呼叫；重做只適用於還沒送出提交的情況——提交的交易送出 COMMIT 之後才斷線時，不重做，回報暫時無法使用（呼叫端重試時，版本比對會讓重複的提交得到衝突，不會寫兩次）。密碼在建立時解析一次：環境變數優先於網址。組錯誤訊息時不放連線網址。

### D6: 檔案系統的配置與提交

資料夾的內容：

- `meta.json`：標記與配置版本。
- `lock`：檔案鎖。儲存物件存在的期間持有獨占的鎖（fs4 的非阻塞加鎖）；拿不到就回 `OpenError::InUse`。鎖由作業系統在程序結束時釋放，所以持有者不正常結束之後可以再開啟。
- `scopes/<跳脫後的範圍>/index.json`：這個範圍唯一的真相——版本號、事件序號、下限，以及每個鍵對應的內容檔名、文件版本與最後修改時間。
- `scopes/<跳脫後的範圍>/blobs/<流水號>`：內容檔，寫出後不再改。
- `scopes/<跳脫後的範圍>/events/<序號>.json`：一筆事件一個檔。

範圍的名稱跳脫成檔名：只保留小寫英文字母、數字、`-` 與 `_`，其餘位元組寫成 `%XX`。大寫也跳脫，避免不分大小寫的檔案系統把兩個範圍當成同一個；`.` 與 `/` 也跳脫，避免跳出資料夾。鍵不當成檔名，只存在 index.json 裡。

提交的步驟（持有程序內的互斥鎖）：

1. 讀 index.json，檢查每個操作的預期；不符就回報衝突，沒有寫出任何檔案。
2. 寫出新的內容檔與事件檔並同步。
3. 寫出 `index.json.new` 並同步，改名成 `index.json`。這一步是唯一的發布點。
4. 刪掉不再被索引引用的內容檔（盡力而為；失敗不影響結果）。

讀取端只相信 index.json：事件檔的序號大於索引的事件序號、或內容檔沒有被索引引用時，一律當成不存在。開啟時與每次提交開始時清掉這類遺留檔與 `index.json.new`。清事件（`trim_events`）先發布帶新下限的索引，再刪事件檔。

跨平台：改名覆蓋既有檔案在三個平台都由標準函式庫支援；同步資料夾本身只在非 Windows 做。路徑一律用 `Path::join` 組。

### D7: 事件記錄的語意

- 序號屬於範圍，從 1 開始，每筆加 1，和範圍的版本號是兩個獨立的計數（一次提交可以帶零到多筆事件）。
- 序號只在提交成功時前進。被拒絕的提交在交易回滾（或沒有發布索引）時一起消失。
- 下限只會前進。清到比目前最大序號還大的值時，下限停在目前最大序號——之後的事件接著編，不會跳號。
- `events_after` 的 `after` 小於下限時，從下限之後開始回傳；呼叫端比較 `after` 與 `event_floor` 就能知道中間有沒有被清掉的事件。
- 範例不自己清事件；什麼時候清由持有儲存的宿主決定。

### D8: 測試的編排

每個 crate 一個整合測試的執行檔（tests/it/main.rs 加模組），涵蓋規格的七條需求：

- 合格測試：實作測試用的接口（每次給一個新的範圍；事件由 `events_after` 讀回），斷言 16 項全部通過。
- 事件：規格「事件與寫入同一次交易並可依序號讀回」與「清除舊事件與下限」的每個情境。
- 配置：空的位置、重新開啟、別的標記（測試自己寫一個帶 `speclink-team-store` 標記的 meta，不相依舊 driver）、較新的版本、沒有標記；拒絕之後比對檔案或資料表沒有變動。
- 一致性：重新開啟後內容相同；交易中途失敗用「同一次提交的第二個操作預期不符」製造，斷言第一個操作沒有留下痕跡；檔案系統另外手動放遺留的內容檔、事件檔與 `index.json.new` 再開啟。
- 兩個儲存物件：SQLite 與 PostgreSQL 開兩個物件；檔案系統用子程序持有鎖，測第二個開啟失敗，以及子程序被強制結束後可以開啟。
- 可用性與失敗的分類：SQLite 與檔案系統在開啟後把檔案或資料夾移走；PostgreSQL 用伺服器端終止連線的函式製造斷線，並測錯誤訊息不含密碼。

不為測試在範例裡加故障注入的公開函式：範例是給人讀的，舊 driver 那種 `crash_at` 的接口不帶過來。

PostgreSQL 的測試沿用舊 driver 的做法：測試執行檔自己當主程式（Cargo.toml 設 `harness = false`），沒有設定 `SPECLINK_TEST_POSTGRES_URL` 時逐一印出測試名稱與 `skipped` 並以 0 結束；有設定時每個測試在自己的新 schema 裡執行，結束後刪除 schema。

### D9: CI

`.github/workflows/ci.yml` 的 PostgreSQL job 在既有的兩行測試之後加一行執行 `speclink-store-port-postgres` 的測試，輸出接到同一個記錄檔，所以既有的「輸出含 skipped 就失敗」守門涵蓋它。SQLite 與檔案系統的範例由既有的 workspace 測試在三個平台執行，不用改 CI。

## Implementation Contract

**行為**（完整條款在規格 store-port-examples）

- 三個範例各自通過合格測試的 16 項，沒有略過。
- 事件有每個範圍自己的序號，可依序號讀回、可清除，下限只前進。
- 開啟時：空的位置初始化；自己的標記開啟；其他一律拒絕且不改動資料，錯誤帶出讀到的標記與版本。
- 重新開啟後內容相同；中斷的提交全有或全無。
- SQLite 與 PostgreSQL 的兩個物件一起遵守版本比對；檔案系統拒絕第二個開啟者。

**介面**

- D2 列的型別與方法簽章。
- 資料表與檔案的配置如 D4、D5、D6。標記 `speclink-store-port`、配置版本 `1`。

**失敗的樣子**

- 版本不符：`PortError::Conflict`，帶鍵、預期與實際。
- 鎖逾時、連線中斷：`PortError::Unavailable`。
- 其他：`PortError::Backend`，訊息不含密碼與連線網址。
- 開啟：`OpenError` 的三種。

**驗收**

- `cargo test -p speclink-store-port-sqlite`、`cargo test -p speclink-store-port-fs` 全部通過。
- 沒有資料庫時 `cargo test -p speclink-store-port-postgres` 的輸出含 `skipped` 且 exit code 為 0；設定 `SPECLINK_TEST_POSTGRES_URL` 後全部通過且輸出不含 `skipped`。
- `cargo tree -p speclink-store-port-sqlite -i speclink-store` 回報找不到這個套件；另外兩個 crate 相同。
- `cargo build --workspace` 成功；Cargo.lock 沒有新增第三方套件（只多三個 workspace 成員）。
- `speclink-server`、`speclink-host` 與舊的 store crates 沒有任何檔案被改動。

**範圍邊界**

- 範圍內：三個新 crate、根目錄的 Cargo.toml 與 Cargo.lock、ci.yml 的一個 job。
- 範圍外：接線、舊 driver、儲存契約、Node SDK、使用者文件。

## Risks / Trade-offs

- [回歸對照] → 這個變更不改任何既有 crate，引擎的 golden 與 CLI 的整合測試不受影響。驗收條件明列「既有 crate 沒有檔案被改動」。
- [跨平台：檔案系統範例在 Windows 的鎖與改名] → 沿用舊 driver 已經在三個平台跑綠的做法（fs4 的鎖、改名覆蓋、Windows 不同步資料夾）；鎖被占用的錯誤碼各平台不同，判斷「使用中」用 fs4 回報的錯誤種類，不比對數字。測試在 CI 的三個平台都跑。
- [跨平台：PostgreSQL 只在 Linux 的 CI job 有資料庫] → 和舊 driver 相同；其他平台回報略過。守門確保 CI 上真的執行。
- [PostgreSQL 的排序規則] → 鍵的欄位明寫 `COLLATE "C"`；合格測試的「列出依位元組順序」會抓到漏寫。
- [COMMIT 之後斷線，呼叫端不知道有沒有寫入] → 不自動重做；回報暫時無法使用。呼叫端重試時由版本比對擋下重複寫入。這個限制寫進 crate 的說明。
- [三份程式重複] → 刻意的（D1）。代價是修一個問題要改三處；三個 crate 的測試各自涵蓋同一組行為，漏改會被抓到。
- [範例被當成正式環境的實作] → 每個 crate 的說明第一段寫明限制；文件在最後一個變更說明定位。

## Migration Plan

沒有要遷移的東西：這個變更只新增程式，不接線。回退方式是移除三個 crate 與 workspace 的成員。

## Open Questions

（無）
