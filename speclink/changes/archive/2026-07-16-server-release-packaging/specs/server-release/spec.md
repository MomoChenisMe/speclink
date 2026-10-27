## ADDED Requirements

### Requirement: Docker 映像可起且健康檢查可用

server SHALL 有多階段 Dockerfile：執行層為最小基底、非 root 使用者、單一 server binary 為 ENTRYPOINT、/data 為資料 volume 掛載點、HEALTHCHECK 打 /healthz。以 SQLite 組態起容器 SHALL 於就緒後 /healthz 與 /readyz 皆回 2xx，且首跑（無 admin）時容器 logs SHALL 含一次性 setup token 行。組態錯誤 SHALL 使容器以非零 exit code 結束（沿用啟動 fail closed），SHALL NOT 以部分預設服務。CI SHALL 含映像建置與上述冒煙驗證。

#### Scenario: 映像冒煙

- **WHEN** CI build 映像並以 SQLite 組態起容器
- **THEN** /healthz 與 /readyz 於就緒後回 2xx；logs 含 setup token 行；停容器後 job 綠

#### Scenario: 壞組態容器即死

- **WHEN** 以 YAML 不可解析的組態起容器
- **THEN** 容器以非零 exit code 結束；不綁定連接埠

---
### Requirement: SQLite compose 一行開箱

SHALL 提供單服務的 SQLite docker compose：named volume 持久化 /data（store 與 identity 資料庫檔）、public url 與埠映射經環境變數/compose 設定。compose up 後 SHALL 能經 logs 取得 setup token 並於瀏覽器完成 /setup 開箱；容器重啟後資料 SHALL 存留。compose 檔 SHALL 以註解明示單一 instance 限制（不得 scale 或多 replica 共用 volume）。compose 檔 SHALL 通過語法驗證並納入 CI。

#### Scenario: 開箱到可連線

- **WHEN** compose up 後取 setup token 完成 /setup、invite 成員並以 CLI 連線執行動詞
- **THEN** 全流程可走；compose restart 後既有資料完整、/setup 維持關閉

---
### Requirement: PostgreSQL profile 的 secret 紀律

SHALL 提供 server ＋ postgres 兩服務的 compose profile：postgres 帶 healthcheck、server 依賴其 healthy 才啟動；密碼 SHALL 僅經環境變數注入（版本控制內的 compose 與範例檔 SHALL NOT 含明文密碼，.env 範例以樣板檔提供且實際 .env 不入版本控制）；identity 資料庫維持 /data 下的 SQLite 檔。

#### Scenario: server 等資料庫就緒

- **WHEN** 以 PostgreSQL profile compose up
- **THEN** server 於 postgres healthcheck 通過後才啟動並就緒；版本控制內無任何明文密碼

---
### Requirement: release 產物含 server 與部署文件

release 工作流 SHALL 於既有矩陣為 server binary 加列（各平台打包附 checksums），並 SHALL 建置與發布版本對齊 tag 的 Docker image。SHALL 有部署文件涵蓋：四種形態啟動方式、setup token 取得、單一 instance 限制、環境變數清單、容器內 backup/restore/verify-backup 與 invite 子命令操作、升級步驟。

#### Scenario: release 定義完備

- **WHEN** 檢視 release 工作流定義與部署文件
- **THEN** 矩陣含 server binary 打包與 checksums；含映像建置發布 job；文件四形態與子命令操作齊備
