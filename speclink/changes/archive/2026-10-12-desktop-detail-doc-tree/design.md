## Context

- 三種詳情頁現況：`packages/ui/src/components/ChangeDetailPage.tsx`、`DiscussionDetailPage.tsx`、`ArchivedDetailPage.tsx` 共用 `DetailHeader.tsx` 的骨架——`DetailPageFrame`（置中 768px 直欄）、`DetailHeader`（標題列＋出身列，動作區在同列右端）、`StatusCard`／`StatusRow`（狀態卡）、`DetailContentCard`（接在 tabs 原語 card 變體分頁列下方的內容卡）。「開啟 ▾」由宿主以 `renderDocumentActions` 傳進來，渲染在 `TabsList` 的 `actions` 槽。`SpecDetailPage.tsx` 也用 `DetailHeader`、`DetailPageFrame`、`DetailContentCard`，本刀不動它。
- 宿主 `apps/desktop/src/App.tsx` 的主區標題列是 `MainTitleBar`（`apps/desktop/src/components/TitleBar.tsx`），`leading` 槽放返回鈕、`trailing` 槽放 `DetailNav`（‹ n／m ›）；標題列整條是 `data-tauri-drag-region="deep"`，槽內的按鈕已證明可點。變更層動作（分析、認領、退回提案中、封存、⋯）的守門與停用原因都在 `ChangeDetailPage` 內組好（`UnavailableAction`、`archiveReason`、`deleteReason`）。
- 本頁目錄的現成實作是 `packages/ui/src/components/ManualToc.tsx`：掃 `bodyRef` 內符合 `headings` 選擇器的元素、給 id、捲動高亮；規格詳情頁已用 `@container` 加 `@min-[1200px]` 在 768px 欄左側空白處顯示它。
- GitHub Alert 提示框的樣式字串（`ALERT_BOX_CLS`、`ALERT_TITLE_CLS`、語意色 `SEMANTIC_SURFACE`／`SEMANTIC_TONE`）目前是 `Markdown.tsx` 的私有常數，sanitize 白名單也由它們展開。
- 工單輪次由 `TicketView.tsx` 的 `useStationTickets` 載入，分頁出現時才載。
- 守門：`packages/ui/src/__tests__/theme.test.ts` 禁止原生色階字面；`changeDetailPage.test.tsx`（94 題）、`discussionDetailPage.test.tsx`（61 題）、`archivedDetailPage.test.tsx`（33 題）與 `apps/desktop/src/__tests__/App.test.tsx` 的「詳情頁層與導覽」群組以 `getByRole("tab")`、`data-status-card`、`data-title-row` 等錨點釘住現有結構。
- 互動原型：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底三列十張畫板，各畫板右側 Tweaks 有深色模式。
- 落點：純前端——`packages/ui`（骨架、原語、頁）與 `apps/desktop`（宿主標題列）。不碰 Rust crate、Protocol、CLI 與 server。

## Goals / Non-Goals

**Goals:**

- 三種詳情頁共用一個三欄骨架：左欄文件樹（身分區、文件節點、狀態列）、中欄閱讀卡、右欄本頁目錄；無底部列。
- 變更層動作進主區標題列右端；文件層「開啟 ▾」進閱讀卡標題列右端。
- 文件節點保留 tab／tabpanel 語意與鍵盤導覽，既有測試的 `getByRole("tab")` 錨點不必重寫。
- 十份文件以現有渲染器落進新骨架，只補目錄錨點；數字磚與提示框只做容器與原語。
- 提示框樣式單一來源，Markdown 的 `> [!NOTE]` 四型與後兩刀共用。

**Non-Goals:**

- 各文件的新畫法、數字磚內容與提示框內容（cut 2、cut 3）。
- 規格詳情頁、手冊頁、server-web。
- 導覽與返回、互斥、進出動畫、載入三態的語意。

## Decisions

### D1 骨架元件 `DetailShell`（`packages/ui/src/components/DetailShell.tsx`）

