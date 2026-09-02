---
title: 啟動 server
section: Remote 模式
order: 410
keywords: [server, npx, Docker, compose, SQLite, PostgreSQL]
sources: [server-release, user-documentation]
generated: 2026-09-02
---

# 啟動 server

Speclink server 是 remote 模式的官方參考實作。它的用途是開箱即用、試用遠端功能。本頁說明四種官方啟動方式，以及啟動後第一件要找的東西：setup 連結。

> [!NOTE]
> 官方 server 不是 remote 模式的唯一路徑。你可以用 Speclink 引擎自建 server 端，CLI 與桌面 app 對自建 server 同樣可用。詳見 [remote 模式總覽](remote-overview.md)。

## 四種官方啟動方式

規格列出四種官方形態，外加一條替代路徑：

- `npx @speclink/server` 一行啟動。這是最短路徑，也是教學的首選。
- Docker 直跑映像。
- SQLite docker compose，單一服務。
- PostgreSQL compose profile，server 加 postgres 兩個服務。
- 替代路徑：從原始碼建置 server 執行檔。

server 的官方發布通路是 Docker 映像與 npm 套件，兩者的版本都對齊 release tag。GitHub Release 不提供 server 壓縮檔下載。

## 用 npx 一行啟動

1. 執行 `npx @speclink/server`。
2. 不帶參數時，啟動器依環境變數產生設定檔，寫入資料目錄，再啟動 server。
3. 首次啟動時，畫面（stdout）印出一行 `/setup?token=...` 連結。
4. 用瀏覽器打開 `/healthz` 確認 server 活著。
5. 打開 setup 連結，開始 [開箱](server-setup.md)。

啟動器讀這些環境變數：

| 環境變數 | 作用 | 預設 |
| --- | --- | --- |
| `SPECLINK_STORE` | 選儲存後端：sqlite、serverfs、postgres | sqlite |
| `SPECLINK_DATA_DIR` | 資料目錄 | `./speclink-data` |
| `SPECLINK_PUBLIC_URL` | 對外網址 | 連動連接埠 |
| `SPECLINK_PORT` | 連接埠 | 規格未載 |
| `SPECLINK_POSTGRES_URL` | postgres 連線字串 | 無 |

會出錯的地方：

- 設了 `SPECLINK_STORE=postgres` 但沒設 `SPECLINK_POSTGRES_URL`：啟動器以非零結束，錯誤訊息點名 `SPECLINK_POSTGRES_URL`。server 不啟動，設定檔也不產生。
- 帶 `--config`、設 `SPECLINK_CONFIG`，或使用子命令（例如 invite）：啟動器只把參數原樣傳給 server，不產生設定檔。行為與直接執行 server 執行檔相同。
- 你的平台沒有對應的子套件：啟動器以可讀錯誤點名不支援的平台。

## 用 Docker 或 compose 啟動

Docker 映像的行為：

- 以非 root 使用者執行單一 server 執行檔。
- `/data` 是資料 volume 的掛載點。
- 健康檢查打 `/healthz`。就緒後 `/healthz` 與 `/readyz` 都回成功。
- 首跑（還沒有 admin）時，容器 logs 含一次性 setup token 那一行。用 `docker logs` 取 token。
- 設定檔錯誤時，容器以非零 exit code 結束，不綁定連接埠。它不會帶著部分預設值上線。

SQLite compose 的行為：

- 單一服務。named volume 持久化 `/data`，裡面是儲存後端與 identity 兩個資料庫檔。
- 對外網址與連接埠映射經環境變數或 compose 設定。
- compose up 後，從 logs 取 setup token，再用瀏覽器完成 `/setup`。
- 容器重啟後資料存留，`/setup` 維持關閉。
- 只能跑單一 instance。compose 檔以註解明示：不得 scale，也不得多個 replica 共用 volume。

PostgreSQL profile 的行為：

- postgres 帶健康檢查，server 等它健康後才啟動。
- 密碼只經環境變數注入。版本控制內的 compose 與範例檔沒有明文密碼。`.env` 以樣板檔提供，實際的 `.env` 不入版本控制。
- identity 資料庫仍是 `/data` 下的 SQLite 檔。

Docker 部署時，backup、restore、verify-backup 與 invite 子命令在容器內執行。子命令的行為見 [備份與還原](server-backup.md) 與 [帳號](accounts.md)。

## 重啟、重置與升級

- 一般重啟不重印 setup token。資料與桌面 app 的 remote 分頁都保留。
- 完全重置：npx 路徑刪除資料目錄；在 checkout 內開發的人改跑 `npm run dev:reset`。重置後重新出現全新的 setup token。
- 升級步驟：規格只要求部署文件涵蓋升級步驟，未載步驟內容。

## 三種網址要分清楚

- Server base URL：server 本身的位址，例如 `http://localhost:8080`。
- 瀏覽器帳號與管理頁 URL：`/login`、`/account`、`/admin` 這些頁面。
- project-scoped API URL：CLI 連接時用的專案位址，例如 `http://localhost:8080/api/speclink/v1/projects/demo`。

CLI 連接用第三種。詳見 [CLI 連接 remote](cli-remote.md)。

**出處**：`server-release`、`user-documentation`
