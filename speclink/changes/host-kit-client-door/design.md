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

- `Method` 是 GET、HEAD、POST、PUT、DELETE 五種；其他方法由呼叫端回 405。
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

1. 沒有身分 → 401 `permission_denied`。
2. `x-speclink-api-version` 不等於目前的 API 版本 → 409 `refused`（訊息與今天相同）。
3. 呼叫 `scopes`，找網址裡的專案：找不到 → 404 `not_found`。
4. 決定 repo：有 `x-speclink-repo` 標頭就用它，清單裡沒有 → 404；沒有標頭而專案只有一個 repo 就用它；沒有標頭而有多個 → 409（既有的多義訊息，列出候選）。
5. 比對路由表。沒有對照 → 回傳「沒有」（不是入口的路徑）。
6. 方法不是 GET 或 HEAD，而這個 repo 的 `can_write` 是假 → 403 `permission_denied`。例外是 Context 快照（它是唯讀的 POST），和今天相同。
7. 執行處理函式。

和今天示範 server 的順序差在第 3 步：今天「專案存在但不是成員」是 403，入口分不出這兩種情況，一律 404。示範 server 要維持 403，做法見 D8。

API 根的三條路徑：`auth-methods` 不檢查身分與版本；`whoami` 與 `scopes` 需要身分，不需要版本標頭（和今天的 `scopes` 相同）。

### D4: 路由的處理函式

從示範 server 搬到 `http/routes.rs`，每個函式的形狀是「收已解析的路徑參數、查詢、本文與呼叫者，回傳線上的型別或錯誤」：

- **執行引擎命令的路由**：建出 `Command` → `HostKit::run` → 把結果轉成線上的型別（轉換函式搬到 `http/wire.rs`）。結果的種類和預期不同時是內部錯誤，和今天相同。
- **不經引擎的路由**：呼叫 `documents` 的函式。看板順序的寫入先問執行前掛鉤（命令名稱 `put-board-order`、參數是空物件），成功後通知提交後掛鉤。
- **帶預期文件版本的 artifact 寫入**：照今天的順序——讀文件版本、比對、執行命令、回傳新的文件版本。
- **握手**：操作者是身分的識別與顯示名稱；專案與 repo 的識別沿用 `prj_<key>`、`repo_<key>`；`capabilities` 裡依能力開關填的欄位用 `can_write` 與 `can_write_policy`，其餘固定為真的欄位不變；更新方式的宣告是 `push_transports` 加上輪詢。
- **輪詢**：`documents::scope_version` 與 `If-None-Match` 比對，相同回 304。
- **專案底下的 whoami**：使用者取自身分，repo 清單取自 `scopes` 的結果。
- **API 根的 whoami**：只有使用者。
- **範圍清單**：`scopes` 的結果轉成既有的回應型別。
- **登入方式的宣告**：有設定就回它，沒有就 404。

每個成功的回應帶 `ETag`，值是執行後的範圍版本號加引號，和今天相同。

錯誤的對映搬到 `http/error.rs`：引擎的錯誤碼、儲存的失敗分類、`documents` 的錯誤，各自對到狀態碼與線上的 reason，表和今天 `api/error.rs` 的相同。示範 server 的 `api/error.rs` 只留它自己的路由需要的部分（身分、管理）。

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

目前不上線的命令與理由（實作時逐一確認，以實際的路由表為準）：需要本機程式碼的溯源、嚴格的重疊檢查（等 skills-verbs-second-wave）、流程設定的顯示與單項編輯（用戶端讀原文、在本機算出新內容、整份寫回）。

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
- 身分的對映：識別是使用者的識別，顯示名稱是「名稱 <email>」，和今天寫進蓋章欄位的字串相同。
- 留在 server 的匯入與事件串流仍用 `Binding` extractor 與編輯者的 middleware。
- 移除的檔案：`api/read_api.rs`、`api/context.rs`、`api/verb.rs`；`api/routes.rs` 只剩匯入與事件串流的處理函式。本文大小的上限（看板順序 4 MiB、匯入 32 MiB）留在 server 的路由層。

事件串流不用改：提交仍經 TeamStore 的轉接寫進 outbox，串流照舊從 outbox 讀。

## Implementation Contract

**行為**

- 對同一批資料與同一個使用者，示範 server 改接前後，既有的每條專案路由與範圍清單的狀態碼、回應本文、`ETag` 相同。
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

## Risks / Trade-offs

- [回歸對照] 約五十條路由換了宿主。→ 處理函式是搬移不是重寫；示範 server 既有的整合測試、`speclink-remote` 的 typed client 測試、CLI 的遠端對等測試（`remote_verb_parity`）與桌面的整條鏈測試都不改斷言。搬移分批進行，每一批跑一次 server 的測試。
- [錯誤順序] 入口的檢查順序和 `Binding` 不完全相同（非成員是 404）。→ 示範 server 在轉交前自己做那一項檢查（D8）；`write_guard` 與 `read_api` 的既有測試會抓到偏差。
- [跨平台] 新增的程式沒有路徑與程序呼叫。Node 的本文用 `Uint8Array` 跨界，三個平台相同。
- [每個請求呼叫一次 `scopes`] 宿主的函式慢時每個請求都慢。→ 文件寫明由宿主快取；示範 server 的實作是幾個索引查詢，和今天 `Binding` 做的事相同。
- [路徑比對自己寫] 手寫的比對容易在邊界出錯（結尾的斜線、編碼過的斜線）。→ 用既有測試的全部網址當對照，另外為「吃掉其餘路徑」的段與百分比編碼寫測試；行為以今天 axum 的結果為準。
- [Node 的宿主沒有推送] 用戶端靠輪詢，更新會慢幾秒。→ 寫成已知缺口；握手不宣告推送時用戶端本來就會輪詢。
- [儲存解耦] 完成後，示範 server 只是「登入與帳號」加上一個轉交；線上協定的實作不再綁在任何一種儲存或框架上。

## Migration Plan

沒有資料搬移。示範 server 的部署方式不變。要退回時還原這個變更的 commit。

## Open Questions

- 示範 server 的轉交處理函式要不要保留今天 `Binding` 的「更新存取金鑰最後使用時間」這個副作用的位置。實作時確認它在解出使用者的那一步就發生；不是的話在轉交前補上。
- 握手的 `capabilities` 裡今天固定為真的欄位，在宿主套件裡是否都成立（例如 Context 快照）。逐一對照路由表確認；有不成立的就改成依路由表決定。
