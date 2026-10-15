## Context

應用程式設定目前是 `AppSettingsView`（本機設定／伺服器兩頁簽，卡片式）、專案設定是 `ProjectSettingsView`（1294 行，config.yaml／Schema／.speclink.yaml 三頁簽，含 remote 的 Workflow 單簽、revision 衝突對照、schema 清單與建立／fork／刪除）。外殼刀的 `ProjectColumn`（200px）在 `boardView` 為 settings 時全部非作用中、`LeftTitleBar` 顯示「設定」、麵包屑一段。`PageHeader`、`SearchField`、`NavItem`、`Card`、`ConfirmDialog`、語意 token 都已就位；`Switch` 原語與 Radix switch 相依尚未加入。深色只跟隨 `prefers-color-scheme`，theme.css 頂部注記「若某宿主要手動切換主題，另加 class 層」。UI 語言偏好走 `packages/ui/src/locale.ts`（localStorage 單鍵、null 跟隨系統）。更新狀態機在 `apps/desktop/src/core/updater.ts`，更新日誌條目在 `apps/desktop/src/release-notes/`。

## Goals / Non-Goals

**Goals**

- 設定模式與專案設定模式各有分類欄；每個分類一頁；全部頁用五個設定零件排版。
- 外觀偏好三選，套用到主視窗與系統匣面板。
- 更新頁（版本、下載與安裝、更新日誌、自動檢查開關）、伺服器頁（連線、預設、登入方式）。
- 專案設定四頁沿用既有讀寫邏輯，只換版面與分頁方式。
- 既有行為（寫入語意、解析驗證、remote 衝突、reader 唯讀、更新節流、CLI 佈署、連線登入）全部保留。

**Non-Goals**

- 關於頁（`desktop-about-page`）、CodeMirror 編輯器（`desktop-settings-editors`）。
- 設定搜尋跳到列、伺服器頁提醒設定、金鑰登入鈕。

## Decisions

### D1 設定狀態與分類目錄

store 新增 `settingsSection: "general" | "appearance" | "updates" | "cli" | "servers"`（預設 general）與 `projectSettingsSection: "context" | "rules" | "workflow" | "speclink"`（預設 context），各有 setter；`setBoardView("settings")` 不改 section（記住上次分類）。`openSettingsUpdate()` 改為切 settings＋section updates＋`focusUpdater`；`openConnectionReauth(id)` 切 settings＋section servers；原生選單 settings → general、checkUpdates → updates、installCli → cli。`apps/desktop/src/settingsCatalog.ts` 以常數列出應用程式分類：`{ id, labelKey, icon, group: "app", rows: string[] /* 該頁各列標籤的 i18n 鍵 */ }`，與專案設定分類 `{ id, labelKey, icon, group: "config" | "project", remote: boolean }`；分類欄的搜尋以分類標籤與 rows 標籤做大小寫不敏感子字串比對。

### D2 分類欄與左側標題列

