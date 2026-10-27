## Context

七點 UI 缺陷全數收斂於討論 desktop-ux-polish（2026-07-11）。前端體系：packages/ui 為元件庫（Tailwind v4＋shadcn 原語＋dnd-kit）、apps/desktop 為 Tauri 宿主（Zustand store、tauriDataSource 經 IPC 委派 speclink-desktop-core）。引擎的 analyze --json 已回傳 dimensions[].status、findings[].location 與 findings[].recommendation，前端現況丟棄；全文搜尋需新增桌面 core 查詢命令（唯一新架構縫，介面深度檢查已於討論通過：單一 adapter、隱藏遍歷比對彙整、刪之則能力消失）。

範圍界線：
- In scope：packages/ui 元件與搜尋規則、apps/desktop 前端狀態與 IPC 接線、apps/desktop/core 查詢層（search 模組＋change 清單補 created）、openspec/LANGUAGE.md 例外條目。
- Out of scope：引擎 CLI 輸出與語意、封存確認流程與 board-card-order 拖排語意、已封存頁搜尋、討論中全卡的既有 slug 標題、任務進度篩選、全文層模糊比對。

## Goals / Non-Goals

- Goals：分析結果可讀（摘要層＋出處＋建議）且可關閉；驗證併入分析單一入口；已轉出討論與討論中同屏可見；slug 在所有討論識別錨點可見可複製；搜尋可發現 artifacts 內文、可篩選、可解釋（高亮＋snippet）；拖曳零 reflow。
- Non-Goals：見 proposal Non-Goals（拆刀、引擎改動、全文 fuzzy、任務進度篩選、已封存頁搜尋皆不做）。

## Decisions

### D1：分析面板兩層結構——結構驗證列＋維度摘要卡＋發現卡

「分析」單鍵同時執行 validate 與 analyze（store 併發呼叫 dataSource.runVerb("validate") 與 runVerb("analyze")——兩動詞互相獨立、無次序依賴，合併為單一結果物件）。VerbDrawerResult 重塑為分析結果形狀：{ change, validate?: {valid, errors}, analyze?: AnalyzeReport, error? }——verb 欄位移除，archive 仍走視窗頂列不受影響。AnalyzePanel 重構為：

1. 結構驗證列：valid=true 呈「✓ 結構驗證通過」單列（成功語意色）；false 呈「✗ 結構驗證 N 個錯誤」並逐條列出 errors（destructive 色）。
2. 維度摘要卡：一排四張（grid-cols-4），繁中維度名對映 Coverage→覆蓋度、Consistency→一致性、Ambiguity→模糊度、Gaps→缺漏；發現數取 analyze 回傳 dimensions[].finding_count；零呈「無問題」（成功語意）、非零呈「N 個問題」（警示語意，Critical 存在時 destructive）。
3. 發現卡：逐條一卡，含嚴重度徽章（沿用既有配色）、location（等寬小字）、summary、recommendation（↳ 前綴的建議行）。發現多時面板內部可捲動（max-height＋overflow-y-auto），不撐爆抽屜 header 區。

UI 藍圖（詳情抽屜）：

```
│ [ ✦ 分析 ● ]                    [🗄 封存] [🗑 刪除] │ ← 驗證鈕移除；分析為切換鈕（開啟時 pressed 樣式）
│ ┌─ 分析結果 ────────────────────────────────── × ┐ │
│ │  ✓ 結構驗證通過                                │ │ ← validate 併入，一列帶過；失敗時逐條列錯
│ │  ┌────────┬────────┬─────────┬────────┐        │ │
│ │  │ 覆蓋度 │ 一致性 │ 模糊度  │  缺漏  │        │ │ ← 維度摘要卡：零=「無問題」綠、非零=「N 個問題」琥珀
│ │  │ 無問題 │ 無問題 │18 個問題│ 無問題 │        │ │
│ │  └────────┴────────┴─────────┴────────┘        │ │
│ │  ┌────────────────────────────────────────────┐│ │
│ │  │ ‹建議› specs/command-runtime/spec.md       ││ │ ← 嚴重度徽章＋location
│ │  │ Scenario「…」沒有具體範例                  ││ │ ← summary
│ │  │ ↳ 加上 ##### Example: 具體 GIVEN/WHEN/THEN ││ │ ← recommendation
│ │  └────────────────────────────────────────────┘│ │
│ └────────────────────────────────────────────────┘ │
```

### D2：動詞結果可關閉——分析鈕切換＋面板關閉鈕＋store 清空動作

