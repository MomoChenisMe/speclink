---
topic: 規格轉操作手冊的 speclink 技能規劃
slug: manual-generation-skill
status: promoted
promoted_to: manual-skill, desktop-manual-page
created: 2026-09-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 規格轉操作手冊的 speclink 技能規劃

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：本 session 以兩個實測驗證了「正式規格 → 新人操作手冊」可行——speclink 專案（有驗收劇本，旅程屬轉寫）與 wadpilot 專案（無劇本、343 份規格，旅程屬重建），成品為兩份 artifact（一書式、一 wiki 式）。使用者確認 wiki 形態即所要，現在要把這套流程固化成 speclink 的技能。目標可驗證，未經磨題階段。待決節點：觸發時機、呈現一致性與 token 經濟、互動式導覽形態、LLM Wiki 適用性。相關正典：skill-routing（入口聯集與交棒句契約）、archive-skill（封存收尾提醒的掛點）、user-documentation（speclink 自身文件地圖的邊界）。進行中變更：無相關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-01)

**Focus**: 技能整體形態，與觸發時機、呈現一致性兩節點的初步方向
**Position**: 做成工具型對外技能；人工觸發＋封存時提醒；固定模板資產產靜態 HTML——使用者四點回覆確認並擴充：
- 產出＝含動畫、互動、JS 的單檔靜態 HTML，版型來自技能自帶的固定模板資產；模型只產內容（確認）
- 觸發＝change 封存且動到正式規格時「提醒使用者更新 wiki」，不自動生成（確認；提醒掛在封存收尾，符合 skill-routing 僅建議不代跑）
- 過期偵測：手冊 manifest 記頁 ↔ 來源 capabilities ＋ 生成基準，重生只做受影響頁——token 大頭省在讀與寫都只碰受影響範圍
- 新需求：技能要有兩種能力——(a) 產靜態 wiki 手冊 (b) 與 AI 互動式 wiki 導覽（新節點，形態待定）
- SSG（Docusaurus/VitePress/Starlight）不當預設、留作升級路徑；內容以 Markdown 產出使遷移近零成本
**Ruled out**: 自動觸發生成——違反 skill-routing「僅建議、SHALL NOT 自動呼叫」；每次全生 HTML——token 最貴且設計漂移（兩次試作實測）；MkDocs Material——2026 已進維護模式
**Open**: 互動式導覽的形態（對話內導覽 vs 頁面內嵌 AI 問答）；LLM Wiki（DeepWiki 型）是否適合本情境；產出落點與 manifest 細節；技能命名；archive 收尾提醒句的 canon 變更範圍

### Round 2 — interview (2026-09-01)

**Focus**: 互動式導覽的形態，與 LLM Wiki（DeepWiki 型）的適用性
**Position**: 導覽走「對話內導覽」（A 案），LLM Wiki 路線出局：
- 技能雙動線：生成（產靜態 wiki）＋導覽（對話內互動導覽，不產檔），以技能引數選模式，例 /speclink-manual 導覽
- 使用者裁定不走 LLM Wiki——DeepWiki 型工具吃程式碼、給開發者看架構；本技能吃正式規格、給操作者看行為，只借其「wiki＋問答」互動模型的形，不接其源
- 頁面內嵌 AI 問答（B 案）降為可選加值：僅在發佈成 claude.ai artifact 且開 runtime capability 時可行，repo 單檔 HTML 無後端做不到
**Ruled out**: LLM Wiki／DeepWiki 路線——來源與讀者皆不合（code→開發者 vs specs→操作者）；B 案作為主形態——綁死託管環境
**Open**: 技能命名與模式引數形狀；產出落點與 manifest 細節；archive 收尾提醒句的 canon 變更範圍

### Round 3 — interview (2026-09-01)

