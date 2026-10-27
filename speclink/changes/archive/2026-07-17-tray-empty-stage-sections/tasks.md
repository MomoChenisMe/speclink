## 1. 常駐分區測試先行（紅）

- [x] 1.1 於 apps/desktop/src/__tests__/trayPanel.test.tsx 撰寫失敗測試——契約（規格需求「面板樣式（macOS）」的生命週期分區常駐行為）：快照全無變更時，TrayPanel 呈現 panel-section-proposed、panel-section-in-progress、panel-section-ready 三張分區卡，各自的 panel-section-count 顯示 0，且查無 panel-empty-changes 佔位卡。驗證：npm test -w apps/desktop 該測試先失敗（紅）。 <!-- speclink-task:tsk_01KXQ8EABY66DPCS2YRHM8DV0K -->
- [x] 1.2 同檔撰寫失敗測試——契約：快照僅含 1 個進行中變更（無提案中、無已就緒）時，三張分區卡皆呈現、計數依序為 0、1、0，DOM 順序固定為提案中→進行中→已就緒，進行中卡列出該變更列。驗證：npm test -w apps/desktop 該測試先失敗（紅）。 <!-- speclink-task:tsk_01KXQ8EABY9ZYFS0FH2N561507 -->

## 2. 面板實作（綠）

- [x] 2.1 修改 apps/desktop/src/panel/TrayPanel.tsx——行為：移除 staged 派生的空階段過濾與全空佔位卡分支，三個生命週期分區常駐渲染；零筆階段卡沿用討論分區空狀態同構樣式（SectionHeader＋計數 0、最小高度、內容垂直置中）；面板不再引用 tray.noChanges 文案（apps/desktop/src/i18n/messages.ts 的該鍵保留，原生選單 apps/desktop/src/tray.ts 仍使用）。驗證：1.1 與 1.2 測試轉綠，npm test -w apps/desktop 全數通過。 <!-- speclink-task:tsk_01KXQ8EABYCJEPMAXKW1QR5VTG -->
- [x] 2.2 回歸確認原生選單行為不變——契約（規格需求「生命週期分區與變更進度」的原生選單範圍界定）：全無變更時原生選單仍顯示「尚無進行中變更」空狀態、非空階段才有分區標題。驗證：npm test -w apps/desktop 中既有 tray.test.ts 相關測試維持通過，未修改 apps/desktop/src/tray.ts。 <!-- speclink-task:tsk_01KXQ8EABYKD565Q4Q5GG9SRK2 -->

## 3. 重構收尾

- [x] 3.1 重構檢視：TrayPanel.tsx 中生命週期分區與討論分區的空狀態卡樣式若出現重複樣式串，抽為共用常數或元件屬性；不動其他無關程式碼。驗證：npm test -w apps/desktop 維持全綠。 <!-- speclink-task:tsk_01KXQ8EABYH7NVZAG852C70JEH -->

## 4. 區塊重排與分割線（實測後追加，TDD）

- [x] 4.1 於 apps/desktop/src/__tests__/trayPanel.test.tsx 撰寫失敗測試——契約（規格「面板樣式（macOS）」的「區塊順序與分割線」情境）：面板由上而下依序為專案 tab 條、分割線、「討論」分區（存在已轉出討論時其後接「已轉出」分區）、分割線、「提案中」「進行中」「已就緒」分區、分割線、「開啟 Speclink」；data-testid 為 panel-divider 的元素恰三個，且「討論」分區位於「提案中」分區之前（以 compareDocumentPosition 斷言順序）、分區卡之間無分割線。驗證：npm test -w apps/desktop 新測試先失敗（紅）。 <!-- speclink-task:tsk_01KXQ9Z7YHZR61W6C4JYXJCBR5 -->
- [x] 4.2 修改 apps/desktop/src/panel/TrayPanel.tsx——行為：重排 JSX 使討論／已轉出區塊移至生命週期區塊之前、「開啟 Speclink」維持最末，並於專案 tab 條後、討論區塊後、生命週期區塊後各插入一條分割線（低透明度細線、與毛玻璃底相容，非 hr 元素——既有「無 hr 分隔線」測試維持通過）；區塊內部不加線。驗證：4.1 測試轉綠，npm test -w apps/desktop 全數通過。 <!-- speclink-task:tsk_01KXQA2B2KRKDEYP1XF020JQVX -->

## 5. 真實面板驗證

- [x] 5.1 macOS 真實面板驗證——行為：npm run build -w apps/desktop 重建前端 dist 後啟動桌面 app（tauri dev 載入靜態 dist，過期 dist 不會自動重建），確認：(a) 全空專案面板呈現三張生命週期分區卡各計數 0、無佔位卡；(b) 有資料專案空階段卡仍常駐；(c) 區塊順序為討論（＋已轉出）→ 生命週期 → 開啟 Speclink，專案 tab 條下與區塊之間共三條分割線；(d) 面板高度自適應無多餘空白。驗證：截圖確認上述可觀察狀態（操作前先確認使用者未在使用螢幕）。 <!-- speclink-task:tsk_01KXQ8EABYMYA9HWYR4CWWZ9EJ -->
