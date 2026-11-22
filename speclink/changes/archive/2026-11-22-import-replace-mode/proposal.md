## Why

內嵌宿主的管理者把 repo 裡的規格資料夾匯入宿主之後，repo 還會繼續變：團隊不會在同一天全部改用宿主。管理者要能再匯入一次。現在 `host.importScope` 只接受空的範圍，所以宿主只有兩條路：

- 另外登記一份新代號的規格資料再匯入。代號不能改，舊的那一份留著，存取設定與工程師的連線（`remote.repo`）都要重做。
- 照 SDK 文件「先在你的儲存裡刪掉這個範圍的全部文件，再匯入」。這是兩步：中間連線中的用戶端會看到空的範圍，也可能寫入；版本號與通知用戶端都留給每個宿主自己處理。

另一個會撞到的情況：流程設定是範圍裡的一份文件（鍵 `config.yaml`）。管理者先調政策、再想匯入時，範圍已經不是空的，匯入被拒絕。

目標使用者是把 `@speclink/engine` 內嵌進自己系統的整合者，以及在宿主的管理頁操作的管理者（他看不到 repo，也不用 CLI）。對應的階段是把既有的本機專案搬進遠端專案，以及搬完之後、團隊完全改用宿主之前的那一段。

來源：討論 `embedded-host-needs-6` 的 cut 2。前兩輪討論都決定「匯入只能進空的範圍」；這次推翻的理由寫在 design.md。

## What Changes

- **`host.importScope` 多一個取代模式。** 呼叫多一個選填欄位 `mode`：`'create'`（預設，行為與現在相同）或 `'replace'`。取代模式在正好一次提交裡讓範圍的內容等於匯入包，沒有例外：
  - 匯入包有而範圍沒有的鍵：新增。
  - 兩邊都有而內容不同的鍵：更新。
  - 兩邊都有而內容相同的鍵：不重寫，版本與修改時間不變。
  - 範圍有而匯入包沒有的鍵：刪除。這包含 `config.yaml`，也包含匯入之後才在宿主裡建立的變更與討論。
- **同一個呼叫可以試跑。** 呼叫多一個選填欄位 `dryRun`。試跑做全部的檢查與比對，回報與真的執行相同的結果，但是不提交，也不通知 `afterCommit`。
- **回報多三個鍵的清單。** `HostImportReport` 多 `added`、`changed`、`removed`，各是鍵的陣列，依鍵的位元組順序排列。兩種模式都給。`documents` 仍是寫入的份數（新增加更新）。
- **`beforeCommand` 看得到模式。** `scope-import` 的參數多 `mode` 與 `dryRun`，兩種模式都給。命令名稱不變，`commandNames` 不變。試跑會讀範圍的內容，所以照樣先問 `beforeCommand`。
- **兩個新的拒絕，各有原因代碼：**
  - 取代模式加上沒有任何文件的匯入包：`code` 是 `invalid_argv`，`reason` 是 `empty_bundle`。一次錯誤的呼叫不能清空整個範圍。
  - 取代模式在列出之後、提交之前，另一方寫入了這次會動到的鍵：`code` 是 `refused`，`reason` 是 `scope_changed`。範圍不變，可以再執行一次。
- **只在程序內的呼叫提供。** HTTP 入口的 `POST /import` 與 Speclink Desktop 的「遷移到 Server」不變，仍然只收空的範圍。
- **SDK 文件（中英兩版）改寫三處：** 「匯出與匯入」說明兩種模式、試跑與回報；「在伺服端匯入」的「要重新匯入」改成取代模式，寫明它會刪掉匯入後在宿主裡做的全部改動，並附上重新套用流程政策的範例；匯入的原因代碼表多兩列。

### 相容性影響

