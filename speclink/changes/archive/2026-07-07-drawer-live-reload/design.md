## Context

桌面 app 的資料流分兩層：清單資料集中在 Zustand store，workspace-changed 事件觸發 refresh() 整批重拉並同步開著的抽屜項（detailChange／detailDiscussion，故標頭計數即時）；文件內容（proposal／design／tasks／specs 原文、meta、討論記錄）由元件自持 state——RichDetailDrawer 的載入 effect 依賴 [open, name]、DiscussionDrawer 依賴 [open, slug]、ChangeListItem 以 undefined 守衛載入一次永久快取。刷新信號止步於 store，內容層沒有任何失效機制。結果：外部變更後「計數新、內容舊」，違反正典需求「外部變更即時反映」（本文明定詳情抽屜秒級自動更新、scenario 明定「抽屜若開啟亦同步」）。

相依現況：DiscussionDrawer.tsx 與 App.tsx 正由 desktop-discussion-board（施工中）修改；task-done-implies-started 的真實視窗驗收（勾首任務後抽屜顯示開工列）依賴本變更的 meta 重載。

## Goals / Non-Goals

**Goals:**

- 單一失效機制：store 每次 refresh 遞增刷新世代，內容型元件掛上它——外部變更後開著的檢視內容秒級重載至磁碟現況。
- 我方操作（勾選／拖曳）完成後受影響內容（任務清單與 meta）一併重載。
- 使用者互動進行中外部重載讓路，互動結束補載，不蓋掉進行中操作。
- packages/ui 元件庫內同一失效契約（含未掛載的 ChangeListItem），未來檢視不再同病復發。

**Non-Goals:**

- 不做細粒度差異更新（哪份文件變了就只重載那份）——本地 IPC 讀取便宜、抽屜同時僅開一個，整組重載即可，差異追蹤屬過度設計。
- 不動 Rust 側（watcher、去抖、事件形狀維持現狀）。
- 不動已封存頁的展開內容（封存文件不可變，載入一次即正確）。
- 「規格／備忘／設定」頁尚不存在，不在本刀（desktop-config-multiproject 範圍）；其實作時掛同一世代即可。

## Decisions

### D1 刷新世代自 store 經 props 下發

store 新增單調遞增的刷新世代值，refresh() 每次完成遞增；App 把世代值以 prop 傳入 RichDetailDrawer、DiscussionDrawer（與 ChangeListItem 的宿主清單）。內容載入 effect 的依賴自 [open, name] 擴為 [open, name, 世代]。
替代方案：React context 或事件匯流排——多一層抽象、消費端仍是 effect 依賴，無新增能力，否決（禁過度設計）；文件內容收進 store 統一管理——喪失按需載入（清單上所有 change 的文件都拉），且 store 與元件庫耦合加深，否決；effect 改依賴 change 物件 identity——依賴「refresh 恰好換物件」的實作細節，語意隱晦且討論抽屜與未來元件無對應物件，否決。世代值語意明確：「世界變了 N 次」，誰要跟上誰掛依賴。

### D2 抽屜重載的互動讓路與 latest-wins

互動進行中（勾選寫回、拖曳——既有 taskBusy 旗標）到達的世代變化不立即重載；互動結束時比對「已載世代」與「當前世代」，落後即補載一次（latest-wins）。載入回應以其發起世代標記，較舊回應到達時丟棄，不得蓋掉較新內容。我方勾選／拖曳完成後不再單獨手動重讀 tasks.md——操作後的 refresh() 遞增世代即觸發整組重載（tasks 與 meta 一併），移除既有的局部重讀路徑，單一資料流。
替代方案：互動中照樣重載——外部重載會重置拖曳中的視覺與勾選中的樂觀狀態，直接打斷使用者，否決；保留局部 reloadTasks 疊加世代重載——同一資料兩條更新路徑，競態面加倍，否決。

### D3 ChangeListItem 快取語意修正

移除 undefined 守衛的載入一次永久快取：展開時一律重抓，並掛上刷新世代。此元件桌面目前未掛載，本決策的價值是元件庫內同一失效契約——避免 web 刀掛載時同病復發。
替代方案：僅加註警告留待掛載時再修——技術債留在共用元件庫，掛載者未必知情，否決（修正成本一次測試可覆蓋）。

