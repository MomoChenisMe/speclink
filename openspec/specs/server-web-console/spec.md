# server-web-console Specification

## Purpose

server 的瀏覽器後台：全部 browser route 由單一 SPA 提供且導覽可被發現、導向遵守伺服器裁決與安全優先序、SPA 資產與 fallback 的安全邊界，以及管理列表的詳情頁與對話框式檢視建立、搜尋篩選分頁與具引導的空狀態、總覽的待辦與行動入口、首次進入的可略過導覽。本 capability 保證後台在共用設計系統下維持高密度且可存取的體驗、互動狀態一致可恢復、不可變識別欄位唯讀（更名是顯式動作），介面語言支援中文與英文。

## Requirements

### Requirement: 全部 browser route 由單一 SPA 提供可發現導覽

Server SHALL 以 Vite、React 與 TypeScript SPA 提供 `/`、`/setup`、`/invite/:token`、`/login`、`/activate`、`/account`、`/admin` 與 `/admin/*`；每個頁面 SHALL 可由應用內導覽或當前流程中的明確動作到達，且 SHALL 支援直接開啟與重新整理。未登入流程 SHALL 使用專注流程殼；已登入的管理員與一般成員 SHALL 共用同一個依角色裁切的主控台殼：左側側欄（頂端品牌字標、中段導覽、底部帳號選單）加主欄（頂列麵包屑、下方主內容），殼 SHALL NOT 含橫貫頂部的 header。管理員的側欄導覽 SHALL 提供總覽、使用者、專案與儲存庫、憑證、系統、稽核紀錄六個目的地，且 SHALL NOT 提供獨立的資料操作目的地；一般成員的側欄導覽 SHALL 只有帳號一項，SHALL NOT 呈現任何管理目的地。帳號 SHALL NOT 列為管理員側欄的目的地，SHALL 由側欄底部帳號選單的「帳號」項進入。帳號選單 SHALL 在觸發鈕上顯示當前使用者的顯示名稱與電子郵件，展開後 SHALL 顯示顯示名稱與「電子郵件 · 管理員」或「電子郵件 · 成員」，並提供帳號、介面語言與登出；登出請求失敗時 SPA SHALL 仍導向登入頁。管理員開啟 `/account` 時側欄 SHALL 完整呈現且無任何項目高亮。主區頂列的麵包屑 SHALL 標示目前位置：管理面為「後台 / <目的地>」且「後台」可點回總覽，帳號頁為「帳號」。SPA SHALL NOT 提供 changes、specs 或 discussions 的檢視與編輯。

#### Scenario: 管理員不需手打 URL 走訪管理功能

- **WHEN** 已登入管理員從 `/` 進入服務並依可見導覽走訪全部管理目的地
- **THEN** 管理員可經側欄到達六個管理目的地、經側欄底部帳號選單到達帳號頁與登出動作，且不需修改瀏覽器網址

#### Scenario: 管理員在帳號頁保有全站導覽

- **WHEN** 已登入管理員由帳號選單的「帳號」項進入 `/account`
- **THEN** 側欄六個目的地維持可見且無項目高亮，麵包屑為「帳號」，管理員可直接前往任一管理目的地而不使用瀏覽器上一頁

#### Scenario: 一般成員不顯示管理導覽

- **WHEN** 已登入但無 admin 旗標的成員開啟 `/account`
- **THEN** 頁面顯示帳號、存取金鑰、登入工作階段與裝置資訊；側欄只有作用中的「帳號」一項與帳號選單，不呈現任何管理目的地

#### Scenario: 資料操作目的地已併入系統

- **WHEN** 已登入管理員檢視側欄並開啟系統目的地
- **THEN** 側欄不含資料操作項目，系統頁單頁呈現執行環境、儲存狀態、匯出與資料結構遷移

#### Scenario: 深連結可直接重新整理

- **WHEN** 管理員直接開啟或重新整理 `/admin/audit`
- **THEN** Server 回傳 SPA shell，SPA 完成 session 檢查後呈現稽核紀錄頁

#### Scenario: 帳號選單提供帳號、語言與登出

