## Context

`speclink-host` 的橋接層（`bridge.rs`）讓引擎跑在 TeamStore 上。它的做法是：

1. `BridgeStore::materialize` 取一個快照，再用 `export` 匯出整個範圍來得知有哪些文件（TeamStore 的快照只能逐份讀取，不能列出），然後把每份文件讀回，放進記憶體的對照表。
2. 引擎在這個對照表上跑。寫入被收成 `StagedOp`，預期版本取自載入時的版本。
3. 有寫入時，開 unit of work、放入全部的寫入、連同事件一次 `commit`。

第 2、3 步的形狀是對的，要保留。第 1 步是問題：每個動詞都載入整個範圍。`drift.rs` 的 `spec_drift` 也用同一個視圖。

變更 store-port-contract 新增了 `speclink-store-port`：`StorePort` 的列出、讀取、提交，以及 `keys` 模組的鍵命名規則。這個變更讓引擎經它讀寫。

限制：

- 引擎內層的儲存介面（`speclink-core` 的 `Store` trait）不改。它的方法簽名是同步的、不回傳儲存錯誤（讀取回傳 `Option`）。
- 示範用的 server 與既有測試在這個變更完成後都要照常運作，所以現有的三個 driver 要透過轉接繼續用。
- 這個變更不引入每次呼叫帶身分的入口。那是變更 host-kit-agent-door 的範圍。

## Goals / Non-Goals

**Goals:**

- 引擎能在任何 `StorePort` 實作上執行全部的命令。
- 只讀取用到的文件；需要多份時一次讀取。
- 一個動詞的全部寫入與事件收成一次提交。
- 證明三個操作足以供應引擎儲存介面的全部方法。
- 用真實大小的資料量測一次，決定要不要加快取。
- 朝「storage 解耦的規格驅動引擎」靠攏：Host 的執行路徑只認得儲存接口，TeamStore 退到轉接後面。

**Non-Goals:**

- 不改引擎內層的儲存介面，不改 `speclink-fs`，不改 Node 的 31 個方法的 JS 儲存。
- 不改線上協定的形狀，不改 server 的其他路徑（讀取 API、事件串流、備份、ETag 的計算仍直接用 TeamStore）。
- 不做依範圍版本號的快取。量測結果出來之前不設計它。
- 不提供嚴格一致的讀取快照。純讀取的動詞在兩次儲存呼叫之間能看到別人剛提交的內容，第一版接受；寫入由提交時的版本比對把關。
- TeamStore 的轉接不追求通過儲存接口的合格測試（原因見 D5）。
- 不在儲存接口上做匯入匯出，不搬動 server 的請求轉換。

## Decisions

### D1: 新增引擎的儲存視圖 PortView

`port_view.rs` 新增 `PortView`，在 `&dyn StorePort` 與一個 `ScopeId` 上實作 `speclink_core::store::Store`。它的生命週期是一次命令執行，可見性是 `pub(crate)`，理由和今天的 `BridgeStore` 相同：視圖的寫入只有經執行入口才會提交，不能交給外部。

內部狀態：

- **列出的記憶**：以固定的前綴為單位——`changes/`、`specs/`、`discussions/`、`archive/changes/`、`archive/discussions/`。每個前綴在一次執行裡最多向儲存列出一次，結果（鍵、版本、最後修改時間）留在記憶裡。
- **讀取的記憶**：每個鍵最多向儲存讀取一次。「沒有」也記下來。
- **暫存的寫入**：每個鍵一筆，後寫的蓋掉先寫的。讀取與列出都先看暫存，所以同一次執行讀得到自己剛寫的內容，也列得到自己剛建立的文件、列不到自己剛刪除的文件。
- **基準版本**：第一次看到某個鍵時（列出或讀取）記下它的版本或「不存在」。預期版本一律取自基準，同一個鍵寫幾次都不變。

儲存介面的每個方法怎麼對到三個操作：

| 引擎的需要 | 做法 |
|---|---|
| 列出一類文件的名稱（變更、capability、已封存的變更、討論） | 列出對應的前綴，從鍵解析出名稱；不讀內容 |
| 某份文件是否存在 | 該前綴已列出過就查記憶；否則讀取那一個鍵 |
| 讀一份文件 | 讀取那一個鍵 |
| 讀一批文件（全部變更的 metadata、全部討論） | 先列出，再用一次讀取取得全部還沒讀過的鍵 |
| 寫一份文件 | 放進暫存；基準還不知道時先讀取那一個鍵 |
| 刪一份文件 | 基準是「不存在」時只丟掉暫存；否則暫存一筆刪除 |
| 刪整個變更 | 列出 `changes/<name>/`（從 `changes/` 的記憶篩出），逐一暫存刪除 |
| 搬移（封存變更、封存討論） | 一次讀取來源的全部文件，暫存寫入目的鍵與刪除來源鍵 |
| 變更的最後修改時間 | 該變更底下各文件最後修改時間的最大值，換成秒 |

