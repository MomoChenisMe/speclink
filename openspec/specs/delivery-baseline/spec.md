# delivery-baseline Specification

## Purpose

專案交付與驗證基線：Node 套件安裝的確定性、root 一道指令跑完的全量驗證、CI 必跑的完整測試面，以及 Node native 套件的全平台交付驗證。本 capability 保證任何人在乾淨環境下都能重現同一組綠燈——含桌面測試套件全綠與測試輸出零 React act 警告。

## Requirements

### Requirement: Node 套件安裝確定性

`crates/adapters/speclink-node` SHALL 在乾淨環境下以 npm ci 成功安裝依賴：package.json 與 package-lock.json 保持同步，且 committed 的 package.json SHALL NOT 宣告未發佈至 npm registry 的套件依賴。

#### Scenario: 乾淨環境 npm ci 成功

- **WHEN** 在乾淨 checkout（無 node_modules）的 crates/adapters/speclink-node 目錄執行 npm ci
- **THEN** 指令以 exit code 0 完成，且後續 napi build 與 vitest 測試可正常執行

#### Scenario: lock file 與 package.json 同步

- **WHEN** package.json 的依賴宣告有任何變更
- **THEN** package-lock.json 同步更新，npm ci 不出現 Missing from lock file 錯誤

---
### Requirement: root 單一指令全量驗證

repo root SHALL 提供單一指令，依序執行 `packages/ui`、`apps/desktop`、`apps/server-web`、Rust workspace 與 `crates/adapters/speclink-node` 五個測試面的測試；`apps/server-web` 測試後 SHALL 執行 production build，使後續 Rust asset integration 以本次 source 產生的 index 與 manifest 驗證。任一測試面或 Web production build 失敗時，指令 SHALL 以非零 exit code 中止。

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

主 CI SHALL 在三個作業系統（Windows、macOS、Linux）上執行 `scripts` 測試面、`packages/ui`、`apps/desktop`、`apps/server-web` 測試、Server Web production build 與 `cargo test --workspace`，而非僅 build 與 smoke；測試與 Web build 步驟 SHALL NOT 設定 `continue-on-error`。`scripts` 測試面守的是設定檔本身的契約（workflow 步驟順序、release 產物組裝、簽章閘門），SHALL 排在其他測試面之前，且其執行方式 SHALL 相容於本 workflow 釘選的 Node 版本與三平台的預設 shell——glob 於傳入 Node 之前完成展開。Rust 測試 SHALL 在 Web production build 後執行；既有 Node SDK 與其他 delivery gate SHALL 保持啟用。

#### Scenario: 測試失敗使 CI 紅燈

- **WHEN** 任一平台上任一 `scripts` 測試、React workspace 測試、Web production build 或 Rust 測試失敗
- **THEN** 該 CI workflow 以失敗狀態結束，不得標記為允許失敗

#### Scenario: push 觸發完整測試

- **WHEN** push 或 pull request 觸發主 CI
- **THEN** workflow 依序完成 `scripts` 測試面、三個 React workspace 測試、Server Web production build 與 Rust workspace 測試，全數通過才回報成功

#### Scenario: scripts 測試面在釘選 Node 版本上實際執行

- **WHEN** 於本 workflow 釘選的 Node 版本與三平台預設環境執行 `scripts` 測試步驟
- **THEN** 該步驟實際載入並執行全部 `scripts` 測試檔，不因 glob 未展開而以「找不到檔案」結束


<!-- @trace
source: release-signing-and-channels
updated: 2026-08-14
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
- **THEN** 串接鏈不於 apps/desktop 步驟中止，可續行至 crates/adapters/speclink-node 步驟

---
### Requirement: 倉庫路徑守門

