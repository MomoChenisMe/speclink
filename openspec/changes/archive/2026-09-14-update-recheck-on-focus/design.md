## Context

桌面 app 的自動更新（desktop-app「桌面自動更新」）目前只有一個觸發點：App.tsx 的啟動 effect 呼叫一次 `checkForUpdates(false)`。狀態機在 apps/desktop/src/core/updater.ts（純 reducer，不依賴 Tauri），adapter 在 apps/desktop/src/adapter/updater.ts（`UpdaterAdapter` 介面：`check`、`relaunch`；Tauri 實作委派 tauri-plugin-updater），store（apps/desktop/src/store.ts）以 closure 變數 `pendingUpdate` 承載待套用更新、以注入的 adapter 驅動 reducer。測試以假 adapter 注入（apps/desktop/src/__tests__/updater.test.ts 的 `storeWith`、App.test.tsx 直接傳 `{ check, relaunch }` 物件）。

Tauri 視窗 API 提供 `getCurrentWindow().onFocusChanged(handler)`（@tauri-apps/api ^2，回傳 unlisten 函式的 Promise）。macOS 系統匣面板是獨立的 `tray-panel` 視窗（apps/desktop/src-tauri/src/panel.rs：nonactivating NSPanel、`can_become_main_window: false`），開關面板不改變主視窗焦點；面板動作「詳情／設定／登入」會聚焦主視窗（tray.rs 的 `focuses_main_window`）。

來源討論 update-check-on-focus-and-cli 已裁定：做前景重檢、節流 1 小時、CLI 不加檢查、獨立小變更。

## Goals / Non-Goals

**Goals:**

- 主視窗回到前景時，距上次檢查逾 1 小時，自動在背景重檢一次更新；結果的呈現規則與啟動時的自動檢查完全相同。
- 改動只落在 apps/desktop 前端與 adapter 層；既有假 adapter 與既有測試不需改寫即可維持綠。

**Non-Goals:**

- 不做定時輪詢（setInterval）——前景事件已覆蓋「使用者回來看」的時機。
- 不為單獨安裝的 CLI（install.sh／brew／npm）加任何更新檢查或提示——通路自帶升級流程、CLI 主要由 agent 以 --json 呼叫、舊 CLI 的降級傷害已由 `speclink update` 的降級守門與桌面啟動自動重佈擋住。
- 不動 src-tauri 殼、不動更新端點與簽章流程（desktop-release）、不新增設定欄位（節流時長為常數）。
- 節流時間戳不持久化——app 重啟本就檢查一次，跨重啟的節流沒有意義。

## Decisions

### D1 焦點訊號經 UpdaterAdapter 的可選方法注入，不另開 adapter 檔、不用 document visibilitychange

`UpdaterAdapter` 新增可選方法 `onFocusChanged?: (handler: (focused: boolean) => void) => Promise<() => void>`；`tauriUpdaterAdapter()` 以 `getCurrentWindow().onFocusChanged` 實作，handler 收到 payload 布林值。可選的理由：App.test.tsx 與 updater.test.ts 既有的假 adapter 只有 `check`／`relaunch`，方法可選讓它們原樣維持綠，且「未注入焦點訂閱＝不重檢」與「未注入 updater＝checkForUpdates 為 no-op」是同一條退化規則。

替代方案：(a) 新開 adapter/window.ts——更新是唯一消費者，多一個檔多一個注入點；(b) WebView 的 `document.visibilitychange`——視窗被其他視窗蓋住但仍可見時不會觸發，且 Tauri 對各平台的 visibility 語意不一致；(c) 在 Tauri 殼（Rust）監聽視窗事件再 emit——多一層跨 IPC 的轉發，違反「Tauri command 只做單行委派」。

### D2 節流判定與狀態守門為 core 純函式，時間戳記在 store closure，任何檢查（自動或手動）都重置

apps/desktop/src/core/updater.ts 新增常數 `FOCUS_RECHECK_INTERVAL_MS = 60 * 60 * 1000` 與兩個純函式：`focusRecheckDue(lastCheckedAt: number | null, now: number): boolean`——`lastCheckedAt` 為 null（從未檢查）或 `now - lastCheckedAt >= FOCUS_RECHECK_INTERVAL_MS` 時回 true；`focusRecheckAllowed(state: UpdaterState): boolean`——只有 idle／upToDate／checkFailed 三態回 true，待同意的提示（available）與安裝失敗的錯誤（error）要留到使用者處置，不能被自動重檢清掉，checking／downloading／restartPending 不重入。store 以 closure 變數 `lastCheckedAt: number | null`（與 `pendingUpdate` 同層、不進 state）記錄，`checkForUpdates(manual)` 在 reducer 接受 `checkStarted`（即真的進入 checking）時寫入 `Date.now()`；新增動作 `recheckOnFocus(now?: number)`：先過 `focusRecheckAllowed(get().updater)`，再 `focusRecheckDue(lastCheckedAt, now ?? Date.now())`，兩者皆 true 才呼叫 `checkForUpdates(false)`。手動檢查也重置時間戳：使用者剛按過「檢查更新」，一小時內切回視窗不需再查。（review 第 1 輪發現：只靠 reducer 擋 downloading／restartPending 不夠，available／error 會被自動重檢清掉，故加狀態守門。）

