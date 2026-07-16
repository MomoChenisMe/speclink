## 1. 色階單一來源（D3 色階單一來源：進度條深淺階梯抽升至 stage.ts）

- [x] 1.1 撰寫失敗測試：packages/ui/src/stage.ts 匯出階段進度條填色與圖示色對應（Record<Stage, string>，值等同現行看板 STAGE_STYLE 的 primary 深淺階梯：proposed 最淺、in-progress 次之、ready 最深）——斷言匯出存在且三階段值互異、深淺遞進。驗證：npm test -w packages/ui 先紅。 <!-- speclink-task:tsk_01KXN49XD07857PHFT0DWQNXQR -->
- [x] 1.2 實作 stage.ts 色階匯出（與 STAGE_BADGE 同模式、同單一來源註解慣例），packages/ui/src/components/KanbanBoard.tsx 的 STAGE_STYLE 改讀共用匯出——看板行為與視覺零變化。驗證：npm test -w packages/ui 全綠（含既有看板測試不回歸）。 <!-- speclink-task:tsk_01KXN49XD0WDKHYAZH0V81TS2G -->

## 2. 專案 tab 條（D1 專案區＝橫向 tab 條，active 實心主色卡）

- [x] 2.1 撰寫失敗測試：apps/desktop/src/__tests__/trayPanel.test.tsx 新增——專案區呈現為橫向 tab 條（每 tab 含專案名首字母 avatar 與專案名）、作用中專案 tab 帶 data-active="true" 與實心主色樣式（bg-primary）、非作用中 tab 無實心底、點擊 tab 觸發 onOpenProject(root)、tab 容器帶橫向捲動樣式（overflow-x）且隱藏捲軸。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXN49XD0EV4PZX5MZQCW12P1 -->
- [x] 2.2 實作 apps/desktop/src/panel/TrayPanel.tsx：垂直專案列（打勾＋名稱）改為橫向 tab 條，行為如 2.1 所述；點 tab 沿用既有 open-project 回呼（原地切換、不喚主視窗——回呼語意不變，僅呈現改變），滿足修訂後規格「面板樣式（macOS）」的 tab 條與原地切換場景。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXN49XD0914D92T1JAT3XC2B -->

## 3. 分區卡片化與主色加量（D2 分區卡片化：一分區一卡，疊在 vibrancy 上）

- [x] 3.1 撰寫失敗測試：生命週期與討論分區各自包在圓角半透明卡片容器內（可斷言容器 class 含 rounded 與半透明底 token）、面板不再輸出 hr 分隔線、分區標題圖示帶主色 class、變更列進度條填色依該列階段套用 1.2 的共用色階（提案中與進行中的變更其進度條 class 互異且對應階梯）。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXN49XD0DTZGTZ7NVX3BS78V -->
- [x] 3.2 實作 TrayPanel.tsx 分區卡片容器（半透明底以主題 token 取色、毛玻璃可透出）、分區標題放大與主色圖示、進度條套用共用色階、面板底加極淡 teal 漸層 wash。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXN49XD0075V6Y12Y722PQKA -->

## 4. 修復複製鈕自動 focus（D4 focus 修復：複製鈕退出 tab 順序（前端解））

- [x] 4.1 撰寫失敗測試：TrayPanel 的複製鈕（CopyButton）帶 tabIndex=-1（不可經 Tab 鍵聚焦），且點擊仍觸發 onCopy 並短暫顯示勾號回饋（既有複製案例不回歸）。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXN49XD0133R3MJ3Z488MGDC -->
- [x] 4.2 實作 CopyButton tabIndex=-1。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXN49XD0BSA6TFZZ9VPWKKYE -->

## 5. 建置與真實視窗驗證（macOS）

- [x] 5.1 前端與桌面殼層建置成功：npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop 皆零錯誤（建置前先關閉執行中的 app，避免 linker 存取被拒）。驗證：兩建置指令 exit 0。 <!-- speclink-task:tsk_01KXN49XD0BH8B8CQHZ0D42C21 -->
- [x] 5.2 真實視窗驗證（jsdom 測不出視覺與焦點；操作前先確認使用者未在使用螢幕）：啟動 release app 開啟系統匣面板並截圖檢視——毛玻璃上可見分區卡片層次與 teal 色階、tab 條呈現且點擊後下方內容原地切換（主視窗未被喚起、面板未收合）、開啟瞬間無藍色 focus ring；深色與淺色模式各驗一次——滿足規格「面板樣式（macOS）」修訂後全部場景。若 focus ring 仍出現，套用 design D4 後備（apps/desktop/src/panel/main.tsx 於視窗 focus 時 blur activeElement）後重驗至通過。驗證：截圖逐項核對上述可觀察行為。 <!-- speclink-task:tsk_01KXN49XD02Z924MHH1XAC1TG6 -->
- [x] 5.3 寬度評估（D5 面板寬度遞延）：依 5.2 截圖判斷 tab 條與卡片內距是否過擠——過擠則同步放寬 apps/desktop/src-tauri/src/panel.rs 與 apps/desktop/src/panel/main.tsx 的面板寬度常數（兩處同值）並重跑 5.1–5.2；不擠則維持現值並於任務完成註記結果。驗證：最終截圖無過擠且兩處寬度常數同值。 <!-- speclink-task:tsk_01KXN49XD0G6BNRTNV2JCE3JBR -->

