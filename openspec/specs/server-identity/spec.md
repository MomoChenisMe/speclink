# server-identity Specification

## Purpose

TBD - created by archiving change 'server-identity-pat'. Update Purpose after archive.

## Requirements

### Requirement: 邀請一次性且到期失效

邀請 SHALL 由 server binary 的 invite 子命令於主機上建立（email、顯示名、指派的 project memberships、可選 admin 旗標、到期時限），並輸出一次性 invite URL；對已有 active user 或未過期邀請的 email SHALL 拒絕重複建立。開啟有效 `/invite/:token` SHALL 由 browser API 回傳設定密碼所需的非祕密邀請摘要；同源提交後 SHALL 原子地建立 active user（含指派 memberships）並耗用邀請，接著建立該 user 的 Web session。成功回應 SHALL 回傳由 Server 裁決的 `destination`：admin invitation 為 `/admin`，一般 invitation 為 `/account`。已用、過期或未知的邀請 token SHALL 得到相同「邀請無效」狀態與公開訊息，SHALL NOT 區分原因，且 SHALL NOT 建立 session。

#### Scenario: 邀請走完即建立帳號並登入

- **WHEN** 以 invite 子命令對新 email 建立含一個 project membership 的邀請，開啟 URL 設定密碼並提交
- **THEN** user 建立為 active 且具該 membership，Web session 已設定並導向 `/account`；同一 URL 再開啟得到「邀請無效」

#### Scenario: Admin 邀請進入管理首頁

- **WHEN** 帶 admin 旗標的有效邀請完成密碼設定
- **THEN** user 與 Web session 建立成功，Server 回 `destination: "/admin"`，SPA 進入管理首頁

#### Scenario: 過期邀請不可用

- **WHEN** 開啟已過到期時限的邀請 URL
- **THEN** 回應與已用邀請相同的「邀請無效」狀態；不建立 user 或 session

#### Scenario: 重複 email 拒絕

- **WHEN** 對已有 active user 的 email 執行 invite 子命令
- **THEN** 子命令以非零 exit code 拒絕並說明原因；不建立邀請

#### Scenario: 建立 session 失敗不偽裝成已登入

- **WHEN** user 與邀請交易成功後 Web session 建立失敗
- **THEN** Server 回不含內部細節且指示可重試登入的 500 recovery error，SHALL NOT 回成功 destination


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
### Requirement: 本機密碼登入與 session 安全屬性

一般使用者 SHALL 能經 browser JSON API 以 email 與本機密碼登入取得 session；密碼 SHALL 以 argon2id 儲存。Session cookie SHALL 具 HttpOnly、Secure 與 SameSite=Strict 屬性；全部 browser mutation SHALL 驗證同源，不符 SHALL 回 403。登入失敗 SHALL 回相同狀態與統一錯誤訊息，SHALL NOT 洩漏 email 是否存在。登出 SHALL 撤銷 server 端 session 記錄；被撤銷或過期的 session 後續 browser API 請求 SHALL 回 401。

登入成功 destination SHALL 由 Server 依序裁決：有效 device `userCode`、安全 `returnTo`、角色 home。安全 `returnTo` SHALL 只接受以單一 `/` 開頭、無 scheme 或 authority，且首段為 `/account`、`/activate` 或 `/admin` 的路徑；一般成員的 `/admin` destination SHALL 回 403。未登入訪問受保護 SPA route SHALL 導向 `/login?returnTo=...`，且只保留通過同一白名單的站內路徑。

#### Scenario: 登入失敗訊息不洩漏帳號存在性

- **WHEN** 分別以不存在的 email 與存在但密碼錯誤的 email 提交登入
- **THEN** 兩者的回應狀態與錯誤訊息文字相同，且皆不建立 session

#### Scenario: 登出後 session 立即失效

- **WHEN** 登入後執行登出，再以同一 cookie 請求 account browser API
- **THEN** 請求回 401；server 端該 session 記錄已標記撤銷，SPA 導向登入頁

#### Scenario: 角色 home 由 Server 回傳

- **WHEN** admin 與一般成員各自在沒有 device code 與 `returnTo` 時登入
- **THEN** admin 成功回應的 destination 為 `/admin`，一般成員為 `/account`

#### Scenario: 安全 returnTo 優先於角色 home

- **WHEN** admin 以 `/account` 作為 `returnTo` 完成登入
- **THEN** Server 驗證站內路徑後回 `destination: "/account"`

#### Scenario: 外部 returnTo 被忽略

- **WHEN** 使用者以 `https://evil.example/path` 作為 `returnTo` 完成登入
- **THEN** Server 不回外部目的地，改回該使用者的角色 home


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
### Requirement: PAT 自助生命週期

登入的使用者 SHALL 能在帳號頁建立 PAT（名稱與到期日）與撤銷自己的 PAT。PAT 明文 SHALL 以可辨識 prefix 開頭且只在建立回應顯示一次；儲存 SHALL 僅含 token id、顯示用 prefix、hash、名稱、到期、撤銷時戳與 last-used，任何頁面或介面 SHALL NOT 能讀回明文。帳號頁的 PAT 清單 SHALL 顯示 prefix、名稱、到期與 last-used。撤銷 SHALL 即時生效。