- **WHEN** 已登入管理員點側欄底部顯示自己名稱與電子郵件的帳號選單
- **THEN** 選單頂端顯示顯示名稱與「電子郵件 · 管理員」，下方依序為帳號、介面語言（中文、英文、跟隨系統三選一且標示目前選擇）與登出；選登出後 session 撤銷並呈現登入頁

#### Scenario: 登出失敗仍回到登入頁

- **WHEN** 已登入使用者在帳號選單選登出，但登出請求回傳錯誤或網路中斷
- **THEN** SPA 導向登入頁，不停留在仍顯示帳號資訊的主控台殼

#### Scenario: 麵包屑標示位置並可回總覽

- **WHEN** 管理員開啟 `/admin/users` 後點麵包屑的「後台」
- **THEN** 點擊前麵包屑為「後台 / 使用者」，點擊後呈現總覽且麵包屑為「後台 / 總覽」


<!-- @trace
source: server-web-shell-alignment
updated: 2026-10-14T09:30:48+08:00
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

Server SPA SHALL 復用 `packages/ui` 的 shadcn/ui 原語、共用 semantic theme、Noto Sans TC 與青綠 focus token，SHALL NOT 建立第二套 theme 或 import Desktop 的 Tauri、Zustand store 與 SDD board 元件。介面上的下拉選單 SHALL 使用 `packages/ui` 的 shadcn/ui Select 原語，SHALL NOT 使用未套用 theme 的原生 `select`；同一組工具列內的搜尋框、下拉與日期輸入 SHALL 具有相同高度；搜尋框 SHALL 為與 Desktop 同一個共用全圓搜尋框並具有無障礙名稱，下拉與日期輸入 SHALL 具有相同圓角與陰影，且各自以無障礙名稱標示用途。下拉展開後的選單 SHALL 具有不透明底色，且在抽屜或對話框內開啟時 SHALL 可正常開啟與選取。主控台殼的側欄頂列與主區頂列高度、側欄寬度與主內容內距 SHALL 與 Desktop 應用程式殼採用相同數值；側欄頂列 SHALL 顯示共用元件庫的橫式字標，主區頂列的麵包屑 SHALL 使用與 Desktop 同一個共用麵包屑元件。主控台殼 SHALL NOT 整頁捲動——側欄與主區頂列 SHALL 恆常留在可視範圍內，只有主內容區捲動。寬度至少 1024px 時主控台殼 SHALL 顯示 icon 加 label 的固定側欄；更窄時 SHALL 隱藏固定側欄，以主區頂列上可見的 trigger 開啟從左側滑出的 Sheet，Sheet 內容 SHALL 與側欄相同（字標、該角色的導覽目的地與帳號選單），管理員與一般成員皆同。每頁標題 SHALL 使用與 Desktop 同一個共用頁標題元件（24px 一般字重標題、灰字說明、右側頁級動作），且頁標題 SHALL 為該頁唯一的第一層標題；頁標題區寬度不足以並排標題與動作時，動作槽 SHALL 換到標題下方一列，SHALL NOT 溢出主內容區。列表頁（使用者、專案與儲存庫、憑證、稽核紀錄）SHALL 全寬排版：搜尋與篩選置於頁標題區的動作槽，其下一張共用列表卡填滿主內容區剩餘高度，列於卡內縱向捲動、卡底為共用換頁列，頁面本身 SHALL NOT 因清單長度而縱向捲動。設定型頁（總覽、系統、帳號）SHALL 以 768px 置中欄排版，每個區塊為一張共用設定型卡（14px 小標、圓角卡、一列一筆、列間細線）。存取金鑰與裝置登入列上的撤銷鈕（帳號頁、憑證頁）SHALL 只在滑鼠移入該列或鍵盤焦點落在該鈕時顯示，隱藏期間 SHALL 仍在 Tab 順序內；裝置不具滑鼠移入能力時 SHALL 恆常顯示。專注流程殼 SHALL 於畫面正中呈現橫式字標，其下為一張與 Desktop 對話框同款的卡片（12px 圓角、15px 標題加灰字說明、頁尾細線上方右側為主要動作）；多步驟流程 SHALL 於卡片頂端標示「步驟 n / 總數」與步驟條；連結無效或無權限時卡片 SHALL 只呈現警示色圖示、標題與說明並以 `role=alert` 宣告。每頁 SHALL 只有一個視覺 primary action。寬度小於 1024px 時清單 SHALL 轉為可換行的清單列（稽核紀錄的表格亦同），換頁列 SHALL 收為筆數範圍與「‹ 目前頁 / 總頁數 ›」，且 SHALL NOT 造成整頁水平捲動。