- **不帶 `mode` 與 `dryRun` 的既有呼叫**：寫入的結果、失敗的 `code`、`reason` 與 `message` 逐字不變。
- **`HostImportReport`** 多三個屬性（`added`、`changed`、`removed`）。既有的 `scopeVersion` 與 `documents` 的意思不變。以整個物件做相等比對的宿主測試要更新。
- **`beforeCommand` 收到的 `scope-import` 參數**多 `mode` 與 `dryRun` 兩個鍵，經 HTTP 入口的匯入也有（`mode` 是 `create`、`dryRun` 是 `false`）。以整個參數物件做相等比對的掛鉤要更新；只讀 `documents` 與 `bytes` 的掛鉤不受影響。
- **只依命令名稱放行 `scope-import` 的宿主**：取代模式用同一個命令名稱。只有宿主自己的程式能傳 `mode`，原版的用戶端經 HTTP 入口傳不了，所以不會有使用者因此多出刪除的能力；要區分時讀 `params.mode`。
- **`scope-imported` 事件**：建立模式的 payload 不變；取代模式的 payload 多一個 `removed`（刪除的份數）。
- **HTTP 入口**：`GET /export` 與 `POST /import` 的請求、狀態碼與 body 都不變。
- 沒有新的 CLI 動詞、旗標或設定欄位；不影響生成的技能；Rust 端 `HostKit::import_scope` 的簽名多一個選項參數（workspace 內的呼叫端同批更新）。

### 三種形態下的結果

- **本機 repo**：不適用。匯入的目標是宿主的儲存；本機專案沒有範圍。`engine.exportScope()` 不變，它的包可以交給取代模式。
- **連到遠端的 checkout**：不適用。原版的用戶端只經 HTTP 入口匯入，那條路維持只收空的範圍。取代之後，連線中的用戶端收到範圍變動的通知並重新讀取，和一般的寫入相同。
- **沒有本機規格檔的內嵌宿主**：可用。宿主的程式呼叫 `host.importScope({ …, mode: 'replace' })`。示範用 Server 沒有對外的操作會用到取代模式，這次不加。

### 既有規格的掃描

`host-kit`「範圍的匯出與匯入」現在規定匯入只接受空的範圍，這次改寫成兩種模式。`node-sdk`「宿主的匯出與匯入方法」與「匯出與匯入的失敗帶原因代碼」規定 JS 端的呼叫、回報與原因代碼，這次都要改寫。`user-documentation`「SDK 文件說明伺服端匯入、釋放認領的掛鉤與蓋章指紋」規定文件寫「宿主先清空自己儲存裡這個範圍的全部文件再匯入」，這次改寫那一句。`workspace-migration`「import 端點僅限 CreateNew 且原子」管的是 HTTP 的匯入端點，這次不動它：取代模式不上線路。`store-port-contract` 的三個操作已經足夠（提交可以同時帶寫入與刪除，各自有預期的版本），不用改。

## Non-Goals (optional)

（design.md 會寫明目標與不做的項目。）

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-kit`: 「範圍的匯出與匯入」從只有建立模式改成兩種模式，並新增一條需求規定取代模式與試跑。
- `node-sdk`: 「宿主的匯出與匯入方法」多 `mode`、`dryRun` 與回報的三個清單；「匯出與匯入的失敗帶原因代碼」多 `empty_bundle` 與 `scope_changed` 兩個代碼。
- `user-documentation`: 「SDK 文件說明伺服端匯入、釋放認領的掛鉤與蓋章指紋」的重新匯入步驟改成取代模式與重新套用流程政策的範例。

## Impact

- Affected specs: `host-kit`、`node-sdk`、`user-documentation`
- Affected crates: `speclink-host`（匯入的實作、宿主套件的入口、HTTP 入口的呼叫點）、`speclink-node`（呼叫的參數、回報與型別說明）；`speclink-server` 的還原與 `apps/desktop` 的遷移走既有的建立模式，不改程式
- Affected code:
  - Modified:
    - crates/host/speclink-host/src/transfer.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/tests/transfer.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/transfer.spec.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - New: (none)
  - Removed: (none)
