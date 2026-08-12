# delivery-baseline Specification

## Purpose

專案交付與驗證基線：Node 套件安裝的確定性、root 一道指令跑完的全量驗證、CI 必跑的完整測試面，以及 Node native 套件的全平台交付驗證。本 capability 保證任何人在乾淨環境下都能重現同一組綠燈——含桌面測試套件全綠與測試輸出零 React act 警告。

## Requirements

### Requirement: Node 套件安裝確定性

`crates/speclink-node` SHALL 在乾淨環境下以 npm ci 成功安裝依賴：package.json 與 package-lock.json 保持同步，且 committed 的 package.json SHALL NOT 宣告未發佈至 npm registry 的套件依賴。

#### Scenario: 乾淨環境 npm ci 成功

- **WHEN** 在乾淨 checkout（無 node_modules）的 crates/speclink-node 目錄執行 npm ci
- **THEN** 指令以 exit code 0 完成，且後續 napi build 與 vitest 測試可正常執行

#### Scenario: lock file 與 package.json 同步

- **WHEN** package.json 的依賴宣告有任何變更
- **THEN** package-lock.json 同步更新，npm ci 不出現 Missing from lock file 錯誤

---
### Requirement: root 單一指令全量驗證

repo root SHALL 提供單一指令，依序執行 `packages/ui`、`apps/desktop`、`apps/server-web`、Rust workspace 與 `crates/speclink-node` 五個測試面的測試；`apps/server-web` 測試後 SHALL 執行 production build，使後續 Rust asset integration 以本次 source 產生的 index 與 manifest 驗證。任一測試面或 Web production build 失敗時，指令 SHALL 以非零 exit code 中止。

#### Scenario: 全部通過

- **WHEN** 五個測試面與 Web production build 皆通過時於 root 執行該指令
- **THEN** 指令依序完成 Web build 與五面驗證並以 exit code 0 結束

#### Scenario: 任一面失敗即中止

- **WHEN** 任一測試面或 Web production build 存在失敗時於 root 執行該指令
- **THEN** 指令於失敗步驟以非零 exit code 中止，不繼續執行後續驗證

#### Scenario: Rust 測試使用當次 Web build

- **WHEN** `apps/server-web` source 有未建置變更並執行 root 全量驗證
- **THEN** 指令先重建 production assets，再執行 Rust workspace 的 embedded asset 與 route tests


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
### Requirement: CI 執行完整測試

主 CI SHALL 在三個作業系統（Windows、macOS、Linux）上執行 `packages/ui`、`apps/desktop`、`apps/server-web` 測試、Server Web production build 與 `cargo test --workspace`，而非僅 build 與 smoke；測試與 Web build 步驟 SHALL NOT 設定 `continue-on-error`。Rust 測試 SHALL 在 Web production build 後執行；既有 Node SDK 與其他 delivery gate SHALL 保持啟用。

#### Scenario: 測試失敗使 CI 紅燈

- **WHEN** 任一平台上任一 React workspace 測試、Web production build 或 Rust 測試失敗
- **THEN** 該 CI workflow 以失敗狀態結束，不得標記為允許失敗

#### Scenario: push 觸發完整測試

- **WHEN** push 或 pull request 觸發主 CI
- **THEN** workflow 依序完成三個 React workspace 測試、Server Web production build與 Rust workspace 測試，全數通過才回報成功


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
### Requirement: Node native 套件全平台交付驗證

Node SDK 的 CI SHALL 在五個宣告平台（x86_64-pc-windows-msvc、x86_64-apple-darwin、aarch64-apple-darwin、x86_64-unknown-linux-gnu、aarch64-unknown-linux-gnu）完成 native module build，且 SHALL 在可原生執行的平台上以測試驗證 build 產物可載入。

#### Scenario: 五平台 build 成功

- **WHEN** 變更觸發 Node SDK workflow
- **THEN** 五個平台的 native module build job 全數成功並上傳 binary artifact

#### Scenario: 可執行平台的載入驗證

- **WHEN** build 完成於可原生執行的平台（win32-x64、darwin-arm64、linux-x64）
- **THEN** 該平台對 build 產物執行測試套件並全數通過

---
### Requirement: 測試輸出無 React act 警告

`packages/ui`、`apps/desktop` 與 `apps/server-web` 的測試輸出 SHALL NOT 含 React `act(...)` 警告（`not wrapped in act` 字樣）；async 狀態更新 SHALL 在測試中被明確等待，而非被壓制或過濾。

#### Scenario: 測試輸出檢查零命中

- **WHEN** 執行 `packages/ui`、`apps/desktop` 與 `apps/server-web` 的完整測試套件並檢查輸出
- **THEN** 輸出不含 `not wrapped in act` 字樣，且所有測試通過

#### Scenario: 禁止以壓制方式清零

- **WHEN** 檢視三個 React workspace 的測試設定與測試檔警告處理方式
- **THEN** 不存在對 act 警告的 console 過濾或全域壓制設定，警告消除一律來自測試側的明確等待


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
### Requirement: 桌面測試套件於乾淨環境全綠

`apps/desktop` 的 vitest 測試套件 SHALL 在乾淨環境下全數通過。其測試執行環境 SHALL 提供瀏覽器 Web Storage 全域（`localStorage` 與 `sessionStorage`），使依賴本機儲存持久化的測試不因環境缺漏而失敗。`apps/desktop` SHALL 直接於自身 package.json 宣告其宣稱使用的測試 DOM 環境（jsdom）為 devDependency，SHALL NOT 僅依賴其他 workspace 的 hoisting。此需求與「root 單一指令全量驗證」互補：後者要求 test:all 於任一面失敗時中止，本需求確保 `apps/desktop` 這一面在乾淨環境下實際通過，使 test:all 為可信的綠燈 gate。測試執行期間 SHALL NOT 有未處理例外（uncaught exception 或 unhandled rejection）使程序非零退出——含在途非同步作業於測試卸載後才觸發者——使 exit 0 為決定性結果而非偶發綠燈。

#### Scenario: 乾淨環境 desktop 測試全綠

- **WHEN** 在乾淨 checkout（無殘留 node_modules 狀態）執行 npm test -w apps/desktop
- **THEN** 指令以 exit code 0 決定性完成（重複執行不偶發非零）、所有測試通過，且測試執行期間無任何未處理例外（含 Web Storage 未定義、以及在途非同步作業於測試卸載後觸發所致的 uncaught 例外）

#### Scenario: 測試環境提供 Web Storage 全域

- **WHEN** desktop vitest 測試存取 localStorage 或 sessionStorage
- **THEN** 兩者皆為可用的 Storage 物件，setItem／getItem／clear 語意正確，且測試檔之間狀態不殘留

#### Scenario: test:all 貫穿 desktop 步驟不中止

- **WHEN** 執行 npm run test:all 且其餘各面測試均通過
- **THEN** 串接鏈不於 apps/desktop 步驟中止，可續行至 crates/speclink-node 步驟