## Context

宿主套件的 HTTP 入口（`speclink-host` 的 `http` 模組）把每一條路由轉成一個處理函式。處理函式有兩種：

- 轉成引擎命令再執行（`Request::run`）。這條路會經過 `HostKit::run_with`，所以會問 `beforeCommand`。
- 直接呼叫 `documents` 模組讀儲存接口。`http/routes.rs` 把它們標成「不是引擎命令的文件」。這條路不經過 `HostKit`，所以不問掛鉤。唯一的例外是看板順序的寫入：它自己呼叫 `HostKit::ask`，名稱是 `put-board-order`。

第二種裡面有八條會回範圍的內容：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived`、`GET /archived/{datedName}/capabilities`、`GET /archived/{datedName}/artifacts/{path}`、`POST /context`。另外 `GET /changes/{name}/drift` 呼叫 `drift` 模組，也不經過 `HostKit`，回變更的 design、tasks 原文與完成記錄；它是 apply 時排查路由表才找到的第九條，提案時漏列。host-kit 的規格只要求「引擎命令與看板順序的寫入」經過掛鉤，所以現況符合規格，但宿主預期的是「每一次呼叫都問一次」。

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
- 把這九條改寫成引擎命令：回應的形狀（原始文件加版本）和命令的結果不同，改寫的範圍遠大於這次的目的。

## Decisions

### 每條文件讀取在讀儲存之前問掛鉤一次

`Request` 加一個小方法 `ask`：收命令名稱與參數，呼叫既有的 `HostKit::ask`。另一個小方法 `ask_as` 收一個 `Command` 的值，用它的名稱與參數呼叫 `ask`。九個處理函式在解析完請求之後、讀儲存之前呼叫它。這是唯一的落點：示範用 Server 與任何宿主都經過同一份處理函式。

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

替代方案：九條都給新名稱。否決，因為宿主對 `show`、`list` 已經有規則，新名稱會讓同一份資料有兩個要維護的權限項目。

### 四條讀取用新的命令名稱並加進 commandNames

`GET /search`、`POST /context`、`GET /board-order`、`GET /changes/{name}/drift` 沒有對應的引擎命令。名稱定為 `workspace-search`、`context-snapshot`、`get-board-order`、`spec-drift`，常數放在 `documents` 模組，和 `PUT_BOARD_ORDER_COMMAND` 放在一起；`kit::command_names()` 把它們加進清單。參數：`workspace-search` 是 `{ "query": <字串> }`；`context-snapshot` 在請求指定變更時是 `{ "change": <名稱> }`，否則是空物件；`get-board-order` 是空物件；`spec-drift` 是 `{ "change": <網址裡的變更名稱> }`。

drift 用新名稱而不沿用 `status`：CLI 的 `speclink drift` 在宿主端組報告，沒有引擎命令；沿用 `status` 會讓同一條規則管兩種不同的資料。

這四個名稱不是引擎命令，所以不進 `read_only.rs` 的歸類，掛鉤的 `read` 函式也不接受它們。

### 檢查的順序

401（沒有身分）、404（範圍清單對不到）、403（沒有需要的能力）、400（請求的本文或查詢字串不合法）都在問掛鉤之前；這些情況掛鉤不被呼叫。問掛鉤之後才讀儲存。`POST /context` 帶著符合的 `If-None-Match` 時也先問掛鉤，放行後才回 304：有沒有變也是範圍的資訊。

一條路由仍在問掛鉤之前讀一次儲存，和流程設定的寫入一樣列為例外：一般的 `GET /plan` 先讀看板順序，再執行 `plan`（掛鉤在執行時才被問）。它被拒絕時仍回 409，不帶範圍的內容。改成先問再讀，`plan` 的掛鉤參數會從看板的對照表變成 null；這次不改。

### 文件的寫入先問掛鉤、再比對版本

品質關卡第一輪（審查站）找到：`put_artifact` 先讀文件的版本、比對 `If-Match`，之後才執行命令（掛鉤在執行時才被問）。所以被掛鉤拒絕、但有寫入能力的呼叫端帶不相符的 `If-Match`，會收到 409 `revision_conflict`「expected version N, actual M」或「actual absent」，掛鉤完全沒有被問。這讓被拒絕的人探得出文件存不存在和它的版本。提案原本把這條列為例外；使用者決定改掉。

做法：`put_artifact` 在格式檢查（本文、`If-Match` 標頭、文件識別）之後，用組好的 `new-artifact` 命令的名稱與參數先問掛鉤（`Request::ask_as`，它改成收 `&Command`），再讀版本、比對 `If-Match`，最後執行命令。`RunOptions` 加一個布林欄位 `asked`：呼叫端已經用同一個命令的名稱與參數問過掛鉤時設為 true，`HostKit::run_with` 就不再問。這樣掛鉤只被問一次，收到的名稱與參數和現在相同。

- `asked` 只在 `run_with` 一般的那條路徑生效；流程設定的寫入仍然在提交之前問（它要帶變動的鍵），不看這個欄位。
- `RunOptions` 是 `pub(crate)`，宿主碰不到；只有 `put_artifact` 設這個欄位。
- `new-artifact` 沒有建立時的自由欄位，所以先問時的參數和 `run_with` 裡算出來的相同。
- 回應的差別只有一種情況：掛鉤拒絕而且 `If-Match` 不相符，從 409 `revision_conflict` 變成 409 `refused`。掛鉤放行、或沒有掛鉤時，回應不變。`If-Match` 不相符而且掛鉤放行時，掛鉤現在多被問一次（原本沒有被問）。

apply 時排查 `http/routes.rs` 裡其他比對版本的處理函式（任務 4.3；搜尋 `check_if_match`、`expected_version`、`header("if-match")`、`header("if-none-match")`，並列出每個處理函式裡讀儲存與問掛鉤的先後）：

- `put_artifact`：唯一呼叫 `check_if_match` 的地方。這次改成先問掛鉤。
- `put_board_order`：先問掛鉤（`put-board-order`），再呼叫 `expected_version`。不用改。
- `put_config`：在執行之前呼叫 `expected_version`。它只在送來的版本號超出合法範圍時讀儲存，回的訊息只帶範圍的版本號；`GET /sync-state` 本來就不問掛鉤而回同一個數字，所以沒有多透露東西。合法但過期的版本號走執行，拒絕優先於衝突（`run_with` 的既有規則）。不改，它仍是例外清單裡的一條。
- `context`：`If-None-Match` 的比對在問掛鉤之後（任務 1.4 已釘住）。`drift`：ETag 的計算在問掛鉤之後。`sync_state`：輪詢，照規格不問掛鉤。
- 其餘經 `Request::run` 的處理函式（`list_changes`、`get_change`、`get_artifact`）在執行之後才讀補充資料；被拒絕時執行先失敗，讀不到。一般的 `GET /plan` 維持例外。

替代方案一：不先讀版本，把 `If-Match` 直接轉成 `write_at` 的預期版本，讓提交時的檢查擋下不相符。否決，因為衝突的訊息要帶文件的實際版本，引擎會在提交失敗之前先讀寫一輪，示範用 Server 的回應有機會變。替代方案二：只把文件與註解改成和現況相同。使用者否決：洞還在。

### 以測試擋住以後新增的路由

在 `speclink-host` 的整合測試加一個走訪路由表的測試：掛鉤一律拒絕，對入口的每一條專案路由送一個合法的請求，斷言除了明列的不問掛鉤的路由之外，回應都是 409 而且儲存沒有收到讀取與提交（流程設定的寫入與一般的計畫可以先讀）。新路由沒有問掛鉤時這個測試會失敗。做法比照 `speclink-server` 的 `write_guard.rs`。

apply 時走訪 `http/router.rs` 路由表的結果（任務 1.5）：

- 經 `Request::run`（或 `run_with`、`try_run`）執行引擎命令、在命令裡問掛鉤：變更、任務、兩個品質關卡、認領、相依、排序、落地、自由欄位、進行中、封存、討論的全部路由，以及 `GET /plan`、`GET /trace/{capability}`、`GET /specs`、`GET`／`PUT /language`、`PUT /config`。
- 自己問掛鉤、不執行引擎命令：`PUT /board-order`（`put-board-order`，和九條讀取用同一個 `Request::ask`）、`GET /export`（`scope-export`）、`POST /import`（`scope-import`）。
- 經新的問掛鉤方法（`ask`、`ask_as`）：這次的九條讀取。
- 不問掛鉤：握手（`GET /binding`）、`GET /sync-state`、專案底下的 `GET /whoami`、`POST /skills`。
- 在問掛鉤之前讀一次儲存的例外：`PUT /config`、一般的 `GET /plan`。守門測試也檢查這兩條真的先讀，所以哪一條改成先問時，測試會要求把它從例外清單拿掉。apply 時這張清單還有 `PUT /changes/{name}/artifacts/{*artifact}`；任務 4.2 把它改成先問掛鉤之後，從清單拿掉。

守門測試是 `tests/http_door.rs` 的 `every_route_that_reads_or_writes_the_scope_asks_the_hook_before_the_store`。原本的 `coverage_points_at_every_route_that_runs_a_command` 把「問了引擎命令的名稱」當成「執行了那個命令」；五條沿用命令名稱的讀取不執行命令，所以列在那個測試的 `ASKED_AS_A_COMMAND`，線上歸屬表（`http/coverage.rs`）維持「不在線上」。

## Implementation Contract

**行為**

- 有 `beforeCommand` 時，九條讀取各問它正好一次，名稱與參數如上面兩個決策。放行時回應與現在相同（狀態碼、本文、ETag）。拒絕時回應是 409、reason 為 `refused`、訊息是宿主給的原因（原因是空字串時用既有的固定文字），儲存沒有收到任何呼叫。
- 沒有 `beforeCommand` 時，每一條路由的回應與儲存收到的呼叫都和現在相同。
- `commandNames`（Node）與 `command_names()`（Rust）是 61 個名稱：引擎的 54 個加宿主套件的七個（`scope-export`、`scope-import`、`put-board-order`、`get-board-order`、`workspace-search`、`context-snapshot`、`spec-drift`）。
- 讀取不提交，`afterCommit` 不被呼叫。
- 文件的寫入先問掛鉤一次（`new-artifact`，參數與現在相同），再讀版本、比對 `If-Match`。掛鉤拒絕時回應是 409 `refused`，儲存沒有收到任何呼叫；放行時回應與現在相同（含 `If-Match` 不相符的 409 `revision_conflict`）。本文、`If-Match` 標頭或文件識別不合法時仍回 400，掛鉤沒有被呼叫。

**介面與資料形狀**

- 不改路由、請求格式、回應格式、線上格式的型別。
- `BeforeCommand` 的形狀不變；新的只有四個名稱與它們的參數。

**失敗模式**

- 掛鉤拋錯或回答形狀不對：沿用既有規則，視為不放行。
- 401、404、403、400：和現在相同，而且掛鉤沒有被呼叫。

**驗收條件**

- `crates/host/speclink-host/tests/http_door.rs`：掛鉤一律拒絕時九條都是 409 加原因、儲存沒有呼叫；掛鉤記下的名稱與參數符合對照表；放行時九條的回應和沒有掛鉤時相同；401、404、400 的情況掛鉤的呼叫次數是 0；握手與 `GET /sync-state` 不問掛鉤。
- 走訪路由表的守門測試通過，而且把任何一條的問掛鉤拿掉時會失敗。例外清單 `READ_BEFORE_ASKING` 只剩 `PUT /config` 與 `GET /plan`。
- `crates/host/speclink-host/tests/http_door.rs`：掛鉤一律拒絕時，`If-Match` 是 0 與 999 的 `PUT /changes/demo/artifacts/design` 都回 409 `refused` 與宿主的原因，儲存沒有呼叫；掛鉤放行時兩種 `If-Match` 的回應和沒有掛鉤時相同，每個請求掛鉤被問一次、名稱是 `new-artifact`；沒有 `If-Match` 標頭時回 400、掛鉤被問 0 次。
- `crates/host/speclink-host` 的 `command_names` 測試斷言清單是引擎的名稱加宿主套件的七個、各一次；Node 的 `commandNames` 測試斷言 61 個。
- 示範用 Server 的整合測試全部不變地通過。

**範圍界線**

- 在範圍內：`speclink-host` 的九個處理函式、四個名稱常數、`command_names()`、Node 的型別說明與測試、`docs/verb-contract` 與 `docs/sdk-node` 的中英文說明；`put_artifact` 問掛鉤的先後與 `RunOptions` 的 `asked` 欄位。
- 不在範圍內：`speclink-core`、`speclink-protocol`、`speclink-remote`、CLI 的輸出、Speclink Desktop 的程式碼、`documents` 模組組回應的邏輯、`PUT /config` 與一般的 `GET /plan` 問掛鉤的先後、文件寫入的回應格式與 `If-Match` 的規則。

## Audit（任務 3.2，sharp-edges 清單）

- **讀之前先問**：九條讀取都在呼叫 `documents` 或 `drift` 模組之前問掛鉤。`GET /changes/{name}/drift` 的 ETag 計算與 `POST /context` 的 `If-None-Match` 比對都在問掛鉤之後。守門測試在一律拒絕時斷言這幾條沒有列出與讀取；`a_snapshot_that_did_not_change_asks_the_hook_first` 釘住 304 的先後。結論：符合。
- **拒絕的回應**：`HttpError::into_response` 只有 content-type 與錯誤本文（status、reason、message），沒有 ETag，也沒有範圍的內容或版本。結論：符合。
- **參數**：`context-snapshot` 只帶 `change`，不帶本文的其他欄位（`flow`）；`workspace-search` 只帶搜尋字串；`spec-drift` 只帶網址裡的變更名稱；五條沿用命令的讀取帶的是命令的欄位。沒有一個帶請求本文的全文。結論：符合。
- **掛鉤失敗**：Rust 的掛鉤只能回答 `Decision`。Node 的掛鉤拋錯或回答形狀不對時，Rust 端先拒絕（儲存沒有收到呼叫），整個 `handleRequest` 再以 `internal` 失敗。`handle-request.spec.ts` 的 `does not let a door read through when beforeCommand throws` 釘住 drift 與文件快照兩條。結論：不放行，符合。
- **壞蛋**：HEAD 走 GET 的處理函式，同樣先問。路徑參數先解碼再交給掛鉤，掛鉤看到的名稱就是讀取用的名稱。沒有任何設定能關掉這次加的詢問。結論：沒有找到繞過的方法。
- **懶惰的開發者**：沒有掛鉤時回應與儲存的呼叫不變。只放行已知名稱的宿主會擋下四個新名稱；這是多擋、不是多放的一側，`docs/sdk-node` 寫明升級時要補哪四個。結論：預設安全。
- **搞混的開發者**：`artifact-cat` 的 `artifact` 在入口是文件路徑（`proposal.md`），在動詞是識別（`proposal`）。宿主若以 `artifact` 的值寫規則（例如只擋 `design`），入口的 `design.md` 不會被那條規則擋下。不修，理由：規格定的是網址裡的文件路徑；改成動詞的識別要再加一份路徑對照，超出這次的範圍；而且宿主的規則通常只看 `change`。`docs/sdk-node` 的表與 `index.d.ts` 的 `beforeCommand` 說明都寫明這個差異。另外，宿主對 `list` 的規則現在也管已封存清單；要分開時看 `params.archived`。

## Risks / Trade-offs

- [只放行已知名稱的宿主升級後擋掉四條讀取] → 提案與更新日誌標成 BREAKING；`commandNames` 的清單讓宿主的權限表測試在升級時失敗並指出缺的名稱；`docs/sdk-node` 寫明四個新名稱各對應哪一條路由。
- [Speclink Desktop 與 CLI 在這九條收到 409 的呈現沒有測過] → 任務裡確認用戶端對 409 `refused` 的處理和 `GET /changes` 被拒絕時是同一條錯誤路徑，並以 CLI 的入口測試釘住「非零 exit code、stderr 是宿主的原因」。
  - apply 時的確認（任務 2.2）：`speclink-remote` 的九個用戶端函式（`config`、`board_order`、`spec_document`、`search`、`archived_list`、`archived_capabilities`、`archived_artifact`、`context_snapshot`、`spec_drift`）都經過 `translate_protocol_error`，和 `list_changes` 是同一個轉換：409 `refused` 原樣帶出宿主的原因。CLI 的 `tests/it/door.rs` 釘住 `speclink workflow-config show` 被拒絕時 exit code 非零、stdout 是空的、stderr 是 `Error: <原因>`，而且和 `speclink list` 被拒絕時的 exit code 與 stderr 相同。
  - Speclink Desktop（`apps/desktop/src-tauri/src/remote.rs`）的七條讀取（九條裡除了 `POST /context` 與 drift）都經過 `run`，錯誤原樣上拋，409 算 Server 有回應、不算離線；Desktop 不用 `POST /context` 與 drift。一個不同的地方：載入看板時，`board_order_doc` 把 `GET /board-order` 的任何失敗當成「沒有看板順序」。所以宿主只拒絕 `get-board-order`、放行 `list` 時，看板安靜地退回預設順序，不顯示錯誤；拖曳排序時的 `GET /board-order` 會帶出錯誤。這是 Desktop 原本「舊 Server 沒有這個端點」的退路，這次不改（Desktop 的程式碼不在範圍內）。
- [`asked` 是「跳過問掛鉤」的開關] → 它是 `pub(crate)` 的欄位，只有 `put_artifact` 在自己問過之後設定；`http_door.rs` 的測試釘住「拒絕時儲存沒有呼叫」與「放行時正好問一次」，漏問或問兩次都會失敗。
- [`If-Match` 不相符的寫入現在也問掛鉤] → 掛鉤多一次呼叫；這是先問再讀必然的結果，`docs/sdk-node` 寫明。
- [每次讀取多一次掛鉤呼叫的成本] → 掛鉤本來就對每個引擎命令的讀取被呼叫；最頻繁的輪詢（`GET /sync-state`）不問。
- [`artifact-cat` 的參數在入口是文件路徑、在動詞是識別] → 規格與文件寫明；宿主的規則通常只看 `change`。
- [回歸對照] → 人眼輸出與 `--json` 輸出不變，golden 不受影響；示範用 Server 的整合測試確認回應不變。
- [跨平台] → 這次沒有路徑處理、沒有檔案系統與 git 的呼叫；Windows、macOS、Linux 的行為相同。
- [前一個變更還沒封存就開工] → `Command::List` 的 `archived` 欄位與 `GET /archived` 的新實作來自 `archived-list-and-detail-reads`；以 `speclink change depends` 記下相依，先做那一個。