- `DetailShell` 為三欄外框：`<div data-detail-shell className="@container flex h-full min-h-0 w-full">`。
  - 左欄 `<aside data-detail-tree className="flex w-[240px] shrink-0 flex-col overflow-y-auto border-r border-border px-2 pt-3.5 pb-3">`，內容由 `tree` 槽傳入。
  - 中欄 `<div data-detail-reading ref={scrollerRef} className="min-h-0 min-w-0 flex-1 overflow-y-auto px-6 py-7">` 內放 `<div className="mx-auto w-full max-w-[768px]">{children}</div>`；`READING_COLUMN_CLS`（96ch）仍套在閱讀卡內文上，兩者取小。
  - 右欄 `<aside data-detail-toc className="hidden w-[200px] shrink-0 @min-[1200px]:flex">` 內放 `ManualToc`，`bodyRef` 指向閱讀卡的 `data-doc-body`、`headings` 為 `"[data-doc-heading], .markdown h2, .markdown h3"`、`markdownBody` 為目前語系＋呼叫端給的鍵（文件 id＋內容字串；換文件、內容重載或換語系就重掃——錨點文字多為譯文）、`className="flex border-l-0"`。`@container` 量的是主區寬：主區 ≥1200px 才顯示右欄；`ManualToc` 沒有錨點時自己不渲染，右欄空著也不佔視覺（無邊框）。
  - 切換文件時 `scrollerRef.current.scrollTop = 0`（由 `activeDoc` 變化的 layout effect 做，畫面畫出前歸零、不閃前一份的位置）；左欄自己捲動、不受影響。
  - 同檔另有三頁共用的 `DOC_TITLE_KEYS`（文件 → 閱讀卡標題詞條）與 `changeDocTocKey`（變更文件的目錄重掃鍵），不自 `index.ts` 匯出。
- `DetailIdentity`（左欄身分區）：props `title`（等寬 14px semibold、`break-all`）、`copyLabel`／`copyValue`（沿用 `CopyButton`）、`badges`（籤列，`flex-wrap`）、`provenance: ReactNode[]`（一行 11px 灰字，缺席段略過，以「·」分隔，`truncate`）、`relations: { mark: string; label: string; children: ReactNode }[]`（每筆一行：標籤＋內容，內容 `flex-wrap`；`data-relation-<mark>` 供測試）、`subtitle`（討論 topic，`line-clamp-3`）。`IdentityName`、`displayName` 自 `DetailHeader.tsx` 沿用。
- `DetailTreeLabel`：左欄小標（「文件」「狀態」），11px semibold 灰字，與手冊頁側欄樹的分組標題同款。
- `DetailStatus`：`<section data-status-card>` 包 `StatusRow` 清單（`StatusRow` 原樣沿用，標籤寬改 `w-14`、字級 12px），放在文件節點之下。
- `DocHeader`（閱讀卡標題區）：`<header data-doc-header>`：`h2` 24px 一般字重（身分區的名稱已是頁的 `h1`）（`text-2xl font-normal`）、`badges` 槽、`actions` 槽（`ml-auto`，放「開啟 ▾」）、其下 `provenance: ReactNode[]`（13px 灰字、「·」分隔、可折行）。
- `DocTiles`：`tiles: { value: ReactNode; label: string }[]`，四欄 grid（`grid-cols-4 gap-2.5`），每格細框 12px 圓角、值 20px semibold tabular、標籤 11px 灰字；`tiles` 空陣列時回 `null`。本刀所有頁都傳空陣列（容器就位、不放磚）。
- `DocBody`：`<div data-doc-body ref>` 包各文件的渲染器輸出，`ManualToc` 的 `bodyRef` 指它。
- 閱讀卡本體為 `Card`（既有原語）`p-6`，由上而下為 `DocHeader`、`DocTiles`（本刀傳空陣列、不渲染，標出 cut 2／3 的插入點）、提示框槽（位在數字磚與內文之間，本刀不放元素）、`DocBody`。
- 以上全部從 `packages/ui/src/index.ts` 匯出；`apps/*` 不得重新定義同名元件（既有守門測試）。

