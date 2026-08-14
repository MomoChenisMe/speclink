# Server 備份、還原與驗證

官方 `speclink-server` 內建三個離線子命令，把 TeamStore 契約的 export／import 接成營運能力：

- `backup` — 產生完整備份
- `verify-backup` — 檢查備份完整性
- `restore` — 還原到空目標，並自動檢查結果

三者都對 `--config` 指向的**未運行**資料操作。

## 備份檔內容

備份是**單一自描述的 tar 檔**（不壓縮），含：

- `manifest.json`：備份格式版本、UTC 建立時間、engine 版本、store manifest（driver、contract 版本）、identity schema 版本、scope 清單與逐 scope 文件數、identity 計數，以及每個成員檔的 digest。
- `manifest.json.sha256`：manifest 自身的 digest 側檔（信任鏈的根）。
- `bundles/<n>.json`：每個 registry scope 一個 export bundle（經 TeamStore export 契約產生，**非**資料庫檔拷貝）。
- `identity.db`：identity 資料庫的時點一致快照（走 SQLite 線上備份 API，WAL 一併收斂）。

備份**不含任何憑證明文**：identity 庫本就只存 hash（密碼 argon2id、token SHA-256）。

## 前提：離線一致性

備份的一致性前提是**備份期間無寫入**。請在停機或部署層的維護窗口執行：先停止 `speclink-server`，再跑 `backup`，完成後才重新啟動。目前不支援執行中 server 的線上快照。

## backup — 產生備份

```bash
speclink-server backup \
  --config /etc/speclink/server.yaml \
  --output /var/backups/speclink/backup-$(date -u +%Y%m%dT%H%M%SZ).tar
```

`--config` 定位 store 與 identity 資料庫，其中 identity 需為 sqlite driver。成功之後，指令把結果摘要寫入 identity 庫的備份記錄。你可以在 `/admin` 的資料操作頁看到最近一次備份資訊。

## verify-backup — 只驗證完整性

這個子命令不還原，也不需要空目標，只讀備份檔。它做三件事：比對 manifest 與逐成員 digest、解析 bundle 結構、檢查備份格式版本。

全數通過就回 0。**任何一個位元被竄改**，或遇上**未知的格式版本**，它回非零並指出原因。

```bash
speclink-server verify-backup --input /var/backups/speclink/backup-latest.tar
```

加上 `--config` 時，會把驗證結果一併寫入該 identity 庫的備份記錄：

```bash
speclink-server verify-backup \
  --input /var/backups/speclink/backup-latest.tar \
  --config /etc/speclink/server.yaml
```

例行備份後排一支 `verify-backup` 當健康檢查。它不需要空環境，就能確認備份可用。

## restore — 還原到空目標並驗證

`restore` **只還原到空目標**，也就是 store 與 identity 都空。目標非空時它直接拒絕，輸出既有內容摘要，一個位元都不寫。沒有覆蓋旗標。

還原分四步：

1. 完整性檢查，等同 `verify-backup`。
2. identity 快照落位。
3. 逐 scope import，全部全新建立。
4. 收尾核對：逐 scope 比對內容 digest 與文件數，並把 identity 計數與 schema version 對 manifest 比對。

```bash
speclink-server restore \
  --config /etc/speclink/target.yaml \
  --input /var/backups/speclink/backup-latest.tar
```

收尾核對全綠就回 0。**任何一項不符**，它回非零、逐項列出差異，並明示該目標不可投產。被竄改的備份或未知格式版本，在第一步就被擋下（fail closed）。

還原到新版 server 之後，資料庫版本升級由既有的 migrate 機制負責。入口是 `/admin` 的 store 遷移，前置 health 檢查通過才會執行。

## 排程範例

備份必須在無寫入窗口執行，所以下面每個範例都走「停機 → 備份 → 檢查 → 啟動」。保留輪替屬於部署層的事，這裡只用 `find` 清理舊檔示意。

### systemd timer

`/etc/systemd/system/speclink-backup.service`：

```ini
[Unit]
Description=Speclink server offline backup
After=speclink-server.service

[Service]
Type=oneshot
# 停機取得無寫入窗口，備份＋驗證後再啟動。
ExecStartPre=/usr/bin/systemctl stop speclink-server.service
ExecStart=/bin/sh -c 'F=/var/backups/speclink/backup-$(date -u +%%Y%%m%%dT%%H%%M%%SZ).tar; \
  /usr/local/bin/speclink-server backup --config /etc/speclink/server.yaml --output "$F" && \
  /usr/local/bin/speclink-server verify-backup --input "$F" --config /etc/speclink/server.yaml && \
  find /var/backups/speclink -name "backup-*.tar" -mtime +14 -delete'
ExecStartPost=/usr/bin/systemctl start speclink-server.service
```

`/etc/systemd/system/speclink-backup.timer`：

```ini
[Unit]
Description=Nightly Speclink backup

[Timer]
OnCalendar=*-*-* 03:30:00 UTC
Persistent=true

[Install]
WantedBy=timers.target
```

啟用：`systemctl enable --now speclink-backup.timer`。

### cron

```cron
# 每日 03:30 UTC：停機 → 備份 → 驗證 → 啟動 → 清理 14 天前的舊備份
30 3 * * * F=/var/backups/speclink/backup-$(date -u +\%Y\%m\%dT\%H\%M\%SZ).tar; \
  systemctl stop speclink-server && \
  speclink-server backup --config /etc/speclink/server.yaml --output "$F" && \
  speclink-server verify-backup --input "$F" --config /etc/speclink/server.yaml; \
  systemctl start speclink-server && \
  find /var/backups/speclink -name 'backup-*.tar' -mtime +14 -delete
```

> 備份檔搬運到異機／異地屬部署層，本刀輸出的是本機檔案。定期用另一台主機的空目標跑一次 `restore` 做災難演練，確認備份真的可還原。
