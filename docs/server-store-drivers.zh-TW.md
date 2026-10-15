# Server 儲存後端

**繁體中文** · [English](server-store-drivers.md)

儲存後端（Store driver）決定 server 把專案資料（規格、變更、討論等）放在哪裡。大多數情況用預設的 `sqlite` 就好。

幾件先知道的事：

- 帳號、成員資格與憑證放在另一個 SQLite 檔（組態檔的 `identity` 段），不跟著儲存後端換。
- 三種儲存後端都通過同一套一致性測試，所以 API 與 CLI 的行為完全相同，換後端不影響使用方式。
- 每種儲存後端都只支援**一個** server instance。

部署方式與完整的組態欄位見 [Server 部署](server-deployment.zh-TW.md)。

## <a id="choose"></a>怎麼選

| 儲存後端 | 資料放哪 | 適合 | 要注意 |
| --- | --- | --- | --- |
| `sqlite`（預設） | 一個資料庫檔 | 大多數情況 | 只能放在本機磁碟。不會擋第二個 instance。 |
| `serverfs` | 一個資料目錄 | 想用純檔案保存資料、不想維運資料庫 | 檔案系統要支援 `flock` 檔案鎖。 |
| `postgres` | PostgreSQL 資料庫 | 已經有 PostgreSQL 維運與備份的團隊 | PostgreSQL 15 以上；不支援 TLS 連線。不會擋第二個 instance。 |
| `memory` | 記憶體 | 只供測試 | 程序結束資料就消失。npx 啟動器不接受它。 |

組態裡寫了不認得的儲存後端名稱，server 不會啟動，錯誤訊息會列出支援的名稱。拼錯 `serverfs` 不會默默改用別的後端。

npx 用環境變數 `SPECLINK_STORE` 選後端；其他跑法在組態檔的 `store` 段設定，見下面各節。

## <a id="sqlite"></a>sqlite（預設）

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

- 資料庫用 SQLite 的 WAL 模式，旁邊會多出 `store.db-wal` 與 `store.db-shm` 工作檔。搬移或複製資料時要一起處理，或者直接用 [`backup`](server-backup.zh-TW.md)。
- 請放在本機磁碟。WAL 模式不支援 NFS、SMB 這類網路檔案系統。
- 只能有一個 server 使用這個檔案。server **不會**擋第二個指向同一個檔案的 instance，所以要靠你自己避免。

## <a id="serverfs"></a>serverfs

```yaml
store:
  driver: serverfs
  path: /var/lib/speclink/store     # 這是目錄，不是檔案
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

`path` 指向一個**資料目錄**。目錄不存在時，server 會建立它；空目錄會被初始化成新的儲存區。

### 檔案鎖

serverfs 用作業系統的檔案鎖（`flock`）保證同一時間只有一個 server 寫入。鎖由作業系統持有：server 被強制結束、當掉或斷電時，鎖會自動釋放，下一個 server 可以直接接手，不需要手動清鎖。

第二個 server 指向同一個目錄時，它會啟動失敗並回報 `unavailable`，不會等待也不會搶走鎖。

部署前請確認資料目錄所在的檔案系統支援 `flock`。本機磁碟都支援；NFS 等網路檔案系統在部分設定下不可靠，不建議使用。

### 拒絕啟動的情況

下列情況 server 不會啟動，而且不改動目錄裡的任何內容：

- 目錄不是空的，也不是 serverfs 建立的（例如路徑打錯，指到別的資料夾）。
- 目錄裡的版本資訊損壞，或版本比這個 server 支援的還新。
- 目錄被別的儲存後端標記過。

### 其他限制

- 目錄格式是 serverfs 私有的。不要手動編輯；要人看得懂的匯出，用 [`backup`](server-backup.zh-TW.md)。
- 檔案的修改時間不代表任何意義。備份工具改寫修改時間，不影響任何行為。
- 每次寫入都會為改動的文件寫一個新版本檔。被取代的舊版本檔要等**下次啟動**時才清掉，所以長時間不重啟，磁碟用量會隨寫入量增加。目前沒有壓縮或歷史裁剪。

## <a id="postgres"></a>postgres

```yaml
store:
  driver: postgres
  url: postgres://speclink@db.internal:5432/speclink   # 不放密碼，見下方
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

server 會在連線的目前 schema 建四張資料表：`documents`、`history`、`outbox` 與 `meta`。所以用不同的 `search_path`，就能讓多個儲存區共用一個資料庫。空的 schema 會被初始化。

### 密碼

密碼請放在環境變數 `SPECLINK_POSTGRES_PASSWORD`。連線 URL 不含密碼時，server 會用這個變數補上。

URL 裡直接寫密碼也能啟動，但 server 會在 stderr 印一行警告：組態檔常被複製、比對、貼進 issue，不該放密碼。URL 已經帶密碼時，環境變數不會覆蓋它。

### 限制

- **最低版本 PostgreSQL 15。** CI 用這個版本跑完整測試。
- **不支援 TLS 連線。** server 一律用不加密的連線。只接受 TLS 的託管 PostgreSQL 會連不上；請把資料庫放在受信任的內部網路。
- **只能有一個 server。** 同一範圍的寫入會依序執行，資料不會損壞，但 server **不會**拒絕第二個指向同一個資料庫的 instance。多台 server 共用一個資料庫不在支援範圍內。
- **資料表格式是私有的。** 不要手動編輯；要人看得懂的匯出，用 [`backup`](server-backup.zh-TW.md)。

### 拒絕啟動的情況

下列情況 server 不會啟動，也不改動資料庫的內容：

- schema 裡已經有資料表，但不是 Speclink 建立的（例如 URL 打錯，指到別的資料庫）。
- `meta` 記錄的版本比這個 server 支援的還新。
- 認證失敗或資料庫不存在。錯誤訊息會附上 PostgreSQL 的原文。

執行中斷線不算資料損壞：請求回 `unavailable`，`/readyz` 回 `503`。連線恢復後，同一個 server 會直接續用，不需要重啟。

## <a id="switch"></a>更換儲存後端

儲存後端之間不能直接搬檔案，要經過備份檔：

1. 停掉 server，用舊組態執行 `backup`。
2. 準備新組態，指向一個空的新儲存區。
3. 用新組態執行 `restore`，再用新組態啟動 server。

備份檔的內容與儲存後端無關，`restore` 會逐項比對還原結果。指令與規則見 [Server 備份與還原](server-backup.zh-TW.md)。
