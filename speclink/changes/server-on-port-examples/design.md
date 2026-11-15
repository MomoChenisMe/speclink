## Context

示範用 server（`crates/host/speclink-server`）今天這樣用儲存：

- 啟動時依設定檔的 `store.driver` 建立一個 TeamStore（SQLite、檔案系統、PostgreSQL 或記憶體）。
- 把它包進 TeamStore 的轉接，交給宿主套件當儲存接口。專案路由全部經宿主套件。
- 另外直接持有 TeamStore，給六處程式用：事件串流（`api/events.rs` 的 `EventHub` 讀待送佇列、已讀位置並做保留）、備份／檢查／還原（`admin/backup.rs`）、管理頁的系統狀態與遷移（`admin.rs`）、`/readyz`（`app.rs`）、初次設定頁（`identity/setup.rs`）。

帳號、登入工作階段、存取金鑰、成員、專案與儲存庫的登記、稽核紀錄都在另一個 identity 儲存裡，和 TeamStore 無關，這個變更不碰。

變更 store-port-examples 提供三個範例。每個範例除了三個函式，還有四個方法：讀某個序號之後的事件、回報下限、清掉舊事件、檢查可用性。範例開啟時只認自己的標記；遇到別的標記會拒絕，並回報讀到的標記。舊 driver 的標記是 `speclink-team-store`（SQLite 與 PostgreSQL）與 `speclink-team-store-fs`（檔案系統）。

宿主套件已經有「範圍的匯出」與「範圍的匯入」（`speclink_host::transfer`），以及把第一版匯入包的文件種類換成鍵的函式。

server 的測試有 44 個檔直接用舊的記憶體儲存塞資料或讀待送佇列；`speclink-remote` 有 3 個測試檔、Speclink Desktop 有 3 個測試檔會自己建一個 server 的狀態。

## Goals / Non-Goals

**Goals:**

- 示範用 server 的正式程式與測試都不再用到 `speclink-store` 與三個舊 driver。
- 用戶端（CLI、Speclink Desktop）看到的行為不變，除了變更清單多了最後修改時間。
- 從舊版升級的人得到明確的拒絕與三個步驟，舊資料不被改動。
- 舊版做的備份檔在新版能檢查、能還原。

**Non-Goals:**

- 不刪除舊的 store crates、TeamStore 的轉接與 `speclink-host` 裡用它們的測試。
- 不自動轉換舊資料。
- 不改 identity 儲存。
- 不支援多個 server 實例。
- 不動使用者文件。

範圍內：`speclink-server`（正式程式與全部測試）、apps/server-web 的系統頁與初次設定頁、`speclink-host` 裡範圍識別的函式搬家、`speclink-remote` 與 Speclink Desktop 建立 server 狀態的測試。範圍外：其餘全部。

## Decisions

### D1: 示範用 server 自己的儲存介面

新檔 `src/storage.rs` 定義 server 需要的那一組能力（`ServerStore`）：儲存接口的三個函式，加上讀事件、回報下限、清事件、檢查可用性，與一個回報種類名稱（`sqlite`、`serverfs`、`postgres`、`memory`）的方法。

- 三個範例各寫一段把這些方法轉給範例同名方法的實作，沒有邏輯。
- 記憶體版：包住儲存契約 crate 的記憶體版參考實作，事件用它既有的「讀回事件」取得並以位置當序號，下限由包裝自己記在記憶體裡。記憶體版只給測試的設定用。
- `build_store` 回傳一個共用的 `ServerStore`。`AppState` 只持有這一個物件：宿主套件拿它當儲存接口，`EventHub`、管理頁、`/readyz`、初次設定頁拿它的其他方法。`SharedStore` 與 `SharedPort` 兩個別名合成一個。
- 設定檔的解析（`config.rs`）不變。

這個介面定義在 server 裡，不放進儲存契約或範例：只有示範用 server 需要把三個範例當成同一種東西。

實作的順序分兩段，讓每一段結束時全部測試都通過：

1. 先定義 `ServerStore`，並寫一個**過渡用的實作**包住今天的 TeamStore 與它的轉接（事件的三個方法轉給待送佇列的讀取、已讀位置與推進）。`AppState`、`EventHub`、管理頁、`/readyz`、初次設定頁、備份與還原都改成只經 `ServerStore`。這一段不換儲存，行為不變。
2. 最後把 `build_store` 換成三個範例與記憶體版，刪掉過渡用的實作與舊的相依。

過渡用的實作只活在這個變更的中間幾個任務裡，變更完成時不存在。