**Focus**: 模板機制三案（A/C1/C2）的取捨與產出格式
**Position**: 產出改純 Markdown，HTML 生成與模板機制整組出局：
- 技能只產 Markdown 頁與來源對應；呈現交給讀取端，使用者可自行拆解利用（餵 SSG、pandoc、任何工具）
- 一致性從「生成端保證」移到「讀取端保證」：speclink desktop 新增 wiki 介面讀取這批 Markdown（形態同 wadpilot 試作：側欄樹＋搜尋＋一頁一主題），設計一致由產品 UI 承擔
- v1 技能層零引擎改動；desktop wiki 介面自成範圍（可能獨立成第二個 change）
**Ruled out**: A（技能附檔機制）、C1（模板動詞）、C2（引擎 build 動詞）——產出改 Markdown 後皆無存在必要；為一次性需求擴機制違反專案慣例
**Open**: Markdown 頁的格式設計（frontmatter schema 與內文慣例）；檔案落點（openspec/manual/ vs docs/manual/）；desktop wiki 介面的 change 切分

### Round 4 — interview (2026-09-01)

**Focus**: Markdown 格式一致性是否仍需 reference 檔；Markdown 可否支撐 desktop 的左側導覽與上下篇
**Position**: 兩者皆不成障礙：
- 格式契約以文字規則形式寫進 SKILL.md 本文即可，不需附檔——格式規範約 30 行文字，與既有技能內嵌輸出格式的慣例同型（discuss 的 round 模板、verify/review 的工單輪格式都內嵌在技能本文）；當初要 references 是因為 HTML 模板是 15-20KB 的程式碼，內嵌會撐爆技能檔，Markdown 格式規範沒有這個問題
- 格式漂移的硬保證留作後手：desktop 對缺欄位寬容降級；若實際發生漂移，循 manual-marker-placement-lint 先例補 validate 級 lint，v1 不預作
- 側欄與上下篇由 frontmatter 機械可推：order 採全域序（跨頁唯一、間隔 10 便於插頁），頁面依 order 排序＝上下篇序列，側欄以 section 對連續頁分組——與 SSG 業界（VitePress/Starlight/MkDocs）同一機制，wadpilot 試作的 JS 導覽即此模型
**Open**: 落結論（含兩個 change 的切分：manual 技能、desktop wiki 介面）

### Round 5 — assumptions (2026-09-02)