字級與控件高度 SHALL 與 Desktop 相同：側欄、麵包屑與選單文字 11–13px，閱讀內文 15px，按鈕高度 28–36px（指共用按鈕元件；麵包屑與語言切換這類行內文字鈕隨所在文字列的字級）。所有互動 SHALL 可由鍵盤完成，並包含 skip link、至少 2px focus ring、連續 heading、icon-only control 的 aria-label、輸入 label／helper text／autocomplete、鄰近欄位且以 `role=alert` 宣告的錯誤與至少 4.5:1 正常文字對比。動畫 SHALL 限於 150–250ms opacity 或 transform；`prefers-reduced-motion` 啟用時 SHALL 停用動畫。

#### Scenario: 375px 完成主要流程

- **WHEN** 使用者以 375px viewport、200% zoom 與鍵盤操作 login、setup 或 invite 流程
- **THEN** 主要內容與操作保持可見、focus 順序合理、沒有整頁水平捲動，且可完成提交

#### Scenario: 窄螢幕管理導覽使用 Sheet

- **WHEN** 管理員以 768px viewport 開啟 `/admin` 並按主區頂列的開啟導覽鈕
- **THEN** 固定側欄不顯示，Sheet 從左側滑出並含字標、六個目的地與帳號選單，關閉後 focus 回到開啟導覽鈕

#### Scenario: 窄螢幕一般成員同樣使用 Sheet

- **WHEN** 一般成員以 375px viewport 開啟 `/account` 並按主區頂列的開啟導覽鈕
- **THEN** 固定側欄不顯示，Sheet 從左側滑出並含字標、「帳號」一項與帳號選單

#### Scenario: 管理殼版面數值與 Desktop 一致

- **WHEN** 管理員以 1440px viewport 開啟任一管理目的地
- **THEN** 側欄頂列與主區頂列高度、側欄寬度與主內容內距與 Desktop 應用程式殼相同，且頁面沒有橫貫頂部的 header

#### Scenario: 只有主內容區捲動

- **WHEN** 管理員開啟內容長度超過視窗高度的管理目的地並向下捲動
- **THEN** 側欄與主區頂列維持在原位，只有主內容區捲動

#### Scenario: 工具列控件高度一致

- **WHEN** 管理員檢視稽核紀錄頁同時具有關鍵字搜尋、下拉篩選與日期輸入的工具列
- **THEN** 搜尋框、下拉篩選與日期輸入的高度相同，搜尋框為全圓，下拉是套用 theme 的 Select 原語而非原生 `select`，且每個控件都有無障礙名稱

#### Scenario: 抽屜內的下拉可正常開啟

- **WHEN** 管理員在抽屜內開啟下拉選單並選取一個選項
- **THEN** 選單以不透明底色呈現於抽屜內、選取生效，且畫面不失去回應

#### Scenario: reduced motion 停用轉場

- **WHEN** 作業系統設定 `prefers-reduced-motion: reduce`
- **THEN** SPA 不執行 route、Sheet、toast 或狀態切換動畫，且功能狀態仍完整可辨識

#### Scenario: 頁標題與 Desktop 同款

- **WHEN** 管理員開啟使用者頁
- **THEN** 頁頂為該頁唯一的第一層標題「使用者」（24px 一般字重），其下為灰字說明，右側為「邀請使用者」動作

#### Scenario: 控件密度與 Desktop 相同

- **WHEN** 管理員以鍵盤在任一管理目的地的側欄、麵包屑與頁級按鈕間移動焦點
- **THEN** 導覽項、麵包屑與選單文字為 11–13px，按鈕高度落在 28–36px，且每個取得焦點的控件顯示至少 2px focus ring

#### Scenario: 專注流程以置中字標與對話框式卡片呈現

