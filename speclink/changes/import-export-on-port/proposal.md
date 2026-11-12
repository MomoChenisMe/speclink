## Why

把一個本機專案搬到遠端，靠的是匯入：用戶端把整個規格資料夾包成一包送到 Server，Server 一次寫進一個空的範圍。反方向是匯出：把一個範圍的全部文件拿出來，做備份或搬到別處。今天這兩件事都綁在舊的儲存層（TeamStore）上：

- **內嵌引擎的宿主沒有匯入匯出。** 宿主套件的入口不處理匯入；示範用 server 的匯入路由留在 server 自己手上，直接呼叫 TeamStore。Speclink Desktop 的「把本機專案搬到 Server」對內嵌的宿主因此不能用。
- **大小上限是示範用 server 裡的常數**（32 MiB）。宿主沒辦法調整。參考量：一個有 165 個已封存變更的規格資料夾約 31 MB，已經貼著上限。
- **匯入包的格式用一張封閉的文件種類清單。** 儲存契約已經改成字串鍵，匯入包卻還要求每份文件標明它是九種裡的哪一種。
- **Speclink Desktop 組匯入包時漏了兩樣。** 逐任務的 evidence 與看板的順序沒有被放進去，搬到遠端後這兩樣就不見了。
- **匯出只有管理介面的下載。** 沒有線上協定的路由，宿主的程式也拿不到。

這是把示範用 server 從舊儲存層拆開的倒數第二步：做完之後，專案路由裡直接呼叫 TeamStore 的只剩推送事件的串流。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（搬入既有的專案、做備份）；用 Speclink Desktop 把本機專案搬到 Server 的開發者。
- **使用情境**：第一次把一個本機專案變成遠端專案；宿主定期匯出做備份或搬到另一個環境。
- **既有規格的對照**：workspace-migration 規定匯入端點只接受空的範圍且原子、Desktop 組包的內容。store-port-contract 明寫契約不含匯入匯出。host-runtime 寫著示範用 server 的專案路由直接呼叫 TeamStore 的只剩匯入與事件串流。host-kit 沒有匯入匯出，也沒有任何大小的設定。node-sdk 沒有對應的方法。

Source discussion: embedded-host-needs（cut 16）。

## What Changes

**建在三個函式上的匯出與匯入**（`speclink-host`）

- 匯出：列出整個範圍，分批讀取全部文件，回傳一個包——每份文件的鍵、內容與內容摘要，加上匯出當時的範圍版本號。匯出期間範圍被改動時重試；一直在變就回報暫時無法完成。
- 匯入：只接受空的範圍。檢查每份文件的摘要、鍵的格式、鍵是不是引擎認得的命名規則、有沒有重複的鍵；然後用**一次提交**寫入全部文件（每一筆都要求「必須不存在」）並帶一筆事件。任何檢查不過，或提交被拒絕，範圍都維持空的。
- 儲存契約本身不變：仍然只有三個函式。匯入匯出是用它們組出來的。

**包的格式**

- 新增第二版：每份文件用字串鍵標示，不再用文件種類。
- 匯入仍然接受第一版（文件種類），由入口換成鍵；所以舊版的 Speclink Desktop 仍然能搬專案。
- 匯出一律給第二版。

**兩個門**

- HTTP 入口：`POST /import`（從示範用 server 搬進宿主套件）與新增的 `GET /export`。
- Agent 工具的門（Rust 與 Node）：宿主的程式直接呼叫匯出與匯入，不經 HTTP。
- 兩者都先問執行前掛鉤（命令名稱 `scope-import` 與 `scope-export`），匯入成功後通知提交後掛鉤。
- 匯入需要寫入的能力；匯出也需要寫入的能力（它一次交出整個範圍，包含全部歷史）。

**大小上限由宿主決定**

- 建立宿主套件時新增一個選填的設定：匯入的大小上限（位元組）。預設 32 MiB。超過時在解析之前以「內容過大」拒絕。
- 匯出沒有上限。
- 不做壓縮：宿主的路由層本來就能壓縮。

**Speclink Desktop**

- 組匯入包時補上 evidence 與看板的順序（從各變更與討論的順序鍵組出看板順序文件）。
- 改送第二版的包。Server 不認得第二版時（舊版 Server），退回第一版。