範圍的識別：把「專案與儲存庫」換成儲存接口的範圍識別字、以及反向的兩個函式，今天寫在 TeamStore 的轉接檔裡。把它們搬到 `speclink-host` 的新檔 `src/scope.rs`，轉接改呼叫新位置；server 改從新位置引用。專案與儲存庫的型別（`ProjectId`、`RepoId`）這個變更仍由 `speclink-store` 定義，`speclink-host` 加上轉出，server 改從 `speclink-host` 引用；型別本身在下一個變更搬家。

### D2: 啟動時遇到舊配置的資料

`build_store` 把範例的開啟失敗分成三種結果：

| 範例回報 | server 的處置 |
| --- | --- |
| 讀到的標記是 `speclink-team-store` 或 `speclink-team-store-fs` | 舊版的資料。印出下面的訊息 |
| 其他不認得的標記、沒有標記、較新的配置版本 | 印出位置、讀到的標記與配置版本；不印三個步驟 |
| 資料夾使用中、連不上、其他失敗 | 沿用今天的啟動失敗訊息 |

三種都以非零的 exit code 結束，發生在綁定連接埠之前。舊標記的兩個字串是 server 這一側的常數，不在範例裡。

舊版資料的訊息（英文，和 server 既有的訊息一致；`<…>` 換成實際的值）：

```
error: the store at <location> was created by speclink-server 0.12 or earlier.
This version uses a different layout and did not change your data.

To move the data to this version:
  1. With the old version, run:
       speclink-server backup --config <config> --output <file>
  2. Point the config at an empty store location and an empty identity database.
  3. With this version, run:
       speclink-server restore --config <config> --input <file>
```

所有會開啟儲存的進入點共用 `build_store`，所以 `backup` 子指令在舊資料上得到同一段訊息。`restore` 的目標若是舊資料，同樣被拒絕（它不是空的目標）。`verify-backup` 不開啟儲存，不受影響。

### D3: 事件串流改讀事件記錄

`EventHub` 的三個呼叫各換成一個：

| 今天 | 之後 |
| --- | --- |
| 讀已讀位置 | 回報下限 |
| 讀某個位置之後的待送事件 | 讀某個序號之後的事件 |
| 推進已讀位置（保留最近 N 筆） | 清掉某個序號以前的事件 |

讀事件有筆數上限，所以補送與泵送都改成分批讀到沒有為止，一批的大小用既有的緩衝設定值。事件的分類照舊交給宿主套件的分類函式（輸入是事件名稱與內容）。SSE 的事件識別仍然是序號。

`EventHub` 以儲存接口的範圍識別字當鍵，不再用 TeamStore 的範圍型別。提交後掛鉤通知 `EventHub` 的方式不變。

行為不變：正式規格 server-event-stream 的條款與測試（續傳不漏不重、保留與 reset、慢消費者）原樣適用，測試的斷言不改。

### D4: 備份、檢查與還原

**備份**（格式版本 2）

- 每個登記的範圍呼叫宿主套件的範圍匯出，得到以鍵標示文件的包，原樣放進備份檔。
- manifest 的 store manifest 欄位換成一個字串欄位：儲存的種類。其他欄位、成員的命名、摘要鏈都不變。
- 管理頁的範圍匯出下載用同一個函式，下載的內容是同一種包。

**檢查**

- 接受格式版本 1 與 2，其他拒絕。兩種版本共用 manifest 與摘要鏈的檢查；包的結構檢查依版本各自解析。

**還原**

- 第 2 版的包：直接交給宿主套件的範圍匯入。
- 第 1 版的包：server 留一組只用來讀舊備份的型別（專案、儲存庫、文件種類與內容），不相依 `speclink-store`；每份文件經宿主套件既有的「文件種類換成鍵」換成鍵，組成第 2 版的包再匯入。
- 順序：檢查備份檔 → 把全部範圍的包準備好（第 1 版在這一步轉換；換不成鍵就停，什麼都還沒寫）→ 確認目標是空的 → 放 identity 快照 → 逐範圍匯入 → 還原後的驗證。
- 逐範圍匯入不是跨範圍的單一交易，和今天相同：中途失敗時報告已完成與失敗的範圍，以非零的 exit code 結束，並明示目標不可投產。每個範圍自己是一次提交，不會只寫一半。
- 還原後的驗證：對每個範圍重新匯出，比對鍵的集合、每份文件的內容摘要與文件數。
- 匯入會帶一筆「範圍已匯入」的事件，所以還原後每個範圍的事件記錄從序號 1 開始。還原前連著的用戶端重連時，手上的事件識別比 server 的新。處置見 D5。

為了測試「舊版的備份能還原」，要有一份真的由舊程式產生的備份檔。在改 `admin/backup.rs` 之前，先用目前的程式對一個塞好資料的記憶體儲存做一份備份，存成測試用的固定檔（一個專案、兩個儲存庫；文件涵蓋九種：進行中的變更與它的產出、metadata、evidence、正式規格、討論、已封存的變更、流程設定、詞彙表、看板順序；identity 只有測試用的假帳號）。

