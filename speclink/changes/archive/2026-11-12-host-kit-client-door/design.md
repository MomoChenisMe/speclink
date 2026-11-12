## Context

這個變更開始時的狀態（前面的變更完成後）：

- `speclink_host::kit::HostKit`：建立時收儲存接口與掛鉤；`run(caller, cmd)` 執行一個引擎命令，執行前問掛鉤、提交後通知掛鉤。身分是識別加顯示名稱。
- `speclink_host::documents`：不經引擎的文件讀寫函式，回傳線上的型別與範圍版本號，不依賴 HTTP 框架。
- 示範 server 的 `api/routes.rs` 仍然擁有每條路由的處理函式：用 axum 取出參數、建出 `Command`、執行、把結果轉成 `speclink-protocol` 的型別、設 `ETag`。錯誤的對映在 `api/error.rs`。
- 示範 server 的身分與範圍由 `Binding` 這個 axum extractor 決定，依序檢查：bearer token → 專案存在 → 是成員 → API 版本標頭 → repo 標頭（或唯一的 repo）。寫入的請求另外由一層 middleware 要求編輯者角色。能改流程政策等於是編輯者。
- 握手是專案底下的 `binding` 路徑，回應有操作者、專案、repo、API 版本、引擎版本與 `capabilities`（含更新方式的宣告：推送的 transports 與輪詢）。
- 推送事件的串流讀 TeamStore 的 outbox；儲存接口沒有讀回事件的操作。
- `speclink-protocol` 已有登入方式宣告的型別（client-browser-login）。

限制：

- `speclink-host` 不相依 axum 與 tokio；async 的邊界只在 server。
- 原版用戶端的網址是固定的：API 在 `/api/speclink/v1/`，裝置登入的端點在網站根目錄。
- 每個變更完成後示範 server 與既有測試都要可用。

## Goals / Non-Goals

**Goals:**

- 宿主只寫一條路由就能服務原版的 CLI 與 Speclink Desktop。
- 示範 server 的專案路由由同一個入口處理，既有的整合測試不改斷言就通過。
- 請求轉命令、結果轉線上格式、錯誤對映只有一份，在 `speclink-host`。
- 引擎新增命令時，不可能忘記決定它要不要上線。

**Non-Goals:**

- 登入、帳號、管理介面、備份、匯入、串流本身。
- 改任何既有路由的網址或格式。
- 可設定的網址前綴。

## Decisions

### D1: 入口的形狀

`speclink_host::http` 新增和框架無關的型別與一個方法：

```rust
pub struct HttpRequest { pub method: Method, pub path: String, pub query: Option<String>, pub headers: Vec<(String, String)>, pub body: Vec<u8> }
pub struct HttpResponse { pub status: u16, pub headers: Vec<(String, String)>, pub body: Vec<u8> }
impl HostKit { pub fn handle(&self, request: &HttpRequest, identity: Option<&Identity>) -> Option<HttpResponse> }
```

- `Method` 是 GET、HEAD、POST、PUT、DELETE 五種；其他方法由呼叫端回 405。示範 server 的轉交處理函式與 Node 的 `handleRequest` 都這樣做，回應帶 `allow` 標頭。
- 回傳「沒有」代表這不是入口的路徑，呼叫端可以交給自己的其他路由。這讓示範 server 把匯入與串流留在自己手上，也讓宿主在同一個前綴底下放自己的東西。
- 入口認的路徑：`/api/speclink/v1/auth-methods`、`/api/speclink/v1/whoami`、`/api/speclink/v1/scopes`，以及 `/api/speclink/v1/projects/{key}/` 底下它有對照的路徑。
- 方法是同步的。示範 server 在 blocking 的執行緒池呼叫它；Node 在背景執行緒呼叫它。
- 標頭名稱不分大小寫比對。回應的標頭只有 `content-type` 與（有的時候）`etag`。

路由的比對用一張靜態的表：方法、路徑的樣式、處理函式。不引入路由的函式庫：樣式只有固定段與具名段兩種，加上少數「吃掉其餘路徑」的段（artifact 的路徑）。

- **否決：入口直接提供 axum 的 router** — Node 的宿主用不了，而且 Host 會相依 axum。
- **否決：入口自己從 `Authorization` 標頭驗 token** — 驗 token 是宿主的事。

### D2: 建立時新增的輸入

