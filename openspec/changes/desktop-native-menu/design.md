## Context

macOS 上 Tauri 2 在 app 未設定選單時給預設英文選單列；本 app 沒有任何 `set_menu` 或 JS 選單呼叫，選單列與 UI 語言脫節。系統匣（`apps/desktop/src/tray.ts`）已用 `@tauri-apps/api/menu` 的 `Menu.new`、`MenuItemOptions`／`PredefinedMenuItemOptions`／`SubmenuOptions` 從 i18n 字典建選單、以 `deps` 注入 Tauri 介面讓 `tray.test.ts` 用 mock 驗證，語言切換時重建。UI 語言在 `App.tsx` 的 `localePref` → `resolveUiLocale` → `I18nProvider`；`apps/desktop/src/i18n/runtime.ts` 的 `appT` 提供 store 層的翻譯。看板搜尋的 ⌘F 在 `packages/ui/src/components/BoardSearchBar.tsx` 以 `window` keydown 聚焦；⌃Tab／⌃1–9 在 `App.tsx` 的 keydown 監聽呼叫 store 的 `cycleTab`／`gotoTab`。更新日誌對話框是 `App.tsx` 本地 state（`setReleaseNotes`）。能力檔已有 `core:menu:default`（含 `allow-new`、`allow-set-as-app-menu`），沒有 `opener:allow-open-url`。

討論結論寫「Rust MenuBuilder 自建」，本設計改走 JS 選單 API（理由見 D1），功能清單與語言行為與結論一致。

## Goals / Non-Goals

**Goals:**

- macOS 選單列六組、文案跟 UI 語言、切換即重建；每個項目有可觀察的動作。
- 原生快捷鍵（⌘O、⌘W、⌘1–4、⌘F、⌘R、⌃Tab、⌘,）經選單進入既有動作。
- 選單模型可單元測試；非 macOS 零影響。

**Non-Goals:**

- Windows／Linux 選單列；「關於」頁（cut 4）；系統匣選單；Rust 端變更。

## Decisions

### D1 以 JS 選單 API 從 i18n 字典建，不在 Rust 端另立文案

與系統匣同一機制：一份文案（`messages.ts`）、一種重建方式（語言或專案狀態變了就整個重建並 `setAsAppMenu()`）。Rust 端不加任何 command。替代方案「Rust MenuBuilder＋Rust 文案表」會讓兩語系鍵集合守門只涵蓋前端一半，且語言切換要跨 IPC 通知 Rust 重建。

### D2 選單模型

`apps/desktop/src/appMenu.ts`：