鍵一律用 `speclink_store_port::keys` 的建立函式產生。把鍵解析回名稱的函式放在 `port_view.rs` 裡（契約的 crate 沒有提供）；每一類鍵都有「建立再解析得到原值」的測試。

引擎的儲存介面讀取時回傳 `Option`，沒有地方回傳儲存錯誤。`PortView` 把第一個儲存錯誤記在內部，該次讀取回傳「沒有」；執行入口在引擎回來後先檢查這個記錄，有錯誤就以儲存錯誤結束，不看引擎的結果，也不提交。

- **否決：在引擎的儲存介面加上錯誤回傳** — 約四十個方法與全部呼叫端都要改，而且本機的檔案實作不需要。
- **否決：一開始就列出空的前綴取得全部的鍵** — 會把已封存變更的鍵（最大的一塊）每次都列出來。

### D2: 執行入口 port_exec::execute

`port_exec.rs` 新增：

- `execute(port, scope, ctx, cmd)`：建立 `PortView`，呼叫引擎的 `execute`，再依結果處理。
- 回傳 `PortExecution { outcome, events, scope_version }`。`scope_version` 在沒有提交時是「沒有」。
- 失敗是 `ExecError::Command(CommandError)` 或 `ExecError::Store(PortError)`，兩套錯誤不合併。

流程與失敗後的狀態：

1. 引擎回傳錯誤 → 回傳 `Command` 錯誤，暫存的寫入全部丟掉，儲存沒有任何變動。
2. `PortView` 記到儲存錯誤 → 回傳 `Store` 錯誤，同上。
3. 暫存是空的且沒有事件 → 不提交，`scope_version` 是「沒有」。
4. 其餘 → 組一個 `Commit`：`ops` 依鍵排序，`events` 依引擎回傳的順序，`actor` 取自執行情境，`command` 用既有的命令標籤，`at` 取呼叫當下的時間。提交一次。
5. 提交回傳衝突或其他錯誤 → 回傳 `Store` 錯誤。因為契約保證整筆不寫，儲存沒有半套狀態。

時間由 Host 取（引擎不讀時鐘）。入口另有一個收時間參數的內部版本，測試用它得到固定的結果。

操作者：既有的執行情境只有顯示名稱。這個變更把顯示名稱同時當成 `Actor` 的 id，匿名時用 `anonymous`。不會變的 id 由之後帶身分的入口提供。

領域事件對到 `Event { name, payload }` 的方式沿用 `commit.rs` 既有的對映（事件名稱與 camelCase 的內容），抽成不依賴 TeamStore 型別的函式，兩邊共用同一份。

### D3: 既有的橋接入口改成轉接加執行入口

`bridge::execute(store: &dyn TeamStore, ctx, cmd)` 的簽名保留，server 的呼叫點不用改。它的內容改成：用 `ctx` 的專案與 repo 建立 `TeamStorePort`（D5），再呼叫 `port_exec::execute`。

- `BridgeStore` 與 `materialize` 移除。同一件事不留兩套實作。
- `BridgeError::Store` 的內容從 `StoreError` 改成 `PortError`。`BridgeExecution.revision` 的值取自提交回傳的範圍版本號。
- `drift::spec_drift` 改用 `PortView`（經 `TeamStorePort`），結束時丟掉視圖，不提交。
- `commit.rs` 的 `begin_unit_of_work` 與 `commit_with_events` 因為這次改動不再有呼叫端，移除；事件對映的函式保留並改成 D2 的共用形狀。

### D4: Server 的錯誤對映

`speclink-server` 的 `api/error.rs` 新增 `PortError` 到 `ApiError` 的對映，取代橋接路徑上原本的 `StoreError` 對映：

| PortError | 狀態碼與 reason |
|---|---|
| `Conflict` | 409、revision_conflict；`message` 是「鍵: expected …, actual …」 |
| `Unavailable` | 503，沿用既有的暫時無法使用訊息 |
| `Backend` | 500 |

`StoreError` 本身的對映保留，server 其他直接用 TeamStore 的路徑還在用。