### D2 文件節點＝tabs 原語新增 `tree` 變體

- `packages/ui/src/components/ui/tabs.tsx` 的 `tabsListVariants` 新增 `tree`：`flex-col items-stretch gap-0.5 w-full`，觸發鈕款式 `TREE_TRIGGER`：`flex h-[30px] items-center gap-2 rounded-lg px-2.5 text-[13px] text-foreground hover:bg-foreground/5 data-[state=active]:bg-primary/12 data-[state=active]:text-primary data-[state=active]:font-medium`（主色淡底＝選取語意，與專案欄的作用中頁同款）；徽章靠右以 `ml-auto`。
- 頁的 `Tabs` 根改 `orientation="vertical"`、`activationMode="manual"`（方向鍵只移焦點、Enter／Space 才切換，避免掃過節點就逐一載文件）；`TabsList variant="tree"` 放在左欄「文件」小標之下，`TabsContent` 放在閱讀卡的 `DocBody` 內。Radix 保留 `role="tab"`／`tabpanel` 與 `aria-selected`，既有測試的 `getByRole("tab", { name })` 照用。
- card 變體不刪：已封存頁的變更／討論切換仍用它；詳情頁三處不再用。

### D3 變更層動作進標題列：`DetailActions` 插槽

- `DetailShell.tsx` 另匯出 `DetailActionsSlotProvider`（宿主提供插槽節點，`createContext<HTMLElement | null>`，值由宿主以 callback ref 取得的 DOM 節點）、`DetailActionsSlot`（宿主渲染的空 `<div data-detail-actions className="flex items-center gap-1.5">`）與 `DetailActions`（頁渲染，`children` 以 `createPortal` 送進插槽；沒有 provider 時原地渲染 children，讓 `packages/ui` 的單元測試與無宿主情境照常看得到按鈕）。
- 宿主 `App.tsx` 的 `MainTitleBar trailing` 改為 `<><DetailNav … /><span className="mx-1.5 h-5 w-px bg-border" /><DetailActionsSlot /></>`（細線只在插槽有子節點時出現：插槽以 `:has(>*)` 不可靠，改由 `DetailActions` 自己在 portal 內容最前面渲染細線）；`DetailActionsSlotProvider` 包住主區，`value` 為插槽節點。
- `ChangeDetailPage` 把現有 `actions` 片段整個搬進 `<DetailActions>`；守門、停用原因、`UnavailableAction` 不動。按鈕款式：「分析」`variant="outline" size="sm"`；階段主要鈕 `size="sm"`（default 變體＝實心主色）；「⋯」改 `variant="outline" size="icon-sm"`（原 subtle）。順序由左而右：分析、認領（有）、退回提案中／封存（依階段）、⋯。
- `DiscussionDetailPage` 的封存動作（concluded 或保留在途）同樣經 `DetailActions` 進標題列；`ArchivedDetailPage` 不渲染 `DetailActions`。
- 標題列高度 48px 不變；按鈕 28px；`data-tauri-drag-region="deep"` 下按鈕可點（與 `DetailNav` 同一條件）。

### D4 「開啟 ▾」進閱讀卡標題列

- `renderDocumentActions` 的回傳改放進 `DocHeader` 的 `actions` 槽；`apps/desktop/src/components/DocumentActionsMenu.tsx` 的觸發鈕改 `variant="outline" size="sm"`（原 ghost）。
- 排程文件 `docTarget` 為 null 不渲染；工單文件不載原文不渲染；remote 無 checkout 只剩「複製內容」——規則沿用 `documentTarget` 與 `DocumentActionsMenu` 現有邏輯。

### D5 三頁落進骨架

