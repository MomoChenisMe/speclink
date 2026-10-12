## Context

- cut 1（`desktop-detail-doc-tree`，已封存）把變更、討論、已封存三種詳情頁換成三欄骨架，留下閱讀卡零件：`DocHeader`（標題、籤槽、「開啟 ▾」、出處行）、`DocTiles`（0–4 格）、`DocBody`（15px／1.75，本頁目錄自 `[data-doc-heading]` 與 Markdown h2／h3 推導）、`DocSummary { badges, tiles, callout }`。cut 2（`desktop-change-doc-views`，已封存）給變更側六份文件各自的渲染器，並留下共用零件 `packages/ui/src/components/DocParts.tsx`（`SECTION_HEADING_CLS` 16px 粗體、`DocSection`、`RowList`、`OptionReasonRows`、`KeyValueRows`、`NameChip`、`MoreToggle`、`Lead`、`InlineMarks`）、純函式解析 `packages/ui/src/docModel.ts`（`parseListItems`、`listItemsOf`、`itemMarkdown`、`splitAlternative`、`fenceFlags` 自 specSections 共用）、提示框原語 `Callout`（note／tip／warning／caution／manual）與推導單一入口 `changeDocSummary.tsx`。
- 討論側現況：`packages/ui/src/components/DiscussionDetailPage.tsx` 內含解析（`splitDiscussionSections`、`splitRounds`、`splitLabeledFields`、`countPriorDiscussions`）與兩個舊檢視（`RoundsView`、`ConclusionView`，欄位標籤用 `LABEL_CLS` 20px 粗體），背景以 `Markdown` 整篇渲染，衍生變更為 `PromotedChangeRow` 平面列；`DocTiles tiles={[]}`、沒有提示框、沒有出處行。已封存討論頁（`ArchivedDetailPage.tsx` 的 `ArchivedDiscussionView`）共用同一組檢視，衍生列資料是宿主派生的 `SourceLinkItem { slug, topic, disabled }` 三態。
- 資料：`DiscussionItem { slug, topic, status, rounds, created, createdBy?, kind?, promotedTo, concluded?, hold? }`；記錄全文由 `loadDocument(slug)` 取得（含 frontmatter，區段切分只認 `## Context`／`## Rounds`／`## Conclusion`）；`ChangeItem` 有 `completedTasks`／`totalTasks`、`reviewStatus`／`verifyStatus`／`reviewedAt`／`verifiedAt`；`ArchivedItem` 有 `date`、`tasksDone`／`tasksTotal`、`reviewStatus`／`verifyStatus`。宿主 store 有 `openSpec`、`openDiscussion`（live 優先、封存後備）、`openDetail`、`openArchived`、`specs`、`discussions.active／archived`。
- 記錄格式由引擎與 `/speclink-discuss` 技能擁有且已凍結：輪標題「### Round N — <mode> (<date>)」、輪四欄位 Focus／Position／Ruled out／Open、結論六欄位 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next；Position 與 Decision 為「一句定論＋`- ` 列點」；分刀寫在 Decision 的列點裡，形如「- **cut 1 `desktop-detail-doc-tree`**: 骨架——……」並有巢狀子項；背景的收尾行為 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:`。
- 原型：討論側四張畫板（ContentConclusion、ContentRounds、ContentContext、ContentPromoted，資料取自 desktop-ui-redesign）已由使用者確認；本設計照畫板落地，差異只在「文中名稱籤」的實作路徑（D10）。
- 約束：不改記錄格式與引擎；只用 `@speclink/ui` 既有原語與 token class，呼叫端不以 className 覆蓋變體色；不碰 Rust；與平行變更 `desktop-spec-manual-pages` 無同名需求（本刀不碰兩條 markdown 需求、「詳情頁的文件動作」與規格詳情頁）。

## Goals / Non-Goals

**Goals:**

