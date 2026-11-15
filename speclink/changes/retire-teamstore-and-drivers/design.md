## Context

在這個變更開始時（前兩個變更已完成），舊的儲存層剩下這些地方還在用：

| 位置 | 用到什麼 | 用途 |
| --- | --- | --- |
| 宿主套件 `teamstore_port.rs` | 整個 TeamStore 介面 | 把 TeamStore 包成儲存接口的轉接（正式程式約 500 行，測試 30 個）。沒有正式的呼叫端 |
| 宿主套件 `bridge.rs` | TeamStore、範圍、版本 | 在 TeamStore 上執行一個引擎命令。只有測試呼叫 |
| 宿主套件 `binding.rs`、`context.rs`、`http/mod.rs`、`scope.rs` | 專案與儲存庫的識別型別 | 公開結構的欄位型別 |
| 宿主套件 `projection.rs`、`documents.rs`、`transfer.rs` | 內容摘要的函式 | `projection` 另外把它轉出，CLI 的一個測試從那裡引用 |
| 宿主套件 `drift.rs`、`projection.rs` 的測試 | 舊的記憶體儲存 | 經轉接塞測試資料 |
| 宿主套件 tests/bridge_dual_path.rs | 舊的記憶體儲存與多個型別 | 「本地檔案與 TeamStore 兩條路徑結果相同」的 6 個測試 |
| Speclink Desktop `remote.rs` | 內容摘要的函式、第一版包的版本常數 | 組匯入包 |
| `speclink-remote` 與 Speclink Desktop 的測試 | 少數殘留的引用 | 前一個變更改過建構方式之後留下的 |

另外有四個 crate 本身、四份正式規格、流程設定的專案說明，以及散在引擎、協定與 server 的幾處註解與套件說明。

引擎的封存沒有「刪除一整個 capability」的操作：delta 把一份規格的需求全部移除之後，會留下一份只有 Purpose 的空規格檔。

## Goals / Non-Goals

**Goals:**

- workspace 裡沒有 `speclink-store` 與三個舊 driver，也沒有任何程式引用它們。
- 仍然需要的三樣東西（識別型別、內容摘要、第一版的版本常數）由宿主套件提供，對外的名稱與行為不變。
- 被刪除的測試所保護的行為，只要還存在，就有對應的測試留著。
- 流程設定的專案說明與正式規格描述現在的架構。

**Non-Goals:**

- 不改任何對外的行為。
- 不改寫使用者文件（只修會讓守門失敗的連結）。
- 不動 LANGUAGE.md、手冊、已封存的變更與討論記錄。

範圍內：四個 crate 的刪除、`speclink-host`、`speclink-remote`、Speclink Desktop 的 Tauri 殼、幾處註解、CI、`speclink/config.yaml`、正式規格、docs/product-status 兩個檔的連結。範圍外：其餘全部。

## Decisions

### D1: 還在用的型別與函式搬進宿主套件

- 專案與儲存庫的識別型別（`ProjectId`、`RepoId`）：定義搬到宿主套件的 `scope.rs`（前一個變更建立，放範圍識別字的函式）。建構方式與方法不變。前一個變更加的轉出改成直接匯出定義。
- TeamStore 的範圍型別（專案加儲存庫的一對）不搬：`scope.rs` 的兩個函式改成直接收專案與儲存庫，回傳範圍識別字；反向回傳專案與儲存庫的一對。
- 內容摘要的函式：定義搬到宿主套件的 `projection.rs`——它今天已經從那裡轉出，CLI 的測試與宿主套件自己的測試都從這個位置引用，所以對外的路徑不變。演算法不變：SHA-256，小寫十六進位，前綴 `sha256:`。宿主套件因此直接相依 `sha2`。
- 第一版包的版本常數：宿主套件 `transfer` 裡既有的常數改成公開，Speclink Desktop 改用它。
- 宿主套件的 Cargo.toml 移除 `speclink-store`。

替代方案：把這幾樣放進儲存契約的 crate。不採用——儲存契約不知道「專案與儲存庫」這個概念，它只認範圍識別字；內容摘要是匯入匯出與投影用的，也不是契約的一部分。

### D2: 刪除轉接、橋接與它們的測試

刪除 `teamstore_port.rs`、`bridge.rs` 與 tests/bridge_dual_path.rs。刪之前先處理測試：