- **ChangeDetailPage**：`Tabs` 根包住 `DetailShell`；左欄 `tree` 槽依序為 `DetailIdentity`（title＝變更名、badges＝階段籤＋章籤、provenance＝建立者／工具／建立相對時間／開工、relations＝來自（既有來源討論籤含「+N」浮層）、同源、規格（頁經 `loadCapabilities` 載入的 delta capability 每個一顆 `Badge variant="outline"` 等寬籤——`ChangeItem` 不帶能力清單；點擊 `setTab("specs")` 並選定該 capability：「開啟 ▾」指向它；規格文件捲到它——捲動是一次性請求，由該 capability 段掛上後自己的 effect 執行（Radix 的 TabsContent 在切換後下一輪才掛上內文），捲完即清，自文件樹切回規格文件時自頂端呈現））、`DetailTreeLabel`「文件」、`TabsList variant="tree"`（提案、設計、任務＋`Badge` n/m、規格＋`DeltaBadges`、排程＋波次籤（`planWave` 有值時「第 N 波」）、審查／驗證條件式＋末輪序號籤）、`DetailTreeLabel`「狀態」、`DetailStatus`（既有三列）。分析結果卡（`data-verb-result`）放閱讀卡之上。閱讀卡 `DocHeader` 的 `title` 為目前文件的分頁詞條（`common.tabProposal` 等）、`provenance` 為該文件相對變更目錄的路徑（`DetailHeader.tsx` 的 `documentArtifact(docTarget)`，即 `documentTarget` 的 artifact 路徑，如 `proposal.md`；排程為 `plan.derivedFrom`「由 speclink plan 推算」詞條）。相對 checkout 根的路徑只有宿主算得出來（worktree 副本、spec 目錄名依設定），頁 props 不變的前提下不顯示；各文件的摘要段屬 cut 2 的渲染器。階段籤的 tone `STAGE_PILL_TONE` 放 `DetailHeader.tsx`，與討論頁的衍生變更籤共用。
  - 末輪序號籤：把 `useStationTickets` 的呼叫自 `TicketTabBody` 上提到頁層（節點出現即載入，與規格「工單 SHALL 於節點出現時載入」一致），`TicketTabBody` 改接 `ticket` prop（已是）。
- **DiscussionDetailPage**：`DetailIdentity`（title＝slug、badges＝狀態籤＋改進籤、subtitle＝topic、provenance＝建立者／N 輪／建立日、relations＝「關聯」列：自 `sections.context` 的 `Prior discussions:` 行以逗號計數，`none` 或缺行時列缺席，內容為「先前討論 N」籤，點擊 `setTab("context")`）、文件節點 結論／討論過程 N／背景／衍生變更 N（切分失敗時 記錄／衍生變更），`DetailStatus`（階段步驟列、保留在途、衍生變更）。預設文件規則不變（`defaultDiscussionTab`）。分頁狀態由 `defaultValue` 改受控 `value`，才能從關聯列切到背景；所選文件不在目前節點組時退回預設，由 `discussionDoc(picked, sections)` 決定（與已封存討論頁共用）。閱讀卡 SHALL NOT 有出處行（討論文件的出處屬 cut 3）。
- **ArchivedDetailPage**：變更變體 `DetailIdentity`（badges＝「已封存」籤＋結局籤、provenance＝建立者／建立日／封存日、relations＝來自）、節點 提案／設計／任務／規格／條件式審查、驗證；討論變體（slug、已封存籤、改進籤、topic、provenance、節點 結論／討論過程／背景／衍生變更）；`DetailStatus` 沿用現有三列；不渲染 `DetailActions`；「開啟 ▾」進 `DocHeader`；出處行規則與變更頁、討論頁相同（變更變體為相對變更目錄的路徑，討論變體無出處行）。
- 目錄錨點：`SectionedDoc` 的章節標籤 div 改 `h2`／`h3` 並加 `data-doc-heading`；`DeltaBadges.tsx` 的 `DeltaSpecView` 色標標頭加 `data-doc-heading`；`TaskList` 群組標題加 `data-doc-heading`；`ChangeDetailPage` 的 `PlanCard` 標題加 `data-doc-heading`；`TicketView` 的輪次標題加 `data-doc-heading`；`DiscussionDetailPage` 的 `RoundsView` 卡頭與 `ConclusionView` 欄位標籤加 `data-doc-heading`。款式本刀不動。

