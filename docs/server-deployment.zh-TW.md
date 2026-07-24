# Server 部署

官方 `speclink-server` 有四種發布形態（架構 §13.1）：native binary、Docker image、SQLite 單容器 compose、PostgreSQL compose profile。每個 release tag 同時產出全部四種：GitHub Release 附各平台 binary 壓縮檔與 `SHA256SUMS.txt`，Docker image 發布於 `ghcr.io/momochenisme/speclink-server`（tag 對齊版本，另附 `latest`）。

若目標是從全新資料完成 `/setup`、membership、Desktop 與 Remote CLI，而不是部署正式服務，請先依
[Remote Server、Desktop 與 CLI 入門教學](remote-getting-started.zh-TW.md)操作。

不論哪種形態，啟動後的行為一致：

- **健康檢查**：`GET /healthz` 回程序存活、`GET /readyz` 回 store 就緒（未就緒為 503）。
- **setup token**：首次啟動（尚無 admin）時，stdout 印一行含 `/setup?token=…` 的一次性連結，24 小時內有效。容器形態用 `docker compose logs server`（或 `docker logs <容器>`）取得。開瀏覽器走完 `/setup`（建立 admin → 首個 project/repo）後 setup 永久關閉。
- **組態**：YAML 檔經 `--config` 指定，欄位與各 store driver 的選型見[Server Store Driver 選型](server-store-drivers.zh-TW.md)。組態壞或 driver 未知時 server 拒絕啟動並以非零 exit code 結束（fail closed），不會以部分預設服務。

## 單一 instance 限制

SQLite 與 serverfs profile **只允許一個 server instance**（架構 §13.1）：不得 `--scale`、不得多個 replica 指向同一個資料目錄或 volume。SQLite 的單寫者檔案鎖會讓第二個 instance 顯性報錯而非靜默共用。需要多 instance 前先換 PostgreSQL driver——即便如此，目前的官方形態仍以單 instance 為設計定位。

## 形態一：native binary

從 GitHub Release 下載對應平台的 `speclink-server-<版本>-<target>.tar.gz`（Windows 為 `.zip`），驗過 checksums 後解出單一 binary。準備組態檔後啟動：

```bash
speclink-server --config /etc/speclink/server.yaml --addr 0.0.0.0:8080
```

`--addr` 預設 `127.0.0.1:8080`（僅本機）；要對外服務必須明示綁定位址。搭配 systemd 等程序管理器時，`Restart=on-failure` 即可承接 fail closed 的退出。

## 形態二：docker run

映像的 ENTRYPOINT 是 server binary，內建預設組態（store 與 identity 兩個 SQLite 檔都在 `/data`）、非 root 使用者（uid 10001）執行、HEALTHCHECK 打 `/healthz`：

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  ghcr.io/momochenisme/speclink-server:latest
docker logs speclink        # 取 setup token
```

要改 `public_url` 等欄位時，掛載自訂組態覆蓋映像內的預設檔（組態 YAML 不做環境變數展開）：

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  -v ./server.yaml:/etc/speclink/config.yaml:ro \
  ghcr.io/momochenisme/speclink-server:latest
```

## 形態三：SQLite compose（一行開箱）

`deploy/docker-compose.yml` 是正典示範：named volume 持久化 `/data`，`public_url` 經環境變數插值。

```bash
cd deploy
docker compose pull          # 取官方映像；略過則直接從原始碼建置
docker compose up -d
docker compose logs server   # 取 setup token，開瀏覽器完成 /setup
```

compose 同時寫了 `image:` 與 `build:`：本機沒有映像時 `up` 會就地從原始碼建置（需要完整的 repo，Rust 編譯數分鐘），因此**首次正式發版前也能起**。要用官方映像就先 `docker compose pull`。

容器重啟後資料存留於 volume，setup token 不會重印、`/setup` 維持關閉。

## 形態四：PostgreSQL compose profile