**示範用 server**

- 匯入的路由改由宿主套件處理；server 裡直接呼叫 TeamStore 匯入的程式移除。本文大小的傳輸上限維持 32 MiB，並把同一個值給宿主套件。
- 管理介面的匯出下載與備份還原不動（它們在變更 retire-teamstore-and-drivers 處理）。

**相容性影響**

- 用戶端：匯入的請求與回應的形狀不變（第一版照舊可用）；錯誤碼不變——非空的範圍是 `refused`、摘要不符是 `invalid_argument`、過大是 413。
- 匯入的事件：今天匯入不產生事件；之後提交帶一筆 `scope-imported`，已連線的用戶端會因此重新整理。
- 新增 `GET /export`：舊版 Server 沒有它，回 404。
- Speclink Desktop 搬專案之後，遠端多了 evidence 與看板順序。
- **CLI 指令、設定欄位、技能**：都沒有新增或變更。

## Non-Goals

- **匯入到非空的範圍、合併、覆寫**：維持只能進空的範圍。
- **壓縮、分段上傳、串流**：一次請求一個 JSON。超過上限的專案由宿主調高上限。
- **CLI 的匯入匯出動詞**：這次不做，寫成已知缺口。搬專案的入口仍是 Speclink Desktop 與宿主的程式。
- **示範用 server 的備份、還原與管理介面的匯出下載**：變更 retire-teamstore-and-drivers。
- **匯出時保留每份文件的版本號與歷史**：包只有內容。匯入後版本從新的範圍重新開始。
- **跨範圍或跨宿主的同步**。
- **匯入時改寫內容**（例如換專案名稱）。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-kit`: 新增「範圍的匯出與匯入」與「匯入的大小上限由宿主設定」。
- `host-runtime`: 修改「不經引擎命令的文件讀寫只用儲存接口」（示範用 server 的專案路由直接呼叫 TeamStore 的只剩事件串流）。
- `client-protocol`: 新增「以鍵標示文件的匯入包與匯出回應」。
- `workspace-migration`: 修改「import 端點僅限 CreateNew 且原子」（由宿主套件處理、接受兩版格式、帶事件）與「Bundle 組裝涵蓋本地 workspace 全集」（補上 evidence 與看板順序）。
- `node-sdk`: 新增「宿主的匯出與匯入方法」。

## Impact

- Affected specs: host-kit（新增兩條）、host-runtime（修改一條）、client-protocol（新增一條）、workspace-migration（修改兩條）、node-sdk（新增一條）
- Affected crates 與 app: `speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-server`、`speclink-node`、apps/desktop（Tauri 殼）。`speclink-core`、`speclink-cli`、store crates 不改。
- Affected code:
  - New:
    - crates/host/speclink-host/src/transfer.rs
    - crates/host/speclink-host/tests/transfer.rs
    - crates/adapters/speclink-node/__test__/transfer.spec.ts
  - Modified:
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/host/speclink-host/src/http/mod.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/error.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/host/speclink-server/src/app.rs
    - crates/host/speclink-server/src/state.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/src/api/forward.rs
    - crates/host/speclink-server/tests/it/api/import_api.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - apps/desktop/src-tauri/src/remote.rs
  - Removed: (none)
- 相依：沒有硬性的先後。它和 station-status-and-landing、host-free-fields 改到宿主套件的同一批檔案，建議排在 host-free-fields 之後依序做。
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案沒有範圍可以匯入匯出；把本機專案搬到遠端是 Speclink Desktop 的動作，它送出的包補上了 evidence 與看板順序。
  - 連到遠端的 checkout：可用。Speclink Desktop 的搬移對示範用 server 與內嵌的宿主都能用。已知缺口：CLI 沒有匯入匯出的動詞。
  - 沒有本機規格檔的內嵌宿主：可用。宿主的程式直接呼叫匯出與匯入，也能經 HTTP 的入口服務用戶端；大小上限由宿主設定。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 「把 Speclink 內嵌進你的系統」與 Node SDK 文件：匯出與匯入的方法、大小上限的設定、路由層的本文上限要不小於它、壓縮由路由層做。
  - 動詞與旗標的參考文件：`GET /export`、第二版的包。
  - 專案能力狀態：CLI 沒有匯入匯出動詞的缺口。
