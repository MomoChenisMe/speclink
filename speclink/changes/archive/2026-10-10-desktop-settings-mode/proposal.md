## Summary

把桌面 app（apps/desktop）的應用程式設定與專案設定改成「設定模式」：點圖示列齒輪後左欄換成設定分類清單（應用程式：一般、語言與外觀、更新、CLI、伺服器），點專案欄「專案設定」後左欄換成專案設定分類（專案說明、產出規則、產出流程、.speclink.yaml），主區每個分類一頁，全部以五個共用設定零件（頁標題、卡、列、下拉、群組標題）排版；新增外觀（淺色／深色／跟隨系統）偏好與更新自動檢查開關。

## Motivation

討論 desktop-ui-redesign 的結論把設定從「主區兩個頁簽各塞幾張卡」改成與 macOS 系統設定同構的分類欄＋單頁：使用者在互動原型上確認了一般、更新、伺服器、專案說明、產出規則五張畫板。外殼刀已把左側做成圖示列＋專案欄，專案欄在設定模式換成分類清單正好對上「專案為頂層脈絡、設定為全域」的模型；卡片標頭式分頁、`PageHeader`、`SearchField`、`NavItem` 都已在前幾刀落地，設定模式只差五個設定零件與 `Switch` 原語。深色模式目前只能跟隨系統，結論要求設定頁提供外觀三選。

目標使用者是調整介面語言、外觀、更新、CLI、伺服器連線，以及編輯專案說明、產出規則、產出流程與 AI 工具的開發者；對應 config 技能與日常使用。

本 change 是 cut 4 的第一塊：模式、零件、六頁中的五頁（一般、語言與外觀、更新、CLI、伺服器）與專案設定四頁的分類化。「關於」頁在 `desktop-about-page`，專案說明與產出規則的 CodeMirror 編輯器在 `desktop-settings-editors`，兩者皆依賴本 change。

## Proposed Solution

- **設定模式**：`boardView` 為 settings 時，左側恆為 256px（圖示列＋設定分類欄，零專案亦然）：分類欄頂部一個全圓搜尋框（比對分類名與各頁列標籤），其下群組「應用程式」：一般、語言與外觀、更新、CLI、伺服器；左側標題列「← 設定」返回鈕（回到進入設定前的頁面）；主區麵包屑「設定 / 分類名」。`boardView` 為 project-settings 時專案欄換成專案設定分類欄：群組「openspec/config.yaml」：專案說明、產出規則、產出流程；群組「專案」：.speclink.yaml（remote 專案無此組）；左側標題列為「← 專案設定」返回鈕＋右端等寬小字專案名；麵包屑「專案名 / 專案設定 / 分類名」。解析失敗的分類項帶警示點。
- **五個設定零件與 `Switch`**：packages/ui 新增 `SettingsPageHeader`（頁標題＋說明＋右側動作＋「僅存於此裝置」註記槽）、`SettingsCard`（14px 小標＋16px 圓角卡）、`SettingsRow`（左標籤＋說明、右控制項，列間細線）、`SettingsChoice`（列右側下拉）、`SettingsGroupHeader`（圖示＋標籤＋計數，分類欄群組與卡標共用）與 Radix `Switch` 原語。
- **應用程式設定六頁中的五頁**：一般（介面語言卡：顯示語言、外觀；軟體更新卡：目前版本、檢查更新、更新日誌；CLI 指令卡：狀態＋安裝／重新安裝；鍵盤快捷鍵卡）、語言與外觀（顯示語言、外觀）、更新（頁標題右側「檢查更新」；版本卡：目前版本、最新版本＋狀態籤；下載與安裝卡依狀態出現；更新日誌卡：最新版本展開、較舊版本一列一個檢視、底部全部更新日誌；自動檢查卡：啟動時在背景檢查的開關，預設開、存 app 本機）、CLI（狀態、安裝、PATH 提示）、伺服器（頁標題右側「新增連線」展開新增卡；連線卡一列一個 server：名稱＋狀態籤、等寬網址、身分，右側登出或登入／重新登入＋「⋯」開啟專案／移除；預設卡：「新增遠端專案時預設使用」下拉，新增專案對話框的 Server 步驟預選；登入方式卡：裝置登入與存取金鑰兩列說明）。
- **外觀偏好**：淺色／深色／跟隨系統存 app 本機，啟動時與切換時套到主視窗與系統匣面板；theme.css 的深色 token 同時提供給系統偏好與明示深色兩種觸發，淺色明示時壓過系統深色。
- **專案設定四頁**：既有 config.yaml／Schema／.speclink.yaml 三頁簽的內容拆成四頁——專案說明（既有卡，編輯／取消／儲存移到頁標題右側）、產出規則（既有卡，同上）、產出流程（產出政策卡：locale、spec_locale 下拉與 tdd、audit、worktree 開關；產出流程卡：清單與唯讀詳情）、.speclink.yaml（檔案路徑列與 AI 工具卡）；寫入語意、解析驗證、remote revision 衝突對照、reader 唯讀全部不變。
- **原生選單與入口**：設定… 進一般頁、檢查更新… 進更新頁、安裝 CLI… 進 CLI 頁；圖示列更新鈕與 toast「查看」進更新頁；登入過期導向進伺服器頁。
- **詞彙**：LANGUAGE.md「產出流程」條目的「Schema」頁籤標籤例外隨頁籤消失而退場；「Workflow」頁簽標籤同樣消失。

