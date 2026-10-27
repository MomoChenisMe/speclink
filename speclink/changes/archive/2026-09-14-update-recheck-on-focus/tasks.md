## 1. 節流判定（core 純函式，D2）

- [x] 1.1 紅燈：在 apps/desktop/src/__tests__/updater.test.ts 新增 `focusRecheckDue` 測試——lastCheckedAt 為 null 回 true、差 1 毫秒回 false、剛好 `FOCUS_RECHECK_INTERVAL_MS` 回 true、超過回 true；執行 `npm test -w apps/desktop` 確認因函式不存在而紅。 <!-- speclink-task:tsk_01M2EXX2DTF8DJ382M5S76RH47 -->
- [x] 1.2 綠燈：apps/desktop/src/core/updater.ts 新增常數 `FOCUS_RECHECK_INTERVAL_MS`（3,600,000）與純函式 `focusRecheckDue(lastCheckedAt, now)`，不依賴 Tauri；`npm test -w apps/desktop` 中 1.1 的測試轉綠。 <!-- speclink-task:tsk_01M2EXX2DTX7D8EGXWK2P9GTQM -->

## 2. adapter 與 store 接線（D1、D2）

- [x] 2.1 紅燈：apps/desktop/src/__tests__/updater.test.ts 以 `storeWith` 注入假 adapter，新增 store `recheckOnFocus` 測試——(a) 從未檢查時呼叫一次 `check`；(b) 啟動檢查後 30 分鐘呼叫 `recheckOnFocus(now)` 不再呼叫 `check`；(c) 滿 1 小時再呼叫 `check` 恰一次；(d) 手動 `checkForUpdates(true)` 後 30 分鐘不重檢；(e) 狀態為 downloading 時逾時也不呼叫 `check`；(f) 未注入 adapter 時為 no-op。`npm test -w apps/desktop` 確認紅。 <!-- speclink-task:tsk_01M2EXX2DT6M3MEVSC8MTA9HHW -->
- [x] 2.2 綠燈：apps/desktop/src/adapter/updater.ts 的 `UpdaterAdapter` 新增可選方法 `onFocusChanged`，`tauriUpdaterAdapter()` 以 `getCurrentWindow().onFocusChanged` 實作並把事件 payload 布林值交給 handler；apps/desktop/src/store.ts 新增 closure 變數 `lastCheckedAt`（`checkForUpdates` 於 reducer 真正進入 checking 時寫入 `Date.now()`）與動作 `recheckOnFocus(now?)`（`focusRecheckDue` 為 true 才呼叫 `checkForUpdates(false)`）；2.1 測試全綠，既有「更新 store 接線」測試不改仍綠。 <!-- speclink-task:tsk_01M2EXX2DTMX7CQMQJ95QPJR60 -->
- [x] 2.3 對新增的 adapter 方法與 store 動作套用 sharp-edges 稽核清單（`speclink instructions --skill audit`）：確認 `onFocusChanged` 缺席時零副作用、`now` 參數未給時取 `Date.now()`、reject 不外洩；結果以一段稽核註記寫進 2.2 的程式碼註解或本 task 的驗證紀錄。 <!-- speclink-task:tsk_01M2EXX2DT9D8JE2H1D8GMGX07 -->

## 3. App 訂閱主視窗焦點（D3）

- [x] 3.1 紅燈：apps/desktop/src/__tests__/App.test.tsx 新增測試——注入含 `onFocusChanged` 的假 updater adapter（handler 由測試捕獲、回傳可斷言的 unlisten），以 vi.useFakeTimers 與 vi.setSystemTime 讓時間前進 1 小時後觸發 handler(true)，斷言 `check` 被呼叫第二次；觸發 handler(false) 不增加呼叫；卸載元件後 unlisten 被呼叫一次；未提供 `onFocusChanged` 的假 adapter 啟動流程不變。`npm test -w apps/desktop` 確認紅。 <!-- speclink-task:tsk_01M2EXX2DT65G1KJZ9QV7EA8WA -->
- [x] 3.2 綠燈：apps/desktop/src/App.tsx 的啟動 effect 在 `updater?.onFocusChanged` 存在時訂閱，handler 收到 true 呼叫 `useStore.getState().recheckOnFocus()`，保存 unlisten 的 Promise 並於 cleanup 以 `.then((unlisten) => unlisten())` 取消，訂閱失敗 `.catch(() => {})` 靜默；3.1 測試全綠，既有 App.test.tsx、updateBanner.test.tsx 不改仍綠。 <!-- speclink-task:tsk_01M2EXX2DTPP6EDDN140JZ7A97 -->

## 4. 規格與收尾（D4、桌面自動更新）

- [x] 4.1 核對 delta spec「桌面自動更新」的七個 scenario 與 Example 表逐列對應到 1.1、2.1、3.1 的測試案例（每列至少一個測試斷言同樣的輸入與結果），並確認本 change 未對 CLI 加任何更新檢查（D4：以 grep 在 crates 目錄搜尋 GitHub releases 端點字串，結果仍為空）；不符處補測試或修 delta spec，執行 speclink validate update-recheck-on-focus 通過。 <!-- speclink-task:tsk_01M2FAXB4SWR5GQRTTAKN7084W -->
- [x] 4.2 收尾驗證：`npm test -w apps/desktop` 全綠，`speclink analyze update-recheck-on-focus` 無 Critical／Warning，`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（詞彙守門與文件連結檢查掃整個 README 與 docs 面）。 <!-- speclink-task:tsk_01M2EXX2DTQTTKESAF7D1EG370 -->
