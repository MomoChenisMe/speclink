## Context

`KanbanBoard.tsx`：欄 `Column` 為 `flex-1 min-w-[250px] max-w-[360px] rounded-xl border-t-4 ${STAGE_BAR} p-2 bg-muted/40`，標頭為圖示＋11px 全大寫灰字＋`STAGE_BADGE` 計數，欄列以 `[justify-content:safe_center]` 置中並可水平捲動；cut 2 已加 `title`／`description` props，搜尋列放在 `PageHeader` 的動作槽（寬 22rem）。`DiscussionColumn.tsx`：上區全卡（`Card size="nested" interactive`，cut 1）、欄底「已轉出 N」收合列（`useState`，不持久化）、promoted 細列 `rounded-md border-border/60 bg-background/60`。`ChangeCard.tsx` 與討論卡已是 `Card size="nested" interactive`。`BoardSearchBar.tsx`：`Input` 填滿寬＋`SlidersHorizontal` 篩選鈕（h-9 w-9）。拖曳浮動卡 `shadow-lg rounded-lg rotate-2`；封存落點浮層 `rounded-xl border-2 border-dashed backdrop-blur-sm`；不合法落點 `data-drop-invalid` 降透明度。色相 token（`stage-*`）與 `STAGE_BAR`／`STAGE_ICON`／`STAGE_BADGE` 對照已由 cut 1 落地。互動原型第 1 張畫板（看板）是視覺基準：欄 `.col{bg:sidebar;border;radius:16px}`、`.colh{bg:card;border-bottom}`、3px 色相條、卡 `.card{bg:card;border;radius:12px}`。

約束：看板資訊架構、欄位判定、拖排與搜尋語意不變；`KanbanBoard` 的 server-web 消費者（不傳 `title`）行為不變；theme 守門（無原生色階）。

## Goals / Non-Goals

**Goals:**

- 欄＝白底細框 16px 圓角容器卡（色相條、白標頭、淡灰卡片區），四欄等寬。
- 討論欄收合列白底細線、展開狀態跨啟動記憶。
- 卡片與浮層只剩 token 色、浮層才有陰影。
- 搜尋 280px 全圓＋篩選圖示鈕在頁標題區右端。

**Non-Goals:**

- 清單頁與手冊頁、詳情頁、設定、系統匣；欄位判定、拖排、搜尋比對；卡片骨架。

## Decisions

### D1 欄容器與等寬

- 欄列：`grid h-full min-h-0 grid-cols-[repeat(4,minmax(220px,1fr))] gap-3 overflow-x-auto`（討論欄缺席時 `grid-cols-[repeat(3,…)]`）；拿掉 `safe_center` 與每欄的 `min-w`／`max-w`。
- 欄殼：新增 `packages/ui/src/components/BoardColumn.tsx`，階段欄與討論欄共用（欄外觀只定義一處），`KanbanBoard` 的 `Column` 移除。`data-column={id}`、`flex h-full min-h-0 flex-col overflow-hidden rounded-2xl border border-border bg-sidebar`；第一個子節點色相條 `h-[3px] shrink-0 ${tone.bar}`（`STAGE_BAR` 已由 cut 1 落地為純 `bg-stage-*`，本刀不改；拿掉的是 `KanbanBoard` 的 `STAGE_STYLE.top`＝`border-t-*` 欄頂色）；色相組 `ColumnTone`（`icon`、`bar`、`iconCls`、`badge`）；標頭 `flex items-center gap-2 bg-card px-3 py-2.5 border-b border-border/70 shrink-0`，含 `Icon`（`STAGE_ICON`）、`h2 text-xs font-semibold text-foreground`（移除 `uppercase tracking-wider`）、`flex-1`、計數 `STAGE_BADGE`；卡片區 `flex-1 min-h-0 overflow-y-auto flex flex-col gap-2 p-2`。
- 空欄：該欄過濾後零卡且非 loading／failed 時渲染 `<p className="px-2 py-6 text-center text-xs text-muted-foreground">`，文案鍵 `board.emptyColumn.proposed`／`board.emptyColumn.in-progress`／`board.emptyColumn.ready`（zh：沒有提案中的變更／沒有進行中的變更／沒有已就緒的變更；en：No proposed changes／No changes in progress／No ready changes）。搜尋無命中時同樣顯示（spec「無命中時顯示空欄與零計數」的「空欄」即此文案）。
- 討論欄共用同一欄殼（`DiscussionColumn` 改用 `BoardColumn`、色相條 `bg-stage-discussion`、標頭取 `DISCUSSION_TONE`）；「尚無討論」文案沿用 `discussion.none`，置中樣式與其他欄一致。