### D5: 用戶端的事件識別比 server 新

還原（或任何讓事件記錄重新開始的情況）之後，重連的用戶端帶來的 Last-Event-ID 可能大於 server 目前最大的序號。今天的程式會把它當成「沒有要補的」，用戶端之後收到的事件序號比它手上的小。

`EventHub` 在訂閱時加一個判斷：Last-Event-ID 大於目前最大的序號時，和落在已清除的範圍同樣處理——送 reset 訊號並從目前最大的序號之後開始推送。對應規格 server-event-stream「保留政策與 reset 訊號」新增的條款與情境。

### D6: 管理頁、健康檢查與初次設定頁

- 系統狀態：儲存的種類來自 D1 的名稱；可用與否來自可用性檢查；每個範圍「保留中的事件筆數」用下限之後分批讀事件來數（保留設定預設 1024 筆，數量有界）。回應的欄位名稱沿用 `storeDriver`、`storeHealthy`、`storeHealthError`、`outboxBacklogs`；移除 `storeContractVersion`、`storeLevel`、`storeCapabilities`、`migrateAvailable`。
- 遷移：移除 `POST /admin/data/migrate` 的路由與處理函式。稽核的動作種類 `store-migrated` 不再產生；稽核頁的動作篩選與標籤留著，既有的紀錄才看得懂。
- `/readyz` 與總覽頁：改用可用性檢查。
- 初次設定頁的 store 物件：`driver`、`healthy`、`healthError`、`identitySchemaVersion`；移除 `contractVersion`、`level`、`capabilities`。

### D7: 管理網頁

- `SystemPage`：移除危險區的卡、確認對話框與遷移的狀態；儲存狀態的卡只列種類、是否可用與各範圍的事件筆數。沿用既有的 `SettingsCard` 與列的元件，不新增元件。
- `SetupPage`：儲存狀態的區塊移除契約版本、等級與能力。
- `client.ts`：`StoreStatus` 與系統頁的型別移除對應欄位，移除 `adminMigrate`。
- `messages.ts`：移除 `system.migrate*` 五個鍵與「危險區」的標題鍵（沒有其他使用處時）；`system.description` 兩個語言的文字拿掉遷移；`audit.action.store-migrated` 留著。
- 測試：`system.test.tsx`、`setup.test.tsx`、`wording.test.tsx` 與測試用的假資料跟著改。

### D8: 測試的搬移

原則：server 的測試不再出現舊儲存層的任何型別。

1. 先在測試的共用模組加三類輔助函式，簽章不帶舊型別：塞一份文件（範圍、鍵、內容）、讀一份文件、讀某個範圍的事件。第一步讓它們的內部仍然用舊的記憶體儲存，並把 44 個測試檔逐一改成只經這些函式——這一步不改任何行為，全部測試維持通過。
2. 正式程式改接（D1 到 D6）時，把輔助函式的內部換成新的記憶體儲存。測試檔本身不用再動。
3. 測試用的「競爭」替身（今天是一個完整的 TeamStore 實作）改成包住 `ServerStore`、在指定的時機插入另一方的提交。
4. 三個 driver 的端到端測試（SQLite 在多個測試裡、檔案系統的 `serverfs_store`、PostgreSQL 的 `postgres_store`）改成在三個範例上執行同樣的流程。
5. `speclink-remote` 與 Speclink Desktop 自己建 server 狀態的測試，改用新的建構方式。

第 1 步完成的判斷：在 server 的測試資料夾搜尋 `speclink_store`，只剩共用模組一處。第 2 步完成後一處都沒有。

### D9: 相依與 CI

