## Context

- 骨架與零件（cut 1、cut 2 已封存）：`DetailShell`（左欄 240px 文件樹、中欄 768px 閱讀卡、右欄 200px 本頁目錄，≥1200px 才顯示；`ManualToc` 的錨點選擇器為 `[data-doc-heading], .markdown h2, .markdown h3`）、`DetailIdentity`／`DetailTreeLabel`／`DetailStatus`、`DocHeader`（標題、籤槽、同列右端動作、出處行）、`DocTiles`、`DocBody`（15px／1.75、`[&_.markdown]:max-w-none`）、`DocSummary`；共用零件 `DocParts.tsx`（`SECTION_HEADING_CLS`、`Lead`、`InlineCode`、`MoreToggle`）、提示框 `Callout`、需求段 `RequirementBlock`（非 delta 畫法：`h2[data-spec-heading]` 15px、需求文、Scenario 左線區塊；delta 畫法：色條、收合、WHEN／THEN 兩欄）、`parseSpecSections`（purpose／intro／requirements（title、body、scenarios、trace）／appendix）、`parseTraceSources`。
- 規格詳情頁現況：`SpecDetailPage.tsx` 用 `DetailPageFrame`＋`DetailHeader`（標題列右端放 `SearchField`、meta 出處列）＋`Tabs variant="card"`（「規格 N」「溯源 M」，`TabsList actions` 放「開啟 ▾」）＋`DetailContentCard`；需求目錄是絕對定位在 768px 欄左側空白處的 `ManualToc`（`headings="[data-spec-heading]"`）；內文容器加了 `[&_.markdown]:text-[15px]`；溯源列用 `ListRow`。這三個標頭零件與 Markdown 的 `READING_COLUMN_CLS` 只剩它在用。
- 手冊頁現況：`ManualPage.tsx` 三欄——`ManualTree`（240px 淡灰底、`h-8` 列、stale 列尾小字「可能過期」、底部未入冊計數）、中欄 `header[data-manual-header]`（24px 標題＋「產生於」）固定在捲動區外、`div[data-manual-body]` 捲動區內一張 `Card` 放 `Markdown`、`footer[data-manual-footer]` 固定底列（出處籤＋上一頁／下一頁）、`ManualToc`（預設錨點 `.markdown h2, .markdown h3`，`bodyRef` 指向捲動區）。索引 `ManualPageItem { slug, title, section, order, keywords, sources, generated, stale }` 由 `speclink-desktop-core` 的 `manual.rs` 推導（過期判定在 Rust）。
- 文件動作：`DocumentRef` 四種（change artifact、discussion、spec、archived）；`apps/desktop/src/adapter/fsActions.ts` 的 `locate` 依種類 invoke `document_location`／`discussion_location`／`spec_location`／`archived_document_location`；Rust 在 `apps/desktop/core/src/query.rs`（`spec_location_at` 等，`DocumentLocation::within`）與 `discussions.rs`，Tauri 殼 `apps/desktop/src-tauri/src/lib.rs` 每個 command 單行委派並在 `generate_handler!` 註冊；`DocumentActionsMenu`（apps/desktop）展開時才查位置、找不到項目停用。
- 引擎常數：`speclink_core::lifecycle::model::PURPOSE_TBD_PREFIX = "TBD - created by archiving"`（`query.rs` 以它算 `purposeTbd`）。
- 原型：ContentSpecDetail、ContentManual 兩張畫板（資料取自 tray-status-menu 與 openspec/manual/tray.md）已由使用者確認；本設計照畫板落地。
- 約束：原始規格與手冊 Markdown 不動；只用既有原語與 token class；不與 cut 3 `desktop-discussion-doc-views` 同名改需求（本變更不碰「詳情頁的文件樹、閱讀卡與本頁目錄」）；桌面邏輯歸 `speclink-desktop-core`、Tauri 殼只委派。

## Goals / Non-Goals

**Goals:**