替代方案：(a) 時間戳進 store state——會觸發 React 重繪，且沒有任何 UI 讀它；(b) 節流放在 App.tsx 的 ref——邏輯離開 store 就無法用 `storeWith` 測；(c) 只有自動檢查重置——手動檢查後一小時內又自動查一次，多一次無意義的請求。

### D3 App.tsx 於啟動 effect 訂閱主視窗焦點、卸載時取消訂閱，只對 focused 為 true 反應

App.tsx 既有的啟動 effect（呼叫 `checkForUpdates(false)` 的那一個）在 `updater?.onFocusChanged` 存在時訂閱：handler 收到 `true` 才呼叫 `useStore.getState().recheckOnFocus()`，收到 `false` 不做事。訂閱回傳的是 unlisten 的 Promise：effect 保存該 Promise（訂閱失敗以 `.catch(() => null)` 靜默收斂成 null），cleanup 以 `.then((unlisten) => unlisten?.())` 取消，避免元件卸載早於訂閱完成而漏掉 unlisten；非 Tauri 環境的靜默與同檔 `appVersion()` 的既有處理一致。

替代方案：獨立一個 effect 專管訂閱——可行但與啟動檢查是同一件事（「更新檢查的觸發點」），放同一個 effect 讓兩個觸發點並列可讀。

### D4 CLI 不加更新檢查機制（決策記錄，無實作工作）

只裝 CLI 的使用者不加任何更新檢查或提示。理由見 Non-Goals；折衷方案「只在 `speclink update` 動詞內一天查一次」留待日後有舊 CLI 回報 bug 的實證再開討論。本決策對應 tasks 中的規格與文件核對，不對應程式碼。

## Implementation Contract

**行為**：Speclink 桌面 app 主視窗從失焦變為取得焦點時，若距上次更新檢查（含啟動檢查與手動檢查）已滿 1 小時，app 在背景檢查一次更新；未滿 1 小時則不檢查。背景檢查的呈現規則與啟動檢查相同：找到新版顯示目標版號並等使用者同意；已是最新或檢查失敗皆靜默；檢查中、下載中或待重啟時不重檢；待同意的提示與安裝失敗的錯誤留到使用者處置，不被重檢清掉。開關 macOS 系統匣面板不觸發重檢；面板的「詳情／設定／登入」動作聚焦主視窗，經同一節流判定。

**介面**：
- `UpdaterAdapter.onFocusChanged?: (handler: (focused: boolean) => void) => Promise<() => void>`（apps/desktop/src/adapter/updater.ts）。
- `FOCUS_RECHECK_INTERVAL_MS`、`focusRecheckDue(lastCheckedAt, now)`、`focusRecheckAllowed(state)`（apps/desktop/src/core/updater.ts）。
- store 動作 `recheckOnFocus(now?: number): Promise<void>`（apps/desktop/src/store.ts）。

**失敗模式**：未注入 updater 或 adapter 無 `onFocusChanged` → 不訂閱、不重檢、無錯誤。訂閱 Promise reject → 靜默。重檢時端點不可達 → reducer 自動模式靜默回閒置（既有行為）。

**驗收**：
- `npm test -w apps/desktop` 通過，含新增測試：`focusRecheckDue` 的邊界（null、剛好 1 小時、差 1 毫秒）；`focusRecheckAllowed` 八個狀態的真值表；store `recheckOnFocus` 逾時呼叫 `check` 恰一次、未逾時零次、手動檢查後重置、下載中／待重啟／待同意／錯誤四態不重檢；App 層注入假 adapter 後觸發 focused=true 逾時會再呼叫 `check`、focused=false 不呼叫、卸載呼叫 unlisten。
- 既有 updater.test.ts、updateBanner.test.tsx、App.test.tsx 不修改仍全綠。
- `speclink validate update-recheck-on-focus` 通過。

**範圍**：in scope＝上列四個檔與其測試、desktop-app 規格 delta。out of scope＝src-tauri、CLI、設定欄位、持久化、定時輪詢、Linux／Windows 的焦點事件語意調校（沿用 Tauri 預設）。

## Risks / Trade-offs

- [跨平台：Linux／Windows 的 `onFocusChanged` 觸發頻率或時機與 macOS 不同] → 節流把任何觸發頻率壓到每小時至多一次，且失敗靜默；不需平台分支。
- [卸載早於訂閱完成，unlisten 漏掉造成重複 handler] → D3 以 Promise 鏈在 cleanup 取消；App 測試斷言卸載後 unlisten 被呼叫。
- [既有假 adapter 缺新方法讓型別檢查變紅] → 方法為可選；前端 tsc 本就不是 CI 守門，vitest 不做型別檢查，但仍以可選避免新增型別錯誤。
- [回歸對照：golden 與 CLI 測試] → 不動 speclink-core／speclink-cli，golden 與 CLI 測試零影響；前端只跑 `npm test -w apps/desktop`。
- [1 小時是猜測值] → 常數集中一處，改值只動一行與一個測試。

## Migration Plan

隨桌面 app 下一版 release 一起出，無資料格式與設定變更；回退＝還原四個檔。

## Open Questions

（無；討論留待實測的「系統匣面板是否觸發主視窗焦點」已由 panel.rs 的 nonactivating NSPanel 與 `can_become_main_window: false` 解答為否。）
