## Context

- cut 1（已封存 `desktop-detail-doc-tree`）留下的插入點：`packages/ui/src/components/DetailShell.tsx` 的 `DocHeader`（`badges` 籤槽）、`DocTiles`（空陣列即不渲染）、`DocBody`（`data-doc-body`，本頁目錄掃 `[data-doc-heading], .markdown h2, .markdown h3`）與 `ui/callout.tsx` 的 `Callout` 原語；`ChangeDetailPage.tsx` 與 `ArchivedDetailPage.tsx` 目前都傳 `tiles={[]}`、籤槽空、提示框槽不放元素。
- 六份文件的現行渲染器：`SectionedDoc.tsx`（h2 白名單→20px 標籤＋Markdown，白名單外併入 prose，無白名單整篇退回；另匯出 `LABEL_CLS`／`SUB_LABEL_CLS` 給 `TaskList`、`DeltaBadges` 與 `DiscussionDetailPage`）、`TaskList.tsx`（平面清單，群組標題 h4，整列 label＋把手＋工具列＋n 快捷鍵，`parseTaskDoc` 給 group／task 兩種節點）、`DeltaBadges.tsx` 的 `DeltaSpecView`（`splitDeltaSections` 切色標區段，區段內文交 Markdown）、`ChangeDetailPage.tsx` 內的 `PlanTab`／`PlanCard`／`PlanName`／`OpTag`（四卡；判定收斂於 `stage.ts` 的 planWave 等入口）、`TicketView.tsx`（`data-ticket-header` 段標題列＋輕量輪列、`useStationTickets`、`TicketTabBody` 三態）。
- 可直接沿用的解析：`specSections.ts` 的 `parseSpecSections`（`### Requirement:` 切段、Scenario、@trace，認 fenced code block）與 `rehypeSpecKeywords`；`delta.ts` 的 `splitDeltaSections`／`specDeltaCounts`；`tasks.ts` 的 `parseTaskDoc`（已剝 `[M]` 與 stable ID）。規格詳情頁 `SpecDetailPage.tsx` 的 `RequirementSection` 是需求段的現成畫法（Requirement 小標、標題、需求文 `specKeywords`、左線 Scenario）。
- Markdown 基準字級在 `apps/desktop/src/index.css` 的 `.markdown { font-size: 1rem; line-height: 1.7 }`；`Markdown.tsx` 的 `READING_COLUMN_CLS`（96ch）套在 `DocBody` 上。
- 宿主：`apps/desktop/src/App.tsx` 掛 `ChangeDetailPage` 與 `ArchivedDetailPage`，store 有 `openSpec(capability)`（開規格詳情頁）、`localDirOf(activeKey)`、`fsActionsAdapter.reveal(path)` 與 `runOnDocument` 的失敗 toast 包裝；`DocumentActionsMenu` 的「在 Finder 顯示」文案鍵為 `fs.revealIn.<platform>`。
- 守門：`theme.test.ts` 禁原生色階字面；`changeDetailPage.test.tsx`、`archivedDetailPage.test.tsx`、`taskList.test.tsx`、`planTab.test.tsx`、`ticketView.test.tsx`、`components.test.tsx` 以 `data-section`、`data-delta-section`、`data-ticket-header`、`data-plan-card-header`、`getByRole("tab")` 等錨點釘住現況；`i18n.test.tsx` 要求 tw／en 鍵集合相等。
- 落點：純前端——`packages/ui`（渲染器、解析器、頁）與 `apps/desktop`（兩條回呼）。不碰 Rust crate、Protocol、CLI 與 server。互動原型見 proposal。

## Goals / Non-Goals

**Goals:**

- 六份變更文件各自的畫法、數字磚、籤槽與提示框全由現有結構推出；Markdown、schema、工單 JSON 一字不動。
- 任何二級標題成段並進本頁目錄；白名單命中才翻譯與專屬畫法；逐項降級，不整篇退回（除非沒有二級標題）。
- 解析與渲染分離：解析器是純函式、可單測；渲染器只消費解析結果。
- 既有互動（任務勾選、拖排、工具列、n 快捷鍵；排程前置增減；工單載入三態）行為不變，測試錨點能保就保。
- 已封存變更詳情頁零成本跟上（共用同一批渲染器）。

**Non-Goals:**

- 討論側四份文件（cut 3）；文件樹、狀態列、標題列動作、出處行（cut 1 定案不動）。
- 規格詳情頁與手冊頁的外觀；server-web。
- 英文 tasks 的驗證切分詞；BEFORE 註解的呈現（照常不直出）。

## Decisions

### D1 章節模型與文件解析器 docModel.ts