#### Scenario: 明文只出現一次

- **WHEN** 建立 PAT 後重新載入帳號頁
- **THEN** 建立回應含完整明文；重新載入後的清單只含 prefix 與 metadata，無任何途徑再取得明文

#### Scenario: 撤銷即時生效

- **WHEN** 以某 PAT 成功呼叫 API 後於帳號頁撤銷它，再以同一 PAT 呼叫
- **THEN** 撤銷後的呼叫回 401 且 reason 為 permission_denied

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: bearer 驗證逐請求生效且分類明確

API 的 bearer 驗證 SHALL 逐請求對 identity 儲存查驗 PAT：hash 命中、未撤銷、未過期、所屬 user 為 active、且 user 為 URL project 的 member 全數成立才得 actor；成功請求 SHALL 更新該 PAT 的 last-used。停權 user 或移除 membership SHALL 在下一個請求即生效，SHALL NOT 有使失效延後的快取。token 無效、過期、撤銷或 user 停權 SHALL 回 401 permission_denied 且 SHALL NOT 區分原因；token 有效但非該 project 成員 SHALL 回 403 permission_denied。

#### Scenario: 停權即時擋下既有 PAT

- **WHEN** 持有效 PAT 的 user 被標記 suspended 後，以該 PAT 呼叫查詢路由
- **THEN** 回 401 permission_denied；server 未執行任何 engine 動詞

#### Scenario: 非成員回 403

- **WHEN** 以有效 PAT 呼叫 actor 不具 membership 的 project 路由
- **THEN** 回 403 且 reason 為 permission_denied，與 token 無效的 401 可區分

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: identity 儲存獨立且版本守門

identity 資料（users、memberships、invitations、PATs、sessions）SHALL 存於 server 自有的資料庫，與 TeamStore 的資料庫分離，SHALL NOT 寫入 TeamStore driver 的檔案。identity 資料庫 SHALL 記錄 schema version：空資料庫初始化為現行版本；version 較新或 schema 非本 server 所建 SHALL 使啟動失敗並印出原因，SHALL NOT 寫入。密碼、PAT、邀請 token 與 session 識別 SHALL 僅以 hash 落庫，SHALL NOT 出現在 log。

#### Scenario: 陌生 identity 資料庫拒啟動

- **WHEN** 組態的 identity 路徑指向一個由其他應用建立的 SQLite 檔並啟動 server
- **THEN** 啟動失敗、stderr 指出該檔與原因；檔案內容位元不變

#### Scenario: 憑證不落明文

- **WHEN** 完成邀請接受、登入與 PAT 建立後檢視 identity 資料庫內容
- **THEN** 密碼、邀請 token、session 識別與 PAT 欄位皆為 hash 形式，庫內不存在任何憑證明文

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: 帳號 browser API 保持憑證祕密邊界

登入使用者 SHALL 能經 `/api/speclink/v1/web/account` 讀取 user、自己的專案隸屬清單、PAT metadata、Web sessions 與 device families，並建立／撤銷自己的 PAT、經 `POST /logout` 結束目前 Web session、撤銷 device family。專案隸屬清單 SHALL 每項含專案 key、專案顯示名與角色（camelCase 欄位），無任何隸屬時 SHALL 為空陣列；admin 與一般成員 SHALL 得到同一形狀。Web session 清單 SHALL 為唯讀呈現，SHALL NOT 提供逐一撤銷其他 session 的操作。讀取 payload SHALL 僅含呈現與 eligibility 所需 metadata，SHALL NOT 包含 PAT hash、password hash、refresh credential 或可重播的 session secret。PAT 建立回應 SHALL 只在該次 `{data}` 內回傳 plaintext；後續讀取 SHALL 僅回 prefix、名稱、到期、撤銷時戳與 last-used。所有 mutation SHALL 驗證同源與 active session。

#### Scenario: PAT 明文只在建立回應出現

- **WHEN** 使用者經 browser API 建立 PAT，接著重新讀取 account summary
- **THEN** 建立回應包含 plaintext；summary 只含 prefix 與 metadata，沒有途徑再次取得 plaintext

#### Scenario: 撤銷 device family 即時生效

- **WHEN** 使用者從帳號頁撤銷一個仍有 active refresh credential 的 device family
- **THEN** family 內 refresh credential 立即失效，後續 account summary 顯示該 family 已撤銷且不回傳 credential

#### Scenario: Account summary 不外洩其他使用者資料

- **WHEN** 一般成員呼叫 account summary
- **THEN** 回應只含該 session user 自己的 user、專案隸屬、PAT、Web session 與 device family metadata

#### Scenario: summary 回傳自己的專案隸屬

- **WHEN** 隸屬兩個專案（一為 editor、一為 viewer）的成員呼叫 account summary
- **THEN** 回應的隸屬清單恰含兩項，各含專案 key、專案顯示名與角色；無隸屬的使用者得到空陣列


<!-- @trace
source: remote-login-ux-gaps
updated: 2026-07-28
code:
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/components/connectionLogin.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/activate.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/pages/AccountPage.tsx
  - apps/server-web/src/pages/ActivatePage.tsx
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/web_account.rs
-->