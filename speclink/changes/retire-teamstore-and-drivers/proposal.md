## Why

舊的儲存層（TeamStore 介面與它的三個 driver）在前面的變更之後已經沒有正式的使用者：

- 引擎經儲存接口（列出、讀取、提交三個函式）讀寫。
- 示範用 server 改用三個直接實作儲存接口的範例（變更 server-on-port-examples）。
- Node SDK 的舊 JS 儲存移除（變更 node-retire-legacy-store）。

還留著的東西是：

- 四個 crate：`speclink-store`（TeamStore 介面、記憶體版與它的合格測試，約 3200 行）與三個 driver（各約 1300 到 1600 行，加測試）。
- 宿主套件裡把 TeamStore 包成儲存接口的轉接（約 1280 行，其中 30 個測試）、一個只有測試在用的執行橋接，以及一份「兩條路徑結果相同」的測試。
- 幾個還從 `speclink-store` 借用的小東西：專案與儲存庫的識別型別、內容摘要的函式、第一版匯入包的版本常數。宿主套件與 Speclink Desktop 在用。
- 四份正式規格（teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store）與 host-runtime 裡三條描述轉接的需求。
- 流程設定的專案說明（`speclink/config.yaml` 的 context）仍然把 TeamStore 與三個 driver 寫成現行架構。之後每一份提案與設計都會讀到這段過期的說明。

這個變更把它們移除。

- **目標使用者**：speclink 的維護者與貢獻者；透過流程設定的專案說明讀架構的 AI agent。
- **使用情境**：讀程式、寫新的變更提案與設計時，只看到一套儲存的概念。
- **既有規格的對照**：四份舊儲存層的規格整份過期。host-runtime 的「Host 承擔 TeamStore 的 UoW 與 event commit」「engine 動詞經橋接於 TeamStore 上執行」「TeamStore 轉接成儲存接口」三條描述即將刪除的程式；其中兩個仍然成立的規則（專案與儲存庫怎麼對映成範圍識別字、第一版的文件種類怎麼換成鍵）只寫在即將刪除的那一條裡，要另立需求保留。host-runtime「不經引擎命令的文件讀寫只用儲存接口」、server-read-api「archived 列舉為 Store 契約的一部分」、server-identity「identity 儲存獨立且版本守門」、change-diff-scope 的兩條需求有提到 TeamStore 的句子。store-port-contract、store-port-examples、host-kit 不受影響。

Source discussion: embedded-host-needs（cut 20）。

## What Changes

**刪除四個 crate**

- `speclink-store`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres` 整個刪除，從 workspace 的成員移除。

**宿主套件**

- 刪除 TeamStore 的轉接、執行橋接與「兩條路徑」的測試。
- 專案與儲存庫的識別型別、內容摘要的函式搬進宿主套件，名稱與行為不變。宿主套件不再相依 `speclink-store`。
- 第一版匯入包的版本常數由宿主套件公開。
- 宿主套件裡原本經轉接塞資料的測試，改用記憶體版的儲存接口。

**其他 crate**

- Speclink Desktop 與 `speclink-remote`：改從宿主套件引用上面那幾樣，移除對 `speclink-store` 的相依。行為不變。
- 引擎、協定、示範用 server 裡提到 TeamStore 的註解與套件說明更新。

**CI**

- PostgreSQL 的那個 job 移除舊 driver 的測試那一行。新範例與 server 的那兩行不變。

**正式規格**

- 刪除四份整份過期的規格：teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store。引擎沒有「刪除一整個 capability」的 delta 操作（把需求全部移除會留下一份空的規格檔），所以這四份直接刪除資料夾，理由記在這份提案與設計裡。
- host-runtime：移除描述轉接的三條需求；新增兩條把仍然成立的規則留下來（專案與儲存庫對映成範圍識別字、文件種類與鍵的對應）；修改一條拿掉 TeamStore 的句子。
- server-read-api、server-identity、change-diff-scope：各修改提到 TeamStore 的句子，行為不變。

**流程設定的專案說明**

- `speclink/config.yaml` 的 context：兩條路徑的說法改成「本機專案」與「遠端專案」；技術棧與 Cargo workspace 的兩段改成儲存契約、三個範例與宿主套件。
- `rules.design` 第一條的「TeamStore contract 與 driver 歸 store crates」改成「儲存契約與範例歸 store crates」。

**使用者文件：只修連結**

- docs/product-status 中英兩版有四列的證據連結指向即將刪除的檔案（三個舊 driver 的測試與「兩條路徑」的測試）。文件的連結守門會因此失敗，所以這個變更把這幾個連結改指到對應的新測試。那四列的文字說明不動，留給最後一個變更改寫。

**相容性影響**

- 沒有使用者看得到的行為改變。CLI、示範用 server、Speclink Desktop、Node SDK 的行為都不變。
- 從原始碼引用 `speclink-store` 或三個舊 driver 的外部程式會編譯失敗。這四個 crate 沒有發布到 crates.io。
- **CLI 指令、`.speclink.yaml` 的欄位、技能、線上協定**：都沒有變更。`speclink/config.yaml` 只改 context 與一條規則的文字，不新增或移除欄位。

## Non-Goals

- **改寫 README 與使用者文件**：變更 docs-embedded-host。這個變更只修會讓守門失敗的連結。
- **LANGUAGE.md 的「儲存後端」詞條**：它的定義提到 TeamStore，和使用者文件的用詞一起在變更 docs-embedded-host 修改。
- **改變儲存契約、三個範例或宿主套件的行為**。
- **重新產生手冊**（speclink/manual）：手冊由規格產生，之後由手冊的技能重新產生。
- **清掉已封存變更與討論記錄裡的舊名稱**：它們是歷史記錄。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-runtime`: 移除「Host 承擔 TeamStore 的 UoW 與 event commit」「engine 動詞經橋接於 TeamStore 上執行」「TeamStore 轉接成儲存接口」；新增「專案與儲存庫對映成範圍識別字」「文件種類與鍵的對應」；修改「不經引擎命令的文件讀寫只用儲存接口」。
- `server-read-api`: 修改「archived 列舉為 Store 契約的一部分」（實作站點的說法）。
- `server-identity`: 修改「identity 儲存獨立且版本守門」（與規格文件的儲存分離的說法）。
- `change-diff-scope`: 修改「Apply 開始前記錄 host-local baseline」與「remote workspace 使用同一 host resolver」（各一句的用詞）。

