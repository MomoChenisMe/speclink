## Summary

把桌面 app（apps/desktop）的「變更詳情」與「討論詳情」兩種右側滑出面板改成佔滿主區的詳情頁：點看板卡片後主區整個換成該項目的頁，麵包屑與 Esc 回看板且看板的捲動、搜尋與篩選原樣保留，麵包屑右端 ‹ n／m › 依看板呈現順序切換上一個／下一個。

## Motivation

討論 desktop-ui-redesign 的結論把詳情從「蓋在看板上的面板」改成「主區就是目前在看的東西」：看板、手冊、設定一路到詳情都是頁，疊層問題（面板蓋面板、面板蓋對話框）從根拔掉。前兩刀已把外殼（標題列、圖示列、專案欄、麵包屑）與看板、清單頁換成新版面，詳情面板是主區最後一塊還在用遮罩與 Sheet 的地方，而且目前的面板寬 720px、內容與看板互搶注意力。使用者在互動原型上確認了詳情頁的版面：22px 等寬名稱列、灰字出身列、右側動作、「狀態」卡三列、分頁列作為內容卡的標頭、任務列整列可點。

目標使用者是用桌面 app 看進度、勾任務、審閱提案與討論記錄的開發者與 PO／PM；對應 apply 期（勾任務、看排程與工單）與 discuss／propose 期（讀討論結論、追衍生變更）。

本 change 是 cut 3b 的前半：頁層、導覽與兩種看板宿主的頁。規格與已封存兩種唯讀詳情頁在後半 `desktop-readonly-detail-pages`。

## Proposed Solution

- **頁層取代面板**：主區保留宿主頁（看板）掛載，開詳情時在主區上層渲染詳情頁、宿主頁改為不可見且不可互動（`inert`），因此看板捲動位置、搜尋字串與篩選不必另存。Esc、麵包屑中段（「變更」）或再點同一入口都關頁回宿主。一次只有一頁，沒有瀏覽歷史堆疊（討論延後題在此定案：不經 history）。
- **麵包屑與 ‹ n／m ›**：主區頂列麵包屑成「專案名 / 變更 / 名稱」，右端 ‹ n／m › 與 ⌥↑／⌥↓ 在看板目前呈現順序（含搜尋與篩選）內切換，到頭不循環；焦點在文字輸入時快捷鍵不作用。
- **變更詳情頁**：內容欄置中 768px；標題列（22px 等寬名稱＋複製＋審查／驗證章籤）、灰字出身列（建立者、工具、相對時間、開工日、來自、同源；email 收進提示）、右側「分析」＋依階段主要鈕（進行中「退回提案中」、已就緒「封存」、remote 可認領時「認領」）＋「⋯」（封存、刪除為底部紅字項，依階段停用並附原因）；「狀態」卡三列（任務進度、品質關卡、排程）；分析結果以卡呈現於狀態卡之下；分頁列為內容卡的標頭（卡片標頭式底線分頁、計數徽章），右端「開啟 ▾」對目前分頁的檔案提供「以預設 App 開啟／在 Finder 顯示／複製路徑」（remote 無本機檔案時只剩「複製內容」；排程分頁無檔案不顯示）；任務分頁每個群組一張卡，整列為勾選框的 label（拖曳把手與連結除外，唯讀與寫回中不變）。提案／設計／規格／排程／審查／驗證各分頁的內容渲染與資料流不變。
- **討論詳情頁**：同骨架——slug 標題＋複製＋狀態籤、topic 一行、出身列（建立者、N 輪、建立日）、右側「封存」（保留在途時停用並說明）；「狀態」卡：階段步驟列（討論中 → 已結論 → 已轉出 → 已封存）、保留在途說明列（有 hold 時）、衍生變更籤列（有 promoted_to 時）；分頁 結論／討論過程 N／背景／衍生變更 N＋「開啟 ▾」。
- **新原語 `StatusPill`**：圓角全圓、語意色淡底深字的狀態籤，承載章籤、討論狀態籤與衍生變更的階段籤，第一個消費者即詳情頁。
- **狀態層**：`openDetail`／`openDiscussion` 仍把宿主切回看板、互斥其他詳情、收掉可取消浮層，不可取消浮層開啟中不開頁；新增單一「關閉詳情頁」動作供 Esc 與麵包屑共用。系統匣「開啟此變更」「開啟此討論」走同一條動作直接落到頁。
- **詞彙**：LANGUAGE.md 的「詳情面板」條目改為「詳情頁」，avoid 增列「詳情面板」；UI 文案「點卡片開詳情」改為「點卡片開詳情頁」。
- **退場**：`RichDetailDrawer`、`DiscussionDrawer` 刪除；桌面不再引用 Sheet 原語（Sheet 本身留在 packages/ui 供 server-web 管理面使用，規格與已封存兩種面板於後半 change 退場）。

