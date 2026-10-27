## Context

macOS 上 Tauri 2 在 app 未設定選單時給預設英文選單列；本 app 沒有任何 `set_menu` 或 JS 選單呼叫，選單列與 UI 語言脫節。系統匣（`apps/desktop/src/tray.ts`）已用 `@tauri-apps/api/menu` 的 `Menu.new`、`MenuItemOptions`／`PredefinedMenuItemOptions`／`SubmenuOptions` 從 i18n 字典建選單、以 `deps` 注入 Tauri 介面讓 `tray.test.ts` 用 mock 驗證，語言切換時重建。UI 語言在 `App.tsx` 的 `localePref` → `resolveUiLocale` → `I18nProvider`；`apps/desktop/src/i18n/runtime.ts` 的 `appT` 提供 store 層的翻譯。看板搜尋的 ⌘F 在 `packages/ui/src/components/BoardSearchBar.tsx` 以 `window` keydown 聚焦；⌃Tab／⌃1–9 在 `App.tsx` 的 keydown 監聽呼叫 store 的 `cycleTab`／`gotoTab`。更新日誌對話框是 `App.tsx` 本地 state（`setReleaseNotes`）。能力檔已有 `core:menu:default`（含 `allow-new`、`allow-set-as-app-menu`），沒有 `opener:allow-open-url`。Rust 端已註冊 `tauri_plugin_opener`，但前端尚未安裝 `@tauri-apps/plugin-opener`，本刀補進 `apps/desktop/package.json`。應用程式設定頁（`apps/desktop/src/views/AppSettingsView.tsx`）的本機設定簽目前有介面語言、軟體更新、安裝 CLI 三張卡；`@speclink/ui` 已有 `Kbd`（等寬按鍵字樣）元件，專案動作選單用它標 ⌘R／Ctrl+R。Windows／Linux 沒有選單列，實際生效的快捷鍵只有 `App.tsx` keydown 的 Ctrl+Tab、Ctrl+1–9、Ctrl+R、Ctrl+W 與 `BoardSearchBar` 的 Ctrl+F，目前 app 內沒有任何地方列出它們。

討論結論寫「Rust MenuBuilder 自建」，本設計改走 JS 選單 API（理由見 D1），功能清單與語言行為與結論一致。

## Goals / Non-Goals

**Goals:**

- macOS 選單列六組、文案跟 UI 語言、切換即重建；每個項目有可觀察的動作。
- 原生快捷鍵（⌘O、⌘W、⌘1–4、⌘F、⌘R、⌃Tab、⌘,）經選單進入既有動作。
- 選單模型可單元測試；非 macOS 零影響。
- 應用程式設定頁有一張「鍵盤快捷鍵」卡，依平台列出實際生效的快捷鍵（D5）。

**Non-Goals:**

- Windows／Linux 選單列；「關於」頁（cut 4）；系統匣選單；Rust 端變更。
- 快捷鍵自訂；快捷鍵一覽對話框與功能旁的就地提示（使用者 2026-10-09 選定只做設定頁一張卡）；系統預設快捷鍵（⌘Q、⌘H、⌘M、編輯組）不列入卡片。

## Decisions

### D1 以 JS 選單 API 從 i18n 字典建，不在 Rust 端另立文案

與系統匣同一機制：一份文案（`messages.ts`）、一種重建方式（語言或專案狀態變了就整個重建並 `setAsAppMenu()`）。Rust 端不加任何 command。替代方案「Rust MenuBuilder＋Rust 文案表」會讓兩語系鍵集合守門只涵蓋前端一半，且語言切換要跨 IPC 通知 Rust 重建。

### D2 選單模型

`apps/desktop/src/appMenu.ts`：

