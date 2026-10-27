## 1. 平台固定樣式

- [x] 1.1 【紅】撰寫 apps/desktop/src/__tests__/store.test.ts 平台分流紅測試：store 的 trayStyle 初值由平台決定——navigator 判為 macOS 時為 panel、否則 native-menu（規格「系統匣圖示與原生選單」的平台分流敘述），且不再讀取 localStorage 偏好；panelFallback 仍將樣式退回 native-menu 並浮出錯誤（規格「面板樣式（macOS）」失敗退回場景——tray 接線層依 store 狀態重掛選單的既有測試維持綠）。驗證：npm test -w apps/desktop 新測試紅、其餘綠。 <!-- speclink-task:tsk_01KXMZFWGN8TG0Z0XATN9S0N05 -->
- [x] 1.2 【綠】實作平台分流：apps/desktop/src/tray.ts 樣式來源改為「store 執行期狀態」且初值依平台（isMacOS → panel、否則 native-menu）；apps/desktop/src/store.ts 移除偏好持久化（trayStyle 初值依平台、setTrayStyle 移除、panelFallback 保留——失敗退回原生選單＋錯誤）；刪除 apps/desktop/src/trayStyle.ts 與 apps/desktop/src/__tests__/trayStyle.test.ts。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXMZFWGNEPBW40H3WMG1SYP5 -->

## 2. 設定頁拆卡

- [x] 2.1 【紅】更新 apps/desktop/src/__tests__/settingsView.test.tsx：本機設定簽不再出現「系統匣樣式」卡（任何平台）；面板失敗錯誤（trayPanelError）改以獨立警示行（role=alert）於本機設定簽浮出（規格「面板樣式（macOS）」失敗場景的設定頁面）。驗證：npm test -w apps/desktop 新測試紅。 <!-- speclink-task:tsk_01KXMZFWGN8DE3R3SY1YQ971D2 -->
- [x] 2.2 【綠】實作拆卡（落實 desktop-config 移除「系統匣樣式偏好」需求）：apps/desktop/src/views/SettingsView.tsx 移除 showTrayStyle／trayStyle／onTrayStyleChange props 與樣式卡，保留 trayPanelError 獨立警示行；apps/desktop/src/App.tsx 同步移除傳參；apps/desktop/src/i18n/messages.ts 移除 settings.trayStyle* 鍵（zh-TW／en key 集合維持相等）。驗證：npm test -w apps/desktop 全綠；npm run build -w apps/desktop 成功。 <!-- speclink-task:tsk_01KXMZFWGN43FJKPZSB0NXCSRP -->

## 3. 收尾驗證

- [x] 3.1 macOS 真實視窗驗證：啟動後未動任何設定，點擊系統匣圖示直接得面板（毛玻璃、貼齊、失焦收合、複製回饋皆如前）；設定頁本機設定簽無「系統匣樣式」卡。驗證：cargo build --release -p speclink-desktop 後手動確認——依專案備忘 GUI 改動須真實視窗驗證，操作前確認使用者未在使用螢幕。 <!-- speclink-task:tsk_01KXMZFWGNT02P3Z3652QT1HS6 -->
