## Context

`SpecList.tsx`：搜尋 `Input`＋計數徽章＋`SpecCard`（`rounded-lg border bg-card p-3 hover:shadow-md`，標題＋複製鈕、需求數／溯源數 icon＋數字、相對時間）堆疊於可捲容器，`ListPager`（上一頁／第 N／M 頁／下一頁，`PAGE_SIZE = 20`）在底部；點卡呼叫 `onOpen`（桌面接 `store.openSpec` 開規格抽屜）。`ArchivedList.tsx`：搜尋 `Input`、`Tabs`（變更／討論，`TabsTrigger` 帶計數）、各節 `ArchivedCard`／`ArchivedDiscussionCard`（同款卡片）＋各自 `ListPager`。`ManualPage.tsx`：`ManualTree`（`w-64`、搜尋、分區、`manual.stale` 琥珀字、未入冊提示）＋中欄（固定頁首 h1、`READING_COLUMN_CLS` 捲動內文、固定 footer：出處籤 `SourceChipRow` 風格＋上一頁／下一頁）＋`ManualToc`（`w-52`，lg 以上顯示）。cut 1 提供 `Card`、`Tabs variant="card"`（`TabsList` 帶 `actions` 槽、內容卡 `rounded-t-none border-t-0` 接在下方）、`Badge`、`Select`；cut 2 提供 `PageHeader`。互動原型第 12（規格頁）、14（已封存頁）、15（手冊頁）張畫板是視覺基準：`.row{display:flex;gap:16px;padding:12px 0;border-bottom:細線}`、底部工具列「第 1–8 筆，共 86 筆」＋「每頁 8 個 ▾」＋頁碼＋跳頁；手冊底列白底上緣細線。

約束：排序（最新在前）、搜尋（名稱子字串／兩節同時過濾）、每頁預設 20、兩節頁碼獨立、換頁回頂、越界鉗制、列上資料與清單 payload 不變；手冊的閱讀序、過期標示、出處跳規格、載入態不變；server-web 不消費這三個元件。

## Goals / Non-Goals

**Goals:**

- 三頁套同一套版面：頁標題區、白底細框列表卡、細線列、固定工具列。
- 換頁工具列有筆數範圍、每頁筆數、第一頁／最後一頁鈕、頁碼與跳頁。
- 每頁筆數跨啟動記住（實機試用後追加，見 D7）。
- 已封存分頁為卡片標頭式；手冊三欄寬度與固定底列對齊 768px。

**Non-Goals:**

- 看板、詳情頁化、設定、系統匣；排序與搜尋規則；頁碼與搜尋字串的記憶；server-web 的每頁筆數記憶。

## Decisions

### D1 `ListRow` 與 `ListCard`

- `packages/ui/src/components/ListRow.tsx`：`ListRow({ title, titleClassName?, copyValue?, copyLabel?, description?, leading?, meta?, onClick, children?, ...dataAttrs })`——`div role="button" tabIndex={0}` `group flex items-center gap-4 px-4 py-3 border-b border-border/70 last:border-b-0 cursor-pointer hover:bg-foreground/[0.03] focus-visible:bg-foreground/[0.03] outline-none`；`leading` 槽（已封存列的日期）在最左 `shrink-0 text-xs text-muted-foreground tabular-nums`；中段 `min-w-0 flex-1`：標題列 `flex items-center gap-1`（標題 `truncate font-mono text-sm font-semibold`＋`CopyButton`（`REVEAL_ON_HOVER`）＋`children`（標題旁的章，如改進小章）），描述 `truncate text-[13px] text-muted-foreground`（缺席時不渲染，列退回單行）；右端 `meta` 槽 `flex shrink-0 items-center gap-3 text-xs text-muted-foreground` 後接 `ChevronRight h-4 w-4 text-muted-foreground`。Enter／Space 觸發 `onClick`；複製鈕點擊不冒泡。
- `packages/ui/src/components/ListCard.tsx`：`ListCard({ header?, footer?, children, className? })`——`Card` 外框（`rounded-2xl border bg-card p-0`，`flex flex-1 min-h-0 flex-col overflow-hidden`），`header` 槽（已封存頁的卡片標頭分頁列）在頂部；`children` 在 `flex-1 min-h-0 overflow-y-auto`（`data-list-scroll`，換頁後捲回頂部的目標）；`footer` 槽 `shrink-0 border-t border-border`。空清單時呼叫端把空態文案放進 `children`（置中灰字）。
- 兩者自 `index.ts` 匯出；`uiSingleSource` 守門涵蓋。