- 規格詳情頁與其他三種詳情頁同一套骨架：左欄換文件、右欄跳段落、閱讀卡標題下有數字磚與提示框。
- 手冊頁閱讀卡與詳情頁閱讀卡同款：標題區（籤槽、「開啟 ▾」）、出處行、數字磚、提示框、15px 內文、16px 章節；三欄與固定底列不動。
- 數字磚、出處行、提示框全由現有結構（需求段、`@trace`、frontmatter、索引 `stale`）推出；缺資料就少一格、沒有條件就沒有提示框。
- 手冊頁的檔案定位走與其他文件同一條路（`DocumentRef` → core 解析），Rust 零新契約型別。

**Non-Goals:**

- 不改規格頁清單、已封存清單、手冊側欄的搜尋與閱讀序、固定底列與上一頁／下一頁。
- 不改「詳情頁的文件樹、閱讀卡與本頁目錄」需求與 `DetailShell` 本身。
- 不在前端重判手冊頁的過期；不擴充 `ManualPageItem`；不做 remote 手冊。
- 不改 Markdown 的 96ch 預設與 GitHub Alert；不碰 server-web、CLI、`--json`。

## Decisions

### D1 規格詳情頁改用 DetailShell：左欄身分區、文件節點、狀態列

`SpecDetailPage.tsx` 改為 `Tabs orientation="vertical" activationMode="manual"`＋`DetailShell data-spec-detail`（換目標仍以 `key={capability}` 重掛、世代重載與 latest-wins 不變）。左欄 `tree`：

- `DetailIdentity`：`title={capability}`（等寬＋複製，`copyLabel` 沿用 `common.copyName`）；`badges` 為 `StatusPill tone="neutral"`「正式規格」；`provenance` 為「N 條需求 · M 個 scenario」（`sdetail.metaRequirements`、新鍵 `sdetail.metaScenarios`）與「最近更新 日期」（`sdetail.stateUpdated` 加日期；無 `@trace` 時缺席）；`relations` 一列 `mark="trace"`、標籤「溯源」：首顆來源變更籤（`Badge variant="outline"` 的 button）＋其餘數「+N」（`Badge variant="secondary"`），點擊都切到溯源文件（`setTab("trace")`）；來源為零時該列缺席。
- `DetailTreeLabel`「文件」＋`TabsList variant="tree"`：`規格`（`FileText`，徽章需求數）、`溯源`（`History`，徽章來源變更數）；徽章只在載入完成後出（既有規則）。
- `DetailTreeLabel`「狀態」＋`DetailStatus`：`StatusRow mark="requirements"`「需求」N、`mark="sources"`「溯源」M、`mark="updated"`「最近更新」日期（變更名灰字；無 `@trace` 時該列缺席）。

主區標題列沒有變更層動作（規格詳情頁不渲染 `DetailActions`；‹ n／m › 由宿主既有接線承擔）。`DetailHeader`、`DetailPageFrame`、`DetailContentCard`、絕對定位的需求目錄與 `data-spec-toc` 全部移除。

### D2 規格閱讀卡：標題區、搜尋需求、出處行、Purpose 引言、需求段收合

`TabsContent value="spec"` 內為一張 `Card`：

