## Summary

把桌面 app（apps/desktop）的視窗殼換成新結構：隱藏原生標題列後由 app 自己畫三平台標題列，左側是「圖示列（品牌標記、每個專案一個方塊、＋、設定）＋固定寬專案欄（變更、已封存、規格、手冊、專案設定）」，左側頂端 48px 是一條連續的標題列承載紅綠燈、純文字專案名與「⋯」檔案系統動作選單，主區頂端 48px 放麵包屑（Windows 時右端自繪三顆視窗鈕），主區內容套用頁標題區（看板頁先套）；零專案時圖示列只剩品牌標記、＋、設定，主區為「開啟一個專案開始」空狀態；「新增 Workspace」對話框換皮並改名「新增專案」，詞彙「Workspace」收斂為「專案」。這是討論 desktop-ui-redesign 的 cut 2 外殼刀的第一塊（外殼結構）；同一刀另拆兩塊：提示歸位（desktop-notice-relocation）與 macOS 原生選單（desktop-native-menu）。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是桌面 app 的每一個畫面——殼是所有頁面共用的外框。四個問題：

- 視窗頂端疊了兩條：原生標題列（只有「Speclink」四個字）＋ app 自己的 48px 頂欄（字標、專案分頁列、「新增 Workspace」），一片空白佔掉近 80px；Windows／Linux 同樣多一條。
- 左側欄把「屬於專案的頁」（變更、已封存、規格、手冊、專案設定）與「全域的設定」排在同一欄，分頁列又在頂欄，看不出「頁面屬於專案、專案之間切換」的層級；專案分頁 10 個排成一列時名稱被截到只剩兩三個字。
- 沒有任何通往檔案系統的動作：使用者要到 Finder、終端機、編輯器只能自己找路徑；專案方塊也沒有右鍵選單。
- 「新增 Workspace」「加入專案」「專案分頁」三個詞指同一件事，使用者可見文案與 docs 各寫各的。

cut 1 已把 token、原語與 BrandMark／Wordmark 立好，殼可以直接用 `bg-sidebar`、`rounded-lg`、`BrandMark` 組裝；後面的看板（cut 3a）、詳情頁（cut 3b）、設定模式（cut 4）都要落在這個殼的主區與專案欄裡，所以殼先做。

## Proposed Solution

