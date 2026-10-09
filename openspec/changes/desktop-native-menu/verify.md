# Verify — desktop-native-menu

## Round 1

**Phase**: discovery
**Patch**: sha256:526e917996116492cae107cd488078eaf9a41c20dafc66b33ae84d76013acfed
**Scope**: apps/desktop/package.json, apps/desktop/src-tauri/capabilities/default.json, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/appMenu.test.ts, apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/__tests__/messages.test.ts, apps/desktop/src/appMenu.ts, apps/desktop/src/i18n/messages.ts, apps/desktop/src/views/AppSettingsView.tsx, package-lock.json, packages/ui/src/__tests__/boardSearchBar.test.tsx, packages/ui/src/components/BoardSearchBar.tsx
- [WARNING] apps/desktop/src/__tests__/App.test.tsx — Correctness: spec 有三個 scenario 的 THEN 子句沒有測試對應。程式碼本身有處理（`apps/desktop/src/App.tsx:473`、`:478`、`:489`），但 `App.test.tsx:2003` 的「macOS 原生選單接線」只測了 viewSpecs、closeProject、github、reportIssue。缺的是：(1)「無專案時相關項目停用」說零專案時按「開啟專案…」會開「新增專案」對話框，沒有測 `dispatch("openProject")`；(2)「檢視組快捷鍵切頁」說「專案欄『規格』為作用中」和「按 ⌘1 回到看板」，都沒驗；(3)「搜尋看板經選單聚焦」只在 boardSearchBar.test 測了「收到事件會聚焦」，沒有從 App 層測 `dispatch("focusSearch")` 到搜尋輸入真的取得焦點。建議在同一個 describe 補三條：零專案下 dispatch("openProject") 後看到新增專案對話框；dispatch("viewSpecs") 後確認專案欄「規格」為作用中，再 dispatch("viewBoard") 確認回到看板；看板顯示時 dispatch("focusSearch") 後搜尋輸入是 `document.activeElement`。
- [SUGGESTION] apps/desktop/src/App.tsx — Correctness: spec 寫「檢查更新…執行手動檢查並進入應用程式設定頁的軟體更新卡」。`App.tsx:469-472` 做的是切到設定頁再呼叫 `checkForUpdates(true)`，但設定頁的頁簽沒有被程式控制，只有初始值（`AppSettingsView.tsx:152` 的 `defaultValue`，有 `focusConnectionId` 時預設是 servers）。所以如果使用者原本就停在「伺服器」簽，按了不會看到軟體更新卡。這條動作也沒有測試。建議在 App.test 補 `dispatch("checkUpdates")` 的測試（斷言 `checkForUpdates(true)` 被呼叫、boardView 是 settings）；這個邊角情況要不要處理，可以等 desktop-notice-relocation 的 `openSettingsUpdate` 落地時一起收。
- [SUGGESTION] apps/desktop/src/__tests__/appSettingsView.test.tsx — Correctness: 「卡片文案跟隨 UI 語言」scenario 要求卡片標題變成「Keyboard Shortcuts」。英文動作名稱已經在 appMenu.test 測過，但英文標題在畫面層沒有測（`appSettingsView.test.tsx:366` 只測中文）。另外「選單文案跟隨 UI 語言」的「切回 zh-TW 後立即變回繁中」也只測了中文切英文這個方向。建議補一條用 en 渲染 `AppSettingsView` 並斷言標題為 Keyboard Shortcuts 的測試，並在 App.test 語言切換那條加上切回 zh-TW 的斷言。

## Round 2

**Phase**: validation
**Patch**: sha256:397d99ffeebc84e8ef28ea56e06a7f89679d4aec8594db862ed0365269317642
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/appMenu.test.ts, apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/appMenu.ts, apps/desktop/src/views/AppSettingsView.tsx, packages/ui/src/components/BoardSearchBar.tsx
- [SUGGESTION] apps/desktop/src/appMenu.ts — Coherence: 這次把 `MODIFIER_SYMBOLS` 裡 `Alt → ⌥` 的對照刪了，但 delta spec「設定頁的鍵盤快捷鍵卡」需求還寫著「按鍵 SHALL 以 ⌘、⌃、⌥ 符號表示修飾鍵」，而 design D5 寫的是「模型只用到這三種修飾鍵」（Cmd／CmdOrCtrl／Ctrl）。目前卡片上沒有任何一列用到 Alt（⌥），畫面和 spec 的 Example 一模一樣。但如果以後加了帶 Alt 的快捷鍵，卡片會顯示「Alt」而不是 ⌥。建議擇一處理：把 spec 裡的「⌥」拿掉，讓 spec、design、程式三方一致；或者把 Alt 的對照加回去。

## Round 3

**Phase**: validation
**Patch**: sha256:9fcd8df1df681ee9d29e3ec608afc7c5347022bd05db960e5462bab33db9da2a
**Scope**: apps/desktop/src/__tests__/appSettingsView.test.tsx, apps/desktop/src/views/AppSettingsView.tsx
