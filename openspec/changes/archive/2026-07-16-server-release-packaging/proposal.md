## Why

架構 §13.4 給官方 server 開的最低配備清單十二項（setup/admin UI、PAT/身分、registry、binding handshake、CAS、交易、immutable revisions、migration、backup/restore、Polling/ETag、SSE、docker-compose），前十一項已隨 Phase 2 各刀落地，唯獨發布形態還是零：沒有 Dockerfile、沒有 compose、release 工作流只發 CLI 不含 server binary——「docker compose up -d → 開 /setup」的開箱承諾（§13.4 開箱流程第一行）目前走不通。§13.1 明訂四種官方發布形態：native binary、Docker image、SQLite one-container compose、PostgreSQL compose profile；§14 Phase 2 第 4 項即本刀。

目標使用者：自架 server 的小型團隊運維者（一行 compose 起服務）與發布維護者（release 工作流一次產出全部形態）。

## What Changes

- 新增 speclink-server 的 Dockerfile：多階段建置（Rust 編譯層 → 最小執行層），單一靜態 binary、非 root 使用者執行、資料目錄為 volume 掛載點、預設組態經環境變數指向；映像含 /healthz 可用的 HEALTHCHECK。
- 新增 SQLite one-container compose：單服務、named volume 持久化 store 與 identity 兩個資料庫檔、埠映射與 public url 環境變數——docker compose up -d 後開瀏覽器走 /setup 即完成開箱（首次啟動的 setup token 經 docker compose logs 取得，文件明示）。
- 新增 PostgreSQL compose profile：server ＋ postgres 兩服務，密碼經環境變數注入（不落 compose 檔明文——沿用 §13.2 secret 紀律與既有的環境變數補全機制）、postgres 服務帶 healthcheck、server 依賴其就緒。
- release 工作流納入 speclink-server：native binary 隨既有 release 矩陣建置打包（含 checksums），Docker image 建置與驗證進 CI（對 image 起容器打 /healthz 與 /readyz 冒煙）。
- 部署文件：四種形態的啟動方式、單一 instance 限制（SQLite/FS profile 僅允許一個 server instance——§13.1）、backup/restore 子命令在容器內的操作路徑、setup token 取得方式。

## Capabilities

### New Capabilities

- `server-release`: 發布形態的行為保證——映像可起且健康檢查可用、SQLite compose 開箱到 /setup 可走、PostgreSQL profile 的 secret 紀律、release 產物含 server binary、單一 instance 語意文件化。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增基建檔案與 CI 工作流段；不動任何 Rust 程式碼與現有行為；parity 31 項、color 16 項、twin 8 情境凍結不動。與 server-drift-api、phase2-e2e-chain 兩刀無檔案交集，可平行。
- Affected specs: `server-release`（新增）
- Affected code:
  - New: crates/speclink-server/Dockerfile、deploy/docker-compose.yml、deploy/docker-compose.postgres.yml、docs/server-deployment.zh-TW.md
  - Modified: .github/workflows/release.yml、.github/workflows/ci.yml
  - Removed: 無
