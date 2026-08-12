# server-web-console Specification

## Purpose

server 的瀏覽器後台：全部 browser route 由單一 SPA 提供且導覽可被發現、導向遵守伺服器裁決與安全優先序、SPA 資產與 fallback 的安全邊界，以及管理列表的抽屜式建立編輯、搜尋篩選分頁與具引導的空狀態、總覽的待辦與行動入口、首次進入的可略過導覽。本 capability 保證後台在共用設計系統下維持高密度且可存取的體驗、互動狀態一致可恢復、不可變識別欄位唯讀（更名是顯式動作），介面語言支援中文與英文。

## Requirements

### Requirement: 全部 browser route 由單一 SPA 提供可發現導覽

Server SHALL 以 Vite、React 與 TypeScript SPA 提供 `/`、`/setup`、`/invite/:token`、`/login`、`/activate`、`/account`、`/admin` 與 `/admin/*`；每個頁面 SHALL 可由應用內導覽或當前流程中的明確動作到達，且 SHALL 支援直接開啟與重新整理。未登入流程 SHALL 使用專注流程殼；已登入的管理員與一般成員 SHALL 共用同一個依角色裁切的主控台殼。管理殼 SHALL 提供總覽、使用者、專案與儲存庫、憑證、系統、稽核紀錄六個目的地，且 SHALL NOT 提供獨立的資料操作目的地。帳號 SHALL NOT 列為側欄目的地，SHALL 由 header 上顯示當前使用者電子郵件的連結進入，並與登出動作並列。管理員開啟 `/account` 時側欄 SHALL 完整呈現且無任何項目高亮；一般成員開啟 `/account` 時 SHALL NOT 呈現側欄。SPA SHALL NOT 提供 changes、specs 或 discussions 的檢視與編輯。

#### Scenario: 管理員不需手打 URL 走訪管理功能

- **WHEN** 已登入管理員從 `/` 進入服務並依可見導覽走訪全部管理目的地
- **THEN** 管理員可到達六個管理目的地、帳號頁與登出動作，且不需修改瀏覽器網址

#### Scenario: 管理員在帳號頁保有全站導覽

- **WHEN** 已登入管理員由 header 的電子郵件連結進入 `/account`
- **THEN** 側欄六個目的地維持可見且無項目高亮，管理員可直接前往任一管理目的地而不使用瀏覽器上一頁

#### Scenario: 一般成員不顯示管理導覽

- **WHEN** 已登入但無 admin 旗標的成員開啟 `/account`
- **THEN** 頁面顯示帳號、存取金鑰、登入工作階段與裝置資訊，且不呈現側欄與任何管理目的地

#### Scenario: 資料操作目的地已併入系統

- **WHEN** 已登入管理員檢視側欄並開啟系統目的地
- **THEN** 側欄不含資料操作項目，系統頁單頁呈現執行環境、儲存狀態、匯出與資料結構遷移

#### Scenario: 深連結可直接重新整理

- **WHEN** 管理員直接開啟或重新整理 `/admin/audit`
- **THEN** Server 回傳 SPA shell，SPA 完成 session 檢查後呈現稽核紀錄頁


<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 導向遵守伺服器裁決與安全優先序

SPA SHALL 以 browser session API 回傳的 `home` 與 mutation 回傳的 `destination` 導向，SHALL NOT 依客戶端角色自行計算安全敏感目的地。登入成功的導向優先序 SHALL 為有效 device `userCode`、通過白名單的 `returnTo`、角色 home；`returnTo` SHALL 僅接受以單一 `/` 開頭、無 scheme 或 authority，且首段為 `/account`、`/activate` 或 `/admin` 的站內路徑。未登入使用者進入受保護 route 時 SHALL 前往帶安全 `returnTo` 的登入頁；route 切換完成後 focus SHALL 移至 `<main>` 標題。

#### Scenario: 裝置核准優先於一般返回路徑

- **WHEN** 使用者以有效 device `userCode` 與 `/account` 的 `returnTo` 完成登入
- **THEN** Server 回傳 activation destination，SPA 先呈現裝置核准流程

#### Scenario: 外部 returnTo 不形成 open redirect