- `DocHeader`：`title` 取 `sdetail.tabSpec`「規格」；`badges` 為需求數 `Badge variant="secondary"`（載入後）；`actions` 為一列 `SearchField`（`sdetail.searchRequirements`，受控 `query`，規則不變：大小寫不敏感子字串比對需求標題、只呈現命中段、Purpose 恆在、不持久化）＋`renderDocumentActions?.({ doc: { spec: capability }, content: doc ?? null })`；`provenance` 為等寬 `openspec/specs/<capability>/spec.md`、「正式規格」（新鍵 `sdetail.canonical`）、「最近更新 日期（變更）」（既有 `sdetail.metaUpdated`；無 `@trace` 時缺席）。出處行的路徑為契約預設版面；spec 目錄名依專案設定改過時，「開啟 ▾」首列的解析路徑才是實際位置（既有規則，不在此重解）。
- `DocTiles`：D4 的規格磚。
- 提示框槽 `div[data-doc-callout]`：D4 的兩種提示框（可同時出現，Purpose 待補在前）。
- `DocBody ref={bodyRef}`：`section[data-purpose]` 內 `h2[data-doc-heading]`「Purpose」（`SECTION_HEADING_CLS`）＋`Lead`（灰底）包 `Markdown content={purpose}`；`section[data-spec-intro]`（有引言時）；命中的需求段逐條 `RequirementBlock`（`anchor` 沿用 `TraceAnchor`；`collapse={{ defaultOpen: i === 0 }}`，`i` 為命中序——搜尋過濾後第一條命中展開）；無命中時 `sdetail.noMatch`；`section[data-spec-appendix]`。無需求標題的規格整篇 `Markdown`、無本頁目錄錨點（既有規則）。載入中 `DocSkeleton`、缺席空態（既有三態）。

本頁目錄由 `DetailShell` 的右欄承接（`tocKey` 為 `${tab}\u0000${doc}\u0000${needle}`）：錨點為「Purpose」與各命中需求的標題，搜尋過濾後只剩命中段（自渲染後的 DOM 推導，不另做過濾）；引言與附錄裡的 Markdown h2／h3 也成為錨點（它們是「需求段之外的區段」）。

### D3 RequirementBlock 的收合模式（非 delta）

`RequirementBlockProps` 新增 `collapse?: { defaultOpen: boolean }`。非 delta 畫法下：標題元素改為 `h2[data-doc-heading]`（`SECTION_HEADING_CLS` 16px 粗體；`data-spec-heading` 退場，規格詳情頁的本頁目錄改走 `DetailShell` 的錨點）；有 `collapse` 時標題列成為 `button[aria-expanded]`（「Requirement」小標、標題、右端 `anchor`；收合時另顯示 Scenario 數籤 `doc.scenarioCount`），展開後為既有的需求文（SHALL 關鍵字等寬灰底）與 Scenario 左線區塊（不做 WHEN／THEN 兩欄、不做「還有 N」收合——規格詳情頁沿用原畫法）；無 `collapse` 時與現在相同（恆展開、無按鈕）。delta 畫法不變。`anchor` 放在 button 內時 `TraceAnchor` 的可點版本以 `stopPropagation` 避免同時切換收合。

### D4 數字磚、提示框與溯源磚的推導：specSummary.ts

新檔 `packages/ui/src/specSummary.ts`（純函式）：

- `specStats(sections: SpecSections | null, sources: { name: string; archived?: ArchivedItem }[]): SpecStats`，`SpecStats { requirements: number; scenarios: number; sourceCount: number; latest: { source: string; updated: string } | null; earliest: { source: string; updated: string } | null; traced: number; untraced: string[]; purposeTbd: boolean }`——`scenarios` 為各需求 `scenarios.length` 加總；`latest`／`earliest` 自各需求段 `trace.updated` 取最大／最小（先比前 10 字的日曆日、再比全文，純日期與 RFC 3339 混用時以日曆日為準）；`traced` 為有 `trace` 的需求數、`untraced` 為沒有 `trace` 的需求標題；`purposeTbd` 為 `sections.purpose` 去頭尾空白後以 `PURPOSE_TBD_PREFIX` 起頭。
- `PURPOSE_TBD_PREFIX` 匯出自 `packages/ui/src/specSections.ts`，值為 `"TBD - created by archiving"`；`specSections.test.ts` 新增跨語言對齊題，讀 `crates/engine/speclink-core/src/lifecycle/model.rs` 的同名常數逐字比對（與既有 `STRUCTURAL_HEADERS` 對齊題同一做法）。
- `specTiles(stats, t): DocTile[]`——「需求」（`requirements`）、「Scenario」（`scenarios`）、「溯源變更」（`sourceCount`）、「最近更新 · <變更>」（值為 `latest.updated` 的日曆日；`latest` 為 null 時缺席）。`sections` 為 null 時空陣列。
- `traceTiles(stats, t): DocTile[]`——「來源變更」、「最早 · <變更>」（`earliest` 日曆日）、「最近 · <變更>」、「有來源的需求」（`traced ／ requirements`）；`sourceCount` 為 0 時空陣列。
- 提示框由頁依 `stats` 組：`purposeTbd` 時 `Callout tone="warning" data-callout="purpose-tbd"` 標題 `specs.purposeTbd`「Purpose 待補」、內文「這份規格的 Purpose 還是封存流程的佔位文字；直接編輯 `openspec/specs/<capability>/spec.md` 的 Purpose 段即可，不走 delta」（`InlineCode` 包路徑）；`untraced.length > 0` 時 `Callout tone="note" data-callout="untraced"` 標題「N 條需求沒有來源變更」、內文列出名稱（頓號相連）與「它們沒有 @trace 註解，溯源文件與「溯源變更」磚都不含它們」。