**Focus**: desktop wiki 介面的設計
**Position**: 新增頂層「手冊」頁，以現有元件組裝、frontmatter 機械推導導覽——使用者五條假設全數確認：
- 入口：與看板、規格、討論、已封存並列的頂層「手冊」頁；無 openspec/manual/ 時顯示空狀態與引導文案；文案受 ui-copy-vocabulary 守門
- 佈局：左側欄樹＋內容區＋搜尋＋上下篇；元件現成——packages/ui 已有 Markdown.tsx（react-markdown＋remark-gfm）、DocumentTree.tsx、DocumentViewer.tsx；唯一新件是 GitHub Alert 語法的小型 remark 插件
- 接線：desktop core query 層直接讀 openspec/manual/*.md，watcher 跟隨→技能重生頁面即時刷新；深度檢查：單一 adapter、frontmatter 解析與導覽模型推導有實質深度；v1 不過 store 抽象
- 出處互動：頁尾 sources 的 capability 名可點，跳既有規格檢視（SpecDrawer/SpecList）——手冊↔規格雙向可跳
- 過期標示：比對頁 generated/sources 與規格 @trace updated，側欄加「可能過期」小標；與 archive 提醒句為同一訊號的 GUI 出口
**Open**: 全盤檢視後收案

### Round 6 — assumptions (2026-09-02)

**Focus**: 收案前的全盤檢視——找漏洞而非復述
**Position**: 結構成立，五處補強後收案：
- 過期有兩種，記錄先前只寫一種：(a) 既有頁的來源規格變了；(b) 新出現的使用者面向 capability 尚無任何頁涵蓋——技能的過期報告與 desktop 的標示都要涵蓋 (b)
- archive 提醒句條件改簡：archive 技能是文字資產，判斷不了「動到使用者面向規格」；條件改為「openspec/manual/ 存在」即提醒可跑 manual 檢查過期，過期判定歸 manual 技能
- 只重生受影響頁時 SHALL 先讀既有 frontmatter、保留 section/order，新頁以間隔插入——否則每次重生側欄順序漂移；此規則進格式契約
- 契約歸屬：frontmatter schema、落點、過期判定基準由一個正典 capability 承載（暫名 manual-pages），manual 技能與 desktop 兩個 change 皆引用；由 manual 技能的 change 建立，desktop change 對 desktop-app 做 delta
- 技能本文要載明讀取策略（token 與品質核心）：以 Purpose/名稱先分流使用者面向 vs 引擎內部，Purpose 為 TBD 時退回讀 Requirement 標題；旅程來源優先找劇本型規格（驗收劇本、routing 交棒表、user-documentation），無則從能力規格重建；必產「首頁」與「本手冊的來源」頁，後者列規格內部新舊矛盾與侷限
- 詞彙：「手冊」「導覽」「可能過期」為新的使用者可見詞，進 LANGUAGE.md 以免與「說明書／指南／文件」混用
**Ruled out**: 以「是否動到使用者面向規格」作 archive 提醒條件——文字資產無法判定
**Open**: 無——全部進結論

## Conclusion

**Decision**: 新增 speclink 工具型對外技能 `manual`（/speclink-manual），雙動線；呈現交給 desktop 新增的「手冊」頁。
- 生成模式：讀正式規格 → 寫 `openspec/manual/*.md`。格式契約內嵌於技能本文：frontmatter（title、section、order 全域序間隔 10、keywords、sources、generated）＋ GitHub Alert 內文慣例＋頁尾出處行；必產「首頁」與「本手冊的來源」頁（含規格內部矛盾與侷限）。重生只做受影響頁，先讀既有 frontmatter 保留 section/order。
- 過期報告：啟動時比對 (a) 既有頁 sources 的規格 @trace updated 晚於 generated、(b) 使用者面向 capability 無頁涵蓋；只讀受影響規格。
- 讀取策略載明於技能本文：Purpose/名稱分流使用者面向 vs 引擎內部（TBD 時退回 Requirement 標題）；旅程優先取劇本型規格（驗收劇本、routing 交棒表、user-documentation），無則從能力規格重建。
- 導覽模式：以引數觸發（如「/speclink-manual 導覽」），對話內互動導覽，以 frontmatter 為索引；無手冊時退回直接掃規格。
- 觸發一律人工；archive 技能收尾在 `openspec/manual/` 存在時加一條提醒句（僅建議不代跑）。
- desktop：頂層「手冊」頁，desktop core query 層讀 `openspec/manual/`、watcher 跟隨；側欄樹＋搜尋＋上下篇由 frontmatter 機械推導；組裝既有 Markdown/DocumentTree/DocumentViewer 元件，新增 GitHub Alert remark 插件；sources 可點跳規格檢視；側欄標示「可能過期」頁與未入冊的新能力；空狀態引導文案。
- 契約歸屬：新 capability（暫名 manual-pages）承載 schema、落點、過期判定基準；manual 技能 change 建立它並對 skill-routing、archive-skill 做 delta；desktop change 對 desktop-app 做 delta 並引用。
**Rationale**: 內容與呈現分離——技能產通用 Markdown（token 最省、使用者可自由拆解），一致性由 desktop UI 硬保證而非生成紀律；技能側 v1 零引擎改動；兩次實測（speclink 有劇本、wadpilot 無劇本）證明規格的 Scenario 密度足以支撐手冊。
**Rejected alternatives**: HTML 產出與模板三案（A 技能附檔機制／C1 模板動詞／C2 引擎 build 動詞）——改 Markdown 後無存在必要；LLM Wiki／DeepWiki 路線——吃程式碼給開發者，源與讀者不合；自動觸發生成——違反 skill-routing 僅建議不代跑；SSG 當預設——工具鏈前置、內容已是 Markdown 可日後遷移；MkDocs Material——2026 維護模式；頁面內嵌 AI 問答當主形態——綁託管環境；archive 以「是否動到使用者面向規格」為提醒條件——文字資產無法判定。
**Deferred**: validate 級 frontmatter lint（漂移實際發生再補）；SSG 匯出模式；claude.ai artifact 內嵌問答加值；remote 模式的手冊投影（PM 無 checkout 的讀取）；speclink 自身文件地圖（user-documentation）與手冊的關係。
**Capture to**: proposal ×2（manual 技能；desktop 手冊頁）、spec（manual-pages）、LANGUAGE.md（手冊／導覽／可能過期）
**Next**: /speclink-propose --from-discussion manual-generation-skill（manual 技能）；desktop 手冊頁以 `speclink discuss promote manual-generation-skill --name desktop-manual-page` 再轉出一個變更後 propose 補齊 artifacts