- **WHEN** 登入請求帶入 `https://evil.example/path` 或 `//evil.example/path` 作為 `returnTo`
- **THEN** Server 忽略該值並回傳角色 home，SPA 不導向外部 origin

#### Scenario: 一般成員不可用 returnTo 進入管理面

- **WHEN** 一般成員以 `/admin` 作為 `returnTo` 完成登入
- **THEN** Server 回 403，SPA 呈現無權限狀態且不降級導向 `/account`


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
### Requirement: SPA 資產與 fallback 具可驗證的安全邊界

Production SPA SHALL 以 compile-time asset embedding 進入 `speclink-server` binary，runtime SHALL NOT 依賴相鄰 `dist`、Node、CDN 或外部字型服務。`/assets/*` SHALL 只回傳 build manifest 內的資產與正確 MIME；內容雜湊資產 SHALL 回 `Cache-Control: public, max-age=31536000, immutable`，SPA shell SHALL 回 `Cache-Control: no-cache` 與 self-only Content Security Policy。SPA fallback SHALL 僅匹配已定義 browser GET route；未知 browser path、asset、`/api/*`、`/auth/*`、health、readiness 與下載 route SHALL NOT 回傳 `index.html`。

#### Scenario: 無外部靜態檔仍可載入 SPA

- **WHEN** 在沒有 Node、`dist` 目錄與外網連線的 runtime 啟動 release binary 並請求 `/login`
- **THEN** 回應載入內嵌 index、hashed JavaScript、CSS、字型與圖示，且所有資產來自相同 origin

#### Scenario: 拼錯 API 不被 SPA fallback 吞掉

- **WHEN** client GET `/api/speclink/v1/web/unknown`
- **THEN** Server 回 JSON 404，回應內容不是 SPA index

#### Scenario: 未知資產不回 shell

- **WHEN** browser GET `/assets/missing.js`
- **THEN** Server 回 404，回應內容不是 SPA index


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
### Requirement: 共用設計系統維持高密度可存取體驗

Server SPA SHALL 復用 `packages/ui` 的 shadcn/ui 原語、共用 semantic theme、Noto Sans TC 與青綠 focus token，SHALL NOT 建立第二套 theme 或 import Desktop 的 Tauri、Zustand store 與 SDD board 元件。介面上的下拉選單 SHALL 使用 `packages/ui` 的 shadcn/ui Select 原語，SHALL NOT 使用未套用 theme 的原生 `select`；同一組工具列內的文字輸入、下拉與日期輸入 SHALL 具有相同高度、內距、圓角與陰影。下拉展開後的選單 SHALL 具有不透明底色，且在抽屜或對話框內開啟時 SHALL 可正常開啟與選取。主控台殼的 header 高度、側欄寬度與主內容內距 SHALL 與 Desktop 應用程式殼採用相同數值。主控台殼 SHALL NOT 整頁捲動——header 與側欄 SHALL 恆常留在可視範圍內，只有主內容區捲動。寬度至少 1024px 時管理殼 SHALL 顯示 icon 加 label 的固定側欄；更窄時 SHALL 以有可見 trigger 的 Sheet 提供相同目的地。每頁 SHALL 只有一個視覺 primary action，手機版資料 SHALL 轉為可換行 row 或 card 且 SHALL NOT 造成整頁水平捲動。

所有互動 SHALL 可由鍵盤完成，並包含 skip link、至少 2px focus ring、連續 heading、icon-only control 的 aria-label、輸入 label／helper text／autocomplete、鄰近欄位且以 `role=alert` 宣告的錯誤、至少 44×44px 互動目標、至少 16px 正文與至少 4.5:1 正常文字對比。動畫 SHALL 限於 150–250ms opacity 或 transform；`prefers-reduced-motion` 啟用時 SHALL 停用動畫。

#### Scenario: 375px 完成主要流程

- **WHEN** 使用者以 375px viewport、200% zoom 與鍵盤操作 login、setup 或 invite 流程
- **THEN** 主要內容與操作保持可見、focus 順序合理、沒有整頁水平捲動，且可完成提交

#### Scenario: 窄螢幕管理導覽使用 Sheet

- **WHEN** 管理員以 768px viewport 開啟 `/admin`
- **THEN** 固定側欄收合、可見 trigger 開啟含六個目的地的 Sheet，關閉後 focus 回到 trigger