替代方案「全白三層」「灰底欄」「無底色欄」在討論中否決。

### D2 討論欄底收合列與記憶

- 收合列：`button` 全寬 `flex items-center gap-2 bg-card px-3 py-2 text-xs border-t border-border/70 shrink-0`，文字「已轉出 N」（既有鍵 `discussion.promotedBar`）＋右端 `ChevronRight`（展開時 `rotate-90`）；展開內容放在卡片區與收合列之間的有界區塊（`max-h-64 overflow-y-auto px-2 pb-2`，淡灰底上、自捲；不加 `shrink-0`，視窗很矮時可被壓縮、收合列不被欄外框切掉）以 promoted 細列列出——欄內全卡再多也看得到，不會被擠到卡片捲動區最底部；細列改 `rounded-lg border border-border bg-card px-2.5 py-2`。
- 可控 props：`DiscussionColumnProps` 新增 `promotedExpanded`、`onPromotedExpandedChange: (v: boolean) => void`，以聯合型別 `PromotedExpansionProps` 規定兩者同進同出（只給其一在型別層擋下）；皆提供時為受控，皆缺席時內部 `useState(false)`。`KanbanBoardProps` 同名透傳。
- 桌面 app：`store.ts` 新增 `promotedExpanded: boolean`（初值自 `localStorage` 鍵 `speclink.board.promotedExpanded` 讀，壞值視為 false）與 `setPromotedExpanded(v)`（寫回同鍵）；`App.tsx` 傳入。server-web 不傳，維持不持久化。

### D3 卡片與浮層

- 變更卡與討論卡：維持 `Card size="nested" interactive`；確認 `ChangeCard`、討論卡、`PromotedRow` 無 `shadow-*`（拖曳浮動卡除外）、無 `bg-background/60`。
- 拖曳浮動卡（`DragOverlay` 內容）：`rounded-xl shadow-lg rotate-2`（浮層保留陰影）。
- 封存落點浮層：`rounded-2xl border-2 border-dashed border-primary/60 bg-primary/10 text-primary`，`isOver` 時 `bg-primary/20`；位置與不參與欄列佈局不變。
- 不合法落點：`opacity-40`（既有），`data-drop-invalid` 不變。

### D4 頁級動作

- `BoardSearchBar`：輸入 `shape="pill"`（全圓）＋`className="w-[280px] pl-8 pr-2 text-[13px]"`（高度取 `Input` 基底 `h-8`；active 時 `pr-24` 留命中數與清除鈕，en 三位數命中數也放得下），容器 `flex items-center gap-1.5`（不再 `w-full`）；篩選鈕 `variant="toggle" size="icon"`（32px、8px 圓角，展開態由 `aria-expanded` 帶出 `bg-muted text-foreground`）；篩選面板位置與內容不變。
- 原語變體（`openspec/config.yaml` rules.design：呼叫端不得以 className 覆蓋變體的顏色、圓角與陰影）：`packages/ui/src/components/ui/input.tsx` 新增 `shape` 變體（`default` 8px、`pill` 全圓），`packages/ui/src/components/ui/button.tsx` 新增 `toggle` 變體（`rounded-lg text-muted-foreground hover:bg-foreground/5 hover:text-foreground aria-expanded:bg-muted aria-expanded:text-foreground`）；呼叫端只選變體，className 只留寬度與內距。
- `KanbanBoard` 的 `PageHeader` 動作槽拿掉 `w-[22rem]` 包裹，直接放 `searchBar`。
- 搜尋輸入 280px 下的命中數與清除鈕仍在輸入框右端內側；文字過長時輸入框內捲動（原生行為）。

### D5 規格字面

- ADDED「看板欄與卡片的容器外觀」：四欄等寬 12px 間距；欄＝白底細框 16px 圓角容器，3px 色相條、白標頭（圖示、名稱、色相淡底計數）下細線、卡片區鋪側欄淡灰；空欄文案；卡 12px 圓角白底細框無陰影、hover 框深底微灰；拖曳浮動卡才有陰影；落點與不合法落點只用 token 色。
- MODIFIED「看板搜尋過濾卡片」：第一段改為頁標題區右端 280px 全圓搜尋輸入＋同列篩選圖示鈕；其餘段落與 scenario 逐字保留。
- MODIFIED「討論於看板第 0 欄兩級呈現」：收合列段落改為「白底細線列、展開狀態存於 app 本機跨啟動保留」；scenario「欄底收合列就地展開與收合」加一句重啟後維持；其餘逐字保留。