`HostKit` 的建立改成收一個選項的結構，欄位都有預設值，既有的 `HostKit::new(store, hooks)` 保留：

| 欄位 | 型別 | 預設 | 用途 |
|---|---|---|---|
| `scopes` | 函式：身分 → 專案的清單 | 沒有 | 範圍清單、把網址裡的專案與 repo 對到範圍識別、能力開關 |
| `auth_methods` | 登入方式宣告的型別 | 沒有 | 回答宣告的路徑 |
| `push_transports` | 推送宣告的清單 | 空的 | 握手的更新方式宣告 |

`scopes` 回傳的形狀：

```rust
pub struct ProjectEntry { pub key: String, pub name: String, pub repos: Vec<RepoEntry> }
pub struct RepoEntry { pub key: String, pub name: String, pub scope: ScopeId, pub can_write: bool, pub can_write_policy: bool }
```

它可以失敗（回傳錯誤訊息），入口把失敗當成儲存層的「暫時無法使用」回應。入口每個請求呼叫它一次，不快取；要快取由宿主自己做，因為只有宿主知道成員關係什麼時候變。

沒有提供 `scopes` 時：`scopes` 的路徑與全部專案底下的路徑回 404。也就是宿主只用 Agent 工具的門時，不會意外對外開放。

### D3: 一個請求的處理順序

對專案底下的路徑：

1. 比對路由表。沒有對照 → 回傳「沒有」（不是入口的路徑）。這一步排在最前面（實作時調整）：不認得的路徑不論有沒有身分都交還宿主，也不會呼叫 `scopes`；和今天示範 server 對不認得的路徑一律回 404 相同。
2. 沒有身分 → 401 `permission_denied`。
3. `x-speclink-api-version` 不等於目前的 API 版本 → 409 `refused`（訊息與今天相同）。
4. 呼叫 `scopes`，找網址裡的專案：找不到 → 404 `not_found`。
5. 決定 repo：有 `x-speclink-repo` 標頭就用它，清單裡沒有 → 404；沒有標頭而專案只有一個 repo 就用它；沒有標頭而有多個 → 409（既有的多義訊息，列出候選）。
6. 方法不是 GET 或 HEAD，而這個 repo 的 `can_write` 是假 → 403 `permission_denied`，訊息沿用今天 reader 收到的那一句（server-verb-api 的情境逐字規定它）。例外是 Context 快照（它是唯讀的 POST），和今天相同。
7. 執行處理函式。

路徑比對的規則與今天 axum 的結果相同：在原始（未解碼）的路徑上比對，所以 `%2F` 留在它的段裡；具名段與「吃掉其餘路徑」的段都不能是空的；兩個樣式都符合時，先出現固定段的那個勝出，它沒有這個方法時就是沒有對照，不退回另一個樣式；HEAD 用 GET 的路由。已知路徑配上它沒有的方法（例如 `POST /changes/{name}/drift`）是「沒有對照」，示範 server 因此回 404，不再是今天的 403／405。`speclink_host::http::match_route` 對外提供這個比對，讓宿主在讀本文之前分辨入口的路徑（示範 server 用它決定本文上限）。

和今天示範 server 的順序差在第 3 步：今天「專案存在但不是成員」是 403，入口分不出這兩種情況，一律 404。示範 server 要維持 403，做法見 D8。

API 根的三條路徑：`auth-methods` 不檢查身分與版本；`whoami` 與 `scopes` 需要身分，不需要版本標頭（和今天的 `scopes` 相同）。

### D4: 路由的處理函式

從示範 server 搬到 `http/routes.rs`，每個函式的形狀是「收已解析的路徑參數、查詢、本文與呼叫者，回傳線上的型別或錯誤」：