#### Scenario: 管理殼版面數值與 Desktop 一致

- **WHEN** 管理員以 1440px viewport 開啟任一管理目的地
- **THEN** header 高度、側欄寬度與主內容內距與 Desktop 應用程式殼相同

#### Scenario: 只有主內容區捲動

- **WHEN** 管理員開啟內容長度超過視窗高度的管理目的地並向下捲動
- **THEN** header 與側欄維持在原位，只有主內容區捲動

#### Scenario: 工具列控件高度一致

- **WHEN** 管理員檢視同時具有關鍵字搜尋與下拉篩選的列表工具列
- **THEN** 搜尋輸入、下拉篩選與日期輸入的高度相同，且下拉展開後的選單套用與其他控件相同的 theme

#### Scenario: 抽屜內的下拉可正常開啟

- **WHEN** 管理員在抽屜內開啟下拉選單並選取一個選項
- **THEN** 選單以不透明底色呈現於抽屜內、選取生效，且畫面不失去回應

#### Scenario: reduced motion 停用轉場

- **WHEN** 作業系統設定 `prefers-reduced-motion: reduce`
- **THEN** SPA 不執行 route、Sheet、toast 或狀態切換動畫，且功能狀態仍完整可辨識


<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: Browser API 互動狀態一致且可恢復

SPA SHALL 只透過 `/api/speclink/v1/web` same-origin JSON API 讀寫 server 資料，所有 raw HTTP 呼叫 SHALL 集中於 typed client。成功 envelope SHALL 為 `{data: T}`；錯誤 envelope SHALL 為 `{error:{code,message,fieldErrors?}}`，欄位 SHALL 使用 camelCase。每個 route SHALL 表示 loading、success、empty、forbidden 與 unexpected error；route chunk 或 render 失敗 SHALL 由 error boundary 顯示重試入口而非白屏。Mutation 期間 SHALL 停用重複提交並顯示進度；成功 SHALL 以 `aria-live=polite` 回饋；失敗 SHALL 保留輸入與原頁資料。停權、撤銷與資料遷移等破壞性操作 SHALL 在送出前以 AlertDialog 顯示確切對象並要求確認。

#### Scenario: 欄位驗證失敗保留輸入

- **WHEN** 使用者提交表單後收到含 `fieldErrors` 的 400 回應
- **THEN** SPA 保留所有非祕密輸入、把錯誤放在對應欄位附近並以 `role=alert` 宣告，且允許修正後重送

#### Scenario: Session 過期回到登入並保留安全路徑

- **WHEN** 已載入的受保護 route 呼叫 browser API 收到 401
- **THEN** SPA 前往 `/login` 並只保留通過白名單的當前 route 作為 `returnTo`

#### Scenario: 破壞性操作阻止重複提交

- **WHEN** 管理員確認撤銷一組憑證且請求仍在進行
- **THEN** 確認按鈕停用並顯示進度，第二次提交不會送出；完成後以可存取訊息回報結果

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
### Requirement: 管理列表以抽屜承載建立與編輯

管理列表頁 SHALL 以列表為主體：表格列與卡片列 SHALL NOT 內嵌文字輸入、下拉選擇或提交按鈕，SHALL 以整列為可點目標開啟該筆資料的細節抽屜。建立、邀請與加入類動作 SHALL 由頁面的單一 primary action 開啟抽屜承載，SHALL NOT 常駐於頁面；此規則 SHALL 一併適用於帳號頁的建立存取金鑰。挑選既有項目加入一筆資料（例如邀請時選擇要加入的專案）SHALL 以下拉選單逐一挑選並列出已選項目，SHALL NOT 為每個候選項各渲染一個常駐勾選框。細節抽屜 SHALL 承載該筆資料的檢視與編輯，並 SHALL 在窄螢幕改為全寬呈現。停權、撤銷憑證、移除成員資格、刪除專案與資料結構遷移 SHALL 在送出前以 AlertDialog 顯示確切對象並要求確認。抽屜內提交失敗 SHALL 保持抽屜開啟、保留非祕密輸入，並把錯誤置於對應欄位附近；提交成功 SHALL 關閉抽屜、更新列表並以 `aria-live=polite` 回饋。只顯示一次的祕密值（存取金鑰明文、邀請）SHALL 附複製動作；邀請 SHALL 以受邀者可直接開啟的連結呈現，SHALL NOT 只呈現裸 token。

