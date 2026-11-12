## Context

- **儲存契約**只有列出、讀取、提交三個函式，明寫不含匯入匯出。列出可以用空的前綴取得整個範圍的鍵；提交可以帶任意多筆「必須不存在」的寫入與事件，全部成功或全部不做。
- **舊儲存層的匯入匯出**：TeamStore 有 `export`（回傳整包）與 `import`（只建立，或覆寫）。包的每份文件用九種文件種類之一標示，另有格式版本、範圍與專案版本號。
- **線上**：`POST /import` 收 `ImportBundle`（線上型別，九種文件種類的標示，嚴格拒絕未知欄位），示範用 server 檢查格式版本、範圍相符、逐份的摘要，再呼叫 TeamStore 的匯入（只建立）；非空的範圍回 `refused`。這條路由在宿主套件搬移時留在 server。沒有線上的匯出；匯出只有管理介面的下載與備份指令。
- **大小**：示範用 server 的轉交層讀本文的上限是 32 MiB，匯入的路由用同一個值。宿主套件的設定裡沒有任何大小。
- **Speclink Desktop**：`migrate_workspace` 用引擎的儲存介面把本機的規格資料夾組成 `ImportBundle` 再呼叫匯入。它收集進行中變更的 metadata 與 artifact、正式規格、討論、已封存的變更、流程設定、詞彙表；沒有收集 evidence 與看板順序，雖然線上的型別有這兩種。本機的看板順序存在各變更與討論自己的順序鍵裡，不是一份文件。
- **鍵與文件種類的對照**今天在過渡用的 TeamStore 轉接裡（把鍵換成文件種類、把文件種類換成鍵）；把鍵解析成種類的函式在儲存視圖裡。
- 匯入今天不產生事件，只在成功後直接通知事件串流。

限制：

- 討論的裁定：兩個門都能用；上限由宿主設定；不做壓縮；匯入維持只能進空的範圍。
- 儲存契約不加第四個操作。

## Goals / Non-Goals

**Goals:**

- 任何實作三個函式的儲存都能匯出與匯入，不需要額外的實作。
- 內嵌的宿主能服務 Speclink Desktop 的搬移，也能自己呼叫匯出與匯入。
- 匯入是一次提交；失敗時範圍仍然是空的。
- 搬移不遺失 evidence 與看板順序。

**Non-Goals:**

- 合併與覆寫、壓縮與串流、CLI 的動詞、示範用 server 的備份、版本歷史的搬移。

## Decisions

### D1: 匯出

`speclink_host::transfer::export(port, scope) -> Result<ScopeBundle, TransferError>`：

1. 以空的前綴列出，得到全部的鍵與範圍版本號 V。
2. 分批讀取（每批最多 200 個鍵），收集內容。列得到但讀不到的鍵代表範圍在中途被改動。
3. 再列出一次；版本號仍是 V 而且第 2 步沒有讀不到的鍵 → 完成。否則從第 1 步重來，最多 3 次；仍然不穩定 → 回報「暫時無法使用」，訊息說明範圍在匯出期間持續變動。
4. 回傳 `ScopeBundle { format_version: 2, scope_version: V, documents }`，每份文件是鍵、內容、內容摘要（SHA-256，和既有的摘要函式相同），依鍵排序。

空的範圍匯出成沒有文件的包，不是錯誤。

成本：N 份文件是 2 次列出加 N/200 次讀取。參考量（約 2300 份文件、22 MB）是 2 次列出加 12 次讀取。

- **否決：一次讀取全部的鍵** — 宿主的儲存常把讀取對成一個帶 IN 清單的查詢，上千個鍵的清單在一些資料庫有上限。
- **否決：不檢查中途的變動** — 備份會拿到新舊混合的內容。

### D2: 匯入

`speclink_host::transfer::import(port, scope, actor, bundle, at) -> Result<ImportReport, TransferError>`，依序：

1. 格式版本不是 2 → 拒絕（第一版在進入這個函式之前由 D3 換成第二版）。
2. 每份文件：鍵符合契約的格式；鍵是引擎認得的命名規則之一（用儲存視圖既有的鍵解析）；摘要等於內容的摘要；沒有重複的鍵。任何一項不過 → `InvalidBundle`，訊息指出第一個有問題的鍵與原因，不呼叫儲存。
3. 以空的前綴列出；有任何鍵 → `NotEmpty`。
4. 一次提交：每份文件一筆寫入，預期「必須不存在」；事件一筆 `scope-imported`，payload 是 `{ documents: <份數> }`；命令標籤 `scope-import`；操作者取自呼叫端。
5. 提交回報衝突（第 3 步之後有人寫入）→ `NotEmpty`。其他儲存失敗原樣回報。
6. 回傳新的範圍版本號與份數。

只有一次提交，所以沒有半套狀態。包裡沒有文件時，不提交，回報 0 份與目前的版本號。

「引擎認得的鍵」這個檢查，擋的是把任意的鍵塞進宿主的儲存。它不檢查內容的格式：壞的 metadata 匯入後會在讀取時回報，和今天相同。

