## Context

示範 server 的專案路由（`api/routes.rs`、`api/read_api.rs`、`api/context.rs`）大多經 `api/verb.rs` 的 `run` 執行一個引擎命令。`run` 呼叫 `bridge::execute`，它為這次執行建一個 TeamStore 的轉接（`TeamStorePort`），再呼叫 `port_exec::execute`。

不經引擎的路由直接呼叫 TeamStore，逐一查證的結果：

| 路由 | 今天怎麼讀 | 真正需要的文件 |
|---|---|---|
| 範圍版本號（`run` 之後、輪詢、whoami、drift 之後） | 一次快照，取專案 revision | 只要版本號 |
| 變更清單補欄位 | 另一次快照，逐一讀每個變更的 metadata | 各變更的 metadata |
| 單筆變更補欄位 | 快照讀 metadata，再**整包匯出**只為了列出該變更的 delta spec | 一份 metadata、該變更底下的鍵 |
| 流程設定的讀取 | 快照讀一份 | `config.yaml` |
| 看板順序的讀取 | 快照讀一份 | `board/order` |
| 看板順序的寫入 | 快照比對專案 revision，自己開 unit of work 寫入並記事件 | `board/order` |
| evidence | 快照讀一份 | 一份 `.evidence.json` |
| 單份正式規格 | 整包匯出後查一份 | 一份 `spec.md` |
| 已封存清單 | 整包匯出 | 已封存變更的鍵、各自的 `tasks.md` 與 metadata |
| 單份已封存文件、已封存的 capability | 整包匯出 | 一份文件；一個已封存變更底下的鍵 |
| 搜尋 | 整包匯出 | 進行中變更的文件、進行中的討論 |
| Context 快照 | 快照（版本號與逐份的文件版本）加整包匯出（列舉） | 進行中變更的 artifact、正式規格、流程設定、詞彙表、進行中的討論 |
| drift | 內部自己建轉接與儲存視圖 | 一個變更的文件、相關的正式規格、流程設定、evidence |
| 帶預期版本的 artifact 寫入 | 快照讀文件 revision、比對、執行命令、再快照讀新的 revision | 一份文件的版本 |

版本號的來源：專案 revision 是整個專案共用的計數器；文件的 revision 是最後寫它的那次提交的專案 revision。轉接把它們原樣對到儲存接口的範圍版本與文件版本。

轉接的現況：綁定一個範圍；第一次列出或讀取時載入整個範圍（一次快照、一次匯出、每份文件一次讀取），之後在這個轉接的生命週期裡重用，只有自己的提交會清掉。所以它被設計成「每個命令建一個」。

引擎的封存：提交階段先把被改寫的正式規格的舊內容寫到 `<workspace>/.speclink/snapshots/<封存名>/`，再寫正式規格，最後搬移變更的文件；另外會移除本機的開工標記檔。經 Host 執行時引擎拿到的是一個根目錄為空的 workspace，這些路徑變成相對於程序的工作目錄。

限制：

- 線上的回應不能變。對照是 `crates/host/speclink-server/tests` 的整合測試、`speclink-remote` 的 typed client 測試與桌面的整條鏈測試。
- 別的工作階段正在改 `routes.rs` 的流程設定寫入路由；這個變更排在它之後。

## Goals / Non-Goals

**Goals:**

- 示範 server 的專案路由裡，除了匯入與事件串流，沒有任何一處直接呼叫 TeamStore。
- 這些讀寫是不依賴 HTTP 框架的函式，下一個變更可以原樣搬進宿主套件的入口。
- 輪詢範圍版本號的成本不因為改走轉接而變成每次匯出整個範圍。
- 經 Host 封存不在程序目錄寫檔。

**Non-Goals:**

- 路由本身的搬移、握手、範圍清單、Node 的入口。
- 匯入、事件串流、管理介面。
- 跨多次儲存呼叫的一致快照。

## Decisions

### D1: 新模組 speclink_host::documents

