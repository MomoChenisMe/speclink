## Why

正典需求「外部變更即時反映」明定外部寫者修改文件後「看板、詳情抽屜與已封存頁 SHALL 秒級自動更新」，且 scenario 寫明「抽屜若開啟亦同步」。實作只做到一半：workspace-changed 觸發的整批 refresh 更新了 store 的清單資料（卡片、抽屜標頭計數），但抽屜的文件內容（proposal／design／tasks／specs 原文與 meta）由元件自持 state、載入 effect 只依賴開啟與 change 名——外部變更後內容不重載。實證：agent 以 speclink task done 勾掉任務 4.2 後，開著的抽屜標頭顯示新計數 10/14、任務清單的 4.2 核取方塊卻仍未勾——「計數新、內容舊」直接違反該需求。同型問題共三處：change 詳情抽屜、討論抽屜（外部 add-round／conclude 後記錄內容不更新）、與尚未掛載的清單展開元件（載入一次即永久快取，連收合重開都不重抓）。另一個隱藏違規：抽屜內我方勾選任務後只重載任務清單、不重載 meta——開工章蓋入後開工歸屬列不出現。

根因是架構性的：刷新信號止步於 store，沒有機制把「世界變了」傳到內容層。修法為單一縫：store 供刷新世代（refresh generation），內容元件的載入 effect 掛上它。

目標使用者與情境：透過 AI 代理跑 SDD 的開發者與旁觀看板的 PO/PM——agent 在旁執行 apply／task done／discuss add-round 時，使用者開著抽屜看進度，期待內容與計數同步跟上。

## What Changes

- store 的整批 refresh 每次遞增一個刷新世代值，經 props 傳入內容型元件——單一機制涵蓋現有與未來的內容檢視。
- change 詳情抽屜：文件內容（proposal／design／tasks／specs、meta）的載入 effect 掛上刷新世代——外部變更後開著的抽屜秒級重載；我方勾選／拖曳完成後 meta 隨任務清單一併重載（開工歸屬列即時出現）；使用者互動進行中（勾選寫回、拖曳）時外部重載讓路，互動結束後補一次重載，不得蓋掉進行中的操作。
- 討論抽屜：討論記錄內容同樣掛上刷新世代——外部 add-round／conclude／促轉後，開著的抽屜四分頁內容秒級更新。
- 清單展開元件（packages/ui 存貨、桌面目前未掛載）：移除「載入一次永久快取”的 undefined 守衛並掛上刷新世代——同一元件庫同一失效契約，避免未來掛載時同病復發。
- desktop-app 規格：MODIFIED「外部變更即時反映」——把「抽屜若開啟亦同步」釘死為內容層級（任務勾選狀態、文件原文、meta 開工歸屬），納入討論抽屜，並明定使用者互動進行中的讓路語意。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 修改（MODIFIED）需求「外部變更即時反映」——自動更新的對象自「呈現」細化為「呈現與已載入內容」：開著的 change 詳情抽屜各分頁、討論抽屜各分頁 SHALL 於外部變更後重載至磁碟現況；新增使用者互動進行中的讓路語意與我方操作後 meta 同步重載。

## Impact

- Affected specs: desktop-app（MODIFIED 一條）
- Affected code:
  - Modified: apps/desktop/src/store.ts（refresh 遞增刷新世代）
  - Modified: apps/desktop/src/App.tsx（世代值傳入兩個抽屜）
  - Modified: packages/ui/src/components/RichDetailDrawer.tsx（內容載入掛世代、meta 隨勾選重載、互動讓路）
  - Modified: packages/ui/src/components/DiscussionDrawer.tsx（內容載入掛世代）
  - Modified: packages/ui/src/components/ChangeListItem.tsx（移除一次性快取守衛、掛世代）
  - Modified: packages/ui/src/__tests__/richDrawer.test.tsx、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/store.test.ts（行為測試同步）
  - New: 無
  - Removed: 無
- 影響的 crate：無——speclink-core／speclink-cli／桌面 Rust 殼零改動，純前端（apps/desktop TS 與 packages/ui）。
- 相容性影響：無 CLI 人眼或 --json 輸出變動，回歸對照不受影響；桌面行為變化僅限「開著的抽屜內容跟上外部變更」——即修正對既有需求的違反。
- 順序前提：desktop-discussion-board（施工中）同樣修改 DiscussionDrawer.tsx 與 App.tsx——本變更 SHALL 於其完成歸檔後再 apply，避免同檔並行施工；且本變更為 task-done-implies-started 任務 5.2(a)（勾首任務後抽屜顯示開工列）的驗收前提，SHALL 排在其真實視窗驗證之前。
