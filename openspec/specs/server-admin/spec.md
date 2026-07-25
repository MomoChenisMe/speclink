# server-admin Specification

## Purpose

TBD - created by archiving change 'server-admin-audit'. Update Purpose after archive.

## Requirements

### Requirement: admin 門禁前置且非 admin 一律 403

admin browser route 與 `/api/speclink/v1/web/admin/*` SHALL 在 active session 認證成功後檢查使用者的 admin 旗標；未登入 browser API SHALL 回 401，已登入但非 admin SHALL 回 403 `permission_denied`，SHALL NOT 新增 wire reason。全部 browser mutation SHALL 在 session 與 admin 檢查前驗證 Origin 或 Referer 同源；既有 bearer admin API SHALL 繼續套用 API version、bearer 與 admin 檢查，SHALL NOT 接受 session cookie 取代 bearer。被停權的 admin SHALL 在下一請求即失去管理面通行。

#### Scenario: 一般成員不可入管理面

- **WHEN** 無 admin 旗標的登入使用者訪問 `/admin` 並呼叫 browser admin API，另以其 PAT 呼叫 bearer admin API
- **THEN** SPA 呈現無權限狀態，兩個 API 皆回 403 `permission_denied`，且不執行任何管理動作

#### Scenario: 停權 admin 即時失效

- **WHEN** admin A 停權 admin B 後，B 以既有 session 呼叫 browser admin API
- **THEN** B 被視同未授權並收到 401；不能讀取或執行管理動作

#### Scenario: 跨 origin mutation 在權限裁決前拒絕

- **WHEN** 已登入 admin 從不同 origin 提交 browser admin mutation
- **THEN** Server 回 403 且不執行管理動作、不新增成功 audit event


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
### Requirement: 管理動作三入口同一實作且功能完備

每個管理動作 SHALL 為單點實作，既有 bearer admin API、SPA 使用的 browser admin API 與 server CLI 子命令 SHALL 呼叫同一路徑。功能集 SHALL 涵蓋：使用者列表與邀請、停權／復權、membership 與 admin 旗標調整、registry 的 project／repo 建立與顯示名變更（key SHALL NOT 可改）、全站憑證 metadata 檢視與強制撤銷。headless 部署 SHALL 能以 CLI 子命令完成停權／復權、token 撤銷與 registry 建立。停權最後一位 active admin SHALL 被拒絕並明示原因。管理 SPA SHALL NOT 提供任何規格內容（changes、specs、discussions）的檢視或編輯。

#### Scenario: 三入口等效停權

- **WHEN** 分別經 bearer admin API、browser admin API 與 CLI 子命令停權三個不同使用者
- **THEN** 三者的下一個 API 請求皆 401；三筆動作皆入 audit，來源分別為 `api`、`web`、`cli`

#### Scenario: 最後一位 admin 不可自斷

- **WHEN** 全站僅剩一位 active admin 時經任一入口嘗試停權該 admin
- **THEN** 動作被拒絕且原因明示；該 admin 仍可通行

#### Scenario: registry key 不可改

- **WHEN** 管理員在 SPA 嘗試變更既有 project 的顯示名與 key
- **THEN** 顯示名可變更；UI 與 browser API 均無 key 變更操作，binding 以原 key 照常運作


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
### Requirement: 憑證監督不可讀回明文

admin 的憑證檢視 SHALL 僅含 metadata：所屬使用者、prefix、名稱、到期、last-used 與建立時間；SHALL NOT 存在讀回 PAT、access token、refresh credential 明文或 hash 的介面。強制撤銷 SHALL 與自助撤銷同一即時生效語意，並記 audit（含操作者與 token 識別，SHALL NOT 記祕密值）。

#### Scenario: 強制撤銷即時且留痕

- **WHEN** admin 於憑證頁強制撤銷某成員的 PAT
- **THEN** 該 PAT 的下一次使用回 401；audit 含一筆 token-revoked 記錄，記 token id 與 prefix 而無 hash 或明文

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: audit log 只增不改且動作全覆蓋

identity 資料庫 SHALL 含 audit 表（schema 演進沿用既有守門），每筆記錄 SHALL 含操作者、封閉集合的動作種類、對象識別、UTC 時間與來源（web、api、cli）。全部變更型管理動作（含 invite 子命令與 setup 流程的建立動作）SHALL 恰寫一筆 audit，與資料變更同 transaction；SHALL NOT 存在更新或刪除 audit 記錄的介面。/admin 的 audit 檢視 SHALL 唯讀倒序；一般使用者 SHALL NOT 可見。

#### Scenario: 管理動作皆留痕

- **WHEN** 依序執行邀請、membership 調整、project 建立、token 撤銷各一筆後開啟 audit 頁
- **THEN** 四筆記錄倒序在列，動作種類、對象與來源正確；無任何編輯或刪除控制

#### Scenario: audit 與動作同生死

- **WHEN** 某管理動作因資料層錯誤失敗
- **THEN** audit 無該動作的記錄——不存在「動作成功無 audit」或「audit 存在動作未生效」的組合

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: 系統資訊唯讀聚合

admin 的系統狀態檢視 SHALL 唯讀聚合：engine 與 API 版本、store manifest（driver、contract version、capabilities、等級）、store health 即時結果、identity schema version、每個 registry scope 的 outbox 積壓量。store 失聯時 SHALL 如實顯示 health 失敗，SHALL NOT 使頁面整體失效；identity 庫的管理功能照常。

#### Scenario: store 失聯不癱管理面

- **WHEN** store 後端不可用時開啟系統狀態頁並執行一筆使用者停權
- **THEN** 頁面顯示 store health 失敗與可得的其餘資訊；停權照常成功且入 audit

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: 管理 browser API 提供最小且完整的頁面 view model

`/api/speclink/v1/web/admin` SHALL 提供總覽、users、registry、credentials、data、system 與 audit 的獨立讀取操作。Overview SHALL 回 active／suspended user 數、project／repo 數、active credential 數、store health、identity schema version 與 setup welcome connection metadata；清單 SHALL 回穩定 id、顯示欄位與 action eligibility。回應 SHALL NOT 包含 PAT hash、PAT plaintext、password hash、refresh credential、setup token 或 invite token。Store health 失敗時，overview、system 與 data SHALL 回傳仍可取得的 identity 資料、`storeHealthy: false` 與可公開的 `storeHealthError`；users 與 credentials 管理 SHALL 保持可用。

#### Scenario: 管理導覽各頁獨立載入

- **WHEN** admin 依序開啟 users、registry、credentials、data、system 與 audit route
- **THEN** 每個 route 只呼叫對應 view-model API 並呈現頁面所需欄位，不取得祕密值

#### Scenario: Store 不健康時 identity 管理仍可用

- **WHEN** TeamStore health check 失敗但 identity store 可讀
- **THEN** overview 明確顯示 `storeHealthy: false`，users 與 credentials API 仍成功，data 與 system 呈現可得資料與可公開錯誤

#### Scenario: 清單回傳 action eligibility

- **WHEN** admin 讀取包含最後一位 active admin 的 users view model
- **THEN** 該使用者項目明確標示不可停權或移除 admin 旗標，且 server mutation 仍獨立執行相同安全檢查

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