- 討論詳情頁與已封存討論頁的結論、討論過程、背景、衍生變更四份文件各有渲染器，第一屏看得出層次：項目型內容用介面元件、散文只佔該佔的位置。
- 每份文件的數字磚、出處行與提示框全由記錄結構與既有清單資料推出，缺資料就少一格、沒有條件就沒有提示框。
- 解析純函式與渲染器分離（`discussionModel.ts` ↔ 四個 Doc 元件），與變更側的 `docModel.ts` ↔ 渲染器同一套分工；逐項降級、不拋錯。
- 兩頁共用同一份渲染器與推導入口；已封存討論頁的衍生列與討論詳情頁同一套資料形狀。

**Non-Goals:**

- 不改討論記錄的格式、scaffold、`speclink discuss` 技能與引擎 JSON；不新增 `discuss show --json` 欄位。
- 不改左欄身分區、文件節點、狀態列、標題列動作與「開啟 ▾」（cut 1 已定）。
- 不改規格詳情頁與手冊頁（`desktop-spec-manual-pages`）、不改變更側六份文件（cut 2）。
- 不做英文環境的 Position／Decision 定論句切分例外：切分規則只看「第一段 prose＋列點」，與語言無關。
- 不碰 Rust、Tauri、server-web。

## Decisions

### D1 討論記錄解析器 discussionModel.ts

新檔 `packages/ui/src/discussionModel.ts`（純函式、不碰 React；與 `docModel.ts` 對稱）。既有四支自 `DiscussionDetailPage.tsx` 搬入、簽名不變：`splitDiscussionSections(text): DiscussionSections | null`、`splitRounds(text): DiscussionRound[] | null`、`splitLabeledFields<L>(lines, labels)`、`countPriorDiscussions(context): number | null`，連同 `DiscussionSections`、`DiscussionRound`、`RoundLabel`、`ROUND_LABELS`、`ConclusionLabel`、`CONCLUSION_LABELS` 一起匯出（`index.ts` 的 `splitDiscussionSections` 改自此匯出）。新增：