每個函式的前兩個參數是 `&dyn StorePort` 與 `&ScopeId`，回傳 `Result<_, DocumentsError>`。回傳值用 `speclink-protocol` 的型別（Host 已經相依它），需要 ETag 的函式一併回傳範圍版本號。

| 函式 | 對儲存的呼叫 | 回傳 |
|---|---|---|
| `scope_version` | 一次列出，前綴是 `config.yaml` | 範圍版本 |
| `config_document` | 一次列出（同上，同時得到範圍版本）、一次讀取 | 原文、解析出的 schema 名稱、範圍版本 |
| `board_order` | 一次列出（前綴 `board/order`）、一次讀取 | 原文或沒有、範圍版本 |
| `put_board_order` | 見 D3 | 新的範圍版本 |
| `change_evidence` | 一次列出（取範圍版本）、一次讀取 | evidence 的項目、範圍版本 |
| `spec_document` | 一次列出、一次讀取 | 原文、範圍版本 |
| `archived_list` | 一次列出（前綴 `archive/changes/`）、一次讀取（全部已封存變更的 `tasks.md` 與 metadata，一次帶多個鍵） | 清單、範圍版本 |
| `archived_artifact` | 一次列出（取範圍版本）、一次讀取 | 原文、範圍版本 |
| `archived_capabilities` | 一次列出（前綴 `archive/changes/<封存名>/`） | 名稱清單、範圍版本 |
| `search` | 兩次列出（`changes/`、`discussions/`）、一次讀取（要掃的文件，一次帶多個鍵） | 命中的清單、範圍版本 |
| `context_snapshot` | 見 D4 | 快照或「沒有變」 |
| `change_extras` | 一次讀取（給定名稱的 metadata，一次帶多個鍵） | 每個變更的補充欄位 |
| `change_delta_capabilities` | 一次列出（前綴 `changes/<name>/specs/`） | 名稱清單 |
| `document_version` | 一次讀取 | 文件版本或沒有 |

共同的規則：

- 取範圍版本用「列出」的回傳值。函式需要的列出剛好是它要的前綴時，同一次呼叫兼作兩用；否則多列一次窄的前綴。
- 鍵一律用 `speclink_store_port::keys` 的函式建立；把鍵解析回名稱用 `port_view` 既有的 `parse_key`。
- 解析、篩選與組合的邏輯原樣從 server 搬來：已封存清單的 metadata 讀取順序與任務進度、搜尋的比對與摘要片段、Context 快照選哪些文件與路徑怎麼組、evidence 的解析（解析失敗仍然是內部錯誤）。搬的時候不改行為。
- `DocumentsError` 有六種：找不到（帶訊息）、無效的設定（流程設定解析失敗）、內容過大（看板順序超過上限）、範圍已移動（看板順序的預期範圍版本不符）、內容損壞（evidence 無法解析）、儲存失敗（`PortError`）。server 把它們對到今天各路由回的狀態碼：404、422、413、409 `revision_conflict`、500、依 `PortError` 既有的對映。錯誤訊息的文字由 `documents` 組出，和今天各路由的訊息相同。（實作時補上：D3 的兩種拒絕需要自己的種類，原本寫的四種不夠。）
- `documents::etag(version)` 把範圍版本組成 ETag（數字加上雙引號），server 全部改用它。

已封存清單今天為了每個項目的規格數，是從整包匯出的鍵數出來的；改成從同一次列出的鍵數。

- **否決：把這些讀取做成引擎命令** — 它們回傳的是線上的型別（例如 Context 快照、原文加版本），不是引擎的結果；做成命令要在引擎裡加上版本的概念。
- **否決：讓這些函式經儲存視圖 `PortView` 讀** — 視圖不對外提供版本，而這些路由多半要回文件版本或範圍版本。

### D2: 一致性

今天每條路由讀一份快照。改寫後一條路由對儲存發出兩到三次呼叫，中間有別人提交時：

- 回傳的範圍版本（ETag）取自第一次呼叫，內容取自之後的呼叫。內容可能比 ETag 新。用戶端下一次輪詢會發現版本前進並重讀，結果收斂。
- 反過來的情況（ETag 比內容新）不會發生，因為版本先取。