### D6 `Callout` 原語（`packages/ui/src/components/ui/callout.tsx`）

- `Callout`：props `tone: "note" | "tip" | "warning" | "caution"`、`title`、`icon?`、`children`；款式＝現有 `ALERT_BOX_CLS`＋`SEMANTIC_SURFACE[tone]`，標題列＝`ALERT_TITLE_CLS`＋`SEMANTIC_TONE[tone]`；四型對應語意色 note→inProgress、tip→success、warning→warning、caution→danger（與 Markdown 現有對照相同）。
- `callout.tsx` 匯出 `CALLOUT_BOX_CLS`、`CALLOUT_TITLE_CLS`、`CALLOUT_TONE`（型→語意色）；`Markdown.tsx` 改引用它們（刪掉私有常數），sanitize 白名單由同一組字串展開——Markdown 提示框的輸出 class 不變，既有 `markdownAlerts.test.tsx` 不動。
- 閱讀卡的提示框槽就是放 `Callout` 的位置；本刀不放任何實例。

### D7 寬度門檻與無目錄狀態

- 右欄顯示門檻以 `@container` 量主區：≥1200px 顯示。jsdom 不跑 container query，測試只斷言右欄帶 `@min-[1200px]:flex` 與 `hidden`。
- 文件沒有任何錨點（背景、衍生變更）時 `ManualToc` 回 null，右欄留空。

### D8 i18n 與詞彙

- `packages/ui/src/i18n.tsx` 新增（tw／en）：`detail.tree`「文件」／"Documents"、`detail.relFrom`「來自」／"From"、`detail.relSibling`「同源」／"Siblings"、`detail.relSpecs`「規格」／"Specs"、`ddetail.relations`「關聯」／"Related"、`ddetail.priorDiscussions`「先前討論 {n}」／"{n} prior discussions"、`plan.derivedFrom`「由 speclink plan 推算」／"Computed by speclink plan"。右欄標題沿用 `manual.toc`。兩語系鍵集合維持相等。
- `openspec/LANGUAGE.md` 詞彙新增三條：「文件樹」（definition：詳情頁左欄——身分區、文件節點與狀態列，與手冊頁的側欄樹對稱；avoid：文件分頁列；why：節點是整份文件、不是分頁）、「本頁目錄」（definition：閱讀卡右側自渲染後標題推導的錨點列，手冊頁與詳情頁共用；avoid：右欄目錄；why：手冊頁既有詞）、「數字磚」（definition：閱讀卡標題下由文件結構推出的計數格，最多四格；avoid：KPI 磚；why：不是指標儀表，是文件的計數）。

### D9 退場與匯出

- `StatusCard` 失去全部消費者 → 自 `DetailHeader.tsx` 與 `index.ts` 移除；`DetailHeader`、`DetailPageFrame`、`DetailContentCard` 仍由 `SpecDetailPage` 使用，保留。
- `index.ts` 新增匯出：`DetailShell`、`DetailIdentity`、`DetailTreeLabel`、`DetailStatus`、`DocHeader`、`DocTiles`、`DocBody`、`DetailActions`、`DetailActionsSlot`、`DetailActionsSlotProvider`、`Callout`。

## Implementation Contract

