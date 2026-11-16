## Why

示範用 server 的資料今天存在舊的儲存層（TeamStore）的三個 driver 裡。專案路由已經全部改經宿主套件與儲存接口，但 server 是靠一層轉接把 TeamStore 包成三個函式。這層轉接有三個缺點：回報的最後修改時間永遠是 0（所以連到示範用 server 時變更清單沒有最後修改時間）、整個專案共用一個版本計數器、過不了儲存契約的合格測試。

另外有六處程式還直接呼叫 TeamStore，不經儲存接口：

- 推送事件的串流：讀 TeamStore 的待送佇列與已讀位置。
- 備份、檢查備份、還原：用 TeamStore 的匯出與匯入。
- 管理頁的系統狀態：TeamStore 的 manifest、健康檢查與待送佇列數量。
- 管理頁的「資料結構遷移」。
- `/readyz` 的健康檢查。
- 初次設定頁的儲存狀態。

變更 store-port-examples 已經寫好三個直接實作儲存接口的範例（SQLite、PostgreSQL、檔案系統）。這個變更讓示範用 server 改用它們，並把上面六處改接。做完之後，示範用 server 不再用到舊的儲存層，下一個變更就能把舊的儲存層刪掉。

- **目標使用者**：試用或自己架設示範用 server 的人；它的管理者。
- **使用情境**：啟動 server、用 CLI 與 Speclink Desktop 連上它、在管理頁看系統狀態、做備份與還原、從舊版升級。
- **既有規格的對照**：reference-server 規定寫入經 Host 提交到 TeamStore、啟動時的設定檢查。server-backup 規定備份檔的內容與還原的步驟，文字綁在 TeamStore 的匯出契約與 store manifest 上。server-admin 規定系統狀態要聚合 store manifest（driver、契約版本、能力、等級）與遷移可用性。server-setup 規定初次設定頁顯示 manifest 的 driver 與能力。server-web-console 有四條需求提到系統頁的「資料結構遷移」。server-event-stream 的條款（事件序號、續傳、保留與 reset）不綁特定的儲存，行為維持；只補上一種新情況：還原之後用戶端手上的事件識別比 server 的新。store-port-examples 規定範例的事件記錄與「只認自己的配置」，是這個變更的地基。

Source discussion: embedded-host-needs（cut 19）。

## What Changes

**示範用 server 的儲存改用三個範例**

- 設定檔不變：`store.driver` 仍然是 `sqlite`、`serverfs`、`postgres`、`memory` 四個值，路徑與網址的欄位不變。四個值分別接到 SQLite 範例、檔案系統範例、PostgreSQL 範例與記憶體版的參考實作。
- 宿主套件直接拿到儲存接口，不再經 TeamStore 的轉接。
- 連到示範用 server 時，變更清單開始有真實的最後修改時間（`modifiedAt`）；每個範圍有自己的版本號。

**舊資料：拒絕啟動並說明怎麼搬**（**BREAKING**）

- 新版用新的資料表與檔案配置，讀不到 0.12.0 以前建立的資料。
- `speclink-server` 啟動時（以及 `backup` 子指令開啟儲存時）遇到舊配置的資料，以非零的 exit code 結束，不綁定連接埠、不改動資料，並在 stderr 印出三個步驟：在舊版執行 `speclink-server backup --config <file> --output <file>`；安裝新版並把設定指向空的儲存位置與空的 identity 資料庫；執行 `speclink-server restore --config <file> --input <file>`。
- 新版的 `verify-backup` 與 `restore` 讀得懂舊版做的備份檔（備份格式第 1 版）。新版做的備份檔是第 2 版。

**推送事件的串流**

- 改讀範例的事件記錄。事件的序號、斷線後續傳、保留筆數與 reset 訊號的行為不變。
- 新增一種 reset：重連的用戶端帶來的事件識別大於 server 目前最大的序號時（還原之後事件序號重新開始），server 送出 reset 訊號，用戶端重新讀取。

**備份、檢查、還原、匯出下載**

- 備份：每個範圍的內容改由「範圍的匯出」（宿主套件既有的函式）產生，是以鍵標示文件的包。備份檔的 manifest 不再帶 store manifest，改記儲存的種類。備份格式版本升到 2。
- 還原：第 2 版的包直接匯入；第 1 版的包先把文件種類換成鍵再匯入。兩種都是每個範圍一次提交。還原後的驗證不變：逐範圍重讀並比對文件數與內容摘要。
- 管理頁的範圍匯出下載：內容改成以鍵標示文件的包（和線上的 `GET /export` 相同）。

