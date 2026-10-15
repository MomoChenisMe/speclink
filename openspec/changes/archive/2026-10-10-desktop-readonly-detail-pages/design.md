## Context

`desktop-detail-page` 落地後，主區的頁層（宿主頁保留掛載並 `inert`、詳情頁覆蓋）、麵包屑三段、`DetailNav` ‹ n／m ›、快捷鍵、`DocumentActionsMenu`、`DetailHeader`、`StatusPill` 與 `closeDetailPage` 都已存在，`detailSpec`／`detailArchived` 兩個欄位仍由 `SpecDrawer`（174 行）與 `ArchivedDrawer`（412 行）承接。規格詳情目前把 spec.md 整篇 markdown 渲染並在標頭以「來自」籤＋「+N」浮層列溯源變更；已封存詳情在同一面板內切換封存變更與封存討論兩種檢視。cut 3a 的 `SpecList`／`ArchivedList` 自持搜尋與換頁；`ManualToc` 依內文 DOM 的 h2／h3 推導錨點列並隨捲動高亮。

## Goals / Non-Goals

**Goals**

- 規格詳情頁：需求分段、來源錨點、關鍵字標示、Scenario 縮排、需求目錄、溯源分頁、搜尋需求。
- 已封存詳情頁：封存變更與封存討論兩個變體，與變更／討論詳情頁同骨架、唯讀。
- 兩者接上頁層與 ‹ ›；宿主頁（規格頁、已封存頁、手冊頁）維持不變。
- `SpecDrawer`、`ArchivedDrawer` 刪除；桌面不再引用 Sheet。

**Non-Goals**

- 頁層、麵包屑、快捷鍵、「開啟 ▾」選單本身（前半 change）。
- 規格頁與已封存頁的清單版面。
- 已封存討論的檔案層動作（檔名帶封存日期而清單資料不帶）。

## Decisions

### D1 規格解析：`specSections.ts`

`packages/ui/src/specSections.ts` 匯出 `parseSpecSections(markdown): SpecSections | null`：

- 以 `### Requirement:` 行切段；第一段之前的內容取 `## Purpose` 區段（缺席為空字串）。
- 每段 `{ title, body, scenarios: { title, body }[], trace: { source, updated } | null }`：`body` 為標題行之後到第一個 `#### Scenario:` 之前的 markdown；每個 `#### Scenario:` 到下一個 `####`／段尾為一個 scenario（`##### Example` 歸屬所在 scenario 的 body）；段尾的 `<!-- @trace … -->` 註解解析 `source` 與 `updated`（取日期部分）後自 body 移除；`---` 分隔線移除。
- 不屬 Purpose 與需求段的內容不丟：`# ` 標題與 `## Requirements` 標題行之外，第一段之前的收進 `intro`、之後的（需求之間或之後的其他 `## ` 區段）收進 `appendix`，頁把兩者照原文排在 Purpose 之後與全部需求段之後。
- 圍欄判定：開頭為 3 個以上 `` ` `` 或 `~`；收尾圍欄須同字元、不短於開頭且其後只有空白——較短或另一種字元的圍欄行是內容，巢狀圍欄內的 `### Requirement:` 不切段。
- 沒有任何 `### Requirement:` 行時回 null——頁以單一 markdown 檢視退回（與討論記錄切分失敗同一路徑）。
- 關鍵字標示不改來源：渲染時以 rehype 外掛 `rehypeSpecKeywords`（同檔匯出）把文字節點中的 `SHALL NOT`／`SHALL`／`MUST NOT`／`MUST`（整詞）包成 `<code class="spec-kw">`，只套在需求文與 scenario body 的 Markdown 實例上（`Markdown` 加 `specKeywords` 布林，模組層常數陣列把外掛接在 sanitize 之後，`spec-kw` class 不被剝）；程式碼區塊內不處理。灰底由 `apps/desktop/src/index.css` 的 `.markdown code.spec-kw` 覆寫 teal code chip。

### D2 規格詳情頁（`SpecDetailPage`）

props：`{ capability, refreshGen?, loadDocument, archivedChanges?, onOpenArchivedChange?, renderDocumentActions? }`（meta 列全由載入後的全文推導，不另收清單的修改時間與需求數）。版面（原型「規格詳情頁」畫板）：

