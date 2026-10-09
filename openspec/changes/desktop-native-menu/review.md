# Review — desktop-native-menu

## Round 1

**Phase**: discovery
**Patch**: sha256:526e917996116492cae107cd488078eaf9a41c20dafc66b33ae84d76013acfed
**Scope**: apps/desktop/package.json, apps/desktop/src-tauri/capabilities/default.json, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/appMenu.test.ts, apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/__tests__/messages.test.ts, apps/desktop/src/appMenu.ts, apps/desktop/src/i18n/messages.ts, apps/desktop/src/views/AppSettingsView.tsx, package-lock.json, packages/ui/src/__tests__/boardSearchBar.test.tsx, packages/ui/src/components/BoardSearchBar.tsx
- [SUGGESTION] apps/desktop/src/App.tsx — Standards: 可能是 Middle Man（多一層轉手）：`case "releaseNotes": return void window.dispatchEvent(new CustomEvent("speclink:show-release-notes"))` 發出的事件，又在同一個 AppInner 裡由 `show = () => setReleaseNotes({ mode: "browse" })` 自己接回來。`setReleaseNotes` 是 useState 的 setter（參照不會變），可以直接呼叫；第 802 行的 `onShowReleaseNotes` 也已經這樣做。CLAUDE.md 規定「不為一次性使用的程式碼建立抽象層」。design D3 有點名這個事件，所以列 SUGGESTION，不列 CRITICAL。
- [SUGGESTION] apps/desktop/src/App.tsx — Standards: 可能過度設計：安裝和重建拆成兩個 effect，加上 `appMenu` ref、eslint-disable，以及 controller 裡的佇列和「輸入相同就跳過」判斷。改成一個 effect，依賴 `[platform, t, hasProject]`，每次重建、清理時用取消旗標擋掉舊結果，應該就夠了（CLAUDE.md：「資深工程師會不會覺得這太複雜？」）。
- [SUGGESTION] apps/desktop/src/appMenu.ts — Standards: 可能是 Speculative Generality（為用不到的情況預先鋪路）：`MODIFIER_SYMBOLS` 的 `Alt: "⌥"`、`Shift: "⇧"` 在選單模型裡沒有任何快捷鍵用到，屬於 CLAUDE.md 說的「不加沒被要求的彈性」。
- [SUGGESTION] apps/desktop/src/appMenu.ts — Standards: 可能是 Shotgun Surgery（改一處要連帶改很多處）：Windows／Linux 的五列快捷鍵表和 `gotoProject("⌃1–9")` 都是手抄 App.tsx keydown 和 BoardSearchBar 的按鍵。註解自己也寫了「改那兩處的按鍵時要同批改這裡」，等於一個按鍵分三處定義。App.tsx 的 `if (st.activeKey) st.closeTab(st.activeKey)` 也和 keydown 裡關閉專案的分支重複。
- [SUGGESTION] apps/desktop/src/appMenu.ts — Standards: 和同類模組 tray.ts 的寫法不一致：tray 的 `TrayDeps` 用 `isMacOS?: boolean`，這裡改成函式 `isMacOS: () => boolean`；tray 直接 import `Menu`，測試用 `vi.mock("@tauri-apps/api/menu")` 替換（見 tray.test.ts），這裡另外加了 `menuApi: { Menu: Pick<typeof Menu, "new"> }` 注入，只是為了測試方便，而 App.test.tsx 本來就整個替換掉 `installAppMenu`；可能是 Speculative Generality；建議沿用 tray 的寫法（CLAUDE.md：比照周邊程式碼的慣用寫法）。
- [SUGGESTION] apps/desktop/src/__tests__/appSettingsView.test.tsx — Standards: 可能是 Mysterious Name（名稱和內容對不上）：「Windows 僅回報狀態」「Linux 非 AppImage 執行僅回報狀態」兩條測試都傳了 `platform="macos"`，和標題矛盾，讀的人會以為平台會影響 CLI 卡。建議改成和標題一致的 platform。
- [SUGGESTION] apps/desktop/src/appMenu.ts — Correctness: `setAsAppMenu()` 會回傳被換下的舊選單（Tauri 會為它另開一個 resource），程式直接丟掉、沒有 `close()`。另外 dispose 之後已經建好但沒套用的 menu 也沒關。結果是每次 rebuild（開或關最後一個專案、切換語言）都會在 Rust resource table 和 JS callback 表各留下一整棵舊選單，含約 19 個 action Channel。每次量不大，但會一直累積。修法：`(await menu.setAsAppMenu())?.close()`。系統匣 tray.ts 也是同樣寫法。
- [SUGGESTION] apps/desktop/src/App.tsx — Correctness: 「搜尋看板 ⌘F」只要有專案就是啟用狀態，但 `speclink:focus-search` 唯一的接收者 BoardSearchBar 只掛在 KanbanBoard 裡。觸發路徑：在已封存、規格、手冊或設定頁按 ⌘F 或點這個選單項，結果什麼都不會發生，而設定頁的快捷鍵卡也把它列成有效。可以改成先 `setBoardView("board")`、等下一個 frame 再派事件，或在非看板頁停用這一項。
- [SUGGESTION] apps/desktop/src/App.tsx — Correctness: 選單是整個 app 共用的。觸發路徑：系統匣面板（panel.rs 建立的 NSPanel 視窗）成為作用中視窗時按 ⌘W，會派發 `closeProject`，關掉主視窗目前的專案。以前預設選單的 ⌘W 是「關閉目前視窗」。必要時可在 dispatch 前確認主視窗是否在焦點上。
- [SUGGESTION] packages/ui/src/components/BoardSearchBar.tsx — Correctness: 新註解寫「原生選單吃掉 ⌘F 後 WebView 收不到 keydown」，和實際行為相反：依照上面 wry 和 WebKit 的流程，是 JS keydown 先收到，preventDefault 後選單就不會觸發。所以看板頁的 ⌘F 走的是 keydown，事件路徑只在 JS 沒攔下時才會跑。目前行為正確，但這個註解會讓之後維護的人誤以為 macOS 的 keydown 分支是死碼。