- 新增 `packages/ui/src/docModel.ts`（純函式、無 React）：
  - `splitH2Sections(text): { preamble; sections: H2Section[] }`——每個 `## ` 行開新段（`(optional)` 剝除後查白名單得 `key`，白名單外 `key` 為 null、`title` 為原文）；第一個 h2 之前的內容為 `preamble`；`### New Capabilities`／`### Modified Capabilities` 只在 Capabilities 段內切成子段。沒有任何 h2 時回 null（呼叫端整篇 Markdown）。fenced code block 內的 `## ` 不是標題（沿 `specSections.ts` 的 `fenceFlags` 寫法，抽成共用 `fenceFlags` 匯出）。
  - `parseListItems(md): ListItem[]`——頂層列點（`- `／`* `）與編號項（`1. `），巢狀行歸入該項 `body`；非列點段落歸 `prose`（渲染器在列前後照 Markdown 排）。
  - `splitAlternative(text)`：首個「——」切 `option`／`reason`，切不到回 `{ option: text, reason: null }`。
  - `parseImpact(md)`：找「Affected specs:」列（以「、」「,」切能力名，去反引號）與「Affected code:」列之下的 `New:`／`Modified:`／`Removed:` 子項（路徑以「、」「,」切、去反引號與空白；`(none)`／「無」為空陣列）；形狀不符回 null。
  - `parseCapabilities(md)`：子段內每個「`name`: desc」列點（`(none)` 為空）；不符形狀的列點回 `{ name: null, text }` 當一般列。
  - `parseGoalsNonGoals(md)`：以 `**Goals:**`／`**Non-Goals:**` 粗體標籤切兩欄；缺任一標籤回 null。
  - `parseDecisions(md)`：每個 `### ` 開一張卡，`id` 取標題開頭的 `D\d+`（無則 null），`bullets` 為頂層列點數，`body` 為卡內文。
  - `parseContractRows(md)`：「- **鍵**：值」或「- **鍵**: 值」列點成 `{ key, value }`；任一列不符回 null（整段 prose）。
  - `splitRisk(text)`：首個「→」切 `risk`／`mitigation`。
  - `splitTaskText(text)`：先剝開頭「X.Y」序號成 `number`（無則 null；`TaskList` 與手動提示框共用這一條規則），再以首個「驗證：」切 `verify`，再以首個全形「：」切 `title`／`body`；標題緊接的 `（design …）`／`(design …)` 括號剝除，其中 `D\d+` 收進 `decisions`、其餘文字（去分號）為 `note` 前置到 body。
  - `removedScenarios(block)`：取 `<!-- REMOVED-SCENARIO: 名稱 -->` 名稱清單並回去掉這些行與 `<!-- BEFORE: … -->` 行的內文。
  - `splitScenarioSteps(body)`：`- **WHEN**`／`- **THEN**`／`- **AND**`／`- **GIVEN**` 列點成 `{ label, text }`，其餘行為 `rest`；沒有任何步驟列點回 null。
  - `findingAxis(text)`：`^(Correctness|Standards|Coherence)\s*[:：]\s*` 取軸向與去前綴描述。
  - `proposalShape(sections)`：Bug Fix（Problem＋Root Cause）／Refactor（Summary＋Motivation）／Feature（Why＋What Changes）／null。
- 測試 `packages/ui/src/__tests__/docModel.test.ts` 逐函式含降級案例（spec「提案與設計章節以中文標籤呈現」「任務群組標題與章節標籤同款式」的 Example 表直接當案例）。

### D2 共用零件 DocParts.tsx 與標籤類別搬家

- 新增 `packages/ui/src/components/DocParts.tsx`：
  - `SECTION_HEADING_CLS = "text-base font-semibold"`（16px 粗體，spec「章節標題為 16px 粗體」）；`LABEL_CLS`／`SUB_LABEL_CLS` 自 `SectionedDoc.tsx` 搬到這裡（`DiscussionDetailPage` 改匯入路徑、款式不變，cut 3 再退場）。
  - `DocSection({ title, count?, icon?, children })`：`<section data-section={key 或 "custom"}>` 包 `h2`（`data-doc-heading`、`SECTION_HEADING_CLS`，右端 `count` 籤 `.cnt` 款式＝`Badge variant="secondary"`）與內文；段間細線（`border-b border-border/70 py-5`，末段無線）。
  - `RowList({ items, icon? })`：細線列（15px、行高 1.7、`divide-y`），每列內文以 `Markdown` 行內渲染（`ROW_MD_CLS`＝`[&_p]:my-0`，匯出給提案能力列共用；空字串不渲染，免出「無內容」占位）；`icon` 給非目標的 `Ban`。
  - `ListSection({ section, title, renderList })`：列式章節（`parseListItems`→頂層列點數籤→`DocSection`＋`ListBlocks`），提案與設計的列式章節共用。
  - `NumberedSteps({ items })`：序號圓標＋內文。
  - `OptionReasonRows({ items })`：方案一行（`font-medium`）、落敗理由灰字一行；`reason` 為 null 時只畫方案列。
  - `KeyValueRows({ rows })`：左鍵（`text-muted-foreground w-28 shrink-0`）右值（Markdown；空值不渲染）。
  - `PathChip({ path, note?, reveal? })`：等寬籤，hover 出 `CopyButton`（既有）與 `reveal` 存在時的 `FolderOpen` 圖示鈕。`reveal` 為 `PathReveal { label; reveal(path) }`——`packages/ui` 不知平台，鈕的文案由宿主連同動作一起給（桌面版取 `fs.revealIn.<platform>`，與「開啟 ▾」選單同一組詞條：macOS「在 Finder 顯示」），ui 不另設泛用詞條。
  - `InlineMarks({ text })`：單行標題的行內標記——`` `code` `` 成等寬籤、`**粗體**` 成粗體，其餘原樣；任務標題、手動提示框、決策卡與風險提示框的標題共用。
  - `NameChip({ name, onClick? })`：能力名／變更名籤（`NAME_CHIP_CLS`＝`font-mono text-xs rounded bg-muted px-1.5`；排程文件的名稱籤同款），`onClick` 有則為 `button`。
  - `MoreToggle({ hidden, label, open, onToggle })`：「展開其餘 N 段」「還有 N 個」鈕（`variant="ghost" size="sm"`，`aria-expanded`）。
  - `Lead({ children })`：灰底引言段（`rounded-xl bg-muted px-4 py-3.5 text-base leading-[1.8]`）。