## 6. 快速加入專案（D7 tab 條尾端快速加入專案）

- [x] 6.1 撰寫失敗測試：apps/desktop/src/__tests__/trayPanel.test.tsx——tab 條尾端有「加入專案」動作項（可及名稱「加入專案」），點擊觸發 onAddProject 回呼且不觸發 onOpenProject；apps/desktop/src/__tests__/tray.test.ts——面板動作事件 kind「add-project」由接線層轉呼資料夾選擇流程（openProjectViaDialog）。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXNK64P793GB6YH15JWHJZCP -->
- [x] 6.2 實作：TrayPanel.tsx tab 條尾端加入動作項（加號圖示 tab 形態、hover 淡主色底、div 非 button 守 D4 焦點約束）；apps/desktop/src/panel/main.tsx 接 onAddProject → act("add-project")；apps/desktop/src/tray.ts 接線 add-project → openProjectViaDialog；apps/desktop/src/i18n/messages.ts 新增「加入專案」鍵（zh-TW 與 en，key 集合相等測試把關）——滿足規格「面板樣式（macOS）」的快速加入專案場景。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXNKARNVV30VYNWW9HG4VS7V -->

## 7. 分區計數與空狀態卡（D8 分區計數徽章與空狀態卡最小高度）

- [x] 7.1 撰寫失敗測試：各分區標題帶項目計數徽章（提案中／進行中／已就緒／討論／已轉出，徽章 class 取 STAGE_BADGE 單一來源）；討論零筆時空狀態卡顯示計數 0、帶最小高度 class 且內容垂直置中；全無變更空狀態卡同樣帶最小高度。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXNKDT3TPM62AVDMRSXJ97J6 -->
- [x] 7.2 實作：SectionHeader 增計數徽章、空狀態卡最小高度與垂直置中、討論空狀態改「分區標題＋計數 0」同構呈現——滿足規格「面板樣式（macOS）」的分區計數場景。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXNKFQQ45J129B4S238YDNTM -->

## 8. 補充功能的建置與真實視窗驗證（macOS）

- [x] 8.1 重建與重裝：npm run build -w apps/desktop、tauri build（--bundles app）皆 exit 0，重裝 /Applications 並重啟；使用者實機核對——tab 條尾端加入專案項開啟資料夾選擇器且選定後專案入列切換、各分區標題計數正確（與看板一致）、討論 0 空狀態卡比例正常、既有行為不回歸——毛玻璃透感（D6 vibrancy 材質改 HudWindow）、無 focus ring、複製回饋。驗證：使用者確認清單全過。

## 9. 原生對話框在地化（D9 原生對話框在地化宣告）

- [x] 9.1 新增 apps/desktop/src-tauri/Info.plist 部分檔：宣告 CFBundleLocalizations（zh-Hant、en）與 CFBundleAllowMixedLocalizations，使 macOS 原生對話框（加入專案的資料夾選擇器等）跟隨系統語言而非固定英文。驗證：tauri build 後以 plutil 讀 bundle 內 Info.plist，斷言兩鍵存在且 CFBundleLocalizations 含 zh-Hant。 <!-- speclink-task:tsk_01KXNKSNF2DCZH38XE6X2C09B8 -->
- [x] 9.2 重裝實機核對：資料夾選擇器介面為繁體中文（按鈕「打開／取消」等隨系統語言）、app 其餘行為不回歸。驗證：使用者確認。 <!-- speclink-task:tsk_01KXNKMCZPFQMX956M94K1GXEV -->

## 10. 加入專案喚起主視窗（D7 tab 條尾端快速加入專案的實測修訂）

- [x] 10.1 撰寫失敗測試：apps/desktop/src/__tests__/tray.test.ts——add-project 動作除轉呼 openProjectViaDialog 外，先喚起主視窗（斷言 win.show 被呼叫，與 open-change 的 openIn 路徑同款）。驗證：npm test -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KXNKYTDHER5YDHSR3XFPCSET -->
- [x] 10.2 實作：apps/desktop/src/tray.ts 的 add-project 接線改走 openIn（先 openMainWindow 再 openProjectViaDialog）——主視窗位於另一桌面時 macOS 自動切換 Space，選擇器於前景可見。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXNKZSSP0G1TCPANYW1K83A8 -->
- [x] 10.3 重建重裝與實機核對：tauri build（--bundles app）exit 0、重裝重啟；主視窗移至另一桌面後從面板按「加入專案」——桌面自動跳轉至主視窗所在處、選擇器於前景出現（與 9.2 的繁中介面核對同輪進行）。驗證：使用者確認。 <!-- speclink-task:tsk_01KXNMDWJZNEJZ5DJN24G94GRW -->