- `type AppMenuAction = "about" | "checkUpdates" | "settings" | "openProject" | "closeProject" | "installCli" | "viewBoard" | "viewArchived" | "viewSpecs" | "viewManual" | "viewProjectSettings" | "focusSearch" | "refresh" | "nextProject" | "bringAllToFront" | "helpManual" | "releaseNotes" | "github" | "reportIssue"`。
- `buildAppMenuModel({ t, hasProject }): AppMenuModel`——六個 `{ id, text, items }`，項目為 `{ kind: "item", id, text, accelerator?, enabled, action }`、`{ kind: "predefined", item: "Services" | "Hide" | "HideOthers" | "ShowAll" | "Quit" | "Undo" | "Redo" | "Cut" | "Copy" | "Paste" | "SelectAll" | "Minimize" | "Maximize", text? }` 或 `{ kind: "separator" }`。
- 內容與快捷鍵：Speclink＝about（「關於 Speclink」）、checkUpdates（「檢查更新…」）、separator、settings（「設定…」`CmdOrCtrl+,`）、separator、Services、separator、Hide（`Cmd+H`）、HideOthers（`Alt+Cmd+H`）、ShowAll、separator、Quit（`Cmd+Q`，文案「結束 Speclink」）；檔案＝openProject（「開啟專案…」`Cmd+O`）、closeProject（「關閉專案」`Cmd+W`，`enabled: hasProject`）、separator、installCli（「安裝 CLI…」）；編輯＝Undo、Redo、separator、Cut、Copy、Paste、SelectAll（文案取 `menu.edit.*` 鍵）；檢視＝viewBoard（「變更」`Cmd+1`）、viewArchived（「已封存」`Cmd+2`）、viewSpecs（「規格」`Cmd+3`）、viewManual（「手冊」`Cmd+4`）、viewProjectSettings（「專案設定」）、separator、focusSearch（「搜尋看板」`Cmd+F`）、refresh（「重新整理」`Cmd+R`）、separator、nextProject（「下一個專案」`Ctrl+Tab`）——檢視組全部 `enabled: hasProject`；視窗＝Minimize（`Cmd+M`）、Maximize（文案「縮放」）、separator、bringAllToFront（「全部移到最前」）；說明＝helpManual（「手冊」`enabled: hasProject`）、releaseNotes（「更新日誌」）、separator、github（「GitHub」）、reportIssue（「回報問題」）。
- 文案鍵：`menu.app.about`、`menu.app.checkUpdates`、`menu.app.settings`、`menu.app.hide`、`menu.app.hideOthers`、`menu.app.showAll`、`menu.app.quit`、`menu.file.title`、`menu.file.openProject`、`menu.file.closeProject`、`menu.file.installCli`、`menu.edit.title`、`menu.edit.undo`、`menu.edit.redo`、`menu.edit.cut`、`menu.edit.copy`、`menu.edit.paste`、`menu.edit.selectAll`、`menu.view.title`、`menu.view.board`、`menu.view.archived`、`menu.view.specs`、`menu.view.manual`、`menu.view.projectSettings`、`menu.view.search`、`menu.view.refresh`、`menu.view.nextProject`、`menu.window.title`、`menu.window.minimize`、`menu.window.zoom`、`menu.window.bringAllToFront`、`menu.help.title`、`menu.help.manual`、`menu.help.releaseNotes`、`menu.help.github`、`menu.help.reportIssue`；兩語系鍵集合相等。

### D3 安裝與重建

- `installAppMenu(deps: { isMacOS: () => boolean; t: (key) => string; hasProject: boolean; dispatch: (action: AppMenuAction) => void; menuApi: { Menu, Submenu, MenuItem, PredefinedMenuItem } })` → 回傳 `{ rebuild({ t, hasProject }), dispose() }`。非 macOS 時 `rebuild` 與 `dispose` 為 no-op 且不呼叫 `menuApi`。每次 rebuild：以模型建 `Submenu`／`MenuItem`（`action: () => dispatch(id)`）／`PredefinedMenuItem`，`Menu.new({ items })` 後 `setAsAppMenu()`。
- `App.tsx`：`useEffect` 於 `uiLocale` 與 `hasProject`（`activeKey !== null`）變化時呼叫 `rebuild`；卸載 `dispose`。`dispatch` 對照：about／installCli → `setBoardView("settings")`；checkUpdates → `checkForUpdates(true)` 後 `openSettingsUpdate()`（desktop-notice-relocation 提供；若尚未落地則 `setBoardView("settings")`，實作時以 store 是否有該動作判斷）；settings → `setBoardView("settings")`；openProject → `openWorkspaceChooser()`；closeProject → `closeTab(activeKey)`；viewBoard／viewArchived／viewSpecs／viewManual／viewProjectSettings／helpManual → `setBoardView(...)`；refresh → `refresh()`；nextProject → `cycleTab()`；focusSearch → `window.dispatchEvent(new CustomEvent("speclink:focus-search"))`；releaseNotes → `window.dispatchEvent(new CustomEvent("speclink:show-release-notes"))`，`App.tsx` 監聽後 `setReleaseNotes({ mode: "browse" })`；bringAllToFront → `getCurrentWindow().setFocus()`；github → `openUrl("https://github.com/MomoChenisMe/speclink")`；reportIssue → `openUrl("https://github.com/MomoChenisMe/speclink/issues/new")`。
- `BoardSearchBar.tsx`：除既有 ⌘F keydown，另監聽 `speclink:focus-search` 事件聚焦輸入（原生快捷鍵由選單消費後 WebView 收不到 keydown）。
- 能力檔：`capabilities/default.json` 加 `{ "identifier": "opener:allow-open-url", "allow": [{ "url": "https://github.com/MomoChenisMe/speclink*" }] }`。