- `type AppMenuAction = "about" | "checkUpdates" | "settings" | "openProject" | "closeProject" | "installCli" | "viewBoard" | "viewArchived" | "viewSpecs" | "viewManual" | "viewProjectSettings" | "focusSearch" | "refresh" | "nextProject" | "bringAllToFront" | "helpManual" | "releaseNotes" | "github" | "reportIssue"`。
- `buildAppMenuModel({ t, hasProject }): AppMenuModel`——六個 `{ id, text, items }`，項目為 `{ kind: "item", id, text, accelerator?, enabled, action }`、`{ kind: "predefined", item: "Services" | "Hide" | "HideOthers" | "ShowAll" | "Quit" | "Undo" | "Redo" | "Cut" | "Copy" | "Paste" | "SelectAll" | "Minimize" | "Maximize", text? }` 或 `{ kind: "separator" }`。
- 內容與快捷鍵：Speclink＝about（「關於 Speclink」）、checkUpdates（「檢查更新…」）、separator、settings（「設定…」`CmdOrCtrl+,`）、separator、Services、separator、Hide（`Cmd+H`）、HideOthers（`Alt+Cmd+H`）、ShowAll、separator、Quit（`Cmd+Q`，文案「結束 Speclink」）；檔案＝openProject（「開啟專案…」`Cmd+O`）、closeProject（「關閉專案」`Cmd+W`，`enabled: hasProject`）、separator、installCli（「安裝 CLI…」）；編輯＝Undo、Redo、separator、Cut、Copy、Paste、SelectAll（文案取 `menu.edit.*` 鍵）；檢視＝viewBoard（「變更」`Cmd+1`）、viewArchived（「已封存」`Cmd+2`）、viewSpecs（「規格」`Cmd+3`）、viewManual（「手冊」`Cmd+4`）、viewProjectSettings（「專案設定」）、separator、focusSearch（「搜尋看板」`Cmd+F`）、refresh（「重新整理」`Cmd+R`）、separator、nextProject（「下一個專案」`Ctrl+Tab`）——檢視組全部 `enabled: hasProject`；視窗＝Minimize（`Cmd+M`）、Maximize（文案「縮放」）、separator、bringAllToFront（「全部移到最前」）；說明＝helpManual（「手冊」`enabled: hasProject`）、releaseNotes（「更新日誌」）、separator、github（「GitHub」）、reportIssue（「回報問題」）。
- 預設項的快捷鍵（Hide `Cmd+H`、HideOthers `Alt+Cmd+H`、Quit `Cmd+Q`、Minimize `Cmd+M`，以及編輯組）由系統預設項自帶——Tauri 的 `PredefinedMenuItemOptions` 不接受 accelerator，模型的 predefined 項因此不帶快捷鍵；實際按鍵由手動驗收確認。
- 文案鍵：`menu.app.about`、`menu.app.checkUpdates`、`menu.app.settings`、`menu.app.hide`、`menu.app.hideOthers`、`menu.app.showAll`、`menu.app.quit`、`menu.file.title`、`menu.file.openProject`、`menu.file.closeProject`、`menu.file.installCli`、`menu.edit.title`、`menu.edit.undo`、`menu.edit.redo`、`menu.edit.cut`、`menu.edit.copy`、`menu.edit.paste`、`menu.edit.selectAll`、`menu.view.title`、`menu.view.board`、`menu.view.archived`、`menu.view.specs`、`menu.view.manual`、`menu.view.projectSettings`、`menu.view.search`、`menu.view.refresh`、`menu.view.nextProject`、`menu.window.title`、`menu.window.minimize`、`menu.window.zoom`、`menu.window.bringAllToFront`、`menu.help.title`、`menu.help.manual`、`menu.help.releaseNotes`、`menu.help.github`、`menu.help.reportIssue`；兩語系鍵集合相等。

### D3 安裝與重建