### D2 `ListPager` 工具列

- props 改為 `{ page, pageCount, total, pageSize, pageSizeOptions?: number[], onPage, onPageSize }`；`total === 0` 時回 null；渲染 `flex items-center justify-between gap-3 px-4 py-2 text-xs text-muted-foreground`：
  - 左：`pager.range`（zh「第 {a}–{b} 筆，共 {n} 筆」、en「{a}–{b} of {n}」；a＝(page−1)×pageSize+1、b＝min(page×pageSize, total)）＋`Select`（`pager.perPage`：zh「每頁 {n} 個」、en「{n} per page」；選項預設 `[20, 50, 100]`）——改每頁筆數時呼叫 `onPageSize(n)`，呼叫端重算 `pageCount` 並把頁碼鉗制。
  - 右（`pageCount > 1` 時）：« 鈕（`ChevronsLeft`，`onPage(1)`）、‹ 鈕、頁碼鈕列、› 鈕、» 鈕（`ChevronsRight`，`onPage(pageCount)`）、「跳到 [__] 頁」：頁碼視窗演算法 `pageWindow(page, pageCount)`——總頁數 ≤ 7 全列；否則恆列 1 與 M，中段為 page−1..page+1，與兩端不相鄰處插入 `…`（純函式、單元測試）；作用中頁碼 `bg-primary/12 text-primary font-medium`，其餘 ghost；« 與 ‹ 於第 1 頁停用、› 與 » 於末頁停用；« » 的 aria-label 為 `pager.first`（zh「第一頁」、en「First page」）與 `pager.last`（zh「最後一頁」、en「Last page」）；跳頁輸入 `type="number" min=1 max=pageCount` 寬 3em，Enter 或失焦時 `onPage(clamp(value))`、非數字忽略；aria-label `pager.jump`（zh「跳到第幾頁」）、輸入框前後的可見文字來自 `pager.jumpTo`（zh「跳到 {n} 頁」）、頁碼鈕 aria-label `pager.pageN`（「第 {n} 頁」）；每頁筆數下拉的 aria-label 即其可見文字（`pager.perPage`）。
- `PAGE_SIZE` 常數保留為預設值，選項陣列匯出為 `PAGE_SIZE_OPTIONS`（`[20, 50, 100]`，桌面 store 驗證記住的值用）；`SpecList`／`ArchivedList` 的每頁筆數可受控（D7），未受控時各自持有 `pageSize` state（預設 20）；過濾結果或每頁筆數改變時頁碼鉗制至末頁、搜尋變更回第 1 頁（既有）。

### D3 規格頁

- 結構：`<PageHeader title description actions={搜尋框} />` → `<ListCard footer={<ListPager …/>}>`（列或空態）。`SpecList` 新增 props `title`、`description`（桌面傳「規格」與「正式規格一覽；點一列看全文與來源變更。」）。
- 搜尋框：`Input` `h-8 w-[280px] rounded-full pl-8`＋左側 `Search` 圖示（與看板同款；抽成 `packages/ui/src/components/SearchField.tsx` 讓看板與兩個清單共用——`SearchField({ value, onChange, placeholder, className? })`，`BoardSearchBar` 於 desktop-board-reskin 落地後改用它；本塊只新增並讓兩個清單用）。
- 列：`title=item.id`（等寬）、`copyValue=item.id`、`description`＝Purpose 摘要（`item.purposeSummary`；佔位時以 `text-status-warning` 顯示「Purpose 待補」）、`meta`＝需求數（`FileText`＋n，tooltip）、溯源數（`History`＋n，>0 才顯示）、相對時間；`onClick=onOpen(item.id)`；`data-spec` 不變。`focus` 行為（捲到指定規格並高亮）維持。
- 空態：無規格「specs.empty」、無命中「specs.noResults」置中於卡內。