1. **三平台標題列**：macOS 以 `tauri.macos.conf.json` 設 `titleBarStyle: "Overlay"`、`hiddenTitle: true`、`trafficLightPosition: {x: 16, y: 26}`（tao 的 y 是標題列容器高度減按鈕高度，26 讓紅綠燈在 48px 列內垂直置中），`decorations` 維持 true；Windows 以 `tauri.windows.conf.json` 設 `decorations: false`、`shadow: true`，app 自繪 48px 拖曳列與右端三顆視窗鈕（縮到最小、最大化／還原、關閉；各 46×48、hover 灰底、關閉 hover 紅底白字）；Linux 維持原生標題列，其下同一套版面。兩條頂列（左側標題列、主區頂列）都是 `data-tauri-drag-region`，專案名一律從下方導覽文字的左緣（98px）起，macOS 紅綠燈落在其左側留白。
2. **左側：圖示列＋專案欄**：圖示列 56px、全域，標題列下方 8px 起由上而下為 `BrandMark`（26px，下留 14px）、每個開著的專案一個 32px 首字母方塊（作用中 teal 底白字、其餘灰底灰字；hover tooltip 顯示全名與路徑；右鍵同一份專案動作選單；路徑失效＝角落紅點；探測中＝spinner；remote 離線或需重新登入＝角落琥珀點；不顯示任何計數徽章）、「＋」虛線方塊開新增專案、底部齒輪為全域設定（作用中 teal 底）。專案欄固定 200px、不收合、底色 `bg-sidebar`：變更、已封存（計數）、規格、手冊，底部專案設定；設定自專案欄移到圖示列底部。分頁的持久化、上限 10、去重、關閉、錯誤態、⌃Tab 與 ⌃1–9 全部保留——改的是呈現位置與形狀，不是資料。
3. **左側連續標題列**：圖示列與專案欄共用一條 48px 標題列（同側欄底色、48px 內無直向分隔線，下緣一條與主區頂列同高的細線，頂列橫貫整個視窗），紅綠燈在 x=16，專案名左緣對齊下方導覽文字：純文字專案名（14px 中粗、截斷、hover 顯示完整路徑）＋右端「⋯」圖示鈕開專案動作選單——在 Finder 顯示、在終端機開啟、以編輯器開啟、複製路徑、分隔線、重新整理 ⌘R、關閉專案 ⌘W（紅字）。檔案管理員名稱隨平台（Finder／檔案總管／檔案管理員）；終端機為 macOS Terminal、Windows 的 Windows Terminal（沒有則 cmd）、Linux 的 x-terminal-emulator（沒有則依序 gnome-terminal、konsole、xfce4-terminal）；編輯器依序找 PATH 上的 code、cursor、zed、subl，都沒有時 toast 說明；啟動後立即失敗退出的候選改試下一個，專案目錄已不存在時 toast 說明；remote 專案沒有本機檔案時只剩重新整理與關閉專案。指令組裝與依序啟動在 speclink-desktop-core（可單元測試），Tauri command 只做委派；「在 Finder 顯示」走 tauri-plugin-opener 的 reveal。選單標示的 ⌘R／Ctrl+R 與 Windows、Linux 的 Ctrl+W 由 app 直接處理；macOS 的 ⌘W 被系統選單先攔截，由 desktop-native-menu 承接。
4. **主區頂列與頁標題區**：主區頂端 48px 麵包屑「專案名 / 變更」（灰字，最後一段深字），Windows 時右端三顆視窗鈕；新增原語 `PageHeader`（24px 一般字重標題＋灰字說明＋右側動作槽），看板頁套用：標題「變更」、說明「依生命週期分欄；拖曳卡片調整順序，點卡片開詳情。」、動作槽放既有的搜尋與篩選工具列（仍在欄位上方、同一列）。規格、已封存、手冊、設定、專案設定的頁標題隨各自的刀落地。
5. **零專案空狀態**：圖示列只有品牌標記、＋、設定，不渲染專案欄；主區置中直欄：`Wordmark`（40px）、24px 標題「開啟一個專案開始」、灰字說明（含「資料夾還沒有 openspec/ 時會先問你要不要初始化」）、「新增專案」主要鈕＋「連線 Server」框線鈕（開對話框並直接進 Server 步驟）、下方「最近開啟」清單卡（與對話框第一步同一份清單元件：本機條目顯示資料夾名與路徑、remote 條目帶「遠端」籤、點開即開啟、失敗轉錯誤態、× 移除）；清單為空時不顯示該區段。
6. **新增專案對話框換皮與改名**：標題「新增專案」、15px 標題＋灰字說明（本機資料夾沒有 openspec/ 時會先問要不要初始化）；第一步兩張來源卡改為可選取卡（選定後主要鈕才可按：本機→「選擇資料夾…」、Server→「下一步」）；「最近開啟」為 14px 小標＋計數，列放進 12px 細框卡、細線分隔、hover 灰底、× hover 才現、遠端項目帶「遠端」籤；步驟 2–4 改單選列（Server 清單、兩層專案／儲存庫、checkout 路徑與記住綁定）；頁尾「取消」ghost＋主要鈕；四步與步驟條、所有開啟語意與錯誤態不變。
7. **詞彙收斂**：`openspec/LANGUAGE.md` 新增「專案」詞條（桌面 app 開著的一個工作目錄，本機資料夾或 server 上的 Project／Repo；圖示列一個方塊＝一個專案），avoid 列 Workspace、workspace、工作區、專案分頁；「Server」詞條定義文中的「新增 Workspace」改「新增專案」。desktop 的 i18n 兩語系 26 處「Workspace」改「專案」（如「新增專案」「開啟專案」「找不到專案」「專案列」），分頁語意的鍵（關閉分頁、自分頁移除）改為「關閉專案」「自專案列移除」；docs 四份文件同步。avoid 詞「工作區」「專案分頁」為中文、會入 `ui-copy-vocabulary` 機械守門，同批把 desktop i18n 的 4 處「工作區」改掉。
8. **原語與共用元件**：packages/ui 新增 `dropdown-menu`（Radix DropdownMenu；Content 12px 圓角 p-1.5 shadow-lg、Item 32px 高 8px 圓角 hover 灰底、destructive 變體紅字、右端 `Kbd` 快捷鍵槽）、`context-menu`（Radix ContextMenu，與 dropdown 同一套 item 樣式，圖示列方塊右鍵用）、`kbd`（等寬 11px 灰字）、`page-header`；領域層新增 `NavItem`（32px、8px 圓角、圖示＋標籤＋右端計數，作用中 teal 淡底 teal 字），desktop 的本地 `NavItem` 與 server-web 的管理面導覽改用它。
9. **server-web 殼同步**：`ConsoleLayout.tsx` 側欄改 `bg-sidebar`、導覽項改共用 `NavItem`；頂列維持 48px 字標＋帳號（server-web 沒有專案與圖示列的概念）。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：無。相容性：`--json` 與 CLI 輸出不變；app 本機持久化鍵（`speclink.projectTabs`、`speclink.recentWorkspaces`）格式不變；新增兩個 Tauri 平台設定檔與五條 `core:window:*` 權限（「在 Finder 顯示」沿用既有的 reveal command，不新增 opener 權限）。