- **執行引擎命令的路由**：建出 `Command` → `HostKit::run` → 把結果轉成線上的型別（轉換函式搬到 `http/wire.rs`）。結果的種類和預期不同時是內部錯誤，和今天相同。
- **不經引擎的路由**：呼叫 `documents` 的函式。看板順序的寫入先問執行前掛鉤（命令名稱 `put-board-order`、參數是空物件），成功後通知提交後掛鉤。
- **帶預期文件版本的 artifact 寫入**：照今天的順序——讀文件版本、比對、執行命令、回傳新的文件版本。命令自己會再讀一次這份文件，並以它讀到的版本當提交的期望值；今天示範 server 靠「每個請求只載入一次」讓兩次讀到同一份，宿主自己的儲存沒有這個保證。所以入口在提交前多守一道（`port_exec::WriteAt`）：命令對這份文件的寫入，期望的版本必須等於用戶端 `If-Match` 指的那一個，否則是 409 `revision_conflict`、不提交。這一道不用多讀儲存，任何儲存都成立；`If-Match: 0`（只能新建）同樣守住。
- **握手**：操作者是身分的識別與顯示用名稱（見下方「顯示用名稱」）；專案與 repo 的識別沿用 `prj_<key>`、`repo_<key>`；`capabilities` 裡依能力開關填的欄位用 `can_write` 與 `can_write_policy`，其餘固定為真的欄位不變；更新方式的宣告是 `push_transports` 加上輪詢。
- **輪詢**：`documents::scope_version` 與 `If-None-Match` 比對，相同回 304。
- **專案底下的 whoami**：使用者取自身分，repo 清單取自 `scopes` 的結果。
- **API 根的 whoami**：只有使用者。
- **範圍清單**：`scopes` 的結果轉成既有的回應型別。
- **登入方式的宣告**：有設定就回它，沒有就 404。

每個成功的回應帶 `ETag`，值是範圍版本號加引號，和今天相同；例外也和今天相同：握手與 API 根的三條路徑（`auth-methods`、`whoami`、`scopes`）不帶 `ETag`。命令有提交時，值是提交落在的版本；沒有提交時，值是「命令開始讀取之前」取的範圍版本（問完執行前掛鉤之後才取，所以被拒絕的請求仍然連讀取都沒有）。這樣 `ETag` 不會比回應的內容新——今天示範 server 靠同一份載入做到，入口不能假設儲存有這個保證。`GET /plan` 的 `ETag` 是讀看板順序時的版本（第一次讀取）。

**顯示用名稱（實作時新增）**：示範 server 今天蓋章用「名稱 <email>」，握手與身分查詢卻顯示使用者的 display，兩者不同；入口的身分原本只有一個顯示名稱，無法同時滿足。`Identity` 因此多一個選填的顯示用名稱（`Identity::shown_as`）：握手的操作者名稱與兩個身分查詢的使用者名稱用它，沒有給時用顯示名稱；它不進提交記錄、也不進蓋章值。Node 的 `HostIdentity` 不變（不給就沿用顯示名稱）。host-kit 的「每次呼叫由宿主給身分與範圍」因此以 MODIFIED 改寫。

**completion evidence 的 repo**：經入口執行的命令把 repo 的 key 交給引擎（`RunOptions.repo`），勾選任務記下的證據照舊帶 `repo`，和今天示範 server 相同；Agent 工具的門不帶。

錯誤的對映搬到 `http/error.rs`：引擎的錯誤碼、儲存的失敗分類、`documents` 的錯誤，各自對到狀態碼與線上的 reason，表和今天 `api/error.rs` 的相同。示範 server 的 `api/error.rs` 只留它自己的路由需要的部分（身分、管理）。

品質關卡第 1 輪之後的調整（審查站指出入口與示範 server 各有一份相同的程式）：

- **錯誤信封只有一套**：`speclink_host::http::HttpError` 對外公開。示範 server 的 `ApiError` 只是包住它、讓它成為 axum 的回應，並保留 server 獨有的兩種對映（TeamStore 的六類失敗、身分儲存的失敗）；不再有第二套同名的建構函式。
- **前置檢查只有一份**：API 版本的檢查（`check_api_version`）、repo 的判定（`choose_repo`，仍由 host-runtime 的 `resolve_binding` 決定「多個候選不自動選」）、沒有寫入能力的 403（`write_refused`）由 `speclink_host::http` 提供，入口、示範 server 留下的 `Binding`（匯入與事件串流）與管理 API 共用。
- **兩個品質關卡的路由合成一組**：review 與 verify 的 show、add-round、stamp、discard 只差命令與結果的變體，改由一個 `Station` 型別實作，八個路由各自只指定站別。這是對「處理函式原樣搬移」的唯一改寫，回應不變，由既有的 `review_api`、`verify_api` 整合測試與 `remote_verb_parity` 守住。
- **討論寫入的拒絕改用型別判斷**：原本從示範 server 搬來的 `refine_discussion_write` 用訊息的字串判斷狀態碼。現在引擎（`speclink-core` 的 `discuss`）對 `new`、`link`、`seal`、`discard` 的拒絕帶上 `WriteRefusal` 與它的種類（找不到、不合格、守衛）；命令層仍把它歸成一般錯誤、訊息逐字不變，所以 CLI 的輸出與結束碼不變。入口依種類回 404／400／409。
- **執行前掛鉤「正好問一次」**：`PUT /config` 的預期版本一開始就過期時，引擎沒有執行、檢查點沒有跑到；這種情況也補問一次掛鉤（變動的鍵是空的），拒絕優先於版本衝突。
- 只剩測試在用的 `port_exec::execute_at_version` 移除（它唯一的正式呼叫端是搬走的設定寫入）；宿主套件改用內部的 `at_version`。

