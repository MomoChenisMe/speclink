## Summary

把桌面 app（apps/desktop）的規格頁、已封存頁與手冊頁換成新版面：規格頁與已封存頁改為「頁標題區＋列式容器卡」——一列一筆（等寬標題＋複製鈕、一行描述、右端計數籤與日期、›），列表卡以 flex 撐滿主區並在卡內捲動，卡底固定一條換頁工具列（左：第 a–b 筆共 N 筆＋每頁筆數下拉；右：‹ 頁碼 1 2 3 … M › 作用中主色淡底＋「跳到 __ 頁」），搜尋為頁標題區右端的全圓 280px 搜尋框；已封存頁以卡片標頭式分頁分「變更」「討論」兩節、各自換頁；手冊頁主區三欄（240px 目錄樹、768px 置中閱讀卡、200px 本頁目錄），閱讀欄底部一條固定列（白底、上緣細線、對齊 768px）放「出處」capability 籤與「上一頁／下一頁」框線鈕。這是討論 desktop-ui-redesign 的 cut 3a 看板刀的第二塊（三個清單型頁面）。清單排序、搜尋規則、列上的資料、換頁語意與手冊的閱讀序、出處跳規格全部不變。

## Motivation

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境是查規格、翻封存、讀手冊。三頁現況：規格與封存是一張張小卡片堆在主區、卡片有陰影與各自的框，與 cut 1 之後「容器＝白底細框卡、項目＝卡內細線列」的規則不符；換頁只有「上一頁／第 N／M 頁／下一頁」，86 份規格要翻 5 次才到底，沒有頁碼也沒有每頁筆數；已封存頁的子頁籤還是舊式；手冊頁的目錄樹、閱讀欄與錨點列寬度未對齊新殼，頁尾出處與上下頁沒有與閱讀欄對齊的固定列。cut 2 立好的頁標題區與 cut 1 的卡片標頭式分頁原語讓三頁可以直接套同一套版面規則。

## Proposed Solution

1. **列與列表卡**：packages/ui 新增領域元件 `ListRow`（整列可點：左側等寬標題＋hover 才現的複製鈕＋一行截斷描述，右端 meta 槽＋›；細線分隔、hover 微灰）與 `ListCard`（白底細框 16px 圓角容器，內容區縱向捲動、底部固定工具列槽）；規格頁與已封存頁的卡片清單改用它們，列上的資料（需求數、溯源數、相對時間、任務徽章、觸及規格數、建立者、來源討論、輪數、衍生數）一筆不少。
2. **換頁工具列**：`ListPager` 改為卡底固定工具列——左「第 a–b 筆，共 N 筆」＋每頁筆數下拉（20／50／100，預設 20，不持久化）；右「‹ 1 2 3 … M ›」（作用中主色淡底、超過 7 頁時以 … 省略中段）＋「跳到 __ 頁」數字輸入（Enter 跳頁、越界鉗制）；清單有筆數即顯示工具列，總頁數 1 時只顯示左側；規格頁與已封存頁兩節共用同一元件。
3. **頁標題區與搜尋**：規格頁「規格」＋說明、已封存頁「已封存」＋說明，右端為全圓 280px 搜尋框（與看板同款），過濾規則不變；搜尋同時過濾已封存兩節、徽章反映各自命中數（既有語意）。
4. **已封存分頁**：子頁籤改 cut 1 的卡片標頭式（`Tabs variant="card"`），分頁列即列表卡的頂部、計數徽章跟在標籤後；兩節各自的頁碼與工具列獨立（既有語意）。
5. **手冊頁三欄**：目錄樹 240px 淡灰底、節點右端「可能過期」琥珀小字（既有）；閱讀欄 768px 置中、頁首 24px 一般字重標題＋灰字「產生於 {generated}」、內文在白底閱讀卡內捲動；本頁目錄 200px；閱讀欄底部固定列（白底、上緣細線、內容對齊 768px）左放「出處」＋capability 籤（存在者可點開規格檢視，不存在者純文字）、右放「上一頁／下一頁」框線鈕（小字標籤＋目標頁標題），閱讀序與換頁語意不變。
6. **規格字面**：desktop-app「清單最新在前與換頁瀏覽」（工具列內容、每頁筆數、頁碼與跳頁）、「規格頁提供清單、搜尋與展開檢視」（列式、頁標題區搜尋、點列開檢視）、「已封存頁含討論節」（卡片標頭式分頁、頁級搜尋）、「規格與封存卡片收合資訊」（卡片改列，資料規則不變）；desktop-manual-page「內頁渲染與出處跳規格」（三欄寬度、閱讀卡、固定底列）。

不新增或變更任何 CLI 指令、技能或 Agent 指令。設定欄位：無。持久化：無新鍵。相容性：`--json` 與 CLI 輸出不變；清單資料欄位不變。

## Non-Goals

- 不改看板欄與卡片（desktop-board-reskin）。
- 不把規格、已封存、手冊的檢視改成頁（cut 3b）：點列仍開既有抽屜。
- 不改排序規則、搜尋規則、每頁預設 20、手冊閱讀序與過期判定。
- 不做每頁筆數的持久化、不做欄位排序切換。

## Alternatives Considered

- 維持卡片堆疊只換圓角——與「容器＝卡、項目＝列」的規則相反，三頁和設定頁、系統匣面板長得不一樣（討論裁定列式）。
- 無限捲動取代換頁——既有換頁語意與測試綁定，且使用者要的是直接跳頁碼（討論裁定 pager 加頁碼）。
- 用 shadcn Pagination——既有 `ListPager` 是受控自建元件，擴充比換掉省。

## Impact

- Affected specs: `desktop-app`（MODIFIED「清單最新在前與換頁瀏覽」「規格頁提供清單、搜尋與展開檢視」「已封存頁含討論節」「規格與封存卡片收合資訊」）、`desktop-manual-page`（MODIFIED「內頁渲染與出處跳規格」）
- Affected code:
  - Modified: `packages/ui/src/components/SpecList.tsx`、`packages/ui/src/components/ArchivedList.tsx`、`packages/ui/src/components/ListPager.tsx`、`packages/ui/src/components/ManualPage.tsx`、`packages/ui/src/components/ManualTree.tsx`、`packages/ui/src/components/ManualToc.tsx`、`packages/ui/src/i18n.tsx`、`packages/ui/src/index.ts`、`packages/ui/src/__tests__/specList.test.tsx`、`packages/ui/src/__tests__/archivedList.test.tsx`、`packages/ui/src/__tests__/listPager.test.tsx`、`packages/ui/src/__tests__/manualPage.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/__tests__/App.test.tsx`
  - New: `packages/ui/src/components/ListRow.tsx`、`packages/ui/src/components/ListCard.tsx`、`packages/ui/src/__tests__/listRow.test.tsx`
  - Removed: 無