**管理頁、健康檢查、初次設定頁**

- 系統狀態：留下儲存的種類、是否可用、每個範圍保留中的事件筆數。移除契約版本、等級與能力清單（它們是 TeamStore 的概念）。
- 「資料結構遷移」整個移除：系統頁的危險區、確認對話框與 `POST /admin/data/migrate`。範例在開啟時自己建立資料表，沒有跨版本的遷移。
- `/readyz` 與總覽頁的儲存健康狀態：改用範例的可用性檢查。行為不變。
- 初次設定頁的儲存狀態：留下儲存的種類、是否可用與 identity 的資料結構版本。

**相容性影響**

- 設定檔、Docker 映像的預設設定、compose 檔、npm 啟動器、開發用的環境變數：不變。
- 線上協定（CLI 與 Speclink Desktop 用的路由）：形狀不變。變更清單多了 `modifiedAt`（選填欄位，舊版用戶端忽略它）。範圍的版本號從「整個專案一個」變成「每個範圍一個」；用戶端只比較是否相同，不受影響。
- 管理頁的 browser API：`system` 回應移除 `storeContractVersion`、`storeLevel`、`storeCapabilities`、`migrateAvailable` 四個欄位；`setup` 的 store 物件移除 `contractVersion`、`level`、`capabilities`。管理網頁和 server 一起出貨，同步修改。`POST /admin/data/migrate` 移除（回 404）。
- 稽核紀錄：不再產生 `store-migrated`；既有的紀錄照常顯示。
- 備份檔：新版做的備份舊版讀不了（格式版本 2）。舊版做的備份新版能檢查與還原。
- 範圍匯出下載的檔案內容改變（改用鍵）。
- 既有部署的升級是手動的三個步驟，不能直接換版本啟動。發版說明要把它列為破壞性變更。
- **CLI（`speclink`）的指令、`speclink/config.yaml` 與 `.speclink.yaml` 的欄位、技能、Node SDK**：都沒有變更。`speclink-server` 的子指令與旗標不變。

## Non-Goals

- **自動轉換舊配置的資料**：討論裁定用備份與還原搬。
- **刪除舊的儲存層與三個舊 driver**：變更 retire-teamstore-and-drivers。這個變更之後 `speclink-server` 不再相依它們，但 crate 還在，`speclink-host` 的轉接與測試還在用。
- **多個 server 實例共用一份儲存**：仍然只支援單一實例。事件串流靠程序內的通知。
- **備份在有寫入時的一致性**：維持「在沒有寫入的條件下執行」。
- **管理頁新增功能**：只移除與改接。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `reference-server`: 修改「寫入原子提交且 CAS 衝突可辨」（提交到儲存接口，事件進事件記錄）與「啟動組態 fail closed」（遇到舊配置的資料拒絕啟動並說明步驟）。
- `server-backup`: 修改四條——備份檔的內容（以鍵標示文件的包、格式版本 2）、檢查接受兩個格式版本、還原接受兩個格式版本、管理面的資料操作（匯出改用鍵、移除遷移）。
- `server-admin`: 修改「系統資訊唯讀聚合」與「管理 browser API 提供最小且完整的頁面 view model」（移除契約版本、等級、能力與遷移可用性）。
- `server-setup`: 修改「setup 流程完成開箱四要素」（儲存狀態的內容）。
- `server-web-console`: 修改四條提到資料結構遷移或危險區的需求。
- `server-event-stream`: 修改「保留政策與 reset 訊號」（事件識別比 server 新時送 reset）。

## Impact

