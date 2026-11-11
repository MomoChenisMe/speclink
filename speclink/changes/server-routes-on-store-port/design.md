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
- `DocumentsError` 有四種：找不到（帶訊息）、無效的設定（流程設定解析失敗）、儲存失敗（`PortError`）、內容損壞（evidence 無法解析）。server 把它們對到今天各路由回的狀態碼：404、422、依 `PortError` 既有的對映、500。

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

- 建立時只收 TeamStore，不綁範圍。每次呼叫的範圍識別是 `<project>/<repo>`，轉接把它拆成 TeamStore 的範圍；格式不合時回報後端錯誤。
- 每次列出或讀取先取一次快照（便宜），拿到專案 revision。這個範圍已載入的內容是在同一個 revision 載入的 → 重用；否則重新載入（一次匯出與逐份讀取）並記下 revision。
- 自己的提交成功後，丟掉該範圍已載入的內容。
- 每個範圍最多留一份已載入的內容，沒有其他淘汰規則。

示範 server 在啟動時建一個轉接，放進共用的狀態，全部請求共用。`api/verb.rs` 的 `run` 改成直接呼叫 `port_exec::execute`，執行後的 ETag 用 `scope_version`。

成本的變化：輪詢與「取版本」從一次快照變成「一次快照加比對」，沒有匯出。一個命令的執行在 revision 沒變時不用再匯出。記憶體多了每個用過的範圍一份內容；示範 server 的定位是示範與試用，接受這個代價，並寫進風險。

既有的 `bridge::execute` 與它的型別別名留著：Host 的雙路徑測試還在用它，移除 TeamStore 是最後階段的變更。它改成內部建一個不綁範圍的轉接，行為不變。

- **否決：每個請求建一個轉接** — 輪詢會變成每次匯出整個範圍。
- **否決：為「只取版本」在儲存接口加第四個操作** — 契約已定為三個；真正的儲存用很窄的前綴列出就很便宜，慢的只是這個過渡用的轉接。
- **否決：依時間淘汰已載入的內容** — 正確性由 revision 比對保證，時間只是多一個要測的維度。

### D7: 沒有 workspace 時封存不寫本機檔案

引擎的封存收到的執行情境沒有 workspace 時：

- 不寫 `.speclink/snapshots/` 底下的備份與清單檔。
- 不移除本機的開工標記檔。
- 結果的 `snapshot_created` 是假。
- 其餘步驟不變：驗證、合併 delta、寫正式規格、搬移變更的文件、一併封存討論。

寫入順序與失敗後的狀態：經儲存接口時，正式規格的寫入與文件的搬移本來就在同一次提交，失敗時什麼都沒寫，所以不需要本機備份當作復原的依據。本機專案（有 workspace）的順序與備份不變。

判斷用的是「執行情境有沒有 workspace」，不是儲存的種類：Node SDK 的 fs 形式有 workspace，照舊寫備份。

實作前先查 CLI 的遠端封存輸出與 Speclink Desktop 的封存結果畫面怎麼用 `snapshotCreated`。它只是多印一行時，遠端少那一行屬於刻意的變更，更新對應的測試期望值並補進 proposal 的相容性影響；它影響流程判斷時停下來回報。

## Implementation Contract

**行為**

- 對同一批資料，改寫前後每條路由的狀態碼、回應本文、`ETag` 標頭逐位元相同。例外只有封存回應的 `snapshotCreated`（D7）。
- 示範 server 處理匯入與事件串流以外的任何專案路由時，TeamStore 只經轉接被呼叫。
- 連續輪詢範圍版本號而沒有提交時，TeamStore 的匯出一次都不被呼叫。
- 經 Host 封存一個會改寫既有正式規格的變更後，程序的工作目錄沒有新增任何檔案。

**介面與資料形狀**

- `speclink_host::documents` 的函式（D1 的表）與 `DocumentsError`。
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
- `cargo test -p speclink-core`：封存在沒有 workspace 時不寫檔的測試；本機封存的既有測試不改。
- 用搜尋確認 `api/routes.rs`、`api/read_api.rs`、`api/context.rs`、`api/verb.rs` 裡，除了匯入與事件串流，沒有 `snapshot(`、`export(`、`begin_unit_of_work` 的呼叫。

**範圍**

- 在範圍內：上面的函式、轉接、server 的改接、封存不寫本機檔。
- 不在範圍內：路由的網址與線上型別、`speclink-cli`、桌面 app、匯入、事件串流、管理介面與備份。

## Risks / Trade-offs

- [回歸對照] 十幾條路由的讀法都換了。→ 既有的 server 整合測試逐條涵蓋（看板順序、流程設定、查詢、已封存、搜尋、drift、Context 快照、輪詢）；不改它們的斷言。evidence 的路由今天只有整條鏈的測試碰到，這次補一個直接的測試。
- [一致性] 一條路由的多次呼叫之間可能有人提交。→ D2 的順序讓 ETag 不會比內容新；用戶端輪詢會收斂。
- [記憶體] 長期持有的轉接為每個用過的範圍留一份內容。→ 示範 server 的定位可以接受；真正的宿主不用這個轉接。寫進專案能力狀態的文件影響。
- [跨平台] 新增的程式不碰路徑與程序。封存不寫檔的測試用暫存的工作目錄，Windows 上刪除時沿用既有測試的重試設定。
- [和進行中的變更衝突] workflow-config-commands 正在改同一個檔的流程設定寫入路由。→ 宣告相依，排在它之後；這個變更不動那條路由的邏輯，只把它取版本的呼叫換成 `scope_version`。
- [儲存解耦] 完成後示範 server 的讀寫只剩兩處認得 TeamStore，離「引擎與宿主套件只認得三個函式」只差搬移。

## Migration Plan

沒有資料搬移，線上格式不變。要退回時還原這個變更的 commit。

## Open Questions

- `snapshotCreated` 在 CLI 與 Desktop 的用法（D7）。實作的第一步查清楚。
- 已封存清單對很大的封存量（參考量 165 個）一次讀取 330 個鍵是否需要分批。量測沿用 `port_cost` 測試的做法加一筆；超過 1 秒時停下來回報，不自行加分頁。