#### Scenario: 加入專案以下拉逐一挑選

- **WHEN** 管理員在邀請抽屜為受邀者選擇要加入的專案
- **THEN** 專案以下拉選單挑選，已選的專案列在下方且可逐一移除，未出現逐一列出全部專案的勾選框

#### Scenario: 邀請以可複製的連結回饋

- **WHEN** 管理員送出邀請
- **THEN** 頁面以 `aria-live=polite` 呈現受邀者可直接開啟的邀請連結並提供複製動作

#### Scenario: 帳號頁的建立金鑰不常駐

- **WHEN** 使用者開啟帳號頁
- **THEN** 建立存取金鑰的欄位不在頁面上，按下頁面的建立動作後才於抽屜出現；建立成功後抽屜關閉，明文金鑰以一次性回饋呈現並提供複製動作

#### Scenario: 邀請表單不常駐列表頁

- **WHEN** 管理員開啟使用者頁
- **THEN** 頁面只呈現使用者列表與工具列，邀請欄位在按下邀請動作開啟抽屜後才出現

#### Scenario: 表格列不含輸入控制項

- **WHEN** 管理員檢視使用者列表中任一列
- **THEN** 該列只呈現使用者、狀態、角色、成員資格與建立日期，不含下拉選擇或提交按鈕

#### Scenario: 點整列開啟細節抽屜

- **WHEN** 管理員點擊使用者列表中的一列
- **THEN** 右側抽屜開啟並呈現該使用者的概要、成員資格、憑證與稽核，且成員資格可於抽屜內新增與移除

#### Scenario: 抽屜提交失敗保留輸入

- **WHEN** 管理員在邀請抽屜送出後收到含 `fieldErrors` 的 400 回應
- **THEN** 抽屜保持開啟、電子郵件與顯示名稱維持已輸入內容，錯誤以 `role=alert` 宣告於對應欄位附近

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 管理列表提供搜尋、篩選、分頁與具引導的空狀態

管理列表頁 SHALL 於列表上方提供工具列。使用者頁 SHALL 提供關鍵字搜尋與狀態篩選；專案與儲存庫頁 SHALL 提供關鍵字搜尋；憑證頁 SHALL 以分頁區分存取金鑰與裝置，並提供關鍵字搜尋與狀態篩選；稽核紀錄頁 SHALL 提供關鍵字搜尋、動作篩選、來源篩選、時間區間篩選與分頁控制項，且篩選與分頁結果 SHALL 由伺服器計算。列表為空時 SHALL 呈現說明該資料用途的文字，SHALL NOT 只呈現單行「尚無資料」訊息。該頁的建立入口 SHALL 常駐於頁首，SHALL NOT 只在列表為空時出現。

#### Scenario: 稽核篩選由伺服器計算

- **WHEN** 管理員在稽核紀錄頁選擇動作篩選並切換到第二頁
- **THEN** SPA 以篩選與頁碼參數呼叫 browser API，並只呈現伺服器回傳的當頁事件與總頁數

#### Scenario: 篩選無結果呈現空狀態

- **WHEN** 管理員套用的稽核篩選沒有任何符合事件
- **THEN** 頁面呈現空狀態說明並保留篩選控制項，不呈現空白表格

#### Scenario: 空憑證頁引導建立

- **WHEN** 管理員開啟尚無任何存取金鑰的憑證頁
- **THEN** 頁面說明存取金鑰的用途，且頁首的建立存取金鑰入口可用

#### Scenario: 已有憑證時建立入口仍在

- **WHEN** 管理員在已有存取金鑰的憑證頁尋找新增入口
- **THEN** 頁首仍提供建立存取金鑰的入口，不因清單非空而消失

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 使用者頁呈現尚未接受的邀請

