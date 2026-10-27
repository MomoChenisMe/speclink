## 1. Dockerfile 與冒煙

- [x] 1.1 建立 crates/speclink-server/Dockerfile：多階段建置（Rust 編譯層 → 最小執行層＋CA bundle）、非 root 使用者、/data volume、ENTRYPOINT 為 server binary、HEALTHCHECK 打 /healthz；基底（alpine/musl 或 distroless/glibc）以 rusqlite bundled 實測定案並在 Dockerfile 註解記錄。驗收：本機 docker build 成功、docker run（SQLite 組態）後 /healthz 與 /readyz 回 2xx、logs 含 setup token 行。 <!-- speclink-task:tsk_01KXMA7K41VWWRGMJZXBQR8X3C -->
- [x] 1.2 【紅→綠】CI 新增 docker 冒煙 job（Linux，涵蓋「Docker 映像可起且健康檢查可用」兩情境）：build → 起容器 → 輪詢 /healthz、/readyz 皆 2xx → 斷言 logs 含 setup token 行 → 停；另以壞組態起容器斷言非零 exit code。驗收：workflow 定義完成且本機以相同指令序列驗證通過；CI 實跑於 push 後確認。 <!-- speclink-task:tsk_01KXMA7K41ADZ0WBWNQH8AS2BB -->

## 2. compose 兩形態

- [x] 2.1 建立 deploy/docker-compose.yml（SQLite 單服務）：named volume 掛 /data、public url 與埠環境變數、restart unless-stopped、單一 instance 限制註解。驗收：docker compose config 通過；本機 compose up → logs 取 token → /setup 完成開箱 → compose restart 後資料存留且 /setup 關閉（涵蓋「開箱到可連線」情境）。 <!-- speclink-task:tsk_01KXMA7K417PP1ZRZXJE3TSK3H -->
- [x] 2.2 建立 deploy/docker-compose.postgres.yml 與 deploy/.env.example（涵蓋「PostgreSQL profile 的 secret 紀律」）：postgres healthcheck、server depends_on healthy、密碼僅經環境變數、identity 維持 /data 的 SQLite 檔；版本控制內無明文密碼（.env 入 gitignore）。驗收：docker compose config 通過；本機 profile 起服務後 server 於 postgres 就緒後啟動、/readyz 綠。 <!-- speclink-task:tsk_01KXMA7K41RFDE1411FH6QKS8J -->
- [x] 2.3 兩份 compose 的語法驗證（docker compose config）納入 CI（併入 1.2 的 job 或獨立步驟）。驗收：CI 定義含該步驟。 <!-- speclink-task:tsk_01KXMA7K41AGHWQ1GV0TEQCHZE -->

## 3. release 工作流與文件

- [x] 3.1 release.yml 為 speclink-server 加列：既有建置矩陣產出各平台 server binary、沿用打包與 checksums 模式；新增 Docker image 建置與發布 job（tag 對齊版本、目標 ghcr）。驗收：workflow 語法驗證通過、與既有 CLI 產物並存不互擾；實跑於下次 release tag 確認。 <!-- speclink-task:tsk_01KXMA7K41266MF2P287FSH9R6 -->
- [x] 3.2 撰寫 docs/server-deployment.zh-TW.md（涵蓋「release 產物含 server 與部署文件」）：四形態啟動、setup token 取得（logs）、單一 instance 限制、環境變數清單、容器內 backup/restore/verify-backup/invite 子命令操作示範、升級步驟（換映像後啟動守門與 migration 行為）；與既有 backup 文件互鏈。驗收：文件涵蓋清單全項。 <!-- speclink-task:tsk_01KXMA7K41KTETN871M6JMAHAM -->

## 4. 回歸

- [x] 4.1 執行 npm run test:all 確認全 workspace 回歸（本刀無 Rust 程式碼變更，凍結應零 diff）。驗收：全數通過。 <!-- speclink-task:tsk_01KXMA7K41YM01DZS637MYBEMS -->