### D5: 能力開關的把關

- `can_write` 是假：D3 的第 6 步。
- `can_write_policy` 是假：流程設定的寫入在提交前檢查變動的鍵，含任何政策鍵（`schema`、`locale`、`spec_locale`、`tdd`、`audit`、`worktree`）時以 403 拒絕，儲存不變。只改 context、rules、審查規則、子代理模型的寫入不受影響。這個檢查和執行前掛鉤用同一個時間點（引擎算出結果之後、提交之前），先檢查能力、再問掛鉤。

示範 server 兩個開關都等於「是不是編輯者」，所以行為和今天相同。

### D6: 每個引擎命令的線上歸屬

`http/coverage.rs`：

```rust
pub enum WireCoverage { Route { method: Method, pattern: &'static str }, NotOnWire { reason: &'static str } }
pub fn wire_coverage(command: &Command) -> WireCoverage
```

用不含萬用分支的 `match` 寫，所以引擎新增一個命令變體時這裡編譯不過，逼出一個決定。

兩個測試把它和路由表綁在一起：

- 每個 `Route` 都在路由表裡，而且該路由的處理函式建出的命令就是這一種。
- 路由表裡每條執行引擎命令的路由都被某個命令的 `Route` 指到。

實作後定下的不上線命令與理由（共五種，其餘 42 種各有一條路由；`List` 依 `specs` 欄位分別對到 `GET /changes` 與 `GET /specs`）：

| 命令 | 理由 |
|---|---|
| `Show` | 用戶端由變更、規格與文件的路由組出 `show`；引擎自己的文字不上線 |
| `Trace` | 溯源要讀本機的程式碼與它的 git 歷史，只有用戶端有 |
| `PlanStrict` | 嚴格的重疊檢查隨變更 skills-verbs-second-wave 上線 |
| `WorkflowConfigShow` | 用戶端讀 `GET /config` 的原文，自己顯示 |
| `WorkflowConfigEdit` | 用戶端在 `GET /config` 的原文上算出新內容，以 `PUT /config` 整份寫回 |

兩個綁定測試的做法：對每條路由送一個能走到處理函式的請求，掛一個全部拒絕並記下命令名稱的執行前掛鉤，以掛鉤看到的命令判斷「這條路由執行哪一種命令」，不另外在路由表登記命令名稱。

### D7: Node 的 handleRequest

```ts
createHost(options: {
  store: StorePort; hooks?: HostHooks
  scopes?: (identity: HostIdentity) => MaybePromise<ProjectEntry[]>
  authMethods?: { methods: string[]; browser?: { issuer: string; clientId: string; scopes?: string[] } }
  pushTransports?: { type: string; url: string; resume?: boolean }[]
}): Host
host.handleRequest(
  request: { method: string; path: string; query?: string; headers: Record<string, string>; body?: Uint8Array },
  identity: HostIdentity | null
): Promise<{ status: number; headers: Record<string, string>; body: Uint8Array } | null>
```

- `ProjectEntry` 的欄位是 camelCase：`key`、`name`、`repos`；repo 是 `key`、`name`、`scope`、`canWrite`、`canWritePolicy`。
- 回傳 `null` 代表不是入口的路徑。
- 在背景執行緒執行，不阻塞事件迴圈。`scopes` 的呼叫方式和掛鉤、儲存函式相同（經 threadsafe function，可回傳 Promise）；它丟出例外時回應是 503。
- 方法不是五種之一時，回 405，不進入口。

宿主的那一條路由長這樣（文件裡的範例，不是這個變更的程式）：驗 token → 得到身分或 `null` → `handleRequest` → 回傳值是 `null` 就交給自己的 404，否則照 `status`、`headers`、`body` 回應。

