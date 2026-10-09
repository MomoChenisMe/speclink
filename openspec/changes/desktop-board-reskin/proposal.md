## Summary

把桌面 app（apps/desktop）看板頁的欄與卡換成新外觀：四欄等寬、間距 12px；每欄是白底細框 16px 圓角的容器卡——頂端 3px 生命週期色相條、白底標頭（色相圖示、名稱、色相淡底計數）下一條細線、卡片區鋪側欄淡灰讓白卡浮出；討論欄底的「已轉出 N ›」改成白底細線分隔的收合列並記住展開狀態；空欄置中灰字；卡片 12px 圓角白底細框無陰影、hover 框變深底微灰；拖曳落點與不合法落點只換成 token 色；搜尋與篩選改為頁標題區右端的全圓 280px 搜尋框＋篩選圖示鈕。這是討論 desktop-ui-redesign 的 cut 3a 看板刀的第一塊（欄與卡）；同一刀另一塊是規格頁、已封存頁與手冊頁（desktop-list-pages-reskin）。看板的資訊架構、欄位判定、拖排與搜尋語意全部不變。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是每天打開的看板。cut 1 把色彩與原語換新、cut 2 把殼換新之後，看板欄還是舊形狀：欄用 `bg-muted/40` 淡灰底加 4px 上框，與設定頁、系統匣面板「白底細框卡＋細線列」的容器語言相反；欄寬夾在 250–360px 之間、靠水平捲動置中，四欄在 1440px 視窗裡兩側留白不均；欄標頭是全大寫小字；討論欄底的「已轉出 N」每次重啟都收回去；搜尋框佔整列寬、與頁標題分成兩層。使用者在原型上已確認「頁、欄、卡」三層的做法：頁白底、欄白底細框、卡片區鋪淡灰、卡白底——欄與面板和設定頁同一套容器規則。

## Proposed Solution

1. **欄容器**：`KanbanBoard` 的四欄改為 `grid grid-cols-4 gap-3` 等寬填滿主區（欄最小 220px，再窄才水平捲動）；每欄 `rounded-2xl border border-border bg-sidebar overflow-hidden`：頂端 3px 色相條（討論 fuchsia、提案中 teal、進行中 sky、已就緒 emerald，取 `stage-*` token）、標頭 `bg-card` 內放色相圖示＋12px 中粗名稱（不再全大寫）＋色相淡底計數徽章、標頭下細線、卡片區 `p-2 gap-2` 在淡灰底上縱向捲動；空欄置中 12px 灰字（「沒有提案中的變更」「沒有進行中的變更」「沒有已就緒的變更」；討論欄沿用「尚無討論」）。skeleton 與載入失敗態不變。
2. **討論欄底收合列**：「已轉出 N ›」改為欄底白底列（上緣細線、右端 › 展開時轉 90°），展開狀態由桌面 app 存於 app 本機（`speclink.board.promotedExpanded`）跨啟動保留；`DiscussionColumn` 以可控 props 承接（不傳時維持元件內狀態、不持久化），promoted 細列改 8px 圓角白底細框。
3. **卡片**：變更卡與討論卡沿用 cut 1 的 `Card size="nested" interactive`（12px 圓角、白底細框無陰影、hover 框變深底微灰）——本塊確認無任何殘留的陰影或 `bg-background/60` 類寫法；拖曳中的浮動卡 `rounded-xl shadow-lg`（浮層才有陰影）；不合法落點降透明度、封存落點浮層的虛線框與底色改 token（`border-primary/60 bg-primary/10`），語意不變。
4. **頁級動作**：`BoardSearchBar` 的輸入改全圓 `rounded-full h-8 w-[280px]`（搜尋圖示在左、清除鈕與命中數在右端內側），篩選改 32px 圖示鈕（啟用計數角標不變），兩者同列放在頁標題區右端；全文比對、模糊比對、篩選面板、快捷鍵全部不變。
5. **規格字面**：desktop-app 新增「看板欄與卡片的容器外觀」需求（欄容器結構、等寬、色相條、標頭、淡灰卡片區、空欄文案、卡片與浮層的外觀）；「看板搜尋過濾卡片」第一段改為頁標題區右端的 280px 全圓搜尋與同列篩選鈕；「討論於看板第 0 欄兩級呈現」的收合列改為跨啟動記憶並描述白底列。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：無。持久化：桌面 app 新增一個 app 本機布林鍵 `speclink.board.promotedExpanded`，不寫入任何專案目錄。相容性：`--json` 與 CLI 輸出不變。

## Non-Goals

- 不改規格頁、已封存頁、手冊頁（desktop-list-pages-reskin）。
- 不改詳情（抽屜→頁是 cut 3b）、設定模式（cut 4）、系統匣（cut 5）。
- 不改欄位判定、拖排寫回、搜尋三層比對與篩選維度、卡片三列骨架、波次章、審查／驗證／改進標示的語意。
- 不做欄收合、欄順序自訂。

## Alternatives Considered

- 頁、欄、卡三層全白——欄與卡靠同一條細框分不出層（討論否決）。
- 看板欄維持灰底——與面板和設定頁的容器語言相反（討論否決）。
- 欄不帶底色只留卡片——標頭與欄底收合列沒有歸屬（討論否決）。
- 把規格、已封存、手冊三頁併進本塊——三頁共用 pager 與列式容器，與欄卡無共用元件，分開各自驗收。

## Impact

- Affected specs: `desktop-app`（ADDED「看板欄與卡片的容器外觀」；MODIFIED「看板搜尋過濾卡片」——搜尋與篩選的位置與形狀；MODIFIED「討論於看板第 0 欄兩級呈現」——收合列外觀與跨啟動記憶）
- Affected code:
  - Modified: `packages/ui/src/components/KanbanBoard.tsx`、`packages/ui/src/components/DiscussionColumn.tsx`、`packages/ui/src/components/BoardSearchBar.tsx`、`packages/ui/src/components/ChangeCard.tsx`、`packages/ui/src/i18n.tsx`、`packages/ui/src/__tests__/kanban.test.tsx`、`packages/ui/src/__tests__/discussionColumn.test.tsx`、`packages/ui/src/__tests__/boardSearchBar.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/__tests__/store.test.ts`
  - New: 無
  - Removed: 無