TeamStore 的六類錯誤經轉接後只剩三類。實作時要盤點三個 driver 在 `snapshot`、`export`、`history`、`commit` 回傳 `NotFound` 與 `PermissionDenied` 的情況：沒有這種情況就對到 `Backend`；有的話，在轉接裡把訊息原樣帶出，並在 server 測試補上對應的案例。

### D5: TeamStore 的轉接 TeamStorePort

`teamstore_port.rs` 新增 `TeamStorePort`，用一個 `&dyn TeamStore` 與一個固定的 `Scope` 建立，實作 `StorePort`。它是過渡用的，在收尾的變更隨 TeamStore 移除。

- **範圍**：轉接綁定建立時給的 `Scope`。它提供對應的 `ScopeId`（專案與 repo 以 `/` 相接）；收到別的 `ScopeId` 時回傳 `Backend` 錯誤。
- **鍵與文件種類**：鍵和 `DocumentId` 的九種雙向對應，依 store-port-contract 的鍵對照表。已封存討論的名稱段原樣對應。對不到任何一種的鍵：讀取回傳「沒有」、提交回傳 `Backend` 錯誤。
- **列出**：TeamStore 只有 `export` 能列舉。轉接在第一次列出時匯出一次並把結果留在自己身上，之後的列出與讀取都從它回答。轉接每次執行建立一個，所以一個動詞最多匯出一次——和今天的成本相同，但不再逐份讀回。自己提交之後，這份記憶作廢。
- **版本**：文件版本是 TeamStore 的文件 revision；範圍版本號是快照的專案 revision。
- **最後修改時間**：TeamStore 沒有便宜的方式取得，轉接一律回報 0。今天的橋接對變更的修改時間也是回報 0，行為不變。
- **提交**：開一個 unit of work（帶 `command` 與 `actor` 的顯示名稱），放入全部的 `ops`，事件轉成 event record（時間用 `at`），`commit`。`RevisionConflict` 轉成 `PortError::Conflict`，鍵由 `DocumentId` 換回。

轉接不通過儲存接口的合格測試，也不要求通過：最後修改時間是 0；同一個專案底下的範圍共用版本號。它有自己的測試（鍵的雙向對應、衝突的轉換、事件進 outbox、一次執行只匯出一次），而且 server 的整套測試會經過它。

- **否決：替三個 driver 各寫一份真正的 `StorePort` 實作** — 它們在收尾的變更會改寫成範例，現在做兩次。
- **否決：每次列出都重新匯出** — 一個動詞會匯出好幾次，比今天慢。

### D6: 量測測試 port_cost 與快取的判定

`tests/port_cost.rs` 把本專案的 `speclink/` 資料夾載入 `MemoryStorePort`（檔案路徑換成鍵，封存的位置依鍵對照表），外面包一層計數的儲存，記下每個命令的列出次數、讀取次數、讀取的鍵數與位元組數、提交次數。

代表性的命令：列出變更、單一變更的狀態、執行計畫、列出討論、新增一份 artifact、勾選一個任務、封存一個變更、搜尋討論。需要特定變更的命令用測試自己種進去的變更，不依賴 repo 當下有哪些進行中的變更。

測試斷言（會讓測試失敗）：

- 列出變更、單一變更的狀態、列出討論：讀取的鍵沒有任何一個在 `archive/` 底下。
- 每個命令：同一個前綴最多列出一次，同一個鍵最多讀取一次。
- 寫入類的命令：提交正好一次。

測試印出每個命令的數字。實作的人把這張表與結論寫進本文件新增的「量測結果」一節。判定規則：

- 上面三項斷言都成立，而且列出變更、單一變更的狀態、列出討論這三個命令對儲存的呼叫各不超過 10 次 → 結論是「不加快取」。
- 任何一項不成立 → 停下來回報，回到討論 embedded-host-needs 修正後面的計畫。

10 次這個門檻是這份設計訂的，依據是「宿主的儲存每次呼叫大約是一次資料庫查詢」。

### D7: 雙路徑測試擴大到全部的儲存操作

既有的 `tests/bridge_dual_path.rs` 比對「本機檔案」與「橋接」兩條路徑的結果。這個變更：

- 既有的五個測試保留，仍經 `bridge::execute`（現在底下是轉接加新的視圖）。
- 新增 `tests/port_dual_path.rs`：同樣的內容分別放進檔案實作與 `MemoryStorePort`，比對結果、錯誤碼與事件種類。涵蓋查詢、寫入、刪除變更、封存變更（搬移）、封存討論、寫流程會讀寫的 evidence。
- 新增一個覆蓋表測試：列出引擎儲存介面的每個方法，各有至少一個經 `PortView` 的斷言。這是「三個操作足以供應全部方法」的證明；有方法做不到時，測試不能寫，實作的人要停下來回報。