- `DetailHeader`：等寬 22px capability 名＋`CopyButton`；出身列改為 meta 列（灰字）：「N 條需求 · 來自 M 個已封存變更 · 最近更新 YYYY-MM-DD（<變更>）」——N 取解析後段數、M 取 @trace source 去重數、最近更新取全部 trace 中最大 `updated` 與其 source；段缺席時整段缺席。右端動作區為「搜尋需求」`SearchField`（共用全圓搜尋框，280px）。
- 內容卡 `Tabs variant="card"`：「規格 N」「溯源 M」；`TabsList actions` 放 `renderDocumentActions({ doc: { spec: capability }, content: 原文 })`（位置由宿主經 core 解析，預設佈局為 `openspec/specs/<capability>/spec.md`；見 D7）。
- 需求目錄：頁根為 `@container`（量的是詳情頁層的寬度，不是 768px 內容欄——與原型「詳情頁寬於 1200px 時顯示」一致）；「規格」分頁作用中且頁寬 ≥1200px 時（`hidden @min-[1200px]:block`），768px 內容欄左側空白處以 sticky 呈現 200px 需求目錄——重用 `ManualToc`（`bodyRef` 指向規格內文；`ManualToc` 加 `className` 槽與標題選擇器參數，捲動改以 capture 監聽包住內文的捲動容器——規格頁的捲動容器是 App 的頁層而非內文本身；Purpose 與每條需求標題渲染為 `h2`，目錄即列出 Purpose 與全部需求，點擊捲至該段、捲動高亮目前段）。
- 「規格」分頁內文欄：Purpose 段（`h2`「Purpose」＋Markdown）、`intro`（有內容時），其後每條需求一段 `<section data-requirement>`：首列「Requirement」等寬小標（`spec-kw` 同款）＋`h2` 標題＋右端 `trace` 錨點「來自 <source> · <updated>」（有 trace 時；source 在封存清單命中時為可點籤、點擊 `onOpenArchivedChange(datedName)`）；需求文 Markdown（15px 行高 1.75）；每個 scenario 為 `border-l-2 border-border pl-4` 區塊：12px 粗體灰字「Scenario：<title>」＋Markdown。全部需求段之後接 `appendix`（有內容時）。段與段間 24px；`parseSpecSections` 回 null 時整篇以 Markdown 渲染、無目錄。
- 「搜尋需求」：以大小寫不敏感子字串比對需求標題，只渲染命中的段（Purpose 段恆在）；目錄因 DOM 變化自動只列命中段；無命中時卡內顯示「沒有符合的需求」；清空還原。字串不持久化。
- 「溯源」分頁：一張列式清單（`ListRow`，加 `disabled` 態：灰字、無 hover、無 ›、點擊與按鍵不觸發）——每個 @trace source 一列：等寬變更名、副標為封存日期（封存清單命中）或「無封存記錄」（未命中，`disabled`）；排序：命中者依封存日期升冪在前、未命中者在後依文件首次出現序；點命中列 `onOpenArchivedChange(datedName)`；無任何 source 時分頁徽章為 0、卡內顯示「此規格沒有來源變更」。
- 載入三態沿「詳情頁文件載入以 skeleton 呈現」；`refreshGen` 遞增就地重載（latest-wins）。換目標以 key 重掛（清空全量載入、搜尋字串與分頁回到初始），載入用與 D3 同一個 `useGenerationLoad`／`settle`（`packages/ui/src/lib/useGenerationLoad.ts`）。

### D3 已封存詳情頁（`ArchivedDetailPage`）

props 同 `ArchivedDrawer` 去掉 `open`／`onOpenChange`、加 `renderDocumentActions` 與 `discussionTopic`（封存討論的 topic 行），`target` 判別變體（各以 key 重掛，換目標即清空重載）；`DetailHeader` 加 `copyValue`（封存變更以變更名為題、複製含日期前綴的封存目錄名）：