- **WHEN** 未登入使用者開啟 `/login` 並以錯誤的密碼提交
- **THEN** 畫面正中為橫式字標，其下卡片含 15px 標題「登入」、灰字說明、電子郵件與密碼欄位，頁尾細線上方右側為「登入」；提交後錯誤以 `role=alert` 緊貼欄位下方，已輸入的電子郵件保留

#### Scenario: 初始設定標示步驟

- **WHEN** 持有效 bootstrap token 的使用者開啟 `/setup` 並完成建立管理員
- **THEN** 卡片頂端先顯示「步驟 1 / 2」且步驟條第一段為主色；建立成功後改為「步驟 2 / 2」且兩段皆為主色

#### Scenario: 連結無效以提示卡呈現

- **WHEN** 使用者以已失效的初始設定或邀請連結開啟頁面
- **THEN** 卡片內只有警示色圖示、標題與說明，以 `role=alert` 宣告，不呈現任何表單欄位

#### Scenario: 列表頁的清單在列表卡內捲動

- **WHEN** 管理員以 1440px viewport 開啟有 50 位使用者的使用者頁，把每頁筆數改為 50 並向下捲動清單
- **THEN** 頁標題區與換頁列維持在原位，只有列表卡內的列捲動，主內容區本身不出現捲軸

#### Scenario: 設定型頁以置中欄與設定型卡呈現

- **WHEN** 管理員以 1440px viewport 開啟系統頁
- **THEN** 內容欄寬 768px 且水平置中，執行環境、儲存狀態、匯出與危險區各為一張帶 14px 小標的卡，卡內一列一筆

#### Scenario: 撤銷鈕滑過才出現

- **WHEN** 使用者在帳號頁把滑鼠移入一筆有效存取金鑰的列，再移出
- **THEN** 移入時該列出現「撤銷」鈕，移出後隱藏；以 Tab 移到該鈕時它同樣顯示，已撤銷的列不出現撤銷鈕

#### Scenario: 觸控裝置恆常顯示撤銷鈕

- **WHEN** 使用者以不具滑鼠移入能力的觸控裝置開啟帳號頁
- **THEN** 每筆有效存取金鑰與裝置登入列的撤銷鈕恆常可見

#### Scenario: 窄螢幕清單不造成水平捲動

- **WHEN** 管理員以 375px viewport 開啟有 12 位使用者的使用者頁與有 30 筆事件的稽核紀錄頁
- **THEN** 兩頁的清單皆為可換行的清單列；使用者頁換頁列顯示「第 1–12 筆，共 12 筆」且右側缺席，稽核紀錄頁換頁列顯示「第 1–20 筆，共 30 筆」與「‹ 1 / 2 ›」；兩頁都沒有水平捲動


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
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

管理列表頁 SHALL 以列表為主體：清單列 SHALL NOT 內嵌文字輸入、下拉選擇或提交按鈕，可開細節的清單列 SHALL 以整列為可點目標（滑鼠點擊與鍵盤 Enter／Space）開啟該筆資料的細節抽屜。建立、邀請與加入類動作 SHALL 由頁面的單一 primary action 開啟抽屜承載，SHALL NOT 常駐於頁面；此規則 SHALL 一併適用於帳號頁的建立存取金鑰。挑選既有項目加入一筆資料（例如邀請時選擇要加入的專案）SHALL 以下拉選單逐一挑選並列出已選項目，SHALL NOT 為每個候選項各渲染一個常駐勾選框。細節抽屜 SHALL 承載該筆資料的檢視與編輯，並 SHALL 在窄螢幕改為全寬呈現。停權、撤銷憑證、移除成員資格、刪除專案與資料結構遷移 SHALL 在送出前以 AlertDialog 顯示確切對象並要求確認。抽屜內提交失敗 SHALL 保持抽屜開啟、保留非祕密輸入，並把錯誤置於對應欄位附近；提交成功 SHALL 關閉抽屜、更新列表並以 `aria-live=polite` 回饋。只顯示一次的祕密值（存取金鑰明文、邀請）SHALL 附複製動作；邀請 SHALL 以受邀者可直接開啟的連結呈現，SHALL NOT 只呈現裸 token。

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

