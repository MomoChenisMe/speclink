## 1. CopyButton 朗讀區不撐高外層

- [x] 1.1 【Red】在 packages/ui/src/__tests__/copyButton.test.tsx 新增測試：CopyButton 渲染後，role="status" 的隱藏朗讀區同時帶 sr-only、top-0、left-0 三個 class——這是 requirement「變更與討論詳情頁的宿主為看板」情境「宿主頁不推動詳情頁外框」的成因鎖（朗讀區釘在定位容器左上角，不停在捲動區外的深處撐高 desktop 的 main）。驗證：npm test -w packages/ui 只有這條新測試紅，既有 CopyButton 測試仍綠 <!-- speclink-task:tsk_01M4WA9Q5C3WVH99WPRXQQGMJJ -->
- [x] 1.2 【Green】packages/ui/src/components/CopyButton.tsx 的朗讀區 class 由 sr-only 改為 sr-only top-0 left-0；朗讀文字（複製成功時「已複製」、平時空字串）、role="status" 與 aria-live="polite" 不變，按鈕本身的 class 與版面不變。驗證：npm test -w packages/ui 全綠（含 1.1 與既有的 status 宣告測試） <!-- speclink-task:tsk_01M4WA9Q5CXT02XYGBR5FA9R4Z -->
- [x] 1.3 【品質關卡 review round 1】看板捲動區成為定位容器：packages/ui/src/components/BoardColumn.tsx 的卡片捲動區、DiscussionColumn.tsx 的已轉出展開區、KanbanBoard.tsx 的 grid 都加 relative，讓卡片內沒釘位置的 absolute 元素受所在捲動區裁切，不跳出去撐高 desktop 的 main（requirement「變更與討論詳情頁的宿主為看板」：宿主頁內容的定位落在其所屬捲動區或定位容器之內）。先在 packages/ui/src/__tests__/discussionColumn.test.tsx 寫紅燈測試：看板展開已轉出區後，grid、四欄卡片區與已轉出展開區共 6 個捲動區都帶 relative。驗證：npm test -w packages/ui 全綠 <!-- speclink-task:tsk_01M4WDEPSFX713EVHF7DVHC984 -->

## 2. 驗收與回歸

- [x] [M] 2.1 使用者在 Mac 以 root 的 npm run dev:desktop 開一個看板卡片多的專案（「進行中」欄需要在欄內捲動），開啟某個有手動任務的變更詳情頁任務文件，點手動任務提示框中清單最後一個手動任務的「前往」：閱讀區捲到底、該任務列短暫高亮，左欄文件樹頂端仍看得到變更名、主區標題列不動、主區底部沒有白底；再於同一頁按一次 n 鍵與一次工具列「下一個未完成」，結果相同；關閉詳情頁後看板沒有上移（情境「宿主頁不推動詳情頁外框」） <!-- speclink-task:tsk_01M4WA9Q5CQFQ0GNKC6NQEGYD5 -->
- [x] 2.2 整體回歸：npm test -w packages/ui、npm test -w apps/desktop、npm test -w apps/server-web 全綠（desktop 與 server-web 經 CopyButton 與看板元件受影響、程式不改）；node --test scripts/*.test.mjs scripts/*/*.test.mjs 全綠；git status 只出現 proposal Impact 列出的 6 個 packages/ui 檔案與本變更目錄 <!-- speclink-task:tsk_01M4WA9Q5CQTXA97T9S04Y0SJR -->