store 新增 clearDrawerVerb() 動作（set drawerVerb: null）。RichDetailDrawer 的分析鈕帶 aria-pressed：結果開啟時再點按呼叫 onClearVerb 收合；面板右上關閉鈕同路徑。既有「換 change 清空」與「關抽屜清空」行為保留。收合後再點「分析」重新執行雙動詞。

### D3：已轉出討論改欄底常駐收合列

DiscussionColumn 移除互斥檢視（showPromoted 狀態、header ↗N 開關、欄標題切換、計數徽章隨檢視切換全數撤除）。欄內改為上下兩區：上區 active 全卡清單（捲動區）、下區欄底常駐收合列——promoted.length > 0 時呈現「↗ 已轉出 N ▸」按鈕列（aria-expanded），點按就地展開 promoted 細列（▾），再點按收合。展開狀態為元件 local state、預設收合、不持久化。計數徽章固定顯示 active 數。「無 active 但有 promoted 時不顯空狀態」規則保留（收合列已傳達）。

UI 藍圖（討論欄）：

```
   收合（預設）                  展開後
┌ 💬 討論 ──────── 2 ┐    ┌ 💬 討論 ──────── 2 ┐
│ ┌────────────────┐ │    │ ┌────────────────┐ │
│ │ board-search-  │ │    │ │ board-search-  │ │
│ │ bar ⧉ ‹討論中› │ │    │ │ bar ⧉ ‹討論中› │ │
│ │ 看板搜尋列…    │ │    │ └────────────────┘ │
│ │ 3 輪 · Ⓜ Momo  │ │    │ ── ↗ 已轉出 1 ▾ ── │
│ └────────────────┘ │    │ ┌────────────────┐ │
│                    │    │ │ collab-scena…⧉ │ │ ← slug 首行＋複製鈕
│                    │    │ │ 多人協作情境…  │ │ ← topic 降為描述行
│ ── ↗ 已轉出 1 ▸ ── │    │ │ ├ engine-typed-core ‹提案中›
└────────────────────┘    │ │ └ teamstore-contract ‹已封存›
                          └────────────────────┘
```

### D4：slug 識別擴至 promoted 細列與討論抽屜標題

PromotedRow 首行改為 slug（等寬字型、font-semibold）＋複製 slug 鈕（沿用 DiscussionCard 的 copied 回饋模式），topic 降為次行描述（truncate）；衍生樹與階段 chip 派生規則不動。DiscussionDrawer 的 SheetTitle 改為 slug＋複製鈕，topic 降為副標行——與變更抽屜「名稱＋複製鈕」對稱。openspec/LANGUAGE.md 的 desktop-card-identity 例外條目適用範圍由「僅限 discuss 卡標題與其複製鈕」擴為「僅限討論識別錨點（討論全卡標題、已轉出細列首行、討論抽屜標題）與其複製鈕」，並註記出處 desktop-ux-polish。

UI 藍圖（討論抽屜標題）：

```
┌ collab-scenario-replan ⧉                × ┐ ← slug 為題＋複製鈕
│ 多人協作情境下的 speclink 架構重新規劃…   │ ← topic 降為副標
│ 9 輪 · 2026-07-10                         │
│ ✓討論中 → ✓已結論 → ●轉出變更             │
```

複製鈕位置規則（ingest 自已結論討論 spec-archive-drawer-ux）：複製鈕緊跟標題文字後方，不以 flex-1 將按鈕推至列右緣。DiscussionDrawer 標題列（標題與複製鈕相鄰、spacer 在後）出生即合規、無需改動；PromotedRow 的 slug 為 break-all 多行，複製鈕改行內尾隨——按鈕直接跟在最後一個字元後流動（多行時位於末行文字尾），hover 顯現與 copied 回饋模式不變。替代方案：flex 群組（標題 truncate＋按鈕 shrink-0）——適用單行截斷標題，對多行 break-all 標題會使按鈕垂直置中於整塊旁側，不符「文字後方」語意，故細列採行內尾隨。依使用者實窗回饋，同規則擴及看板卡片標題：變更卡名稱與討論卡 slug 的複製鈕一律行內尾隨、不推至卡片右緣。

按鈕 hover 統一規則（實窗回饋）：所有可點按鈕一律走 shadcn Button（ghost/outline 變體自帶 hover 底色回饋），不得以裸 button 僅做透明度變化——Sheet 關閉鈕（ui/sheet.tsx 的 Close）與變更詳情抽屜的放大鈕依此改為 ghost icon 同款樣式並同高同尺寸並排。

### D5：搜尋列元件化與篩選 chips