每個函式都依這個順序：先取版本，再讀內容。

### D3: 看板順序的寫入

`put_board_order(port, scope, actor, content, expected_scope_version, at)` 的步驟：

1. 內容超過 1 MiB → 拒絕（今天的 413）。
2. 列出前綴 `board/order`：得到範圍版本與這份文件的版本（或不存在）。
3. 範圍版本不等於 `expected_scope_version` → 版本衝突（今天的 409）。
4. 提交一筆寫入：文件存在時預期是它的版本，不存在時預期是「必須不存在」；事件一筆 `board-order-updated`，payload 是空物件；命令標籤 `put-board-order`；操作者取自呼叫端。
5. 回傳提交後的範圍版本。

只有一筆寫入，所以沒有半套狀態。第 3 步與第 4 步之間別人改了別的文件時不會被擋，只有改到看板順序文件才會在提交時衝突；這和今天相同。

### D4: Context 快照

`context_snapshot(port, scope, request, if_none_match)`：

1. 列出 `changes/`，取範圍版本。`if_none_match` 等於它 → 回「沒有變」，不再讀。
2. 請求指定了變更而 `changes/<name>/` 底下沒有任何鍵 → 找不到。
3. 再列出 `specs/` 與（沒有指定變更時）`discussions/`。
4. 依今天的規則選文件：進行中變更的 artifact（指定變更時只選它的；metadata 與 evidence 不選）、全部正式規格、流程設定、詞彙表、沒有指定變更時的進行中討論。
5. 一次讀取全部選到的鍵。列得到但讀不到的略過。
6. 每份文件回路徑、內容、文件版本、內容摘要；整體的摘要、`snapshotId`（範圍版本）、`policyRevision`（流程設定的文件版本）照今天的算法。

### D5: drift 改收儲存接口

`drift::spec_drift(port: &dyn StorePort, scope: &ScopeId, change: &str)`。內部照舊在儲存視圖上呼叫引擎的 drift 函式，只是不再自己建 TeamStore 的轉接。回傳型別與線上的轉換不變。

### D6: TeamStore 的轉接可以長期持有

`TeamStorePort` 改成：

- 建立時只收 TeamStore，不綁範圍。每次呼叫的範圍識別是 `<project>/<repo>`，轉接把它拆成 TeamStore 的範圍；格式不合時回報後端錯誤。project 與 repo 的 key 都可能含 `/`，所以 project 裡的 `%` 與 `/` 寫成 `%25`、`%2F`：第一個 `/` 一定是 project 的結尾，兩個不同的範圍不會得到同一個識別。一般的 key 識別不變（例如 `default/main`）。
- 每次列出或讀取先取一次快照，拿到專案 revision。這個範圍已載入的內容是在同一個 revision 載入的 → 重用；否則重新載入（一次匯出與逐份讀取）並記下 revision。
- 自己的提交成功後，丟掉該範圍已載入的內容。
- 每個範圍最多留一份已載入的內容；兩次載入同時完成時，留 revision 較新的那份。沒有其他淘汰規則。
- 同一個範圍的載入一次一個（品質關卡主 checkout 第 1 輪修正）：一個範圍有提交之後，同專案的訂閱者會同時重讀；沒有這條規則時，每個錯過載入的請求都各自匯出一次。等待中的請求在 revision 相同時直接用剛完成的載入；不同範圍互不等待。

**每個請求一份固定的載入（品質關卡第 1 輪修正）。** 原本寫「每次先取一次快照（便宜）」是錯的：TeamStore 的快照會把整個範圍的文件與 history 讀進記憶體（SQLite、PostgreSQL、記憶體版都是）。照原寫法，一個命令有幾次列出或讀取就複製幾次整個範圍，命令內的讀取也可能來自不同的 revision。所以轉接另外提供 `TeamStorePort::pinned(scope)`：一個請求用的視圖。