### D4 已封存頁

- 結構：`<PageHeader title="已封存" description actions={搜尋框} />` → `<Tabs defaultValue="changes">` 包 `<ListCard header={<TabsList variant="card">變更 N／討論 N</TabsList>} footer={目前節的 ListPager}>`，`TabsContent` 各渲染該節的列；`ListCard` 的內容卡以 `rounded-t-none border-t-0` 接在分頁列下（`Tabs variant="card"` 的既定接法）。
- 變更列：`leading=item.date`、`title=item.name`、`copyValue=item.datedName`、`description`＝Why 首句（缺席則單行）、`meta`＝任務徽章（`Badge`，未全完成 warning 樣式）、觸及規格數、建立者頭像圓點、來源討論籤（既有元件）；`data-archived` 不變。討論列：`title=slug`、`copyValue=slug`、`description=topic`、`meta`＝日期、「N 輪」、衍生變更數；改進小章放標題旁 `children`。
- 兩節各自 `page`、`pageSize`；搜尋同時過濾兩節、徽章顯示各自命中數（既有）。

### D5 手冊頁

- `ManualTree`：`w-60`（240px）`bg-sidebar border-r`，列為 `NavItem` 風格（32px、8px 圓角、作用中主色淡底）；「可能過期」琥珀小字與未入冊提示不變；搜尋框改 `SearchField` 全圓。
- 中欄：手冊頁自有 `MANUAL_COLUMN_CLS = "mx-auto w-full max-w-[768px]"`（共用的 `READING_COLUMN_CLS` 仍供規格與封存抽屜使用、不動）；頁首 `h1 text-2xl font-normal`＋灰字一行「產生於 {generated}」（`generated` 缺席時不顯示；文案鍵 `manual.generatedAt`）；內文區維持捲動（`px-6 py-5`），內文包在 `Card`（`rounded-2xl border bg-card p-6`）內；底列 `footer` 改 `shrink-0 border-t border-border bg-card px-6 py-2.5`，內層 `mx-auto w-full max-w-[768px] flex items-center justify-between gap-3`：左「出處」標籤＋capability 籤（`Badge variant="outline"` 等寬字、存在者為 `button` 可點、不存在者純文字），右上一頁／下一頁 `Button variant="outline" size="sm"`（內含 11px 灰字「上一頁」／「下一頁」＋目標頁標題，無目標時該鈕缺席）。
- `ManualToc`：`w-[200px]`，列 12px、作用中主色；顯示條件不變。

### D6 規格字面

- 「清單最新在前與換頁瀏覽」：換頁段改為「卡底固定工具列：左側第 a–b 筆共 N 筆與每頁筆數下拉（20／50／100，預設 20，三個清單各自跨啟動記住）；右側於總頁數大於 1 時為 « ‹ 頁碼 › » 與跳頁輸入，作用中頁碼主色淡底，超過 7 頁以省略號收中段；工具列於清單非空時常駐、不隨列捲動」；scenario「超過 20 筆顯示換頁控制列」改含頁碼與跳頁、「20 筆以內換頁控制列缺席」改為「只顯示筆數範圍與每頁筆數、無頁碼」；新增「改每頁筆數鉗制頁碼」「第一頁與最後一頁鈕」「每頁筆數跨啟動記住」scenario。
- 「規格頁提供清單、搜尋與展開檢視」：以列式容器與頁標題區搜尋改寫；點列開啟該規格的唯讀檢視（現為規格抽屜）；展開／縮合段落移除；空態與唯讀條款保留。
- 「已封存頁含討論節」：子頁籤改卡片標頭式（分頁列為列表卡頂部、計數徽章跟隨）、搜尋框在頁標題區右端；其餘保留。
- 「規格與封存卡片收合資訊」：「卡」改「列」、資料規則逐字保留；hover 顯示複製鈕、› 收尾。
- desktop-manual-page「內頁渲染與出處跳規格」：加三欄寬度（240／768 置中／200）、閱讀卡、固定底列（白底、上緣細線、對齊 768px）與上一頁／下一頁框線鈕帶目標頁標題；其餘保留。

