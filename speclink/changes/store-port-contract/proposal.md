## Why

規格離開程式碼 repo 之後，儲存必須自己補上 git 原本給的兩件事：一次提交改多份文件，以及兩個人不互相覆蓋。今天能做到這兩件事的只有 TeamStore 契約，而它要求 speclink 自己開資料庫連線、自己建表與遷移，文件種類是封閉的九種，實作只能用 Rust 寫。已經有自己的資料存取層（連線池、遷移工具、交易）的宿主用不上它。Node SDK 的自訂 JS 儲存有 31 個方法，沒有版本比對、沒有交易、事件被丟掉。

討論 embedded-host-needs 定案：儲存媒介整個交給宿主，speclink 只提供一份小的儲存契約（列出、讀取、提交三個函式）、一套任何實作都能跑的合格測試、一份記憶體版的參考實作。這是十六個變更的第一個；後面的變更（引擎改經這份契約讀寫、宿主套件、匯入匯出）都建在它上面，所以它要先落地。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（宿主的開發者）。他們實作三個函式，用合格測試證明自己的實作正確。間接受益的是宿主系統裡的 PO、PM 與工程師。
- **使用情境**：這個變更不對應任何 workflow 階段或技能。它只新增一個 Rust crate，還沒有任何程式呼叫它。
- **既有規格的對照**：最接近的是 teamstore-contract（speclink 持有連線的儲存契約）、store-abstraction（引擎內層儲存介面的相容性）、node-sdk（createEngine 的兩種儲存形式）。三份都沒有涵蓋「由宿主實作、以字串鍵定址、三個函式」的契約。

## What Changes

- 新增 crate `speclink-store-port`，放在 store crates 底下。它不依賴 `speclink-core`，也不依賴既有的 `speclink-store`。
- 定義儲存契約 `StorePort`：同步、object-safe、可跨執行緒共用的 trait，只有三個方法。
  - **列出**：給範圍與鍵的前綴，回傳範圍的版本號，以及每份相符文件的鍵、文件版本、最後修改時間，依鍵排序。
  - **讀取**：一次收多個鍵，逐一回傳內容、文件版本、最後修改時間；不存在的鍵回傳「沒有」，不算錯誤。
  - **提交**：一次帶全部的寫入與刪除、每份文件的預期版本（某個版本，或「必須不存在」）、事件、操作者、命令名稱與時間；回傳新的範圍版本號。
- 契約的三項保證：一次提交全部成功或全部不做；任何一份文件的版本對不上就整筆拒絕，並指出是哪一份；事件與寫入在同一次提交裡交給儲存。
- 文件用字串鍵定址。鍵的格式由契約的型別檢查，儲存不需要認得文件種類。
- 新增鍵的命名規則（純函式，不做 I/O）：進行中的變更、已封存的變更、正式規格、討論、已封存的討論、流程設定、詞彙表、看板順序、evidence 各有固定的前綴。
- 版本號的型別與範圍寫進契約：範圍版本號與文件版本都是 0 到 2^53−1 的整數，讓 JavaScript 的 number 能無損表示。
- 新增記憶體版的參考實作 `MemoryStorePort`，支援多個範圍，並記下每次提交的事件。
- 新增合格測試（conformance suite）：一個公開的入口函式，對任何 `StorePort` 實作跑同一組檢查並回傳報告。涵蓋版本衝突、整筆不寫、事件同批、併發提交只有一筆成功、批次讀取不會看到提交到一半的內容。
- 沒有 CLI 指令、設定欄位、技能或線上協定的變動。既有的人眼輸出與 `--json` 都不變，沒有相容性影響。

## Capabilities

### New Capabilities

- `store-port-contract`: 由宿主實作的儲存契約——三個函式的輸入與輸出、三項保證、鍵的格式與命名規則、版本號的型別，以及合格測試與記憶體版參考實作。既有的 teamstore-contract 規定的是 speclink 自己持有連線的契約（manifest、snapshot、Unit of Work、outbox、匯出匯入共十二個方法，文件種類封閉），store-abstraction 只保證引擎內層介面抽象化後行為不變，兩者都不適用於宿主在自己的交易裡完成提交的形態。

### Modified Capabilities

(none)

## Impact

- Affected specs: store-port-contract（新增）
- Affected crates: 新增 `speclink-store-port`。`speclink-core`、`speclink-host`、`speclink-server`、`speclink-node`、`speclink-cli`、既有的 store crates 與桌面 app 都不改。
- Affected code:
  - New:
    - crates/store/speclink-store-port/Cargo.toml
    - crates/store/speclink-store-port/src/lib.rs
    - crates/store/speclink-store-port/src/types.rs
    - crates/store/speclink-store-port/src/error.rs
    - crates/store/speclink-store-port/src/keys.rs
    - crates/store/speclink-store-port/src/memory.rs
    - crates/store/speclink-store-port/src/conformance/mod.rs
  - Modified:
    - Cargo.toml
    - Cargo.lock
  - Removed: (none)
- 三種形態下的結果（這個變更定義了文件的鍵，所以逐一寫明）：
  - 本機 repo：不適用。本機專案繼續用檔案實作，這個變更沒有接線。
  - 連到遠端的 checkout：不適用。示範用的 server 仍走 TeamStore，線上協定不變。
  - 沒有本機規格檔的內嵌宿主：這次不做，是已知缺口。契約與合格測試已經存在，但引擎還不會經它讀寫（由變更 engine-on-store-port 補上），Node 也還拿不到它（由變更 host-kit-agent-door 補上）。
- 文件影響（留給最後一個變更 docs-embedded-host 一次更新，這個變更不動使用者文件）：docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 要新增儲存接口與合格測試的說明；docs/server-store-drivers.zh-TW.md 與 docs/server-store-drivers.md 要改寫成儲存接口的範例；docs/product-status.zh-TW.md 與 docs/product-status.md 要列出已知缺口。