- **轉接的 30 個測試**：測範圍識別字的（跳脫、反向、形式不合）搬到 `scope.rs`；測「文件種類與鍵的對應」的，`transfer` 已有同樣的測試就刪，沒有就搬過去；其餘都是轉接特有的行為（重用載入、請求用的視圖、版本取 TeamStore 的 revision），刪除。
- **「兩條路徑」的 6 個測試**：逐一對照宿主套件既有的「本地檔案與儲存接口兩條路徑結果一致」測試（規格 host-runtime「engine 動詞經儲存接口執行」的情境「兩條路徑結果一致」）。已經有同樣斷言的刪除；沒有的改寫成在記憶體版的儲存接口上跑，放進既有的那個測試檔。
- **`drift.rs` 與 `projection.rs` 裡經轉接塞資料的測試**：改成直接對記憶體版的儲存接口以鍵寫入。斷言不變。

任務的完成說明要列出這 36 個測試各自的去向：搬到哪裡、由哪個既有測試涵蓋、或因為是轉接特有的行為而刪除。

### D3: 刪除四個 crate

順序：先做完 D1 與 D2，確認 workspace 裡沒有任何 Cargo.toml 還列著這四個 crate，再刪除四個資料夾並從根目錄 Cargo.toml 的成員清單移除。

Cargo.lock 只會減少：rusqlite、postgres、fs4 仍然被三個範例使用，`sha2` 與 `chrono` 仍然被其他 crate 使用。

`speclink-remote` 的測試用相依 `speclink-store` 移除。Speclink Desktop 的相依 `speclink-store` 移除，兩處引用改成宿主套件的 `projection::content_digest` 與 `transfer` 的第一版常數。

### D4: 正式規格

**直接刪除四份**：teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store 的資料夾。不寫 delta：把需求全部移除的 delta 在封存後會留下四份空的規格檔，而引擎沒有刪除 capability 的操作。刪除的理由是：這四份規定的介面與實作已經不存在；仍然成立的保證由 store-port-contract（契約）與 store-port-examples（三個範例）承接。

刪除前先搜尋其他正式規格、scripts、crates 與 apps 裡提到這四個名稱的地方。目前已知兩處：示範用 server 兩個測試檔開頭的註解（`postgres_store.rs`、`serverfs_store.rs`），改成指向 store-port-examples。

**host-runtime 的 Purpose**：delta 不更新既有 capability 的 Purpose，所以直接改正式規格檔的那一段（只改這一段），把「以及 TeamStore 的 unit of work 與事件提交歸屬」換成「以及引擎命令經儲存接口執行、一個動詞收成一次提交的歸屬」。

**其餘用 delta**：host-runtime、server-read-api、server-identity、change-diff-scope，內容見各自的 delta。

### D5: 流程設定的專案說明

用流程設定的動詞改 `speclink/config.yaml`，不手改檔案。

context 改三段，其餘原樣保留：

- 第一段的最後一句改成：「同時支援本機專案（`speclink/` ＋ Git、可完全離線）與遠端專案（規格由一台 Server 持有：內嵌了引擎的宿主，或官方的示範用 server）兩條路徑。」
- 技術棧那一段的「store driver 為 rusqlite／postgres／檔案系統」改成「儲存接口的三個範例用 rusqlite／postgres／檔案系統」。
- Cargo workspace 那一段裡 `speclink-store` 與三個 driver 的說明改成：「`speclink-store-port`（儲存契約：列出、讀取、提交三個操作，合格測試與記憶體版參考實作）＋ `speclink-store-port-{sqlite,fs,postgres}`（三個範例，也是示範用 server 的儲存）」；`speclink-host` 的說明改成「引擎唯一的應用層邊界與宿主套件：execution context、命令經儲存接口執行、掛鉤、HTTP 入口、匯入匯出」；`speclink-server` 的說明加上「示範用」。

`rules.design` 第一條的「TeamStore contract 與 driver 歸 store crates」改成「儲存契約與範例歸 store crates」，其餘文字不變。

寫回時的注意事項：`workflow-config show --json` 取得的 context 少結尾的換行，原樣寫回會讓 YAML 的區塊標記改變、整段被重寫。寫回前補上結尾的換行，先用 `--dry-run` 看差異只有上面列的幾行，再真的寫入。`rules design` 是整個區段取代，要把全部規則一起送回。

### D6: CI、文件連結與註解

