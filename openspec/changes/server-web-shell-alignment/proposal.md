## Summary

server 網頁後台（apps/server-web）的殼、全部頁標題與登入這一族四頁，改成 desktop 的樣子：拿掉頂部橫條、側欄頂端放橫式 Logo、主區頂列放麵包屑、帳號資訊收進側欄底部的帳號選單；頁標題改用共用的頁標題區；登入、初始設定、接受邀請、裝置登入改成置中 Logo 加一張對話框式卡片。

## Motivation

目標使用者是架設 Speclink server 的管理員，以及透過 AI 代理跑遠端 SDD 流程、要登入後台管理存取金鑰的成員（開發者／PO／PM）。使用情境是「連線 Server」前後：在瀏覽器完成初始設定、接受邀請、登入、核准桌面 app 或 CLI 的裝置登入，再到後台管理使用者、專案與憑證。

- desktop 已依 desktop-ui-redesign 全面改版（左側標題列、主區麵包屑、頁標題區、設定零件、列式清單），server 後台的殼與版面仍停在 admin-console-redesign 的樣子：頂部橫貫的 header、email 連結加語言下拉加登出鈕並排、頁標題是 24px 粗體。同一個產品的兩個入口長得像兩套軟體。
- 兩邊的顏色與字型早已同源（兩邊的根樣式都 import `packages/ui/src/theme.css`），不一致的是殼、版面、元件與互動模式。
- 規格「共用設計系統維持高密度可存取體驗」要求正文至少 16px、互動目標至少 44×44px，但沒有任何測試檢查，共用按鈕本來就是 h-9／h-7，整個 server-web 只有 header 的 email 連結有 `min-h-11`。承諾與實作早已分家，也讓後台控件比 desktop 大一號。

本變更是討論 server-web-desktop-alignment 定案的三刀中的第一刀（殼、頁標題、登入這一族）。第二刀 server-web-list-pages（列表頁與設定型頁）、第三刀 server-web-detail-pages（抽屜改詳情頁、建立類改對話框）依序在本刀封存後轉出。視覺基準是討論中確認的互動原型（https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底「server 後台對齊 desktop」A／B／C 三列）。

## Proposed Solution

- **殼（ConsoleLayout）**：拿掉頂部橫貫的 header，管理員與一般成員共用同一種形狀。
  - 側欄寬 200px：頂端 48px 放有文字的橫式 Logo（共用 `Wordmark`，高 20px）；中段是導覽；底部是帳號選單鈕（頭像字、顯示名稱、email）。
  - 管理員的導覽維持六個目的地（總覽、使用者、專案與儲存庫；維運：憑證、系統、稽核紀錄），在 `/account` 時無項目高亮；一般成員的導覽只有「帳號」一項。
  - 帳號選單（共用 dropdown-menu 原語）：選單頂端顯示名稱與「email · 管理員／成員」，選單項為帳號、介面語言（中文／English／跟隨系統三選一）、登出。
  - 主區頂列 48px 放麵包屑：管理面為「後台 / <目的地>」，「後台」可點回總覽；帳號頁為「帳號」。
  - 寬度小於 1024px 時隱藏固定側欄，主區頂列左端出現「開啟導覽」鈕，導覽以 Sheet 從左側滑出，內容與側欄相同（Logo、導覽、帳號選單）。
  - 只有主內容區捲動，側欄與頂列恆常可見；主內容內距維持 desktop 看板的 20px。
- **頁標題**：六個管理頁與帳號頁的標題改用共用 `PageHeader`（24px 一般字重標題、灰字說明、右側頁級動作）。既有的頁級動作（邀請使用者、建立專案、建立存取金鑰、重看導覽）移進動作槽，`data-tour` 標記隨之保留。頁標題仍是頁面唯一的第一層標題。
- **登入這一族（FocusLayout）**：畫面正中央放 40px 橫式 Logo，下方一張對話框式卡片（12px 圓角、15px 標題加灰字說明、頁尾細線上方右側放主要鈕），卡片下方是「中文 · English」語言切換。
  - 初始設定：卡片頂端加「初始設定 · 步驟 n / 2」與兩段步驟條。
  - 連結無效（初始設定、接受邀請）與無權限（一般成員開 `/admin`）：琥珀色圖示、標題與說明。
  - 裝置登入四個狀態：輸入代碼、確認（大字代碼加「核准後以 <名稱>（<email>）的身分登入」）、已核准、已拒絕；說明文字寫「桌面 app 或 CLI」。
  - 錯誤維持 `role=alert`，緊貼在欄位下方。
- **共用元件（packages/ui）**：
  - 新增 `Breadcrumbs`：desktop 主區頂列的麵包屑搬進共用元件庫，desktop 與 server-web 共用一份。
  - `PageHeader` 加標題層級屬性，server-web 以第一層標題使用，desktop 維持第二層。
  - `SheetContent` 加從左側滑出的選項。
  - dropdown-menu 補單選群組與單選項，供帳號選單的介面語言使用。