- **封存變更**：`DetailHeader` 標題為變更名（等寬 22px）、`CopyButton` 複製含日期前綴的封存目錄名、章籤以 `StatusPill` 呈現審查／驗證結局（已審查 stamp、曾審查未通過 destructive；驗證同）；出身列：建立者頭像＋名字（email 收提示）、建立日、「封存 YYYY-MM-DD」、「來自」＋來源討論籤（點擊 `onOpenDiscussion(slug)`——宿主改以 `openArchived({ kind: "discussion", slug })` 開已封存討論頁，記錄 live 優先、封存後備）；無動作區。「狀態」卡三列：任務 `n/m`（無 tasks.md 時「—」）、品質關卡（結局籤；皆 none 時「未經品質關卡」）、封存日期。內容卡分頁 提案／設計／任務／規格＋條件式 審查（`reviewedNotPassed`）／驗證（`verifiedNotPassed`）；`renderDocumentActions({ doc: { archived: datedName, artifact }, content: 原文 })`（預設佈局為 `openspec/changes/archive/<datedName>/<artifact>`；規格分頁取清單第一份 capability 的 `specs/<cap>/spec.md`，與變更詳情頁同；見 D7）。任務分頁 `TaskList` 唯讀：無把手、無工具列、勾選框不可點、整列不回應。
- **封存討論**：標題為 slug＋`CopyButton`＋「已封存」`StatusPill neutral`＋改進籤；topic 一行；出身列：建立者、建立日、「封存 YYYY-MM-DD」——封存討論的清單項不帶封存日期（日期只在封存檔名），宿主不傳 `archivedDate`，該欄依「資料缺席時該欄缺席」恆缺席。「狀態」卡：階段列現站「已封存」（與討論詳情頁同一步驟列元件）、衍生變更列（`promotedChanges` 三態：已封存→副標封存日期可點、活躍→副標階段詞可點、已刪除→不可點「無封存記錄」；點擊 `onOpenPromotedChange`）；無 promoted_to 時列缺席。內容卡分頁 結論／討論過程／背景／衍生變更（結論與輪卡的欄位標籤渲染沿用；區段缺失以單一檢視退回）；`renderDocumentActions({ doc: null, content: 原文 })` → 選單只有「複製內容」（見 D7）。
- 寬度約束、skeleton、外部變更重載沿前半 change 的需求。載入以 `useGenerationLoad(gen, load)`（開啟全量、世代遞增就地重載、`isLatest` 丟棄過期回應）與 `settle`（首載失敗才落空態）收斂，與 D2 共用 `packages/ui/src/lib/useGenerationLoad.ts`。
- 與討論詳情頁共用（`DiscussionDetailPage.tsx` 匯出）：階段步驟列 `DiscussionStageRow`、預設分頁判定 `defaultDiscussionTab`、衍生變更分頁列 `PromotedChangeRow` 與狀態卡衍生籤 `PromotedChangeChip`（狀態標示由呼叫端給：討論詳情頁為階段籤、已封存討論頁為副標；`onOpen` 缺席＝不可開啟）。

### D4 宿主與序列

- `SpecList` 與 `ArchivedList` 各加 `onVisibleOrderChange`：`SpecList` 回報過濾後全部頁的 capability 順序；`ArchivedList` 回報 `{ changes: datedName[], discussions: slug[] }`（各自過濾後全部頁）。
- App 的 `visibleOrder` state 增加 `specs`、`archivedChanges`、`archivedDiscussions`；序列選擇：宿主為對應清單頁時取回報值，否則（手冊頁、規格詳情頁開已封存等）取 store 清單的未過濾順序（`s.specs` 依規格頁預設排序、`s.archived` 清單序、`s.discussions.archived` 清單序）。
- App 以模組層 `openDetailOf(state, visibleOrder)` 一處分派四種詳情頁：回 `{ kind, current, sequence, open }`（或 null），序列、麵包屑、‹ › 切換與頁的渲染都讀它，不各自寫巢狀判斷。
- 麵包屑：規格「專案名 / 規格 / capability」；已封存變更「專案名 / 已封存 / 變更名」；已封存討論「專案名 / 已封存 / slug」；自手冊頁開啟的規格詳情頁為「專案名 / 手冊 / capability」且中段點擊回手冊。
- `openSpec`／`openArchived` 不切宿主（既有）；`closeDetailPage` 共用。`SpecList` 的 `focus={s.detailSpec}` 保留（回規格頁時列已捲至該規格）。