### D8: 示範 server 改接

- 啟動時建一個 `HostKit`：儲存是長期持有的 TeamStore 轉接；`scopes` 函式查示範 server 自己的身分資料（成員關係、專案、repo），兩個能力開關都是「是編輯者」；`auth_methods` 是 `device` 與 `token`；`push_transports` 是今天握手宣告的那一筆 SSE；提交後掛鉤呼叫既有的事件通知。
- `/api/speclink/v1/scopes`、新增的兩條 API 根路徑與 `/api/speclink/v1/projects/{key}/*` 由一個轉交的處理函式接手（`api/forward.rs`）：
  1. 用既有的函式從 bearer token 解出使用者；解不出來時身分是「沒有」。
  2. 兩個示範 server 自己的前置檢查，維持今天的回應：帳號已停用 → 403；專案存在但這個人不是成員 → 403。
  3. 把 axum 的請求轉成 `HttpRequest`，在 blocking 的執行緒池呼叫 `HostKit::handle`。
  4. 回傳「沒有」時，交給留在 server 的路由：匯入與事件串流。兩者都不符 → 404。
- 身分的對映：識別是使用者的識別，顯示名稱是「名稱 <email>」，和今天寫進蓋章欄位的字串相同；顯示用名稱是使用者的 display，所以握手與身分查詢顯示的名稱不變。
- 留在 server 的匯入與事件串流仍用 `Binding` extractor 與編輯者的 middleware。
- 移除的檔案：`api/read_api.rs`、`api/context.rs`、`api/verb.rs`；`api/routes.rs` 只剩匯入與事件串流的處理函式。本文大小的上限（看板順序 4 MiB、匯入 32 MiB）留在 server 的路由層。

事件串流不用改：提交仍經 TeamStore 的轉接寫進 outbox，串流照舊從 outbox 讀。

實作時補上的細節：

- **每個請求只載入一次範圍**：今天每個請求經 `TeamStorePort::pinned` 只做一次 TeamStore snapshot（SQLite 與 Postgres 的 snapshot 是整個範圍的複製）。宿主套件長期持有同一個轉接，沒有每個請求的 view，所以轉接新增 `TeamStorePort::pin_thread`：guard 存在期間，同一個執行緒對同一個範圍的讀取共用第一次的載入，這個執行緒自己的提交會丟掉那個範圍的載入。轉交的處理函式在 blocking 執行緒呼叫入口前開 guard。
- **兩個前置檢查的確切範圍**：範圍清單用「允許停用帳號」的驗證，停用的帳號回 403 `account is suspended`（今天 `IdentityOnly` 的行為）；專案路由與 API 根的身分查詢用一般驗證，停用帳號的 token 不合格，身分是「沒有」，入口回 401（今天的行為）。非成員的 403 只在路徑是入口的路由時才檢查。
- **更新存取金鑰最後使用時間**：在解出使用者、通過示範 server 自己的前置檢查之後發生（Open Questions 的第一題）；被前置檢查以 403 拒絕的請求（非成員、範圍清單的停用帳號）不更新。和今天的差別是：之後被入口以版本、專案或 repo 拒絕的請求也會更新，因為那些檢查現在在入口裡。
- **看板順序提交的操作者**：TeamStore 的轉接記錄操作者的顯示名稱。今天看板順序的提交記的是使用者的識別，其他命令記的是「名稱 <email>」；改接後看板順序和其他命令一致，都是「名稱 <email>」。這只出現在儲存的記錄裡。
- **本文**：轉交的處理函式先把本文收滿（上限 32 MiB，理由同 `read_body_before_routing`），再依 `match_route` 的結果套上限：看板順序 4 MiB，其餘 2 MiB（axum 的預設）。超過回 413 的錯誤信封。
- **握手的 repo 名稱**：入口用 `RepoEntry.name`；今天示範 server 的握手把 repo 的 key 當名稱。兩者只在 repo 的名稱與 key 不同時有差別。
- **write_guard 的計數測試**：原本用 `include_str!` 數 `app.rs` 的寫入路由登記數，改成數入口路由表的非 GET 路由（扣掉唯讀的 Context 快照）加上留在 server 的 `/import`。

## Implementation Contract

**行為**