- 全部零件只用 token class；不新增原語檔，`ui/` 不動（`Callout`、`Badge`、`Button`、`Card` 沿用）。

### D3 提案渲染器 ProposalDoc

- 新增 `packages/ui/src/components/ProposalDoc.tsx`：props `content`、`empty`、`deltaCaps: string[]`（已載入 delta 規格的能力名）、`onOpenCapability?(cap)`、`pathReveal?: PathReveal`（影響段「移除」組的路徑籤不給顯示鈕——那些檔案在 apply 後已不存在）。
- `splitH2Sections` 回 null → `<Markdown>` 整篇退回；否則 `preamble` 先照 Markdown，再逐段 `DocSection`：
  - Summary → `Lead`（段內文整段）。
  - Why／Motivation／Problem／Root Cause／Success Criteria → `parseListItems`：列點 `RowList`（count＝列點數），`prose` 段照 Markdown。
  - What Changes／Proposed Solution → `NumberedSteps`（列點與編號項合併依出現序）。
  - Non-Goals → `RowList icon=Ban`。
  - Alternatives Considered → `OptionReasonRows`（`splitAlternative`）。
  - Capabilities → 子段標籤 `h3`（`data-doc-heading`、`SUB_LABEL_CLS` 改 14px semibold）＋每列 `NameChip`（`onOpenCapability` 且 `deltaCaps` 含該名時可點）＋說明；空為「無」；之後接提示框（D9 算好傳進來的 `capabilityNotes`，由頁放在 Capabilities 段尾——渲染器接 `afterCapabilities?: ReactNode`）。
  - Impact → `parseImpact` 非 null 時「規格」列（`NameChip`）＋「新增 N」「修改 N」「移除 N」三組 `PathChip`；null 時 `RowList`。
  - 白名單外 → `DocSection title=原文`＋Markdown。
- 本頁目錄錨點＝各段 `h2`（含白名單外）與 Capabilities 子段 `h3`。

### D4 設計渲染器 DesignDoc 與 D 籤跳轉

- 新增 `packages/ui/src/components/DesignDoc.tsx`：props `content`、`empty`、`jump: { id: string; seq: number } | null`、`onJumped()`。
  - Context → `RowList`；Goals / Non-Goals → `parseGoalsNonGoals` 兩欄（`grid-cols-2 gap-4`，各欄小標「目標 N」「非目標 N」＋`RowList`），null 時 Markdown；Decisions → 每張 `DecisionCard`（`Card` 細框：卡頭 `button`（`aria-expanded`）含 `id` 籤（`NameChip` 款）、`h3 data-doc-heading` 標題（去掉開頭的 D 編號，`InlineMarks`）、右端「N 點」；內文 Markdown）；Implementation Contract → `KeyValueRows`（null 時 Markdown）；Risks / Trade-offs → 每列 `Callout tone="warning"`（`splitRisk`：title＝risk 首行以 `InlineMarks` 呈現、children＝其餘行與 mitigation）；Migration Plan／Open Questions／白名單外 → Markdown。
  - 跳轉：`jump` 的 `id` 命中某卡時該卡 `open` 設 true 並 `scrollIntoView({ block: "start" })`，完成後 `onJumped()`；每次點 D 籤 `seq` 遞增，同一張卡可重跳；沒命中時只 `onJumped()`（頁已切到設計文件、自頂端呈現）。捲動放在卡自己的 effect（Radix `TabsContent` 下一輪才掛內文，與 cut 1 的 `SpecCapSection` 同理）。
  - 預設全部收合；`jump` 與使用者點擊是唯二展開來源。
- 數字磚由 D9 自解析結果算，渲染器不算磚。

### D5 任務群組卡與三層任務列（TaskList）

- `packages/ui/src/components/TaskList.tsx`：
  - 列渲染改「群組卡」：`parseTaskDoc` 的序列依 group 節點切成卡（第一個 group 之前的任務為無卡頭清單）；卡＝`Card` 細框，卡頭 `h3 data-doc-heading`（`SECTION_HEADING_CLS`）含序號籤（標題開頭 `^(\d+)\.\s*` 取出）、標題餘文、右端小進度 `done/total`（等寬 12px）；`SortableGroupHeading` 改渲染卡頭（仍是 disabled sortable、落點語意不變，`resolveDropTarget` 不動）。
  - `TaskRowBody` 改三層：`splitTaskText(item.text)` 得 `title`／`body`／`verify`／`decisions`／`note`；預設列＝勾選框、序號籤（`item.ordinal` 對應 tasks.md 的「X.Y」文字——序號自原文取：描述開頭的 `^\d+(\.\d+)*\s+` 剝到標題外、成等寬籤）、標題（`Markdown` 行內：`[&_p]:inline`）、D 籤（`NameChip`，`onJumpDecision?(id)`，`button` 要 `e.preventDefault()` 免觸發 label 勾選）、手動籤（D14：移到列尾）、列尾 `ChevronRight`／`ChevronDown` 展開鈕（`aria-expanded`、`aria-label` 取 `tasks.expand`／`tasks.collapse`，同樣 `preventDefault`）。展開後在列下方畫 `note`＋`body`（Markdown）與「驗證」小標＋`verify`。
  - 新 prop `onJumpDecision?: (id: string) => void`；`DragOverlay` 的浮起列只畫標題層。
  - 唯讀：勾選框 disabled、展開鈕照常；工具列、n 快捷鍵、`rowLabelProps`、`ignoreClickAfterSelecting` 不動。
  - 文字 `text-base` 改 `text-[15px]`（spec「閱讀卡內文字級為 15px」）。