#### Scenario: 清單列不含輸入控制項

- **WHEN** 管理員以 1440px viewport 檢視使用者列表中任一列
- **THEN** 該列只呈現頭像字、顯示名稱與電子郵件、管理員籤（僅管理員）、狀態籤與所屬專案數，不含下拉選擇或提交按鈕

#### Scenario: 點整列開啟細節抽屜

- **WHEN** 管理員點擊使用者列表中的一列，或以 Tab 移到該列後按 Enter
- **THEN** 右側抽屜開啟並呈現該使用者的概要、成員資格、憑證與稽核，且成員資格可於抽屜內新增與移除

#### Scenario: 抽屜提交失敗保留輸入

- **WHEN** 管理員在邀請抽屜送出後收到含 `fieldErrors` 的 400 回應
- **THEN** 抽屜保持開啟、電子郵件與顯示名稱維持已輸入內容，錯誤以 `role=alert` 宣告於對應欄位附近


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
-->

---
### Requirement: 管理列表提供搜尋、篩選、分頁與具引導的空狀態

管理列表頁 SHALL 於頁標題區的動作槽提供工具列。使用者頁 SHALL 提供關鍵字搜尋與狀態篩選；專案與儲存庫頁 SHALL 提供關鍵字搜尋；憑證頁 SHALL 以列表卡頂端的分頁區分存取金鑰與裝置（各分頁標示該類的總筆數），並提供關鍵字搜尋與狀態篩選；稽核紀錄頁 SHALL 提供關鍵字搜尋、動作篩選、來源篩選與時間區間篩選。使用者、專案與儲存庫、憑證三個列表 SHALL 換頁瀏覽：清單非空時列表卡底部 SHALL 常駐與 Desktop 同一個共用換頁列——左側為「第 a–b 筆，共 N 筆」與每頁筆數下拉（20／50／100，預設 20），過濾後筆數超過每頁筆數時右側為第一頁、上一頁、頁碼、下一頁、最後一頁與跳頁輸入，筆數不超過每頁筆數時右側 SHALL 缺席；搜尋字串、篩選或分頁變更 SHALL 使清單回到第 1 頁；清單縮短使頁碼越界時 SHALL 鉗制至末頁；換頁後列表卡 SHALL 捲回頂部。這三個列表的搜尋、篩選與換頁 SHALL 只在前端進行，SHALL NOT 改變清單的載入方式與資料欄位。稽核紀錄頁 SHALL 使用同一條換頁列，篩選、頁碼與每頁筆數 SHALL 作為參數交由伺服器計算，「共 N 筆」SHALL 取自伺服器回傳的符合篩選總筆數；任一篩選或每頁筆數變更 SHALL 使頁碼回到第 1 頁。每頁筆數與頁碼 SHALL NOT 跨重新整理保留。列表為空時 SHALL 呈現說明該資料用途的文字，SHALL NOT 只呈現單行「尚無資料」訊息；搜尋或篩選後沒有符合項目時 SHALL 呈現與「清單本來就空」可區辨的無符合結果狀態，且 SHALL 保留工具列。該頁的建立入口 SHALL 常駐於頁首，SHALL NOT 只在列表為空時出現，空狀態內 SHALL NOT 另放第二個建立鈕。

#### Scenario: 稽核篩選由伺服器計算

- **WHEN** 管理員在稽核紀錄頁選擇動作篩選並切換到第二頁
- **THEN** SPA 以篩選、頁碼與每頁筆數參數呼叫 browser API，只呈現伺服器回傳的當頁事件，換頁列的筆數範圍與總頁數取自伺服器回傳的總筆數與總頁數

#### Scenario: 篩選無結果呈現空狀態

- **WHEN** 管理員套用的稽核篩選沒有任何符合事件
- **THEN** 頁面呈現空狀態說明並保留篩選控制項，不呈現空白表格

#### Scenario: 空憑證頁引導建立

- **WHEN** 管理員開啟尚無任何存取金鑰的憑證頁
- **THEN** 頁面說明存取金鑰的用途，且頁首的建立存取金鑰入口可用

