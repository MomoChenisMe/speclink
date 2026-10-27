# server-release Specification

## Purpose

Speclink server 的交付與部署形式：Docker 映像可起且帶健康檢查、SQLite compose 一行開箱、PostgreSQL profile 的 secret 紀律，以及 release 產物涵蓋 server 與部署文件。本 capability 保證交付物內嵌的 SPA 資產與 server 同版本，不會出現前後端版本錯開的部署。

## Requirements

### Requirement: Docker 映像可起且健康檢查可用

server SHALL 有多階段 Dockerfile：執行層為最小基底、非 root 使用者、單一 server binary 為 ENTRYPOINT、/data 為資料 volume 掛載點、HEALTHCHECK 打 /healthz。以 SQLite 組態起容器 SHALL 於就緒後 /healthz 與 /readyz 皆回 2xx，且首跑（無 admin）時容器 logs SHALL 含一次性 setup token 行。組態錯誤 SHALL 使容器以非零 exit code 結束（沿用啟動 fail closed），SHALL NOT 以部分預設服務。CI SHALL 含映像建置與上述冒煙驗證。

#### Scenario: 映像冒煙

- **WHEN** CI build 映像並以 SQLite 組態起容器
- **THEN** /healthz 與 /readyz 於就緒後回 2xx；logs 含 setup token 行；停容器後 job 綠

#### Scenario: 壞組態容器即死

- **WHEN** 以 YAML 不可解析的組態起容器
- **THEN** 容器以非零 exit code 結束；不綁定連接埠

---

<!-- @trace
source: server-release-packaging
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
-->

---
### Requirement: SQLite compose 一行開箱

SHALL 提供單服務的 SQLite docker compose：named volume 持久化 /data（store 與 identity 資料庫檔）、public url 與埠映射經環境變數/compose 設定。compose up 後 SHALL 能經 logs 取得 setup token 並於瀏覽器完成 /setup 開箱；容器重啟後資料 SHALL 存留。compose 檔 SHALL 以註解明示單一 instance 限制（不得 scale 或多 replica 共用 volume）。compose 檔 SHALL 通過語法驗證並納入 CI。

#### Scenario: 開箱到可連線

- **WHEN** compose up 後取 setup token 完成 /setup、invite 成員並以 CLI 連線執行動詞
- **THEN** 全流程可走；compose restart 後既有資料完整、/setup 維持關閉

---

<!-- @trace
source: server-release-packaging
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
-->

---
### Requirement: PostgreSQL profile 的 secret 紀律

SHALL 提供 server ＋ postgres 兩服務的 compose profile：postgres 帶 healthcheck、server 依賴其 healthy 才啟動；密碼 SHALL 僅經環境變數注入（版本控制內的 compose 與範例檔 SHALL NOT 含明文密碼，.env 範例以樣板檔提供且實際 .env 不入版本控制）；identity 資料庫維持 /data 下的 SQLite 檔。

#### Scenario: server 等資料庫就緒

- **WHEN** 以 PostgreSQL profile compose up
- **THEN** server 於 postgres healthcheck 通過後才啟動並就緒；版本控制內無任何明文密碼

---

<!-- @trace
source: server-release-packaging
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
-->

---
### Requirement: release 產物含 server 與部署文件

release 工作流 SHALL 於既有矩陣建置各平台 server binary 並執行無前端 dist 環境的冒煙驗證（/login 回 HTML、未知 browser API 回 JSON 404）作為發布品質閘門，但 SHALL NOT 將 server binary 打包上傳至 GitHub Release assets——server 的官方發布通路 SHALL 為版本對齊 tag 的 Docker image 與 npm 套件（見「npm 套件一行啟動 server」），並 SHALL 建置與發布該映像。SHALL 有部署文件涵蓋：npx 快速啟動、Docker 直跑、SQLite compose、PostgreSQL compose 四種官方形態與從原始碼建置 binary 的替代路徑、setup token 取得、單一 instance 限制、環境變數清單、容器內 backup/restore/verify-backup 與 invite 子命令操作、升級步驟；文件 SHALL NOT 指向 Release 的 server 壓縮檔下載。

