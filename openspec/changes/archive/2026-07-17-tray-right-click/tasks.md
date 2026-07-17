## 1. 右鍵開閉面板（spec「面板樣式（macOS）」：點擊不分滑鼠按鍵）

- [x] 1.1 撰寫失敗測試：面板樣式下 tray 點擊事件 button 為 "Right"、buttonState 為 "Up" 時觸發 onPanelToggle（與左鍵等價），"Down" 事件與非面板樣式仍不觸發，左鍵既有行為不回歸——apps/desktop/src/__tests__/tray.test.ts，執行 npm test -w apps/desktop 確認新案例先紅 <!-- speclink-task:tsk_01KXQF7CXQA3F8T6XBQFYH980X -->
- [x] 1.2 實作按鍵過濾放寬：apps/desktop/src/tray.ts 的 tray action handler 對 Left 與 Right 的 Click-Up 皆呼叫 onPanelToggle，positioner 座標餵入時機不變——滿足需求「面板樣式（macOS）」的「點擊不分滑鼠按鍵」；驗證 1.1 測試轉綠且既有 tray 測試全數通過 <!-- speclink-task:tsk_01KXQF7CXQSE60NVR67ZRE1XA3 -->

## 2. 動作區加入「設定」項（spec「系統匣圖示與原生選單」「開啟視窗與結束動作」）

- [x] 2.1 撰寫失敗測試：buildTrayModel 動作區輸出順序為「開啟 Speclink」→「設定」→「結束」三項（新增 settings 項目種類），原生選單接線層點擊「設定」時喚起主視窗並呼叫 store 的 setBoardView("settings")——apps/desktop/src/__tests__/tray.test.ts，npm test -w apps/desktop 確認先紅 <!-- speclink-task:tsk_01KXQF7CXQT9SEZH2PCWGCF39Z -->
- [x] 2.2 實作選單模型與接線：apps/desktop/src/tray.ts 的 TrayMenuItem 聯集新增 settings 種類、buildTrayModel 於動作區插入該項、toOptions 將其接至「喚起主視窗＋切換設定頁」（與「開啟此變更」同一喚起語意）、TrayStoreApi 介面補 setBoardView；apps/desktop/src/i18n/messages.ts 新增系統匣「設定」繁中文案 key——滿足需求「系統匣圖示與原生選單」的動作區三項定義；驗證 2.1 測試轉綠 <!-- speclink-task:tsk_01KXQF7CXQ8NX2N1CN1FAPP26C -->

## 3. 結束 app 的能力橋接（spec「開啟視窗與結束動作」：自面板結束 app）

- [x] 3.1 撰寫失敗測試：主視窗端收到 tray-panel-action 的 open-settings 動作時喚起主視窗並呼叫 setBoardView("settings")、收到 quit 動作時呼叫結束 app 的 Tauri command（invoke 以測試樁替驗證）——apps/desktop/src/__tests__/tray.test.ts，npm test -w apps/desktop 確認先紅 <!-- speclink-task:tsk_01KXQF7CXQ0883WH33W6BYGWTQ -->
- [x] 3.2 實作 Rust 端命令：apps/desktop/src-tauri/src/lib.rs 新增結束 app 的 Tauri command（對 app.exit 的單行委派，跟隨既有薄包裝模式）並註冊進 invoke_handler；驗證 cargo build --release -p speclink-desktop 編譯通過（行程結束行為屬 GUI 驗證，見 5.2） <!-- speclink-task:tsk_01KXQF7CXQ62BG0J50CKVZNN6A -->
- [x] 3.3 實作前端分派：apps/desktop/src/tray.ts 的 tray-panel-action listener 新增 open-settings 與 quit 兩種動作的分派（open-settings 走喚起主視窗＋切設定頁、quit 呼叫 3.2 的命令）——滿足需求「開啟視窗與結束動作」的「設定」與面板端「結束」行為；驗證 3.1 測試轉綠 <!-- speclink-task:tsk_01KXQF7CXQSTSGSV5N6DQ608AW -->

## 4. 面板動作區塊補齊（spec「面板樣式（macOS）」：動作區塊三項）

- [x] 4.1 撰寫失敗測試：面板動作區塊由上而下渲染「開啟 Speclink」「設定」「結束」三列，點擊各列分別發送 tray-panel-action 的 open-app、open-settings、quit 動作——apps/desktop/src/__tests__/trayPanel.test.tsx，npm test -w apps/desktop 確認先紅 <!-- speclink-task:tsk_01KXQF7CXQPTTSCS66Y2SGKBCX -->
- [x] 4.2 實作面板動作區塊：apps/desktop/src/panel/TrayPanel.tsx 動作區塊新增「設定」「結束」兩列（沿用既有動作列樣式與分割線規則——區塊間分割線仍恰為三條），apps/desktop/src/panel/main.tsx 接線新增兩個動作 props（發送 open-settings 與 quit，沿用 panel 端既有事件通道）；驗證 4.1 測試轉綠 <!-- speclink-task:tsk_01KXQF7CXQ0S5FRHFF8AHFXRN5 -->

## 5. 全量驗證與真實視窗檢查

- [x] 5.1 全量測試與建置通過：npm test -w apps/desktop 全綠、npm run build -w apps/desktop 產出 dist、cargo build --release -p speclink-desktop 編譯成功，無新增警告 <!-- speclink-task:tsk_01KXQF7CXQDJCNYW37QN5MRBEW -->
- [x] 5.2 macOS 真實視窗驗證（jsdom 測不出 tray 原生互動；操作前先確認使用者沒在使用螢幕）：右鍵點擊系統匣圖示面板彈出、再右鍵收合且與左鍵行為一致；面板「設定」喚起主視窗並顯示設定頁；面板「結束」使 app 行程結束、系統匣圖示消失；左鍵開閉不回歸。順帶實測 Ctrl+左鍵是否被回報為右鍵（討論 tray-right-click 的 deferred 項）並將結果記入變更記錄；非 macOS 原生選單的「設定」項僅以本組模型／接線測試覆蓋，Windows 實機檢查留待該平台機器 <!-- speclink-task:tsk_01KXQF7CXQWFMR2BR2C1C7CBPV -->