### D3: 包的兩個版本

`speclink-protocol`：

- 第二版（新增）：`{ "formatVersion": 2, "documents": [ { "key", "content", "digest" } ] }`。匯出的回應多一個 `scopeVersion`。
- 第一版（既有的 `ImportBundle`）：不改。
- `POST /import` 的本文先讀 `formatVersion` 再決定用哪個型別解析。其他值 → `refused`（既有的錯誤）。

第一版換成第二版：把九種文件種類換成鍵，用既有的建立鍵的函式。這段對照今天在 TeamStore 的轉接裡；把它搬到 `transfer` 模組成為公開的函式，轉接改呼叫它，不留兩份。第一版裡的範圍與專案版本號：範圍必須等於請求的範圍（既有的檢查，保留）；專案版本號忽略（既有行為）。換不成鍵的文件（名稱含 `/` 等）→ `invalid_argument` 並指出該文件。

回應：既有的 `ImportReportResponse` 逐份列出文件種類與結果。第一版的請求照舊回這個型別；第二版的請求回 `{ "scopeVersion", "documents": <份數> }`。

### D4: 兩個門

Rust（`HostKit`）：

- `export_scope(&self, caller) -> Result<ScopeBundle, ExecError>`
- `import_scope(&self, caller, bundle) -> Result<ImportReport, ExecError>`

兩者的流程和執行命令相同：先問執行前掛鉤——命令名稱 `scope-export`（參數空物件）或 `scope-import`（參數 `{ documents, bytes }`：份數與內容的總位元組數；不含內容）——再執行；匯入成功後通知提交後掛鉤（命令名稱 `scope-import`、新的版本號、那一筆事件）。掛鉤拒絕時以 `refused` 結束，儲存不變。

這兩個名稱不是引擎命令，所以不進線上歸屬表；它們和看板順序的寫入一樣，是「不經引擎命令的文件讀寫」。

HTTP 入口：

- `GET /export`：回傳第二版的包。需要寫入的能力（`can_write`）。
- `POST /import`：需要寫入的能力。本文超過上限（D5）→ 413；之後依 D3 解析並呼叫 `import_scope`。
- 錯誤的對映：`InvalidBundle` → 400 `invalid_argument`；`NotEmpty` → 409 `refused`；格式版本不認得 → 409 `refused`；儲存的失敗照既有的對映。

匯出需要寫入能力的理由：它一次交出整個範圍；示範用 server 今天只讓管理者匯出。宿主要讓唯讀的人也能匯出時，自己呼叫 `export_scope`。

Node：`host.exportScope({ identity, scope })` 回傳包的物件；`host.importScope({ identity, scope, bundle })` 回傳 `{ scopeVersion, documents }`。`bundle` 可以是第一版或第二版。失敗時的錯誤碼：`invalid_argv`（包不合格）、`refused`（非空、掛鉤拒絕）、儲存的三種。

### D5: 大小上限

`HostKitOptions` 新增 `import_limit_bytes: Option<usize>`，沒有給時是 32 MiB（`33_554_432`）。

- HTTP 入口在 `POST /import` 先比本文的長度，超過就回 413，不解析。
- `import_scope`（直接呼叫）比的是內容的總位元組數，超過時 `InvalidBundle`。
- 入口拿到的是宿主的框架已經讀完的本文，所以宿主的框架自己的本文上限必須不小於這個值；這一點寫進型別的註解與文件。
- Node：`createHost({ limits: { importBytes } })`。值不是正整數時 `createHost` 同步拋出錯誤。
- 示範用 server：傳輸層的上限與這個設定用同一個常數。

匯出沒有上限：資料是宿主自己的儲存裡的，大小由它決定。

### D6: Speclink Desktop 的組包

`build_import_bundle` 的兩個補充：

- **evidence**：每個進行中的變更，引擎的儲存介面讀得到 evidence 時加一份。已封存變更的 evidence 是封存文件集的一份；確認既有的「已封存變更的全部文件」有沒有包含它，沒有就補上。
- **看板順序**：從各變更的 metadata 與各討論記錄讀出順序鍵，組成看板順序文件（格式是 `speclink-protocol` 既有的型別），至少有一個順序鍵時加進包裡。變更的 metadata 裡的 `board_rank` 原樣留著（遠端不讀它，留著無害，搬回本機時還在）。

Desktop 改送第二版：把收集到的文件換成鍵（用 D3 公開的對照）。握手之後先送第二版；Server 以「格式版本不認得」拒絕時，改送第一版重試一次（舊版 Server）。兩版的內容來自同一次收集。

組包的邏輯留在桌面的 Tauri 殼（它已經在那裡，而且只有 Desktop 用）；只把「文件種類換成鍵」共用。

- **否決：這次順便做 CLI 的匯入匯出動詞並把組包搬進共用的 crate** — 沒有需求來源；列為已知缺口。

### D7: 示範用 server