### D5 Sheet 自桌面退場

刪除 `SpecDrawer.tsx`、`ArchivedDrawer.tsx` 與兩個測試檔；三個標示測試（審查／驗證／改進）中渲染 `ArchivedDrawer` 的案例改渲染 `ArchivedDetailPage`；因抽屜退場而無人引用的 `lib/useLingering.ts`、store 的 `closeSpec`／`closeArchived`（關頁統一走 `closeDetailPage`）一併移除；`packages/ui/src/index.ts` 不再匯出兩者；`Sheet*` 匯出保留（server-web 的 `DetailSheet`、`ConsoleLayout`、`Tour` 仍用）。驗證以 `grep` 確認 `apps/desktop/src` 無 `Sheet` 引用。

### D6 文案

i18n 新增 `sdetail.*`（`metaRequirements`、`metaSources`、`metaUpdated`、`tabSpec`、`tabTrace`、`searchRequirements`、`noMatch`、`noSources`、`noArchive`、`requirementLabel`、`scenarioLabel`）與 `adetail.*`（`archivedOn`、`statusArchived`、`noQuality`、`tasksNone`）；既有 `spec.*`／`archived.*` 鍵沿用，但因抽屜退場而無人引用的鍵（`rdrawer.fullScreen`、`rdrawer.restore`、`adrawer.promotedTo`、`archived.createdOn`、`archived.archivedOn`、`archived.sectionContext`、`archived.sectionRounds`、`archived.sectionConclusion`）自兩語系移除。兩語系鍵集合相等。

### D7 文件定位（「開啟 ▾」）

沿前半 change「頁不拼路徑、位置由宿主向 core 查」：spec 目錄名可設定，頁內寫死 `openspec/…` 會在自訂佈局的專案指錯位置。

- `DocumentRef`（`DetailHeader.tsx`）新增 `{ spec: capability }` 與 `{ archived: datedName; artifact }`；`DocumentTarget.doc` 可為 `null`＝沒有可定位的檔案、只帶原文。
- `apps/desktop/core/src/query.rs` 新增 `spec_location_at(root, capability)` 與 `archived_document_location_at(root, datedName, artifact)`：以 store 的 `Layout` 解析（尊重 spec 目錄名）、路徑穿越參數回 `None`，與 `document_location_at` 同款回 `DocumentLocation`；`src-tauri/src/lib.rs` 加 `spec_location`／`archived_document_location` 兩支指令；`fsActions.ts` 的 `locate` 依 ref 型別分派。
- `DocumentActionsMenu`：`doc` 為 `null` 時不論本機與否只給「複製內容」。

### D8 返回（回宿主頁）

手動驗收發現：詳情頁只能靠 Esc 或麵包屑回清單，畫面上沒有返回的入口。