- **Behavior**：開任一變更、討論或已封存詳情頁，主區為左欄 240px 文件樹（身分區、「文件」節點、「狀態」列）、中欄置中 768px 閱讀卡（24px 標題＋「開啟 ▾」、出處行、內文）、主區 ≥1200px 時右欄 200px 本頁目錄；主區標題列右端為 ‹ n／m ›、細線、分析、依階段主要鈕、⋯（討論頁為封存；已封存頁無）。點文件節點即切換閱讀卡並捲回頂端，節點立即高亮、內文未到時 skeleton。十份文件的內容與現在相同。
- **Interface / data shape**：`ChangeDetailPage`、`DiscussionDetailPage`、`ArchivedDetailPage` 的 props 不變（`renderDocumentActions` 仍為「開啟 ▾」來源）。新匯出見 D9；`Tabs` 原語新增 `variant="tree"`。DOM 錨點新增 `data-detail-shell`、`data-detail-tree`、`data-detail-reading`、`data-detail-toc`、`data-doc-header`、`data-doc-body`、`data-doc-heading`、`data-detail-actions`、`data-relation-<mark>`；保留 `data-status-card`、`data-status-<mark>`、`data-change-detail`、`data-discussion-detail`、`data-archived-detail`、`data-verb-result`、`data-plan-section`；移除 `data-title-row`、`data-provenance-row`、`data-reading-column` 在三頁的使用（`SpecDetailPage` 仍有）。
- **Failure modes**：沒有 `DetailActionsSlotProvider` 時動作原地渲染（無宿主情境）；`ManualToc` 無錨點回 null；文件載入失敗沿用各頁既有收斂（骨架或前值）。沒有新的錯誤路徑。
- **Acceptance criteria**：`npm test -w packages/ui -- detailShell changeDetailPage discussionDetailPage archivedDetailPage markdownAlerts tabs i18n theme` 全綠；`npm test -w apps/desktop -- App` 全綠（含新增「標題列含分析與封存、閱讀卡標題列含開啟 ▾、無 card 分頁列」題）；`npm run build -w apps/desktop` 成功；`speclink validate desktop-detail-doc-tree` 通過；macOS 手動依規格「詳情頁的文件樹、閱讀卡與本頁目錄」五個 scenario 對照。
- **Scope boundaries**：in——`packages/ui` 的 `DetailShell.tsx`、`ui/callout.tsx`、`ui/tabs.tsx`、`DetailHeader.tsx`、三個詳情頁、`Markdown.tsx`、`SectionedDoc.tsx`、`DeltaBadges.tsx`、`TaskList.tsx`、`TicketView.tsx`、`SourceDiscussionChip.tsx`（前綴標籤可省略）、`i18n.tsx`、`index.ts`、對應測試；`apps/desktop` 的 `App.tsx`、`DetailNav.tsx`、`DocumentActionsMenu.tsx`、`App.test.tsx`、`remoteCapabilities.test.tsx`（動作鈕改在標題列查詢）；`openspec/LANGUAGE.md`。out——`SpecDetailPage`、`ManualPage`、server-web、各文件渲染器的款式與內容推導、Rust 與 Protocol。

## Risks / Trade-offs

- [Radix Tabs 直向＋manual 啟動改變鍵盤行為：方向鍵不再直接切頁] → 與瀏覽器分頁列慣例相同，Enter／Space 切換；既有測試只用點擊，不受影響。
- [`getByRole("tab")` 錨點保住，但 `data-title-row`／`data-provenance-row` 斷言失效] → 三個測試檔的對應題改用 `data-relation-*` 與身分區選擇器；題數不減。
- [標題列 48px 放五個控制項，窄視窗會擠] → 變更名不在標題列（在左欄），麵包屑 `truncate`；控制項固定 28px 不折行；主區最窄 1024px 時仍放得下（麵包屑剩約 500px）。
- [portal 進標題列後 Esc 的浮層順序] → 「⋯」選單仍是 Radix DropdownMenu，Esc 先關選單的既有處理不變（App.test 既有題覆蓋）。
- [`@container` 在 jsdom 不生效] → 單元測試只斷言 class token；寬窄兩態的實際驗證在 [M] 手動任務。
- [跨平台] → 純 CSS 與 React，Windows 自繪視窗鈕在 `trailing` 之後、不受插槽影響（`MainTitleBar` 結構不動）；golden 與 CLI 測試無變動面。

## Migration Plan

無資料遷移。元件退場（`StatusCard`）與測試改寫在同一變更內完成；`speclink update` 不受影響（無技能檔變動）。

## Open Questions

無。