- 視圖的第一次列出或讀取取一次快照，revision 變了才匯出；之後這個請求的列出與讀取都由同一份載入回答。
- 經視圖的提交照常送到 TeamStore，之後視圖重新載入，所以讀得到自己寫的內容。
- 轉接本身的規則不變：每次呼叫依 revision 判斷是否重用（規格「TeamStore 轉接成儲存接口」）。視圖只是讓一個請求內的呼叫共用同一次判斷。

示範 server 在啟動時建一個轉接，放進共用的狀態，全部請求共用。每個請求由 `verb::on_port` 在 blocking pool 上取一個 pinned view 再執行；`api/verb.rs` 的 `run` 直接呼叫 `port_exec::execute`。執行後的 ETag：有提交時是提交回傳的範圍版本，沒有提交時是視圖的範圍版本。

成本的變化：

- 執行命令的路由：改寫前是一次快照、一次匯出，加上取 ETag 的一次快照；改寫後是一次快照，revision 沒變時沒有匯出。命令有提交時，同一請求之後的讀取（例如帶 `If-Match` 的寫入要讀新的文件版本）會重新載入一次。
- 輪詢：一次快照，沒有匯出，和改寫前相同。
- 記憶體多了每個用過的範圍一份內容；示範 server 的定位是示範與試用，接受這個代價，並寫進風險。

既有的 `bridge::execute` 與它的型別別名留著：Host 的雙路徑測試還在用它，移除 TeamStore 是最後階段的變更。它改成內部建一個不綁範圍的轉接並取 pinned view，所以一個命令仍是一次快照、一次匯出，行為不變。

**實作時補上的一塊：在預期的範圍版本才執行。** 流程設定的寫入（`bridge::execute_at_revision`）原本靠「比對與命令的讀取用同一次載入」保證命令不會讀到比對之後別人寫的內容再蓋掉它。轉接改成 revision 變了就重新載入之後，這個保證不再成立：比對在 N，命令可能讀到 N+1 的內容並以 N+1 的版本提交成功。所以新增 `port_exec::execute_at_version(port, scope, ctx, cmd, expected)`，在任何儲存接口上成立：

1. 先取範圍版本，不等於 `expected` → 回「已移動」，命令不執行。
2. 命令照常執行，但提交前再取一次範圍版本；不等於 `expected` → 不提交，回「已移動」。
3. 命令沒有東西可提交時，結束前也再取一次；不等於 `expected` → 回「已移動」。

第 2、3 步看到版本沒變，表示命令讀取的期間沒有任何提交落地，讀到的都是呼叫端看到的那一版。第 2 步與提交之間落地的寫入，由提交本身的版本比對擋下，和改寫前相同。取範圍版本用 `documents::scope_version`（列出 `config.yaml` 這個最窄的前綴）；`port_exec` 的這個函式與所有文件讀取都用它。

流程設定的寫入路由改成在請求的 pinned view 上呼叫它，比對、執行、回傳版本的邏輯不變。在 pinned view 上，命令讀的就是比對時的那份載入，第 2、3 步不會看到版本移動；寫入由提交本身的版本比對把關，行為和改寫前的 `bridge::execute_at_revision` 相同。`bridge::execute_at_revision` 與 `AtRevision` 因此沒有使用者，一併移除；Host 雙路徑測試裡對它的那一條改成在 TeamStore 的轉接上直接測 `execute_at_version`。

`expected` 的型別是 `Version`。線上的數字超過版本上限時，它不可能是任何範圍的版本，server 直接回 409（訊息與版本不符時相同）。

- **否決：每個請求建一個轉接** — 輪詢會變成每次匯出整個範圍。
- **否決：為「只取版本」在儲存接口加第四個操作** — 契約已定為三個；真正的儲存用很窄的前綴列出就很便宜，慢的只是這個過渡用的轉接。
- **否決：依時間淘汰已載入的內容** — 正確性由 revision 比對保證，時間只是多一個要測的維度。
- **否決：給 TeamStore 加一個只取 revision 的操作**（第 1 輪修正時考慮過）— 三個 driver 都要改，而且命令內的讀取仍可能來自不同的 revision。

