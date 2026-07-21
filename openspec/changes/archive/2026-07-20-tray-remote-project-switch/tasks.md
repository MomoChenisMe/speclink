## 1. 紅：切換走 locator key 的失敗測試

- [x] 1.1 撰寫原生選單專案切換測試（apps/desktop/src/__tests__/tray.test.ts）：規格「選單專案切換」——點選非作用中專案（分別覆蓋 local 與 remote 分頁）時，選單 action 以該分頁的 locator key 呼叫 store 的 activateTab，且不呼叫 openProjectAt；remote 分頁案例點選後不得靜默無事。改寫既有「點非作用中專案呼叫 openProjectAt」斷言為 activateTab 語意。驗證：`npm test -w apps/desktop` 該組測試紅燈（現行實作仍以 root 呼叫 openProjectAt）。 <!-- speclink-task:tsk_01KXYK47VP7ZDT9GZHJWZA9PJQ -->
- [x] 1.2 撰寫面板切換測試（apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/tray.test.ts）：規格「面板樣式（macOS）」的「點擊 remote 專案 tab 原地切換」——TrayPanel 專案 tab 點擊以分頁 key（非 root）呼叫 onOpenProject 回呼；tray 接線層收到 open-project 面板動作事件（載荷 id＝locator key，含 remote 的 key）時呼叫 activateTab(key)，空 root 不再是前提。驗證：`npm test -w apps/desktop` 該組測試紅燈。 <!-- speclink-task:tsk_01KXYK47VPET80NPW9QKYPT1N3 -->

## 2. 綠：tray 切換動作改接 activateTab

- [x] 2.1 實作 tray 接線層切換（apps/desktop/src/tray.ts）：TraySnapshot 的 tabs 移除 root 欄位（切換把手統一為 locator key）；TrayStoreApi 以 activateTab(key) 取代 openProjectAt（openProjectViaDialog 保留——服務「加入專案」路徑）；原生選單 project 項的 action 與面板 open-project 動作事件 handler 皆改呼叫 activateTab(key)。行為：規格「選單專案切換」——local 與 remote 分頁點選一視同仁完成切換。驗證：1.1 測試轉綠。 <!-- speclink-task:tsk_01KXYK47VPF06DRD5HBGKSBWGC -->
- [x] 2.2 實作面板 tab 點擊傳 key（apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx）：onOpenProject 回呼參數由 root 改為 key，tab 點擊傳 tab.key；面板入口以 key 發 open-project 動作事件。行為：規格「面板樣式（macOS）」——點擊 remote 專案 tab 原地切換、面板不收合。驗證：1.2 測試轉綠。 <!-- speclink-task:tsk_01KXYK47VPBTFM42DQBD3GN1V9 -->
- [x] 2.3 全套回歸：`npm test -w apps/desktop` 全綠；`npm test -w packages/ui` 全綠（本變更對 packages/ui 零改動，綠燈即確認無外溢）。 <!-- speclink-task:tsk_01KXYK47VPYXJGGWDJ0DV2DH1W -->

## 3. 重構與真實視窗驗證

- [x] 3.1 清理孤兒與過時註解（apps/desktop/src/tray.ts、apps/desktop/src/panel/TrayPanel.tsx）：移除因 root 欄位退場而失效的註解（「remote 本刀無建構路徑」等）與型別殘留；確認切換路徑已無 openProjectAt 引用（僅「加入專案」路徑保留）。驗證：以 grep 檢視 apps/desktop/src/tray.ts 與 apps/desktop/src/panel/ 無以 root 為切換把手的殘留，`npm test -w apps/desktop` 仍全綠。 <!-- speclink-task:tsk_01KXYK47VPHJ7VA4Q3B6Y4CHSM -->
- [x] 3.2 真實視窗手動驗證（macOS；GUI 改動不得只依 jsdom）：`npm run build -w apps/desktop` 重建前端 dist 後啟動桌面 app，開啟兩個以上專案分頁（含一個 remote workspace）。斷言規格「面板樣式（macOS）」：tray 面板點擊 remote 專案 tab 完成原地切換——該 tab 轉實心主色、面板內容切為該專案、主視窗未被喚起、面板未收合；local tab 切換行為不變。斷言規格「選單專案切換」失敗語意：remote 連線或驗證失敗時 tray 靜默、看板該分頁呈現既有錯誤態、app 未崩潰。驗證：上述斷言逐項成立（截圖留證）。 <!-- speclink-task:tsk_01KXYK47VP37G2J3TYQE4APKTK -->