新元件 BoardSearchBar（packages/ui）：單列工具列（Jira／Linear 式；實作中依使用者回饋兩度修訂——原「置中大輸入框＋下方常駐 chips 列」視覺重心過大、後續「同列展開 chips」高度不齊且展開時擁擠）。搜尋輸入 SHALL 填滿工具列剩餘寬度（flex-1、h-9），帶搜尋 icon 置左內側；輸入非空時右側呈清除鈕（點按清空並保持聚焦）與即時命中數（「N 張卡」＝過濾後全欄卡片總數）；全域快捷鍵 ⌘F（macOS）／Ctrl+F（其他平台）聚焦輸入框。輸入框右側為同高（h-9）的篩選開關鈕（漏斗 icon）：點按於其下方彈出篩選面板（絕對定位浮層，右對齊開關鈕）；再點開關、點面板外或按 Esc 關閉；關閉不清除已啟用篩選。面板內三個維度選單直欄堆疊（label＋NativeSelect），各維度選回「全部」即單獨清除；存在啟用中篩選時面板底部呈「清除全部篩選」、開關鈕帶啟用計數徽章。篩選三維度：

- 建立者：下拉自 active 變更與討論的 createdBy 去重清單；選定後僅顯該建立者的卡。
- 建立時間：近 7 天／近 30 天／更早三擇一；變更卡以 created 日期（desktop-core 查詢補傳）、討論卡以 created 比對。
- 來源討論：下拉自 promotedTo 非空的討論；選定後顯示該討論卡自身與 fromDiscussions 含該 slug 的變更卡。

多維度與搜尋字串一律 AND 交集。篩選狀態與搜尋字串同壽命（不持久化、與已封存頁獨立）。過濾函式收斂於 packages/ui/src/search.ts（純函式，jsdom 可測）。工具列控制項（漏斗、清除鈕、面板選單）SHALL 具明確 hover 回饋。

UI 藍圖（搜尋列＋篩選面板）：

```
┌───────────────────────────────────────────────┐ ┌───┐
│ 🔍 搜尋看板卡片…                    3 張卡  ✕ │ │ ▽①│ ← input 填滿、與漏斗同高
└───────────────────────────────────────────────┘ └─┬─┘
                                        ┌────────────┴──┐
                                        │ 建立者    ‹▾› │ ← 點漏斗彈出面板
                                        │ 建立時間  ‹▾› │    （右對齊、浮層）
                                        │ 來源討論  ‹▾› │
                                        │ ── 清除全部 ── │ ← 有啟用時才出現
                                        └───────────────┘
```

附帶視覺修正（實作中使用者回饋，同屬本刀 UI 微調範圍）：
- 全域捲軸細化：WebView 預設捲軸過寬，於窄容器（欄底收合區等）視覺突兀——apps/desktop/src/index.css 以 ::-webkit-scrollbar 全域改為細軌（約 8px、透明軌道、muted 圓角滑塊）。
- 變更詳情抽屜的放大鈕與 Sheet 關閉鈕對齊：放大鈕自標題列移為與關閉鈕同高的絕對定位（同列並排），不再一高一低。

### D6：全文搜尋走桌面 core 單一查詢命令