整份刪除、不經 delta 的 capability：teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store。

## Impact

- Affected specs: host-runtime（移除三條、新增兩條、修改一條）、server-read-api（修改一條）、server-identity（修改一條）、change-diff-scope（修改兩條）；刪除 teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store 四份
- Affected crates 與 app: 刪除 `speclink-store`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres`；修改 `speclink-host`、`speclink-remote`、apps/desktop（Tauri 殼）；`speclink-core`、`speclink-protocol`、`speclink-server` 只改註解與套件說明。`speclink-cli`、`speclink-node`、儲存契約與三個範例不改。
- Affected code:
  - New: (none)
  - Modified:
    - Cargo.toml
    - Cargo.lock
    - crates/host/speclink-host/Cargo.toml
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/scope.rs
    - crates/host/speclink-host/src/binding.rs
    - crates/host/speclink-host/src/context.rs
    - crates/host/speclink-host/src/http/mod.rs
    - crates/host/speclink-host/src/projection.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/transfer.rs
    - crates/host/speclink-host/src/drift.rs
    - crates/host/speclink-host/src/change_diff.rs
    - crates/host/speclink-host/tests/documents.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/transfer.rs
    - crates/protocol/speclink-remote/Cargo.toml
    - crates/protocol/speclink-remote/tests/it/read_api.rs
    - crates/protocol/speclink-remote/tests/it/events_sse.rs
    - apps/desktop/src-tauri/Cargo.toml
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/tests/it/common/mod.rs
    - apps/desktop/src-tauri/tests/it/login_orchestration.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - crates/host/speclink-server/Cargo.toml
    - crates/host/speclink-server/Dockerfile
    - crates/engine/speclink-core/src/lifecycle/tasks.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - .github/workflows/ci.yml
    - speclink/config.yaml
    - speclink/specs/host-runtime/spec.md（只改 Purpose 那一段；需求由 delta 處理）
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed:
    - crates/store/speclink-store（整個資料夾）
    - crates/store/speclink-store-sqlite（整個資料夾）
    - crates/store/speclink-store-fs（整個資料夾）
    - crates/store/speclink-store-postgres（整個資料夾）
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/host/speclink-host/src/bridge.rs
    - crates/host/speclink-host/tests/bridge_dual_path.rs
    - speclink/specs/teamstore-contract/spec.md
    - speclink/specs/sqlite-team-store/spec.md
    - speclink/specs/serverfs-team-store/spec.md
    - speclink/specs/postgres-team-store/spec.md
- 相依：server-on-port-examples（示範用 server 要先離開舊儲存層，這是硬性的先後）與 node-retire-legacy-store（Node SDK 的型別宣告裡還有提到舊儲存層的註解，由那個變更移除；排在它之後，這個變更「搜尋不到舊名稱」的驗收才成立）。
- 三種形態下的結果：
  - 本機 repo：不適用。這個變更不改任何動詞的行為。
  - 連到遠端的 checkout：不適用。示範用 server 的行為不變。
  - 沒有本機規格檔的內嵌宿主：不適用。宿主套件對外的行為不變；專案與儲存庫的識別型別換了定義的位置，從宿主套件引用的名稱不變。
- 文件影響（留給最後一個變更 docs-embedded-host；這個變更只修連結）：
  - docs/product-status.zh-TW.md 與 docs/product-status.md：「SQLite TeamStore」「Server FS TeamStore」「PostgreSQL TeamStore」三列與「Command Runtime、Host 與 Protocol」一列的名稱、說明與限制要改寫；這個變更只把證據連結改指到新的測試。
  - docs/development.zh-TW.md 與 docs/development.md：repo 結構表裡 crates/store 那一列。
  - README.md 與 README.zh-TW.md：「目前狀態」裡「三種 TeamStore」的說法。
  - LANGUAGE.md 的「儲存後端」詞條。
  - 手冊（speclink/manual）裡由這四份規格產生的頁面要重新產生。
