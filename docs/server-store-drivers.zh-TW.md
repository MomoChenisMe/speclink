# Server Store Driver 選型

官方 `speclink-server` 的持久層由組態檔的 `store` 段決定，經單一建構點接線。所有 driver 都通過同一套 TeamStore conformance suite（能力檢查、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 四故障點、tenant scope），因此**動詞行為與 driver 無關**——換 driver 不會改變 API 的可觀察結果。

| driver | 承載 | 適用 |
|---|---|---|
| `sqlite` | 單一資料庫檔 | **預設**。無特殊前提，任何檔案系統可用 |
| `serverfs` | 單一資料目錄 | 偏好純檔案持久層（備份工具直接可見目錄、無資料庫維運）；**前提見下** |
| `postgres` | PostgreSQL 資料庫 | 已有 PostgreSQL 維運基礎（統一資料庫棧、既有備份與監控直接套用）；**前提見下** |
| `memory` | 記憶體 | **僅供測試組態**，行程結束即消失 |

未知的 driver 名稱使啟動失敗並列出支援清單（fail closed）——拼錯 `serverfs` 不會靜默退回別的持久層。

## sqlite（預設）

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

## serverfs

```yaml
store:
  driver: serverfs
  path: /var/lib/speclink/store     # 資料目錄，非檔案
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

`path` 指向**資料目錄**。目錄不存在會被建立；空目錄會初始化為現行 schema version。

### 前提：檔案系統須支援 flock 語意

serverfs 以資料目錄內鎖檔的 **OS advisory lock（flock 語意）**取得單寫者排他。鎖由 kernel 持有，因此持有者程序死亡（被 kill、panic、斷電）時鎖自動釋放，新 server 可直接接管——不需要偵測、接管或破除殘留鎖。

**部署前請確認資料目錄所在的檔案系統支援 flock**：本地碟一律可用；網路檔案系統（NFS 尤其）的 advisory lock 語意在部分設定下不可靠。driver **不會**改用「鎖檔存在性／mtime」自製鎖——那正是無法區分「持有者還活著」與「持有者已死」的失效模式，會讓 store 要嘛永久卡死、要嘛從活著的寫者手上偷走鎖。

其他前提：

- **Single-node only。** 一個資料目錄同時只允許一個 server。第二個 server 指向同一目錄會**啟動失敗**（`unavailable`），不會等待、不會搶占，也不會交錯寫入。driver 不宣告 cluster 能力。
- **目錄是 driver 私有格式。** 手動編輯視同損毀（版本守門與 index 參照會攔下大部分亂改）。要人類可讀的匯出，用 `backup` 產生的 export bundle。
- **檔案時間戳不承載任何語意。** 排序與 revision 全部出自 index 與檔名內的序號，備份/還原工具重寫 mtime 不影響任何行為。

### 拒用的情況（fail closed）

以下情況啟動失敗且**目錄內容位元不變**——不會留下 marker、鎖檔或任何痕跡：

- 目錄非空且不是本 driver 建立的（例如路徑打錯，指到既有資料夾）
- meta 檔損毀、或記錄的 schema version 高於本 driver 支援
- 目錄由其他 driver 標記

### 磁碟用量

每次 commit 為異動文件寫入一個新的 revision 內容檔；被取代的舊 revision 檔在**下次開啟時**由孤兒掃描清除。長時間執行的 server 在重啟前，磁碟用量會隨累計寫入量成長。目前不做壓縮、去重或歷史裁剪。

## postgres

```yaml
store:
  driver: postgres
  url: postgres://speclink@db.internal:5432/speclink   # 密碼建議留空，見下
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

四張表（`documents`、`history`、`outbox`、`meta`）建在連線的 current schema，因此 URL 帶 `search_path` 即可讓多個 store 共用一個資料庫。空 schema 會初始化為現行 schema version。

### 密碼來源

密碼**優先來自環境變數 `SPECLINK_POSTGRES_PASSWORD`**：URL 省略密碼時由它補全。URL 內嵌密碼仍可啟動，但會在 stderr 輸出一行警告——組態檔會被複製、diff、貼進 issue，密碼不該躺在裡面。URL 已帶密碼時環境變數不覆蓋它。

### 前提

- **Single-node only。** 同 scope 的寫入以 PostgreSQL **transaction-scoped advisory lock**（`pg_advisory_xact_lock`）序列化：鎖隨 transaction 結束或連線死亡自動釋放，**不會有殘留鎖**；跨 scope 寫入互不阻塞；讀取不取鎖。
  但與 serverfs 不同，**兩個 server 指向同一資料庫不會被拒絕**——advisory lock 只序列化、不獨占。正確性（CAS、transaction 原子性）不受影響，資料不會損毀，但 driver **不宣告 cluster 能力**，多節點請靠部署紀律。cluster 模式待 distributed coordination 完成後另行處理。
- **最低支援版本：PostgreSQL 15。** CI 以該版本執行完整測試集。driver 未使用更新的功能，但只有 15 以上是受測的。
- **schema 是 driver 私有格式。** 手動編輯視同損毀（版本守門會攔下大部分亂改）。要人類可讀的匯出，用 `backup` 產生的 export bundle。

### 拒用的情況（fail closed）

以下情況啟動失敗且**資料庫內容不變**——偵測全程唯讀，不會留下任何痕跡：

- schema 已有資料表但不是本 driver 建立的（例如 URL 打錯，指到既有資料庫）
- `meta` 記錄的 schema version 高於本 driver 支援
- 認證失敗或資料庫不存在（回 `backend` 並帶伺服器原文）

運行中連線中斷不算損毀：請求回 `unavailable`、`/readyz` 轉紅，連線恢復後同一 server 直接續用，不需重啟。

### 測試前提

本 driver 的測試集需要**真實 PostgreSQL 實例**，由環境變數 `SPECLINK_TEST_POSTGRES_URL` 指定。未設定時測試以顯性 `skipped` 結束並印出啟用指引——不會靜默回報通過。因此 `npm run test:all` 在沒有 PostgreSQL 的機器上仍可全綠，但**完整驗證本 driver 需要 PG**；CI 另有一個 job 起 PostgreSQL 15 service container 必跑該測試集，且對 `skipped` 直接紅燈。

一行啟用：

```
docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=speclink --name speclink-pg postgres:15 && export SPECLINK_TEST_POSTGRES_URL=postgres://postgres:speclink@localhost:5432/postgres
```

## 更換 driver

資料遷移走 driver 無關的 export bundle（見[備份、還原與驗證](server-backup.zh-TW.md)）：以舊組態 `backup`，換上新組態後 `restore` 到空目標。bundle 的逐文件 digest 由契約定義，跨 driver 一致。