### D5 溯源文件

`TabsContent value="trace"`：`DocHeader title="溯源"`＋`badges` 為來源變更數籤、無 `actions`（推導文件不放「開啟 ▾」，與排程同規則）；`provenance` 為「自全文的 @trace 註解推導，不是檔案」（`sdetail.traceDerived`）與「封存清單命中者在前、依封存日期升冪」（`sdetail.traceOrder`）；`DocTiles` 為 D4 的溯源磚；內文為既有 `ListRow` 列（排序、可點／不可點、副標、空態 `sdetail.noSources` 全部不變）。

### D6 手冊閱讀卡：標題區、出處行、數字磚、提示框、字級

`ManualPage.tsx`：移除 `header[data-manual-header]`；捲動區 `div[data-manual-body]` 內的 `Card` 改為由上而下——

- `DocHeader`：`title={heading}`（內文首個非空行為 `# 標題` 時取之且內文不重複，否則索引 `title`——既有規則）；`badges` 為分區籤（`Badge variant="secondary"`，`section` 為 null 時「其他」）與 `stale` 時 `StatusPill tone="warning" data-manual-stale-pill`「可能過期」；`actions` 為 `renderDocumentActions?.({ doc: { manual: current.slug }, content: loaded?.body ?? null })`（宿主未傳時缺席）；`provenance` 為等寬 `openspec/manual/<slug>.md`、`generated` 時「產生於 {generated}」（既有 `manual.generatedAt`，`data-manual-generated` 搬到這裡）、「出處」＋出處籤（名單、去重、可點規則與底列相同：正典存在者為可點籤呼叫 `onOpenSpec`，不存在者純文字）。
- `DocTiles`：「章節」（內文 h2 數）、「小節」（h3 數）、「關鍵字」（`keywords.length`）、「出處規格」（去重後的 sources 數）；內文未載入或載入失敗時整列缺席。計數用 `manualDoc.ts` 新增的 `countHeadings(md): { h2: number; h3: number }`（圍欄外的 ATX 標題行；setext 不算）。
- 提示框槽：`stale` 時 `Callout tone="warning" data-callout="stale"` 標題「可能過期」，內文每個出處規格一行——頁以新 prop `loadSpec?: (capability: string) => Promise<string | null>` 讀該規格全文、`parseSpecSections` 後取各需求 `trace.updated` 最大者與其 `source`，寫「出處 <cap> 於 <日曆日> 由變更 <source> 更新」；`loadSpec` 缺席、讀取失敗或規格無 `@trace` 時寫「出處 <cap> 在本頁產生後更新過」；最後一行「本頁產生於 <generated>」與「用 `/speclink-manual` 重生這一頁」（`InlineCode`）。讀取為 latest-wins、換頁作廢；過期與否只看索引 `stale`，不在前端重判。
- `DocBody ref` 不另設（本頁目錄仍以捲動區為 `bodyRef`），內文容器改用 `DocBody`（15px／1.75）並在外層加 `[&_.markdown_h2]:text-base [&_.markdown_h2]:font-semibold [&_.markdown_h3]:text-sm [&_.markdown_h3]:font-semibold`（Streamdown 的標題尺寸為 utilities 層，後代選擇器特異性足夠）。`DocHeader` 的標題是 `h2` 但不在 `.markdown` 內、也無 `data-doc-heading`，不進本頁目錄。
- 底列、`ManualToc`、跨頁連結攔截、換頁回頂、世代重載、三態、空狀態全部不變。