`deploy/docker-compose.postgres.yml` 起 server ＋ postgres 兩服務：postgres 帶 `pg_isready` healthcheck，server 等它 healthy 才啟動。密碼只經環境變數注入——store url 不含密碼，由 `SPECLINK_POSTGRES_PASSWORD` 補全（見 [Server Store Driver 選型](server-store-drivers.zh-TW.md)的密碼來源）。identity 資料庫維持 `/data` 下的 SQLite 檔。

```bash
cd deploy
cp .env.example .env         # 填入 SPECLINK_POSTGRES_PASSWORD；.env 不入版本控制
docker compose -f docker-compose.postgres.yml pull
docker compose -f docker-compose.postgres.yml up -d
```

## 環境變數清單

compose 形態可用的環境變數（`.env` 或 shell 匯出皆可）：

| 變數 | 預設 | 說明 |
| --- | --- | --- |
| `SPECLINK_PUBLIC_URL` | `http://localhost:8080` | 對外網址。同源檢查與 setup 連結都以此為準，正式部署必設。 |
| `SPECLINK_PORT` | `8080` | 對外埠映射（容器內固定 8080）。 |
| `SPECLINK_POSTGRES_PASSWORD` | （必填，僅 PostgreSQL profile） | 資料庫密碼。同一值初始化 postgres 服務並補全 server 連線 URL；server 程序本身也讀這個變數。 |

## 容器內執行子命令

映像的 ENTRYPOINT 是 server binary，`docker run`／`docker compose run` 的尾參數就是子命令。`backup`、`verify-backup`、`restore` 對**未運行**的資料操作（離線一致性，詳見[Server 備份、還原與驗證](server-backup.zh-TW.md)），流程是停 server → 以同一 volume 起一次性容器跑子命令 → 再啟動。

映像以 uid 10001 執行，因此**掛進去的宿主目錄必須先開放給該 uid 寫入**，否則 `--output` 會得到 Permission denied（Docker Desktop 會做 ownership 重映射而看不出來，Linux 主機則必然踩到）：

```bash
cd deploy
mkdir -p backups && sudo chown 10001:10001 backups   # 一次性；容器內的 uid

docker compose stop server
docker compose run --rm -v ./backups:/backups server \
  backup --config /etc/speclink/config.yaml --output /backups/backup-$(date -u +%Y%m%dT%H%M%SZ).tar
docker compose start server

# 驗證備份不需要停機、不需要空目標
docker compose run --rm -v ./backups:/backups server \
  verify-backup --input /backups/backup-latest.tar
```

還原**只接受空目標**（store 與 identity 皆空，非空即拒絕）。在全新環境把 volume 準備好但**不啟動 server**（不走 /setup），直接以一次性容器還原，完成後再啟動：

```bash
cd deploy
docker compose create server      # 建立容器與空 volume，不啟動
docker compose run --rm -v ./backups:/backups server \
  restore --config /etc/speclink/config.yaml --input /backups/backup-latest.tar
docker compose start server
```

`invite` 等 identity 操作可直接在運行中的容器內執行：

```bash
docker compose exec server speclink-server invite \
  --config /etc/speclink/config.yaml \
  --email dev@example.com --display "Dev" --project demo
```

## 升級

升級＝換映像重啟，沒有滾動更新（單 instance 定位）：

1. 先備份（上一節），確認 `verify-backup` 綠。
2. `docker compose pull && docker compose up -d` 以新映像重建容器（volume 不動）。
3. 啟動守門承接相容性：組態或資料不相容時容器以非零 exit code 結束，資料不會被半升級；資料庫 schema 落後新版時 server 會啟動但 `/readyz` 回 503。
4. 需要資料層遷移時，由 admin 在 `/admin` 資料操作頁觸發 store 遷移（前置 health 檢查通過才執行，成功記入 audit）。
5. `/healthz`、`/readyz` 皆綠即完成。

## 相關文件

- [Server 備份、還原與驗證](server-backup.zh-TW.md)——backup/verify-backup/restore 的完整語意與排程範例
- [Server Store Driver 選型](server-store-drivers.zh-TW.md)——sqlite/serverfs/postgres 的組態欄位、前提與 fail closed 條件
- [平台架構](platform-architecture.zh-TW.md) §13——發布形態、secret 紀律與開箱流程的正典定義