## Implementation Contract

**Behavior**

- 看板四欄等寬填滿主區；每欄白底細框圓角，頂端色相條（討論桃紫、提案中主色、進行中藍、已就緒綠），標頭白底（圖示、名稱、計數）下細線，卡片區淡灰；過濾後零卡的欄顯示置中灰字。
- 卡片白底細框 12px 圓角無陰影，hover 框變深底微灰；拖曳中的浮動卡帶陰影；封存落點浮層與不合法落點以主色淡底／降透明度呈現，語意不變。
- 討論欄底「已轉出 N」為白底列、› 展開旋轉；展開後重啟 app 仍展開（桌面）。
- 頁標題「變更」右端為 280px 全圓搜尋框與篩選圖示鈕，搜尋、篩選、快捷鍵、命中數、清除鈕行為不變。

**Interface / data shape**

- `KanbanBoardProps`／`DiscussionColumnProps` 新增 `promotedExpanded`、`onPromotedExpandedChange`（`PromotedExpansionProps`：兩者同進同出或皆缺席）。
- `STAGE_BAR` 沿用 cut 1 的純 `bg-stage-*`（消費者：`BoardColumn` 色相條、`ChangeCard` 的 `barClass` 進度條）。
- `BoardColumn`（`id`、`title`、`tone: ColumnTone`、`count: number | null`、`empty: string | null`、`footer?`、`children`）：`packages/ui` 內部元件，不對外匯出。
- 原語：`Input` 新增 `shape?: "default" | "pill"`；`Button` 新增 `variant="toggle"`。
- i18n 新鍵（`packages/ui/src/i18n.tsx` 兩語系）：`board.emptyColumn.proposed`、`board.emptyColumn.in-progress`、`board.emptyColumn.ready`。
- 桌面持久化鍵：`speclink.board.promotedExpanded`（"true"／"false"）。

**Failure modes**

- `localStorage` 不可用或壞值：展開狀態視為 false、不拋錯。
- 欄寬低於 220px×4：欄列水平捲動，不壓縮卡片。

**Acceptance criteria**

- `npm test -w packages/ui`、`npm test -w apps/desktop` 全綠；`theme.test.ts` 守門綠。
- `kanban.test.tsx`：欄容器 class 含 `rounded-2xl`、`bg-sidebar`，色相條元素帶 `bg-stage-proposed` 等三種，標頭含 `bg-card`，空欄文案三種，欄列為 grid 四欄；`discussionColumn.test.tsx`：收合列 `bg-card`、受控 props 下點擊呼叫 `onPromotedExpandedChange(true)`、非受控仍可展開；`boardSearchBar.test.tsx`：輸入 class 含 `rounded-full` 與 `w-[280px]`、篩選鈕展開態走 `aria-expanded` 變體，既有搜尋／篩選案例通過；`ui.test.tsx`：`Input shape="pill"` 與 `Button variant="toggle"` 的 class；`store.test.ts`：`promotedExpanded` 讀寫 localStorage。
- 手動：淺色與深色各看一次看板；拖一張已就緒卡看浮層與浮動卡；展開已轉出後重啟。

**Scope boundaries**

- In：`KanbanBoard`、`DiscussionColumn`、`BoardSearchBar` 的外觀 class 與空欄文案、共用欄殼 `BoardColumn`、`Input`／`Button` 各一個新變體、i18n 三鍵、desktop store 一個布林鍵、對應測試、三條 spec 字面；`ChangeCard` 只確認無陰影、不改。
- Out：清單頁、手冊頁、詳情、設定、系統匣；欄位判定、拖排寫回、搜尋比對、篩選維度、卡片骨架與標示。

## Risks / Trade-offs

- [等寬 grid 取代置中可捲欄列，窄視窗時版面變化] → 220px 下限＋水平捲動；視窗預設 1440 下四欄各約 300px。
- [欄頂色條從 `border-t-*` 改為色相條元素，影響既有斷言] → `kanban.test.tsx` 與 `discussionColumn.test.tsx` 同批改為斷言首子節點 `bg-stage-*`；`STAGE_BAR` 值不變，`stage.test.ts` 不動。
- [展開狀態持久化只在桌面] → ui 元件受控／非受控雙模式，server-web 行為不變。
- [回歸對照] → 不動 CLI、golden、`--json`；ui 與 desktop 測試為基準。
- [跨平台] → 純 CSS／TS。

## Migration Plan

無資料遷移；新持久化鍵缺席時視為收合。回滾即還原本刀 commit。

## Open Questions

無。