## Non-Goals

- 「關於 Speclink」頁與分類欄的「關於」群組——`desktop-about-page`。
- 專案說明與產出規則的 CodeMirror 編輯器、編輯／預覽切換、⌘S——`desktop-settings-editors`；本 change 沿用多行文字區。
- 伺服器頁「登入過期時提醒」列——沒有對應行為，不做；「以金鑰登入…」鈕——金鑰登入仍由伺服器不支援裝置登入時就地出現。
- 更新頁的「安裝日期」與下載大小——更新端點與本機都沒有這些資料。
- 設定搜尋跳到頁內某一列——只過濾分類。
- server-web 的主題切換。

## Alternatives Considered

- 保留兩頁簽只換皮——結論已否決：設定與專案設定要和「主區就是目前在看的東西」同一套版面，分類欄才對得上專案欄的位置。
- 深色 token 改用 `light-dark()` 單寫一次——WKWebView 要 macOS 14.5 才支援，與既有深色支援面不一致；改為兩個選擇器區塊各一份並以測試釘住兩份逐行相同。
- 把產出政策卡留在專案說明頁——政策（locale、tdd、audit、worktree）與流程（schema）同屬「怎麼產出」，放產出流程頁一起看；原型分類未列政策卡，此為補位。
- 設定搜尋做全文（含列說明）——只有十幾列，分類名加列標籤就夠。

## Impact

- Affected specs: desktop-config（設定頁圖形化讀寫兩層設定、UI 介面語言支援 zh-TW 與 en、設定頁編輯專案說明與產出規則、設定頁政策下拉的未知值顯性呈現、產出流程的編輯入口、remote 模式的內建限縮與誤解析修正 MODIFIED；設定頁的產出流程頁籤以 REMOVED＋ADDED 更名為產出流程頁；ADDED 外觀偏好）、desktop-app（側欄導覽結構、桌面自動更新、安裝 CLI 指令到 PATH、更新日誌彈窗、設定頁的鍵盤快捷鍵卡 MODIFIED）、desktop-connections（伺服器管理最小面 MODIFIED）
- Affected code:
  - New: `packages/ui/src/components/ui/switch.tsx`、`packages/ui/src/components/ui/settings.tsx`、`packages/ui/src/theme.ts`、`packages/ui/src/__tests__/switch.test.tsx`、`packages/ui/src/__tests__/settingsParts.test.tsx`、`packages/ui/src/__tests__/themePreference.test.ts`、`apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/components/SettingsColumn.tsx`、`apps/desktop/src/components/ProjectSettingsColumn.tsx`、`apps/desktop/src/views/settings/GeneralPage.tsx`、`apps/desktop/src/views/settings/AppearancePage.tsx`、`apps/desktop/src/views/settings/UpdatesPage.tsx`、`apps/desktop/src/views/settings/CliPage.tsx`、`apps/desktop/src/views/settings/ServersPage.tsx`、`apps/desktop/src/views/project-settings/useProjectSettings.ts`、`apps/desktop/src/views/project-settings/ContextPage.tsx`、`apps/desktop/src/views/project-settings/RulesPage.tsx`、`apps/desktop/src/views/project-settings/WorkflowPage.tsx`、`apps/desktop/src/views/project-settings/SpeclinkPage.tsx`、`apps/desktop/src/views/project-settings/parts.tsx`、`apps/desktop/src/__tests__/settingsColumn.test.tsx`、`apps/desktop/src/__tests__/updatesPage.test.tsx`、`apps/desktop/src/__tests__/serversPage.test.tsx`、`apps/desktop/src/__tests__/themeBoot.test.ts`
  - Modified: `package-lock.json`、`packages/ui/package.json`、`packages/ui/src/index.ts`、`packages/ui/src/theme.css`、`packages/ui/src/__tests__/theme.test.ts`、`packages/ui/src/components/NavItem.tsx`、`packages/ui/src/components/Brand.tsx`、`packages/ui/src/__tests__/brand.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/main.tsx`、`apps/desktop/src/panel/main.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/components/TitleBar.tsx`、`apps/desktop/src/components/ProjectColumn.tsx`、`apps/desktop/src/components/ServersPanel.tsx`、`apps/desktop/src/components/UpdateInstallRow.tsx`、`apps/desktop/src/components/WorkspaceChooser.tsx`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`、`apps/desktop/src/__tests__/serversPanel.test.tsx`、`apps/desktop/src/__tests__/titleBar.test.tsx`、`apps/desktop/src/__tests__/workspaceChooser.test.tsx`、`apps/desktop/src/__tests__/updateInstallRow.test.tsx`、`apps/desktop/src/__tests__/updateRailButton.test.tsx`、`apps/desktop/src/__tests__/messages.test.ts`、`apps/desktop/src/__tests__/emptyWorkspace.test.tsx`、`apps/server-web/src/__tests__/admin-console-shell.test.tsx`、`apps/desktop/index.html`、`apps/desktop/src-tauri/capabilities/macos.json`、`apps/desktop/src/adapter/connections.ts`、`packages/ui/src/components/ui/status-pill.tsx`、`apps/desktop/src/__tests__/remoteResilience.test.tsx`、`openspec/LANGUAGE.md`
  - Removed: （無）