- Affected specs: reference-server（修改兩條）、server-backup（修改四條）、server-admin（修改兩條）、server-setup（修改一條）、server-web-console（修改四條）、server-event-stream（修改一條）
- Affected crates 與 app: `speclink-server`、`speclink-host`（只搬一個函式並加上轉出）、apps/server-web。`speclink-remote` 與 apps/desktop 只改測試。`speclink-core`、`speclink-cli`、`speclink-node`、三個範例的 crate 與舊的 store crates 不改。
- Affected code:
  - New:
    - crates/host/speclink-server/src/storage.rs
    - crates/host/speclink-server/tests/it/legacy_layout.rs
    - crates/host/speclink-server/tests/it/admin/legacy_backup.rs
    - crates/host/speclink-server/tests/fixtures/backup-format-1.tar
    - crates/host/speclink-server/tests/fixtures/backup-format-1.md
    - crates/host/speclink-host/src/scope.rs
    - crates/protocol/speclink-remote/tests/it/seed.rs
    - apps/server-web/src/components/StoreStatus.tsx
  - Modified:
    - crates/host/speclink-server/Cargo.toml
    - Cargo.lock
    - crates/host/speclink-server/src/lib.rs
    - crates/host/speclink-server/src/main.rs
    - crates/host/speclink-server/src/config.rs
    - crates/host/speclink-server/src/state.rs
    - crates/host/speclink-server/src/app.rs
    - crates/host/speclink-server/src/api/events.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/src/api/error.rs
    - crates/host/speclink-server/src/api/forward.rs
    - crates/host/speclink-server/src/admin.rs
    - crates/host/speclink-server/src/admin/backup.rs
    - crates/host/speclink-server/src/admin/audit.rs
    - crates/host/speclink-server/src/identity/sqlite.rs
    - crates/host/speclink-server/src/identity/setup.rs
    - crates/host/speclink-server/tests/it/common/mod.rs
    - crates/host/speclink-server/tests/it/main.rs
    - crates/host/speclink-server/tests/it/serverfs_store.rs
    - crates/host/speclink-server/tests/postgres_store.rs
    - crates/host/speclink-server/tests/pg/mod.rs
    - crates/host/speclink-server/tests/it/e2e_cli.rs
    - crates/host/speclink-server/tests/it/phase2_chain.rs
    - crates/host/speclink-server/tests/it/startup.rs
    - crates/host/speclink-server/tests/it/admin/backup_restore.rs
    - crates/host/speclink-server/tests/it/api/sse_events.rs
    - crates/host/speclink-server/tests/it/api/import_api.rs
    - crates/host/speclink-server/tests/it/api（其餘直接用舊記憶體儲存塞資料的測試檔，逐檔改用共用的輔助函式；清單在任務裡盤點）
    - crates/host/speclink-server/tests/it/admin（同上）
    - crates/host/speclink-server/tests/it/identity（同上）
    - crates/host/speclink-server/tests/it/web（同上）
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/protocol/speclink-remote/Cargo.toml
    - crates/protocol/speclink-remote/tests/it/read_api.rs
    - crates/protocol/speclink-remote/tests/it/main.rs
    - crates/protocol/speclink-remote/tests/it/events_sse.rs
    - crates/protocol/speclink-remote/tests/it/device_flow.rs
    - apps/desktop/src-tauri/Cargo.toml
    - apps/desktop/src-tauri/tests/it/common/mod.rs
    - apps/desktop/src-tauri/tests/it/login_orchestration.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - apps/server-web/src/api/client.ts
    - apps/server-web/src/pages/admin/SystemPage.tsx
    - apps/server-web/src/pages/SetupPage.tsx
    - apps/server-web/src/i18n/messages.ts
    - apps/server-web/src/__tests__/system.test.tsx
    - apps/server-web/src/__tests__/setup.test.tsx
    - apps/server-web/src/__tests__/wording.test.tsx
    - apps/server-web/src/__tests__/helpers/adminHarness.tsx
    - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - Removed: (none)
- 相依：store-port-examples（要先有三個範例）。
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案不經 server。
  - 連到遠端的 checkout：可用。連到示範用 server 的 CLI 與 Speclink Desktop 行為不變，變更清單多了最後修改時間；apply 技能「閒置太久」的判斷因此在示範用 server 上也能用最後修改時間。從舊版升級的 server 要先走備份與還原。
  - 沒有本機規格檔的內嵌宿主：不適用。這個變更只改示範用 server；內嵌的宿主用自己的儲存。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - docs/server-deployment.zh-TW.md 與 docs/server-deployment.md：升級一節要寫從 0.12.0 以前升級的三個步驟，以及回退的方式（舊版的資料沒有被動過）。
  - docs/server-backup.zh-TW.md 與 docs/server-backup.md：備份格式第 2 版、新版讀得懂第 1 版、範圍匯出下載的內容。
  - docs/server-store-drivers.zh-TW.md 與 docs/server-store-drivers.md：三種儲存現在是儲存接口的範例；移除契約版本、等級、能力與遷移的說明。
  - docs/product-status.zh-TW.md 與 docs/product-status.md：移除「連到示範用 server 時沒有最後修改時間」這個缺口；三列儲存的說明。
  - Speclink 後台的截圖：系統頁少了危險區，`server-overview.png` 以外若有拍到系統頁的要重拍。
  - 發版說明：列出升級步驟這一項破壞性變更。