## Round 2

**Phase**: validation
**Patch**: sha256:803af0c8dab26de3c0031d1ca767cb39cb96b02faf599f4b051750535b09c29e
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/appMenu.test.ts, apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/appMenu.ts, apps/desktop/src/views/AppSettingsView.tsx, packages/ui/src/components/BoardSearchBar.tsx
- [SUGGESTION] apps/desktop/src/__tests__/appSettingsView.test.tsx — Standards: 可能不必要地複雜。`view` 的參數型別用兩層 conditional type（`extends infer U ? U extends { state: infer S } …`）從 props 反推更新狀態的型別。其實 `UpdaterState` 已經從 `../core/updater` 匯出，AppSettingsView 自己也有 import，測試直接 `import type { UpdaterState }` 就好（CLAUDE.md：「資深工程師會不會覺得這太複雜？」）。
- [SUGGESTION] apps/desktop/src/views/AppSettingsView.tsx — Standards: `key` 可能已經是孤兒。Tabs 改成受控後，`focusConnectionId` 一變就由 `useEffect` 重設 `tab` 狀態。原本 `<Tabs key={focusConnectionId ?? "settings"}>` 是靠重新掛載讓 `defaultValue` 再生效，這個用途已經被取代，等於兩套重設機制並存；ServersPanel 也自己用 effect 追蹤 `focusConnectionId`。如果這個 key 沒有其他用途，依 CLAUDE.md「清掉因這次改動而孤兒化的程式」應移除；如果是故意留著（例如要重置 ServersPanel 內部的狀態），建議補一行註解說明。

## Round 3

**Phase**: validation
**Patch**: sha256:9fcd8df1df681ee9d29e3ec608afc7c5347022bd05db960e5462bab33db9da2a
**Scope**: apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/views/AppSettingsView.tsx