- 數字磚、籤槽與手動提示框在 D9（頁層自 `parseTaskDoc` 算）。

### D6 規格 delta 的需求段 RequirementBlock 與 DeltaSpecView

- 新增 `packages/ui/src/components/RequirementBlock.tsx`：自 `SpecDetailPage.tsx` 抽出 `RequirementSection` 的畫法成 `RequirementBlock({ req, anchor?, delta? })`，`delta` 為 `{ op, removedScenarios, defaultOpen }`（缺席＝規格詳情頁畫法：不收合、無色條、無移除提示）：
  - 標題列：`Requirement` 等寬小標（`KW_LABEL_CLS` 搬來）、`h2`／`h3`（規格詳情頁維持 `h2 data-spec-heading`；有 `delta` 時用 `h3 data-doc-heading`）、右端 `anchor` 槽（規格詳情頁的溯源錨點）；有 `delta` 時左緣 `border-l-4` 取 `DELTA_COLORS` 對應的邊框色（`SEMANTIC_TONE` 文字色→同色邊框 class，於 `DeltaBadges.tsx` 新增 `DELTA_BORDER`）。
  - `delta.removedScenarios` 非空 → 需求文之前 `Callout tone="caution"` 標題「本變更移除 N 個 scenario」、內文 `<del>` 名稱清單＋說明句。
  - 需求文：`req.body` 以空行切段，>3 段時只畫前三段＋`MoreToggle`「展開其餘 N 段需求文」。
  - Scenario：`splitScenarioSteps` 非 null → 左線縮排內 `grid-cols-[4rem_1fr]` 兩欄（標籤欄等寬小寫灰字 WHEN／THEN／AND／GIVEN，內容欄 Markdown `specKeywords`）＋`rest` Markdown；null → 現行畫法。>3 個時 `MoreToggle`「還有 N 個 scenario」。
  - 有 `delta` 時整段可收合：收合列＝小標、標題、「N 個 scenario」籤、有移除時「移除 N 個 scenario」籤；點列展開。
  - `SpecDetailPage` 改用 `RequirementBlock`（不帶 `delta`：不收合、無色條、無移除提示，外觀與現在相同——`specSections.test.ts` 與規格詳情頁測試不動）。
- `packages/ui/src/components/DeltaBadges.tsx` 的 `DeltaSpecView` 改：每個 op 區段 → 色標標頭（款式改 `SECTION_HEADING_CLS`，`data-delta-section` 與 `data-doc-heading` 不變）→ `parseSpecSections(section.content)` 非 null 時每條需求 `RequirementBlock`（`delta={ op, removedScenarios: removedScenarios(需求原文), defaultOpen: 該區段第一條 }`——每個 ADDED／MODIFIED 區段的第一條展開），null（REMOVED／RENAMED 內文）時 Markdown。`removedScenarios` 要在 `parseSpecSections` 之前對每條需求原文處理：改為 `parseSpecSections` 回的 `req.body` 仍含註解行（Markdown 會丟棄），`removedScenarios(req.body)` 取名單即可。
- 能力頭：`ChangeDetailPage` 的 `SpecCapSection` 與 `ArchivedDetailPage` 的對應段改共用 `DeltaBadges.tsx` 新匯出的 `CapabilityHeader({ cap, counts, onOpenSpec? })`（等寬名、`DeltaBadges`、`onOpenSpec` 有時「看正式規格」`Button variant="ghost" size="sm"`，`h3 data-doc-heading`）。

### D7 排程文件 PlanDoc：數字磚、提示框、卡頭

- `PlanTab`／`PlanCard`／`PlanName`／`OpTag` 自 `ChangeDetailPage.tsx` 搬到新檔 `packages/ui/src/components/PlanDoc.tsx`（匯出 `PlanDoc`，props 不變；`planTab.test.tsx` 仍經 `ChangeDetailPage` 測）。
- `PlanCard` 卡頭改 `h3 data-doc-heading`（`SECTION_HEADING_CLS`，不再全大寫字距）、右端 `badge`（前置卡狀態徽章）或筆數籤（重疊卡、封存順序卡）；`data-plan-section`、`data-plan-card-header`、`data-plan-card-body` 與全部互動不動。
- 數字磚、籤槽、提示框由 D9 的 `planSummary(change, t)` 算：wave 為 null 回全空；否則磚四格、籤＝狀態徽章同一枚、提示框依 spec Example 表的五列組標題（`plan.calloutStart`／`plan.calloutBlocked`＋`plan.calloutArchiveFree`／`plan.calloutArchiveWait`／`plan.calloutConflict` 四段詞條以「，」接）、tone 由兩子句皆綠決定、內文列前置名、archiveAfter 名、衝突對方名與 `plan.calloutRecompute` 一句。

### D8 工單文件：籤槽、數字磚、可蓋章提示框、輪卡與軸向籤