### D7: 沒有 workspace 時封存不寫本機檔案

引擎的封存收到的執行情境沒有 workspace 時：

- 不寫 `.speclink/snapshots/` 底下的備份與清單檔。
- 不移除本機的開工標記檔。
- 結果的 `snapshot_created` 是假。
- 其餘步驟不變：驗證、合併 delta、寫正式規格、搬移變更的文件、一併封存討論。

寫入順序與失敗後的狀態：經儲存接口時，正式規格的寫入與文件的搬移本來就在同一次提交，失敗時什麼都沒寫，所以不需要本機備份當作復原的依據。本機專案（有 workspace）的順序與備份不變。

判斷用的是「執行情境有沒有 workspace」，不是儲存的種類：Node SDK 的 fs 形式有 workspace，照舊寫備份。

實作時的做法：引擎的命令層在執行情境沒有 workspace 時，交給封存的是 root 為空的 workspace；封存裡既有的兩個守門（linked worktree、章失效的內容錨）已經用「root 是空的」判斷沒有 workspace。這個判斷收成 `Workspace::is_local()`，封存的提交階段沿用它，所以 `command/mod.rs` 不用改。另外一個本機效果也一併跳過：移除舊格式的 touched 記錄檔（`legacy_touched_file`），它和開工標記一樣是本機檔案。提交階段的錯誤訊息只在真的寫了備份時才指出備份位置；有 workspace 時訊息不變。

同一類的問題（品質關卡第 1 輪修正，依專案審查規則「同一種寫法在 repo 其他位置有同樣的問題要一併修正」）：discard 也會移除 touched 記錄檔，root 為空時同樣指向程序的工作目錄。discard 改成同樣只在 `is_local()` 時移除；本機專案的 discard 行為不變。apply 的 preflight 也有同一類寫法（拿 root 接上提案裡的路徑判斷檔案在不在）：主 checkout 第 1 輪修正讓它在沒有 workspace 時不查本機檔案。preflight 本來就不經線上回應，所以遠端用戶端看不到差別。

實作前先查 CLI 的遠端封存輸出與 Speclink Desktop 的封存結果畫面怎麼用 `snapshotCreated`。它只是多印一行時，遠端少那一行屬於刻意的變更，更新對應的測試期望值並補進 proposal 的相容性影響；它影響流程判斷時停下來回報。

## Implementation Contract

**行為**

- 對同一批資料，改寫前後每條路由的狀態碼、回應本文、`ETag` 標頭逐位元相同。例外：
  - 封存回應的 `snapshotCreated`（D7）。
  - 儲存失敗時的訊息與狀態碼：兩個請求幾乎同時、比對之後提交之前別人改了同一份文件時，409 的訊息文字改用儲存接口的格式；儲存回報後端錯誤時 500 的訊息文字改用儲存接口的格式；TeamStore 回報找不到或無權限時改回 500（改寫前是 404／403）。儲存接口只有衝突、暫時無法使用、後端錯誤三種失敗，保不住原本的對映；現有的三個 driver 不會回報找不到或無權限。
- 示範 server 處理匯入與事件串流以外的任何專案路由時，TeamStore 只經轉接被呼叫。
- 連續輪詢範圍版本號而沒有提交時，TeamStore 的匯出一次都不被呼叫。
- 經 Host 封存一個會改寫既有正式規格的變更後，程序的工作目錄沒有新增任何檔案。

**介面與資料形狀**

