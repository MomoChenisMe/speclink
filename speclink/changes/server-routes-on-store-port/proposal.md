## Why

討論 embedded-host-needs 的第 7 個變更 host-kit-client-door 要把官方示範 server 的請求處理搬進宿主套件，讓任何宿主掛一條路由就能服務原版的 CLI 與 Speclink Desktop。宿主套件只認得儲存接口的三個函式（列出、讀取、提交）。

盤點示範 server 的路由後，發現它們分成兩類：

- 約 40 條把請求轉成引擎命令再執行。引擎已經經儲存接口讀寫，這些可以直接搬。
- 約 12 條不經引擎，直接呼叫舊的儲存層（TeamStore）的快照與整包匯出：流程設定的讀取、看板順序的讀寫、evidence、單份正式規格、已封存的清單與內容、搜尋、Context 快照、drift，以及變更清單與單筆變更回應裡補上的幾個欄位。計算範圍版本號（回應的 ETag 與「有沒有變」的輪詢）也是直接問 TeamStore。

第二類不先改寫就搬不動。所以第 7 個變更在提案時拆成兩個：這個變更先把第二類改成只用三個函式，並收成不依賴 HTTP 框架的函式；下一個變更 host-kit-client-door 再把全部路由搬進宿主套件。這次只動示範 server 與 Host，線上的回應不變，既有的 server 測試就是對照。

盤點時另外找到兩個問題，一併處理：

- **經 server 封存會在 server 程序所在的資料夾寫檔。** 引擎封存前會把正式規格的舊內容備份到本機的 `.speclink/snapshots/`。經 Host 執行時沒有 workspace，路徑變成相對於程序的工作目錄。內嵌的宿主不該因為封存而在自己的程序目錄多出檔案。
- **單筆變更的回應為了列出 delta 的 capability，匯出了整個範圍。** 它需要的只是那個變更底下的鍵。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（間接：下一個變更才讓他們用到）；維護示範 server 的人。CLI 與 Speclink Desktop 的使用者看不到差別。
- **使用情境**：連到遠端專案時的每一個讀取畫面與動詞——看板、已封存頁、搜尋、設定頁、`speclink drift`、apply 與 review 用的 Context 快照。
- **既有規格的對照**：host-runtime 的「engine 動詞經儲存接口執行」只涵蓋引擎命令；「TeamStore 轉接成儲存接口」規定轉接綁定一個範圍、一次動詞執行最多匯出一次。server-verb-api、remote-board-order、server-policy-write、client-protocol 規定這些路由的線上行為，這個變更不改它們。沒有正式規格規定「不經引擎的讀取也只用儲存接口」。

Source discussion: embedded-host-needs（cut 7 的第一塊；拆分的理由見上，結論的文字等沒有變更在實作中時再更新）。

## What Changes

- `speclink-host` 新增一個模組，收進「不經引擎命令的文件讀寫」：每個函式收儲存接口與範圍，回傳線上的型別與範圍版本號，不依賴 axum。涵蓋：
  - 範圍版本號（ETag 與輪詢用）。
  - 流程設定的原文與版本。
  - 看板順序的讀取與整份寫入（帶預期的範圍版本、上限 1 MiB、事件 `board-order-updated`）。
  - 某個變更的 evidence。
  - 單份正式規格。
  - 已封存的清單、單份已封存文件、已封存變更的 capability 清單。
  - 搜尋。
  - Context 快照。
  - 變更清單與單筆變更回應補上的欄位（認領人、開工時間、建立者、來源討論、delta 的 capability）。
  - 單份文件的讀取與「帶預期文件版本的寫入」前的版本比對。