受邀者在接受邀請前沒有使用者資料，管理面 SHALL 於使用者頁另闢區塊列出仍有效（未使用且未過期）的邀請，涵蓋受邀者的電子郵件、顯示名稱、角色、要加入的專案與到期時間。已接受或已過期的邀請 SHALL NOT 出現在該區塊。該區塊 SHALL NOT 與正式使用者列表混列——待啟用者無法被停權、沒有憑證也沒有細節可檢視。沒有任何待啟用邀請時 SHALL NOT 渲染該區塊。view model SHALL NOT 帶出邀請 token 或其 hash。每筆待啟用邀請 SHALL 提供撤回動作，並 SHALL 在送出前以 AlertDialog 指名受邀者要求確認；撤回後該邀請連結 SHALL 立即失效且該筆離開待啟用區塊。已接受的邀請 SHALL NOT 可撤回——其背後已有真實帳號，該走停權。

#### Scenario: 剛送出的邀請立即可見

- **WHEN** 管理員送出邀請後回到使用者頁
- **THEN** 該筆邀請出現在待啟用區塊，標示受邀者、要加入的專案與到期時間

#### Scenario: 接受後離開待啟用區塊

- **WHEN** 受邀者接受邀請並建立帳號
- **THEN** 該筆邀請不再出現在待啟用區塊，受邀者出現在使用者列表

#### Scenario: 撤回邀請使連結立即失效

- **WHEN** 管理員撤回一筆待啟用邀請並於確認框確認
- **THEN** 該筆離開待啟用區塊，受邀者手上的連結不再能建立帳號，且留下一筆指名受邀者的稽核事件

#### Scenario: 沒有待啟用邀請時不留空區塊

- **WHEN** 管理員開啟沒有任何有效邀請的使用者頁
- **THEN** 頁面不渲染待啟用區塊，不留空標題與空清單

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 不可變識別欄位唯讀且更名為顯式動作

專案代號與儲存庫代號建立後不可變更，介面 SHALL 以唯讀文字呈現並標示其不可變更，SHALL NOT 以輸入框樣式呈現。專案名稱與儲存庫名稱 SHALL 預設以唯讀文字呈現，SHALL 於使用者觸發更名動作後才呈現輸入框與確認及取消動作。

#### Scenario: 代號不可編輯

- **WHEN** 管理員在專案抽屜檢視專案代號
- **THEN** 代號以唯讀文字呈現並標示建立後不可變更，畫面上沒有可編輯代號的輸入框

#### Scenario: 更名需先觸發動作

- **WHEN** 管理員開啟專案抽屜並按下更名
- **THEN** 名稱轉為輸入框並提供確認與取消；未按下更名前名稱以唯讀文字呈現

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 總覽提供可行動入口與待辦

總覽頁 SHALL 以指標卡呈現使用者、專案、憑證與待啟用邀請的計數，每張指標卡 SHALL 可點擊並前往對應的管理目的地。總覽 SHALL 呈現系統健康摘要與最近稽核事件，兩者 SHALL 分別提供前往系統目的地與稽核紀錄目的地的入口。總覽 SHALL 在存在待處理事項時呈現待辦區塊，每則待辦 SHALL 附帶處理該事項的動作入口；沒有待處理事項時 SHALL NOT 呈現該區塊。識別資料結構版本 SHALL 呈現於系統健康摘要，SHALL NOT 作為獨立指標卡。

#### Scenario: 指標卡可點入對應頁

- **WHEN** 管理員點擊總覽的使用者指標卡
- **THEN** 前往使用者目的地

#### Scenario: 無待辦時不呈現待辦區塊

- **WHEN** 管理員開啟總覽且系統沒有任何待處理事項
- **THEN** 頁面不呈現待辦區塊，不留下空白標題或空清單

#### Scenario: 待辦附帶處理入口

- **WHEN** 系統沒有任何有效憑證且管理員開啟總覽
- **THEN** 待辦區塊呈現該事項並提供建立存取金鑰的動作入口

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 首次進入提供可略過的分步導覽