- `SettingsColumn`（apps/desktop/src/components/SettingsColumn.tsx）：`<nav aria-label="設定分類">` 200px，頂部 `SearchField`（全圓、h-8），其下 `SettingsGroupHeader`「應用程式」＋ `NavItem` 五項（搜尋無命中時顯示灰字「沒有符合的設定」）。設定模式下左側容器恆為 256px（圖示列＋分類欄），零專案亦然——分類欄是全域設定的導覽，不依賴專案。
- `ProjectSettingsColumn`：群組「openspec/config.yaml」：專案說明、產出規則、產出流程；群組「專案」：.speclink.yaml（remote 專案不渲染第二組）；config.yaml 解析失敗時前三項帶警示點、.speclink.yaml 解析失敗時第四項帶警示點（既有 `TabWarningDot` 改放 `NavItem` 的 `badge` 槽）。
- `LeftTitleBar` 新增 `back?: { label, onClick }` 與 `trailing?: ReactNode`：專案設定模式時左端為 ghost 小鈕「← 專案設定」、右端等寬 11px 灰字專案名；設定模式左端為「← 設定」（apply 期補上——原設計只留標題，設定模式沒有離開途徑）。兩個返回鈕都呼叫 store 的 `leaveSettings()`：回到 `settingsReturnView`——`setBoardView` 從非設定頁進入任一設定模式時記下來處，兩種設定模式之間切換不改寫。返回收成全系統一個動作：`App.tsx` 的 `resolveBack(state)` 依畫面決定去處（詳情頁開著→關頁，確認框待確認時為 null；設定模式→`leaveSettings`，專案設定有未存修改時改為開「要捨棄未儲存的修改嗎？」確認框、確認才 `leaveSettings`；其餘為 null），返回鈕、Esc、⌘[／Alt+←、滑鼠側鍵與 `SwipeBack` 都經它，各頁不各自處理；有對話框開著時所有返回都不作用——這一條在觸發當下判斷（`SwipeBack` 的開關只看狀態：渲染時讀到的 DOM 是上一刻，拿來決定開關會在對話框關掉後卡住）；設定頁的 `<main>` 會捲動，`SwipeBack` 因此移到 `<main>` 外、掛在主區欄（`relative overflow-hidden`：指示自左緣外推出，主區欄要裁掉左緣外那段，否則疊到分類欄上）。
- 麵包屑：settings 為「設定」（點擊回 general）／「分類名」；project-settings 為「專案名」（點擊回看板）／「專案設定」（點擊回 context）／「分類名」。

### D3 五個設定零件與 `Switch`

`packages/ui/src/components/ui/settings.tsx`：

- `SettingsPageHeader({ title, description, actions, note })`：包 `PageHeader`，`note` 為其下一行 12px 灰字（本機頁放「此頁內容僅存於此裝置，不寫入版本庫」）。
- `SettingsCard({ title, count?, icon?, children })`：`SettingsGroupHeader` 當 14px 小標＋`Card`（16px 圓角）；children 為列，列間 `divide-y`。
- `SettingsRow({ label, description, control, htmlFor?, warning? })`：`grid grid-cols-[1fr_auto] items-center gap-4 px-4 py-3`，左 14px 標籤＋12px 灰字說明，右控制項；`warning` 為 true 時說明以警示色。
- `SettingsChoice({ value, options, onChange, ariaLabel, disabled })`：包既有 `Select`，觸發鈕 h-8、文字 13px，放在 `SettingsRow.control`。
- `SettingsGroupHeader({ icon, label, count })`：12px 粗體灰字＋可選圖示與計數徽章。
- `packages/ui/src/components/ui/switch.tsx`：Radix `@radix-ui/react-switch`，軌 h-5 w-9、thumb 16px，開為 `bg-primary`、關為 `bg-input`，focus 環沿 Button。

### D4 外觀偏好

- `packages/ui/src/theme.ts`：`ThemePreference = "light" | "dark" | null`，`readThemePreference`／`writeThemePreference`（localStorage 鍵 `speclink.theme`）、`applyThemePreference(pref, root = document.documentElement)`：light／dark 設 `data-theme`，null 移除屬性。
- `theme.css`：深色 token 區塊改為兩個選擇器各一份——`@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }` 與 `:root[data-theme="dark"] { … }`；`theme.test.ts` 新增「兩個深色區塊的宣告逐行相同」與「保留 data-theme 選擇器」兩案，既有「保留系統偏好查詢」案維持。
- 套用點：主視窗由 `apps/desktop/index.html` head 的 inline script 在首次繪製前讀 `THEME_STORAGE_KEY` 設 `data-theme`（品質站修正：module script 延後執行，原本放 `main.tsx` 會先閃一次系統主題）；`apps/desktop/src/panel/main.tsx` 啟動時 `applyThemePreference(readThemePreference())`，並以 `getCurrentWindow().setTheme(pref)` 讓原生視窗外觀跟上（vibrancy 材質跟原生外觀走；`capabilities/macos.json` 加 `core:window:allow-set-theme`）；讀寫 localStorage 包 try/catch，不可用時視同跟隨系統；App 持 `themePref` state，切換時寫入並套用，並以 Tauri 事件 `speclink:theme-changed` 通知面板 webview 重套（面板監聽後 `applyThemePreference`）。server-web 不讀此鍵、不設屬性。
- 不經 token 的深色呈現同樣要跟上明示外觀（apply 期補記）：Tailwind 內建的 `dark:` 只看 `prefers-color-scheme`，Streamdown 程式碼區塊的 `dark:` class 會在明示外觀與系統相反時反向。`theme.css` 以 `@custom-variant dark` 改為與深色 token 同一套觸發（系統深色且未明示淺色、或明示深色）；`Wordmark` 由 `<picture media>` 改為深淺兩張 `img` 以 `dark:` 切換（隱藏的一張不進無障礙樹）。server-web 不設屬性，行為與改前相同。

### D5 應用程式設定頁

`AppSettingsView` 改為依 `settingsSection` 渲染 `views/settings/*Page.tsx`，每頁 `mx-auto w-full max-w-[768px] pt-7 px-6 pb-10 flex flex-col gap-6`：

- **GeneralPage**：`SettingsPageHeader`「一般」＋ note；卡「介面語言」：列「顯示語言」（`SettingsChoice`：跟隨系統／繁體中文／English）、列「外觀」（`SettingsChoice`：跟隨系統／淺色／深色）；卡「軟體更新」（有 updater 時）：列「目前版本」（等寬版號）、列「檢查更新」（說明為 `updater.help`；控制為 outline 鈕＋行內狀態 `UpdaterInlineStatus`——品質站修正後與更新頁「最新版本」列共用 `latestOf` 的狀態籤與等寬版號 `VersionText`，閒置與安裝失敗不顯示；原行內文字鍵 `updater.checking|upToDate|checkFailed` 刪除）、列「更新日誌」（outline「檢視」）；卡「CLI 指令」（有 cliInstall 時）：列「speclink CLI」（說明 `cliInstall.help`；控制為狀態文字＋安裝／重新安裝鈕）＋ PATH 提示與錯誤列；卡「鍵盤快捷鍵」（既有內容改列式：動作名左、`Kbd` 右）；`trayPanelError` 於頁底。
- **AppearancePage**：「語言與外觀」：卡「介面」兩列（顯示語言、外觀），與一般頁同一組控制元件（抽成 `LocaleRow`／`ThemeRow`）。
- **UpdatesPage**（D6）。
- **CliPage**：「CLI」：卡「speclink CLI」列（狀態、安裝鈕）、列「佈署位置」（等寬 deployDir；null 時缺席）、PATH 提示列、平台說明列（Windows 安裝器管理／Linux 非 AppImage）。
- **ServersPage**（D7）。

### D6 更新頁

`UpdatesPage`：`SettingsPageHeader`「更新」＋右側「檢查更新」outline 鈕（checking 時 disabled）＋說明。

- 卡「版本」：列「目前版本」（等寬 `currentVersion`）、列「最新版本」（等寬版號＋`StatusPill`：available／downloading／restartPending 為目標版本＋「有新版本」warning；upToDate 為目前版本＋「已是最新」success；checking 為「檢查中」neutral；checkFailed 為「無法檢查」destructive；idle 為「—」＋「尚未檢查」neutral）。
- 卡「下載與安裝」：只在 available／downloading／restartPending／error 出現，一列：標題「Speclink <目標版本>」、說明依狀態（「安裝後會重新啟動」／進度／「更新已就緒」／錯誤訊息）、控制為既有 `UpdateInstallRow` 的按鈕組。
- 卡「更新日誌」：第一列為最新版本——等寬版號＋日期＋依「新功能／修正／改善」分組列點（預設展開）；其後每個較舊版本一列（等寬版號＋日期＋首條摘要），右側 ghost「檢視」開瀏覽模式對話框；底列「全部更新日誌」ghost 鈕同樣開對話框。
- 卡「自動檢查」：一列「啟動時在背景檢查新版本」（說明「只檢查，不會自動下載」）＋`Switch`；偏好 `speclink.updater.autoCheck`（localStorage，預設 true）由 store 持有：為 false 時啟動檢查與前景重檢 SHALL 不發出、手動檢查照常、節流時間戳不受影響。
- `focusUpdater` 旗標改為捲到「下載與安裝」卡（無該卡時捲到「版本」卡）。

### D7 伺服器頁

`ServersPage` 以 `ServersPanel` 的既有 props 重排：

- `SettingsPageHeader`「伺服器」＋右側 secondary「新增連線」（切換頁頂「新增連線」卡：URL、顯示名、新增／取消；新增成功即收起）＋說明 `servers.help`。
- 卡「連線」：每個 connection 一列——左：顯示名＋`StatusPill`（已登入 success／未登入 neutral／登入過期 warning／等待授權 progress／失敗 destructive）、等寬 origin、身分名；右：登出（已登入）或登入／重新登入（未登入／過期，重新登入沿用 reauth 聚焦）；「過期」除 reauth 導向的 `focusConnectionId` 外，也含開著的 remote session 回報 needs-reauth 的連線（`expiredConnectionIds`，不搶焦點）＋「⋯」`DropdownMenu`（開啟專案、移除；移除為警示色）；等待授權與 PAT 輸入沿既有 `AwaitingApproval`／`PatLoginInput` 於列下展開；空清單顯示 `servers.empty`。
- 卡「預設」：一列「新增遠端專案時預設使用」（`SettingsChoice`：已登入連線＋「不預選」）；偏好 `speclink.defaultConnection`（localStorage，存 connection id）由 store 持有；`WorkspaceChooser` 新 prop `defaultConnectionId`，Server 步驟渲染時若該連線存在且已登入則預選（進入 scopes 前仍可改選）；帶入口意圖（`initialConnectionId`、marker 指向的 `initialServerUrl`）開啟時不預選。「存在且已登入」的判定由 `adapter/connections.ts` 的 `defaultConnectionOf` 一處提供，預設卡與對話框共用。
- 卡「登入方式」：列「裝置登入」（說明＋右側灰字「預設」）、列「存取金鑰」（說明：伺服器不支援裝置登入時就地改用）。無按鈕。

### D8 專案設定四頁

`ProjectSettingsView` 拆成 `useProjectSettings(settings)` hook（既有全部 state、載入、儲存、衝突、schema 操作原樣搬入）＋四頁：

- **ContextPage**「專案說明」：`SettingsPageHeader` 說明「注入 AI 指令的專案自由文字說明」、右側既有 `CardEditControls`（編輯／取消＋儲存）、note 為等寬 `openspec/config.yaml · context`（remote 為 policy revision）；內容卡：唯讀 markdown＋收合／顯示更多，編輯為既有 `Textarea`。
- **RulesPage**「產出規則」：同構，每個固定鍵一段（等寬鍵名＋「N 條」）；編輯為既有 `Textarea`。
- **WorkflowPage**「產出流程」：卡「產出政策」五列——locale、spec_locale（`SettingsChoice`，未知值顯性呈現沿既有）、tdd、audit、worktree（`Switch`）、儲存／訊息列；卡「產出流程」：既有 schema 清單（切換下拉、建立、fork、開啟資料夾、刪除確認）與唯讀詳情原樣。
- **SpeclinkPage**「.speclink.yaml」（remote 不可達）：列「檔案」（等寬路徑，唯讀）；卡「AI 工具」：claude／codex／copilot 各一列＋`Switch`，自訂工具描述子為唯讀列。
- 解析失敗：對應頁頂 `ParseErrorBanner`、表單停用（既有）；分類欄警示點見 D2。remote 衝突對照卡維持在各頁頂部（既有元件）。
- 未存修改（品質站修正）：hook 匯出 `dirty`——AI 工具與五個政策欄位對照快照（寫入成功後把已存值併回快照）、兩張文字卡只看編輯中的草稿；`ProjectSettingsView` 以 `onDirtyChange` 回報給 App 的返回動作，卸載時清掉解析失敗與未存修改兩種回報。remote 專案的 .speclink.yaml 退回專案說明只在 App 做一次，頁面照收到的分類渲染。共用片段只收需要的欄位：`ConflictPanel`（conflict、onReload、onRetry）、`PageNote`（hook 匯出的 `remoteNote`：revision 與是否為檢視者）。

### D9 文案與詞彙

- i18n 新增 `settings.section.*`（五個分類名）、`settings.group.app`、`settings.searchPlaceholder`、`settings.noMatch`、`settings.pageNote`、`settings.appearance`、`settings.theme.light|dark|system`、`settings.projectSection.*`、`settings.group.configFile`、`settings.group.project`、`updater.page.*`（latest、pill 五態、autoCheck、autoCheckHint、viewNotes、allNotes、installHint）、`servers.page.*`（addConnection、connectionsCard、defaultsCard、defaultForNew、noDefault、loginCard、deviceLogin、deviceLoginHint、pat、patHint、isDefault）、`cli.page.*`（deployDir）；`settings.localTabNote` 改為頁語意「此頁內容僅存於此裝置，不寫入版本庫」（apply 期以同義的 `settings.pageNote` 承接、舊鍵刪除，不留兩個同義鍵）；外觀三選的「跟隨系統」沿用既有 `settings.followSystem`；`settings.schemaTab`、`settings.localTabLabel`、`settings.serversTabLabel`、`remote.workflowTab` 刪除。品質站修正：「← 專案設定」沿用 `app.navProjectSettings`（原 `settings.backToProject` 同字，刪除）；隨改版孤兒化的 `servers.title`、`settings.rulesLabel` 刪除；新增 `settings.discardTitle`、`settings.discardHint`、`settings.discard`。兩語系鍵集合相等。
- `openspec/LANGUAGE.md`「產出流程」條目：明文例外段改為「已隨設定分類欄取代頁籤而退場（desktop-settings-mode，2026-10-10）」；「專案說明」「產出規則」definition 的「設定頁的編輯區段名」改「專案設定的分類名」。

## Risks / Trade-offs

- **ProjectSettingsView 拆分**：1294 行與 1321 行測試一起搬；hook 抽出不改邏輯，但測試要改以 section 進頁。以逐頁搬移、每頁跑既有案例控風險。
- **深色 token 兩份**：以測試釘住逐行相同，漂移即紅。
- **自動檢查關閉**：使用者可能錯過安全更新；開關說明寫明只影響背景檢查，手動與前景入口仍在。
- **預設連線偏好**：連線被移除或登出後偏好指向無效；chooser 只在存在且已登入時預選，否則忽略。
- **設定模式下零專案的左側 256px**：與零專案空狀態（56px）切換時版面寬度跳動；可接受，設定模式本來就換了左欄。

## Migration Plan

1. D3、D4 先行（原語、零件、主題偏好與 theme.css）。
2. D1、D2 接 store 與分類欄；`AppSettingsView` 先以既有卡包進頁，再逐頁換成零件（D5–D7）。
3. D8 抽 hook、拆四頁；測試逐頁搬。
4. 文案、LANGUAGE.md、守門。

## Open Questions

- 無。