### D7 手冊目錄樹的節點款式與「過期」籤

`ManualTree.tsx` 的頁列改為文件樹同款：`h-[30px]`、`rounded-lg px-2.5 text-[13px]`、hover `bg-foreground/5`、作用中 `bg-primary/12 font-medium text-primary`（與 `TabsList variant="tree"` 的 trigger 同一組 class，仍是 `button[data-manual-page]`、`aria-current`）；stale 列尾改為籤 `span[data-manual-stale]`「過期」（新鍵 `manual.staleShort`；`rounded-full px-1.5 text-[10px] font-medium` 加 `SEMANTIC_SURFACE.warning`／`SEMANTIC_TONE.warning`）。搜尋列、分區標題、底部未入冊提示不變。

### D8 手冊頁的文件定位：DocumentRef、locate 與 Rust

- `DetailHeader.tsx` 的 `DocumentRef` 加第五種 `{ manual: string }`（slug）；`documentArtifact` 不變（手冊無 artifact，回 null）。
- `apps/desktop/src/adapter/fsActions.ts` 的 `locate`：`"manual" in doc` 時 `invoke("manual_location", { root, slug: doc.manual })`。
- `apps/desktop/core/src/manual.rs` 新增 `pub fn manual_location_at(root: &Path, slug: &str) -> Option<DocumentLocation>`：slug 守門與 `manual_page_at` 同一條（`is_safe_path_param` 且不含 `/`、`\`），`init_core_context(root)?`，路徑為 `ctx.workspace.spec_dir().join("manual").join(format!("{slug}.md"))`，回 `DocumentLocation::within(&ctx.workspace.root, path)`；不檢查檔案存在（缺檔由 opener 回報，與其他 location 同）。
- `apps/desktop/src-tauri/src/lib.rs` 新增 `#[tauri::command] async fn manual_location(root: PathBuf, slug: String)`，`spawn_blocking` 單行委派 `speclink_desktop_core::manual::manual_location_at`，於 `generate_handler!` 的 `spec_location` 之後註冊。
- `DocumentActionsMenu`、store 的 `locateDocument`／`openDocumentDefault`／`revealDocument`／`copyDocumentPath` 不變：remote 無 checkout 時只剩「複製內容」、「以預設 App 開啟」只開 markdown、失敗 toast——全部沿既有規則。

### D9 宿主接線

`apps/desktop/src/App.tsx`：`ManualPage` 加 `renderDocumentActions={renderDocumentActions}` 與 `loadSpec={(capability) => dataSource?.getSpecDocument(capability) ?? Promise.resolve(null)}`；`SpecDetailPage` 的 props 不變。store 不新增動作。

### D10 退場與匯出

`DetailHeader.tsx` 刪 `DetailHeader`、`DetailHeaderProps`、`DetailPageFrame`、`DetailContentCard`（`STAGE_PILL_TONE`、`DocumentRef`、`documentArtifact`、`DocumentTarget`、`displayName`、`IdentityName`、`DotSeparated`、`StatusRow` 留下）；`Markdown.tsx` 刪 `READING_COLUMN_CLS`；`index.ts` 對應刪匯出、新增 `specStats`／`specTiles`／`traceTiles`／`PURPOSE_TBD_PREFIX`／`countHeadings`。