- `packages/ui/src/components/TicketView.tsx`：
  - 刪 `data-ticket-header` 段標題列（資訊搬到籤槽與數字磚）；`RoundBlock` 改卡：`Card` 細框，卡頭 `button`（`aria-expanded`）含 `h3 data-doc-heading`「第 N 輪」、階段詞、「範圍 N 檔」、「N 條」與各嚴重度計數籤（`SEVERITY_CLS` 同色、為零不出）；展開後先列 scope 路徑（等寬，不再另設 scopeOpen 開關）再列 findings。
  - `FindingRow` 加軸向籤：`findingAxis(text)` 有軸向時出 `Badge variant="outline"` 文字為軸向、描述去前綴；`splitAcceptedToken` 在去前綴之後跑（順序：先切 accepted 尾 token、再取軸向前綴——Example 表第三列「Standards: (accepted) 語意不清」結果為軸向 Standards、描述「(accepted) 語意不清」、無已接受籤，兩種順序結果相同；取「先尾後首」）。
  - `data-ticket-round`、`data-ticket-round-toggle`、`data-ticket-finding`、`data-ticket-severity`、`data-ticket-path`、`data-ticket-accepted`、`data-ticket-text`、`data-ticket-scope-path` 錨點不變，新增 `data-ticket-axis`。
- 籤槽（「第 N 輪」＋階段詞）、數字磚、tip 提示框由 D9 的 `ticketSummary(station, ticket, t)` 算；`ticket` 為 undefined／null 回全空。stamp 指令文字：`speclink review stamp`／`speclink verify stamp` 依站。

### D9 數字磚、提示框與籤槽的推導單一入口 changeDocSummary.tsx 與頁的接線

- 新增 `packages/ui/src/components/changeDocSummary.tsx`，匯出 `changeDocSummary(tab, data, t): DocSummary`（`{ badges; tiles: DocTile[]; callout }`，型別與 `DocTile` 同放 `DetailShell.tsx`），`data` 為 `{ proposal, design, tasksMd, specDocs, change, tickets, readOnly, onLocateTask }`（皆為頁已持有的狀態）；內部依 tab 分派：
  - proposal：`proposalShape` → 籤；磚依 spec 順序取存在的前四格；提示框＝Modified Capabilities 中 `specDocs` 有的能力各一行（`specDeltaCounts(doc).modified`＋`removedScenarios` 總數），空則 null。
  - design：磚四格（缺段缺格）；無籤、無提示框。
  - tasks：`parseTaskDoc` → 籤「全部完成」或 `n/m`；磚四格；手動任務有則手動色提示框（`Callout tone="manual"`，D14；標題 `tasks.manualCallout`、內文列序號＋標題）。
  - specs：籤＝各 op 計數籤（`DELTA_LABEL_KEYS`＋`DELTA_COLORS`）；磚四格（`sumDeltaCounts`＋各 doc 的 `removedScenarios` 總數）；無提示框。
  - plan：D7 的 `planSummary`。review／verify：D8 的 `ticketSummary`。
  - 文件未載入（undefined）或無內容（null）時三者皆空。
- `ChangeDetailPage.tsx`：`DocHeader badges={summary.badges}`、`<DocTiles tiles={summary.tiles} />`、提示框槽 `{summary.callout}`（在 `DocTiles` 與 `DocBody` 之間，已是 cut 1 留的位置）；`TabsContent` 內改掛 `ProposalDoc`（`deltaCaps=Object.keys(specDocs ?? {})`、`onOpenCapability`、`pathReveal` 透傳）、`DesignDoc`、`TaskList onJumpDecision`／`locate`、`CapabilityHeader onOpenSpec`（提案列為 New Capabilities 的能力不給——封存前沒有正式規格）、`PlanDoc`、`TicketTabBody`（不變）。跳轉狀態（選定能力與捲動請求、決策卡請求 `{ id, seq }`、任務列定位請求 `{ ordinal, seq }`）收在兩頁共用的 `useDocNav(setTab)`（新檔 `packages/ui/src/components/useDocNav.ts`）。提示框槽 `<div data-doc-callout>` 只是錨點，`Callout` 以 `className="my-0"` 去掉 prose 外距（`Callout` 多收 `className` 與 `data-*` 等 div 屬性）。新 props：`onOpenSpec?: (capability: string) => void`、`pathReveal?: PathReveal`。
- `ArchivedDetailPage.tsx` 變更變體同樣接 `changeDocSummary`（`readOnly`，排程不在）、`ProposalDoc`／`DesignDoc`／`TaskList onJumpDecision`／`CapabilityHeader`；新 props 同上；討論變體不動。
- 本頁目錄重掃鍵 `changeDocTocKey` 不變（內容一變就重掃；展開收合不改錨點集合——收合的需求與決策卡的標題仍在 DOM）。

### D10 字級與行寬：DocBody 15px／1.75 與卡欄上限

- `DetailShell.tsx` 的 `DocBody` className 改 `w-full text-[15px] leading-[1.75] [&_.markdown]:max-w-none [&_.markdown]:text-[15px]! [&_.markdown]:leading-[1.75]!`：`index.css` 的 `.markdown { font-size; line-height }` 不在 cascade layer，一般宣告就壓過 Tailwind `@layer utilities` 的規則（與特異性無關），字級與行高的覆蓋因此帶 important（`!`）；不改 `apps/desktop/src/index.css`；規格詳情頁與手冊頁不經 `DocBody`，維持 16px／1.7。`READING_COLUMN_CLS` 自 `DocBody` 移除（卡欄 768px 就是上限）；`Markdown.tsx` 的 `max-w-[96ch]` 同在 utilities 層，由 `DocBody` 的後代變體 `[&_.markdown]:max-w-none` 以特異性解除，規格詳情頁與手冊頁照舊 96ch。`Markdown` 元件另接 `className?`（以 `cn` 合併）供列內排版用；`READING_COLUMN_CLS` 仍由 `DetailHeader` 的內容卡用。
- 討論詳情頁同經 `DocBody`，內文同步 15px（spec「markdown 內容保留文件結構呈現」涵蓋討論側；cut 3 不再動這條）。