- `speclink_host::documents` 的函式（D1 的表）、`DocumentsError`、`etag`、`scope_moved`、`check_board_order_size`、`BOARD_ORDER_LIMIT`。每個讀取函式回傳線上型別與範圍版本；`documents::scope_version` 是取範圍版本的唯一入口；看板順序寫入的預期版本是 `Version`。
- `speclink_host::port_exec::execute_at_version(port, scope, ctx, cmd, expected: Version) -> AtVersion`（D6 的實作補充）與 `now()`（提交時間）。
- `speclink_host::teamstore_port::scope_id(&Scope)`：TeamStore 的範圍轉成範圍識別（project 段跳脫 `%` 與 `/`）。
- `TeamStorePort::pinned(&ScopeId)`：一個請求的視圖（D6）。
- `speclink_core::workspace::Workspace::is_local()`（D7）。
- 測試用：`speclink_store::memory::MemoryStore::snapshot_count()` 與 `export_count()`，取代各測試自己包的計數 TeamStore。
- 示範 server 的 `AppState::new(store, identity, config, events)`：建立時一併建出共用的轉接（欄位 `port`）。
- `speclink_host::drift::spec_drift(&dyn StorePort, &ScopeId, &str)`。
- `speclink_host::teamstore_port::TeamStorePort::new(store)`，不綁範圍；它是 `Send + Sync`，可以放進 `Arc`。
- 線上：沒有新增或改變的型別。

**失敗的樣子**

| 情況 | 今天 | 改寫後 |
|---|---|---|
| 流程設定無法解析 | 422 | 相同 |
| 不存在的正式規格、不存在的已封存文件 | 404 | 相同 |
| 不存在的已封存變更的 capability 清單 | 空陣列 | 相同 |
| 不存在的變更的 evidence | 空的項目 | 相同 |
| evidence 的內容無法解析 | 500 | 相同 |
| 看板順序：預期的範圍版本不符 | 409 revision_conflict | 相同 |
| 看板順序：內容超過 1 MiB | 413 | 相同 |
| 儲存暫時無法使用 | 503 | 相同 |
| 範圍識別的格式不合（只會是程式錯誤） | 不適用 | 500 |

**驗收**

- `cargo test -p speclink-host --test documents`：每個函式用記憶體版儲存測正常與異常的情況，並用計數的儲存斷言 D1 表裡的呼叫次數。
- `cargo test -p speclink-host`：轉接的測試涵蓋多個範圍、revision 沒變時不匯出、提交後重新載入、別人提交後重新載入。
- `cargo test -p speclink-server`：既有的整合測試不改斷言就通過（除了斷言 `snapshotCreated` 為真的，如果有）。
- `cargo test -p speclink-remote` 與 `cargo test -p speclink-desktop` 既有的遠端整條鏈測試通過。
- `cargo test -p speclink-core`：本機封存照舊寫備份的測試（`a_local_archive_backs_up_the_spec_it_rewrites`）、`Workspace::is_local` 與 preflight 在沒有 workspace 時不查本機檔案的單元測試；本機封存的既有測試不改。沒有 workspace 時封存不寫檔的測試在 Host（`archive_writes_no_local_files`）：它要切換程序的工作目錄，放在 core 會在 crate 資料夾留下檔案。
- 用搜尋確認 `api/routes.rs`、`api/read_api.rs`、`api/context.rs`、`api/verb.rs` 裡，除了匯入與事件串流，沒有 `snapshot(`、`export(`、`begin_unit_of_work` 的呼叫。

**範圍**

- 在範圍內：上面的函式、轉接、server 的改接、封存不寫本機檔。
- 不在範圍內：路由的網址與線上型別、`speclink-cli`、桌面 app、匯入、事件串流、管理介面與備份。

## Risks / Trade-offs