- `installAppMenu(deps: { isMacOS: boolean; t: (key) => string; hasProject: boolean; dispatch: (action: AppMenuAction) => void })` → 回傳取消函式。與 `tray.ts` 相同，直接 import `@tauri-apps/api/menu` 的 `Menu`（測試以 `vi.mock` 替換）。非 macOS 時不呼叫任何選單 API、取消函式為 no-op。macOS：與系統匣同一做法，把模型轉成 `SubmenuOptions`／`MenuItemOptions`（`action: () => dispatch(id)`）／`PredefinedMenuItemOptions` 的整棵選項物件，一次交給 `Menu.new({ items })`（一次 IPC 建好，不逐項 `.new()`）後 `setAsAppMenu()`，再 `close()` 它回傳的被換下的舊選單，釋放 Rust 端資源；取消後才建好的選單不套用，直接 `close()`。
- `App.tsx`：一個 `useEffect`（依賴 `platform`、`t`、`hasProject`＝`activeKey !== null`）呼叫 `installAppMenu`，以回傳的取消函式為 cleanup——語言或專案有無改變時重裝，尚未套用的上一次作廢。`dispatch` 對照：about／installCli → `setBoardView("settings")`；checkUpdates → `checkForUpdates(true)` 後 `openSettingsUpdate()`（desktop-notice-relocation 提供；若尚未落地則 `setBoardView("settings")`，實作時以 store 是否有該動作判斷）；設定頁在手動檢查開始時（updater 狀態為 `checking` 且 `manual`）切到本機設定簽，讓軟體更新卡可見，背景自動檢查不切簽；settings → `setBoardView("settings")`；openProject → `openWorkspaceChooser()`；closeProject → `getCurrentWindow().isFocused()` 為真才關閉作用中專案（系統匣面板是 nonactivating NSPanel，它在前景時 ⌘W 也會進到 app 選單），與非 macOS 的 Ctrl+W 共用 `closeActiveProject`；viewBoard／viewArchived／viewSpecs／viewManual／viewProjectSettings／helpManual → `setBoardView(...)`；refresh → `refresh()`；nextProject → `cycleTab()`；focusSearch → 在看板時 `window.dispatchEvent(new CustomEvent("speclink:focus-search"))`，不在看板時先 `setBoardView("board")`、以 ref 記下待聚焦，等 `boardView` 變為 board 的 effect 再派事件（子元件的搜尋列 effect 先掛上監聽）；releaseNotes → `setReleaseNotes({ mode: "browse" })`；bringAllToFront → `getCurrentWindow().setFocus()`；github → `openUrl("https://github.com/MomoChenisMe/speclink")`；reportIssue → `openUrl("https://github.com/MomoChenisMe/speclink/issues/new")`。
- `BoardSearchBar.tsx`：除既有 ⌘F keydown，另監聽 `speclink:focus-search` 事件聚焦輸入。看板頁的 ⌘F 由 WebView 的 keydown 先收到並 `preventDefault`，原生選單不再觸發；點選單項目，或不在看板時按 ⌘F，才走事件。
- 能力檔：`capabilities/default.json` 加 `{ "identifier": "opener:allow-open-url", "allow": [{ "url": "https://github.com/MomoChenisMe/speclink*" }] }`。

### D4 「關於」的暫時目的地

cut 4 的「關於 Speclink」頁落地前，about 開應用程式設定頁；cut 4 把目的地改為關於頁（本設計與 spec 都寫明「關於頁落地前為設定頁」）。

### D5 設定頁的鍵盤快捷鍵卡

- 資料來源：`apps/desktop/src/appMenu.ts` 新增純函式 `buildShortcutList({ t, platform }): ShortcutRow[]`，`ShortcutRow = { id: AppMenuAction | "gotoProject"; text: string; keys: string }`。
  - macOS：取 `buildAppMenuModel({ t, hasProject: true })` 中帶 `accelerator` 的自訂項目，依選單順序（settings、openProject、closeProject、viewBoard、viewArchived、viewSpecs、viewManual、focusSearch、refresh、nextProject），`keys` 由同檔的 `formatAccelerator` 轉成符號：`Cmd`／`CmdOrCtrl` → ⌘、`Ctrl` → ⌃（模型只用到這三種修飾鍵），去掉 `+`（`CmdOrCtrl+,` → `⌘,`、`Ctrl+Tab` → `⌃Tab`）；最後補一列 `{ id: "gotoProject", text: t("settings.shortcutsGotoProject"), keys: "⌃1–9" }`。由選單模型導出，卡片與選單只有一份快捷鍵定義，不會各寫一份而漂移。
  - Windows 與 Linux（同一張表——keydown 判斷只分 macOS 與非 macOS）：固定五列，對齊 `App.tsx` keydown 與 `BoardSearchBar` 的實際處理——closeProject `Ctrl+W`、focusSearch `Ctrl+F`、refresh `Ctrl+R`、nextProject `Ctrl+Tab`、gotoProject `Ctrl+1–9`；動作名稱沿用同一動作的 `menu.*` 鍵（`menu.file.closeProject`、`menu.view.search`、`menu.view.refresh`、`menu.view.nextProject`）。`App.test.tsx` 逐列按下這五列的按鍵，確認都有 keydown 接手（`preventDefault`），手寫的表和實際按鍵漂移時測試會紅。
