## Context

桌面 app 的殼目前是：視窗原生標題列（macOS 只有「Speclink」字樣）＋ app 自繪 48px 頂欄（`Wordmark`、`ProjectTabs` 分頁列、「新增 Workspace」鈕）＋ 200px 左側欄（`App.tsx` 本地 `NavItem` 六項：變更、已封存、規格、手冊、專案設定、設定）＋主區。分頁列是 `tabs.ts` 的持久化清單（locator 身分、上限 10、去重、關閉、錯誤態、⌃Tab／⌃1–9 由 `App.tsx` 的 keydown 監聽呼叫 store 的 `cycleTab`／`gotoTab`）。零分頁時主區為 `EmptyState`（圖示＋標題＋說明＋「新增 Workspace」鈕）。「新增 Workspace」是 `WorkspaceChooser.tsx` 的四步 `AlertDialog`（source／server／scopes／checkout），第一步兩張 `ChoiceCard` 點了就動作、下方最近開啟清單（`recents.ts`）。沒有任何檔案系統動作。Tauri 2.11 `tauri.conf.json` 只有一個 main 視窗設定（1440×900 置中），無平台分檔；`capabilities/default.json` 沒有 window 操作權限。平台判定只有 `tray.ts` 的 `detectMacOS()`。server-web 的 `ConsoleLayout.tsx` 抄了 desktop 殼的尺寸（h-12／w-[200px]／p-5），`AdminNav.tsx` 自己寫導覽項樣式（作用中實心主色）。

