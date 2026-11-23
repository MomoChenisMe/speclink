## Context

宿主套件的 HTTP 入口（`speclink-host` 的 `http` 模組）把每一條路由轉成一個處理函式。處理函式有兩種：

- 轉成引擎命令再執行（`Request::run`）。這條路會經過 `HostKit::run_with`，所以會問 `beforeCommand`。
- 直接呼叫 `documents` 模組讀儲存接口。`http/routes.rs` 把它們標成「不是引擎命令的文件」。這條路不經過 `HostKit`，所以不問掛鉤。唯一的例外是看板順序的寫入：它自己呼叫 `HostKit::ask`，名稱是 `put-board-order`。

第二種裡面有八條會回範圍的內容：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived`、`GET /archived/{datedName}/capabilities`、`GET /archived/{datedName}/artifacts/{path}`、`POST /context`。host-kit 的規格只要求「引擎命令與看板順序的寫入」經過掛鉤，所以現況符合規格，但宿主預期的是「每一次呼叫都問一次」。

限制與前提：

- 示範用 Server 與任何宿主共用這一份入口；示範用 Server 沒有設定 `beforeCommand`（`speclink-server` 的 `state.rs`），它的回應不能變。
- 入口在進處理函式之前已經做完三個檢查：沒有身分回 401、範圍清單對不到專案或 repo 回 404、需要寫入能力而沒有時回 403（`http/mod.rs`）。
- 前一個變更 `archived-list-and-detail-reads` 讓 `Command::List` 多了 `archived` 欄位，`GET /archived` 也改成呼叫 core 的共用函式。這個變更建立在它之上。

## Goals / Non-Goals

**Goals:**

- 入口讀寫範圍內容的每一條路由都問 `beforeCommand` 正好一次，拒絕時儲存沒有收到呼叫。
- 宿主只需要一張權限表：既有命令名稱的規則自動涵蓋對應的入口讀取。
- 以後新增路由時，忘記問掛鉤會被測試擋下。

**Non-Goals:**

- 握手、輪詢、身分查詢、範圍清單、登入方式的宣告、`POST /skills`：不問掛鉤。它們不回範圍裡的文件內容；輪詢每幾秒一次。
- 範圍清單對不到專案時的 404：不變，也不帶掛鉤的原因。
- 入口回應的欄位表：討論已決定延後。
- 改變 Speclink Desktop 的畫面：它收到 409 時沿用現在 `GET /changes` 被拒絕時的呈現。
- 把這八條改寫成引擎命令：回應的形狀（原始文件加版本）和命令的結果不同，改寫的範圍遠大於這次的目的。

## Decisions

### 每條文件讀取在讀儲存之前問掛鉤一次

`Request` 加一個小方法（暫名 `ask_read`）：收命令名稱與參數，呼叫既有的 `HostKit::ask`。八個處理函式在解析完請求之後、呼叫 `documents` 之前呼叫它。這是唯一的落點：示範用 Server 與任何宿主都經過同一份處理函式。

替代方案：在路由表上標記「要問掛鉤」，由分派的地方統一問。否決，因為每條路由的參數不同（capability 名稱、搜尋字串、帶日期的名稱），要到處理函式裡才取得到。

### 五條讀取沿用引擎命令的名稱與參數

名稱與參數由 `Command` 的值產生（`name()` 與 `params()`），不手寫字串，所以和 `host.dispatch` 對同一個動詞問掛鉤時完全相同：

| 路由 | 用來產生名稱與參數的命令 | 掛鉤收到的名稱 | 參數的重點 |
| --- | --- | --- | --- |
| `GET /config` | `WorkflowConfigShow` | `workflow-config-show` | 沒有參數 |
| `GET /specs/{capability}/document` | `Show`，`item` 是 capability、`item_type` 是 `spec` | `show` | `item`、`itemType` |
| `GET /archived/{datedName}/artifacts/{path}` | `ArtifactCat`，`artifact` 是網址裡的文件路徑、`change` 是帶日期的名稱 | `artifact-cat` | `artifact`、`change` |
| `GET /archived` | `List`，`archived` 是 true，其餘是預設值 | `list` | `archived` |
| `GET /archived/{datedName}/capabilities` | 同上 | `list` | `archived` |

只產生名稱與參數，不執行這些命令：回應仍由 `documents` 的函式組出，形狀不變。`artifact-cat` 的 `artifact` 在這裡是文件路徑（例如 `proposal.md`、`specs/auth/spec.md`），和動詞用的識別（`proposal`、`specs/auth`）不同；規格寫明這一點。

替代方案：八條都給新名稱。否決，因為宿主對 `show`、`list` 已經有規則，新名稱會讓同一份資料有兩個要維護的權限項目。

### 三條讀取用新的命令名稱並加進 commandNames

`GET /search`、`POST /context`、`GET /board-order` 沒有對應的引擎命令。名稱定為 `workspace-search`、`context-snapshot`、`get-board-order`，常數放在 `documents` 模組，和 `PUT_BOARD_ORDER_COMMAND` 放在一起；`kit::command_names()` 把它們加進清單。參數：`workspace-search` 是 `{ "query": <字串> }`；`context-snapshot` 在請求指定變更時是 `{ "change": <名稱> }`，否則是空物件；`get-board-order` 是空物件。

這三個名稱不是引擎命令，所以不進 `read_only.rs` 的歸類，掛鉤的 `read` 函式也不接受它們。

### 檢查的順序

401（沒有身分）、404（範圍清單對不到）、403（沒有需要的能力）、400（請求的本文或查詢字串不合法）都在問掛鉤之前；這些情況掛鉤不被呼叫。問掛鉤之後才讀儲存。`POST /context` 帶著符合的 `If-None-Match` 時也先問掛鉤，放行後才回 304：有沒有變也是範圍的資訊。

### 以測試擋住以後新增的路由

在 `speclink-host` 的整合測試加一個走訪路由表的測試：掛鉤一律拒絕，對入口的每一條專案路由送一個合法的請求，斷言除了明列的不問掛鉤的路由之外，回應都是 409 而且儲存沒有收到讀取與提交。新路由沒有問掛鉤時這個測試會失敗。做法比照 `speclink-server` 的 `write_guard.rs`。

## Implementation Contract

**行為**

- 有 `beforeCommand` 時，八條讀取各問它正好一次，名稱與參數如上面兩個決策。放行時回應與現在相同（狀態碼、本文、ETag）。拒絕時回應是 409、reason 為 `refused`、訊息是宿主給的原因（原因是空字串時用既有的固定文字），儲存沒有收到任何呼叫。
- 沒有 `beforeCommand` 時，每一條路由的回應與儲存收到的呼叫都和現在相同。
- `commandNames`（Node）與 `command_names()`（Rust）是 60 個名稱：引擎的 54 個加宿主套件的六個（`scope-export`、`scope-import`、`put-board-order`、`get-board-order`、`workspace-search`、`context-snapshot`）。
- 讀取不提交，`afterCommit` 不被呼叫。

**介面與資料形狀**

- 不改路由、請求格式、回應格式、線上格式的型別。
- `BeforeCommand` 的形狀不變；新的只有三個名稱與它們的參數。

**失敗模式**

- 掛鉤拋錯或回答形狀不對：沿用既有規則，視為不放行。
- 401、404、403、400：和現在相同，而且掛鉤沒有被呼叫。

**驗收條件**

- `crates/host/speclink-host/tests/http_door.rs`：掛鉤一律拒絕時八條都是 409 加原因、儲存沒有呼叫；掛鉤記下的名稱與參數符合對照表；放行時八條的回應和沒有掛鉤時相同；401、404、400 的情況掛鉤的呼叫次數是 0；握手與 `GET /sync-state` 不問掛鉤。
- 走訪路由表的守門測試通過，而且把任何一條的問掛鉤拿掉時會失敗。
- `crates/host/speclink-host` 的 `command_names` 測試與 Node 的 `commandNames` 測試都是 60 個。
- 示範用 Server 的整合測試全部不變地通過。

**範圍界線**

- 在範圍內：`speclink-host` 的八個處理函式、三個名稱常數、`command_names()`、Node 的型別說明與測試、`docs/verb-contract` 與 `docs/sdk-node` 的中英文說明。
- 不在範圍內：`speclink-core`、`speclink-protocol`、`speclink-remote`、CLI 的輸出、Speclink Desktop 的程式碼、`documents` 模組組回應的邏輯。

## Risks / Trade-offs

- [只放行已知名稱的宿主升級後擋掉三條讀取] → 提案與更新日誌標成 BREAKING；`commandNames` 的清單讓宿主的權限表測試在升級時失敗並指出缺的名稱；`docs/sdk-node` 寫明三個新名稱各對應哪一條路由。
- [Speclink Desktop 與 CLI 在這八條收到 409 的呈現沒有測過] → 任務裡確認用戶端對 409 `refused` 的處理和 `GET /changes` 被拒絕時是同一條錯誤路徑，並以 CLI 的入口測試釘住「非零 exit code、stderr 是宿主的原因」。
- [每次讀取多一次掛鉤呼叫的成本] → 掛鉤本來就對每個引擎命令的讀取被呼叫；最頻繁的輪詢（`GET /sync-state`）不問。
- [`artifact-cat` 的參數在入口是文件路徑、在動詞是識別] → 規格與文件寫明；宿主的規則通常只看 `change`。
- [回歸對照] → 人眼輸出與 `--json` 輸出不變，golden 不受影響；示範用 Server 的整合測試確認回應不變。
- [跨平台] → 這次沒有路徑處理、沒有檔案系統與 git 的呼叫；Windows、macOS、Linux 的行為相同。
- [前一個變更還沒封存就開工] → `Command::List` 的 `archived` 欄位與 `GET /archived` 的新實作來自 `archived-list-and-detail-reads`；以 `speclink change depends` 記下相依，先做那一個。