### D11 i18n 詞條

`packages/ui/src/i18n.tsx` 兩語系同鍵集，新增：`sdetail.canonical`、`sdetail.metaScenarios`、`sdetail.relationTrace`、`sdetail.stateRequirements`、`sdetail.stateSources`、`sdetail.stateUpdated`、`sdetail.tileRequirements`、`sdetail.tileScenarios`、`sdetail.tileSources`、`sdetail.tileUpdated`、`sdetail.tileSourceCount`、`sdetail.tileEarliest`、`sdetail.tileLatest`、`sdetail.tileTraced`、`sdetail.traceDerived`、`sdetail.traceOrder`、`sdetail.purposeTbdBody`、`sdetail.untraced`、`sdetail.untracedBody`；`manual.tileSections`、`manual.tileSubsections`、`manual.tileKeywords`、`manual.tileSources`、`manual.staleShort`、`manual.staleCalloutSource`、`manual.staleCalloutSourceUnknown`、`manual.staleCalloutGenerated`、`manual.staleCalloutHint`。既有 `sdetail.*`、`manual.*`、`specs.purposeTbd`、`doc.scenarioCount` 沿用；`sdetail.tabSpec`／`sdetail.tabTrace` 改作文件節點與閱讀卡標題。

## Implementation Contract

- **行為**：自規格頁、手冊出處或變更文件的能力籤開啟任一正式規格，主區為三欄骨架——左欄身分區（能力名、「正式規格」籤、需求與 scenario 數、最近更新、「溯源」關聯列）、「文件」節點規格／溯源、「狀態」三列；閱讀卡「規格」標題列有需求數籤、「搜尋需求」與「開啟 ▾」，出處行、四格數字磚、條件式提示框、Purpose 引言段、需求段（第一條展開、其餘一列可展開）；右欄本頁目錄列 Purpose 與需求標題並隨搜尋縮減；「溯源」文件有出處行、四格磚與既有列規則。手冊頁的閱讀卡標題區含頁標題、分區籤、「可能過期」籤、「開啟 ▾」，出處行含檔案路徑、產生時間與可點出處籤，四格數字磚，過期時 warning 提示框列各出處規格的最近 `@trace` 與來源變更；內文 15px、章節 16px；左樹節點 30px 列、過期頁列尾「過期」籤；「開啟 ▾」能以預設 App 開啟、在 Finder 顯示、複製 `openspec/manual/<slug>.md` 的路徑。
- **介面與資料形狀**：`RequirementBlock` 的 `collapse` prop（D3）；`specStats`／`specTiles`／`traceTiles`、`SpecStats`、`PURPOSE_TBD_PREFIX`（D4）；`countHeadings`（D6）；`ManualPage` 新 props `renderDocumentActions?`、`loadSpec?`（D6）；`DocumentRef` 的 `{ manual: string }`（D8）；Tauri command `manual_location(root, slug) -> Option<DocumentLocation>` 與 core `manual_location_at`（D8）。DOM 錨點：`[data-spec-detail]`、`[data-relation-trace]`、`[data-purpose]`、`[data-doc-heading]`、`[data-requirement]`、`[data-trace-anchor]`、`[data-trace-row]`、`[data-doc-callout]`、`[data-callout="purpose-tbd"|"untraced"|"stale"]`、`[data-doc-tiles]`、`[data-doc-provenance]`、`[data-manual-stale-pill]`、`[data-manual-stale]`、`[data-manual-generated]`、`[data-manual-sources]`、`[data-manual-footer]`。
- **失敗模式**：規格文件缺席→空態（既有）；無 `### Requirement:`→整篇 Markdown、無磚、無提示框；無 `@trace`→最近更新段、溯源關聯列、最近更新磚與狀態列缺席、溯源文件空態；手冊內文載入失敗→失敗文案、無磚、無提示框、底列照常；`loadSpec` 失敗或規格無 `@trace`→提示框該行改「在本頁產生後更新過」；`manual_location` 收到含路徑分隔或 `..` 的 slug→`None`、選單項停用；remote 無 checkout→只剩「複製內容」。渲染不寫任何檔案。
- **驗收**：`npm test -w packages/ui -- specDetailPage specSummary specSections deltaSpecView manualPage i18n components detailShell`、`npm test -w apps/desktop -- App fsActions documentActionsMenu`、`cargo test -p speclink-desktop-core manual`、`npm run build -w apps/desktop`、`speclink validate desktop-spec-manual-pages` 全綠；`grep -rn "DetailPageFrame\|DetailContentCard\|READING_COLUMN_CLS\|data-spec-heading\|data-spec-toc\|data-manual-header" packages/ui/src apps/desktop/src` 零命中；手動開 tray-status-menu 的規格詳情頁與手冊〈系統匣選單〉各看一次。
- **範圍邊界**：in——`SpecDetailPage`、`RequirementBlock` 收合模式、`specSummary.ts`、`ManualPage`、`ManualTree`、`manualDoc.ts`、`DocumentRef`、`fsActions.locate`、core `manual_location_at`、Tauri `manual_location`、文案、匯出退場、宿主 props；out——`DetailShell`、規格頁與已封存頁清單、手冊索引與過期判定、底列、Markdown 預設、server-web、CLI。