### D11 宿主接線：onOpenSpec 與 pathReveal

- `apps/desktop/src/App.tsx`：`ChangeDetailPage` 與 `ArchivedDetailPage` 加 `onOpenSpec={s.openSpec}`；`pathReveal` 只在 `hasLocalDir` 時接：`label` 取 `fs.revealIn.<platform>` 詞條（與「開啟 ▾」選單同一組）、`reveal` 為 `(path) => void s.revealProjectPath(path)`（remote 或無本機目錄時 undefined → 路徑籤只有複製鈕）。
- `apps/desktop/src/store.ts` 新增 `revealProjectPath(relative)`：`localDirOf(get().activeKey)` 為 null 直接回；否則 `runOnDocument(relative, () => fsActionsAdapter.reveal(dir + "/" + relative))`（相對路徑為正斜線、Windows 的 Rust `reveal_in_folder` 接受混合分隔符；失敗走既有失敗 toast）。單行委派、不新增 Tauri command。
- `apps/desktop/src/__tests__/App.test.tsx` 加兩題：點提案「能力」段的能力籤切到規格文件；開已封存變更的提案文件時路徑籤有複製鈕、有本機目錄時有顯示鈕。

### D12 i18n 詞條

- `packages/ui/src/i18n.tsx` 新增（tw／en，兩語系鍵集合相等）：`doc.shapeFeature`「Feature」、`doc.shapeBugFix`「Bug Fix」、`doc.shapeRefactor`「Refactor」；磚標籤 `doc.tileSteps`「解法步驟」、`doc.tileNonGoals`「非目標」、`doc.tileCriteria`「成功準則」、`doc.tileCapabilities`「能力」、`doc.tileFiles`「影響的檔案」、`doc.tileAlternatives`「替代方案」、`doc.tileDecisions`「決策」、`doc.tileGoals`「目標／非目標」、`doc.tileRisks`「風險與取捨」、`doc.tileOpenQuestions`「開放問題」；`doc.none`「無」、`doc.specsRow`「規格」、`doc.codeNew`「新增 {n}」、`doc.codeModified`「修改 {n}」、`doc.codeRemoved`「移除 {n}」、`doc.goals`「目標 {n}」、`doc.nonGoals`「非目標 {n}」、`doc.points`「{n} 點」、`doc.capabilityRewritten`「{cap} 有 {n} 條需求被改寫」、`doc.capabilityRemovedScenarios`「，其中 {m} 個 scenario 以 REMOVED-SCENARIO 宣告」、`doc.moreParagraphs`「展開其餘 {n} 段需求文」、`doc.moreScenarios`「還有 {n} 個 scenario」、`doc.scenarioCount`「{n} 個 scenario」、`doc.removedScenarios`「本變更移除 {n} 個 scenario」、`doc.removedScenariosHint`「封存時正式規格的這些 scenario 會被拿掉。」、`doc.removedScenariosChip`「移除 {n} 個 scenario」、`doc.viewSpec`「看正式規格」、`doc.tileCapsTouched`「觸及的能力」、`doc.tileModifiedReqs`「修改的需求」、`doc.tileRemovedScenarios`「移除的 scenario」、`doc.tileAddedRenamed`「新增／更名」；`tasks.tileTotal`「任務」、`tasks.tileDone`「已完成」、`tasks.tileManual`「手動」、`tasks.tileGroups`「群組」、`tasks.allDone`「全部完成」、`tasks.expand`「展開任務 {n}」、`tasks.collapse`「收合任務 {n}」、`tasks.verifyLabel`「驗證」、`tasks.manualCallout`「{n} 個手動任務」、`tasks.manualCalloutHint`「要由你親自做完再勾；agent 跑 apply 時會停在它前面等你。」；`plan.tileWave`「波次」、`plan.tileDepends`「宣告的前置」、`plan.tileOverlaps`「同名需求重疊」、`plan.tileArchiveAfter`「封存前要等」、`plan.calloutStart`「可以開工」、`plan.calloutBlocked`「等 {n} 項前置」、`plan.calloutArchiveFree`「也可直接封存」、`plan.calloutArchiveWait`「封存前要等 {n} 個變更」、`plan.calloutConflict`「有同名衝突要先改」、`plan.calloutRecompute`「排程會在看板刷新時重算。」；`ticket.tileRounds`「輪」、`ticket.tileLast`「{sev}（末輪）」、`ticket.stampable`「可以蓋章」、`ticket.stampableHint`「末輪只剩 {n} 條 SUGGESTION，不擋章。」、`ticket.stampableNone`「末輪無發現。」、`ticket.stampHow`「蓋章請用 CLI 或 agent 跑 {cmd}。」、`ticket.findingsCount`「{n} 條」。路徑籤顯示鈕的文案不在 ui 詞條（宿主給，見 D2）。
- 「可以開工」「等 N 項」的籤沿用 `plan.canStart`／`plan.blockedCount`；「第 N 輪」沿用 `ticket.round`；階段詞沿用 `ticket.phase.*`；delta 操作詞沿用 `delta.*`。