- 宿主套件的路由表加入 `/import` 與 `/export`，所以轉交的處理函式不再把 `/import` 留給 server。server 的匯入處理函式與它對 TeamStore 匯入的呼叫移除。
- 建立宿主套件時傳入匯入的上限（既有的常數）。
- 匯入成功後對事件串流的通知，改由既有的提交後掛鉤觸發（和其他寫入相同），不再由路由另外呼叫。
- 管理介面的匯出下載、備份與還原不動。

## Implementation Contract

**行為**

- 對任何通過合格測試的儲存：匯出一個範圍，再把得到的包匯入另一個空的範圍，兩個範圍列出的鍵相同、每份文件的內容相同。
- 匯入非空的範圍、摘要不符、鍵不是引擎認得的規則、重複的鍵：各自被拒絕，目標範圍沒有任何變動。
- 匯入成功時儲存收到正好一次提交，帶一筆 `scope-imported` 事件。
- 示範用 server：既有的匯入整合測試（第一版的包）不改斷言就通過；Speclink Desktop 既有的搬移整條鏈測試通過，而且遠端多了 evidence 與看板順序。
- 宿主以較小的上限建立宿主套件：超過的匯入得到 413，範圍不變。
- 執行前掛鉤對 `scope-import` 回答拒絕：匯入以 `refused` 失敗，範圍不變。

**介面與資料形狀**

- Rust：`speclink_host::transfer::{export, import, ScopeBundle, BundleDocument, ImportReport, TransferError, key_of_document}`；`HostKit::{export_scope, import_scope}`；`HostKitOptions.import_limit_bytes`。
- 線上：第二版的匯入包；`GET /export` 的回應；第二版匯入的回應。第一版不變。
- Node：`host.exportScope`、`host.importScope`、`CreateHostOptions.limits.importBytes`。

**失敗的樣子**

| 情況 | HTTP | 直接呼叫 |
|---|---|---|
| 本文或內容超過上限 | 413 | 包不合格 |
| 格式版本不認得 | 409 `refused` | 包不合格 |
| 摘要不符、鍵不合格、不認得的鍵、重複的鍵 | 400 `invalid_argument` | 包不合格 |
| 第一版的範圍與請求不符 | 409 `refused`（既有） | 不適用 |
| 目標範圍不是空的（含提交時才發現） | 409 `refused` | `refused` |
| 沒有寫入的能力 | 403 | 不適用（宿主自己決定誰能呼叫） |
| 掛鉤拒絕 | 409 `refused`，宿主的原因 | `refused` |
| 匯出期間範圍持續變動 | 503 `unavailable` | 暫時無法使用 |

**驗收**

- `cargo test -p speclink-host --test transfer`：用記憶體版儲存涵蓋 D1、D2、D3 與上表；往返的測試用本專案的規格資料夾當資料（沿用成本測試的載入方式）。
- `cargo test -p speclink-host --test http_door`：兩條路由、上限、能力、掛鉤。
- `cargo test -p speclink-server`：`import_api` 既有的測試不改斷言；新增 `GET /export` 的測試與「匯入後事件串流收到事件」。
- `cargo test -p speclink-desktop`：組包含 evidence 與看板順序；第二版失敗時退回第一版。
- Node：`transfer.spec.ts`。

**範圍**

- 在範圍內：上面列的介面、Desktop 的組包、示範用 server 的匯入改接。
- 不在範圍內：`speclink-core`、`speclink-cli`、store crates、備份與還原、使用者文件。

## Risks / Trade-offs

- [回歸對照] 匯入的路由換了實作。→ `import_api` 的既有測試逐條涵蓋（只建立、非空拒絕、reader 拒絕、過大、沒有半套狀態、匯入後各讀取面可讀）；不改斷言。
- [回歸對照] 匯入多了一筆事件。→ 刻意的變更；事件串流的測試新增一條。
- [提交的大小] 一次提交帶幾千筆寫入與幾十 MB。→ 宿主的儲存在一個交易裡寫入；參考量用記憶體版儲存與過渡的轉接各跑一次並記下耗時。轉接底下的舊 driver 若有單次寫入的限制，在任務裡回報，不自行分批（分批會破壞「不留半套」）。
- [記憶體] 整包在記憶體裡出現兩到三份（本文、解析後、提交）。→ 上限由宿主設定；預設值和今天相同。
- [跨平台] 沒有路徑與程序呼叫；鍵一律用 `/`。Desktop 組包時本機路徑換成鍵沿用既有的轉換。
- [匯出的一致性] 靠版本號重試，不是真正的快照。→ 三次都不穩定就失敗，不交出混合的內容。
- [儲存解耦] 匯入匯出從「每個 driver 各自實作」變成「用三個函式組出來」，宿主的儲存不用多做任何事。

## Migration Plan

沒有資料搬移。舊版的 Speclink Desktop 送第一版的包，照常可用。要退回時還原 commit。

## Open Questions

- 過渡用的 TeamStore 轉接能不能在一次提交裡寫入參考量的全部文件（三個舊 driver 各自的限制）。實作時量測；不能時停下來向使用者回報。
- 匯出的每批 200 個鍵是不是合適的值。實作時用參考量比較 100、200、500 三種的耗時，差異不大就維持 200。