## Non-Goals

- 規格詳情頁、已封存詳情頁、手冊頁出處所開的規格頁——在 `desktop-readonly-detail-pages`。
- 詳情頁的瀏覽歷史（上一頁回到前一個項目）——定案為不做：一層頁、Esc 回宿主。
- 提案／設計／任務／規格／排程／工單分頁的內容渲染、任務寫回、排程編輯、分析面板內容——全部沿用既有元件與資料流，不改行為。
- 看板卡片、欄位、搜尋與篩選——cut 3a 已定。
- server-web 的 Sheet 用法與 `DetailSheet`。
- 系統匣面板樣式（cut 5）。

## Alternatives Considered

- 把看板卸載、改存捲動位置與搜尋字串再還原——要序列化四欄各自的捲動位置與篩選面板狀態，比保留掛載多一組狀態且容易漏；頁層蓋住並 `inert` 宿主最簡單也最忠於「主區就是目前在看的東西」。
- 以 history（pushState／popstate）做返回——桌面 app 沒有網址列，history 堆疊只會讓 Esc 與麵包屑的語意分歧；討論的延後題在此定案為不經 history。
- 四種詳情頁一次做完——規格詳情頁的需求分段與需求目錄、已封存頁的兩種唯讀變體各自有獨立的解析與測試面，合在一起超過 15 項任務；拆成兩個 change、後者依賴前者的頁層與導覽。

## Impact

- Affected specs: desktop-app（約 30 條含「抽屜」字面的需求改為詳情頁語意，其中名稱含「抽屜」者以 REMOVED＋ADDED 更名）、tray-status-menu（「變更子選單動作」「討論列表」的開啟語意）
- Affected code:
  - New: `packages/ui/src/components/ui/status-pill.tsx`、`packages/ui/src/components/ChangeDetailPage.tsx`、`packages/ui/src/components/DiscussionDetailPage.tsx`、`packages/ui/src/components/DetailHeader.tsx`、`packages/ui/src/__tests__/statusPill.test.tsx`、`packages/ui/src/__tests__/changeDetailPage.test.tsx`、`packages/ui/src/__tests__/discussionDetailPage.test.tsx`、`apps/desktop/src/components/DetailNav.tsx`、`apps/desktop/src/components/DocumentActionsMenu.tsx`、`apps/desktop/src/__tests__/detailNav.test.tsx`、`apps/desktop/src/__tests__/documentActionsMenu.test.tsx`
  - Modified: `packages/ui/src/index.ts`、`packages/ui/src/i18n.tsx`、`packages/ui/src/components/TaskList.tsx`、`packages/ui/src/components/KanbanBoard.tsx`、`packages/ui/src/__tests__/taskList.test.tsx`、`packages/ui/src/__tests__/kanban.test.tsx`、`packages/ui/src/__tests__/planTab.test.tsx`、`packages/ui/src/__tests__/reviewBadge.test.tsx`、`packages/ui/src/__tests__/verifyBadge.test.tsx`、`packages/ui/src/__tests__/improveBadge.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/components/TitleBar.tsx`、`apps/desktop/src/adapter/fsActions.ts`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/core/src/fs_actions.rs`、`apps/desktop/core/src/query.rs`、`apps/desktop/core/src/discussions.rs`、`packages/ui/src/components/ArchivedDrawer.tsx`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/titleBar.test.tsx`、`openspec/LANGUAGE.md`
  - Removed: `packages/ui/src/components/RichDetailDrawer.tsx`、`packages/ui/src/components/DiscussionDrawer.tsx`、`packages/ui/src/__tests__/richDrawer.test.tsx`、`packages/ui/src/__tests__/discussionDrawer.test.tsx`