### D13 退場與匯出

- 刪 `packages/ui/src/components/SectionedDoc.tsx`；`LABEL_CLS`／`SUB_LABEL_CLS` 改由 `DocParts.tsx` 匯出，`TaskList`、`DeltaBadges`、`DiscussionDetailPage` 改匯入路徑。
- `packages/ui/src/index.ts` 新增匯出 `ProposalDoc`、`DesignDoc`、`PlanDoc`、`RequirementBlock`、`CapabilityHeader`、`changeDocSummary` 與 `docModel.ts` 的解析函式（含 `newCapabilities`；型別名 `H2Section`、`DecisionData`，與元件 `DocSection`／`DecisionCard` 分開）；`apps/*` 不得重新定義同名元件（既有守門）。
- `components.test.tsx` 若有 `SectionedDoc` 題改為 `ProposalDoc`；`ticketView.test.tsx` 的 `data-ticket-header` 題改為籤槽與數字磚題（經 `ChangeDetailPage`）。

### D14 手動任務列：整列淡底＋列尾徽章與暖黃手動色

- 使用者在 apply 期以真機看過舊畫法（徽章獨佔描述上方一行）後裁定改版：徽章插在序號前後會破壞縮排、獨佔一行則比例不對稱；比較頁（`https://claude.ai/artifact/FESc3evxWy8WUG2YTVo9dA`）三案中選「整列淡底＋列尾徽章」，手動色要暖色、有提醒感、又不撞既有色。
- 色盤現況：主色青綠、進行中藍、成功綠、警示琥珀、錯誤紅、品質站蓋章紫、改進標示靛、討論洋紅都已占用；暖色只剩黃。新增狀態語意 token `--status-manual`（淺色 `oklch(0.6 0.13 92)`、深色 `oklch(0.84 0.15 96)`，與警示琥珀相差約 30 度）與 `--color-status-manual` 映射；`tone.ts` 的 `SEMANTIC_TONE`／`SEMANTIC_SURFACE` 加 `manual`。
- `TaskList.tsx`：`[M]` 列（`data-task-manual`）未勾時整列 `bg-status-manual/12`；描述欄只剩標題列與展開層（舊的上方徽章行刪除）；徽章移到標題列尾——D 籤之後、展開鈕之前（`aria-label` 與文案沿用 `tasks.manual`，`SEMANTIC_SURFACE.manual`＋`SEMANTIC_TONE.manual`），勾完淡底退掉、徽章保留不劃線。拖曳浮起列同樣帶徽章。
- `ui/callout.tsx`：`Callout` 的 `tone` 多收 `"manual"`（對到 `manual` 語意色）；Markdown 的 GitHub Alert 仍只認四型（`CALLOUT_TONE` 不動）。`changeDocSummary` 的手動任務提示框改 `tone="manual"`。
- `ChangeCard.tsx` 的「待手動」章改取 `SEMANTIC_TONE.manual`（spec「介面狀態語意色分層」：待手動為暖黃系），與任務列、提示框同一個 token。
- 手動提示框的「前往」（使用者於 apply 期追加）：`TaskList` 新增 `locate?: { ordinal; seq } | null` 與 `onLocated?()`——命中時捲到該列並沿用「下一個未完成」的高亮（抽成同一個 `highlightRow`），完成即回報；唯讀時列也掛定位 ref（已封存頁的提示框照常可前往）。`changeDocSummary` 的 `data` 多收 `onLocateTask?(ordinal)`，有時每列附「前往」鈕（`tasks.locate`）；頁持有 `taskLocate` 狀態（seq 遞增）傳給 `TaskList`。提示框標題的行內標記改用 `DocParts` 的 `InlineMarks`（自 `TaskList` 的私有 `inlineCode` 抽出共用，並多認 `**粗體**`；見 D2）。

## Implementation Contract