## Implementation Contract

**行為**

- Server 執行任何動詞時，不再把整個範圍讀回記憶體；讀取類的動詞不讀封存文件的內容（經真正的儲存接口時）。經 TeamStore 轉接時，一個動詞最多匯出一次。
- 動詞的結果、錯誤碼、領域事件與今天相同。例外是提案列出的兩項相容性影響。
- 用 Rust 內嵌的宿主能呼叫 `port_exec::execute`，把自己的 `StorePort` 交給引擎。

**介面**

- `speclink_host::port_exec::{execute, PortExecution, ExecError}`：公開。
- `speclink_host::teamstore_port::TeamStorePort`：公開（server 的測試會用到）。
- `speclink_host::port_view::PortView`：`pub(crate)`。
- `speclink_host::bridge::{execute, BridgeExecution, BridgeError}`：保留；`BridgeError::Store` 的內容改成 `PortError`。
- `speclink-host` 新增對 `speclink-store-port` 的依賴。

**失敗的樣子**

- 引擎拒絕：`ExecError::Command`，儲存沒有變動。
- 讀取途中儲存失敗：`ExecError::Store`，不提交。
- 提交衝突：`ExecError::Store(PortError::Conflict)`，帶鍵、預期、實際；經 server 是 409 與 revision_conflict。
- 轉接收到對不到文件種類的鍵要寫入：`PortError::Backend`，訊息指出那個鍵。

**驗收**

- `cargo test -p speclink-host` 全部通過，包含既有的雙路徑測試、新的 `port_dual_path`、覆蓋表測試、`port_cost` 的三項斷言。
- `cargo test -p speclink-server` 全部通過（背景跑；`review_api` 的既有 flaky 不算）。
- 在 `speclink-host` 與 `speclink-server` 搜尋不到 `BridgeStore` 與 `materialize(` 的呼叫（`projection.rs` 的同名函式是別的東西，不在此列）。
- 設計文件有「量測結果」一節，含數字表與結論。

**範圍**

- 範圍內：`speclink-host` 的新模組與橋接、drift 的改接；`speclink-server` 的錯誤對映；上述測試。
- 範圍外：引擎的程式、`speclink-store-port` 的契約（發現契約不夠時停下來回報，不在這裡改）、store crates、Node 綁定、CLI、線上協定、使用者文件。

## Risks / Trade-offs

- [回歸對照] 橋接視圖整個換掉，約五百行的對應邏輯重寫 → 既有的雙路徑測試與 server 的整套整合測試是對照；新的 `port_dual_path` 補上刪除、封存與 evidence。CLI 的 golden 不受影響，因為引擎與 CLI 都沒改。
- [跨平台] 鍵一律用 `/`，不經路徑型別；`port_cost` 從磁碟載入時把 Windows 的分隔符換成 `/`，並用 `CARGO_MANIFEST_DIR` 逐段組出資料夾位置。
- [錯誤被吞] 引擎的讀取回傳 `Option`，儲存錯誤只能記在視圖裡，引擎會把它當成「沒有」繼續跑 → 執行入口在引擎回來後一定先檢查，有錯誤就不提交。測試用一個在第 N 次呼叫失敗的儲存確認。
- [沒看過就寫要多讀一次] 引擎覆寫一份它沒讀過的文件時，視圖要先讀取來取得版本 → `port_cost` 會把這類多出來的讀取算進去；數量多時在量測結果裡列出，交給後面的計畫決定要不要在契約加上不檢查版本的寫入。
- [讀取不一致] 兩次儲存呼叫之間別人提交，讀取類動詞看到新舊混合 → 已定案接受。經 TeamStore 轉接時不會發生，因為轉接只匯出一次。
- [版本衝突的訊息文字變了] 有測試或用戶端比對 409 的 `message` 全文時會壞 → 實作時搜尋 server 與 CLI 的測試裡對這段文字的斷言，一併更新；reason 與狀態碼不變。
- [事件時間改用提交時間] outbox 記錄的時間和事件自己的時間差幾毫秒 → 沒有消費端依賴這個差異；列為相容性影響。
- [契約還沒實作完] 這個變更依賴 store-port-contract；它的 apply 若調整契約，這裡的文件要跟著改 → 已記錄依賴，先後順序由 plan 排。

## Open Questions

- 三個 driver 是否會在執行動詞的路徑上回傳 `NotFound` 或 `PermissionDenied`（D4），實作時盤點。
- 10 次的門檻是否合理，等量測的實際數字。