### D4 規格把「同步」釘死到內容層級

MODIFIED「外部變更即時反映」：自動更新對象自「呈現」細化為「呈現與已載入內容」（任務勾選狀態、文件原文、meta 開工歸屬、討論記錄分頁），納入討論抽屜；明定互動讓路語意與 app 內操作後的內容重載。既有三個 scenario 保留（首個 THEN 強化為核取方塊層級），新增討論抽屜與讓路 scenario。
替代方案：不動規格只修實作——「亦同步」的模糊語意已讓違規存活一輪，不釘死會再犯，否決。

## Implementation Contract

**行為（使用者可觀察）**

- 抽屜開著、外部執行 speclink task done：數秒內抽屜任務清單該項核取方塊變勾、標頭計數一致；全程無 app 內操作、無需重開抽屜。
- 抽屜開著、外部執行 speclink in-progress add：數秒內抽屜出現開工者與開工日。
- 討論抽屜開著、外部執行 speclink discuss add-round／conclude：數秒內回合／結論分頁出現新內容，標頭回合數一致。
- 使用者拖曳任務進行中，外部同時改動該 change：拖曳不被打斷；放開後數秒內內容重載至磁碟現況。
- 使用者於抽屜勾選／拖曳完成後：任務清單與 meta 一併更新（無殘留舊 meta）。

**介面／資料形狀**

- store：AppState 新增單調遞增的刷新世代欄位（number，初值 0），refresh() 完成時遞增；不新增任何 dataSource 方法、不改 SpeclinkDataSource 介面。
- 元件 props：RichDetailDrawer、DiscussionDrawer、ChangeList（轉發至 ChangeListItem）各新增可選的世代 prop（number，預設 0——未傳時行為等同現狀，元件庫向後相容）。
- IPC、CLI、引擎介面零變動。

**失敗模式**

- 重載中某份文件讀取失敗（如 change 恰被刪除）：沿現行缺件語意呈現空狀態，不彈錯誤；下一世代重載自然恢復。
- 世代快速連跳（外部批量寫入）：僅最新世代的回應生效（latest-wins），中間回應丟棄；不去抖、不排隊。

**驗收準則**

- npm test -w apps/desktop：store 世代遞增測試；App 傳遞世代至抽屜。
- npm test -w packages/ui：世代變化觸發內容重載（含 meta）；互動中讓路、結束補載；舊世代回應不蓋新內容；ChangeListItem 重展開重抓。
- 真實視窗驗證：上述行為契約前三條實際操作留證。
- 回歸：既有 richDrawer／taskList／kanban 測試全綠（勾選、拖曳行為不變）。

**範圍邊界**

- In scope：apps/desktop/src/store.ts、apps/desktop/src/App.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ChangeListItem.tsx 及對應測試。
- Out of scope：Rust 側（watcher／事件）、已封存頁展開內容、尚不存在的規格／備忘／設定頁、SpeclinkDataSource 介面、CLI 與引擎。

## Risks / Trade-offs

- [與 desktop-discussion-board 同檔並行施工（DiscussionDrawer.tsx、App.tsx）] → 順序閘門：本變更於該刀歸檔後才 apply；tasks 首項為前提檢查。
- [整組重載造成閃爍或捲動位置重置] → 重載僅替換文字 state、不重置分頁與捲動容器；實窗驗證項含觀察無閃爍；若肉眼可見閃爍，重載改為回應到達後單次 setState（不先清空）。
- [兩次載入交錯、舊回應蓋新內容] → D2 的 latest-wins：回應帶發起世代，落後即丟棄；packages/ui 測試以亂序 resolve 釘死。
- [外部高頻寫入時重載風暴] → 後端 watcher 既有去抖已合併事件；前端每世代至多一組讀取，本地 IPC 成本低；不另加節流（禁過度設計）。
- [task-done-implies-started 的實窗驗收依賴本刀] → 排序：本刀先 apply；其 tasks 已加註前提。