scripts 測試面 SHALL 含一條靜態斷言，遞迴走訪 repo 內的文字檔（排除 .git、node_modules、target、dist、`.speclink`、`.dev`、`openspec/changes`、`openspec/discussions` 與 CHANGELOG.md，並跳過 `<!-- @trace` 到 `-->` 之間的行、以及帶 `path-guard:allow` 標記的行），確認不殘留搬移前的舊形路徑：(a) `crates/speclink-<name>` 形式的 crate 路徑（現行正典為 `crates/<group>/speclink-<name>`，group ∈ engine／host／store／protocol／adapters）；(b) 已搬入 `scripts/<group>/` 的腳本以 `scripts/<檔名>` 直接引用的形式（group ∈ release／npm／desktop／docs／dev）；(c) 已搬入 `crates/engine/speclink-core/src/<group>/` 的引擎模組以 `crates/engine/speclink-core/src/<模組>.rs` 直接引用的形式（group ∈ lifecycle／quality／workspace；模組清單為 lifecycle 的 model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace，quality 的 station、validate、analyzer、drift，workspace 的 init、skills、instructions、config、schema、workspace；根層的 lib、store、util、keylines、testkit、teststore、demo 與 command 目錄不在清單）；(d) 已搬入 `crates/host/speclink-server/src/<group>/` 的伺服器模組以 `crates/host/speclink-server/src/<模組>.rs` 直接引用的形式（group ∈ identity／admin／api／web；模組清單為 identity 的 identity_sqlite（現為 identity/sqlite.rs）、auth、device、setup，admin 的 audit、backup，api 的 routes、read_api、verb、events、context、error，web 的 assets；根層的 main、lib、app、config、state 與四個主模組檔 identity、admin、web、api 不在清單），以及已搬入 `crates/host/speclink-server/tests/it/<group>/` 的測試檔以 `crates/host/speclink-server/tests/it/<檔名>.rs` 直接引用的形式（檔名清單為搬移前的 44 個檔名，含 admin_*、web_* 去前綴前的舊名；根層的 main、e2e_cli、phase2_chain、startup、serverfs_store 與 common 目錄不在清單）。`scripts/install.sh`、`scripts/install.ps1` 與 `scripts/install.test.mjs` SHALL 維持於 `scripts/` 根且 SHALL NOT 被此斷言視為舊形——README 與 getting-started 的 raw GitHub 安裝網址逐字不變。`.speclink` 與 `.dev`（本機執行與開發時產生的狀態，非版本控管內容）與 `openspec/changes`（進行中提案逐字引用搬移前路徑）SHALL 一併排除。逐字引用舊形路徑當反例的文件行 SHALL 以 `path-guard:allow` 標記豁免，其餘行 SHALL NOT 豁免。命中時 SHALL 以非零 exit 失敗並列出檔案與行號；零命中時通過。本斷言由 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 執行，主 CI 的 scripts 測試步驟 SHALL 以同一組樣式呼叫（bash 展開，兩個明確樣式，不依賴 globstar）。 <!-- path-guard:allow -->

#### Scenario: 殘留舊形 crate 路徑被抓出

- **WHEN** repo 內任一文字檔（非排除範圍、非 @trace 註解區塊、未帶豁免標記）含字串 crates/speclink-core/src/init.rs，執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs <!-- path-guard:allow -->
- **THEN** 該測試失敗、非零 exit code，輸出列出該檔案與行號

#### Scenario: 殘留舊形引擎模組路徑被抓出

- **WHEN** repo 內任一文字檔（非排除範圍、非 @trace 註解區塊、未帶豁免標記）含字串 crates/engine/speclink-core/src/init.rs（init 已搬入 workspace/），執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs <!-- path-guard:allow -->
- **THEN** 該測試失敗、非零 exit code，輸出列出該檔案與行號；同一檔案引用根層模組 crates/engine/speclink-core/src/util.rs 的行不算命中

#### Scenario: 殘留舊形伺服器模組或測試檔路徑被抓出

- **WHEN** repo 內任一文字檔（非排除範圍、非 @trace 註解區塊、未帶豁免標記）含字串 crates/host/speclink-server/src/identity_sqlite.rs 或 crates/host/speclink-server/tests/it/admin_e2e.rs，執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs <!-- path-guard:allow -->
- **THEN** 該測試失敗、非零 exit code，輸出列出該檔案與行號；同一檔案引用根層 crates/host/speclink-server/src/app.rs 或 crates/host/speclink-server/tests/it/e2e_cli.rs 的行不算命中

#### Scenario: 安裝腳本的根路徑不算舊形

- **WHEN** README.md 含 raw GitHub 網址 https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh，且 repo 內無其他舊形路徑，執行同一指令
- **THEN** 該測試通過，exit code 0

#### Scenario: 歷史記錄與 @trace 區塊不受檢

- **WHEN** openspec/changes 底下的提案、openspec/discussions 的記錄、`.speclink` 與 `.dev` 的本機產生狀態、或正式規格 `<!-- @trace` 區塊內的檔案清單含 crates/speclink-cli/src/commands.rs，且非排除範圍內零命中 <!-- path-guard:allow -->
- **THEN** 該測試通過，exit code 0

#### Scenario: 反例文件以豁免標記排除自身命中

- **WHEN** openspec/specs/delivery-baseline/spec.md 的某行逐字引用 crates/speclink-core/src/init.rs 當反例，且該行帶 `path-guard:allow` 標記，且 repo 內無其他舊形路徑，執行同一指令
- **THEN** 該測試通過，exit code 0；同一檔案內未帶標記的行仍受檢

#### Scenario: 主 CI 以兩個明確樣式跑 scripts 測試面

- **WHEN** 檢視 .github/workflows/ci.yml 的 scripts 測試步驟
- **THEN** 步驟指令為 node --test scripts/*.test.mjs scripts/*/*.test.mjs 且該步驟宣告 shell: bash；delivery-gate 測試以此字面斷言


<!-- @trace
source: server-module-groups
updated: 2026-09-11T09:37:30+08:00
-->