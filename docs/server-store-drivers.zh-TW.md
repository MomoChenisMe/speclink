# 儲存接口的範例

**繁體中文** · [English](server-store-drivers.md)

儲存接口是宿主實作的三個函式：列出、讀取、提交。這個 repo 附了三個寫好的範例，分別把資料放在 SQLite、PostgreSQL 與檔案系統。三個範例都通過同一套合格測試。

這份文件給兩種讀者：

- **要把 Speclink 內嵌進自己系統的整合者**：找到和你同類的儲存，看它的資料表、一次提交怎麼寫、事件怎麼記，再照著寫自己的三個函式。內嵌的整體說明見[把 Speclink 內嵌進你的系統](../README.zh-TW.md#embedding)。
- **示範用 server 的管理者**：示範用 server 的儲存後端就是這三個範例之一。從[怎麼選](#choose)讀起。

範例是拿來讀、拿來照著寫的，不是給正式環境用的儲存。它們刻意不做的事列在[範例不做的事](#limits)。

## <a id="examples"></a>三個範例在哪裡

| 範例 | 位置 | 資料放哪 |
| --- | --- | --- |
| SQLite | [`crates/store/speclink-store-port-sqlite`](../crates/store/speclink-store-port-sqlite/src/lib.rs) | 一個 SQLite 檔 |
| PostgreSQL | [`crates/store/speclink-store-port-postgres`](../crates/store/speclink-store-port-postgres/src/lib.rs) | PostgreSQL 的一個 schema |
| 檔案系統 | [`crates/store/speclink-store-port-fs`](../crates/store/speclink-store-port-fs/src/lib.rs) | 一個資料夾 |

每個範例的程式都在一個檔案 `src/lib.rs` 裡，開頭的說明寫了配置與提交的步驟；測試在 `tests/it/`。

契約本身在 [`crates/store/speclink-store-port`](../crates/store/speclink-store-port/src/lib.rs)：三個操作的定義、文件種類與鍵的對照（`src/keys.rs`）、合格測試（`src/conformance/`），以及一個只把資料放在記憶體的實作，給測試用。

合格測試共 16 項檢查。三個範例都是 16 項通過、沒有任何一項略過；怎麼把一個實作接上合格測試，看各範例的 `tests/it/conformance.rs`。

用 Node.js 內嵌的宿主，三個函式是用 JavaScript 寫的：

- 輸入、輸出與三項保證見 Node SDK 文件的[實作三個儲存函式](sdk-node.zh-TW.md#store-port)。
- 寫完之後照[用合格測試檢查你的實作](sdk-node.zh-TW.md#conformance)跑同一套檢查。
- 目前沒有 JavaScript 寫的範例。這三個 Rust 範例可以當成資料表與交易寫法的參考。這一項列在專案能力狀態的[已知缺口](product-status.zh-TW.md#known-gaps)。

這三個範例同時是示範用 server 的儲存：示範用 server 啟動時，依組態開啟其中一個。

## <a id="choose"></a>怎麼選

這一節給示範用 server 的管理者。先知道三件事：

- 帳號與成員資格放在另一個 SQLite 檔（組態檔的 `identity` 段），不跟著儲存後端換。
- 三個範例通過同一套合格測試，所以 API 與 CLI 的行為相同。換儲存後端不影響使用方式。
- 不論選哪一種，示範用 server 都只支援**一個**實例。

| 儲存後端 | 資料放哪 | 適合 | 要注意 |
| --- | --- | --- | --- |
| `sqlite` | 一個資料庫檔 | 大多數情況。npx 與 Docker 映像預設用它 | 放在本機磁碟。第二個實例開同一個檔案不會被擋下 |
| `serverfs` | 一個資料夾 | 想用純檔案保存資料、不想維運資料庫 | 資料夾所在的檔案系統要支援檔案鎖 |
| `postgres` | PostgreSQL 的一個 schema | 已經有 PostgreSQL 維運與備份 | 連線不加密（沒有 TLS）。第二個實例連同一個 schema 不會被擋下 |

設定寫在組態檔的 `store` 段。`sqlite` 與 `serverfs` 用 `path`，`postgres` 用 `url`：

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
```

```yaml
store:
  driver: serverfs
  path: /var/lib/speclink/store     # 這是資料夾，不是檔案
```

```yaml
store:
  driver: postgres
  url: postgres://speclink@db.internal:5432/speclink   # 不放密碼，見下方
```

- **PostgreSQL 的密碼**放在環境變數 `SPECLINK_POSTGRES_PASSWORD`。這個變數有值時，server 用它連線，連線 URL 裡的密碼不會被使用。URL 裡直接寫密碼也能啟動，但 server 啟動時會在 stderr 印一行警告：組態檔常被複製、比對、貼進 issue，不該放密碼。錯誤訊息不會印出連線 URL 與密碼。
- **npx** 用環境變數 `SPECLINK_STORE` 選儲存後端（`sqlite`、`serverfs` 或 `postgres`，預設 `sqlite`）。資料放在 `SPECLINK_DATA_DIR` 底下：`sqlite` 是 `store.db`，`serverfs` 是 `store` 資料夾。選 `postgres` 時要另外給 `SPECLINK_POSTGRES_URL`。
- **`driver` 寫了不認得的名稱**，或少了 `path`、`url`，server 不會啟動，錯誤訊息會指出組態檔與原因。拼錯 `serverfs` 不會默默改用別的儲存後端。
- **`memory`** 是契約裡那個記憶體版的實作，只給測試用的組態。程序結束資料就消失，npx 不接受它。

完整的組態欄位、環境變數與各種跑法見 [Server 部署](server-deployment.zh-TW.md#configuration)。

## <a id="layout"></a>資料表與檔案配置

三個範例存的是同一種東西：

- **範圍（scope）**：一組互不相干的文件。示範用 server 的一個範圍是一組專案與儲存庫，名稱寫成 `<專案代號>/<儲存庫代號>`，例如 `demo/backend`。每個範圍有自己的版本號，從 0 開始，每次成功的提交加 1。
- **文件**：一個鍵對一份內容。鍵是字串，和本機專案規格資料夾底下的相對路徑相同，例如 `changes/<變更名稱>/proposal.md`、`specs/<capability>/spec.md`、`discussions/<slug>.md`、`config.yaml`、`LANGUAGE.md`。兩個例外：看板順序的鍵是 `board/order`；封存的變更與討論放在 `archive/changes/` 與 `archive/discussions/` 底下。每份文件記著最後寫入它的那次提交的範圍版本號與時間。
- **事件**：每次提交附帶的事件，見[事件記錄與序號](#events)。

每個範例都在固定的位置寫一個標記 `format`，值是 `speclink-store-port`，以及配置版本 `layout_version`，值是 `1`。開啟時只認這一組：

- 位置是空的：建立配置。
- 標記與配置版本都對：照原樣開啟，不改既有的資料。
- 其他情況：拒絕開啟，不建立也不改動任何資料表與檔案。錯誤會帶出讀到的標記與配置版本。

示範用 server 遇到拒絕開啟時不會啟動。資料是 `speclink-server` 0.12 以前的版本寫的時候，錯誤訊息會列出搬資料的三個步驟，見 [Server 部署的升級一節](server-deployment.zh-TW.md#upgrade)。

SQLite 與 PostgreSQL 兩個範例用同樣的四張資料表：

| 資料表 | 欄位 | 放什麼 |
| --- | --- | --- |
| `meta` | `key`、`value` | 兩列：`format` 與 `layout_version` |
| `scopes` | `scope`、`version`、`event_seq`、`event_floor` | 每個範圍一列：範圍版本號、最後一筆事件的序號、事件的下限 |
| `documents` | `scope`、`key`、`content`、`version`、`modified_at` | 每份文件一列，主鍵是 `scope` 加 `key`。`modified_at` 是毫秒（Unix 時間） |
| `events` | `scope`、`seq`、`version`、`name`、`payload` | 每筆事件一列，主鍵是 `scope` 加 `seq`。`version` 是那次提交之後的範圍版本號 |

### <a id="sqlite"></a>SQLite

- 文字欄位是 `TEXT`，數字欄位是 `INTEGER`。事件的內容 `payload` 以 JSON 文字存在 `TEXT` 欄位。
- 檔案不存在，或檔案裡沒有任何資料表時，範例建立四張資料表。檔案裡有資料表但沒有 `meta` 時，拒絕開啟。
- 資料庫用 SQLite 的 WAL 模式，所以資料庫檔旁邊會多出檔名結尾是 `-wal` 與 `-shm` 的工作檔。搬移或複製資料時要一起處理，或者直接用 [`backup`](server-backup.zh-TW.md)。
- 請放在本機磁碟。WAL 模式不支援 NFS、SMB 這類網路檔案系統。
- 一個儲存物件只有一條連線。等不到寫入鎖超過 5 秒時，回報「暫時無法使用」，不是衝突。
- 兩個儲存物件可以開同一個檔案，版本比對仍然正確：SQLite 自己的鎖讓兩邊的提交輪流進行。所以示範用 server **不會**擋第二個指向同一個檔案的實例，要靠你自己避免。
- 可用性檢查會確認檔案還在原本的路徑，而且寫得進去。檔案被搬走或刪掉時，示範用 server 的 `/readyz` 回 `503`。

### <a id="postgres"></a>PostgreSQL

- 四張資料表建在連線**目前的 schema**。在連線字串設定 `search_path`，就能選 schema，也就能讓多份儲存共用一個資料庫。
- schema 裡沒有 `meta` 資料表時，範例建立四張資料表；schema 裡原有的其他資料表不受影響。已經有一張同名的資料表（例如 `documents`）但沒有 `meta` 時，開啟失敗，什麼都不建立。
- `scope` 與 `key` 兩欄是 `text COLLATE "C"`：契約規定鍵依位元組排序，而資料庫預設的排序跟著地區設定走。
- `content` 是 `bytea`：`text` 放不了 NUL 字元，而契約允許文件內容含有它。
- `payload` 是放 JSON 文字的 `text`，不用 `jsonb`：`jsonb` 會改掉數字的寫法與鍵的順序。
- 數字欄位都是 `bigint`。
- 一個儲存物件只有一條連線，沒有連線池。連線斷了，下一次呼叫會重新連線一次，連上就照常執行。資料庫暫時連不上時，示範用 server 的請求回 `unavailable`（HTTP 503），`/readyz` 回 `503`；連線恢復後同一個 server 直接續用，不需要重啟。
- 有一種情況不會重跑：`COMMIT` 已經送出、連線卻在回應之前斷掉。範例無法知道資料庫有沒有套用，所以回報「暫時無法使用」。呼叫端重試時，已經套用過的寫入會被版本比對擋下，不會寫兩次。
- 連線不加密，沒有 TLS。只接受 TLS 的託管 PostgreSQL 會連不上；請把資料庫放在受信任的內部網路。
- 兩個儲存物件可以連到同一個 schema，版本比對仍然正確。所以示範用 server **不會**擋第二個連到同一個 schema 的實例。
- CI 用 PostgreSQL 15 跑這個範例的測試。

### <a id="serverfs"></a>檔案系統

示範用 server 的組態把這個範例叫做 `serverfs`。資料夾裡的檔案如下：

```text
<資料夾>/
├── meta.json
├── lock
└── scopes/
    └── <範圍的資料夾名稱>/
        ├── index.json
        ├── blobs/<編號>
        └── events/<序號>.json
```

| 檔案 | 放什麼 |
| --- | --- |
| `meta.json` | `format` 與 `layout_version` 兩個欄位 |
| `lock` | 鎖檔。儲存物件存活期間一直持有它的獨占鎖 |
| `scopes/<範圍>/index.json` | 這個範圍唯一的依據。欄位是 `version`、`event_seq`、`event_floor`、`next_blob` 與 `documents`；`documents` 裡每個鍵記著 `blob`（內容檔的編號）、`version` 與 `modified_at` |
| `scopes/<範圍>/blobs/<編號>` | 一份文件的內容。寫好之後不再改動 |
| `scopes/<範圍>/events/<序號>.json` | 一筆事件，欄位是 `name`、`payload` 與 `version` |

- **範圍的資料夾名稱**：小寫英文字母、數字、`-` 與 `_` 照原樣，其他每個位元組寫成 `%XX`。範圍 `demo/backend` 的資料夾是 `demo%2Fbackend`。文件的鍵不會變成檔名。
- **讀取只相信 `index.json`**。`index.json` 沒有提到的內容檔，以及序號比 `index.json` 記的還大的事件檔，對讀取來說不存在；每次開啟與每次提交都會清掉這類檔案。
- **沒有舊版本的歷史**。一份文件被改寫或刪除後，舊的內容檔在那次提交的最後就刪掉。
- **檔案鎖**：鎖由作業系統持有。程序結束（包含被強制結束、當掉）時鎖會自動釋放，下一次開啟可以直接接手，`lock` 這個檔案留著也沒關係。第二個儲存物件開同一個資料夾時，馬上失敗並回報資料夾使用中，不等待也不搶鎖；示範用 server 因此啟動失敗。
- 部署前請確認資料夾所在的檔案系統支援檔案鎖。請用本機磁碟；NFS 等網路檔案系統的檔案鎖在部分設定下不可靠。
- 檔案系統上的修改時間沒有意義。文件的最後修改時間記在 `index.json`。
- 過程中會短暫出現 `index.json.new`、`meta.json.new` 與 `probe.tmp` 三個暫存檔名。
- 範圍名稱很長時，換成資料夾名稱後最多是三倍長，可能超過檔案系統對單一名稱的長度上限，這時提交會失敗。在 Windows 上，名稱是 `con`、`nul` 這類裝置名的範圍可能建不出資料夾。
- 資料夾的格式是這個範例自己的。不要手動編輯；要人看得懂的匯出，用 [`backup`](server-backup.zh-TW.md)。

## <a id="commit"></a>一次提交的步驟

引擎執行一個動詞時，只向儲存要它用到的文件：同一次執行裡，同一個前綴最多列出一次，同一個鍵最多讀取一次，需要多份文件時用一次讀取取得。它不會把整個範圍載入記憶體。動詞的寫入先暫存；動詞成功後，全部的寫入、刪除與事件收成**一次提交**交給儲存。查詢不提交。

提交裡每一筆寫入都帶著預期：這份文件「必須不存在」，或「必須在某個版本」。每一筆刪除也帶著預期的版本。儲存要做到三項保證：

1. 一次提交的操作全部成功，或全部不做。
2. 有一筆預期不符，整次提交就被拒絕，並回報衝突：哪一個鍵、呼叫端的預期、文件目前的版本。被拒絕的提交不改任何文件、不動範圍版本號、不留事件。
3. 事件和寫入在同一次提交裡。

PostgreSQL 的範例把一次提交寫成一次交易。下面是它依序執行的 SQL（`BEGIN` 與 `COMMIT` 代表交易的開始與提交）：

```sql
BEGIN;

-- 1. 範圍那一列：沒有就先建立，再鎖住
INSERT INTO scopes (scope, version, event_seq, event_floor)
VALUES ($1, 0, 0, 0) ON CONFLICT (scope) DO NOTHING;
SELECT version, event_seq FROM scopes WHERE scope = $1 FOR UPDATE;

-- 2. 每一筆操作各查一次目前的版本，全部比對完才開始寫
SELECT version FROM documents WHERE scope = $1 AND key = $2;

-- 3. 寫入與刪除；寫入的文件版本是新的範圍版本號
INSERT INTO documents (scope, key, content, version, modified_at)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (scope, key) DO UPDATE SET
    content = excluded.content,
    version = excluded.version,
    modified_at = excluded.modified_at;
DELETE FROM documents WHERE scope = $1 AND key = $2;

-- 4. 事件，序號接著 event_seq 往上編
INSERT INTO events (scope, seq, version, name, payload)
VALUES ($1, $2, $3, $4, $5);

-- 5. 更新範圍那一列，然後提交
UPDATE scopes SET version = $2, event_seq = $3 WHERE scope = $1;
COMMIT;
```

每一步在做什麼：

1. **鎖住範圍那一列。** `SELECT … FOR UPDATE` 讓同一個範圍的提交輪流進行，不論它們來自哪一條連線、哪一個程序。不同範圍的提交互不等待。等鎖超過 5 秒（`lock_timeout`）就回報「暫時無法使用」。
2. **先比對版本，再寫。** 每一筆操作的預期都在第一筆寫入之前比對完。有一筆不符就回滾交易並回報衝突，所以不會出現只寫了一半的提交。不要一筆一筆地邊比對邊寫。
3. **寫入與刪除。** 新的範圍版本號是原本的版本號加 1。這次寫入的每份文件，`version` 都是這個新的版本號，`modified_at` 是提交帶來的時間。
4. **事件寫在同一次交易。** 交易沒有提交，事件就不存在；被拒絕的提交不會用掉序號。
5. **更新範圍並提交。** `COMMIT` 是唯一的生效點。回傳值是新的範圍版本號。

列出與讀取各自在一次唯讀的交易裡執行（`REPEATABLE READ`），所以一次列出或一次讀取的結果都來自同一次提交，不會混到兩次提交的內容。

另外兩個範例做的是同一件事：

- **SQLite**：整次提交是一次 `BEGIN IMMEDIATE` 交易。它一開始就拿到寫入鎖，所以比對完的版本在寫入之前不會被別人改掉。步驟的順序與上面相同。
- **檔案系統**：先讀 `index.json` 比對全部的預期；再寫出新的內容檔與事件檔並同步到磁碟；然後寫出 `index.json.new`、同步，把它改名成 `index.json`。這一次改名是唯一的生效點：改名之前中斷，讀取看到的是提交之前的狀態。改名之後才刪掉被取代的內容檔。有一種情況沒有確定的答案：改名之後同步資料夾失敗，這時提交已經看得到，但斷電後不保證還在。範例回報儲存失敗；呼叫端要重新列出範圍，再決定要不要重送。

## <a id="events"></a>事件記錄與序號

每次提交可以附帶事件（例如某個產物被寫入、某個變更被封存）。三個範例都把事件記進自己的事件記錄：

- 每個範圍的事件有自己的**序號**，從 1 開始，每筆加 1。同一次提交的事件依它們在提交裡的順序連續編號。
- 每筆事件記著名稱、內容，以及那次提交之後的範圍版本號。
- 被拒絕的提交不留事件，也不用掉序號。
- 只帶事件、沒有寫入的提交同樣有效，也會讓範圍版本號前進。

契約的三個操作沒有「把事件讀回來」這一項：儲存要不要保留事件，由宿主決定。範例在契約之外多提供四個方法，給擁有這個儲存的宿主用：

| 方法 | 做什麼 |
| --- | --- |
| `events_after(scope, after, limit)` | 回傳序號大於 `after` 的事件，由小到大，最多 `limit` 筆，每筆帶著序號 |
| `event_floor(scope)` | 回傳下限：已經被清掉的最大序號。還沒清過任何事件時是 0 |
| `trim_events(scope, through)` | 清掉序號小於或等於 `through` 的事件。最多清到目前最大的序號；之後的事件接著原本的序號往上編，不重用被清掉的序號。不改任何文件與版本號 |
| `probe()` | 檢查儲存是否可用：讀得到，也寫得進去 |

範例不會自己清事件；不呼叫 `trim_events`，事件就一直留著。要知道有沒有漏掉事件，拿手上的序號和下限比：手上的序號比下限小，代表中間有事件已經被清掉。

示範用 server 是這樣用這四個方法的：

- 事件流 `GET /events`（見動詞與旗標文件的[事件流](verb-contract.zh-TW.md#events)）用 `events_after` 讀事件。每筆推送的事件編號就是序號，用戶端斷線重連時帶上最後收到的編號，server 從那之後補送。
- 推送之後，server 用 `trim_events` 清掉較舊的事件，每個範圍保留最近的 `events.retention` 筆（預設 1024，在組態檔設定）。
- `/readyz` 與後台的「系統」頁用 `probe` 判斷儲存後端是否可用。

需要「至少送達一次」的宿主，做法和範例相同：在提交的同一次交易裡把事件寫進自己的事件表，再由自己的程式從那張表送出。見 Node SDK 文件的[一個動詞的四個步驟](sdk-node.zh-TW.md#four-steps)。

## <a id="limits"></a>範例不做的事

三個範例都刻意不做下面這些事。照著範例寫自己的儲存時，請依你的環境補上：

- **連線池。** 一個儲存物件只有一條連線，全部的呼叫輪流使用它。
- **TLS。** PostgreSQL 的範例一律用不加密的連線。
- **配置版本之間的遷移。** 範例只開啟配置版本 `1` 的資料，遇到別的版本就拒絕開啟，不會自己搬資料。
- **多個實例之間的事件傳遞。** 一個實例提交之後，沒有任何機制通知另一個實例有新的事件。
- **舊版本文件的歷史。** 文件被改寫之後，舊的內容不保留。
- **自動清除事件。** 清不清、留多少，由宿主決定。

因為這些限制，示範用 server 只支援一個實例：不要 `--scale`，也不要讓兩個 server 指向同一份資料。`serverfs` 會擋下第二個實例；`sqlite` 與 `postgres` 不會。兩個實例開同一份資料時，資料不會壞，但一個實例提交之後另一個實例不會被通知，連在它上面的用戶端就不會即時收到這次更新。

## <a id="switch"></a>換一種儲存

三個範例的配置各不相同，不能直接搬檔案或複製資料表。換儲存後端要經過備份檔：

1. 停掉 server，用舊的組態執行 `speclink-server backup --config <舊組態> --output <備份檔>`。
2. 準備新的組態：`store` 指向一個空的位置，`identity` 指向一個新的、空的帳號資料庫。
3. 用新的組態執行 `speclink-server restore --config <新組態> --input <備份檔>`，再用新的組態啟動 server。

備份檔的內容與儲存後端無關，`restore` 會逐項核對還原的結果。指令的細節與「只能還原到空的目標」的條件見 [Server 備份與還原](server-backup.zh-TW.md)。