- 對同一批資料與同一個使用者，示範 server 改接前後，既有的每條專案路由與範圍清單的狀態碼、回應本文、`ETag` 相同。例外只有這幾項，各自在所屬的 Decision 說明：已知路徑配上它沒有的方法回 404（D3）；五種以外的方法回 405（D1）；握手的 repo 名稱用 repo 的名稱（D8）；儲存的 Backend 失敗用固定訊息（稽核結論）。
- 用記憶體版儲存與一個固定的 `scopes` 函式建立的 `HostKit`，能直接回應握手、變更清單、建立變更、勾選任務、封存、輪詢，不需要任何 HTTP 框架。
- 沒有提供 `scopes` 的宿主套件，對專案與範圍清單的路徑回 404。
- 執行前掛鉤拒絕一個經入口來的命令時，回應是 409 `refused`，訊息是宿主的原因。
- `can_write` 為假的人送出寫入 → 403；`can_write_policy` 為假的人改政策鍵 → 403，改 context → 成功。
- `wire_coverage` 對每個命令變體都有答案；兩個綁定路由表的測試通過。

**介面與資料形狀**

- Rust：`speclink_host::http::{HttpRequest, HttpResponse, Method, WireCoverage, wire_coverage}`、`HostKit::handle`、`HostKitOptions`、`ProjectEntry`、`RepoEntry`。
- Node：`host.handleRequest`；`createHost` 的 `scopes`、`authMethods`、`pushTransports`。
- 線上新增：`GET /api/speclink/v1/auth-methods`（型別已存在）、`GET /api/speclink/v1/whoami`（回應是既有的身分查詢型別）。既有路由不變。

**失敗的樣子**

| 情況 | 回應 |
|---|---|
| 沒有身分 | 401 `permission_denied` |
| API 版本不符 | 409 `refused` |
| 專案或 repo 不在這個人的清單裡 | 404 `not_found` |
| 多個 repo 而沒有指定 | 409，列出候選 |
| 沒有寫入的能力 | 403 `permission_denied` |
| 掛鉤拒絕 | 409 `refused`，宿主的原因 |
| `scopes` 函式失敗 | 503 `unavailable` |
| 儲存的版本衝突、暫時無法使用、其他失敗 | 409 `revision_conflict`、503、500（既有的對映） |
| 本文不是合法的 JSON 或欄位不合 | 400 `invalid_argument` |
| 入口不認得的路徑 | 回傳「沒有」，由呼叫端決定 |

**驗收**

- `cargo test -p speclink-host --test http_door`：上面「行為」與「失敗的樣子」的每一項，用記憶體版儲存。
- `cargo test -p speclink-server`：既有的整合測試不改斷言就通過；新增兩條 API 根路徑的測試。
- `cargo test -p speclink-remote`、`cargo test -p speclink-cli`、`cargo test -p speclink-desktop`：既有的遠端測試通過。
- Node：`handle-request.spec.ts` 用 JS 的記憶體儲存走一遍握手、清單、寫入、輪詢、掛鉤拒絕、沒有身分、`null` 的回傳。
- `speclink-host` 的相依裡沒有 axum 與 tokio。

**範圍**

- 在範圍內：入口、三個建立時的輸入、Node 的 `handleRequest`、示範 server 的改接、兩條新的 API 根路徑。
- 不在範圍內：`speclink-cli`、`speclink-remote`、桌面 app、匯入、串流、登入與管理。

## 稽核結論

以 sharp-edges 清單逐項檢查新增的介面（入口、`HostKitOptions`、示範 server 的轉交、Node 的 `handleRequest`）。每一項都有 `http_door.rs` 裡 `audit_` 開頭的測試釘住。