- **Behavior**：開變更或已封存變更詳情頁——提案文件每個二級標題一段（白名單翻譯＋專屬畫法、其餘原文標題）、籤槽模板形狀、四格以內數字磚、能力段下的改寫提示框；設計文件決策卡可收合、風險成 warning 提示框、四格數字磚；任務文件群組卡＋三層列（預設只有標題，展開鈕開內文與驗證，D 籤跳設計決策卡）、手動任務列整列暖黃淡底＋列尾「手動」徽章、四格數字磚、暖黃手動提示框；看板「待手動」章同為暖黃；規格文件能力頭＋色標標頭＋需求段（色條、REMOVED-SCENARIO 提示框、WHEN／THEN 兩欄、收合）、計數籤、四格數字磚；排程文件四卡（互動不變）＋數字磚＋兩態提示框＋狀態籤；工單文件輪卡（末輪展開）＋軸向籤＋數字磚＋可蓋章提示框。閱讀卡內文 15px／1.75、行寬以 768px 卡欄為上限。
- **Interface / data shape**：`ChangeDetailPage` 與 `ArchivedDetailPage` 新增可選 props `onOpenSpec(capability)`、`pathReveal: { label, reveal(path) }`；`TaskList` 新增 `onJumpDecision(id)`；`Markdown` 新增 `className?`；`RequirementBlock` props 見 D6；`changeDocSummary(tab, data, t)` 回 `{ badges, tiles, callout }`；`docModel.ts` 匯出 D1 全部函式。DOM 錨點：保留 `data-section`、`data-delta-section`、`data-plan-section`、`data-plan-card-header`、`data-plan-card-body`、`data-ticket-round`、`data-ticket-round-toggle`、`data-ticket-finding`、`data-ticket-severity`、`data-ticket-path`、`data-ticket-accepted`、`data-ticket-text`、`data-ticket-scope-path`、`data-doc-heading`、`data-doc-tiles`、`data-doc-tile`、`data-spec-cap`、`data-requirement`、`data-scenario`；新增 `data-doc-callout`（頁的提示框槽）、`data-task-manual`（手動任務列）、`data-decision`（決策卡，值為 D 編號或序）、`data-task-group`、`data-task-expand`、`data-task-decision`、`data-ticket-axis`、`data-capability-header`、`data-path-chip`、`data-capability-chip`、`data-removed-scenarios`；移除 `data-ticket-header`。
- **Failure modes**：解析不符形狀一律逐項降級（prose 或一般列），不拋錯、不整篇退回；`pathReveal.reveal` 失敗走宿主既有失敗 toast；D 籤或能力籤找不到目標只切換文件；沒有新的錯誤路徑。
- **Acceptance criteria**：`npm test -w packages/ui -- docModel proposalDoc designDoc deltaSpecView taskList planTab ticketView changeDetailPage archivedDetailPage components specSections i18n theme` 全綠；`npm test -w apps/desktop -- App` 全綠；`npm run build -w apps/desktop` 成功；`speclink validate desktop-change-doc-views` 通過；macOS 手動依 spec 七條需求的新增 scenario 對照。
- **Scope boundaries**：in——`packages/ui` 的 `docModel.ts`、`DocParts.tsx`、`ProposalDoc.tsx`、`DesignDoc.tsx`、`RequirementBlock.tsx`、`PlanDoc.tsx`、`changeDocSummary.tsx`、`useDocNav.ts`、`DeltaBadges.tsx`、`TaskList.tsx`、`TicketView.tsx`、`SpecDetailPage.tsx`（抽需求段，外觀不變）、`ChangeDetailPage.tsx`、`ArchivedDetailPage.tsx`、`DetailShell.tsx`（DocBody 字級與寬）、`Markdown.tsx`（className）、`specSections.ts`（`fenceFlags` 共用匯出）、`DiscussionDetailPage.tsx`（匯入路徑）、`i18n.tsx`、`index.ts`、`theme.css`（`--status-manual`）、`tone.ts`、`ui/callout.tsx`（`manual` 型、`className` 與 div 屬性）、`ChangeCard.tsx`（待手動章配色）、對應測試；`apps/desktop` 的 `App.tsx`、`store.ts`、`App.test.tsx`、`store.test.ts`。out——討論側渲染器、文件樹、狀態列、標題列動作、出處行、規格詳情頁與手冊頁外觀、Rust、Protocol、server-web、`index.css`。

## Risks / Trade-offs

- [任務列三層靠「：」與「驗證：」切，舊 tasks.md 寫法不一致時標題可能過長或無驗證層] → 切不到就整段當標題、無驗證層，與現在的平面列等價；不猜第二種切法（討論裁定）。
- [整列是勾選框的 label，列內新加的 D 籤與展開鈕會觸發勾選] → 兩者皆 `button` 且 `onClick` 先 `preventDefault()`＋`stopPropagation()`（與把手同手法），`taskList.test.tsx` 加「點展開鈕與 D 籤不切換勾選」題。
- [`parseSpecSections` 認 `## ` 為段落終點，delta 的 op 區段先經 `splitDeltaSections` 切過才餵它] → 每個 op 區段內沒有 `## `，需求段完整；REMOVED／RENAMED 區段無 `### Requirement:` 回 null 走 Markdown。
- [收合的需求與決策卡若把標題移出 DOM，本頁目錄會少錨點] → 收合只隱藏內文、標題常駐；測試斷言收合態 `[data-doc-heading]` 數量不變。
- [DocBody 改 15px 會連動討論側] → spec 已把討論側納入同一條；cut 3 不再動字級，兩刀 delta 不重疊。
- [`Markdown` 的 `max-w-[96ch]` 在 15px 下窄於 768px 卡欄，磚與內文不齊] → `DocBody` 以後代變體 `[&_.markdown]:max-w-none` 一處解除，卡欄即上限（spec「markdown 文件內容行寬有上限」改寫）；討論側同經 `DocBody`，不必逐個 `Markdown` 傳 class。
- [`SpecDetailPage` 抽需求段是重構，有回歸面] → `RequirementBlock` 預設值等於現狀、規格詳情頁測試不改，`npm test -w packages/ui -- specDetailPage` 綠為準。
- [`index.css` 的 `.markdown` 基準字級不在 cascade layer，會壓過 Tailwind utilities 層的任意變體（特異性再高也一樣）] → 字級與行高的變體帶 important（`[&_.markdown]:text-[15px]!`），建置產物已確認為 `font-size:15px!important`；jsdom 不算樣式，單元測試只斷言 class token，真機看 [M] 任務。
- [跨平台] → 純 React／CSS；`revealProjectPath` 只做字串拼接與既有 command 委派，Windows 路徑混合分隔符由 Rust `std::path` 吸收；golden 與 CLI 測試無變動面。

## Migration Plan

無資料遷移。`SectionedDoc.tsx` 退場與三個匯入端改路徑在同一變更內完成；`speclink update` 不受影響（無技能檔變動）。

## Open Questions

無。
