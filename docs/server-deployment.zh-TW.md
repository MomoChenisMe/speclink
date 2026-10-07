# Server 部署

**繁體中文** · [English](server-deployment.md)

這份文件說明怎麼把官方 `speclink-server` 跑起來給團隊用：選跑法、設定、HTTPS、管理指令，以及升級與回退。第一次試用請先照 [Remote 入門](remote-getting-started.zh-TW.md)走一次；那份文件帶你完成初始設定、成員資格與 Desktop、CLI 的連線。

## <a id="reference-implementation"></a>官方 server 是參考實作

`speclink-server` 是 Speclink 官方的**參考實作**，讓你開箱即用、試遠端功能。遠端模式不綁這一份 server：它由 `openspec/specs/` 底下的兩份公開契約 `host-runtime` 與 `client-protocol` 定義。你可以用 Speclink 引擎自己寫 server，接上自家的認證、資料庫與權限模型，CLI 與 Desktop 一樣接得上。本文只講官方這一份怎麼部署。

## <a id="choose"></a>選一種跑法

| 跑法 | 適合 | 資料位置 |
| --- | --- | --- |
| [npx](#npx) | 有 Node.js 的單機試用 | `./speclink-data/` |
| [Docker](#docker) | 單一容器 | 容器內 `/data` |
| [Docker Compose（SQLite）](#compose) | 正式部署的預設選擇 | named volume |
| [Docker Compose（PostgreSQL）](#compose) | 已經有 PostgreSQL 維運的團隊 | PostgreSQL＋named volume |
| [從原始碼建置](#build-from-source) | 需要自己的 binary | 組態檔指定的位置 |

不論哪種跑法，都是同一支 binary。瀏覽器主控台（`apps/server-web`）在編譯時就內嵌進 `speclink-server`，和 API 由同一個網址提供。執行時不需要 Node.js、外部的 `dist` 資料夾、CDN，也不需要另一個靜態檔服務。

每種儲存後端都只支援**一個** server instance。不要 `--scale`，也不要讓兩個 server 指向同一份資料；SQLite 與 PostgreSQL 不會擋第二個 instance，詳見 [Server 儲存後端](server-store-drivers.zh-TW.md)。

啟動後，每種跑法的行為都一樣：

- `GET /healthz` 回 `200` 代表程序活著；`GET /readyz` 回 `200` 代表儲存後端可用，不可用時回 `503`。
- 第一次啟動（還沒有管理員）時，stdout 印出一次性的 `/setup?token=…` 連結，24 小時內有效。完成設定後，設定就永久關閉。
- 組態有錯（檔案讀不到、格式不對、未知的儲存後端）時，server 不開埠，直接以非零 exit code 結束。

## <a id="npx"></a>npx

需要 Node.js 18 以上。支援 macOS（arm64、x64）、Linux（x64、arm64）與 Windows（x64）。

```bash
npx @speclink/server
```

不帶參數時，啟動器做三件事：

1. 依[環境變數](#configuration)產生 `speclink-data/config.yaml`。每次啟動都重新產生，手改的內容會被蓋掉。
2. 預設用 SQLite，資料放在 `./speclink-data/`。
3. 啟動 server，只聽 `127.0.0.1:<SPECLINK_PORT>`。

帶了參數時，啟動器把參數原封不動交給 server binary，例如 `npx @speclink/server invite --config ./speclink-data/config.yaml …`。想要固定的 `speclink-server` 指令，可以 `npm i -g @speclink/server`。

npx 預設只有本機連得到。要給團隊用，建議改用 Docker Compose，並在前面架 [HTTPS 反向代理](#https)。如果一定要用 npx，請自己維護一份組態檔，用 `npx @speclink/server --config <檔案> --addr 0.0.0.0:8080` 啟動；帶 `--config` 時，啟動器不會改寫你的組態檔。

## <a id="docker"></a>Docker

官方映像是 `ghcr.io/momochenisme/speclink-server`，支援 `linux/amd64` 與 `linux/arm64`。每次發版會推三個 tag：完整版號（例如 `0.8.0`）、`主.次` 版號（例如 `0.8`）與 `latest`。

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  ghcr.io/momochenisme/speclink-server:latest
docker logs speclink        # 取得一次性設定連結
```

映像內建一份組態：store 與 identity 兩個 SQLite 檔都放在 `/data`，`public_url` 是預設的 `http://localhost:8080`。server 以 uid 10001 的非 root 使用者執行，容器的健康檢查打 `/healthz`。

組態檔不展開環境變數。要改 `public_url` 或換儲存後端，就掛載自己的組態檔，蓋掉映像內的那一份：

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  -v ./server.yaml:/etc/speclink/config.yaml:ro \
  ghcr.io/momochenisme/speclink-server:latest
```

埠對映不是 `8080:8080`，或對外網址不是 `http://localhost:8080` 時，一定要改 `public_url`。否則設定連結的網址不對，瀏覽器送出的表單也會因為來源不符而被拒。

## <a id="compose"></a>Docker Compose

repo 的 `deploy/` 有兩份 compose 檔。兩份都用 named volume 保存 `/data`，並由環境變數產生組態。

SQLite（預設）：

```bash
cd deploy
docker compose pull          # 先拉官方映像
docker compose up -d
docker compose logs server   # 取得一次性設定連結
```

PostgreSQL（server＋postgres 兩個服務）：

```bash
cd deploy
cp .env.example .env         # 填入 SPECLINK_POSTGRES_PASSWORD；.env 不進版本控制
docker compose -f docker-compose.postgres.yml pull
docker compose -f docker-compose.postgres.yml up -d
```

PostgreSQL 版的 server 會等 postgres 健康後才啟動；帳號資料（identity）仍是 `/data` 底下的 SQLite 檔。儲存後端的限制見 [Server 儲存後端](server-store-drivers.zh-TW.md#postgres)。

compose 檔同時寫了 `image:` 與 `build:`。沒有先 `pull` 而本機也沒有映像時，`up` 會從原始碼建置映像；這需要完整的 repo，Rust 編譯要幾分鐘。

容器重啟後，資料留在 volume 裡，設定連結不會再印。

## <a id="build-from-source"></a>從原始碼建置

從原始碼建置的順序是 `npm ci`、`npm run build -w apps/server-web`，再編譯 `speclink-server`。在 repo 的 checkout 裡執行：

```bash
npm ci
npm run build -w apps/server-web
cargo build --release -p speclink-server
```

產物是 `target/release/speclink-server`。第二步產出的 `apps/server-web/dist` 會在第三步內嵌進 binary。少了 `apps/server-web/dist/index.html`，release 建置會直接編譯失敗，不會產出只有 API、沒有主控台的 binary。Docker 映像用多階段建置走同一個順序，最終映像裡沒有 Node.js。

準備好組態檔後啟動：

```bash
./target/release/speclink-server --config /etc/speclink/server.yaml --addr 0.0.0.0:8080
```

binary 一定要帶 `--config`。`--addr` 預設是 `127.0.0.1:8080`，只有本機連得到；要對外服務，就明確指定綁定位址。用 systemd 等程序管理器時，設 `Restart=on-failure` 即可。

## <a id="configuration"></a>環境變數與組態檔

### 環境變數

| 變數 | 適用跑法 | 預設 | 說明 |
| --- | --- | --- | --- |
| `SPECLINK_STORE` | npx | `sqlite` | 儲存後端：`sqlite`、`serverfs` 或 `postgres`。 |
| `SPECLINK_DATA_DIR` | npx | `./speclink-data` | 資料目錄，組態檔、資料庫都放這裡。 |
| `SPECLINK_PORT` | npx、Compose | `8080` | npx：server 在 `127.0.0.1` 上聽的埠。Compose：主機對外的埠（容器內固定是 8080）。 |
| `SPECLINK_PUBLIC_URL` | npx、Compose | `http://localhost:<SPECLINK_PORT>` | 對外網址，也就是使用者瀏覽器網址列上的協定、主機與埠。設定連結、邀請連結與同源檢查都以它為準。正式部署一定要設。 |
| `SPECLINK_POSTGRES_URL` | npx | 無（`postgres` 時必填） | PostgreSQL 連線 URL，建議不含密碼。 |
| `SPECLINK_POSTGRES_PASSWORD` | 所有跑法 | 無（Compose 的 PostgreSQL 版必填） | PostgreSQL 密碼。server 程式自己讀這個變數，補進不含密碼的連線 URL。 |
| `SPECLINK_CONFIG` | npx | 無 | 改用一份既有的組態檔。設了之後，上面 npx 專用的變數都不作用，也不帶 `--addr`，所以 server 聽 `127.0.0.1:8080`。這時 `invite` 這類子命令仍要自己帶 `--config`。 |

直接執行 binary 或 `docker run` 時，只有 `SPECLINK_POSTGRES_PASSWORD` 會被讀取；其他設定都寫在組態檔裡。

### 組態檔

server 只讀一份 YAML 組態檔：

| 欄位 | 必填 | 說明 |
| --- | --- | --- |
| `store` | 是 | 儲存後端（`driver` 加上 `path` 或 `url`），見 [Server 儲存後端](server-store-drivers.zh-TW.md)。 |
| `identity` | 是 | 帳號資料庫：`driver: sqlite` 加上 `path`。 |
| `public_url` | 否 | 對外網址，預設 `http://localhost:8080`。 |
| `events` | 否 | 即時事件串流的調整值：`retention`（每個範圍保留的事件數，預設 1024）、`buffer`（每條連線的緩衝，預設 256）、`heartbeat_secs`（心跳間隔秒數，預設 15）。 |

範例：

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
public_url: https://speclink.example.com
```

舊版組態裡的 `tokens` 或 `projects` 段已經廢除。留著它們，server 會拒絕啟動，並說明替代做法。

## <a id="https"></a>HTTPS 與反向代理

server 本身不處理 TLS。給團隊用時，請在前面架一層反向代理，由它提供 HTTPS。

這是必要的：登入用的 cookie 一律帶 `Secure` 屬性，瀏覽器只在 HTTPS 或 `http://localhost` 下保存它。用 `http://` 加上非 localhost 的網址部署時，瀏覽器不會保存登入狀態，主控台與裝置登入的核准頁都登不進去。

反向代理要做到這幾件事：

- 提供 HTTPS，把請求轉給 server 的埠。
- 把 `public_url` 設成使用者實際開的 HTTPS 網址。協定、主機或埠不一致時，瀏覽器送出的寫入請求會被拒（`403`，`same_origin_required`）。
- 不要緩衝 `/api/speclink/v1/projects/<專案代號>/events`。這是長時間開著的事件串流（Server-Sent Events），server 每 15 秒送一次心跳；代理的閒置逾時要比這個長。

## <a id="manage"></a>管理指令

管理主控台能做的事，大多也有命令列版本，適合寫成腳本。每個子命令都要用 `--config` 指向 server 的組態檔：

| 指令 | 用途 |
| --- | --- |
| `invite --email <email> --display <名稱>` | 建立一次性邀請，印出接受網址。`--project <專案代號>` 可重複，對方接受後成為該專案的 `editor`；`--admin` 給管理員身分；`--expires-in-days` 預設 7 天。 |
| `user suspend --email <email>`／`user reactivate --email <email>` | 停權或復權。最後一位有效的管理員不能停權。 |
| `token revoke --token-id <id>` | 撤銷一把存取金鑰。id 以 `pat_` 開頭，不是金鑰明文；主控台只顯示前綴，所以一般在 `/admin/credentials` 撤銷比較方便。 |
| `project create --key <專案代號> [--name <名稱>]` | 建立專案。 |
| `repo create --project <專案代號> --key <儲存庫代號> [--name <名稱>]` | 在專案裡建立儲存庫。 |

沒有替既有帳號加成員資格的命令列指令；請用 `/admin/users`。

依跑法不同，指令的開頭也不同：

```bash
# npx（在資料目錄的上一層執行）
npx @speclink/server invite --config ./speclink-data/config.yaml \
  --email dev@example.com --display "Dev" --project demo

# Docker Compose（server 執行中）
docker compose exec server speclink-server invite \
  --config /etc/speclink/config.yaml \
  --email dev@example.com --display "Dev" --project demo
```

備份、驗證與還原的指令見 [Server 備份與還原](server-backup.zh-TW.md)。

## <a id="verify-deployment"></a>部署後檢查

部署好、完成 `/setup` 之後，用下面三項確認 server 真的能用。

第一，看主控台總覽。剛完成設定時，總覽最上面會出現一次「開始使用」區塊，列出服務網址、專案代號與儲存庫代號：

![Server 主控台總覽，顯示服務網址、專案與儲存庫代號、計數與系統健康](assets/screenshots/server-overview.png)

- 服務網址要和使用者實際開的網址一樣。不一樣就改 `public_url`。
- 「系統健康」顯示「正常」，資料結構版本有值。
- 「需要處理」寫著「尚無有效憑證——遠端工作流程無法連線」是正常的。有人用 Desktop 或 CLI 登入、或建立存取金鑰之後，它就會消失。

第二，看使用者頁，確認成員資格。能登入不等於看得到專案；角色與成員資格是兩個欄位。初始設定建立的管理員，成員資格一開始是空的：

![Server 主控台的使用者頁與使用者詳情，顯示角色與成員資格欄](assets/screenshots/server-members.png)

每位要用遠端的人，都要在「成員資格」欄看到目標專案。加成員資格的步驟見 [Remote 入門](remote-getting-started.zh-TW.md#grant-membership)。

第三，從另一台電腦打一次健康檢查：

```bash
curl -o /dev/null -w "%{http_code}\n" https://<你的網址>/healthz
```

回 `200` 才算通過。

## <a id="upgrade"></a>升級

升級就是換新版重啟。server 只跑一個 instance，所以沒有滾動更新。

1. 先備份，並確認 `verify-backup` 通過。做法見 [Server 備份與還原](server-backup.zh-TW.md)。
2. 換成新版並重啟：
   - Docker Compose：`docker compose pull && docker compose up -d`。volume 不受影響。
   - npx：`npx @speclink/server@<新版號>`。
   - 自建 binary：換掉 binary 後重啟。
3. 確認 `/healthz` 與 `/readyz` 都回 `200`。

新版需要較新的帳號資料庫（identity）結構時，server 啟動時會自動升級它。組態或資料不相容時，server 直接以非零 exit code 結束，不會帶著錯誤啟動。

## <a id="rollback"></a>回退

回退就是部署**上一版**的 binary 或映像：

- Docker Compose：把 compose 檔的 `image:` 改成上一版的 tag（例如 `ghcr.io/momochenisme/speclink-server:0.7.0`），再 `docker compose up -d`。volume 不受影響。
- npx：`npx @speclink/server@<上一版號>`。
- 自建 binary：checkout 上一版的 tag 重新建置，或改用上一版的映像。

到 0.8.0 為止，各版的資料格式都相同（帳號資料庫結構版本 6、儲存格式版本 1），回退不需要動資料。主控台「系統」頁的「資料結構版本」顯示目前的帳號資料庫版本。

之後如果有新版提高帳號資料庫的版本，升級時會自動改寫資料庫，而舊版 server 會拒絕開啟較新的資料庫。那時要回退，就得用升級前的備份，還原到空的目標。所以升級前一定要先備份。

## <a id="related"></a>相關文件

- [Remote 入門](remote-getting-started.zh-TW.md)：第一次設定、成員資格、Desktop 與 CLI 連線。
- [Server 儲存後端](server-store-drivers.zh-TW.md)：SQLite、serverfs、PostgreSQL 怎麼選。
- [Server 備份與還原](server-backup.zh-TW.md)：`backup`、`verify-backup`、`restore` 與排程。
