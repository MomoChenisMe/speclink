# server-release Specification

## Purpose

TBD - created by archiving change 'server-release-packaging'. Update Purpose after archive.

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

release 工作流 SHALL 於既有矩陣為 server binary 加列（各平台打包附 checksums），並 SHALL 建置與發布版本對齊 tag 的 Docker image。SHALL 有部署文件涵蓋：四種形態啟動方式、setup token 取得、單一 instance 限制、環境變數清單、容器內 backup/restore/verify-backup 與 invite 子命令操作、升級步驟。

#### Scenario: release 定義完備

- **WHEN** 檢視 release 工作流定義與部署文件
- **THEN** 矩陣含 server binary 打包與 checksums；含映像建置發布 job；文件四形態與子命令操作齊備

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