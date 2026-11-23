## Why

內嵌引擎的宿主用 `beforeCommand` 掛鉤決定每一次呼叫能不能做。但 HTTP 入口（`host.handleRequest`）裡有九條讀取不經過引擎命令，直接讀儲存，所以不問掛鉤：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived`、`GET /archived/{datedName}/capabilities`、`GET /archived/{datedName}/artifacts/{path}`、`POST /context`、`GET /changes/{name}/drift`。其中 `POST /context` 回整個範圍的文件，`GET /changes/{name}/drift` 回變更的 design、tasks 原文與完成記錄。結果是：掛鉤對某個身分一律拒絕時，`GET /changes` 回 409 與宿主的原因，這九條卻回 200 與內容。宿主因此要同時理解兩條權限路徑，而且拿不到掛鉤給的拒絕原因。

目標使用者是內嵌 `@speclink/engine` 並開放 HTTP 入口給 speclink CLI 與 Speclink Desktop 的宿主開發者。使用情境是「讀取狀態」的每一個階段：看板、已封存瀏覽、規格內文、搜尋、技能取得文件快照，以及 CLI 的 drift 檢查。

這是討論 embedded-host-needs-7 五個變更裡的第 2 個（G4 的權限那一半）。它要等 `archived-list-and-detail-reads`：兩條列出已封存的路由用那個變更加的 `list --archived` 的命令名稱與參數問掛鉤。

## What Changes

- 入口的九條讀取在讀儲存之前先問 `beforeCommand` 一次。掛鉤拒絕時回應是 409、reason 為 `refused`、訊息是宿主給的原因，儲存沒有收到任何呼叫。
- 五條用既有的命令名稱，宿主現有的規則自動生效：
  - `GET /config` 用 `workflow-config-show`。
  - `GET /specs/{capability}/document` 用 `show`，參數是 `item`（capability 名稱）與 `itemType`（`spec`）。
  - `GET /archived/{datedName}/artifacts/{path}` 用 `artifact-cat`，參數是 `artifact`（網址裡的文件路徑）與 `change`（帶日期的名稱）。
  - `GET /archived` 與 `GET /archived/{datedName}/capabilities` 用 `list`，參數裡 `archived` 是 true。
- 四條用新的命令名稱，並加進 `commandNames`（和既有的 `put-board-order` 同一種做法）：
  - `GET /search` 用 `workspace-search`，參數是 `query`。
  - `POST /context` 用 `context-snapshot`，請求指定了變更時參數帶 `change`。
  - `GET /board-order` 用 `get-board-order`，沒有參數。
  - `GET /changes/{name}/drift` 用 `spec-drift`，參數是 `change`（網址裡的變更名稱）。
- `commandNames` 從 57 個變成 61 個。
- 不改的部分：握手、輪詢（`GET /sync-state`）、兩個身分查詢、範圍清單、登入方式的宣告與 `POST /skills` 仍然不問掛鉤。它們不回範圍裡的文件內容；輪詢每幾秒一次，問掛鉤的成本不合理。範圍清單裡沒有這個專案時仍然回 404、不問掛鉤。
- 文件的寫入（`PUT /changes/{name}/artifacts/...`）改成先問掛鉤，再讀文件的版本來比對 `If-Match`。原本的順序相反：被掛鉤拒絕的呼叫端帶不相符的 `If-Match` 時，收到的是 409 `revision_conflict` 與文件的實際版本，掛鉤完全沒有被問，所以能探出文件存不存在和它的版本。這是品質關卡第一輪找到的。
- 一條路由仍在問掛鉤之前讀一次儲存，和流程設定的寫入一樣列為例外：一般的 `GET /plan` 先讀看板順序才組出命令。被拒絕時回應仍是 409，不帶範圍的內容。
- 不新增 CLI 指令、旗標、設定欄位；不涉及生成的技能與 Agent 指令。讀取不提交，所以 `afterCommit` 不受影響。

三種形態下的結果：

| 形態 | 結果 |
| --- | --- |
| 本機 repo | 不適用：本機專案不經過 HTTP 入口，也沒有掛鉤 |
| 連到遠端的 checkout（示範用 Server） | 行為不變：示範用 Server 沒有設定 `beforeCommand` |
| 連到遠端的 checkout（內嵌宿主的入口） | 可用：這九條讀取會照宿主的掛鉤放行或拒絕；被拒絕時 CLI 與 Speclink Desktop 收到 409 與宿主的原因，和現在 `GET /changes` 被拒絕時相同 |
| 沒有本機規格檔的內嵌宿主 | 可用：`host.handleRequest` 的每一條讀寫都經過 `beforeCommand` |

相容性影響：

- **BREAKING（只影響有設定 `beforeCommand` 的宿主）**：掛鉤只放行自己認得的名稱時，升級後 `workspace-search`、`context-snapshot`、`get-board-order`、`spec-drift` 會被拒絕，Speclink Desktop 的搜尋、看板順序與 CLI 的文件快照、drift 檢查會收到 409，直到宿主把四個名稱加進權限表。照 `commandNames` 檢查權限表的宿主，測試會在升級時失敗並指出缺的名稱。
- 掛鉤對 `show`、`artifact-cat`、`list`、`workflow-config-show` 的既有規則，現在也會作用在對應的入口讀取上。原本靠「這幾條不問掛鉤」讀到內容的身分，升級後會被拒絕；這是這個變更要的結果。
- 文件的寫入：`If-Match` 不相符時，掛鉤現在也會被問一次（原本在比對失敗時不被問）。掛鉤拒絕時回應是 409 `refused`，不再是 409 `revision_conflict`；掛鉤放行時回應與原本相同。
- 沒有設定 `beforeCommand` 的宿主與示範用 Server：每一條路由的狀態碼、本文與 ETag 不變。
- 人眼輸出與 `--json` 輸出都不變；線上格式不變。

掃描過的相關規格（沒有列進下面清單的）：server-read-api 的「正式規格內文可讀」「archived 瀏覽三端點」「workspace 全文搜尋與桌面語意對齊」規定回應的內容與最低角色，這次不改；client-protocol 的錯誤 reason 清單已有 `refused`；reference-server 不設定掛鉤，行為不變；host-kit「範圍與能力由宿主的函式決定」的 404 規則不變。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `host-kit`: 修改「HTTP 入口處理原版用戶端的請求」（經過掛鉤的範圍從「引擎命令與看板順序的寫入」擴大到入口讀寫範圍內容的每一條路由）；新增「入口的文件讀取先問執行前掛鉤」（含文件的寫入先問掛鉤、再比對版本）
- `node-sdk`: 修改「SDK 匯出命令名稱清單」（宿主套件自己的名稱從三個變成七個，總數從 57 變成 61）

## Impact

- Affected specs: `host-kit`、`node-sdk`
- Affected code:
  - Modified:
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/adapters/speclink-cli/tests/it/door.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/engine.spec.ts
    - crates/adapters/speclink-node/__test__/handle-request.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New: （無）
  - Removed: （無）