### D4 「關於」的暫時目的地

cut 4 的「關於 Speclink」頁落地前，about 開應用程式設定頁；cut 4 把目的地改為關於頁（本設計與 spec 都寫明「關於頁落地前為設定頁」）。

## Implementation Contract

**Behavior**

- macOS：選單列為 Speclink、檔案、編輯、檢視、視窗、說明六組，文案依 UI 語言（zh-TW／en），於設定頁切換語言後立即換語言；項目與快捷鍵如 D2；無作用中專案時關閉專案、檢視組、說明的手冊為停用。各項動作：開啟專案開「新增專案」對話框；關閉專案關掉作用中專案；⌘1–4 切頁；搜尋看板聚焦看板搜尋輸入；重新整理重讀作用中專案；下一個專案循環切換；檢查更新執行手動檢查並進設定頁；設定／安裝 CLI／關於進設定頁；更新日誌開更新日誌對話框；GitHub 與回報問題開外部瀏覽器；視窗組為系統行為。
- Windows／Linux：無選單列、無任何選單 API 呼叫，既有 keydown 快捷鍵不變。

**Interface / data shape**

- `buildAppMenuModel({ t, hasProject })`、`installAppMenu(deps)`、`AppMenuAction`（D2）。
- `window` 自訂事件：`speclink:focus-search`、`speclink:show-release-notes`（無 detail）。
- i18n 新鍵 36 個（D2），兩語系鍵集合相等。
- 能力檔新增一條 opener 權限。

**Failure modes**

- 選單 API 失敗（非 macOS 誤判、權限缺）：`installAppMenu` 捕捉並 `console.error`，app 照常運作、無選單列。
- `openUrl` 失敗：toast 錯誤原文。
- 沒有作用中專案時觸發已停用項：項目停用、不可觸發。

**Acceptance criteria**

- `apps/desktop/src/__tests__/appMenu.test.ts`：模型六組與順序、每個快捷鍵、`hasProject=false` 時停用集合、zh-TW 與 en 文案各取自字典；`installAppMenu` 以 mock `menuApi` 驗證——macOS 時 `Menu.new` 與 `setAsAppMenu` 各呼叫一次、rebuild 再呼叫一次、項目 action 觸發 dispatch 對應 id；非 macOS 零呼叫。
- `App.test.tsx`：語言切換後 rebuild 被呼叫且文案為新語言；`speclink:show-release-notes` 事件開更新日誌對話框。
- `packages/ui/src/__tests__/boardSearchBar.test.tsx`：派發 `speclink:focus-search` 後輸入取得焦點。
- `messages.test.ts` 鍵集合相等；`npm test -w apps/desktop`、`npm test -w packages/ui` 綠。
- 手動：macOS 實機看六組選單、切語言、⌘1–4、⌘F、⌘O、⌘W、GitHub。

**Scope boundaries**

- In：`appMenu.ts` 與測試、`App.tsx` 接線、`store.ts` 無新動作（只呼叫既有）、`BoardSearchBar` 事件監聽、i18n、能力檔一條。
- Out：Windows／Linux 選單、關於頁、系統匣、Rust 端。

## Risks / Trade-offs

- [選單快捷鍵吃掉 keydown 後既有 keydown 處理在 macOS 失效] → 動作改經選單 dispatch 到同一個 store 動作；非 macOS 的 keydown 不受影響；測試驗 dispatch 與 keydown 兩條路都到同一動作。
- [Tauri JS 選單 API 在 `setAsAppMenu` 後舊選單未釋放] → 與系統匣相同做法，重建只在語言或專案有無改變時發生（低頻）。
- [`PredefinedMenuItem` 的 `text` 覆寫在部分項目被系統忽略] → Services／Hide 等以系統文案為準，模型仍提供文案；手動驗收記錄實際呈現。
- [回歸對照] → 不動 CLI、golden、`--json`；desktop 與 ui 測試為基準。
- [跨平台] → 非 macOS 走 no-op 分支，`appMenu.test.ts` 有「非 macOS 零呼叫」案。

## Migration Plan

無資料遷移；回滾即還原本刀 commit。

## Open Questions

無。
