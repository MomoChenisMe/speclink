## Why

儲存契約（列出、讀取、提交三個函式）今天只有兩個實作：記憶體版的參考實作，與把 JS 函式接進來的橋接。沒有任何一個實作把資料存進真的資料庫或檔案。

這造成兩個問題：

- **整合者沒有可以照著寫的例子。** 要把 speclink 內嵌進自己系統的人，要在自己的資料庫交易裡實作「一次提交改多份文件、版本對不上就整筆拒絕、事件和寫入同一次提交」。記憶體版沒有交易，看不出這三件事在 SQL 裡要怎麼做。
- **示範用 server 離不開舊的儲存層。** 它的資料存在三個舊的 driver（SQLite、Postgres、檔案系統）裡，三個都實作舊的 TeamStore 介面。server 靠一層轉接把 TeamStore 包成三個函式；這層轉接回報的最後修改時間永遠是 0，整個專案共用一個版本計數器，而且過不了儲存契約的合格測試。

討論的裁定是：三個舊 driver 不再提供，改用三個函式重新實作，定位是給使用者參考的範例。這個變更先把三個範例寫出來並測好，不接線；讓示範用 server 改用它們是下一個變更。

- **目標使用者**：要實作儲存接口的整合者（讀範例、複製範例）；示範用 server 的維護者。
- **使用情境**：整合者在寫自己的三個函式之前，先看一個同類型儲存的完整範例與它的測試。
- **既有規格的對照**：store-port-contract 規定三個操作、三項保證、合格測試與記憶體版參考實作，它的邊界明寫「止於契約本身」，不含任何有持久化的實作。sqlite-team-store、serverfs-team-store、postgres-team-store 規定的是舊 TeamStore 介面的三個 driver，條款綁在舊介面的概念上（manifest、Unit of Work、outbox 游標），不能沿用。host-kit 規定需要「至少送達一次」的宿主在提交的同一次交易裡把事件寫進自己的事件表，但沒有任何實作示範這件事。所以這是新的 capability。

Source discussion: embedded-host-needs（cut 18）。

## What Changes

**三個新的 crate，各是一個儲存接口的範例**

| crate | 儲存 | 用到的函式庫 |
| --- | --- | --- |
| `speclink-store-port-sqlite` | 一個 SQLite 檔案 | rusqlite（內建 SQLite） |
| `speclink-store-port-postgres` | 一個 PostgreSQL 資料庫的一個 schema | postgres |
| `speclink-store-port-fs` | 一個資料夾 | fs4（檔案鎖） |

三個都：

- 只實作列出、讀取、提交三個函式，通過儲存契約的合格測試（16 項檢查全部通過，沒有略過）。
- 文件帶真實的最後修改時間；每個範圍有自己的版本號。
- **把事件和寫入放在同一次交易裡**，寫進範例自己的事件記錄。每個範圍的事件有從 1 開始、每筆加 1 的序號。
- 另外提供三個契約之外的方法，給持有儲存的宿主用：讀某個序號之後的事件、清掉某個序號以前的事件、檢查儲存是否可用。
- 在資料的固定位置寫一個識別標記與配置版本。開啟時只認自己的標記：空的位置就初始化；標記是自己的就開啟；其他情況（別的標記、較新的配置版本）一律拒絕，不改動任何東西，並回報讀到的標記。
- 不相依舊的儲存層（`speclink-store`）。

**這個變更不接線**

- 示範用 server 仍然用舊的三個 driver。沒有任何既有程式改用新的 crate。
- 舊的三個 driver 與 TeamStore 不動。

**CI**

- PostgreSQL 的那個 job 多跑新的 Postgres 範例的測試；「測試回報略過就算失敗」的守門涵蓋它。

**相容性影響**

- 沒有。這個變更只新增三個 crate 與它們的測試。
- **CLI 指令、設定欄位、技能、線上協定、Node SDK**：都沒有變更。

## Non-Goals

- **讓示範用 server 改用範例**：變更 server-on-port-examples。
- **移除舊的 driver 與 TeamStore**：變更 retire-teamstore-and-drivers。
- **讀取或轉換舊 driver 的資料**：範例不認得舊的配置，只會拒絕開啟並回報它讀到的標記。舊資料的搬法（備份與還原）在變更 server-on-port-examples。
- **把範例放進 npm 套件或對 Node 開放**：範例是 Rust 的，給 Rust 的宿主直接用、給其他語言的宿主當參考。
- **正式環境的保證**：連線池、TLS、跨版本的遷移、保存每個版本的內容、多個宿主實例之間的事件傳遞都不做。範例的說明要寫明這些限制。
- **把「讀回事件」加進儲存契約**：契約維持三個函式。讀事件的方法是範例自己的，不是契約的一部分。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

- `store-port-examples`: 三個有持久化的儲存接口範例（SQLite、PostgreSQL、檔案系統）共同的行為——通過合格測試、事件記錄、配置的識別與守門、重新開啟與中斷後的一致性、兩個儲存物件開同一份資料時的行為。沒有既有規格涵蓋它：store-port-contract 只到契約與記憶體版；三份舊的 driver 規格綁在 TeamStore 介面上，會在變更 retire-teamstore-and-drivers 移除。

### Modified Capabilities

(none)

## Impact

- Affected specs: store-port-examples（新增）
- Affected crates 與 app: 新增 `speclink-store-port-sqlite`、`speclink-store-port-postgres`、`speclink-store-port-fs`。`speclink-store-port`、`speclink-core`、`speclink-host`、`speclink-server`、`speclink-cli`、`speclink-node`、apps/desktop 與舊的 store crates 都不改。
- Affected code:
  - New:
    - crates/store/speclink-store-port-sqlite/Cargo.toml
    - crates/store/speclink-store-port-sqlite/src/lib.rs
    - crates/store/speclink-store-port-sqlite/tests/it/main.rs
    - crates/store/speclink-store-port-postgres/Cargo.toml
    - crates/store/speclink-store-port-postgres/src/lib.rs
    - crates/store/speclink-store-port-postgres/tests/it/main.rs
    - crates/store/speclink-store-port-fs/Cargo.toml
    - crates/store/speclink-store-port-fs/src/lib.rs
    - crates/store/speclink-store-port-fs/tests/it/main.rs
  - Modified:
    - Cargo.toml
    - Cargo.lock
    - .github/workflows/ci.yml
  - Removed: (none)
- 相依：沒有。它只新增 crate，可以和 node-retire-legacy-store 同時做。
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案直接讀寫規格資料夾，不經儲存接口。
  - 連到遠端的 checkout：這次不做。示範用 server 還沒改用範例，用戶端看不到任何變化；由變更 server-on-port-examples 接上。
  - 沒有本機規格檔的內嵌宿主：可用。用 Rust 內嵌的宿主可以直接把範例交給宿主套件，或照著它寫自己的實作。已知缺口：沒有 JavaScript 寫的範例；用 Node 內嵌的宿主要自己用資料庫函式庫寫三個函式，範例只能當參考。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - docs/server-store-drivers.zh-TW.md 與 docs/server-store-drivers.md：改寫成「儲存接口的範例」——三個範例在哪裡、各自的資料表或檔案配置、一次提交在 SQL 裡怎麼寫、事件記錄怎麼用、範例的限制。
  - 「把 Speclink 內嵌進你的系統」：宿主要做的第一件事（實作三個函式並通過合格測試）連到這三個範例。
  - docs/development.zh-TW.md 與 docs/development.md：repo 結構表裡 crates/store 的說明。
  - docs/product-status.zh-TW.md 與 docs/product-status.md：列出「沒有 JavaScript 寫的範例」這個缺口。