管理員首次開啟管理面時，SPA SHALL 自動啟動分步導覽。每一步 SHALL 指向畫面上實際存在的元素並附一句說明其用途，說明卡片 SHALL 依該元素的位置擺放且 SHALL NOT 遮住它，且 SHALL 提供前往下一步、返回上一步與略過整個導覽的動作。導覽走完或被略過後 SHALL 記錄為已檢視，之後開啟管理面 SHALL NOT 再自動啟動；介面 SHALL 保留重新啟動導覽的入口。導覽 SHALL NOT 讀寫任何管理資料，SHALL NOT 阻擋鍵盤離開，且某一步的目標元素不存在時 SHALL 跳過該步而非中斷導覽。

#### Scenario: 首次進入自動啟動

- **WHEN** 管理員在尚未檢視過導覽的瀏覽器開啟管理面總覽
- **THEN** 導覽自動啟動並指向第一個目的地，同時提供下一步與略過

#### Scenario: 略過後不再自動啟動

- **WHEN** 管理員略過導覽後重新整理管理面
- **THEN** 導覽不再自動啟動，且介面提供重新啟動導覽的入口

#### Scenario: 目標元素缺席時跳過該步

- **WHEN** 導覽的某一步指向當前角色或版面下不存在的元素
- **THEN** 導覽跳過該步繼續進行，不呈現空的高亮框也不中斷

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 介面語言支援中文與英文

Server SPA SHALL 以中文（zh-TW）與英文（en）提供全部使用者可見文案，兩種語言的訊息集合 SHALL 完全對應。使用者未明示偏好時 SHALL 依瀏覽器語言決定——以 `zh` 開頭者為中文，其餘為英文。介面 SHALL 於 header 提供語言切換，可選中文、英文或跟隨系統；切換 SHALL 立即生效並在重新載入後維持。語言選擇 SHALL NOT 影響 artifacts 產出語言、CLI 輸出、稽核事件的動作名稱或伺服器回傳的錯誤訊息。

#### Scenario: 未設定偏好時跟隨瀏覽器語言

- **WHEN** 尚未選過語言的使用者以英文瀏覽器開啟管理面
- **THEN** 介面文案為英文

#### Scenario: 切換語言即時生效並持續

- **WHEN** 管理員在 header 將語言切換為英文並重新整理頁面
- **THEN** 管理面文案維持英文，不因重新載入而還原

#### Scenario: 兩種語言的訊息集合對應

- **WHEN** 檢視任一管理目的地的中文與英文版本
- **THEN** 兩者呈現相同的欄位、動作與說明，沒有任何一方缺漏或退回顯示訊息代碼

<!-- @trace
source: admin-console-redesign
updated: 2026-07-26
code:
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - apps/desktop/vitest.setup.ts
  - apps/server-web/src/App.tsx
  - apps/server-web/src/__tests__/a11y.test.tsx
  - apps/server-web/src/__tests__/account.test.tsx
  - apps/server-web/src/__tests__/admin-console-shell.test.tsx
  - apps/server-web/src/__tests__/admin.test.tsx
  - apps/server-web/src/__tests__/app.test.tsx
  - apps/server-web/src/__tests__/audit.test.tsx
  - apps/server-web/src/__tests__/credentials.test.tsx
  - apps/server-web/src/__tests__/helpers/adminHarness.tsx
  - apps/server-web/src/__tests__/i18n.test.tsx
  - apps/server-web/src/__tests__/overview.test.tsx
  - apps/server-web/src/__tests__/reducedMotion.test.ts
  - apps/server-web/src/__tests__/registry.test.tsx
  - apps/server-web/src/__tests__/setup.test.tsx
  - apps/server-web/src/__tests__/system.test.tsx
  - apps/server-web/src/__tests__/toolbar.test.tsx
  - apps/server-web/src/__tests__/tour.test.tsx
  - apps/server-web/src/__tests__/users.test.tsx
  - apps/server-web/src/__tests__/wording.test.tsx
  - apps/server-web/src/api/client.ts
  - apps/server-web/src/components/AdminNav.tsx
  - apps/server-web/src/components/CopyButton.tsx
  - apps/server-web/src/components/DataList.tsx
  - apps/server-web/src/components/DetailSheet.tsx
  - apps/server-web/src/components/EmptyState.tsx
  - apps/server-web/src/components/HeaderAccount.tsx
  - apps/server-web/src/components/ListToolbar.tsx
  - apps/server-web/src/components/LocaleSwitch.tsx
  - apps/server-web/src/components/LogoutButton.tsx
  - apps/server-web/src/components/RouteErrorBoundary.tsx
  - apps/server-web/src/components/SkipLink.tsx
  - apps/server-web/src/components/Tour.tsx
  - apps/server-web/src/i18n/LocaleContext.tsx
  - apps/server-web/src/i18n/messages.ts
  - apps/server-web/src/index.css
  - apps/server-web/src/layouts/AccountLayout.tsx
  - apps/server-web/src/layouts/AdminLayout.tsx
  - apps/server-web/src/layouts/ConsoleLayout.tsx
  - apps/server-web/src/lib/tourSeen.ts
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
  - apps/server-web/src/routes/AppRoutes.tsx
  - apps/server-web/vitest.setup.ts
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/assets.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_audit_filter.rs
  - crates/speclink-server/tests/admin_overview_view.rs
  - crates/speclink-server/tests/admin_system_view.rs
  - crates/speclink-server/tests/admin_users_view.rs
  - crates/speclink-server/tests/admin_web_api.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/select.test.tsx
  - packages/ui/src/__tests__/selectInSheet.test.tsx
  - packages/ui/src/__tests__/theme.test.ts
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/ui/portal-container.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/locale.ts
  - packages/ui/vitest.config.ts
  - packages/ui/vitest.setup.ts