cut 1 已落地：`theme.css` 有 `sidebar` token、原語扁平化、`BrandMark`／`Wordmark`／`ConfirmDialog`／`EmptyState` 在 packages/ui、`uiSingleSource.test.ts` 守門 apps/* 不得定義與 ui 匯出同名的元件。互動原型（討論結論的連結）第 1、3、16、17、18、20 張畫板是本塊的視覺基準。

約束：分頁資料模型（`tabs.ts`、`recents.ts`、`session.ts`）不動；所有既有切換、探測、錯誤態、快捷鍵行為不變；「開啟專案」的靜默成功語意不變；Windows 無機器可實測（討論 Deferred），自繪視窗鈕以 Tauri window API 的回傳值與 vitest 驗證、實機由 [M] 任務在有 Windows 時補。

## Goals / Non-Goals

**Goals:**

- 原生標題列的空白消失：macOS 紅綠燈落在 app 自己的左側標題列、Windows 自繪拖曳列與視窗鈕、Linux 保留原生列但其下版面一致。
- 左側兩層導覽：圖示列（全域：品牌、專案方塊、＋、設定）＋專案欄（每專案：四頁＋專案設定），零專案時只剩圖示列。
- 專案層檔案系統動作一個入口（標題列「⋯」與方塊右鍵同一份選單），指令組裝可單元測試、跨三平台。
- 主區頂列麵包屑＋頁標題區原語，看板頁先套。
- 新增專案對話框換新外觀並改名，詞彙「Workspace」在使用者可見文案收斂為「專案」。
- 四個新原語與共用 `NavItem` 只在 packages/ui 一份，desktop 與 server-web 都用。

**Non-Goals:**

- 提示歸位（技能檔提示卡、「有新版本」鈕）——desktop-notice-relocation。
- macOS 原生選單——desktop-native-menu。
- 設定模式的分類欄與「設定」標題列——cut 4；本塊設定仍是既有 `AppSettingsView`。
- 看板欄與卡片、搜尋列形狀——cut 3a；詳情頁——cut 3b；系統匣——cut 5。
- 分頁資料模型與最近開啟記錄的格式。
- 專案欄可收合、專案名下拉、第二欄專案清單——討論已否決。

## Decisions

### D1 三平台標題列：平台分檔 + 兩條拖曳列

- `apps/desktop/src-tauri/tauri.macos.conf.json`：`{"app":{"windows":[{"label":"main","titleBarStyle":"Overlay","hiddenTitle":true,"trafficLightPosition":{"x":16,"y":26}}]}}`——Tauri 依平台把它合併進 `tauri.conf.json`，`decorations` 維持預設 true。`apps/desktop/src-tauri/tauri.windows.conf.json`：`{"app":{"windows":[{"label":"main","decorations":false,"shadow":true}]}}`（shadow 保留 Win11 圓角與陰影；代價是失去貼齊版面浮窗，討論已接受）。Linux 不加分檔。Tauri 以 JSON Merge Patch 合併平台檔，陣列整個取代不逐項合併，所以兩個覆蓋檔的 main 視窗都重寫 `title`、`width`、`height`、`center`，與 `tauri.conf.json` 同值。
- 前端新增 `apps/desktop/src/platform.ts`：`detectPlatform(ua = navigator.userAgent): "macos" | "windows" | "linux"`（Macintosh／Mac OS → macos、Windows → windows、其餘 linux）；`tray.ts` 的 `detectMacOS` 改為 `detectPlatform() === "macos"`（舊名保留，`tray.test.ts` 不改）。
- 兩條 48px 頂列都帶 `data-tauri-drag-region`：左側標題列（圖示列＋專案欄共用，`bg-sidebar`）與主區頂列（`bg-background`、底緣細線）。拖曳列內的可點元素（專案名、⋯、麵包屑連結、視窗鈕）不帶該屬性，照常接收點擊。`capabilities/default.json` 加 `core:window:allow-start-dragging`、`core:window:allow-minimize`、`core:window:allow-toggle-maximize`、`core:window:allow-close`、`core:window:allow-is-maximized`。
- 內容起點：三平台一律 `pl-[98px]`——專案名左緣對齊下方專案欄導覽文字（圖示列 56＋專案欄內距 8＋導覽項內距 10＋圖示 16＋間距 8），macOS 紅綠燈落在其左側留白。左側標題列下緣 `border-b`，與主區頂列的細線同高，頂列成為橫貫視窗的一條帶，圖示列分隔線從這條線往下（實機驗收時發現：沒有這條線時，分隔線會從紅綠燈下方往上指進燈組）。
- 紅綠燈的 y：tao 的 `inset_traffic_lights` 把標題列容器高度設為「按鈕高度＋y」，按鈕保持原本離容器底部的距離，所以 y 不是按鈕頂端；實測 y=17 時燈的中心約在 15.5px，y=26 才與 48px 列內的文字垂直置中。
- Windows 視窗鈕：`apps/desktop/src/components/TitleBar.tsx` 匯出的 `WindowControls`（只在 `detectPlatform() === "windows"` 渲染）三顆 46×48 無底按鈕：縮到最小（`getCurrentWindow().minimize()`）、最大化／還原（`toggleMaximize()`，圖示依 `isMaximized()` 與 `onResized` 事件切換）、關閉（`close()`）；hover `bg-muted`，關閉 hover `bg-destructive text-white`；aria-label 走 i18n（`titlebar.minimize`／`titlebar.maximize`／`titlebar.restore`／`titlebar.close`）。
- Linux：原生標題列照舊；左側標題列與主區頂列仍 48px、專案名左緣同樣對齊導覽文字。

替代方案「Windows 維持原生標題列」在討論中否決（空白條還在）。

### D2 外殼結構與各模式下的左側

`App.tsx` 的殼改為：

```
<div class="flex h-screen">
  <div class="flex w-[256px] shrink-0 flex-col bg-sidebar border-r">        ← 左側（零專案時 w-14：只剩圖示列）
    <TitleBar.Left>  48px drag：紅綠燈留白 ＋ 專案名(truncate, title=路徑) ＋ ⋯ </TitleBar.Left>
    <div class="flex flex-1 min-h-0">
      <ProjectRail/>      ← 56px，右緣細線（只在標題列以下）
      <ProjectColumn/>    ← 200px（零專案時不渲染）
    </div>
  </div>
  <main class="flex flex-1 min-w-0 flex-col">
    <TitleBar.Main> 48px drag：麵包屑 ＋ (Windows) WindowControls </TitleBar.Main>
    …既有主區內容（技能檔提示、設定頁、空狀態、復原頁、專案設定、手冊、規格、看板）…
  </main>
</div>
```

- 左側標題列的內容依模式：有作用中專案＝專案名＋「⋯」；設定模式（`boardView === "settings"`）＝文字「設定」、無「⋯」；零專案＝空（只有紅綠燈留白）——零專案的設定模式同樣留白：左側只有 56px，放不下文字，macOS 紅綠燈也佔住這一段，「設定」只出現在主區麵包屑。`App.tsx` 以 `projectTab`（設定模式時為空的作用中分頁）一處導出專案名、路徑提示、「⋯」與麵包屑。專案設定模式本塊維持專案名＋「⋯」（「← 專案設定」是 cut 4）。
- 麵包屑 `TitleBar.Main` 內容：`<專案名> / <頁名>`（頁名＝變更、已封存、規格、手冊、專案設定）；設定模式＝`設定`；零專案＝空。文字 12px 灰字、最後一段 `text-foreground`、分隔「/」半透明；專案名可點＝回看板（`setBoardView("board")`）。
- 設定模式的專案欄本塊不換（cut 4 換成分類欄）：四頁＋專案設定維持渲染、無作用中項；圖示列齒輪為作用中。零專案且設定模式：圖示列齒輪作用中、主區為設定頁（既有「零分頁仍可進入應用程式設定」語意）。
- 既有 `header`（字標、`ProjectTabs`、「新增 Workspace」鈕）與 `aside` 整段移除；`Wordmark` 不再出現在殼上（只在零專案空狀態）。remote stale 橫幅（`remote-stale-banner`）、`UpdateBanner`、技能檔提示包裹層本塊保留：`UpdateBanner` 移到主區頂列之下（頂列要與左側標題列同高、Windows 的自繪視窗鈕要留在右上角），其餘原位——提示的歸位是 desktop-notice-relocation。

### D3 圖示列 `ProjectRail`

`apps/desktop/src/components/ProjectRail.tsx`，取代 `ProjectTabs.tsx`，props 沿用 `ProjectTabsProps` 的同名同義項（`tabs`、`activeKey`、`tabErrors`、`recoveryStates`、`connectionStates`、`connectionNames`、`pendingKey`、`onActivate`、`onOpen`），另加 `onOpenSettings`、`settingsActive`、`platform` 與 `actions`（右鍵選單的六個動作，含關閉；見 D5）。

- 版面：`w-14 flex flex-col items-center`，標題列下方 `pt-2`；`BrandMark size={26}` 下留 `mb-3.5`；方塊之間 `gap-2`；底部 `mt-auto` 放齒輪，`mb-3`。
- 專案方塊：32×32 `rounded-lg` 按鈕，內容為顯示名第一個字元大寫（CJK 取第一個字）；作用中 `bg-primary text-primary-foreground`，其餘 `bg-muted text-muted-foreground hover:bg-muted/70`；`role="tab"`、`aria-selected`、`aria-label` 為顯示名（錯誤態附錯誤訊息、切換中附「正在切換」）；`Tooltip` 內容兩行：顯示名＋路徑（remote 顯示 checkout 路徑或 locator key，沿用 `ProjectTabs` 的 tooltip 規則）；容器 `role="tablist" aria-label=t("app.projectRail")`，`data-project-rail`。
- 狀態點（右下角 8px 圓點、`border-2 border-sidebar`）：`tabErrors[key]` 或 recovery 錯誤＝`bg-destructive`；remote `offline`／`needs-reauth`＝`bg-status-warning`；探測中（`pendingKey === key`）＝字母換成 `LoaderCircle` 旋轉圖示；restoring 同探測中。既有 `ProjectTabs` 的狀態對應表逐條搬過來，aria-label 文案鍵不變。
- 錯誤態點擊：不切換；開啟右鍵同一份選單（D5）的錯誤變體——首列為不可點的錯誤訊息，之後只有「自專案列移除」（`onClose`）。右鍵同樣彈出錯誤變體：`ContextMenuTrigger` 直接包住方塊按鈕（包在 `DropdownMenu` 這個非 DOM 的 Root 外面時，Slot 傳下的 `onContextMenu` 會被丟掉）。
- 「＋」：32×32 虛線框方塊（`border border-dashed border-border bg-transparent`）、aria-label「新增專案」、點擊 `onOpen`。
- 齒輪：`rail-btn` 同尺寸、`bg-transparent text-muted-foreground`，作用中 `bg-primary text-primary-foreground`；aria-label「設定」。
- 鍵盤：`App.tsx` 既有 keydown 監聽（⌃Tab、⌃1–9）不變，另加專案動作選單標示的快捷鍵：⌘R（macOS）／Ctrl+R（Windows、Linux）＝重新整理作用中專案，Ctrl+W（Windows、Linux）＝關閉作用中專案。macOS 的 ⌘W 先被系統預設選單攔走（關閉視窗），網頁層收不到，由 desktop-native-menu 的原生選單承接。
- 不顯示任何計數徽章（spec）。

### D4 專案欄 `ProjectColumn` 與共用 `NavItem`

- packages/ui 新增領域元件 `NavItem`（`packages/ui/src/components/NavItem.tsx`）：`NavItem({ icon, label, active, count?, ariaLabel?, onClick?, asChild?, className? })`——32px 高、`rounded-lg px-2.5 gap-2 text-[13px]`，作用中 `bg-primary/12 text-primary font-medium`，其餘 `text-foreground hover:bg-foreground/5`；`count` 以全圓 `bg-muted text-muted-foreground text-[11px]` 徽章渲染在右端；`asChild` 走 Radix Slot 讓 server-web 以 `NavLink` 當子元素。`App.tsx` 本地 `NavItem` 刪除（`uiSingleSource.test.ts` 會擋住同名本地定義）。
- `apps/desktop/src/components/ProjectColumn.tsx`：`w-[200px] shrink-0 flex flex-col p-2 pb-3`，四個 `NavItem`（變更 `GitBranch`、已封存 `Archive`＋`count={archived.length}`、規格 `FileText`、手冊 `BookOpen`）＋ `mt-auto` 的專案設定 `SlidersHorizontal`；active 由 `boardView` 決定、設定模式下全部非作用中；點擊即 `setBoardView`（切頁語意、無 toggle，aria-label 與 i18n 鍵沿用 `app.navChanges`／`app.archived`／`app.navSpecs`／`app.navManual`／`app.navProjectSettings`）。
- server-web：`AdminNav.tsx` 的 `itemClass` 刪除，改 `<NavItem asChild icon label active={isActive}><NavLink …/></NavItem>`（`NavLink` 的 `className` 函式提供 `isActive`）；`ConsoleLayout.tsx` 側欄 `bg-card` 改 `bg-sidebar`、保留 w-[200px] 與 p-2；頂列不動。`admin-console-shell.test.tsx` 的作用中斷言從實心主色改為 `text-primary`。

### D5 專案動作選單與檔案系統動作

- packages/ui 新增原語：`components/ui/dropdown-menu.tsx`（Radix `@radix-ui/react-dropdown-menu`；`DropdownMenuContent` ＝ `min-w-56 rounded-xl border bg-card p-1.5 shadow-lg shadow-black/5`，`DropdownMenuItem` ＝ `flex h-8 items-center gap-2.5 rounded-lg px-2.5 text-[13px] data-[highlighted]:bg-muted`，`variant="destructive"` ＝ `text-destructive data-[highlighted]:bg-destructive/10`，`DropdownMenuSeparator` ＝ `my-1.5 mx-2 h-px bg-border`）與 `components/ui/context-menu.tsx`（Radix `@radix-ui/react-context-menu`，Content／Item／Separator 共用同一組 class 常數 `menuContentClass`／`menuItemClass`，從 dropdown-menu.tsx 匯出給 context-menu.tsx 用）；`components/ui/kbd.tsx`：`Kbd` ＝ `<kbd class="font-mono text-[11px] text-muted-foreground tracking-wide">`，`MenuShortcut` ＝ 右端對齊（`ml-auto`）的 `Kbd`，兩套選單共用的快捷鍵槽。`packages/ui/package.json` 加兩個 Radix 相依。
- `apps/desktop/src/components/ProjectActionsMenu.tsx`：純內容元件 `ProjectActionItems({ menu, tab, platform, actions, error? })`（`actions` 的六個動作都帶該專案的 locator key），由 `DropdownMenu`（標題列「⋯」）與 `ContextMenu`（圖示列方塊右鍵）各自包一層渲染同一份項目：
  - 本機專案：在 {Finder｜檔案總管｜檔案管理員} 顯示、在終端機開啟、以編輯器開啟、複製路徑（右端等寬灰字顯示路徑尾段）、分隔線、重新整理（`MenuShortcut` ⌘R／Ctrl+R）、關閉專案（destructive，`MenuShortcut` ⌘W／Ctrl+W）。
  - remote 專案（`locator.kind === "remote"`）：有 `checkoutRoot` 時四個檔案系統項目作用於 checkout 路徑；無 checkout 時只有重新整理與關閉專案。
  - 錯誤變體（`error` 非空）：首列 `DropdownMenuLabel` 顯示錯誤訊息（`text-destructive`），之後只有「自專案列移除」。
  - 檔案管理員名稱鍵：`fs.revealIn.macos`／`fs.revealIn.windows`／`fs.revealIn.linux`（zh：在 Finder 顯示／在檔案總管顯示／在檔案管理員顯示）。
- 指令組裝落在 speclink-desktop-core：`apps/desktop/core/src/fs_actions.rs` 匯出 `pub enum Platform { MacOs, Windows, Linux }`、`pub struct Spawn { pub program: String, pub args: Vec<String> }`、`pub fn terminal_candidates(platform, path) -> Vec<Spawn>`（macOS：`open -a Terminal <path>`；Windows：`wt -d <path>`，再 `cmd /c start cmd /K cd /d <path>`；Linux：`x-terminal-emulator --working-directory=<path>`、`gnome-terminal --working-directory=<path>`、`konsole --workdir <path>`、`xfce4-terminal --working-directory=<path>`）、`pub fn editor_candidates(platform, path) -> Vec<Spawn>`（`code`、`cursor`、`zed`、`subl` 各帶 `<path>`；Windows 找 PATH 只補 `.exe`，VS Code 與 Cursor 在 PATH 上的入口是 `.cmd`，所以 Windows 寫 `code.cmd`、`cursor.cmd`）；Windows 的路徑會經過殼層命令列解析，含該殼層語法字元的候選跳過（cmd：`& | < > ^ % "`；wt：`;` 是子指令分隔），免得資料夾名被拆成另一個指令；`#[cfg(test)]` 逐平台斷言 program 與 args。
- 依序啟動也在 core：`pub fn launch_first(dir, candidates, path_env, none_found) -> Result<(), String>`——`dir` 不是目錄時先回 `Err("fs.dirMissing")`、一個候選都不試；否則依序 spawn（stdin／stdout／stderr 接 null，Windows 加 `CREATE_NO_WINDOW` 免得 `cmd` 與 `.cmd` 包裝先閃主控台視窗），每個候選啟動後最多等 0.5 秒：期間以非零碼結束（如終端機不認得工作目錄參數、立刻退出）視同失敗、改試下一個；以零結束或仍在執行即算成功，仍在執行的子程序另起執行緒回收；全部失敗回 `Err(none_found)`。`Platform::current()` 依編譯目標決定平台。`#[cfg(unix)]` 測試以 `sh -c` 候選斷言跳過失敗者、成功即止、全失敗回鍵、目錄不存在不 spawn、長駐子程序不阻塞。
- Tauri command（`apps/desktop/src-tauri/src/fs_actions.rs`，`lib.rs` 的 `generate_handler!` 註冊）：「在檔案管理員顯示」沿用既有的 `reveal_in_folder(path)`（已是 `tauri_plugin_opener` 的 `reveal_item_in_dir`，產出流程面板也在用），不另開同功能 command；`open_in_terminal(path)` 與 `open_in_editor(path)` 只委派：在 `spawn_blocking` 裡呼叫 `launch_first(path, terminal_candidates(Platform::current(), path)／editor_candidates(Platform::current(), path), user_shell_path(), "fs.noTerminal"／"fs.noEditor")`。錯誤鍵由前端查 i18n 顯示 toast（zh：「找不到可用的終端機」／「找不到可用的編輯器（支援 VS Code、Cursor、Zed、Sublime）」／「專案目錄已不存在」）。PATH 帶使用者 login shell 的值（沿用 `cli_install::user_shell_path`）——macOS 從 Dock 啟動的 app 只有系統預設 PATH，找不到 Homebrew 下的 `code` 等指令。reveal 的路徑不存在由 opener 報錯、前端 toast 原文。
- 前端 adapter `apps/desktop/src/adapter/fsActions.ts`：`FsActionsAdapter { reveal, openTerminal, openEditor, copyPath }` 與 `tauriFsActionsAdapter()`（三個 `invoke` 包裝＋`@tauri-apps/plugin-clipboard-manager` 的 `writeText`），照既有 adapter 慣例由 `main.tsx` 經 `App` 的 props 注入 `AppStoreDeps.fsActions`（未注入時四個動作不動作，測試注入假 adapter）；同檔匯出 `projectDirOf(locator)`——local 為專案根、remote 為 checkout、remote 無 checkout 為 null，store 與選單共用這條規則。`store.ts` 新增 `revealProject(key)`、`openProjectInTerminal(key)`、`openProjectInEditor(key)`、`copyProjectPath(key)`；關閉 ＝ 既有 `closeTab(key)`；重新整理：作用中專案＝既有 `refresh()`，背景專案＝`activateTab(key)`（背景專案沒有載入中的資料可重整，切過去即重新載入）。成功一律靜默（spec「看板全域操作成功靜默」的同一語意），失敗以 `toast.error` 顯示錯誤文字。

替代方案「文件另存為」「專案名帶 ▾ 下拉」在討論中否決。

### D6 主區頂列麵包屑與 `PageHeader`

- packages/ui 新增原語 `components/ui/page-header.tsx`：`PageHeader({ title, description?, actions?, className? })` ＝ `flex items-end justify-between gap-3`，標題 `h2 text-2xl font-normal`、說明 `text-muted-foreground text-[13px] mt-0.5`、動作槽 `flex items-center gap-1.5 shrink-0`。
- `KanbanBoard` 新增選配 props `title?: string`、`description?: string`：提供時在搜尋列原位置改渲染 `<PageHeader title description actions={<BoardSearchBar …/>} />`，搜尋與篩選仍是欄位上方的同一列、同高；不提供時行為與現狀相同（server-web 與既有測試不受影響）。`App.tsx` 傳 `title={t("app.navChanges")}`、`description={t("board.pageDesc")}`。
- 麵包屑實作在 `TitleBar.tsx` 的 `MainTitleBar({ crumbs: Array<{ label, onClick? }>, platform })`；`App.tsx` 依 `boardView` 組 crumbs。

### D7 零專案空狀態 `EmptyWorkspace` 與共用 `RecentList`

- `apps/desktop/src/components/RecentList.tsx`：自 `WorkspaceChooser.tsx` 第一步抽出的最近開啟清單（條目渲染、錯誤態、連線狀態判定、移除鈕），props `entries`、`errors`、`connections`、`onOpen(entry)`、`onRemove(entry)`、`variant: "dialog" | "card"`；兩個 variant 只差外框（dialog ＝ 12px 細框卡、card ＝ 同款但含 14px 小標「最近開啟」＋計數）。清單判定（過濾已開分頁、連線已移除／已登出）維持在既有 `recents.ts`／chooser 的函式，不複製。開啟流程抽成同檔的 `useRecentOpener`：持有錯誤態、忙碌旗標與「連線清單是否讀取成功」（`connectionsReady`），`reset()` 清錯誤態並重讀連線清單（對話框每次打開、空狀態掛載時各呼叫一次），`refreshConnections()` 只重讀（加 server 後）。點擊開啟失敗的列仍可再點重試，重試開始時清掉該列的錯誤；連線已移除／已登出由連線清單判定，才停用開啟。
- `apps/desktop/src/components/EmptyWorkspace.tsx`：主區置中、寬 420px 直欄、`gap-4`：`<Wordmark className="h-10" />`、`h2 text-2xl font-normal`「開啟一個專案開始」、灰字說明（`app.emptyDesc` 改寫為含初始化提示的一句）、按鈕列「新增專案」（default）＋「連線 Server」（outline，`openWorkspaceChooser({ initialStep: "server" })`）、`RecentList variant="card"`（空則不渲染）、底部一句灰字「也可以用終端機：speclink init 之後再從這裡開啟」。取代 `App.tsx` 的 `EmptyState` 用法；`EmptyState` 原語留給其他空態。
- `WorkspaceChooserIntent` 新增 `initialStep?: "server"`：有則起始步驟為 server（與既有 `initialConnectionId`／`initialServerUrl` 的判定合併為 `initialStep ?? (有連線參數 ? "server" : "source")`）。

### D8 新增專案對話框換皮與改名

- 第一步：兩張 `ChoiceCard` 改為可選取（`aria-pressed`、選定 `border-primary bg-primary/5`），點卡只選取；頁尾主要鈕依選取：本機 →「選擇資料夾…」（呼叫既有 `chooseLocal`）、Server →「下一步」（`setStep("server")`）、未選取時停用；「取消」為 ghost。既有 `localProject` 分支（資料夾已是專案／需初始化／帶 remote marker 的分流）的卡片與鈕文案不變。
- 最近開啟改用 `RecentList variant="dialog"`。
- 步驟 2–4：Server 清單、Project／Repo 兩層清單、checkout 的「連接本機 checkout／略過」改為單選列（`role="radio"`、整列可點、選中 `bg-primary/5` 左側圓點），資料與事件處理不變。
- 標題鍵 `chooser.sourceTitle` 改「新增專案」、`chooser.open` 改「開啟專案」、`app.workspaceTabs` 改「專案列」（鍵名不改）；`app.removeTab`→「自專案列移除」；兩個鍵名本身含「Workspace」、會讓 5.2 的 grep 驗證命中，改名為 `app.addWorkspace`→`app.newProject`（「新增專案」）、`servers.openWorkspace`→`servers.openProject`（「開啟專案」）；`app.closeTab` 只有退場的 `ProjectTabs` 在用，隨之刪除（選單用 `fs.closeProject`），同理刪除 `chooser.recentTitle`（共用清單改用 `recent.title`）與 `app.currentProject`（舊頂欄佔位）；`chooser.conflictTitle`「同時偵測到本機與 Remote Workspace」→「同時偵測到本機與遠端專案」；`tray.recovery.*`、`remote.recovery.*`、`servers.openWorkspace` 的「Workspace」全改「專案」；en 同步（Workspace → project）。`messages.test.ts` 兩語系鍵集合不變。
- 外觀：`AlertDialogContent` 已是 12px 圓角與 30% 遮罩（cut 1）；對話框 `p-5 max-w-xl`、標題 15px 中粗、說明灰字；步驟條維持四格。

### D9 詞彙與文件

- `openspec/LANGUAGE.md` 新增詞條「專案（開啟的工作目錄）」：definition＝桌面 app 開著的一個工作目錄——本機資料夾或 server 上的 Project／Repo；圖示列一個方塊＝一個專案、專案欄的四頁與專案設定都屬於它；avoid＝Workspace、workspace、工作區、專案分頁（使用者可見文案中）；why＝「新增 Workspace」「加入專案」「專案分頁」三處指同一件事（討論 desktop-ui-redesign 第 n 回合「詞彙漂移」），依「一個概念一個詞」收斂，且與 server 端的 Project 詞義分層（server 的 Project 是 scope 的上層分組，中文一律寫「Project／Repo」）。「Server」詞條定義文中「新增 Workspace」改「新增專案」。
- docs 四份（`docs/product-status.md`、`docs/product-status.zh-TW.md`、`docs/remote-getting-started.md`、`docs/remote-getting-started.zh-TW.md`）的「Workspace」依語言改「專案」／「project」；`scripts/docs/vocabulary-guard.test.mjs` 讀 LANGUAGE.md 的 avoid 欄掃 docs 與 i18n，新 avoid 詞「工作區」「專案分頁」入機械守門，desktop i18n 的 4 處「工作區」同批改掉；ASCII 的 Workspace 不入機械守門但同批改完——含小寫 workspace 與指專案分頁的「分頁」，以及 `adapter/workspace.ts` 四則遠端錯誤訊息的「遠端工作區」；docs 兩處連結目標 `WorkspaceChooser.tsx` 是檔名、不是文案，grep 驗證時這兩處命中屬預期。

### D10 server-web 殼同步的範圍

只改側欄底色與導覽項樣式（D4），頂列、Sheet 版導覽、`HeaderAccount` 不動；server-web 沒有專案與圖示列概念，不加標題列。

## Implementation Contract

**Behavior**

- macOS：視窗頂端沒有原生標題列文字；紅綠燈位於左側標題列 x=16；拖曳左側標題列或主區頂列可移動視窗。Windows：無原生標題列，右上三顆視窗鈕可縮到最小、最大化／還原、關閉；拖曳頂列可移動視窗。Linux：原生標題列照舊，其下版面與 macOS 同。
- 有專案時：左側標題列顯示作用中專案名（hover 顯示完整路徑）與「⋯」；圖示列由上而下為品牌標記、每個開著的專案一個方塊（作用中 teal 底）、「＋」、底部齒輪；專案欄為變更、已封存（計數）、規格、手冊與底部專案設定；主區頂列顯示「專案名 / 頁名」。
- 零專案時：左側只剩圖示列（品牌標記、＋、齒輪），主區置中顯示字標、標題、說明、「新增專案」「連線 Server」與最近開啟卡。
- 設定模式：左側標題列顯示「設定」（零專案時留白，「設定」只在麵包屑），齒輪作用中，專案欄無作用中項，主區為既有應用程式設定頁。
- 專案方塊：點擊＝切換（探測中顯示 spinner、原畫面可互動）；錯誤態＝角落紅點、點擊開錯誤選單（訊息＋自專案列移除）；remote 離線／需重新登入＝角落琥珀點；右鍵＝專案動作選單（錯誤態為錯誤變體）；hover tooltip＝顯示名與路徑；⌃Tab 與 ⌃1–9 不變；上限 10、去重、關閉、持久化不變。
- 專案動作選單（標題列「⋯」與方塊右鍵同一份）：在 Finder 顯示／在終端機開啟／以編輯器開啟／複製路徑／重新整理／關閉專案；remote 無 checkout 時只剩後兩項；檔案管理員名稱隨平台；終端機與編輯器依 D5 的候選順序啟動，啟動後立即失敗退出的候選改試下一個，全部失敗 toast 說明，專案目錄已不存在時 toast 說明；背景專案的「重新整理」切到該專案；⌘R／Ctrl+R 與 Windows、Linux 的 Ctrl+W 觸發同一動作；成功靜默。
- 看板頁頂端為頁標題「變更」＋說明，搜尋與篩選在同一列右端，仍在欄位上方。
- 新增專案對話框：標題「新增專案」；第一步選卡後主要鈕才可按；最近開啟列有 × 與「遠端」籤；步驟 2–4 為單選列；四步語意、探測、錯誤態與既有一致。
- 使用者可見文案不再出現「Workspace」「工作區」「專案分頁」。
- server-web 管理面側欄為淡灰底、導覽項作用中為 teal 淡底 teal 字。

**Interface / data shape**

- 新元件：`ProjectRail`（props 見 D3）、`ProjectColumn({ boardView, archivedCount, onNavigate })`、`TitleBar.Left({ title?, path?, menu? })`、`TitleBar.Main({ platform, crumbs })`、`WindowControls`、`ProjectActionItems`、`RecentList`、`EmptyWorkspace`。
- `@speclink/ui` 新匯出：`DropdownMenu`、`DropdownMenuTrigger`、`DropdownMenuContent`、`DropdownMenuItem`、`DropdownMenuLabel`、`DropdownMenuSeparator`、`ContextMenu`、`ContextMenuTrigger`、`ContextMenuContent`、`ContextMenuItem`、`ContextMenuLabel`、`ContextMenuSeparator`、`Kbd`、`MenuShortcut`、`PageHeader`、`NavItem`；`KanbanBoardProps` 新增 `title?`、`description?`。
- Tauri commands：沿用 `reveal_in_folder(path: String) -> Result<(), String>`；新增 `open_in_terminal(path: String) -> Result<(), String>`、`open_in_editor(path: String) -> Result<(), String>`；錯誤字串為 i18n 鍵（`fs.noTerminal`、`fs.noEditor`、`fs.dirMissing`）或 opener 的原始訊息。
- core：`speclink_desktop_core::fs_actions::{Platform, Spawn, terminal_candidates, editor_candidates, launch_first}`。
- store：`WorkspaceChooserIntent.initialStep?: "server"`；`AppStoreDeps.fsActions?: FsActionsAdapter`（`AppProps` 同名）；新動作 `revealProject`、`openProjectInTerminal`、`openProjectInEditor`、`copyProjectPath`。
- i18n 新鍵（兩語系同步）：`app.projectRail`、`app.newProject`（`app.addWorkspace` 改名）、`servers.openProject`（`servers.openWorkspace` 改名）、`app.projectColumn`（專案欄的「專案導覽」標籤）、`app.projectActions`（「⋯」的「專案動作」標籤）、`app.settingsTitle`、`app.connectServer`、`app.emptyCli`、`board.pageDesc`、`titlebar.minimize`、`titlebar.maximize`、`titlebar.restore`、`titlebar.close`、`fs.revealIn.macos`、`fs.revealIn.windows`、`fs.revealIn.linux`、`fs.openTerminal`、`fs.openEditor`、`fs.copyPath`、`fs.refresh`、`fs.closeProject`、`fs.noTerminal`、`fs.noEditor`、`fs.dirMissing`、`chooser.chooseFolder`、`chooser.selectSource`、`chooser.remoteTag`、`recent.title`。
- 設定檔：`tauri.macos.conf.json`、`tauri.windows.conf.json` 兩個平台覆蓋檔；`capabilities/default.json` 加五條 `core:window:*` 權限。
- 持久化鍵與格式不變。

**Failure modes**

- 終端機／編輯器候選全部失敗（spawn 失敗或啟動後 0.5 秒內以非零碼結束）：`Err("fs.noTerminal")`／`Err("fs.noEditor")`，前端 toast 對應文案；部分候選失敗、後續成功：靜默。
- 專案目錄不存在（如 remote checkout 已刪除）：`Err("fs.dirMissing")`，toast「專案目錄已不存在」，不試任何候選。
- reveal 失敗（路徑不存在）：opener 錯誤原文 toast。
- 複製路徑失敗：toast 錯誤，不改畫面。
- Windows 視窗 API 呼叫被拒（權限缺）：按鈕無效並 console.error；由 capabilities 任務的測試守住權限清單。
- 專案方塊路徑失效：既有探測失敗流程（錯誤態、不切換）。
- `detectPlatform` 在非瀏覽器環境：回 `linux`（測試環境預設）。

**Acceptance criteria**

- `npm test -w apps/desktop`、`npm test -w packages/ui`、`npm test -w apps/server-web` 全綠；`cargo test -p speclink-desktop-core fs_actions` 綠。
- `apps/desktop/src/__tests__/projectRail.test.tsx` 涵蓋：方塊順序與作用中、tooltip 內容、錯誤紅點與錯誤選單、remote 琥珀點、探測中 spinner、「＋」與齒輪、無計數徽章。
- `titleBar.test.tsx`：左側標題列 `pl-[98px]` 與下緣細線、macOS 無視窗鈕；Windows 三顆視窗鈕呼叫對應 window API（mock `@tauri-apps/api/window`）；麵包屑段落與專案名可點。
- `projectActionsMenu.test.tsx`：本機六項、remote 無 checkout 兩項、錯誤變體、平台名稱、各項以該專案的 key 呼叫對應 callback。
- `emptyWorkspace.test.tsx`：字標 alt、兩鈕、最近開啟卡存在／為空不渲染。
- `App.test.tsx`：零分頁時無專案欄、有分頁時專案欄五項且設定不在其中、設定模式左側標題列為「設定」（零專案時留白）、背景方塊右鍵重新整理切到該專案、Ctrl+R／Ctrl+W 與 macOS ⌘R、更新通知列在主區頂列之下、⌃Tab 與 ⌃1–9 既有案例通過。
- `workspaceChooser.test.tsx`：標題「新增專案」、選卡後主要鈕可按、未選停用、既有流程案例不改語意通過。
- `grep -rn "Workspace\|工作區\|專案分頁" apps/desktop/src/i18n/messages.ts docs/*.md` 零命中；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠。
- `uiSingleSource.test.ts` 綠（desktop 無本地 `NavItem`）。
- 手動：macOS 實機看紅綠燈位置、拖曳、三個檔案系統動作各一次。

**Scope boundaries**

- In：`App.tsx` 殼、六個新殼元件、`ProjectTabs` 退場、store 四個動作與 chooser intent、`platform.ts`、chooser 換皮改名、i18n 兩語系、LANGUAGE.md 與 docs 四份、Tauri 兩個平台覆蓋檔與權限、core `fs_actions.rs` 與 Tauri 三個 command、packages/ui 四個原語＋`NavItem`＋`KanbanBoard` 兩個 props、server-web 側欄樣式、對應測試。
- Out：提示歸位、原生選單、設定模式分類欄、看板欄／卡片／搜尋列形狀、詳情頁、系統匣、分頁資料模型、Linux 標題列、編輯器可設定（Deferred：cut 4 設定頁再議）。

## Risks / Trade-offs

- [回歸對照] → 不動 CLI、golden、`--json`；前端三個 workspace 的既有測試是基準，`projectTabs.test.tsx` 的案例逐條搬到 `projectRail.test.tsx`，刪舊檔前先確認案例數相等。
- [Windows 無機器實測自繪視窗鈕與 `decorations:false` 的拖曳] → window API 以 vitest mock 驗證呼叫；[M] 任務標明「有 Windows 機器時補」，不擋封存（討論 Deferred）。
- [macOS `Overlay` 隱藏標題後，雙擊標題列縮放與全螢幕行為] → `data-tauri-drag-region` 自帶雙擊縮放；全螢幕時紅綠燈由系統接管、`pl-[98px]` 留白維持（可接受）。
- [`AlertDialog` 當作四步對話框的既有做法] → 不換成 Dialog，避免動到 focus 與 Esc 語意；只換內容外觀。
- [`KanbanBoard` 加 `title`／`description` 讓 server-web 與 desktop 分岔] → 選配 props、預設行為不變；server-web 不傳即維持現狀。
- [新 avoid 詞「工作區」「專案分頁」入機械守門會照出存量] → 本批掃描面只有 desktop i18n 4 處「工作區」、docs 0 處；同批改掉；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 為驗證。
- [編輯器候選清單寫死四個] → 討論未定；先給常見四個，找不到時 toast 明說支援清單；可設定項列為 Deferred。清單與兩語系文案三處同步，`store.test.ts` 以 `?raw` 讀 Rust 清單比對文案，漏改任一處即紅。
- [終端機 spawn 在 Linux 依發行版差異] → 四個候選依序嘗試；`x-terminal-emulator` 指向不認得 `--working-directory` 的終端機時會立刻退出，0.5 秒的提早退出判定讓它改試下一個；全失敗才報錯；不讀子程序輸出。
- [Windows 的 `.cmd` 名稱與 `CREATE_NO_WINDOW`] → 本機無 Windows 編譯目標，`#[cfg(windows)]` 區塊由 Windows CI 編譯；實機行為由 [M] 任務在有 Windows 時補看。
- [跨平台] → `detectPlatform` 以 UA 判定、三個分支各有測試；Rust 候選表三平台各有單元測試。
- [`ContextMenu` 與 `DropdownMenu` 兩套 Radix] → 共用同一組 class 常數，樣式只寫一次；兩者各只有一個消費者，皆有消費者（D5）。

## Migration Plan

無資料遷移：`speclink.projectTabs` 與 `speclink.recentWorkspaces` 格式不變。發版隨桌面 app 一般發版；回滾即還原本刀 commit（平台覆蓋檔一併移除）。

## Open Questions

無。