## Non-Goals

- 不搬提示：技能檔提示仍在主區頂部、版本更新通知列仍在視窗頂端，歸位到專案欄底部提示卡與圖示列「有新版本」鈕是同刀的 desktop-notice-relocation。
- 不做 macOS 原生選單中文化與六組功能，那是同刀的 desktop-native-menu；本塊只保證殼上的動作（開新增專案、關閉專案、切頁、重新整理）可被選單呼叫。
- 不改設定模式：點齒輪仍進既有的應用程式設定頁，專案欄在設定模式維持四頁＋專案設定（無作用中項），分類欄與「設定」標題列是 cut 4。
- 不改看板欄、卡片、搜尋列的形狀（cut 3a）、不改抽屜（cut 3b）、不改系統匣（cut 5）；規格、已封存、手冊、設定頁的頁標題隨各自的刀。
- 不改分頁資料模型：`tabs.ts`、`recents.ts`、`session.ts` 的持久化格式、locator 身分、上限與去重不動。
- 不改 Linux 的原生標題列。
- 不碰 Rust 端其他 command 與 CLI。

## Alternatives Considered

- 圖示列只放頁面導覽、專案分頁留頂列——沒有表達「頁面屬於專案」的層級（討論否決）。
- 第二欄放專案清單——專案通常只有 2–5 個，第二欄多為空白（討論否決）。
- 專案欄可收合成圖示欄——與 56px 圖示列並排成兩條圖示欄，分不清專案與頁面（討論否決）。
- 紅綠燈塞進 56px 圖示列或把圖示列加寬到 72px——三顆燈需約 68px，前者擠不下、後者多出全高空白（討論否決）。
- 標題列的專案名帶「▾」下拉——會被讀成切換專案，與圖示列重複（討論否決）。
- Windows 維持原生標題列——空白條在 Windows 還在（討論否決）。
- 把提示歸位與原生選單一起做進本塊——三者可獨立驗收，合在一起超過 25 個任務且 Rust 選單與前端殼互不相依。

## Impact

