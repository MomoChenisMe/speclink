# Server Store Driver 選型

官方 `speclink-server` 的持久層由組態檔的 `store` 段決定，經單一建構點接線。所有 driver 都通過同一套 TeamStore conformance suite（能力檢查、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 四故障點、tenant scope），因此**動詞行為與 driver 無關**——換 driver 不會改變 API 的可觀察結果。

| driver | 承載 | 適用 |
|---|---|---|
| `sqlite` | 單一資料庫檔 | **預設**。無特殊前提，任何檔案系統可用 |
| `serverfs` | 單一資料目錄 | 偏好純檔案持久層（備份工具直接可見目錄、無資料庫維運）；**前提見下** |
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

## 更換 driver

資料遷移走 driver 無關的 export bundle（見[備份、還原與驗證](server-backup.zh-TW.md)）：以舊組態 `backup`，換上新組態後 `restore` 到空目標。bundle 的逐文件 digest 由契約定義，跨 driver 一致。