- i18n 新鍵 2 個：`settings.shortcutsTitle`（「鍵盤快捷鍵」／「Keyboard Shortcuts」）、`settings.shortcutsGotoProject`（「跳到第 1–9 個專案」／「Go to Project 1–9」）；兩語系鍵集合相等。
- `AppSettingsView.tsx`：新增必填 prop `platform: Platform`（與 `MainTitleBar`、`ProjectRail` 相同，由 `App.tsx` 既有的 `platform` 傳入）；本機設定簽在安裝 CLI 卡之後、面板錯誤列之前渲染 `Card`（`data-testid="shortcuts-card"`），標題 `settings.shortcutsTitle`，內容為逐列「動作名稱＋`Kbd` 按鍵」，沿用 `@speclink/ui` 的 `Card` 與 `Kbd`，不以 className 覆蓋變體的顏色、圓角與陰影。卡片只顯示、無互動，不依賴作用中專案。
- `App.tsx`：`<AppSettingsView platform={platform} …>`；既有測試的 `AppSettingsView` 渲染點補 `platform`。

## Implementation Contract

**Behavior**

- macOS：選單列為 Speclink、檔案、編輯、檢視、視窗、說明六組，文案依 UI 語言（zh-TW／en），於設定頁切換語言後立即換語言；項目與快捷鍵如 D2；無作用中專案時關閉專案、檢視組、說明的手冊為停用。各項動作：開啟專案開「新增專案」對話框；關閉專案在主視窗為焦點時關掉作用中專案；⌘1–4 切頁；搜尋看板聚焦看板搜尋輸入（不在看板時先切回看板）；重新整理重讀作用中專案；下一個專案循環切換；檢查更新執行手動檢查並進設定頁的本機設定簽（軟體更新卡）；設定／安裝 CLI／關於進設定頁；更新日誌開更新日誌對話框；GitHub 與回報問題開外部瀏覽器；視窗組為系統行為。
- Windows／Linux：無選單列、無任何選單 API 呼叫，既有 keydown 快捷鍵不變。
- 設定頁「鍵盤快捷鍵」卡（D5）：本機設定簽最後一張卡，零專案時照常顯示；macOS 依序 11 列（選單快捷鍵 10 列＋⌃1–9），Windows／Linux 依序 5 列（Ctrl+W、Ctrl+F、Ctrl+R、Ctrl+Tab、Ctrl+1–9）；標題與動作名稱跟 UI 語言、與選單同一動作的文案相同；無編輯操作。

**Interface / data shape**

- `buildAppMenuModel({ t, hasProject })`、`installAppMenu(deps)`、`AppMenuAction`（D2）。
- `installAppMenu` 回傳取消函式（D3）。
- `window` 自訂事件：`speclink:focus-search`（無 detail）。
- i18n 新鍵 36 個（D2）＋2 個（D5），兩語系鍵集合相等。
- `buildShortcutList({ t, platform })`、`ShortcutRow`（D5）；`AppSettingsViewProps.platform: Platform`（必填）。
- 能力檔新增一條 opener 權限；`apps/desktop/package.json` 新增 `@tauri-apps/plugin-opener` 相依。

**Failure modes**

- 選單 API 失敗（非 macOS 誤判、權限缺）：`installAppMenu` 捕捉並 `console.error`，app 照常運作、無選單列。
- `openUrl` 失敗：toast 錯誤原文。
- 系統匣面板在前景時的 ⌘W：主視窗不在焦點，不關專案。
- 沒有作用中專案時觸發已停用項：項目停用、不可觸發。
- 快捷鍵卡為靜態內容，無失敗路徑。

**Acceptance criteria**

