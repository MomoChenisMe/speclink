## Summary

把 desktop store 的整批載入旗標 `refreshing` 從「全域布林＋各路徑手動記帳」改為「per-workspace 在途計數導出」，並補上首訪載入失敗的終態呈現，讓「載入失敗」與「真的沒東西」在看板與 tray 面板上可區分。

## Motivation

desktop-loading-skeleton-ux 落地過程中，`refreshing` 的記帳錯誤累計復發四次（探測失敗永久骨架、空窗假空態、開修復頁標了沒人收、同 key 重疊早收旗標）。病根一致：旗標的真相在「載入動作本身」，卻用一個全域布林在旁邊手抄，六個翻頁入口要記得標（現以必填 willRefresh 參數強制表態）、三個載入出口要記得收（現以 activeKey＋世代雙守衛把關）。守衛能擋住已知路徑，擋不住下一個新入口／新出口忘記記帳——結構上「忘記」永遠可能。review 站亦留有對應紀錄：refresh() 三條離開路徑重複清旗標、TraySnapshot 三欄整組穿透（Data Clumps）。

另一筆欠帳：首訪載入失敗後畫面落回與真空態相同的空態文案（讀不到 ≠ 確認是空的）。verify 站已記錄 remote 修復頁不遮看板時的具體情境。

## Proposed Solution

- store 以 per-workspace 在途計數（key → 進行中 refresh 筆數）取代 `refreshing` 布林：refresh 於第一個 await 前計數 +1、settle 一律 -1（單點 finally），骨架條件改「activeKey 有在途 且 !loaded」。計數天生解同 key 重疊與跨 workspace 互清，activeKey 守衛、世代守衛、willRefresh 參數、翻頁預標全數刪除——「忘記記帳」在結構上不再存在。
- TraySnapshot 把三欄（pendingTabKey／workspaceLoaded／workspaceRefreshing）收斂為面板所需的最小導出，載入態納入即時推送面——失敗時面板骨架即時收掉，desktop-loading-skeleton-ux design D5 的去抖例外隨之作廢。
- WorkspaceSnapshot 增首訪載入失敗終態：首訪整批載入失敗且無快取時，看板與 tray 面板分區顯示「載入失敗」提示文案（i18n 新鍵），不再與真空態同貌；成功載入即清除。

## Non-Goals

- 不加重試按鈕或任何新互動——重載沿用既有路徑（重切分頁、watcher、remote online 事件）。
- 不動 remote recovery 狀態機與其遮蔽語意（修復頁不遮看板的 `!activeSession` 條件維持現狀）。
- 不動抽屜文件三態與其失敗收斂（desktop-loading-skeleton-ux 已定案）。
- server-web 不接線（skeleton 基元共用現狀維持）。
- 不處理 review 票其餘 SUGGESTION（抽屜三態重複、計數徽章重複、statusIcon 巢狀等）。

## Alternatives Considered

- 維持布林＋繼續補守衛：已證明每個守衛只擋已知路徑，第五次復發只是時間問題。
- 世代計數判斷「切換後首輪」：desktop-loading-skeleton-ux design D2 已淘汰（過度設計），在途計數是其正確版本——記「有幾發在跑」而非「第幾輪」。
- 失敗終態放 recovery 狀態機：只覆蓋 remote，本地首訪讀取失敗同樣需要區分，故放 workspace 快照層。

## Impact

- Affected specs: desktop-app（ADDED：首訪載入失敗終態呈現）、tray-status-menu（ADDED：面板分區載入失敗終態）
- Affected code:
  - Modified: apps/desktop/src/store.ts、apps/desktop/src/tray.ts、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/App.tsx、apps/desktop/src/i18n/messages.ts、packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/i18n.tsx、apps/desktop/src/__tests__/ 與 packages/ui/src/__tests__/ 對應測試
  - New: （無）
  - Removed: （無——刪除的是 store.ts 內的守衛與參數，非整檔）
- 施工依賴：須待 desktop-loading-skeleton-ux 經 worktree-merge 併回 main 後動工（本案重構其落地的旗標機制）；spec 面無依賴（兩條均為 ADDED 新需求）。
