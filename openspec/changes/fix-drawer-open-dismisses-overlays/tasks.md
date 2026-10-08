## 1. 狀態層：開啟抽屜時收掉可取消浮層

- [ ] 1.1 `apps/desktop/src/store.ts` 新增回傳部分狀態的純函式（依 design D5，例如 `cancelableOverlaysCleared()`），`openDetail` 與 `openDiscussion` 在既有的同一次 set 內展開它（design D1：互斥仍由 store 開啟動作保證；D2：歸零集合只含可取消浮層）：`workspaceChooser`、`pendingInit`、`pendingAdopt`、`pendingArchive`、`pendingArchiveDiscussion` 設為 null，`pendingArchiveSettled` 設回 `{ review: false, verify: false }`，其餘既有欄位（自身抽屜、`boardView: "board"`、其他三個抽屜欄位、`drawerVerb`）照舊。驗證：`apps/desktop/src/__tests__/store.test.ts` 新增案例「open 動作清除可取消浮層」——五種浮層各開一個，分別呼叫 `openDetail` 與 `openDiscussion`，斷言浮層欄位歸零、對應抽屜欄位已設、`boardView` 為 board；既有「detail 抽屜互斥」四組轉移案例不改且通過；`npm test -w apps/desktop -- store`。 <!-- speclink-task:tsk_01M4CTE5R5ZTQC3P4W1KG576T2 -->
- [ ] 1.2 `apps/desktop/src/store.ts` 的 `openDetail` 與 `openDiscussion` 在 `migrationRoot` 或 `pendingRemoteConflict` 非 null 時直接返回、不呼叫 set（design D4）；name／slug 找不到的既有靜默行為不變。驗證：`store.test.ts` 新增案例「不可取消浮層開啟中 open 動作不改狀態」——兩種浮層各非 null 時呼叫兩個動作，斷言 `getState()` 回傳的物件與呼叫前為同一參考（淺比較相等）、`boardView` 未切換；`npm test -w apps/desktop -- store`。 <!-- speclink-task:tsk_01M4CTE5R60ZJC6GV9DSSR10ZR -->

## 2. 更新日誌對話框跟隨抽屜關閉

- [ ] 2.1 `apps/desktop/src/App.tsx` 新增一個 effect：`detailChange` 或 `detailDiscussion` 自 null 變為非 null 時把本地 `releaseNotes` 設為 null（design D3）；不呼叫記錄「已看過」的路徑，whatsNew 下次啟動照舊彈出；`releaseNotes` 不搬進 store。驗證：`apps/desktop/src/__tests__/releaseNotesDialog.test.tsx` 新增案例「抽屜開啟時更新日誌關閉且不記為已看過」——以 whatsNew 模式渲染、觸發 store `openDetail`，斷言對話框不在畫面、已看過版號未寫入；`npm test -w apps/desktop -- releaseNotesDialog`。 <!-- speclink-task:tsk_01M4CTE5R63129644RCR4RKF6P -->

## 3. 驗收與收尾

- [ ] [M] 3.1 手動驗收（macOS 安裝版或 `npm run dev` 的 desktop）：主視窗按「新增 Workspace」停在步驟 1，自系統匣面板點某變更的「開啟此變更」，畫面只剩變更詳情抽屜、底層為看板；再以更新日誌（設定頁「檢視更新日誌」）開啟中自面板點「開啟此討論」，更新日誌關閉、討論抽屜開啟；遷移對話框開啟中自面板點「開啟此變更」，主視窗到前景、遷移對話框維持、抽屜不開。驗證：三條路徑的觀察結果與 spec「detail 抽屜互斥」新增的 scenario 一致。 <!-- speclink-task:tsk_01M4CTE5R6WMVXY01B5KGYVQC5 -->
- [ ] 3.2 全部 delta 與程式碼對齊：`./target/debug/speclink validate fix-drawer-open-dismisses-overlays` 通過；`npm test -w apps/desktop` 全綠（含既有 `tray.test.ts` 的 open-change 交接案例不變）；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。 <!-- speclink-task:tsk_01M4CTE5R66XJ1FG38ZKF0TF97 -->