## Risks / Trade-offs

- [`RequirementBlock` 的標題元素與錨點屬性改動波及 delta 檢視與既有測試] → delta 畫法不改、只動非 delta 分支；`deltaSpecView.test.tsx` 既有題目不改全綠為守門；`specDetailPage.test.tsx` 改以 `[data-doc-heading]` 斷言。
- [引言與附錄的 Markdown h2／h3 也進本頁目錄] → 這是「需求段之外的區段」，列入目錄合理；規格全文 h2 只有 Purpose／Requirements 層級、需求為 h3（已切成需求段），實務上不會重複。
- [出處行寫死 `openspec/specs/<capability>/spec.md`，專案改過 spec 目錄名時字面不符] → 「開啟 ▾」首列才是解析後的實際路徑（既有規則）；出處行只是契約預設版面的提示，與變更文件的 `proposal.md` 同為相對提示。
- [手冊過期提示框多讀 N 份規格全文] → 只在 `stale` 為 true 時讀、latest-wins、換頁作廢；讀不到就退回不帶日期的句子。
- [Streamdown 標題尺寸覆蓋失效] → 以任意變體後代選擇器覆蓋（與 `DocBody` 覆蓋 `.markdown` 字級同手法）；`manualPage.test.tsx` 斷言容器 class 存在。
- [Rust 新 command 漏註冊] → `cargo test -p speclink-desktop-core` 測 `manual_location_at`；`fsActions.test.ts` 斷言 invoke 名稱；手動驗收點「在 Finder 顯示」。
- [回歸對照] → 前端 vitest 與 `speclink-desktop-core` 單元測試；不動 golden 與 CLI 測試；跨平台只涉及路徑組裝（`spec_dir().join`，與其他 location 同）。

## Migration Plan

1. D4 純函式（先寫紅燈）→ D3 `RequirementBlock` 收合模式 → D1／D2／D5 規格詳情頁 → D6／D7 手冊頁與目錄樹 → D8 `DocumentRef`、`fsActions`、core 與 Tauri → D9 宿主接線 → D10／D11 退場、匯出與文案。
2. 無資料遷移、無設定變更；手冊與規格檔案不動。整變更 revert 即回到 cut 2 封存後的狀態。

## Open Questions

- 無——討論期擱置的「搜尋需求是否同時過濾右欄目錄」在本設計下自然成立（目錄自渲染後的 DOM 推導）；「節點『過期』籤的英文文案」定為 `Outdated`。
