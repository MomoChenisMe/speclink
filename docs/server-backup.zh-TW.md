# Server 備份與還原

**繁體中文** · [English](server-backup.md)

官方 server 是示範與試用用的。它讓你不寫程式就能試遠端專案。正式使用請把引擎內嵌進自己的系統。見[把 Speclink 內嵌進你的系統](../README.zh-TW.md#embedding)。

官方的 `speclink-server`（示範用 server）內建三個子命令，負責備份、驗證與還原：

| 子命令 | 做什麼 | server 要停嗎 |
| --- | --- | --- |
| `backup` | 把專案資料與帳號資料打包成一個備份檔 | 要停 |
| `verify-backup` | 只檢查備份檔是否完整，不還原 | 不用 |
| `restore` | 把備份還原到**空的**目標，並逐項核對結果 | 要停 |

備份檔與儲存後端無關：從 `sqlite` 備份，可以還原到 `serverfs` 或 `postgres`。更換儲存後端的步驟見[儲存接口的範例](server-store-drivers.zh-TW.md#switch)。

備份格式目前是第 2 版。0.12.0 與更早的版本做的是第 1 版，新版一樣能檢查與還原；見[備份格式的版本](#format-versions)。從 0.12.0 或更早的版本升級，也是靠備份與還原，步驟見 [Server 部署](server-deployment.zh-TW.md#upgrade-from-0-12)。

## <a id="rules"></a>兩條規則

**一、備份前先停 server。** 備份期間不能有任何寫入，否則備份可能前後不一致。目前不支援對執行中的 server 做線上備份。流程固定是：停 server → `backup` → 啟動 server。

**二、還原只接受空目標。** 「空」的意思是：

- 帳號資料庫裡還沒有任何使用者；而且
- 備份裡的每個專案／儲存庫，在目標的儲存後端裡都沒有內容。

目標不是空的，`restore` 會直接拒絕、列出目標裡已有的東西，一個位元都不寫。沒有「強制覆蓋」的選項。最簡單的做法是準備全新的資料目錄或空資料庫，還原前不要完成 `/setup`。還原時，目標的 server 也要停著。

## <a id="contents"></a>備份檔裡有什麼

備份是一個沒有壓縮的 tar 檔，裡面有：

- `manifest.json`：備份格式版本、UTC 建立時間、引擎版本、來源儲存後端的種類、帳號資料庫版本、每個專案／儲存庫的文件數、帳號資料的筆數，以及每個成員檔的雜湊值。
- `manifest.json.sha256`：`manifest.json` 自己的雜湊值，用來確認清單沒被改過。
- `bundles/<n>.json`：每個專案／儲存庫一份匯出檔。每份文件用它的鍵（key）標示，並附上內容與內容的雜湊值。它是把文件一份一份讀出來組成的，不是資料庫檔案的直接拷貝。
- `identity.db`：帳號資料庫在備份當下的完整快照。

匯出檔長這樣：

```json
{
  "formatVersion": 2,
  "scopeVersion": 7,
  "documents": [
    {
      "key": "changes/add-auth/proposal.md",
      "content": "…",
      "digest": "sha256:…"
    }
  ]
}
```

鍵是文件在儲存後端裡的名字，例如 `changes/add-auth/proposal.md`、`specs/accounts/spec.md`、`config.yaml`。`digest` 是 `content` 的 SHA-256 雜湊值。`scopeVersion` 是匯出當下這個專案／儲存庫的版本號。

備份檔裡沒有任何密碼或金鑰的明文：密碼以 argon2id 雜湊保存，存取金鑰與工作階段只存 SHA-256 雜湊。即使如此，備份檔仍含帳號與專案資料，請當成機密檔案保管。

### <a id="scope-export"></a>主控台的匯出下載

管理主控台「系統」頁的「匯出」區，可以逐一下載某個專案／儲存庫的匯出檔，檔名是 `<專案代號>__<儲存庫代號>.bundle.json`。

下載到的內容和備份檔裡的 `bundles/<n>.json` 是同一種：每份文件用鍵標示，附內容與雜湊值。API 的 `GET /export` 路由回的也是這一種。0.12.0 與更早的版本下載到的是舊格式，用文件種類標示文件；讀這個檔案的腳本要跟著改。

匯出檔不含帳號資料，不能取代 `backup`。

## <a id="format-versions"></a>備份格式的版本

備份格式的版本寫在 `manifest.json` 裡，`verify-backup` 的輸出會印出來。目前有兩版：

| 備份格式 | 誰做的 | 匯出檔怎麼標示文件 |
| --- | --- | --- |
| 第 1 版 | 0.12.0 與更早的版本 | 用文件種類 |
| 第 2 版 | 0.12.0 之後的版本 | 用鍵 |

哪個版本讀得了哪一種備份：

| server 版本 | 第 1 版的備份 | 第 2 版的備份 |
| --- | --- | --- |
| 0.12.0 之後的版本 | 能檢查、能還原 | 能檢查、能還原 |
| 0.12.0 與更早的版本 | 能檢查、能還原 | 讀不了 |

新版還原第 1 版的備份時，會先把每份文件的種類換成鍵，再匯入。換的動作在寫入任何資料之前做完。有文件換不成鍵時，`restore` 回非零，什麼都不寫，訊息以 `this backup cannot be restored, and nothing was written:` 開頭，後面寫出是哪個專案／儲存庫的哪一份文件。

舊版拿到第 2 版的備份時，`verify-backup` 與 `restore` 都回非零：

```text
backup integrity check failed: incompatible backup format version 2 (this build reads 1)
```

新版遇到它不認得的版本，也會拒絕：

```text
backup integrity check failed: incompatible backup format version 3 (this build reads 1 and 2)
```

所以升級到 0.12.0 之後的版本以後，新做的備份只有新版讀得了。升級前用舊版做的那一份請留著：舊版只讀得了它。

## <a id="backup"></a>備份

```bash
speclink-server backup \
  --config /etc/speclink/server.yaml \
  --output /var/backups/speclink/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar
```

`--config` 用來找到儲存後端與帳號資料庫；帳號資料庫必須是 `sqlite`。輸出檔名由你決定，本文的範例都用 UTC 時間戳命名。

**預期輸出**：

```text
備份完成：1 個 scope、2 個成員 → /var/backups/speclink/speclink-20261007T084144Z.tar
```

「scope」是專案／儲存庫的組合，「成員」是 tar 裡的資料檔數。備份成功後，server 會在帳號資料庫留一筆紀錄；在主控台的 `/admin/audit` 用動作「備份紀錄」篩選就看得到。

儲存位置放的是 0.12.0 或更早版本的資料時，新版的 `backup` 會拒絕執行，不產生備份檔，並印出升級的三個步驟。這種資料要用舊版備份，見 [Server 部署](server-deployment.zh-TW.md#upgrade-from-0-12)。

## <a id="verify"></a>驗證備份

`verify-backup` 只讀備份檔，不需要停 server，也不需要空目標。它會比對清單與每個成員檔的雜湊值、檢查匯出檔的結構，並確認備份格式版本是這個 server 認得的。

```bash
speclink-server verify-backup --input /var/backups/speclink/speclink-20261007T084144Z.tar
```

**預期輸出**：

```text
備份完整：格式版本 2、2 個成員、1 個 scope
```

0.12.0 與更早的版本做的備份，這裡印的是「格式版本 1」。

全部通過就回 0。檔案被改過任何一個位元，或格式版本不認得，它會回非零並說明原因，例如：

```text
backup integrity check failed: digest mismatch for bundles/0.json
```

加上 `--config` 時，驗證結果也會寫進那個帳號資料庫的備份紀錄。

每次例行備份之後，建議都跑一次 `verify-backup`。

## <a id="restore"></a>還原

先確認目標是空的（見[兩條規則](#rules)），而且目標的 server 停著。

```bash
speclink-server restore \
  --config /etc/speclink/target.yaml \
  --input /var/backups/speclink/speclink-20261007T084144Z.tar
```

`restore` 依序做六件事。前兩件只讀備份檔，不碰目標：

1. 檢查備份檔是否完整，等同 `verify-backup`。被改過或格式不認得的備份，在這一步就會被擋下。
2. 把每個專案／儲存庫的匯出檔準備好，並確認每份文件都能匯入。第 1 版的備份在這一步把文件種類換成鍵。
3. 確認目標是空的。不是空的就停下，目標原有的內容不動。
4. 把帳號資料庫快照放到目標位置。
5. 把每個專案／儲存庫的內容匯入目標的儲存後端。每個專案／儲存庫是一次提交，不會只寫一半。
6. 逐項核對：每個專案／儲存庫的鍵、每份文件的內容雜湊值與文件數，以及帳號資料的筆數與版本，都要和備份一致。

**預期輸出**：

```text
還原完成且驗證通過：1 個 scope 全數比對一致
```

核對有任何一項不符，`restore` 會回非零、逐項列出差異，並說明這個目標不能拿來使用。

第 5 件事不是跨專案／儲存庫的單一交易。匯入在中途失敗時，`restore` 回非零，列出已經還原的與失敗的專案／儲存庫，並說明這個目標不完整、不能拿來使用。這時請換一個空的目標重新還原。

目標不是空的時，輸出像這樣，而且目標完全沒被改動：

```text
restore target is not empty (restore only into an empty target): 1 user(s) exist
```

目標的儲存位置放的是 0.12.0 或更早版本的資料時，`restore` 同樣拒絕，並印出升級的三個步驟，見 [Server 部署](server-deployment.zh-TW.md#refused-start)。

還原完成後啟動 server。帳號資料已經還原，所以 server 不會印設定連結，原本的帳號可以直接登入。

## <a id="docker"></a>在 Docker 裡備份與還原

映像的進入點就是 `speclink-server`，所以 `docker compose run` 後面接的參數就是子命令。下面的範例在 `deploy/` 目錄、用 SQLite 版的 compose 檔執行。

容器以 uid 10001 執行。掛進去的主機目錄必須讓這個 uid 寫得進去，否則會看到 `backup io failed: Permission denied (os error 13)`。在 Linux 主機上，先執行一次：

```bash
mkdir -p backups && sudo chown 10001:10001 backups
```

備份（中間會短暫停機）：

```bash
docker compose stop server
docker compose run --rm -v ./backups:/backups server \
  backup --config /etc/speclink/config.yaml \
  --output /backups/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar
docker compose start server
```

驗證（不需要停機）：

```bash
docker compose run --rm -v ./backups:/backups server \
  verify-backup --input /backups/speclink-20261007T084144Z.tar
```

還原到全新的環境：先建立容器與空的 volume，但不要啟動 server，也不要走 `/setup`。

```bash
docker compose create server
docker compose run --rm -v ./backups:/backups server \
  restore --config /etc/speclink/config.yaml \
  --input /backups/speclink-20261007T084144Z.tar
docker compose start server
```

## <a id="schedule"></a>排程備份

因為備份要停機，排程的流程固定是：停 server → 備份 → 驗證 → 啟動 server。下面把流程寫成一個腳本，再交給 systemd timer 或 cron 執行。腳本假設 server 是名為 `speclink-server` 的 systemd 服務。

`/usr/local/bin/speclink-backup.sh`：

```sh
#!/bin/sh
set -eu
CONFIG=/etc/speclink/server.yaml
DIR=/var/backups/speclink
F="$DIR/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar"

systemctl stop speclink-server
# 不論備份成功或失敗，腳本結束時都把 server 啟動回來。
trap 'systemctl start speclink-server' EXIT
speclink-server backup --config "$CONFIG" --output "$F"
speclink-server verify-backup --input "$F" --config "$CONFIG"
# 只保留 14 天內的備份。
find "$DIR" -name 'speclink-*.tar' -mtime +14 -delete
```

記得 `chmod +x /usr/local/bin/speclink-backup.sh`。任何一步失敗，腳本會以非零結束，server 仍會被啟動回來。

### systemd timer

`/etc/systemd/system/speclink-backup.service`：

```ini
[Unit]
Description=Speclink server offline backup

[Service]
Type=oneshot
ExecStart=/usr/local/bin/speclink-backup.sh
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

在 root 的 crontab（`crontab -e`）加一行，每天 03:30 執行。時間以主機的時區為準：

```cron
30 3 * * * /usr/local/bin/speclink-backup.sh
```

### 異地保存與還原演練

備份檔只寫在本機。搬到另一台主機或異地保存，請用你既有的部署工具處理。也請定期在另一台主機上，把備份還原到空目標一次，確認備份真的能用。