- **規格**：server-web-console 三條需求 MODIFIED。
  - 「全部 browser route 由單一 SPA 提供可發現導覽」：帳號入口改為側欄底部帳號選單；一般成員也有側欄（只有帳號一項）。
  - 「共用設計系統維持高密度可存取體驗」：header 改為側欄頂列與主區頂列；16px 正文與 44×44px 互動目標改為 desktop 密度（外殼 11–13px、閱讀內文 15px、按鈕 28–36px）；窄螢幕 Sheet 改為兩種角色都有、從左側滑出。
  - 「介面語言支援中文與英文」：語言切換從 header 移到帳號選單與登入這一族的卡片下方。討論結論只列前兩條，這一條是 propose 期查規格時發現「於 header 提供語言切換」會隨 header 一起失效而補上。

相容性影響：只改 server 網頁後台與共用 UI 套件的畫面。網址、六個目的地、server API、CLI 人眼輸出與 `--json`、引擎全部不變；desktop 的麵包屑外觀與行為不變（只換成共用元件）。既有使用者不需遷移；語言偏好沿用同一個瀏覽器儲存鍵。

## Alternatives Considered

- 做 desktop 的 56px 圖示列：後台沒有多專案切換，圖示列會是空的。
- 一般成員維持無側欄，或用 56px 窄欄加裸標記：殼會有兩種形狀，同一個後台會有兩套左上角品牌。
- 側欄頂端用裸標記加純文字：使用者裁定用有文字的橫式 Logo。
- 保留 16px 正文與 44px 互動目標：後台控件會比 desktop 大一號，一致只做一半；共用按鈕也從未遵守。
- server-web 自寫一份麵包屑：同一個介面元件兩份實作，正是 desktop-app「共用元件唯一來源」要消掉的換皮成本。
- 介面語言做成帳號選單的子選單：要多一組子選單原語；三個選項直接以單選項放在選單內就夠用。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `server-web-console`: 「全部 browser route 由單一 SPA 提供可發現導覽」帳號入口改側欄底部帳號選單、一般成員有側欄；「共用設計系統維持高密度可存取體驗」殼結構改為側欄頂列加主區頂列、密度改為 desktop 數值、窄螢幕 Sheet 兩種角色都有；「介面語言支援中文與英文」語言切換改在帳號選單與登入這一族的卡片下方。

## Impact

- Affected specs: server-web-console（3 條 MODIFIED）
- Affected app／package: `apps/server-web`（後台 SPA）、`packages/ui`（共用 UI 元件）、`apps/desktop`（只換用共用麵包屑）；speclink-core、speclink-cli、speclink-host、server crates 皆不受影響
- Affected code:
  - New: `packages/ui/src/components/Breadcrumbs.tsx`、`packages/ui/src/__tests__/breadcrumbs.test.tsx`、`packages/ui/src/__tests__/sheet.test.tsx`、`apps/server-web/src/components/AccountMenu.tsx`
  - Modified: `apps/server-web/src/layouts/ConsoleLayout.tsx`、`apps/server-web/src/layouts/FocusLayout.tsx`、`apps/server-web/src/components/AdminNav.tsx`、`apps/server-web/src/routes/AppRoutes.tsx`、`apps/server-web/src/pages/LoginPage.tsx`、`apps/server-web/src/pages/SetupPage.tsx`、`apps/server-web/src/pages/InvitePage.tsx`、`apps/server-web/src/pages/ActivatePage.tsx`、`apps/server-web/src/pages/AccountPage.tsx`、`apps/server-web/src/pages/admin/OverviewPage.tsx`、`apps/server-web/src/pages/admin/UsersPage.tsx`、`apps/server-web/src/pages/admin/RegistryPage.tsx`、`apps/server-web/src/pages/admin/CredentialsPage.tsx`、`apps/server-web/src/pages/admin/SystemPage.tsx`、`apps/server-web/src/pages/admin/AuditPage.tsx`、`apps/server-web/src/i18n/messages.ts`、`apps/server-web/src/__tests__/admin-console-shell.test.tsx`、`apps/server-web/src/__tests__/i18n.test.tsx`、`apps/server-web/src/__tests__/app.test.tsx`、`apps/server-web/src/__tests__/setup.test.tsx`、`apps/server-web/src/__tests__/invite.test.tsx`、`apps/server-web/src/__tests__/activate.test.tsx`、`packages/ui/src/components/ui/page-header.tsx`、`packages/ui/src/components/ui/sheet.tsx`、`packages/ui/src/components/ui/dropdown-menu.tsx`、`packages/ui/src/index.ts`、`packages/ui/src/i18n.tsx`、`packages/ui/src/__tests__/pageHeader.test.tsx`、`packages/ui/src/__tests__/menus.test.tsx`、`apps/desktop/src/components/TitleBar.tsx`、`apps/desktop/src/App.tsx`
  - Removed: `apps/server-web/src/components/HeaderAccount.tsx`、`apps/server-web/src/components/LocaleSwitch.tsx`、`apps/server-web/src/components/LogoutButton.tsx`
