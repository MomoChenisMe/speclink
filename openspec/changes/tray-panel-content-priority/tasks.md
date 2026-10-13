## 1. 收合狀態記憶模組

- [ ] 1.1 先寫失敗測試，鎖定決策「收合狀態記在面板本機、所有專案共用一份」的讀寫契約：鍵缺席、值非 JSON、值非陣列、localStorage 丟例外時讀回空集合（全部展開）；陣列中的未知 id 被忽略；寫入以 JSON 陣列整份覆寫，丟例外時靜默。測試加在 apps/desktop/src/__tests__/trayPanel.test.tsx 的新 describe，執行 `npm test -w apps/desktop -- trayPanel` 確認紅燈。 <!-- speclink-task:tsk_01M4SCV3KAQQ9J7QZ37GWZQ8QV -->
- [ ] 1.2 新增 apps/desktop/src/panel/sectionCollapse.ts：匯出鍵常數 `speclink.tray.collapsedSections`、讀取函式（回傳 proposed／in-progress／ready／discussions 中收合的集合）與寫入函式，形狀比照 apps/desktop/src/panel/promotedExpanded.ts；依 `speclink instructions --skill audit` 的 sharp-edges 清單檢查讀取路徑，確認壞值與未知 id 不會讓面板丟例外。驗證：`npm test -w apps/desktop -- trayPanel` 中 1.1 的測試轉綠。 <!-- speclink-task:tsk_01M4SCV3KATRQ3N83PPE6YT1VV -->

## 2. 面板樣式（macOS）：重排與動作區塊卡單列

- [ ] 2.1 先改寫失敗測試，對應「面板樣式（macOS）」需求的新行為：區塊順序測試改為提案中、進行中、已就緒三張卡在討論卡之前、動作區塊卡在最末；動作區塊測試改為卡內有「開啟 Speclink」文字與三個 aria-label 依序為「專案設定」「設定」「結束」的按鈕（各帶同名 title、tabIndex 為 -1），點擊各自觸發開啟專案設定、開啟設定、結束的回呼；其餘以可見文字查找「專案設定」「設定」「結束」的既有斷言改以按鈕名稱查找。測試在 apps/desktop/src/__tests__/trayPanel.test.tsx，執行 `npm test -w apps/desktop -- trayPanel` 確認新斷言紅燈。 <!-- speclink-task:tsk_01M4SCV3KAME7FNPM2VWB26V3W -->
- [ ] 2.2 實作決策「中段重排：生命週期卡在前、討論卡在後」：apps/desktop/src/panel/TrayPanel.tsx 的可捲中段改為提案中、進行中、已就緒、討論卡的順序，stale 狀態列仍在中段最上，復原卡分支不變。驗證：`npm test -w apps/desktop -- trayPanel` 的區塊順序測試轉綠。 <!-- speclink-task:tsk_01M4SCV3KAAQQ697RQ0KGG5AD9 -->
- [ ] 2.3 實作決策「動作區塊卡改為單列：文字鈕＋三顆圖示鈕」：apps/desktop/src/panel/TrayPanel.tsx 的動作區塊卡改為一列，左為「開啟 Speclink」文字鈕（佔滿其餘寬度），右為專案設定（SlidersHorizontal）、設定（Settings）、結束（Power）三顆 button（type="button"、tabIndex 為 -1、aria-label 與 title 取 app.navProjectSettings／tray.settings／tray.quit 詞條），「結束」前多一段間距，四個動作的回呼不變。驗證：`npm test -w apps/desktop -- trayPanel` 的動作區塊測試轉綠。 <!-- speclink-task:tsk_01M4SCV3KAMWZ6N8HTEYZHKRS2 -->
- [ ] 2.4 先改寫失敗測試，對應「面板樣式（macOS）」需求的補光層濃度：既有「根層不疊漸層、以側欄面色約九成補光」測試改為斷言 panel-root 的 class 含 `bg-sidebar/60`、不含 `bg-sidebar/90`；並斷言專案 tab 條卡、動作區塊卡與各內容卡仍含 `bg-card`（白卡不透明不變）。測試在 apps/desktop/src/__tests__/trayPanel.test.tsx，執行 `npm test -w apps/desktop -- trayPanel` 確認紅燈。 <!-- speclink-task:tsk_01M4SE3R01PK1CB2PACXN7S2NW -->
- [ ] 2.5 實作決策「毛玻璃找回：補光層調回約六成、白卡不變」：apps/desktop/src/panel/TrayPanel.tsx 的 panel-root 補光層由 `bg-sidebar/90` 改為 `bg-sidebar/60`，卡片樣式與 Rust 端 vibrancy 不動。驗證：`npm test -w apps/desktop -- trayPanel` 中 2.4 的測試轉綠。 <!-- speclink-task:tsk_01M4SE3R0147AJ5XTA32EYPGE5 -->

## 3. 面板內容卡收合

