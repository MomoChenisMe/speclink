## 0. 順序前提

- [x] 0.1 確認 desktop-discussion-board 已完成歸檔（speclink list --json 不含該 change）——本刀與其同檔施工（packages/ui/src/components/DiscussionDrawer.tsx、apps/desktop/src/App.tsx），未歸檔前不得開工。驗證：speclink list --json 輸出無 desktop-discussion-board。

## 1. D1 刷新世代自 store 經 props 下發

- [x] 1.1 紅：apps/desktop/src/__tests__/store.test.ts 新增測試——store 含單調遞增的刷新世代欄位（初值 0），每次 refresh() 完成後遞增；apps/desktop/src/__tests__/App.test.tsx 驗證世代值以 prop 傳入 RichDetailDrawer 與 DiscussionDrawer。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 1.2 綠：apps/desktop/src/store.ts 實作世代欄位與遞增，apps/desktop/src/App.tsx 傳遞至兩抽屜；packages/ui 各內容元件新增可選世代 prop（未傳時行為等同現狀，元件庫向後相容）。驗證：npm test -w apps/desktop 全綠。

## 2. D2 抽屜重載的互動讓路與 latest-wins

- [x] 2.1 紅：packages/ui/src/__tests__/richDrawer.test.tsx 新增測試，涵蓋 spec 需求「外部變更即時反映」的內容層級——(a) 世代 prop 變化時開著的抽屜重載 proposal／design／tasks／specs 與 meta（loadDocument／loadMeta 被再次呼叫且畫面反映新內容，含核取方塊與開工歸屬列）；(b) 互動進行中（taskBusy）世代變化不重載，互動結束補載一次；(c) 兩次載入亂序 resolve 時，較舊世代回應不覆蓋較新內容；(d) 勾選／拖曳完成後任務清單與 meta 一併更新（無殘留舊 meta），且不再存在獨立的局部重讀路徑。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 2.2 綠：packages/ui/src/components/RichDetailDrawer.tsx 內容載入 effect 掛上世代、實作讓路與 latest-wins（回應帶發起世代、落後即丟棄）、移除勾選／拖曳後的局部 reloadTasks 改走世代重載；重載僅替換文字 state，不重置分頁與捲動。驗證：2.1 測試全綠，既有 richDrawer／taskList 測試不破。

## 3. 討論抽屜掛世代（design D1 同機制）

- [x] 3.1 紅：packages/ui 討論抽屜測試新增——世代 prop 變化時重載討論記錄，四分頁呈現新內容（涵蓋 spec Scenario「外部推進討論後抽屜內容更新」）。驗證：npm test -w packages/ui 預期紅燈。
- [x] 3.2 綠：packages/ui/src/components/DiscussionDrawer.tsx 載入 effect 掛上世代（latest-wins 同 D2）。驗證：3.1 測試全綠，desktop-discussion-board 既有抽屜測試不破。

## 4. D3 ChangeListItem 快取語意修正

- [x] 4.1 [P] 紅：packages/ui 測試——ChangeListItem 收合後再展開重新抓取文件（不再一次性快取）；世代 prop 變化且展開中時重載。驗證：npm test -w packages/ui 預期紅燈。
- [x] 4.2 [P] 綠：packages/ui/src/components/ChangeListItem.tsx 移除 undefined 一次性快取守衛、掛上世代（經 ChangeList 轉發）。驗證：4.1 測試全綠。

## 5. 整合建置與真實視窗驗證（D4 規格把「同步」釘死到內容層級）

- [x] 5.1 建置：npm run build -w apps/desktop 後 cargo build --release -p speclink-desktop（重建前先關閉執行中的 speclink-desktop.exe）。驗證：建置成功、npm test -w apps/desktop 與 npm test -w packages/ui 全綠。
- [x] 5.2 真實視窗驗證 specs/desktop-app/spec.md 各 Scenario（操作前先確認使用者沒在使用螢幕；以拋棄式測試 change／討論操作，驗證後清理）：(a) 開著詳情抽屜，外部 speclink task done → 數秒內核取方塊變勾、標頭計數一致、無閃爍或捲動重置；(b) 開著詳情抽屜，外部 speclink in-progress add → 數秒內開工列出現；(c) 開著討論抽屜，外部 speclink discuss add-round → 數秒內回合分頁出現新回合。驗證：每項截圖留證。