### D7 每頁筆數記憶（實機試用後追加）

- 記憶歸桌面 store（`apps/desktop/src/store.ts`），與看板「已轉出」展開記憶（`speclink.board.promotedExpanded`）同一套做法：共用元件只收受控 props，不碰 localStorage。
- store 欄位 `pageSizes: { specs: number; archivedChanges: number; archivedDiscussions: number }`，動作 `setPageSize(list, size)`；`list` 為上述三鍵之一。
- localStorage 單鍵 `speclink.list.pageSizes`，值為 JSON 物件（如 `{"specs":20,"archivedChanges":50,"archivedDiscussions":20}`），只存在 app 本機、不寫入任何專案目錄。
- 讀取（store 建立時）：鍵缺席、JSON 壞掉、localStorage 不可用、或某個欄位不在 `PAGE_SIZE_OPTIONS` 之內——該欄位一律退回 `PAGE_SIZE`（20），不拋錯；合法的欄位照常採用。不接受 0、負數或任意數字：0 會讓總頁數變成無限大。
- 寫入（`setPageSize`）：先更新 state，再把整個物件寫回該鍵；寫入失敗（私密模式、配額）只失去跨啟動記憶，本次執行期照常生效。
- 受控 props：`SpecList` 加 `pageSize?: number`、`onPageSizeChange?: (size: number) => void`；`ArchivedList` 加 `changesPageSize?`、`onChangesPageSizeChange?`、`discussionsPageSize?`、`onDiscussionsPageSizeChange?`。值與回呼都給才算受控，否則退回元件內 state（server-web 與單元測試走這條）。頁碼仍由元件持有：改每頁筆數時元件照舊鉗制頁碼並捲回頂部。
- `App.tsx` 把三組值與 `setPageSize` 接上。dev 版（`http://localhost:1420`）與安裝版的 WebView origin 不同，兩邊的記憶各自獨立。

## Implementation Contract

**Behavior**

- 規格頁：頁標題「規格」＋說明，右端全圓搜尋；白底細框列表卡填滿主區，一列一份規格（等寬名稱、hover 複製鈕、Purpose 摘要或「Purpose 待補」、需求數、溯源數、相對時間、›），列在卡內捲動；卡底工具列顯示筆數範圍與每頁筆數，超過一頁時有 « ‹ 頁碼 › » 與跳頁；點列開規格檢視。
- 已封存頁：頁標題「已封存」＋說明＋搜尋；列表卡頂部為卡片標頭式分頁「變更 N」「討論 N」；列上資料與現狀相同；兩節各自換頁。
- 手冊頁：240px 淡灰目錄樹、768px 置中閱讀卡（頁首標題與產生時間、內文捲動）、200px 本頁目錄；底列固定、白底上緣細線、對齊 768px，左出處籤可點開規格檢視、右上一頁／下一頁框線鈕帶目標頁標題。
- 每頁筆數：規格頁、已封存「變更」節、「討論」節各自記住，重開 app 後沿用；首次使用為 20。
- 排序、搜尋、每頁預設 20、頁碼獨立、換頁回頂、越界鉗制、閱讀序、過期標示、出處跳規格全部與現狀一致。

**Interface / data shape**

- `@speclink/ui` 新匯出：`ListRow`、`ListCard`、`SearchField`、`pageWindow`、`PAGE_SIZE`、`PAGE_SIZE_OPTIONS`；`ListPagerProps` 新形狀（D2）；`SpecListProps`／`ArchivedListProps` 新增 `title`、`description` 與 D7 的受控每頁筆數 props。
- i18n 新鍵（兩語系）：`pager.range`、`pager.perPage`、`pager.jump`（跳頁輸入的 aria-label）、`pager.jumpTo`（跳頁輸入前後的可見文字「跳到 {n} 頁」，以 `{n}` 切成兩段包住輸入框）、`pager.pageN`、`pager.first`、`pager.last`、`specs.pageDesc`、`archived.pageDesc`、`manual.generatedAt`、`manual.prevLabel`、`manual.nextLabel`；移除 `pager.page`。
- 清單資料欄位：不變。
- 桌面 store：`pageSizes`、`setPageSize`；localStorage 鍵 `speclink.list.pageSizes`（D7）。