- Affected specs: `desktop-app`（MODIFIED「側欄導覽結構」——側欄拆成圖示列與專案欄、設定移到圖示列底部、零專案無專案欄；MODIFIED「分頁切換中即時回饋」——spinner 落在圖示列方塊；MODIFIED「共用元件唯一來源」——字標離開頂欄，品牌資產場景改指零專案空狀態與圖示列；ADDED「隱藏原生標題列與三平台視窗殼」「專案層檔案系統動作」）、`desktop-config`（MODIFIED「專案分頁列存於 app 本機」——頂欄分頁列改為圖示列專案方塊，持久化／上限 10／去重／關閉／錯誤態／快捷鍵全保留，新增 tooltip、右鍵選單與角落狀態點）、`workspace-chooser`（RENAMED「新增 Workspace 的來源分流」→「新增專案的來源分流」並 MODIFIED——入口改為圖示列「＋」、空狀態兩鈕、標題列「⋯」不含；來源卡可選取＋主要鈕；MODIFIED「最近開啟清單」——文案改「新增專案」、空狀態共用同一份清單、開啟失敗的條目可再點重試）、`openspec/LANGUAGE.md`（新增「專案」詞條、修「Server」詞條）
- Affected code:
  - Modified: `apps/desktop/src/App.tsx`、`apps/desktop/src/main.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/tray.ts`、`apps/desktop/src/panel/TrayPanel.tsx`、`apps/desktop/src/components/WorkspaceChooser.tsx`、`apps/desktop/src/components/ServersPanel.tsx`、`apps/desktop/src/adapter/workspace.ts`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src-tauri/src/cli_install.rs`、`apps/desktop/src-tauri/capabilities/default.json`、`apps/desktop/core/src/lib.rs`、`packages/ui/src/components/KanbanBoard.tsx`、`packages/ui/src/index.ts`、`packages/ui/package.json`、`package-lock.json`、`apps/server-web/src/layouts/ConsoleLayout.tsx`、`apps/server-web/src/components/AdminNav.tsx`、`openspec/LANGUAGE.md`、`docs/product-status.md`、`docs/product-status.zh-TW.md`、`docs/remote-getting-started.md`、`docs/remote-getting-started.zh-TW.md`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/workspaceChooser.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/workspace.test.ts`、`apps/desktop/src/__tests__/serversPanel.test.tsx`、`apps/desktop/src/__tests__/remoteCapabilities.test.tsx`、`apps/desktop/src/__tests__/remoteResilience.test.tsx`、`apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx`、`packages/ui/src/__tests__/kanban.test.tsx`、`apps/server-web/src/__tests__/admin-console-shell.test.tsx`
  - New: `apps/desktop/src-tauri/tauri.macos.conf.json`、`apps/desktop/src-tauri/tauri.windows.conf.json`、`apps/desktop/src-tauri/src/fs_actions.rs`、`apps/desktop/core/src/fs_actions.rs`、`apps/desktop/src/platform.ts`、`apps/desktop/src/components/TitleBar.tsx`、`apps/desktop/src/components/ProjectRail.tsx`、`apps/desktop/src/components/ProjectColumn.tsx`、`apps/desktop/src/components/ProjectActionsMenu.tsx`、`apps/desktop/src/components/RecentList.tsx`、`apps/desktop/src/components/EmptyWorkspace.tsx`、`apps/desktop/src/adapter/fsActions.ts`、`packages/ui/src/components/ui/dropdown-menu.tsx`、`packages/ui/src/components/ui/context-menu.tsx`、`packages/ui/src/components/ui/kbd.tsx`、`packages/ui/src/components/ui/page-header.tsx`、`packages/ui/src/components/NavItem.tsx`、`apps/desktop/src/__tests__/titleBar.test.tsx`、`apps/desktop/src/__tests__/projectRail.test.tsx`、`apps/desktop/src/__tests__/projectActionsMenu.test.tsx`、`apps/desktop/src/__tests__/recentList.test.tsx`、`apps/desktop/src/__tests__/emptyWorkspace.test.tsx`、`apps/desktop/src/__tests__/platform.test.ts`、`packages/ui/src/__tests__/menus.test.tsx`、`packages/ui/src/__tests__/pageHeader.test.tsx`、`packages/ui/src/__tests__/navItem.test.tsx`
  - Removed: `apps/desktop/src/components/ProjectTabs.tsx`、`apps/desktop/src/__tests__/projectTabs.test.tsx`（分頁列的測試案例搬到 `projectRail.test.tsx`）