- 「回上一頁」＝回宿主清單頁，與 Esc、麵包屑中段同一個 `closeDetailPage`；不做瀏覽歷史堆疊——與「詳情頁互斥」模型一致，連續跳轉（規格詳情頁→已封存詳情頁）後返回直接回宿主。
- 觸發：麵包屑左側的 `DetailBack`（`apps/desktop/src/components/DetailNav.tsx`；`MainTitleBar` 加 `leading` 槽）；⌘[（macOS）／Alt+←（其他平台）併入既有的詳情頁快捷鍵處理，守門與 Esc 相同（焦點在文字輸入、已被浮層處理時不作用）；滑鼠側鍵以 window `mouseup` 的 `button === 3` 接——WebView 是否送出此事件依平台而異，以真機驗收為準。
- 確認框守門：封存、退回、刪除、討論封存的確認框開著（store 的 `pendingArchive`／`pendingRevert`／`pendingDelete`／`pendingArchiveDiscussion` 任一非 null）或確認在途時，切換與返回都不作用——確認框屬於目前詳情頁，關頁或換頁會連帶收掉它。Esc、返回快捷鍵、滑鼠側鍵與滑動共用 App 的 `backFromDetail`，與 ‹ › 的 `stepDetail` 同一個 `detailActionsBlocked` 判定；確認框開著時 `SwipeBack` 停用（指示不推出）。以 store 狀態判定而非 DOM 的 `role="alertdialog"`：更新日誌彈窗也是 AlertDialog，但不屬詳情頁，‹ › 照常換頁並收掉它。
- 切換主區頁面（`setBoardView`）一併清 `detailSpec`／`detailArchived`：抽屜時代這兩者被 Sheet 遮住專案欄而不需清，改成頁後專案欄可點，不清會留下「已封存 / desktop-app」這種宿主與內容對不上的麵包屑。連線失效橫幅的「重新登入」（`openConnectionReauth`）在主區外、詳情頁開著時仍點得到，也經 `setBoardView` 切到設定頁。
- i18n（desktop app 層）新增 `detail.back`（「返回{page}」）與 `detail.backHint`（「{label}（Esc 或 {keys}）」）。
- 返回快捷鍵以 `isBackShortcut(e, platform)` 判定（`DetailNav.tsx`）：macOS 以 `e.code`（`BracketLeft`／`ArrowLeft`）比對 ⌘[ 與 ⌘←——中文輸入法啟用時 `[` 鍵送進頁面的 `e.key` 不一定是「[」；其他平台 Alt+←。
- 觸控板滑動返回（`apps/desktop/src/components/SwipeBack.tsx`，掛在主區）：手冊驗收時使用者指的「上一頁」是觸控板兩指滑動並在左緣出現「←」指示（同 Codex app）。WKWebView 原生的前後頁滑動要在 Rust 端開啟且需詳情頁寫入瀏覽歷史，呈現是 Safari 的整頁滑動而非指示，不採用；改以 JS 監聽 `wheel`：水平分量為主時依 `webkitDirectionInvertedFromDevice`（自然捲動設定）換算手指方向、累計朝返回方向的位移（反向滑回會扣減），指示依累計／門檻推出、達門檻轉實心；DOM 的 wheel 事件不帶手指放開的相位，以「最後一個事件後靜止 120ms」視為手勢結束（含慣性捲動的尾段），屆時已達門檻即 `closeDetailPage`。手勢開始於仍可沿 `deltaX` 方向捲動的水平捲動區塊（`overflow-x` 為 auto／scroll）時不接手——方向看 `deltaX` 本身而非手指方向：自然捲動關閉時手指向右是把區塊往右捲；只裁切的 `overflow: hidden`（截斷的長名稱）不算。
- 根層水平彈性回彈：macOS WebView 在水平無可捲時仍讓整頁隨手指平移再彈回，滑動返回時整個畫面跟著動。`apps/desktop/src/index.css` 對 `html, body` 設 `overscroll-behavior-x: none`；只放桌面——瀏覽器裡的 server-web 要保留瀏覽器自己的兩指滑動返回（Chrome 在根層關掉水平回彈會連帶停用它）。vitest 把 CSS 匯入一律換成空字串，此規則無法以單元測試讀取，以建置產物含該規則與真機驗收為準。

## Risks / Trade-offs

- **每條需求一個 Markdown 實例**：desktop-app 有 86 條需求、各帶數個 scenario，約 400 個 Streamdown 實例；首次渲染約數百毫秒。不做虛擬化（頁內搜尋與目錄需要完整 DOM）；若實測卡頓，退路是只在段進入可視區時渲染 scenario body。
- **`@container` 在 jsdom 不生效**：目錄的顯示條件只能斷言 class；版面在真機驗收。
- **溯源分頁取代「+N」浮層**：溯源資訊從標頭移到分頁，多一次點擊；換得全部來源一覽與封存日期排序可讀。
- **已封存討論無檔案動作**：清單資料沒有封存檔名；只給「複製內容」。

## Migration Plan

1. `specSections.ts` 與測試先行（純函式）。
2. `SpecDetailPage` 搬 `specDrawer.test.tsx` 的案例；`ArchivedDetailPage` 搬 `archivedDrawer.test.tsx` 的案例。
3. App 接線兩頁、清單回報順序；刪除兩個 Drawer；跑 grep 守門與全部測試。
4. 規格、手冊規格與文案對齊。

## Open Questions

- 無。