- `speclink-server` 的 Cargo.toml：加上三個範例與 `speclink-store-port`；移除 `speclink-store`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres`（含測試用的相依）。`postgres` 這個測試用的相依視新的測試需要決定去留。
- CI 的 PostgreSQL job 不用改：舊 driver 的 crate 還在，它自己的測試照跑到下一個變更刪除為止；server 在 PostgreSQL 上的端到端測試維持同一個測試目標名稱（`postgres_store`），job 裡的指令不變。
- Dockerfile、compose 檔、npm 啟動器、開發腳本與它們的測試：設定的鍵與值不變，不用改。用既有的煙霧測試確認。

## Implementation Contract

**行為**

- 以既有的設定檔在空的位置啟動：和今天相同。用戶端的全部路由行為不變；`GET` 變更清單的每一項多了 `modifiedAt`。
- 以既有的設定檔在舊版的資料上啟動：非零的 exit code、D2 的訊息、資料不變、沒有綁定連接埠。
- 事件串流：server-event-stream 既有的全部情境照舊成立；另外 Last-Event-ID 大於最大序號時收到 reset（這個變更新增的情境）。
- `backup` 產生格式版本 2 的備份；`verify-backup` 與 `restore` 接受 1 與 2。
- 管理頁的系統狀態與初次設定頁的回應如 D6；`POST /admin/data/migrate` 回 404。

**介面**

- `ServerStore`：D1 列的方法。只在 `speclink-server` 內部使用。
- 備份檔 manifest：`backupFormatVersion` 是 2；原本的 store manifest 欄位換成儲存種類的字串欄位。欄位的實際名稱沿用檔案既有的命名風格，並在實作時寫進這份 design 的「稽核結論」。
- browser API 的欄位：D6。
- 啟動失敗的訊息：D2 的文字。測試比對其中三行指令與 `0.12 or earlier`。

**失敗的樣子**

- 儲存暫時無法使用：路由回既有的 unavailable（503），`/readyz` 回非 2xx，管理頁顯示儲存不可用但其餘功能照常。
- 還原時第 1 版的文件換不成鍵：非零的 exit code，指出範圍與文件，目標沒有被寫入。
- 還原在逐範圍匯入的中途失敗：非零的 exit code，報告哪些範圍完成、哪個失敗，明示目標不可投產。

**驗收**

- `cargo test -p speclink-server` 全部通過；server-event-stream、reference-server、server-backup、server-admin、server-setup 既有情境的測試不改斷言（除了這個變更修改的條款）。
- `cargo tree -p speclink-server -e normal,dev -i speclink-store-sqlite`、對 `speclink-store-fs` 與 `speclink-store-postgres` 的同一個指令，都回報找不到套件。在 crates/host/speclink-server 底下搜尋 `speclink_store::` 與 `TeamStore`，`src` 與 `tests` 都沒有命中（註解也清掉）。
- `cargo test -p speclink-remote`、`cargo test -p speclink-desktop`、`cargo test -p speclink-host` 通過。
- `npm test -w apps/server-web` 通過。
- 用固定檔測試：0.12 的備份檔在新版檢查通過、還原後的驗證全綠。
- 以真的 PostgreSQL 執行 `cargo test -p speclink-server --test postgres_store`，輸出不含 `skipped`。

**範圍邊界**

- 範圍內：D1 到 D9 列的檔案。
- 範圍外：刪除舊 crate、`speclink-host` 的轉接與它的測試、docs、README。

## Risks / Trade-offs

- [回歸對照：線上回應的位元級輸出] → 宿主套件的路由不變，只換底下的儲存。CLI 的 `remote_verb_parity` 測試與 server 的 API 測試不改斷言。會變的只有：變更清單多 `modifiedAt`、範圍版本號的數值（每個範圍各自從 1 開始）、ETag 的值跟著版本號變。測試裡寫死版本號數值的地方要逐一確認是斷言「有變」還是斷言「等於某個數」，後者依新的計數修正並在任務的完成說明列出。
- [44 個測試檔的機械式改寫出錯] → D8 分兩步：先在舊儲存上換成輔助函式（行為不變、全部通過），再換輔助函式的內部。兩步各自有可驗證的終點。
- [舊備份檔的固定檔是二進位檔] → 它由舊程式產生一次後進版控。檔案只含測試用的假資料；在任務裡寫明產生的方式，之後不需要重新產生。
- [跨平台] → 檔案系統範例的鎖與改名已在它自己的變更處理。server 這一側新增的只有啟動訊息裡的路徑顯示：用系統原生的路徑字串，不自行轉換分隔符號。測試比對訊息時不比對路徑的分隔符號。
- [PostgreSQL 的端到端測試本機跑不了] → 和今天相同：沒有資料庫時回報略過，CI 的 job 把略過當失敗。任務要求以真的資料庫驗證一次。
- [升級的人沒有先備份就換了版本] → server 拒絕啟動且不改動資料；訊息第一步就是「用舊版」做備份。回退只要換回舊版。
- [還原後事件序號重新開始] → D5 的 reset。
- [管理頁的 API 移除欄位] → 管理網頁和 server 同一個建置出貨，沒有獨立的使用者。

## Migration Plan

給既有的部署：

1. 停止寫入，以舊版執行 `speclink-server backup`。
2. 換成新版；把設定檔的儲存位置與 identity 資料庫指向空的位置（或把舊的檔案移走保存）。
3. 以新版執行 `speclink-server restore`，確認驗證報告全綠，再啟動。

回退：換回舊版並把設定指回原本的位置。舊的資料沒有被新版動過。

還原之後，每個範圍的版本號與事件序號重新開始；文件內容不變。用戶端重連時會收到一次 reset 並重新讀取。

## Open Questions

（無）