- `.github/workflows/ci.yml` 的 PostgreSQL job：移除執行 `speclink-store-postgres` 測試的那一行。
- docs/product-status.md 與 docs/product-status.zh-TW.md：四列的證據連結改指到存在的檔案——三個儲存的列各指向對應範例的整合測試檔；「Command Runtime、Host 與 Protocol」那一列原本指向 bridge_dual_path.rs 的連結，改指到保有「兩條路徑結果一致」測試的檔案。連結的文字、列的其他欄位、日期都不動，兩個語言版本改同樣的地方（文件的對照守門要求兩版的列與日期一致）。
- 註解與套件說明：引擎 `lifecycle/tasks.rs` 的一句、協定 `query.rs` 的一句、宿主套件與示範用 server 的 Cargo.toml 的 description、示範用 server 的 Dockerfile 的一句註解、宿主套件 `lib.rs` 與 `change_diff.rs` 的註解。改成不提 TeamStore。
- 完成的判斷：在 crates、apps、packages、scripts、deploy、.github 底下搜尋 `TeamStore`、`speclink_store::`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres`，沒有命中。允許留下的只有：更新日誌與發版說明的歷史內容、Speclink Desktop 測試裡當作詞彙表內容的字串。docs、README 與 LANGUAGE.md 不在這次搜尋的範圍（留給最後一個變更）。

## Implementation Contract

**行為**

- 對外的行為沒有改變。CLI、示範用 server、Speclink Desktop、Node SDK 的全部既有測試不改斷言就通過。
- `speclink_host::projection::content_digest` 仍然存在，輸出相同。
- 從宿主套件引用的 `ProjectId` 與 `RepoId` 仍然存在，建構方式與方法相同。

**介面**

- 宿主套件 `scope`：專案與儲存庫 ↔ 範圍識別字的一對函式（規格 host-runtime「專案與儲存庫對映成範圍識別字」）。
- 宿主套件 `transfer`：公開第一版的版本常數；「文件種類與鍵的對應」的一對函式維持公開（規格 host-runtime「文件種類與鍵的對應」）。

**失敗的樣子**

- 沒有新的失敗情況。範圍識別字形式不合的錯誤文字維持原樣（示範用 server 的測試有斷言）。

**驗收**

- `cargo build --workspace` 成功；`cargo metadata` 的 workspace 成員沒有四個舊 crate。
- `cargo test -p speclink-host`、`cargo test -p speclink-server`、`cargo test -p speclink-remote`、`cargo test -p speclink-desktop`、`cargo test -p speclink-cli` 全部通過。
- D6 的搜尋沒有命中（允許清單以外）。
- D2 的 36 個測試去向表完整。
- `speclink workflow-config show` 顯示新的 context 與規則；`speclink validate retire-teamstore-and-drivers` 通過。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過（含文件的連結檢查與中英對照）。

**範圍邊界**

- 範圍內：D1 到 D6。
- 範圍外：docs 與 README 的文字、LANGUAGE.md、手冊、三個範例與儲存契約的程式。

## Risks / Trade-offs

- [刪測試時丟掉仍然存在的行為] → D2 要求 36 個測試逐一列出去向；「兩條路徑結果一致」在刪除前先確認新的位置有同樣的斷言。
- [回歸對照] → 引擎的 golden 與 CLI 的整合測試不改；CLI 的 `remote_read_path` 測試從宿主套件引用內容摘要的路徑不變。驗收條件要求 CLI 的測試照常通過。
- [跨平台] → 這個變更沒有新的檔案或路徑處理。刪除 crate 之後在三個平台的建置由 CI 確認。
- [流程設定的 context 被整段重寫] → D5 規定補結尾換行並先 `--dry-run`。
- [文件的守門在刪除檔案之後變紅] → D6 在同一個變更修連結；最後一個任務跑全部的 scripts 測試。守門紅的時候 CI 會跳過其他四組測試，所以這一步不能漏。
- [直接刪除規格資料夾繞過了 delta] → 這是刻意的（D4）。理由寫在提案與這份設計；仍然成立的兩個規則已經用 delta 在 host-runtime 另立需求。
- [本機的 CLI 產物被 workspace 測試換成舊的] → 跑完 `cargo test -p speclink-desktop` 之後先 `cargo build -p speclink-cli` 再執行 speclink 的動詞。

## Migration Plan

沒有資料要遷移。回退方式是還原這個變更的 commit。

合併之後，開發者本機的 `target` 資料夾裡會留著四個舊 crate 的編譯產物；它們不影響建置。

## Open Questions

（無）