- `parseConclusion(text): { lead: string; fields: Partial<Record<ConclusionLabel, string>> } | null` — 以六詞白名單切欄位；一個白名單欄位都沒有時回 null（呼叫端整篇 Markdown 退回，與既有規則同）。
- `splitVerdict(md): { verdict: string; details: ListItem[] }` — 用 `parseListItems`：第一個 prose 段為定論句（沒有則為空字串）、全部頂層列點依序為細節。Position 與 Decision 共用。
- `parseCuts(details: ListItem[]): { cuts: CutCard[]; rest: ListItem[] }` — `CutCard { id: string; title: string; names: string[]; items: string[] }`。一條列點的首行以粗體開頭（`**…**`）、粗體內去掉反引號片段後以 `cut`（大小寫不拘）或 `刀` 起頭、或符合「第 N 刀」時成為一張分刀卡：`id` 為粗體內去掉反引號片段後的文字（如 `cut 1`、`cut 3a`、`刀二`），`names` 為粗體內的反引號片段依序，`title` 為粗體之後去掉開頭冒號（半形或全形）、再以第一個「——」切掉後半的文字，`items` 為該列 body 的頂層列點文字（`parseListItems`）。其餘列點進 `rest`。沒有任何命中時 `cuts` 為空陣列。
- `splitRows(md): string[]` — 內文有列點時取頂層列點（`itemMarkdown`）；否則以頂層（反引號與括號之外）的 `；`／`;` 切段；去頭尾空白、丟空段。Ruled out、Open、Deferred 共用。
- `openItems(md): { none: boolean; items: string[] }` — 去頭尾空白後以「無」「None」「none」起頭即 `none: true`（items 為原文一條，供呈現「無：寫結論」這類說明）；否則 `items = splitRows(md)`。
- `roundStats(rounds): { modes: { mode: string; count: number }[]; days: number; first: string | null; last: string | null }` — mode 依首次出現序計數（透傳字串，不設白名單）；`days` 為不同日期字串數；`first`／`last` 為首末輪日期。
- `parseContextMeta(context): { specs: string[] | null; changes: string[] | null; prior: string[] | null; sourceDoc: string | null; body: string }` — 圍欄外、行首（容許前導空白）以 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:` 起頭的行自內文剝除成 `body`；名單解析：先去掉括號內備註（`（…）`／`(…)`），以 `,`／`，`／`、`／`;`／`；` 切項，每項取第一個 `:`／`：` 之前、再取第一個空白分隔的 token；`none`／`None` 起頭為空陣列；行缺席為 null。`sourceDoc`：行缺席 null、`none` 為空字串、其餘為去頭尾空白的原文。
- `splitLead(body): { lead: string; rest: string }` — 圍欄外第一個空行之前為引言段（引言本身為 prose；若第一段是列點或標題則 lead 為空、全部進 rest）。
- `matchNames(text, names: string[]): string[]` — 在圍欄與行內反引號之外的文字比對名稱：命中條件為名稱前後皆非 `[A-Za-z0-9_-]`（中文名稱亦適用此邊界）；名稱先依長度降冪比對避免前綴吞併；回傳依首次出現序去重的命中名單。
- `commandChips(md): string[]` — 擷取 `/\/speclink-[a-z-]+(?:\s+(?:--[a-z-]+|[A-Za-z0-9][\w.-]*))*/g` 的命中（反引號內外皆算），去重保序。

### D2 結論渲染器 ConclusionDoc

新檔 `packages/ui/src/components/ConclusionDoc.tsx`：`ConclusionDoc({ text, empty, stageOf, onOpenSpec, specNames })`。`parseConclusion` 為 null 時 `<Markdown content={text} empty={empty} />`。否則：lead（第一個標籤前的內文）先以 Markdown 排；其後依白名單順序只畫存在的欄位，每欄一個 `DocSection`（`sectionKey` 取 `conclusion.<field>`，標題取既有 `conclusion.*` 詞條；`h2 data-doc-heading` 為本頁目錄錨點）：

- 決定：`splitVerdict` 的定論句畫成 `p[data-verdict]`（`text-[17px] leading-relaxed`）；`parseCuts` 之後的 `rest` 為定案細節，`RowList` 預設只畫前 4 條，餘下以 `MoreToggle`「展開其餘 N 項定案細節」一次展開；`DocSection` 右端計數為細節總數。
- 分刀：`cuts` 非空時在「決定」之後多一段 `DocSection`（`sectionKey` `conclusion.cuts`、標題「分刀」、計數為卡數）；每刀一張 `section[data-cut=<id>]` 細框卡：卡頭為 id 籤（`NAME_CHIP_CLS`）、`title`、「N 個變更」；`names` 逐顆 `NameChip`，`stageOf(name)` 回 `{ label, tone, onOpen? }` 時籤旁附 `StatusPill` 階段籤、籤可點（`onOpen`）；回 null（不在 `promoted_to`）時為不可點籤無階段籤；`items` 為 `RowList`。
- 理由：Markdown。
- 否決替代案：`splitRows` 逐條 `splitAlternative` 成 `OptionReasonRows`；計數為列數。
- 擱置：同否決，但每列列首帶 `Clock` 圖示（`RowList` 的 `icon` 不適用於方案／理由兩行，`OptionReasonRows` 新增可選 `icon` prop）。
- 記錄去向：以 `、`／`,`／`，` 切成籤；籤文字命中 `specNames` 且有 `onOpenSpec` 時為可點 `NameChip`，其餘為不可點籤；切不出籤（整段散文）時照 Markdown。
- 下一步：Markdown 之後接一列 `commandChips` 的 `span[data-command-chip]`（等寬籤＋`CopyButton`，複製指令原文）；沒有命中時只有 Markdown。

### D3 討論過程渲染器 RoundsDoc

新檔 `packages/ui/src/components/RoundsDoc.tsx`：`RoundsDoc({ text, empty })`。`splitRounds` 為 null 時整篇 Markdown；空陣列時空態文案。否則 `ol[data-round-timeline]`（左側一條細線）每輪一個 `li[data-round=<N>]`：節點色點依 mode——`assumptions` 取 `bg-primary`、`interview` 取 `bg-stage-discussion`（看板討論欄色相 token）、其他 mode 取 `bg-muted-foreground/50`（mode 字串透傳、不設白名單）。輪卡卡頭為 `button[aria-expanded]`：輪次籤（`data-doc-heading`，「Round N」，本頁目錄錨點）、焦點（`Focus` 欄位首行，`h3`；缺 Focus 時只有輪次籤）、mode 籤（中性）、日期。展開後：lead（標籤前內文）Markdown；立場（`splitVerdict`：定論句 `p[data-position-verdict]` 字重 medium＋細節 `RowList`）；否決（`splitRows` 逐條，列首 `X` 圖示，`data-ruled-out`）；待決（`openItems`：`none` 時一行灰字帶 `Check` 圖示的原文 `data-open-none`，否則每條一顆琥珀籤 `span[data-open-item]`，款式 `SEMANTIC_SURFACE.warning`＋`SEMANTIC_TONE.warning`）。缺席欄位不畫空標籤（既有規則）。預設展開第一輪與最後一輪（只有一輪時即那一輪），其餘收合為卡頭一列；點卡頭切換。

### D4 背景渲染器 ContextDoc

新檔 `packages/ui/src/components/ContextDoc.tsx`：`ContextDoc({ text, empty, slug, created, createdBy, kind, recordPath, specNames, activeNames, archivedNames, discussionSlugs, onOpenSpec, onOpenChange, onOpenArchivedChange, onOpenDiscussion })`。`parseContextMeta` 剝掉四行收尾行後 `splitLead`。三段 `DocSection`（都是本頁目錄錨點）：

- 起因與範圍（`context.intro`）：引言段以 `Lead`（灰底）排，其餘以 `Markdown` 排並帶 `nameChips`（D10）：命中的規格名（`specNames`）、作用中變更名（`activeNames`）、已封存變更名（`archivedNames`）、討論 slug（`discussionSlugs`，排除本身）成可點籤；容器 `onClick` 以 `closest("[data-name-chip]")` 讀 `kind:name` 分派到四個回呼（缺回呼的種類為不可點籤）。全文為空時只畫空態文案。
- 關聯（`context.relations`）：鍵值列，每列值為籤列——正典規格＝`meta.specs` ∪ 文中命中的 `specNames`；在途變更＝`meta.changes`∩`activeNames` ∪ 文中命中的 `activeNames`；已封存變更＝`meta.changes`∩`archivedNames` ∪ 文中命中的 `archivedNames`；先前討論＝`meta.prior` ∪ 文中命中的 `discussionSlugs`；來源文件＝`meta.sourceDoc`（null 列缺席、空字串顯示「無」、其餘等寬文字）。值為空的列缺席；整段沒有任何列時「關聯」段缺席。籤的可點規則同上。
- 記錄（`context.record`）：建立＝`created` · `displayName(createdBy)`（缺席段略過）；種類＝`isImproveKind(kind)` 為「改進討論」否則「一般討論」；檔案＝`recordPath`（等寬＋`CopyButton`；缺席列缺席）。

### D5 衍生變更渲染器 PromotedDoc 與列資料推導

新檔 `packages/ui/src/components/PromotedDoc.tsx`：

- `PromotedRow { name: string; stage: { label: string; tone: StatusPillTone }; alive: boolean; tasks: { done: number; total: number } | null; review: boolean; verify: boolean; date: string | null; onOpen?: () => void }`。
- `promotedRows(names, changes, archivedChanges, t, openActive, openArchived): PromotedRow[]` — 依 `promoted_to` 原序逐名推導：在作用中清單→階段籤取 `STAGE_PILL_TONE[changeStage(c)]` 與既有 `discussionChipStage` 詞條、`tasks` 取 `completedTasks/totalTasks`、`review` 為 `reviewStatus` 為 reviewed 或 reviewedStale、`verify` 同理、`date` 取 `reviewedAt`／`verifiedAt` 中較晚者的日期段（皆缺席為 null）、`onOpen` 為 `openActive(name)`；在已封存清單→階段「已封存」中性、`tasks` 取 `tasksDone/tasksTotal`（任一缺席為 null）、`review` 為 `reviewStatus === "reviewed"`、`verify` 為 `verifyStatus === "verified"`、`date` 為封存日期、`onOpen` 為 `openArchived(datedName)`；兩者皆無→階段「已刪除」destructive、`alive: false`、無任務、無日期、不可開。兩頁共用這一支；已封存討論頁沒拿到兩份清單時退回 `SourceLinkItem`（階段籤文字＝`topic`、中性、`alive = !disabled`、無任務、無日期）。
- `PromotedDoc({ rows, cuts, empty })`：`rows` 為空時空態文案（既有 `ddrawer.notPromoted`）。`cuts` 非空時依 cut 分組：每組 `section[data-cut-group=<id>]`，組頭 `h3[data-doc-heading]`（id 籤、`title`、「N 個變更」）；組內列為該 cut `names` 中存在於 `rows` 的列、依 `names` 序；不屬任何 cut 的列接在最後、無組頭。`cuts` 為空時單一平面清單、無組頭、無本頁目錄錨點。每列 `div[data-promoted-row=<name>]`：`StatusPill` 階段籤（已刪除加刪除線）、等寬名稱、任務進度（細進度條＋`n/m`；`tasks` 為 null 缺席）、「審」「驗」小籤（各自只在 `review`／`verify` 為 true 時出現，`data-stamp`）、日期（等寬灰字；null 缺席）、右端 `ArrowUpRight` 開啟鈕（`onOpen` 缺席不畫）。`PromotedChangeRow` 退場；狀態列用的 `PromotedChangeChip` 不動。

### D6 推導單一入口 discussionDocSummary.tsx

新檔 `packages/ui/src/components/discussionDocSummary.tsx`，匯出 `discussionDocSummary(tab, data, t): DocSummary` 與 `discussionDocProvenance(tab, data, t): ReactNode[]`；`DiscussionDocData { sections: DiscussionSections | null; rounds: DiscussionRound[] | null; conclusion: ReturnType<typeof parseConclusion>; cuts: CutCard[]; status: string; hold: boolean; created: string | null; rows: PromotedRow[]; relations: { specs: number; changes: number; archived: number; prior: number | null }; recordPath: string | null }`。記錄未載入或區段切分失敗時三者皆空、出處行為空陣列。

- 結論：磚依序「定案細節」（`rest` 數）、「分刀」（值為卡數、標籤「分刀 · N 個變更」，`cuts` 為空缺席）、「否決的替代方案」（列數；欄位缺席則缺席）、「擱置」（同）；出處行為 `recordPath`（null 缺席）與「N 輪之後寫入」（N 為輪數；結論欄位全缺時出處行只剩路徑）；無籤、無提示框。
- 討論過程：磚「輪」（總數）、「天」（不同日期數）、其後依首次出現序每個 mode 一格（標籤為 mode 原字串、值為輪數），最多四格；出處行為「首日 – 末日」（同日只寫一次）與各 mode 輪數；提示框：`conclusion` 非 null 時 tip「討論已收斂」，內文「第 N 輪之後寫入結論」＋末輪 `Open` 為 `none` 時「最後一輪待決：無」、否則「最後一輪仍列 M 項待決」（末輪缺 Open 時只寫前半句）。
- 背景：磚「命中的正典規格」「在途變更」「相關已封存變更」（各為關聯列的籤數，0 缺席）、「先前討論」（`prior` 為 null 缺席，含 0）；出處行「記錄建立時寫入一次」與建立日；提示框：首輪 mode 為 `assumptions` 時 note「沒有 grill 階段」，內文「首輪直接列假設」。
- 衍生變更：磚「變更」（列數）、「已封存」、「作用中」、「任務」（作用中與已封存列的 done／total 加總，`n/m`；沒有任何任務資料缺席）；出處行「依結論的分刀分組」（`cuts` 非空）或「依轉出順序」；提示框：`status === "promoted"` 且 `hold` 為 false 且恰有一列 `alive` 時 tip「只差最後一刀」，內文「<name>（<階段>）封存後，這份討論會自動隨之封存」。

### D7 兩頁的接線

`DiscussionDetailPage.tsx`：刪 `RoundsView`、`ConclusionView` 與搬走的解析；新增 props `specNames?: string[]`、`discussionSlugs?: string[]`、`onOpenSpec?`、`onOpenDiscussion?`、`onOpenArchivedChange?: (datedName: string) => void`。頁內推導 `rounds`、`conclusion`、`cuts`（取自 `splitVerdict(conclusion.fields.Decision)` 的細節）、`rows`（`promotedRows`，`openActive` 走 `onOpenChangeCard`、`openArchived` 走 `onOpenArchivedChange`）；`DocHeader` 接 `discussionDocProvenance`、`DocTiles` 接磚、磚與 `DocBody` 之間放 `div[data-doc-callout]`；四個 `TabsContent` 分別掛 `ConclusionDoc`（`stageOf` 由 `rows` 查）、`RoundsDoc`、`ContextDoc`（`recordPath` 為 `openspec/discussions/<slug>.md`，`activeNames`／`archivedNames` 自兩份清單取名）、`PromotedDoc`。記錄格式非預期時「記錄」文件維持整篇 Markdown、衍生變更仍走 `PromotedDoc`（`cuts` 為空）。

`ArchivedDetailPage.tsx` 的討論變體：新增 props `changes?: ChangeItem[]`、`archivedChanges?: ArchivedItem[]`、`specNames?: string[]`、`discussionSlugs?: string[]`；兩份清單都有時 `rows` 走 `promotedRows`（`openActive`／`openArchived` 都委派既有 `onOpenPromotedChange(name)`，宿主已會依所在開頁），否則退回 `promotedChanges` 的 `SourceLinkItem`；`recordPath` 為 null（出處行與記錄列都不出路徑）；`status` 傳 `"archived"`（不出「只差最後一刀」）；名稱籤的變更回呼走 `onOpenPromotedChange`、討論走 `onOpenDiscussion`、規格走 `onOpenSpec`。

### D8 宿主接線

`apps/desktop/src/App.tsx`：`DiscussionDetailPage` 加 `specNames={s.specs.map((x) => x.id)}`、`discussionSlugs={[...s.discussions.active, ...s.discussions.archived].map((d) => d.slug)}`、`onOpenSpec={s.openSpec}`、`onOpenDiscussion={s.openDiscussion}`、`onOpenArchivedChange={(datedName) => s.openArchived({ kind: "change", datedName })}`；`ArchivedDetailPage` 加 `changes={s.changes}`、`archivedChanges={s.archived}`、`specNames`、`discussionSlugs`（同上）。store 不新增動作。

### D9 i18n 詞條

`packages/ui/src/i18n.tsx` 兩語系同鍵集：`conclusion.cuts`、`conclusion.cutChanges`、`conclusion.moreDetails`、`conclusion.tileDetails`、`conclusion.tileCuts`、`conclusion.tileRejected`、`conclusion.tileDeferred`、`conclusion.afterRounds`、`conclusion.copyCommand`；`rounds.tileRounds`、`rounds.tileDays`、`rounds.modeRounds`、`rounds.dateRange`、`rounds.converged`、`rounds.convergedBody`、`rounds.convergedOpenNone`、`rounds.convergedOpenLeft`；`context.intro`、`context.relations`、`context.record`、`context.relSpecs`、`context.relChanges`、`context.relArchived`、`context.relPrior`、`context.relSource`、`context.none`、`context.created`、`context.kind`、`context.kindImprove`、`context.kindGeneral`、`context.file`、`context.tileSpecs`、`context.tileChanges`、`context.tileArchived`、`context.tilePrior`、`context.noGrill`、`context.noGrillBody`、`context.writtenOnce`；`promoted.byCuts`、`promoted.byOrder`、`promoted.tileChanges`、`promoted.tileArchived`、`promoted.tileActive`、`promoted.tileTasks`、`promoted.lastCut`、`promoted.lastCutBody`、`promoted.stampReview`、`promoted.stampVerify`、`promoted.groupChanges`。既有 `conclusion.*` 六個標籤、`rounds.*` 四個標籤、`ddrawer.openCard`、`ddrawer.notPromoted` 沿用。

### D10 Markdown 的名稱籤外掛

`packages/ui/src/components/Markdown.tsx` 新增可選 prop `nameChips?: { kind: string; name: string }[]`：有值時 rehype 管線在 sanitize 之後接 `[rehypeNameChips, { names }]`（以選項物件傳入——Streamdown 以外掛函式名＋選項 JSON 快取 processor，閉包式外掛會讓不同文件共用同一份），外掛走訪文字節點（略過 `code`／`pre`／`a` 子樹），以 D1 `matchNames` 同一條邊界規則把命中片段換成 `<span class="name-chip" data-name-chip="<kind>:<name>">name</span>`（接在 sanitize 之後，不經放行清單，與 `rehypeSpecKeywords` 同一路）。籤款式由呼叫端容器以 `[&_.name-chip]:…` 任意變體給（等寬、灰底、圓角、主色、指標手形）；點擊由呼叫端容器攔。`nameChips` 缺席時管線不變、輸出逐位元不變。

### D11 退場與匯出

`DocParts.tsx` 刪 `LABEL_CLS`（最後消費者退場）；`DiscussionDetailPage.tsx` 刪 `RoundsView`、`ConclusionView`、`PromotedChangeRow`；`index.ts` 匯出 `ConclusionDoc`、`RoundsDoc`、`ContextDoc`、`PromotedDoc`、`promotedRows`、`discussionDocSummary`、`discussionDocProvenance` 與 `discussionModel.ts` 的全部解析函式與型別（`splitDiscussionSections` 的匯出來源改為 `discussionModel`）。

## Implementation Contract

- **行為**：開啟任一討論詳情頁或已封存討論頁，結論文件呈現決定主句、定案細節（預設前 4 條＋展開鈕）、分刀卡（有 cut 列點時）、理由、否決與擱置的方案／理由列、記錄去向籤與下一步指令籤；討論過程文件呈現時間軸，首末輪展開、其餘收合，每輪卡頭為輪次、焦點、mode、日期；背景文件呈現引言、帶名稱籤的內文、關聯列與記錄列；衍生變更文件依分刀分組（讀不到時平面清單），每列階段籤、名稱、任務進度、兩站章、日期與開啟鈕。四份文件的閱讀卡標題下有出處行與數字磚；提示框只在三個條件成立時出現。點名稱籤、分刀卡的變更籤、記錄去向的規格籤與衍生列的開啟鈕分別開啟對應的規格詳情頁、變更詳情頁、已封存詳情頁或討論詳情頁。
- **介面與資料形狀**：D1 的函式簽名；`ConclusionDoc`／`RoundsDoc`／`ContextDoc`／`PromotedDoc` 的 props（D2–D5）；`PromotedRow`、`CutCard`、`DiscussionDocData`（D5、D1、D6）；`DiscussionDetailPage` 與 `ArchivedDetailPage` 新增的 props（D7）；`Markdown` 的 `nameChips`（D10）。DOM 錨點：`[data-verdict]`、`[data-cut]`、`[data-command-chip]`、`[data-round-timeline]`、`[data-round]`、`[data-position-verdict]`、`[data-ruled-out]`、`[data-open-item]`、`[data-open-none]`、`[data-name-chip]`、`[data-cut-group]`、`[data-promoted-row]`、`[data-stamp]`、`[data-doc-callout]`、`[data-doc-tiles]`、`[data-doc-provenance]`、`[data-doc-heading]`。
- **失敗模式**：六欄位全缺的結論、非 scaffold 的討論過程、缺區段的記錄——維持既有整篇 Markdown 退回，不報錯；分刀讀不到→無分刀段、衍生變更平面清單；否決／擱置切不出「——」→整條為方案；背景四行缺席→對應關聯列缺席；名稱比對零命中→無籤；數字磚缺資料→少一格；記錄載入中→骨架、載入失敗→空態（既有三態）。渲染不寫任何檔案。
- **驗收**：`npm test -w packages/ui -- discussionModel conclusionDoc roundsDoc contextDoc promotedDoc discussionDetailPage archivedDetailPage markdown i18n` 與 `npm test -w apps/desktop -- App` 全綠；`speclink validate desktop-discussion-doc-views` 通過；`grep -rn "LABEL_CLS\|RoundsView\|ConclusionView\|PromotedChangeRow" packages/ui/src apps/desktop/src` 零命中；手動以本討論（detail-content-redesign，三刀）與 desktop-ui-redesign（已封存、13 個變更、7 刀）各看一次四份文件。
- **範圍邊界**：in——`packages/ui` 的討論側渲染器、解析器、文案、匯出與兩頁接線，`apps/desktop/src/App.tsx` 的 props 接線與測試；out——記錄格式、引擎、Rust／Tauri、左欄與標題列、規格詳情頁、手冊頁、變更側文件、`docModel.ts` 既有函式。

## Risks / Trade-offs

- [分刀列點的寫法不一（`cut 1`、`cut 3a`、`刀二`、反引號缺席）] → `parseCuts` 只認「粗體開頭＋cut／刀」與反引號片段，認不出就沒有分刀段、衍生變更退回平面清單；`discussionModel.test.ts` 以 detail-content-redesign 與 desktop-ui-redesign 兩份真實結論當案例。
- [名稱比對誤命中（短名稱出現在他名之內、中文名無邊界）] → 長名稱優先、前後字元邊界；只對宿主清單裡存在的名稱比對，不做模糊比對；誤命中只多一顆籤、不影響內文。
- [Streamdown 的 processor 快取] → `rehypeNameChips` 以選項物件形式傳入（函式名＋選項 JSON 為快取鍵），不同討論的名單各得一份 processor；`nameChips` 缺席時管線與既有輸出逐位元相同（`markdown` 測試既有題目不改）。
- [已封存討論頁拿不到兩份清單（舊宿主）] → 退回 `SourceLinkItem` 三態，列只少任務進度、兩站章與日期。
- [回歸對照] → 前端 vitest（`packages/ui`、`apps/desktop`）為守門；不動 Rust、golden 與 CLI 測試；跨平台只涉及 WebView 渲染，無平台分支。
- [本頁目錄錨點數暴增（20 輪＋7 刀）] → 右欄可捲動、隨閱讀位置高亮（cut 1 既有），錨點只收卡頭與區段標題、不收欄位。

## Migration Plan

1. D1 解析器（先寫紅燈測試）→ D10 Markdown 外掛 → D2–D5 四個渲染器（各自測試）→ D6 推導入口 → D7 兩頁接線與既有測試改寫 → D8 宿主接線 → D9 文案與 D11 退場。
2. 無資料遷移、無設定變更；舊檢視與 `LABEL_CLS` 在同一刀刪除，無回滾路徑需求（整刀 revert 即回到 cut 2 狀態）。

## Open Questions

- 無——討論期擱置的「引擎回拆好的 rounds」與「英文環境切分」皆不在本刀。