#### Scenario: 已有憑證時建立入口仍在

- **WHEN** 管理員在已有存取金鑰的憑證頁尋找新增入口
- **THEN** 頁首仍提供建立存取金鑰的入口，不因清單非空而消失

#### Scenario: 使用者清單在前端換頁

- **WHEN** 管理員開啟有 45 位使用者的使用者頁並按第 3 頁
- **THEN** 換頁列左側由「第 1–20 筆，共 45 筆」變為「第 41–45 筆，共 45 筆」，清單只列第 41 到 45 位，且沒有發出新的 browser API 請求

#### Scenario: 搜尋使清單回到第一頁

- **WHEN** 管理員在使用者頁停在第 2 頁時輸入只符合 3 位使用者的關鍵字
- **THEN** 清單回到第 1 頁並只列那 3 位，換頁列顯示「第 1–3 筆，共 3 筆」且右側沒有頁碼鈕

#### Scenario: 稽核改每頁筆數回到第一頁

- **WHEN** 管理員在稽核紀錄頁停在第 3 頁時把每頁筆數由 20 改為 50
- **THEN** SPA 以頁碼 1 與每頁 50 筆重新呼叫 browser API，換頁列顯示「第 1–50 筆，共 N 筆」（N 為伺服器回傳的總筆數；N 小於 50 時為「第 1–N 筆」）

#### Scenario: 篩選無符合項目與清單本來就空可區辨

- **WHEN** 管理員在已有使用者的使用者頁輸入不符合任何人的關鍵字
- **THEN** 列表卡內呈現無符合結果的提示（不含建立鈕），搜尋框與狀態篩選仍在，換頁列不呈現


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
-->

---
### Requirement: 使用者頁呈現尚未接受的邀請

受邀者在接受邀請前沒有使用者資料，管理面 SHALL 於使用者頁列表卡頂端以「使用者」與「邀請中」兩個分頁區分正式使用者與仍有效（未使用且未過期）的邀請，兩個分頁 SHALL 各自標示總筆數。邀請中分頁的每筆邀請 SHALL 呈現受邀者的電子郵件、顯示名稱、角色、要加入的專案與到期時間。已接受或已過期的邀請 SHALL NOT 出現在邀請中分頁。邀請 SHALL NOT 與正式使用者混列於同一分頁——待啟用者無法被停權、沒有憑證也沒有細節可檢視，因此邀請列 SHALL NOT 可點開細節。沒有任何待啟用邀請時邀請中分頁 SHALL 仍呈現且計數為 0，切換過去 SHALL 呈現說明邀請用途的空狀態。目前分頁 SHALL 反映於網址查詢參數（`tab=pending` 為邀請中，缺席為使用者），以帶該參數的網址開啟使用者頁 SHALL 直接顯示邀請中分頁；切換分頁 SHALL NOT 新增瀏覽紀錄。關鍵字搜尋 SHALL 套用於目前分頁（邀請比對電子郵件與顯示名稱），狀態篩選 SHALL 只在使用者分頁呈現。view model SHALL NOT 帶出邀請 token 或其 hash。每筆待啟用邀請 SHALL 提供恆常顯示的撤回動作，並 SHALL 在送出前以 AlertDialog 指名受邀者要求確認；撤回後該邀請連結 SHALL 立即失效且該筆離開邀請中分頁。已接受的邀請 SHALL NOT 可撤回——其背後已有真實帳號，該走停權。

#### Scenario: 剛送出的邀請立即可見

- **WHEN** 管理員送出邀請後切到邀請中分頁
- **THEN** 該筆邀請出現在清單中，標示受邀者、要加入的專案與到期時間，且邀請中分頁的計數加一

#### Scenario: 接受後離開邀請中分頁

- **WHEN** 受邀者接受邀請並建立帳號
- **THEN** 該筆邀請不再出現在邀請中分頁，受邀者出現在使用者分頁

#### Scenario: 撤回邀請使連結立即失效

- **WHEN** 管理員撤回一筆待啟用邀請並於確認框確認
- **THEN** 該筆離開邀請中分頁，受邀者手上的連結不再能建立帳號，且留下一筆指名受邀者的稽核事件

