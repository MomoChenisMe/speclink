## 1. 看板搜尋狀態（apps/desktop store）

- [x] 1.1 紅：apps/desktop/src/__tests__/store.test.ts 撰寫失敗測試——boardQuery 初始為空字串、setBoardQuery 更新其值、boardQuery 與既有 query（已封存頁）互不影響（各自獨立設值互不覆蓋）。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 1.2 綠：apps/desktop/src/store.ts 新增 boardQuery 狀態與 setBoardQuery action（不入任何 persist 機制，維持「不跨啟動保留」契約），1.1 測試轉綠。驗證：npm test -w apps/desktop 全綠。

## 2. 看板搜尋過濾卡片（packages/ui KanbanBoard）

- [x] 2.1 紅：packages/ui/src/__tests__/kanban.test.tsx 撰寫「看板搜尋過濾卡片」需求的失敗測試——傳入 query 時：變更卡以名稱與摘要、討論卡以主題與 slug 過濾（去頭尾空白、不分大小寫、子字串，對齊 delta spec 的 Example 表）；各欄欄頭計數等於過濾後卡片數；無命中時欄位為空且計數 0、欄結構仍在；query 為空或僅空白時全量呈現。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 2.2 綠：packages/ui/src/components/KanbanBoard.tsx 新增選配 query/onQuery props——提供時於欄位上方渲染搜尋輸入（placeholder「搜尋看板卡片…」，硬編 zh-TW 與現況一致），以比對規則過濾 changes 與 discussions.active 後再分欄與計數，2.1 測試轉綠。驗證：npm test -w packages/ui 全綠。
- [x] 2.3 重構：把 trim＋lowercase＋includes 的比對規則抽為共用純函式，KanbanBoard 與 ArchivedList 共同使用（保證「與已封存頁一致」的規格條款單一真相），兩處既有測試維持綠。驗證：npm test -w packages/ui 全綠。

## 3. App 接線（apps/desktop）

- [x] 3.1 紅：apps/desktop/src/__tests__/App.test.tsx 撰寫失敗測試——看板視圖渲染搜尋輸入，鍵入後看板卡片被過濾且 store.boardQuery 更新；切至已封存頁時其搜尋輸入不含看板字串（規格「搜尋字串不跨啟動保留且與已封存頁獨立」的獨立性；不跨啟動由 1.2 的無 persist 保證）。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 3.2 綠：apps/desktop/src/App.tsx 將 store 的 boardQuery/setBoardQuery 接進 KanbanBoard 的 query/onQuery，3.1 測試轉綠。驗證：npm test -w apps/desktop 全綠。

## 4. 建置與真實視窗驗證

- [x] 4.1 前端與桌面殼建置成功且無型別錯誤。驗證：npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop 皆 exit 0（重建前先關閉執行中的 speclink-desktop exe）。
- [x] 4.2 真實視窗驗證（jsdom 測不出的互動；操作前先確認使用者未在使用螢幕）：於臨時測試工作區（非本 repo，備妥數個測試變更與討論）啟動 release exe，實際鍵入關鍵字確認各欄過濾與計數更新、清空還原全量；過濾狀態下點擊卡片開啟詳情抽屜、拖曳已就緒卡至封存落點流程正常（對應 Scenario「過濾狀態下卡片互動不受影響」，封存動作僅作用於測試工作區）；重啟 exe 確認搜尋輸入為空。驗證：CopyFromScreen 截圖逐項檢視相符。
