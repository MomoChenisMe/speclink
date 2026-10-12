## Summary

討論詳情頁與已封存討論頁的四份文件（結論、討論過程、背景、衍生變更）換上各自的渲染器：結論拆成決定主句、分刀卡與否決列，討論過程成時間軸，背景長出關聯與記錄列，衍生變更依結論的分刀分組；每份文件帶數字磚、出處行與推得出的提示框。這是討論 detail-content-redesign 的第三刀、也是最後一刀（cut 3 `desktop-discussion-doc-views`）。

## Motivation

- 現況（cut 1 `desktop-detail-doc-tree` 與 cut 2 `desktop-change-doc-views` 都已於 2026-10-12 封存）：變更詳情頁的六份文件已有各自的渲染器、數字磚與提示框，討論側的四份文件還停在舊畫法——結論與討論過程是「20px 粗體大標籤＋整段 Markdown」（`LABEL_CLS`，DocParts 註明「cut 3 再退場」）、背景整篇 Markdown、衍生變更是一條平面列；閱讀卡的數字磚容器是空的（`DocTiles tiles={[]}`），提示框容器沒有內容，討論文件也沒有出處行（cut 1 的規格明寫「討論與已封存討論的文件 SHALL NOT 顯示出處行」，留給本刀推導）。
- 目標使用者：透過 AI 代理跑 SDD 的開發者、PO 與 PM，在桌面 app 回看一份討論「決定了什麼、分幾刀、每刀落到哪個變更、還剩哪一刀」；對應 `/speclink-discuss` 收斂後與 `/speclink-propose --from-discussion` 轉出期間的檢視。現在要從結論的整段文字裡自己找「cut 2」在哪、再回看板找那個變更。
- 為什麼是現在：三刀的分工是「骨架一刀、渲染器兩刀」，前兩刀落地後，討論側是最後一塊還沒對齊的面；原型十張畫板（討論側四張：ContentConclusion、ContentRounds、ContentContext、ContentPromoted，資料取自 desktop-ui-redesign）已在討論期由使用者逐面確認。
- 影響的 app：`apps/desktop`（宿主接線）與 `packages/ui`（渲染器、解析器、文案）；不碰 Rust 與 Tauri，不新增 `--json` 欄位，不改任何 CLI 指令與輸出——相容性影響為零。

## Proposed Solution

