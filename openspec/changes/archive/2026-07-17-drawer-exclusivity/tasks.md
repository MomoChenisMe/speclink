## 1. detail 抽屜互斥不變量（store 層）

- [x] 1.1 撰寫 detail 抽屜互斥的失敗測試：於 `apps/desktop/src/__tests__/store.test.ts` 新增案例——`openDetail`、`openDiscussion`、`openSpec`、`openArchived` 任一動作執行後，另外三個 detail 欄位（`detailChange`、`detailDiscussion`、`detailSpec`、`detailArchived`）皆為 null（後開者取代先開者），且 `openDetail` 後 `drawerVerb` 歸零的既有行為不受影響。驗證：npm test -w apps/desktop 該批新測試紅燈（既有實作不清其他欄位）。 <!-- speclink-task:tsk_01KXQHKG5RA40VSSQKQDN7VWKZ -->
- [x] 1.2 實作 detail 抽屜互斥：`apps/desktop/src/store.ts` 的四個 open* 動作於設定自身選定欄位時同時清除另外三個 detail 欄位；`openDetail` 既有的 `drawerVerb: null` 清理保留。驗證：1.1 的測試轉綠，npm test -w apps/desktop 全數通過。 <!-- speclink-task:tsk_01KXQHKG5SE2J8G573G88M79KT -->
- [x] 1.3 移除 App 殼層冗餘的手動先關再開：`apps/desktop/src/App.tsx` 抽屜內跳轉（RichDetailDrawer 的 onOpenDiscussion 先呼叫 closeDetail、DiscussionDrawer 的 onOpenChangeCard 先呼叫 closeDiscussion）改為直接呼叫對應 open*，跳轉結果不變——目標抽屜開啟、來源抽屜關閉；若既有測試斷言「先關再開」的中間狀態則更新為等效的最終狀態斷言。驗證：npm test -w apps/desktop 全數通過（含 App.test.tsx 既有跳轉測試）。 <!-- speclink-task:tsk_01KXQHKG5SDYW9RT65X26YJE8Q -->

## 2. 真實視窗驗證

- [x] 2.1 真實視窗驗證互斥行為：npm run build -w apps/desktop 重建前端 dist 後啟動桌面 app，實測「自系統匣開啟討論抽屜 → 再開啟變更詳情抽屜」，斷言先開者關閉、同時僅一個抽屜可見；再實測抽屜內跳轉（衍生變更 → 變更詳情、來源討論 → 討論抽屜）行為不變。驗證：實際視窗操作與截圖確認，無抽屜疊加。 <!-- speclink-task:tsk_01KXQHKG5SSXVG63SQV6YCASGD -->