-->

---
### Requirement: 帳號頁呈現我的專案

`/account` SHALL 呈現我的專案區塊，列出目前使用者隸屬的每個專案（顯示名與角色；顯示名缺席時以專案 key 呈現），資料 SHALL 來自 account summary、SHALL NOT 另打管理端點。admin 與一般成員 SHALL 看到同一區塊與同一形狀（admin 不因管理身分而多列非隸屬專案）。無任何隸屬時 SHALL 呈現引導性空狀態（說明由管理員授予隸屬），SHALL NOT 隱藏整個區塊。本區塊 SHALL 為唯讀，SHALL NOT 提供任何隸屬變更操作。

#### Scenario: 成員看到自己的專案與角色

- **WHEN** 隸屬兩個專案的一般成員開啟 /account
- **THEN** 我的專案區塊列出兩個專案的顯示名與各自角色，無任何編輯操作

#### Scenario: admin 看到的是自己的隸屬而非全部專案

- **WHEN** 具 admin 旗標、僅隸屬一個專案的使用者開啟 /account
- **THEN** 我的專案區塊僅列該一個專案；其餘專案不出現（全部專案屬 /admin 的治理視角）

#### Scenario: 無隸屬時的空狀態

- **WHEN** 無任何專案隸屬的使用者開啟 /account
- **THEN** 我的專案區塊呈現空狀態文字，說明隸屬由管理員授予；區塊本身仍可見

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

---
### Requirement: 後台狀態徽章語意色

Console 的狀態徽章 SHALL 依語意上色:正常/啟用/有效為綠系;異常為紅系;停權為琥珀系;已撤銷為中性且與有效態可一眼區辨。祕密揭示橫幅(PAT 建立後的一次性明碼、邀請連結揭示)SHALL 為綠系成功樣式,SHALL NOT 使用主題主色。純 metadata 徽章(成員角色、稽核來源)SHALL 維持中性。同一事實於同頁的橫幅與徽章 SHALL 呈現一致的語意色層級。

#### Scenario: 儲存健康徽章

- **WHEN** 總覽頁與系統頁呈現儲存 online/offline 狀態
- **THEN** 徽章分別以綠系/紅系呈現,與同頁的離線警示橫幅語意一致

#### Scenario: 成員狀態徽章

- **WHEN** 成員清單呈現 active 與停權成員
- **THEN** active 為綠系、停權為琥珀系

#### Scenario: 憑證狀態徽章

- **WHEN** PAT 清單、裝置憑證清單或工作階段清單呈現有效與已撤銷項目
- **THEN** 有效為綠系、已撤銷為中性,兩態可區辨

#### Scenario: 揭示橫幅為成功語意

- **WHEN** 建立 PAT 後顯示一次性明碼、或產生邀請連結
- **THEN** 揭示橫幅以綠系成功樣式呈現,非主題主色

<!-- @trace
source: semantic-color-system
updated: 2026-08-05
-->