#### Scenario: release 定義完備

- **WHEN** 檢視 release 工作流定義與部署文件
- **THEN** 工作流含各平台 server 建置與無 dist 冒煙步驟、映像建置發布 job，打包與上傳步驟不含 server 壓縮檔；文件涵蓋四種官方形態、原始碼建置替代路徑與子命令操作，無 Release 壓縮檔下載指示


<!-- @trace
source: release-signing-and-channels
updated: 2026-08-14
-->

---
### Requirement: Server 交付物內嵌同版本 SPA 資產

Release binary、Docker image 與本機 production build SHALL 在編譯期內嵌 `apps/server-web` 同一次 source revision 產生的 Vite `index.html`、manifest、hashed JavaScript／CSS、字型與圖示。建置順序 SHALL 為安裝 lockfile 固定的 npm dependencies、執行 `apps/server-web` production build、再編譯 `speclink-server`；缺少 index 或 manifest 時 server release build SHALL 以非零 exit code 失敗並指出需先完成 Web workspace build，SHALL NOT 產生只有 API 而沒有 UI 的成功 artifact。Runtime SHALL 只需要 non-root server binary，SHALL NOT 需要 Node、外部 `dist` volume、CDN 或第二個 Web service。

#### Scenario: Release binary 在空 runtime 載入 SPA

- **WHEN** 將 release server binary 放入沒有 Node 與 `apps/server-web/dist` 的 runtime，啟動後 GET `/login` 與 HTML 引用的 hashed assets
- **THEN** index 與全部資產成功回應、版本來自同一 binary，且未知 `/api/speclink/v1/web/missing` 回 JSON 404

#### Scenario: 缺少 Web build 使 release build 失敗

- **WHEN** production index 或 manifest 不存在時執行 server release build
- **THEN** build 以非零 exit code 結束並輸出先建置 `apps/server-web` 的可執行提示，不產生可發布 server artifact

#### Scenario: Docker multi-stage 不攜帶 Node runtime

- **WHEN** Docker workflow 依 lockfile 建 Web assets、編譯 server 並檢視最終 image
- **THEN** 最終 image 只有執行 server 所需檔案與 non-root 使用者，沒有 Node runtime 或獨立靜態檔服務

#### Scenario: Release workflow 保持資產與版本對齊

- **WHEN** tag 觸發 server binary 與 Docker image 發布
- **THEN** 每個 server artifact 都先通過同 revision 的 Web test 與 production build，並在無外部 assets 的 smoke test 載入 `/login`