#### Scenario: 沒有待啟用邀請時分頁計數為零

- **WHEN** 管理員開啟沒有任何有效邀請的使用者頁並切到邀請中分頁
- **THEN** 分頁計數為 0，列表卡內呈現說明邀請用途的空狀態，不呈現空白清單或換頁列

#### Scenario: 網址直達邀請中分頁

- **WHEN** 管理員開啟 `/admin/users?tab=pending`
- **THEN** 使用者頁直接顯示邀請中分頁；切回使用者分頁後網址不再帶 `tab` 參數，且瀏覽器上一頁不會回到邀請中分頁


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
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

總覽頁 SHALL 以 768px 置中欄排版。頁頂 SHALL 以四格數字磚（與 Desktop 閱讀卡的數字磚同款）呈現有效使用者、專案、有效憑證與待啟用邀請的計數，每格的說明 SHALL 帶出次要計數（停權人數、儲存庫數）或該計數的定義；每格 SHALL 可點擊並前往對應的管理目的地，待啟用邀請那格 SHALL 前往使用者頁的邀請中分頁。總覽 SHALL 以設定型卡呈現系統健康摘要與最近稽核事件，兩張卡的標頭 SHALL 分別提供前往系統目的地與稽核紀錄目的地的入口。總覽 SHALL 在存在待處理事項時於數字磚下方呈現「需要處理」卡，每則待辦為一列並 SHALL 附帶處理該事項的動作入口，待接受邀請的入口 SHALL 前往使用者頁的邀請中分頁；沒有待處理事項時 SHALL NOT 呈現該卡。識別資料結構版本 SHALL 呈現於系統健康摘要，SHALL NOT 作為獨立數字磚。

#### Scenario: 數字磚可點入對應頁

- **WHEN** 管理員點擊總覽的使用者數字磚
- **THEN** 前往使用者目的地並顯示使用者分頁

#### Scenario: 待啟用數字磚開到邀請中分頁

- **WHEN** 管理員點擊總覽的待啟用數字磚
- **THEN** 前往 `/admin/users?tab=pending`，使用者頁顯示邀請中分頁

#### Scenario: 無待辦時不呈現需要處理卡

- **WHEN** 管理員開啟總覽且系統沒有任何待處理事項
- **THEN** 頁面不呈現需要處理卡，不留下空白小標或空卡

#### Scenario: 待辦附帶處理入口

- **WHEN** 系統沒有任何有效憑證且管理員開啟總覽
- **THEN** 需要處理卡呈現該事項並提供建立存取金鑰的動作入口


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
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

Server SPA SHALL 以中文（zh-TW）與英文（en）提供全部使用者可見文案，兩種語言的訊息集合 SHALL 完全對應。使用者未明示偏好時 SHALL 依瀏覽器語言決定——以 `zh` 開頭者為中文，其餘為英文。已登入時介面 SHALL 於側欄底部帳號選單提供語言切換，可選中文、英文或跟隨系統，且 SHALL 標示目前選擇；專注流程殼 SHALL 於卡片下方提供中文與英文兩個切換鈕並標示目前語言。切換 SHALL 立即生效並在重新載入後維持，兩處切換 SHALL 寫入同一個偏好。語言選擇 SHALL NOT 影響 artifacts 產出語言、CLI 輸出、稽核事件的動作名稱或伺服器回傳的錯誤訊息。

#### Scenario: 未設定偏好時跟隨瀏覽器語言

- **WHEN** 尚未選過語言的使用者以英文瀏覽器開啟管理面
- **THEN** 介面文案為英文

#### Scenario: 切換語言即時生效並持續

- **WHEN** 管理員在帳號選單將語言切換為英文並重新整理頁面
- **THEN** 管理面文案維持英文，不因重新載入而還原

#### Scenario: 兩種語言的訊息集合對應

- **WHEN** 檢視任一管理目的地的中文與英文版本
- **THEN** 兩者呈現相同的欄位、動作與說明，沒有任何一方缺漏或退回顯示訊息代碼

#### Scenario: 跟隨系統清除偏好

