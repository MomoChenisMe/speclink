## 1. 根因確診（實機 runtime probe）

- [x] 1.1 於面板樣式下實機重現徽章殘留：在 apps/desktop/src/tray.ts 的 store 訂閱去抖 callback（卸選單、關左鍵開選單、寫入徽章三步驟）加上 console 觀測後啟動 app，切換 workspace 並觸發資料變動，確認徽章寫入未落地的具體失敗步驟（頭號嫌疑：卸選單步驟於 macOS 擲錯且 rejection 被 void async 吞掉）。完成條件：console 記錄呈現失敗點證據，並將結論記入本檔完成註記；若失敗點與嫌疑不符，先更新 2.1 測試設計再繼續。 <!-- speclink-task:tsk_01KY68ZG4TSC6YP116ZXCXF100 -->

## 2. 徽章更新修復（需求：macOS 進行中數文字徽章）

- [x] 2.1 紅：在 apps/desktop/src/__tests__/tray.test.ts 新增三個對應 spec 場景的失敗測試——(a) 面板樣式下資料變動後徽章以新值寫入（setTitle 收到新徽章字串），且前置步驟（setMenu）擲錯時徽章寫入仍須發生；(b) 切換作用中專案後徽章反映新作用中專案的進行中變更數；(c) 作用中 remote 專案處於 error 或 restoring 時徽章清空（setTitle 收到 null）。驗證：npm test -w apps/desktop 顯示三案例對現行實作為紅。 <!-- speclink-task:tsk_01KY68ZG4TSYTYRRM8Z1PHGT5V -->
- [x] 2.2 綠：修正 apps/desktop/src/tray.ts 面板樣式的去抖更新流程，使徽章寫入不被前置步驟失敗阻斷，滿足「macOS 進行中數文字徽章」需求的全部場景（隨資料變動更新、隨作用中專案切換更新、error／restoring 隱藏、0 隱藏）。驗證：2.1 三案例轉綠且 npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KY68ZG4TGS0V89GCWKHG2D71 -->
- [x] 2.3 重構：整理去抖 callback 的錯誤處理，消除靜默吞錯（失敗至少留下 console 記錄），行為不變。驗證：npm test -w apps/desktop 仍全綠。 <!-- speclink-task:tsk_01KY68ZG4TQ03FV92JH8FGQHEY -->
- [x] 2.4 孤兒 tray 清理（1.1 確診新增）：tray 以固定 id "speclink-tray" 建立，initTray 於建立前先移除同 id 的既有 tray——webview 重建（重載、視窗重開）再跑 initTray 時，前一個 context 的 tray 無人 dispose 即成殭屍，title 永遠凍在最後值，是「徽章不更新」的實際根因。驗證：tray.test.ts 新增案例斷言 removeById("speclink-tray") 先於 TrayIcon.new 且建立選項帶同 id；npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KY6CZY4P2DFVVTJPW58D6JHM -->

## 3. 恢復卡按鈕文案縮短

- [x] 3.1 紅：更新 apps/desktop/src/__tests__/trayPanel.test.tsx 對恢復卡次要按鈕的文案斷言——繁中「查看問題」、英文 "View issue"。驗證：npm test -w apps/desktop 該案例對現行文案為紅。 <!-- speclink-task:tsk_01KY68ZG4VCQFZQWRVHJZ0FENN -->
- [x] 3.2 綠：修改 apps/desktop/src/i18n/messages.ts 的 tray.recovery.open 兩個 locale 條目（繁中「在 Speclink 中查看問題」→「查看問題」、英文 "View issue in Speclink" → "View issue"）。驗證：3.1 案例轉綠且 npm test -w apps/desktop 全綠；恢復卡排版結構（主鈕全寬＋兩顆次鈕各半欄）未改動。 <!-- speclink-task:tsk_01KY68ZG4VNX8A6FNFFNACD0T8 -->

## 4. 實機驗證（GUI 改動須真實視窗確認）

- [x] 4.1 建置前端（npm run build -w apps/desktop）後於 macOS 實機啟動桌面 app，逐項確認：系統匣圖示旁無任何數字（切換各 workspace、連線失敗狀態下皆無）、選單列僅一個 Speclink 圖示、主視窗分頁上無計數徽章、恢復卡次要按鈕「查看問題」單行呈現不換行。驗證：實機觀察四項皆符合（必要時截圖留證）。 <!-- speclink-task:tsk_01KY6EDR3G42AY4ANNFF5SED60 -->

## 5. 移除系統匣數字徽章（使用者裁定 2026-07-23）

- [x] 5.1 紅：tray.test.ts 改斷言不顯示數字——初始化選項不帶 title、資料變動與點擊圖示皆不呼叫 setTitle；移除徽章值斷言（含 buildTrayModel 的 badge 欄位斷言）。驗證：npm test -w apps/desktop 對現行實作為紅。 <!-- speclink-task:tsk_01KY6E0BG8EZ3Z6W4HSTYG45QT -->
- [x] 5.2 綠：apps/desktop/src/tray.ts 移除 TrayModel.badge 與其計算、建立時 title 選項、syncBadge 及其去抖與點擊呼叫；保留固定 id 孤兒清理與選單步驟錯誤隔離（console 記錄）。驗證：5.1 轉綠且 npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KY6E0BNZ0WHAW0XVH2S9NGJ9 -->

## 6. 移除分頁「待收尾數」徽章（同批裁定）

- [x] 6.1 紅：tabs.test.ts／store.test.ts／App.test.tsx 改斷言——分頁不渲染計數徽章（data-badge 元素不存在）、ProjectTab 無 badge 欄位、pendingWrapUpCount 移除；啟動時背景 local 分頁仍探測路徑、失效轉錯誤態。驗證：npm test -w apps/desktop 對現行實作為紅。 <!-- speclink-task:tsk_01KY6E0BTMBXDTNC714RT83ZG4 -->
- [x] 6.2 綠：移除 apps/desktop/src/tabs.ts 的 pendingWrapUpCount 與 ProjectTab.badge、apps/desktop/src/components/ProjectTabs.tsx 的 TabBadge、apps/desktop/src/store.ts 的徽章寫入（看板刷新與啟動查詢），啟動查詢降為路徑有效性探測；移除 apps/desktop/src/i18n/messages.ts 的 app.tabBadgeTooltip 兩則。驗證：6.1 轉綠且 npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KY68ZG4V0DX3XB4JQPQH14TJ -->