**Failure modes**

- 跳頁輸入非數字或越界：忽略或鉗制，不報錯。
- 每頁筆數改變使頁碼越界：鉗制至末頁。
- 手冊 `generated` 缺席：產生時間行缺席。
- 記住的每頁筆數缺席、損毀、不在 20／50／100 之內，或 localStorage 不可用：該清單退回 20，不報錯；寫入失敗只失去跨啟動記憶。

**Acceptance criteria**

- `npm test -w packages/ui`、`npm test -w apps/desktop` 全綠。
- `listRow.test.tsx`：標題／描述／meta／›、無描述單行、Enter 觸發、複製不冒泡；`listPager.test.tsx`：範圍文案、每頁下拉呼叫 `onPageSize`、`pageWindow` 七種輸入（含省略號）、作用中頁碼 class、跳頁 Enter 與鉗制、單頁時無頁碼、« » 於首末頁停用且點擊呼叫 `onPage(1)`／`onPage(pageCount)`；`specList.test.tsx`／`archivedList.test.tsx`：改為列斷言後既有排序、搜尋、換頁、徽章、焦點案例通過，加「頁標題與搜尋框」「分頁列為 card variant」；`manualPage.test.tsx`：底列固定、出處籤、上一頁／下一頁帶標題、產生時間行；`App.test.tsx` 三頁 title 傳入；`store.test.ts`：`pageSizes` 讀取（缺席、壞 JSON、非法值、部分合法）與 `setPageSize` 寫回；`specList.test.tsx`／`archivedList.test.tsx`：受控每頁筆數時以 prop 為準並呼叫回呼；`App.test.tsx`：在規格頁改每頁 50 後重建 App，規格頁仍為每頁 50、已封存兩節仍為 20。
- 手動：三頁淺色深色各看一次；86 份規格翻頁、跳頁、« » 跳首末頁、改每頁 50 後重開 app 仍為 50；已封存兩節切換；手冊讀三頁。

**Scope boundaries**

- In：三個頁面元件與 `ListPager`、三個新元件、i18n、`App.tsx` 傳 title 與每頁筆數、桌面 store 的每頁筆數記憶、對應測試、五條 spec 字面。
- Out：看板、詳情頁化、設定、系統匣、server-web（含其每頁筆數記憶）、排序與搜尋規則、頁碼與搜尋字串的記憶。

## Risks / Trade-offs

- [`ListPager` props 形狀改變] → 消費者只有 `SpecList`、`ArchivedList`（同批改）；`DiscussionDrawer` 匯入的是 `PAGE_SIZE`，常數保留。
- [`pager.page` 鍵移除] → 兩語系同批移除，`i18n.test` 鍵集合相等。
- [手冊閱讀欄從全寬改 768px 置中] → 討論裁定；寬表格仍在卡內橫捲（既有 markdown 規則）。
- [回歸對照] → 不動 CLI、golden、`--json`；ui 與 desktop 測試為基準，既有案例只改結構斷言不改語意。
- [每頁筆數記憶的壞值] → 只接受 `PAGE_SIZE_OPTIONS` 內的值，其他一律退回 20；store 測試釘住缺席、壞 JSON、非法值三種輸入。
- [跨平台] → 純 CSS／TS；localStorage 由各平台 WebView（WKWebView、WebView2、WebKitGTK）提供，不可用時退回預設值、不報錯。

## Migration Plan

無資料遷移：`speclink.list.pageSizes` 是新鍵，舊版 app 不讀它；回滾即還原本刀 commit，留在本機的鍵無害。

## Open Questions

無。