- `apps/desktop/src/__tests__/appMenu.test.ts`：模型六組與順序、每個自訂項目的快捷鍵與 Hide／HideOthers／Quit／Minimize 為系統預設項、`hasProject=false` 時停用集合、zh-TW 與 en 文案各取自字典；`installAppMenu` 以 `vi.mock("@tauri-apps/api/menu")` 驗證——macOS 時 `Menu.new` 與 `setAsAppMenu` 各呼叫一次並關掉被換下的舊選單、取消後才建好的選單不套用並關掉、項目 action 觸發 dispatch 對應 id；非 macOS 零呼叫。
- `App.test.tsx`：語言切換後以新語言重裝選單（取消上一次），切回 zh-TW 變回繁中；`dispatch` 的切頁（專案欄作用中項）、零專案開新增專案對話框、關專案（主視窗不在焦點時不關）、搜尋看板（看板與非看板頁）、檢查更新、更新日誌；非 macOS 快捷鍵卡每列按鍵都有 keydown 接手。
- `packages/ui/src/__tests__/boardSearchBar.test.tsx`：派發 `speclink:focus-search` 後輸入取得焦點。
- `appMenu.test.ts`：`buildShortcutList` 在 macOS 回 11 列且 `text`／`keys` 逐列等於 spec Example「macOS 的卡片內容」、在 windows 與 linux 各回 5 列且逐列等於 Example「Windows 與 Linux 的卡片內容」、以 en 字典建時動作名稱為英文。
- `appSettingsView.test.tsx`：`platform="macos"` 與 `platform="windows"` 各渲染一次，本機設定簽最後一張卡為「鍵盤快捷鍵」且列數與按鍵如上、windows 畫面無 ⌘；en 介面標題為 Keyboard Shortcuts；停在伺服器簽時手動檢查開始即切回本機設定簽，背景自動檢查不切。
- `messages.test.ts` 鍵集合相等；`npm test -w apps/desktop`、`npm test -w packages/ui` 綠。
- 手動：macOS 實機看六組選單、切語言、⌘1–4、⌘F、⌘O、⌘W、GitHub；設定頁快捷鍵卡內容與選單一致。

**Scope boundaries**

- In：`appMenu.ts` 與測試、`App.tsx` 接線、`store.ts` 無新動作（只呼叫既有）、`BoardSearchBar` 事件監聽、i18n、能力檔一條、`@tauri-apps/plugin-opener` 前端相依、`buildShortcutList` 與設定頁快捷鍵卡（`AppSettingsView.tsx` 與其測試）。
- Out：Windows／Linux 選單、關於頁、系統匣、Rust 端、快捷鍵自訂、快捷鍵一覽對話框與就地提示。

## Risks / Trade-offs

- [同一按鍵同時有 keydown 處理與選單快捷鍵] → WebView 的 keydown 先收到按鍵；已 `preventDefault` 的鍵（⌘R、⌃Tab、看板頁的 ⌘F）不會再觸發選單，只動作一次；JS 不攔的鍵（⌘W、⌘1–4、⌘O、⌘,）由選單接手。非 macOS 的 keydown 不受影響。
- [每次重建留下舊選單的 Rust 端資源] → `setAsAppMenu()` 回傳被換下的舊選單，安裝後 `close()`；取消後才建好的選單也 `close()`。
- [`PredefinedMenuItem` 的 `text` 覆寫在部分項目被系統忽略] → Services／Hide 等以系統文案為準，模型仍提供文案；手動驗收記錄實際呈現。
- [回歸對照] → 不動 CLI、golden、`--json`；desktop 與 ui 測試為基準。
- [跨平台] → 非 macOS 走 no-op 分支，`appMenu.test.ts` 有「非 macOS 零呼叫」案。
- [快捷鍵卡與實際行為漂移] → macOS 列表由選單模型導出；Windows／Linux 是手寫的五列，改 `App.tsx` 或 `BoardSearchBar` 的 keydown 時要同批改 `buildShortcutList`（`appMenu.ts` 該段註解寫明對應來源）；`App.test.tsx` 的防漂移測試逐列按鍵確認有人接手。

## Migration Plan

無資料遷移；回滾即還原本刀 commit。

## Open Questions

無。