- `speclink-host` 的 drift 計算改成收儲存接口，不再收 TeamStore。
- 示範 server 的對應路由改成呼叫這些函式；執行引擎命令的共用函式改成直接經儲存接口的執行入口。之後示範 server 的專案路由裡，直接碰 TeamStore 的只剩兩處：匯入（留給變更 import-export-on-port）與推送事件的串流（它讀的是 TeamStore 的 outbox，儲存接口沒有讀回事件的操作）。
- TeamStore 的轉接改成可以長期持有：一個轉接服務多個範圍；專案 revision 沒有改變時重用已載入的內容，不再每次匯出。原因是輪詢每幾秒就問一次範圍版本號，照現在「每個轉接載入整個範圍」的做法，每次輪詢都會匯出整個範圍。
- 引擎的封存在沒有 workspace 時不寫本機的備份檔，也不動本機的開工標記；回應的 `snapshotCreated` 在這條路徑上是 false。本機專案的封存不變。
- **線上行為**：全部路由的網址、請求、回應的欄位與型別、狀態碼、ETag 的值都不變。回應裡的各種版本號（範圍的、文件的）數值不變，因為轉接把 TeamStore 的 revision 原樣對到儲存接口的版本。
- **相容性影響**：
  - CLI 與 Speclink Desktop：沒有可觀察的差別，除了下面一項。
  - 遠端封存的回應 `snapshotCreated` 從「有備份時 true」變成一律 false。CLI 與 Desktop 對這個欄位的用法在實作時先查清楚；有畫面或輸出依賴它時，同批調整並在這裡補記。
  - 示範 server 的程序目錄不再因為封存而出現 `.speclink/snapshots/` 與開工標記的變動。
  - 讀取的一致性：今天每個路由讀的是一份快照；之後一個路由可能對儲存發出兩到三次呼叫，中間有別人提交時會看到新舊混合的內容。這是討論已接受的第一版行為；寫入仍由版本比對把關。
- **CLI 指令、設定欄位、技能**：都沒有新增或變更。

## Non-Goals

- **把路由搬進宿主套件、Node 的 HTTP 入口、握手、範圍清單、登入方式的宣告**：下一個變更 host-kit-client-door。
- **匯入**：仍然直接呼叫 TeamStore，留給變更 import-export-on-port。
- **推送事件的串流**：仍然讀 TeamStore 的 outbox。
- **管理介面與備份的匯出**：它們是示範 server 自己的功能，不進宿主套件。
- **讀取的嚴格一致**：不做跨多次呼叫的快照。
- **改變任何版本號的語意**：流程設定與看板順序回應裡的 `revision` 仍然是範圍版本號，artifact 的 `version` 仍然是文件版本。
- **收緊「帶預期版本的 artifact 寫入」在比對與提交之間的空窗**：維持今天的行為。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-runtime`: 新增「不經引擎命令的文件讀寫只用儲存接口」與「沒有 workspace 時封存不寫入本機檔案」；修改「TeamStore 轉接成儲存接口」（可以長期持有、服務多個範圍、依專案 revision 重用載入的內容）。

## Impact

- Affected specs: host-runtime（新增兩條、修改一條）
- Affected crates: `speclink-core`、`speclink-host`、`speclink-server`。`speclink-protocol`、`speclink-remote`、`speclink-cli`、桌面 app 不改（除非 `snapshotCreated` 的查證發現要調整）。
- Affected code:
  - New:
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/tests/documents.rs
  - Modified:
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/drift.rs
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/host/speclink-host/src/bridge.rs
    - crates/host/speclink-host/tests/port_cost.rs
    - crates/host/speclink-server/tests/it/api/query_routes.rs
    - crates/host/speclink-server/tests/it/api/sync_state.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/src/api/read_api.rs
    - crates/host/speclink-server/src/api/context.rs
    - crates/host/speclink-server/src/api/verb.rs
    - crates/host/speclink-server/src/state.rs
    - crates/host/speclink-server/src/main.rs
    - crates/engine/speclink-core/src/lifecycle/archive.rs
    - crates/engine/speclink-core/src/command/mod.rs
  - Removed: (none)
- 相依：排在變更 workflow-config-commands 之後（它正在改同一批路由，流程設定的寫入路由已由它改成引擎命令）。
- 三種形態下的結果：
  - 本機 repo：不適用。本機不經 Host 的這些函式；封存的備份照舊寫在專案裡。
  - 連到遠端的 checkout：可用，行為不變（除了 `snapshotCreated`）。
  - 沒有本機規格檔的內嵌宿主：這次不做。這些函式要等 host-kit-client-door 才經宿主套件對外；這個變更先讓它們能在任何儲存接口上執行。封存不寫程序目錄這一點，內嵌宿主經 Agent 工具的門立刻受益。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：沒有使用者看得到的變化要寫；專案能力狀態要記一句「遠端封存不留本機備份」。