1. **解析器搬家並擴充**：把討論記錄的純函式解析（`splitDiscussionSections`、`splitRounds`、`splitLabeledFields`、`countPriorDiscussions`）從 `packages/ui/src/components/DiscussionDetailPage.tsx` 搬進新檔 `packages/ui/src/discussionModel.ts`（與變更側的 `docModel.ts` 對稱），再加上本刀要的形狀：結論六欄位的切分、決定的「主句＋定案細節」切分、分刀卡（決定細節中以 `cut N`／`刀` 粗體開頭的列）、否決與擱置的「方案／理由」切分（沿用 `splitAlternative`）、待決與否決的逐條切分、輪的 mode 與日期統計、背景的 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:` 四行解析、文中名稱比對（規格名、變更名、討論 slug）、下一步裡的 `/speclink-*` 指令擷取。形狀不符一律逐項降級（當一般列、退回平面清單），不拋錯；記錄格式本身不動。
2. **結論渲染器**（ConclusionDoc）：決定主句放大、定案細節細線列預設只展開前幾條、「展開其餘 N 項」；分刀每刀一張卡（cut 籤、範圍句、變更籤附階段籤可點開該變更、子項）；理由段；否決與擱置拆「方案／理由」列（擱置帶時鐘圖示）；記錄去向成籤（規格名可點開規格詳情頁）；下一步的 `/speclink-*` 指令為可複製籤。六欄位都缺席時整篇 Markdown 退回（既有規則）。
3. **討論過程渲染器**（RoundsDoc）：時間軸——每輪一個節點，mode 以色點區分（assumptions 主色、interview 取看板討論欄的色相）；每輪卡標題為焦點、欄位為立場（定論句＋列點）、否決（× 列）、待決（琥珀籤；「無」為收斂標記）；首輪與末輪預設展開、其餘收合為一列。非 scaffold 格式整篇 Markdown 退回（既有規則）。
4. **背景渲染器**（ContextDoc）：第一段放大為引言、其餘照 Markdown，文中比對得到的規格名、變更名、討論 slug 自動成可點籤（共用 Markdown 元件加一個接在 sanitize 之後的 rehype 外掛，與 SHALL 關鍵字同一條路）；「關聯」鍵值列（正典規格、在途變更、已封存變更、先前討論、來源文件）；「記錄」列（建立日與建立者、種類、檔案路徑可複製）。
5. **衍生變更渲染器**（PromotedDoc）：依結論分刀段的 cut 清單分組，讀不到就退回轉出順序的平面清單；每列階段籤、等寬名稱、任務進度、審查與驗證章、封存日或最近蓋章日、開啟鈕；討論詳情頁與已封存討論頁共用，已封存頁改由宿主傳入作用中與已封存變更清單推導同一份列資料。
6. **數字磚、出處行與提示框的推導單一入口**（discussionDocSummary，對稱 changeDocSummary）：結論四格（定案細節、分刀與變更數、否決、擱置）、討論過程四格（輪、天數、各 mode 輪數）、背景四格（命中的正典規格、在途變更、相關已封存變更、先前討論）、衍生變更四格（變更、已封存、作用中、任務 n/m）；缺資料少一格。提示框只有三處：討論過程的「討論已收斂」（結論已寫入）、背景的「沒有 grill 階段」（首輪 mode 為 assumptions）、衍生變更的「只差最後一刀」（已轉出、hold 已落、只剩一個作用中子變更）。出處行：結論「記錄路徑 · N 輪之後寫入」、討論過程「日期範圍 · 各 mode 輪數」、背景「記錄建立時寫入一次 · 建立日」、衍生變更「依結論的分刀分組」或「依轉出順序」；已封存討論頁不出記錄路徑段（清單資料不帶封存檔名）。
7. **頁與宿主接線**：討論詳情頁與已封存討論頁改掛四個渲染器與推導入口，提示框槽放在數字磚與內文之間；新增 props 接規格名清單、討論 slug 清單與開規格／開討論／開已封存變更的回呼，`apps/desktop/src/App.tsx` 由 store 既有動作接上；`LABEL_CLS` 與舊的 `ConclusionView`／`RoundsView` 退場。
8. **規格**：`desktop-app` 四條需求 MODIFIED——「討論結論以欄位標籤呈現」「討論輪以卡片呈現」（新畫法、數字磚、提示框，既有切分與退回規則原文保留）、「討論詳情頁檢視與轉出變更」（背景與衍生變更文件的內容）、「詳情頁的文件樹、閱讀卡與本頁目錄」（討論文件改為有推導的出處行、目錄錨點改列各文件的區段標題）。與平行變更 `desktop-spec-manual-pages`（規格詳情頁與手冊頁）無同名需求：那刀改「桌面 app 呈現 change 與 spec 的清單與內容」、兩條 markdown 需求與「詳情頁的文件動作」，本刀都不碰。

## Non-Goals

- 不改討論記錄的格式、scaffold 與 `speclink discuss` 技能；討論記錄不進 schema 模板；不新增 `discuss show --json` 的拆好 rounds（討論已擱置：等有第二個消費者）。
- 不改規格詳情頁與手冊頁（另一變更 `desktop-spec-manual-pages` 負責）。
- 不改左欄身分區、「文件」節點、「狀態」列、標題列動作與「開啟 ▾」規則（cut 1 已定，「詳情頁的文件動作」需求不動）。
- 不改變更側的六份文件（cut 2 已封存）、不改 `docModel.ts` 的既有函式簽名（只新增共用）。
- 不碰 Rust、Tauri 與 server；不改 CLI 與 `--json`。

## Alternatives Considered

- 引擎在 `discuss show --json` 回拆好的 rounds 與結論欄位——格式已凍結、前端解析可行，討論期擱置；本刀維持前端行掃描。
- 背景的名稱籤改以預處理 Markdown 連結語法（`[name](speclink://…)`）——sanitize 會擋自訂協定，連結樣式也不是籤；改走接在 sanitize 之後的 rehype 外掛，與 `rehypeSpecKeywords` 同一條路，不動放行清單。
- 分刀卡改只讀 `promoted_to` 不讀結論——拿不到每刀的範圍句、子項與「哪幾個變更屬同一刀」；結論讀不到 cut 清單時才退回平面清單。
- 已封存討論頁的衍生列維持宿主派生的 `SourceLinkItem` 三態——只有一個副標字串，畫不出任務進度、兩站章與日期；改由宿主多傳兩份清單、頁自己推導，與討論詳情頁同一套。
- 一刀連規格詳情頁一起做——使用者另開討論 spec-manual-page-alignment 處理，兩刀 spec delta 互不重疊、可平行。

## Capabilities

### New Capabilities

（none）——規格掃描命中 `desktop-app`（討論側四條需求皆在）；`desktop-manual-page`、`discuss-skill`、`workflow-schemas` 與本刀無關，不改。

### Modified Capabilities

- `desktop-app`：「討論結論以欄位標籤呈現」（決定主句與細節收合、分刀卡、否決與擱置的方案／理由列、記錄去向籤、下一步指令籤、數字磚）、「討論輪以卡片呈現」（時間軸、焦點為標題、立場定論句與列點、否決與待決、首末輪展開、數字磚與「討論已收斂」提示框）、「討論詳情頁檢視與轉出變更」（背景文件的引言、名稱籤、關聯列、記錄列、數字磚與「沒有 grill 階段」提示框；衍生變更文件依分刀分組、列的現況欄位、數字磚與「只差最後一刀」提示框；已封存討論頁同款）、「詳情頁的文件樹、閱讀卡與本頁目錄」（討論與已封存討論的文件改為有推導的出處行；目錄錨點改為結論的區段標題、討論過程的輪節點、背景與衍生變更的區段標題）。

## Impact

- Affected specs: desktop-app（4 條 MODIFIED）
- Affected code:
  - New: packages/ui/src/discussionModel.ts、packages/ui/src/components/ConclusionDoc.tsx、packages/ui/src/components/RoundsDoc.tsx、packages/ui/src/components/ContextDoc.tsx、packages/ui/src/components/PromotedDoc.tsx、packages/ui/src/components/discussionDocSummary.tsx、packages/ui/src/__tests__/discussionModel.test.ts、packages/ui/src/__tests__/conclusionDoc.test.tsx、packages/ui/src/__tests__/roundsDoc.test.tsx、packages/ui/src/__tests__/contextDoc.test.tsx、packages/ui/src/__tests__/promotedDoc.test.tsx
  - Modified: packages/ui/src/components/DiscussionDetailPage.tsx、packages/ui/src/components/ArchivedDetailPage.tsx、packages/ui/src/components/DocParts.tsx、packages/ui/src/components/Markdown.tsx、packages/ui/src/i18n.tsx、packages/ui/src/index.ts、packages/ui/src/__tests__/discussionDetailPage.test.tsx、packages/ui/src/__tests__/archivedDetailPage.test.tsx、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/App.test.tsx
  - Removed: （none）
