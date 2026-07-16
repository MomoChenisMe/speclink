## Context

speclink-server 是單一 Rust binary（axum、rusqlite bundled 靜態連結——sqlite-team-store 刀的決策正是為單 binary 發布鋪的）；啟動需 --config（YAML：store、identity、public url、事件段）與 --addr。首次啟動無 admin 時在 stdout 印一次性 setup token（server-setup-registry 刀）；/healthz 回程序存活、/readyz 回 store health。release.yml 既有矩陣建置 CLI binary 並附 checksums 發 GitHub Release；ci.yml 已有 PostgreSQL service（postgres-team-store 刀）。postgres driver 的密碼可由環境變數補全（SPECLINK_POSTGRES_PASSWORD）。§13.1：SQLite/FS profile 僅允許一個 server instance；§13.2：deployment secret 優先環境變數；§13.4：開箱流程以 docker compose up -d 起點。

## Goals / Non-Goals

**Goals:**

- 四種發布形態齊備且每種都有自動化驗證——「能 build」不算完成，起得來、健康檢查過、setup 可走才算。
- compose 檔是文件的一部分：不懂 Rust 的運維者照 README 一行起服務。
- secret 不落檔：compose 與映像內無任何明文密碼。

**Non-Goals:**

- 不做 Kubernetes manifests、Helm chart（有需求再議；compose 是 §13.1 明訂的交付形態）。
- 不做映像的多架構矩陣最佳化（先 linux/amd64＋linux/arm64 兩架構；更多目標隨需求）。
- 不做自動升級/rolling update 機制（單 instance 定位；升級＝換映像重啟，migration 由既有啟動守門與 admin 觸發承擔）。
- 不做 registry 發布自動化以外的行銷產物（docker hub 描述頁等隨首次正式發版處理）。
- 不動 desktop/CLI 的發布路徑（既有 release 矩陣不變，只是加列）。

## Decisions

### 決策 1：多階段 Dockerfile，distroless 級最小執行層

建置層以官方 Rust 映像編譯 release binary（rusqlite bundled 使執行層零系統依賴）；執行層用最小基底（distroless 或 alpine 擇一，以 TLS 憑證與非 root 使用者需求定案），只放 binary 與 CA bundle。非 root（獨立 uid）執行；/data 為 volume 掛載點（store 與 identity 資料庫檔都在其下）；組態檔經掛載或以環境變數指定路徑；EXPOSE 埠與 HEALTHCHECK 打 /healthz。映像的唯一 ENTRYPOINT 是 server binary——backup/restore/invite 等子命令以 docker run 覆寫參數執行，文件示範。

### 決策 2：SQLite compose 是「一行開箱」的正典示範

deploy/docker-compose.yml 單服務：named volume 掛 /data、環境變數設 public url 與 bind、埠映射。首跑後 docker compose logs 取 setup token → 瀏覽器 /setup。compose 檔內以註解標明單一 instance 限制（不得 scale、不得多 replica 指向同 volume——§13.1）。restart policy 為 unless-stopped。

### 決策 3：PostgreSQL profile 的 secret 只進環境

deploy/docker-compose.postgres.yml：postgres 服務（官方映像、healthcheck pg_isready、named volume）＋ server 服務（depends_on 條件為 healthy；連線 URL 不含密碼，密碼經 SPECLINK_POSTGRES_PASSWORD 與 POSTGRES_PASSWORD 同源環境變數注入——.env 檔示範但 .env 不進版本控制，.env.example 進）。identity 資料庫維持 SQLite 檔於 /data（identity 本就獨立於 TeamStore driver 選擇）。

### 決策 4：CI 驗證「起得來」，release 驗證「發得出」

ci.yml 新增 docker 冒煙 job（Linux）：build 映像 → 起容器（SQLite 組態）→ 輪詢 /healthz 與 /readyz 皆 2xx → 斷言 logs 含 setup token 行 → 停。release.yml 的建置矩陣為 server binary 加列（沿用既有打包與 checksums 模式），並加 Docker image 建置推送 job（tag 對齊 release 版本；推送目標 ghcr，跟隨既有 repo 權限）。compose 檔以 docker compose config 驗證語法進 CI。

### 決策 5：部署文件單一入口

docs/server-deployment.zh-TW.md 涵蓋：四形態啟動（native binary、docker run、兩種 compose）、setup token 取得（logs）、單 instance 限制、環境變數清單（public url、postgres 密碼）、容器內 backup/restore/verify-backup/invite 子命令操作、升級步驟（換映像 → 啟動守門與 migration 行為）。與既有 backup 刀文件互鏈不重複。

## Implementation Contract

- Behavior：運維者 docker compose up -d → logs 取 token → /setup 完成開箱 → invite 成員 → CLI 連線全動詞可用；重啟容器資料存留於 volume；PostgreSQL profile 下 server 等 postgres 健康才啟動。
- Interface / data shape：映像 ENTRYPOINT 為 server binary、HEALTHCHECK 打 /healthz、/data volume；deploy/ 下兩份 compose 與 .env.example；release 產物含各平台 server binary 與 ghcr 映像。
- Failure modes：組態壞或 driver 未知 → 容器以非零 exit code 結束（既有 fail closed）且 compose 呈現失敗；postgres 未就緒 → server 不啟動（depends_on healthy）；健康檢查失敗 → 容器標 unhealthy。
- Acceptance criteria：CI docker 冒煙 job 綠（build、起、healthz/readyz、token 行）；docker compose config 對兩份 compose 通過；release 工作流 dry-run（或 tag 實跑）產出含 server binary 與映像；npm run test:all 全綠零 diff。

## Risks / Trade-offs

- ghcr 推送需要 repo 權限與首次 tag 實跑才能全驗 → CI 冒煙覆蓋「build＋起得來」，推送 job 以 workflow 語法驗證＋首次 release 實證（與 postgres CI job 同一「定義層本地驗、實跑雲端驗」紀律）。
- alpine（musl）vs distroless（glibc）的選擇影響建置目標 → 實作時以 rusqlite bundled 與 ring 等依賴的 musl 相容性實測定案，兩者都符合最小面原則，決策記錄在 Dockerfile 註解。
- compose 檔擋不住使用者硬 scale → 註解與文件明示；單 writer 的檔案鎖（SQLite WAL 單寫者）使誤用大機率顯性報錯而非靜默損毀。

## Migration Plan

純新增基建；與另兩把刀無檔案交集可平行。首次正式發版時 tag 觸發 release 實跑完成端到端驗證。回退即刪除新增檔案與工作流段。

## Open Questions

（無）