- [ ] 3.1 先寫失敗測試，對應「面板內容卡收合」需求：(a) 點「進行中」標題後變更列消失、aria-expanded 為 false、計數仍為 3，再點一次恢復；(b) 收合「討論」後重新掛載面板、再送入另一個作用中專案的快照，討論卡仍收合，localStorage 的值為含 discussions 的 JSON 陣列；(c) localStorage 存壞值時四張卡全部展開；(d) 收合「已就緒」後送入多一筆已就緒變更的快照，卡仍收合、計數由 0 變 1；(e) 收合的空狀態卡不帶空狀態最小高度、標題下沒有細線；(f) 展開「還有 N 個…」與「已轉出」收合列後收起再展開討論卡，溢出列回到未展開、已轉出收合列維持展開；(g) 專案 tab 條卡、動作區塊卡、復原卡、stale 狀態列沒有收合觸發；(h) 收合觸發的 tabIndex 為 -1。測試在 apps/desktop/src/__tests__/trayPanel.test.tsx，執行 `npm test -w apps/desktop -- trayPanel` 確認紅燈。 <!-- speclink-task:tsk_01M4SCV3KAARCVATWSCNHXNV6S -->
- [ ] 3.2 實作決策「收合觸發沿用已轉出收合列的模式」：apps/desktop/src/panel/TrayPanel.tsx 中四張內容卡的 SectionHeader 外包可點容器（role="button"、aria-expanded、tabIndex 為 -1、Enter／Space 可切換），方向箭頭以 SectionHeader 的 action 插槽放 ChevronRight（展開時旋轉 90 度），不改 @speclink/ui 的 SectionHeader；面板以 1.2 的讀取函式初始化收合集合，點擊時更新狀態並以寫入函式寫回。驗證：`npm test -w apps/desktop -- trayPanel` 中 3.1 的 (a)(b)(c)(g)(h) 轉綠。 <!-- speclink-task:tsk_01M4SCV3KAH7QRQVC2AGX46JTR -->
- [ ] 3.3 實作決策「收合中的卡只留標題一行、不自動展開」：apps/desktop/src/panel/TrayPanel.tsx 中收合的卡只渲染標題（不渲染列、骨架、失敗提示、溢出列、已轉出列與已轉出收合列，不套空狀態最小高度、標題不畫細線），收合集合只由點擊改變、快照更新不影響。驗證：`npm test -w apps/desktop -- trayPanel` 中 3.1 的 (d)(e)(f) 轉綠，整個 trayPanel 測試檔全綠。 <!-- speclink-task:tsk_01M4SCV3KAXJ0V238NMBM5CHC2 -->
- [ ] 3.4 先改寫失敗測試，對應「討論列表」與「討論於看板第 0 欄兩級呈現」需求的箭頭方向（決策「收合箭頭統一規則：收合 ›、展開時指向展開出來的內容」）：apps/desktop/src/__tests__/trayPanel.test.tsx 既有「已轉出收合列展開後箭頭含 rotate-90」斷言改為展開後 class 含 `-rotate-90` 且以空白切開後不含單獨的 `rotate-90`、收合時兩者皆無；packages/ui/src/__tests__/discussionColumn.test.tsx 欄底收合列的箭頭斷言同樣改寫。執行 `npm test -w apps/desktop -- trayPanel` 與 `npm test -w packages/ui -- discussionColumn` 確認新斷言紅燈。 <!-- speclink-task:tsk_01M4SFQZQVNEG9HRQ2X6HMJWK9 -->
- [ ] 3.5 實作決策「收合箭頭統一規則：收合 ›、展開時指向展開出來的內容」：apps/desktop/src/panel/TrayPanel.tsx 的「已轉出」收合列與 packages/ui/src/components/DiscussionColumn.tsx 的欄底收合列，ChevronRight 展開時的旋轉由 90 度改為 -90 度（朝上），收合時不旋轉，版面與展開位置不變；內容卡標題的收合箭頭維持展開朝下。驗證：`npm test -w apps/desktop -- trayPanel` 與 `npm test -w packages/ui -- discussionColumn` 中 3.4 的斷言轉綠。 <!-- speclink-task:tsk_01M4SFQZQVW54FM5VE2K9TYFTW -->

## 4. 收尾驗證

- [ ] [M] 4.1 使用者於 macOS 以 root 的 `npm run dev:desktop` 開啟系統匣面板，切到討論很多的專案（例如 wadpilot）確認：打開即看得到提案中、進行中、已就緒三張卡；動作區塊卡為單列，滑過三顆圖示鈕會出現「專案設定」「設定」「結束」；收起「討論」後面板變矮，切到其他專案、關閉再開啟面板後仍收合；分別在色彩鮮明的淺色桌布與深色桌布（或全螢幕深色 IDE）前開啟面板，面板外框與卡片縫隙看得出毛玻璃透出的色調，卡片內文字清楚、未偏暗；展開系統匣與看板的「已轉出」收合列時箭頭朝上、收合時為 ›。 <!-- speclink-task:tsk_01M4SCV3KAQPYBS6SZVH5XK74X -->
- [ ] 4.2 跑受影響面的完整測試確認沒有回歸：`npm test -w apps/desktop` 與 `npm test -w packages/ui` 全綠，且 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（詞彙守門、中英對照與文件連結檢查）。 <!-- speclink-task:tsk_01M4SCV3KAF1E57FY5E2VPX863 -->