- **WHEN** 已選過英文的使用者在帳號選單選「跟隨系統」，且瀏覽器語言為 zh-TW
- **THEN** 偏好被清除，介面文案立即回到中文，重新整理後仍為中文

#### Scenario: 登入前可切換語言

- **WHEN** 尚未選過語言、瀏覽器為 zh-TW 的使用者在登入頁點卡片下方的「English」後登入
- **THEN** 登入頁立即改為英文且「English」標示為目前語言，登入後的管理面文案也是英文


<!-- @trace
source: server-web-shell-alignment
updated: 2026-10-14T09:30:48+08:00
-->

---
### Requirement: 帳號頁呈現我的專案

`/account` SHALL 以 768px 置中欄與設定型卡排版，依序為我的專案、存取金鑰、裝置登入與登入工作階段四張卡。我的專案卡 SHALL 列出目前使用者隸屬的每個專案，每列左側為顯示名（顯示名缺席時以專案 key 呈現）與等寬的專案 key、右側為角色籤；資料 SHALL 來自 account summary、SHALL NOT 另打管理端點。admin 與一般成員 SHALL 看到同一張卡與同一形狀（admin 不因管理身分而多列非隸屬專案）。無任何隸屬時卡內 SHALL 呈現引導性空狀態（說明由管理員授予隸屬），SHALL NOT 隱藏整張卡。本卡 SHALL 為唯讀，SHALL NOT 提供任何隸屬變更操作。

#### Scenario: 成員看到自己的專案與角色

- **WHEN** 隸屬兩個專案的一般成員開啟 /account
- **THEN** 我的專案卡列出兩個專案的顯示名、專案 key 與各自角色籤，無任何編輯操作

#### Scenario: admin 看到的是自己的隸屬而非全部專案

- **WHEN** 具 admin 旗標、僅隸屬一個專案的使用者開啟 /account
- **THEN** 我的專案卡僅列該一個專案；其餘專案不出現（全部專案屬 /admin 的治理視角）

#### Scenario: 無隸屬時的空狀態

- **WHEN** 無任何專案隸屬的使用者開啟 /account
- **THEN** 我的專案卡內呈現空狀態文字，說明隸屬由管理員授予；卡本身與小標仍可見

#### Scenario: 帳號頁四張卡的順序

- **WHEN** 使用者開啟 /account
- **THEN** 由上而下依序為我的專案、存取金鑰、裝置登入、登入工作階段四張卡，內容欄寬 768px 且水平置中


<!-- @trace
source: server-web-list-pages
updated: 2026-10-14T17:14:35+08:00
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

---
### Requirement: 網頁後台可點元素顯示手指游標

網頁後台的可點元素——按鈕與圖示鈕、可選取的清單列與卡片、分頁、選單項與下拉選項、側欄導覽——滑鼠移入時 SHALL 顯示手指游標。手指游標 SHALL 來自共用樣式的游標設定預設值（與桌面 app 共用同一份設定，桌面 app 於根樣式覆寫為箭頭），網頁後台的根樣式 SHALL NOT 覆寫該設定；元件 SHALL NOT 自行寫死手指游標 class，可點但非按鈕的元素（如可選取的表格列與卡片）SHALL 取用同一份設定。

停用或唯讀的元素 SHALL 為箭頭游標，SHALL NOT 顯示禁止游標，SHALL NOT 顯示 hover 底色。文字輸入的文字游標不受本需求約束。

#### Scenario: 管理列表的按鈕與可選取列顯示手指

- **WHEN** 管理員在瀏覽器開啟網頁後台的管理列表，滑鼠移到工具列的建立按鈕，再移到一筆可選取的列
- **THEN** 兩者游標皆為手指

#### Scenario: 選單項顯示手指

- **WHEN** 管理員在網頁後台展開任一下拉選單，滑鼠移到其中一個可用的項目
- **THEN** 游標為手指

#### Scenario: 停用的按鈕為箭頭且無 hover 底色

- **WHEN** 管理員在網頁後台把滑鼠移到一個停用的按鈕上
- **THEN** 游標為箭頭（非禁止游標），按鈕底色與未 hover 時相同

<!-- @trace
source: clickable-cursor-hover-rule
updated: 2026-10-13T11:49:40+08:00
-->