- [回歸對照] 十幾條路由的讀法都換了。→ 既有的 server 整合測試逐條涵蓋（看板順序、流程設定、查詢、已封存、搜尋、drift、Context 快照、輪詢）；不改它們的斷言。evidence 的路由今天只有整條鏈的測試碰到，這次補一個直接的測試。
- [一致性] 一條路由的多次呼叫之間可能有人提交。→ D2 的順序讓 ETag 不會比內容新；用戶端輪詢會收斂。
- [記憶體] 長期持有的轉接為每個用過的範圍留一份內容。→ 示範 server 的定位可以接受；真正的宿主不用這個轉接。寫進專案能力狀態的文件影響。同一範圍的載入一次一個，所以同專案的訂閱者同時重讀時不會各自匯出。
- [一份壞文件拖累整個範圍] 轉接以匯出載入整個範圍，範圍裡只要有一份文件 TeamStore 解不開，流程設定、看板順序、evidence、whoami、sync-state 這些原本只讀一份文件的路由也會一起回 500。→ 接受：觸發條件是 TeamStore 本身已經存了壞資料；要避開就得讓單一文件改走快照，會打破「每個請求一份固定載入」。
- [範圍識別的長度] 範圍識別有 256 位元組的上限，project 裡的 `/` 與 `%` 跳脫後各多 2 位元組；project 加 repo 太長時，whoami、sync-state 這類原本只取快照的路由也會回 500。→ 接受：改寫前全部引擎路由就有同樣的上限；要根治得在建專案與 repo 時驗證長度，不在這個變更的範圍。
- [跨平台] 新增的程式不碰路徑與程序。封存不寫檔的測試用暫存的工作目錄，Windows 上刪除時沿用既有測試的重試設定。
- [和進行中的變更衝突] workflow-config-commands 正在改同一個檔的流程設定寫入路由。→ 宣告相依，排在它之後；這個變更不動那條路由的邏輯，只把它取版本的呼叫換成 `scope_version`。
- [儲存解耦] 完成後示範 server 的讀寫只剩兩處認得 TeamStore，離「引擎與宿主套件只認得三個函式」只差搬移。

## 稽核結論

任務 6.2 用 sharp-edges 清單（壞蛋、懶惰的開發者、搞混的開發者三個角度）看過新增的介面。結論：找到 1 個要修的地方，已修並補測試；其餘六項都成立。

- **來自網址的輸入只組得出自己那一類的鍵**：成立。變更名稱、capability、已封存名稱、artifact 路徑都先經 `speclink_store_port::keys` 的建立函式；能帶多段路徑的兩種（已封存文件、變更的 artifact）再用 `parse_key` 反解，名稱與種類都要對得上才用。`..`、`/`、`archive` 這類名稱組不出鍵，當作找不到或沒有。測試：`request_names_cannot_reach_another_kind_of_document`。
  - **修正**：已封存的 capability 清單與變更的 delta capability 清單，前綴原本由輸入組出後以 `expect` 建立。前面的鍵檢查保證它不會失敗，但新的呼叫端可能漏掉檢查而 panic。改成建不出前綴就回「沒有」；`prefix()` 只收常數（型別是 `&'static str`）。
- **長期持有的轉接不會把一個範圍的內容回給另一個範圍**：成立。載入的內容以範圍識別為鍵存放，填入的內容取自同一個識別拆出的 TeamStore 範圍。測試：`one_adapter_serves_two_scopes_without_mixing_them`。第 1 輪修正補上：範圍識別與 TeamStore 範圍一對一（project 段跳脫），project key 含 `/` 時也不會落到別的範圍。測試：`a_scope_id_names_exactly_one_teamstore_scope`、`two_scopes_that_join_to_the_same_text_stay_apart`；pinned view 只服務自己的範圍：`a_pinned_view_serves_its_own_scope_only`。
- **revision 比對沒有「先回舊內容再更新」的路徑**：成立。每次呼叫先取快照，只有已載入內容的 revision 等於這次快照的 revision 才重用，否則用這次的快照重新載入後才回答。兩個請求同時載入時，較舊的載入可能蓋過較新的；下一次呼叫的 revision 比對會發現而重新載入，不會回傳比那次呼叫開始時舊的內容。測試：`a_commit_made_straight_to_the_teamstore_is_read_back`。
- **看板順序的三種拒絕都沒有任何寫入**：成立。內容過大不呼叫儲存；預期範圍版本不符不提交；比對之後有人寫了看板順序時，提交的版本比對失敗、儲存不變、事件沒有落地。測試：`an_oversized_board_order_never_reaches_the_store`、`a_stale_board_order_write_commits_nothing`、`a_board_order_written_by_another_writer_meanwhile_is_a_conflict`。
- **錯誤訊息不帶出文件全文**：成立。找不到的訊息只帶請求裡的名稱；evidence 損壞只帶 JSON 解析器的位置訊息；流程設定的 422 沿用既有的解析錯誤文字（和改寫前相同）；儲存的錯誤沿用既有的對映。
- **封存在沒有 workspace 時沒有其他寫本機檔案的路徑**：成立。封存的兩個守門在 root 為空時本來就不碰檔案；提交階段的備份、清單檔、開工標記與舊格式 touched 記錄都改成只在有 workspace 時處理；一併封存討論的 `close_if_finished` 只用儲存。測試：`archive_writes_no_local_files`。第 1 輪修正補上 discard 的同類問題，測試：`discard_removes_no_local_file`。
- **參數可能被對調**（搞混的開發者）：接受，不改。`document_version(port, scope, change, artifact)` 的兩個字串對調時會讀不到文件，帶正數 `If-Match` 的寫入就全部變成 409；`command_routes` 裡「兩個寫入帶同一個版本、後到的回 409」的既有測試，先到的那筆也會失敗，所以錯用會被測試抓到。`execute_at_version` 與 `put_board_order` 的 `expected` 是範圍版本，型別是 `Version`（第 1 輪修正統一）。