| 檢查項 | 結論 |
|---|---|
| 沒有身分時不外洩專案資料 | 通過。對路由表的每一條路由送不帶身分的請求，全部 401，儲存沒有被呼叫，範圍函式也沒有被問。`auth-methods` 只回宿主自己的宣告。 |
| 網址的專案與 repo 只能對到範圍函式給的範圍識別 | 通過。呼叫者的範圍只取自 `RepoEntry.scope`；`X-Speclink-Repo` 只比對 repo 的 key，填範圍識別（`s1`、`alpha/web`）一律 404。 |
| 網址的名稱組不出別類文件的鍵 | 通過。以百分比編碼的 `..%2F`、`%2E%2E`、`specs%2F..` 等名稱打每一類讀取與寫入路由：讀不到討論的內容，範圍裡沒有任何文件被寫入或刪除。名稱一律經 `keys::*` 建鍵並回讀比對（`documents`），或由引擎驗證。 |
| 能力開關沒有可繞過的路由 | 通過。`can_write` 為假時，路由表裡每條非 GET／HEAD 的路由（Context 快照除外）都是 403，所有 GET 路由都沒有提交。`can_write_policy` 只影響流程設定的整份寫入；單項編輯不上線（D6）。 |
| 掛鉤拒絕後沒有提交 | 通過。掛一個全部拒絕的掛鉤，打完路由表每一條路由，儲存沒有收到任何提交（看板順序的寫入也經同一個掛鉤）。 |
| 入口不讀 `Authorization` | 通過。帶 token 而身分是「沒有」仍是 401；身分有效而 `Authorization` 是亂碼照常 200。Node 的 `handleRequest` 只用第二個參數的身分。 |
| 錯誤回應不帶出文件全文或內部路徑 | **找到並修正一項**：儲存回報的 `Backend` 失敗原本把儲存自己的文字（可能含資料庫位址或檔案路徑）原樣放進 500 的訊息。改成固定訊息「the store failed; the host's store has the cause」；暫時無法使用（503）本來就是固定訊息。範圍函式失敗的 503 也不帶宿主的文字。 |
| 範圍函式失敗不被當成「沒有範圍」 | 通過。失敗是 503，不是 404，也不讀任何範圍。Node 的 `scopes` 回傳形狀不對時以 `internal` 拒絕，訊息指出缺的欄位（`canWrite`、`canWritePolicy` 沒有預設值），不默默變成 503 或「沒有權限」。 |

示範 server 的轉交處理函式本身讀 `Authorization`（那是宿主的工作）：解不出使用者時身分是「沒有」，由入口回 401；非成員與停用帳號的兩個前置檢查維持今天的 403。

## Risks / Trade-offs

- [回歸對照] 約五十條路由換了宿主。→ 處理函式是搬移不是重寫（例外是品質關卡後合成一組的兩站路由與討論寫入的錯誤判斷，見 D4）；示範 server 既有的整合測試、`speclink-remote` 的 typed client 測試、CLI 的遠端對等測試（`remote_verb_parity`）與桌面的整條鏈測試都不改斷言。搬移分批進行，每一批跑一次 server 的測試。
- [錯誤順序] 入口的檢查順序和 `Binding` 不完全相同（非成員是 404）。→ 示範 server 在轉交前自己做那一項檢查（D8）；`write_guard` 與 `read_api` 的既有測試會抓到偏差。
- [跨平台] 新增的程式沒有路徑與程序呼叫。Node 的本文用 `Uint8Array` 跨界，三個平台相同。
- [每個請求呼叫一次 `scopes`] 宿主的函式慢時每個請求都慢。→ 文件寫明由宿主快取；示範 server 的實作是幾個索引查詢，和今天 `Binding` 做的事相同。
- [路徑比對自己寫] 手寫的比對容易在邊界出錯（結尾的斜線、編碼過的斜線）。→ 用既有測試的全部網址當對照，另外為「吃掉其餘路徑」的段與百分比編碼寫測試；行為以今天 axum 的結果為準。
- [Node 的宿主沒有推送] 用戶端靠輪詢，更新會慢幾秒。→ 寫成已知缺口；握手不宣告推送時用戶端本來就會輪詢。
- [儲存解耦] 完成後，示範 server 只是「登入與帳號」加上一個轉交；線上協定的實作不再綁在任何一種儲存或框架上。

## Migration Plan

沒有資料搬移。示範 server 的部署方式不變。要退回時還原這個變更的 commit。

## Open Questions

- 示範 server 的轉交處理函式要不要保留今天 `Binding` 的「更新存取金鑰最後使用時間」這個副作用的位置。實作時確認它在解出使用者的那一步就發生；不是的話在轉交前補上。→ 已確認：在解出使用者並通過前置檢查後發生（見 D8 的「實作時補上的細節」）。
- 握手的 `capabilities` 裡今天固定為真的欄位，在宿主套件裡是否都成立（例如 Context 快照）。逐一對照路由表確認；有不成立的就改成依路由表決定。→ 已確認：`contextSnapshots`、`validate`、`analyze` 對應的路由都在路由表裡，而且不受 `can_write` 限制（Context 快照是唯讀的 POST），所以維持固定為真。