apps/desktop/core 新增 search 模組：search_workspace(root, query) 遍歷 active changes 的 artifacts（proposal.md、design.md、tasks.md、specs/**/spec.md）與 openspec/discussions/*.md（active 討論記錄），不分大小寫子字串比對，回傳命中清單 [{ kind: change|discussion, id, artifact, snippet }]——每張卡取首個命中 artifact 的首個命中處，snippet 為命中前後各約 30 字元的裁切（含命中原文）。Tauri command search_workspace 單行委派；DataSource 介面加 searchWorkspace(query) 方法（tauriDataSource 經 invoke 實作）。前端於 query 非空時以 200ms 去抖觸發查詢，回應以 latest-wins 序號防交錯；欄位比對即時、全文命中到達後併入可見集合。卡片可見規則：欄位命中 OR 全文命中 OR 篩選命中集合內——再與篩選 chips 取 AND。空 query 不觸發查詢。查詢失敗（IPC 錯誤）時靜默退回欄位比對、不阻斷輸入（搜尋是輔助路徑，失敗不彈錯）。

### D7：模糊比對限名稱層、命中高亮與 snippet

search.ts 新增 subsequence 模糊比對（字元依序出現即命中，如 etc 命中 engine-typed-core），僅套用於變更卡名稱與討論卡 slug；摘要與 topic 維持子字串。命中高亮：欄位以子字串命中時，卡片上該字段以 mark 樣式（teal 底）標示命中原文；模糊命中（無連續子字串）不高亮、僅保留卡片。全文命中的卡片於卡身底部呈 snippet 行：📄 artifact 檔名＋裁切前後文，命中原文同樣 mark 高亮。

UI 藍圖（命中卡片）：

```
┌──────────────────────────────┐
│ engine-typed-core            │ ← 名稱子字串命中 → 該段高亮
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│ 📄 design.md 「…唯一 Command │ ← 全文命中 → snippet 行＋命中高亮
│ Runtime 的【dispatch】相容…」│
│ ▓▓▓░░░░░░░ 0/18              │
└──────────────────────────────┘
```

### D8：封存落點浮層化

ArchiveDropZone 改為絕對定位浮層：看板欄列容器設 relative，落點以 absolute 疊於容器右緣（inset-y 留邊、right 貼齊）、z-index 高於欄位、不參與 flex 佈局——欄寬零變動。浮現條件由 dragging 收斂為「拖曳中且 active 卡為變更卡」（activeCard?.kind === "change"）；拖討論卡時不浮現。useDroppable id、封存確認流程、board-card-order 拖排語意全部不變。isOver 時 teal 邊框加亮沿用。

UI 藍圖（拖曳中）：

```
┌──────┬──────┬──────┬──────┐
│ 討論 │ 提案 │ 進行 │ 就緒 │ ← 欄寬完全不變（零 reflow）
│      │ ┌╌╌┐ │      │  ┏━━━━━┓
│      │ ┊原┊ │ [卡]→│  ┃ 🗄  ┃ ← absolute 浮層疊於右緣上方
│      │ ┊位┊ │ 拖曳 │  ┃拖到 ┃
│      │ └╌╌┘ │      │  ┃此封存┃
│      │      │      │  ┗━━━━━┛
└──────┴──────┴──────┴──────┘
  拖「討論卡」時浮層不出現
```

## Risks / Trade-offs

- 任務數超過常規上限：一刀全包為使用者明確裁定（見討論 Ruled out），以任務分組對齊決策編號控管。
- 全文查詢的效能：每次去抖觸發全量遍歷 active changes 與討論檔——本專案量級（十數卡）下無感；不預建索引（YAGNI），量級成長時再議。
- ChangeItem 補 created 欄位動到桌面 core 查詢 payload：僅疊加欄位、不改既有欄位形狀，CLI 輸出不受影響（desktop-core 與 CLI 分離）。
- 浮層落點疊於最右欄上方，拖曳目標靠近右緣時可能與「已就緒」欄卡片視覺重疊——落點寬度收窄（約 120px）並半透明背景緩解；真實視窗驗證把關。
- jsdom 測不出拖曳與浮層互動（CLAUDE.md 開發備忘）：dnd 與浮層行為以純函式（浮現條件、落點解析）單元測試＋真實視窗手動驗證雙軌把關。

## Migration Plan

單版本內完成，無資料遷移。LANGUAGE.md 例外條目於同變更內同步擴充。舊互斥檢視的 i18n 鍵（discussion.showPromoted、discussion.headingPromoted）隨元件移除一併清理。

## Implementation Contract

- 觀察行為：
  - 詳情抽屜動作列僅有「分析」「封存」「刪除」；點「分析」一次呼叫 validate 與 analyze 並展開分析面板（結構驗證列＋四張繁中維度摘要卡＋逐條發現卡含 location 與 recommendation）；再點「分析」或面板 × 收合。
  - 討論欄同屏呈現 active 全卡與欄底「已轉出 N」收合列；展開後細列以 slug 為首行且可複製；討論抽屜標題為 slug＋複製鈕。
  - 看板搜尋列有 icon、清除鈕、命中數、⌘F/Ctrl+F 聚焦；篩選 chips（建立者、建立時間、來源討論）與字串 AND 交集；名稱/slug 支援 subsequence 模糊命中；子字串命中高亮；全文命中卡片帶 snippet 行。
  - 拖曳變更卡時封存落點以浮層浮現、四欄寬度不變；拖討論卡不浮現。
- 介面與資料形狀：VerbDrawerResult 改為 { change, validate?, analyze?, error? }；DataSource 增 searchWorkspace(query) 回傳 [{ kind, id, artifact, snippet }]；ChangeItem 增 created?: string | null。
- 失敗模式：validate/analyze 任一失敗→面板呈 core 錯誤訊息不靜默；searchWorkspace 失敗→靜默退回欄位比對；複製鈕於剪貼簿 API 缺席時不崩潰（既有 ?. 模式）。
- 驗收條件:npm test -w packages/ui 與 npm test -w apps/desktop 全綠（新增與改寫的測試如任務所列）；cargo test -p speclink-desktop-core 全綠（search 模組單元測試）；真實視窗手動驗證浮層落點零 reflow 與拖曳互動。
- 範圍邊界：不動引擎 CLI、封存語意、已封存頁、board-card-order 規格行為。