## Migration Plan

沒有資料搬移，線上格式不變。要退回時還原這個變更的 commit。

## Open Questions

- `snapshotCreated` 在 CLI 與 Desktop 的用法（D7）。實作的第一步查清楚。
  - **查證結果（任務 1.1）**：不影響流程判斷，只少印一行。
    - 對照基準：改寫前 `cargo test -p speclink-server` 全部通過——lib 單元測試 36 條、`it` 整合測試 362 條（1 條 ignored）、`postgres_store` 3 條因沒有 PostgreSQL 略過。
    - 斷言封存回應 `snapshotCreated` 的 server 測試：沒有。server 的整合測試沒有任何一條讀這個欄位。
    - 其他讀到這個欄位的地方：
      - `crates/host/speclink-server/src/api/routes.rs`：把引擎結果的 `snapshot_created` 原樣放進回應。
      - `crates/adapters/speclink-cli/src/verbs/lifecycle.rs`：遠端封存把它轉回引擎的結果（缺席讀作 false），交給共用的渲染。
      - `crates/adapters/speclink-verbs/src/render/lifecycle.rs`：只在 true 時多印一行「Snapshot created for unarchive support.」。不影響結束碼與其他輸出。
      - `crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs`：用模擬的 server 回 `true`，比對遠端與本機輸出相同。它不經真的 server，這次不受影響。
      - `crates/protocol/speclink-protocol/src/command.rs`、`crates/protocol/speclink-remote/tests/it/typed_client.rs`：線上型別的序列化測試，不受影響。
      - `apps/desktop/src-tauri/src/remote.rs` 與 `lib.rs`：遠端封存的回應原樣轉給前端；前端（`apps/desktop/src`、`packages/ui/src`）沒有讀這個欄位。
      - `apps/desktop/core/src/verbs.rs`：本機封存（有 workspace），不受影響。
    - 結論：經 server 封存後，CLI 少印「Snapshot created for unarchive support.」這一行；Desktop 沒有可見差別。
- 已封存清單對很大的封存量（參考量 165 個）一次讀取 330 個鍵是否需要分批。量測沿用 `port_cost` 測試的做法加一筆；超過 1 秒時停下來回報，不自行加分頁。
  - **量測結果（任務 6.1，`port_cost` 的 `the_archived_list_is_one_list_and_one_read`）**：不需要分批。
    - 資料：本專案的 `speclink/` 資料夾，307 個已封存變更（比參考量多將近一倍）。
    - 對儲存的呼叫：1 次列出（前綴 `archive/changes/`）、1 次讀取，讀取帶 614 個鍵、讀回 3,113,825 位元組。
    - 耗時：三次量測 447、665、683 毫秒（未最佳化的 debug 建置、記憶體版儲存），都低於 1 秒。
    - 讀回的位元組幾乎都是各封存的 `tasks.md`：任務數要從它算出來。回應本身只帶計數，不帶內容。