<!-- @trace
source: web-service-navigation-redesign
updated: 2026-07-25
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/src/index.css
  - apps/server-web/index.html
  - apps/server-web/package.json
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/activate.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/build.test.ts
  - apps/server-web/src/__tests__/invite.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/app/context.tsx
  - apps/server-web/src/assets/logo-mark.png
  - apps/server-web/src/assets/speclink-wordmark.png
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/Field.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Wordmark.tsx
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/FocusLayout.tsx
  - apps/server-web/src/lib/formError.ts
  - apps/server-web/src/lib/returnTo.ts
  - apps/server-web/src/lib/useAsync.ts
  - apps/server-web/src/lib/useFocusMain.ts
  - apps/server-web/src/lib/useMediaQuery.ts
  - apps/server-web/src/main.tsx
  - apps/server-web/src/pages/AccountPage.tsx
  - apps/server-web/src/pages/ActivatePage.tsx
  - apps/server-web/src/pages/InvitePage.tsx
  - apps/server-web/src/pages/LoginPage.tsx
  - apps/server-web/src/pages/SetupPage.tsx
  - apps/server-web/src/pages/admin/AdminSection.tsx
  - apps/server-web/src/pages/admin/AuditPage.tsx
  - apps/server-web/src/pages/admin/CredentialsPage.tsx
  - apps/server-web/src/pages/admin/DataPage.tsx
  - apps/server-web/src/pages/admin/OverviewPage.tsx
  - apps/server-web/src/pages/admin/RegistryPage.tsx
  - apps/server-web/src/pages/admin/SystemPage.tsx
  - apps/server-web/src/pages/admin/UsersPage.tsx
  - apps/server-web/src/pages/admin/states.tsx
  - apps/server-web/src/pages/admin/stubs.tsx
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/src/vite-env.d.ts
  - apps/server-web/tsconfig.json
  - apps/server-web/vite.config.ts
  - apps/server-web/vitest.config.ts
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/Dockerfile
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_data.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/phase2_chain.rs
  - crates/speclink-server/tests/setup_flow.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_assets.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
  - crates/speclink-server/tests/web_session.rs
  - crates/speclink-server/tests/web_setup.rs
  - docs/remote-getting-started.md
  - docs/remote-getting-started.zh-TW.md
  - docs/server-deployment.zh-TW.md
  - package-lock.json
  - package.json
  - packages/ui/src/__tests__/table.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/ui/label.tsx
  - packages/ui/src/components/ui/table.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
  - scripts/delivery-gate.test.mjs
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: npm 套件一行啟動 server

專案 SHALL 提供 server 的 npm 通路：主套件帶 bin launcher 與各平台子套件的 optionalDependencies，每個子套件以 os／cpu 欄位對應平台且內容物為該平台的 speclink-server binary，安裝時只下載符合平台者；套件版本 SHALL 與 release tag 對齊。launcher 於無參數（或僅環境變數）啟動時 SHALL 依環境變數產生組態 YAML 寫入資料目錄後帶 --config 啟動 binary：SPECLINK_STORE 選擇 store driver（sqlite 為預設、serverfs 與 postgres 可選）、SPECLINK_DATA_DIR 指定資料目錄（預設 ./speclink-data）、SPECLINK_PUBLIC_URL 與 SPECLINK_PORT 決定對外位址（public_url 預設連動 port）；SPECLINK_STORE=postgres 而 SPECLINK_POSTGRES_URL 缺席時 SHALL 以非零結束並點名缺項。使用者帶 --config、設 SPECLINK_CONFIG 或使用子命令時 launcher SHALL 純透傳參數與 exit code，SHALL NOT 產生組態。平台無對應子套件時 SHALL 以可讀錯誤點名不支援的平台。發布 SHALL 由 release 管線於 NPM_TOKEN 存在時執行（npm publish --access public），缺席時 SHALL 跳過且不影響 Release 結果。

#### Scenario: 零參數啟動走 sqlite 預設

- **WHEN** 以 npx 執行主套件且未帶參數、未設 SPECLINK_STORE
- **THEN** 資料目錄產生（含組態 YAML 與 sqlite 的 store 與 identity 檔路徑宣告）、server 啟動並於首跑印出 setup token

#### Scenario: postgres 缺連線 URL 即死

- **WHEN** SPECLINK_STORE=postgres 而 SPECLINK_POSTGRES_URL 未設定時啟動
- **THEN** launcher 以非零結束，錯誤訊息點名 SPECLINK_POSTGRES_URL，不啟動 server、不產生組態

#### Scenario: 自帶組態純透傳

- **WHEN** 以 npx 執行主套件並帶 --config 指向既有 YAML（或執行 invite 等子命令）
- **THEN** launcher 將參數原樣傳給 binary、exit code 一致，行為與直接執行 binary 相同，資料目錄不產生新組態

#### Scenario: NPM_TOKEN 缺席跳過發布

- **WHEN** NPM_TOKEN 未設定且 push tag
- **THEN** npm 發布 job 跳過，Release 照常發布，workflow 整體綠

<!-- @trace
source: release-signing-and-channels
updated: 2026-08-14
-->