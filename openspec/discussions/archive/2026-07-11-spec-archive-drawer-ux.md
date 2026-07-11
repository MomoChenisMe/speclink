---
topic: 規格頁與已封存頁改抽屜呈現＋收合卡片資訊強化＋專案頁籤徽章語意
slug: spec-archive-drawer-ux
status: promoted
promoted_to: spec-archive-drawer, desktop-ux-polish, board-card-anatomy
created: 2026-07-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 規格頁與已封存頁改抽屜呈現＋收合卡片資訊強化＋專案頁籤徽章語意

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者附截圖指出：規格頁與已封存頁目前以行內收合/展開呈現內容，希望改成與變更頁一致的抽屜（Sheet）模式；並想強化兩頁收合卡片的資訊設計。後續補入第三題：專案頁籤的「進行中變更數」徽章意義不明，徵求改進想法。

模式：assumptions（探勘即找到 SpecList.tsx、ArchivedList.tsx、RichDetailDrawer.tsx、DiscussionDrawer.tsx、ChangeCard.tsx、adapter.ts、store.ts、tabs.ts，脈絡充足）。

相關現況：ArchivedRow 展開內容（提案/設計/任務/規格四分頁）與 RichDetailDrawer 分頁結構同構；SpecItem 僅 id＋modifiedAt、ArchivedItem 僅日期/名稱/任務數（packages/ui/src/adapter.ts:26-51）；頁籤徽章＝changeStage 為 in-progress 的變更數（apps/desktop/src/tabs.ts:inProgressCount），active 分頁由 live changes 派生、背景分頁取 stats 快照（store.ts:448）。

相關變更/討論：desktop-ux-polish（已轉出）為前一輪桌面 UX 微調；本討論為後續一輪。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-11)

**Focus**: 規格頁與已封存頁從行內展開改抽屜的做法與邊界
**Position**: 四項假設全數確認，抽屜寬度統一與變更抽屜相同：
- 新建唯讀抽屜（規格抽屜＋封存抽屜），重用 Sheet／SectionedDoc／TaskList readOnly／DeltaSpecView 與 DiscussionDrawer 的 RoundsView/ConclusionView；不在 RichDetailDrawer 加唯讀分支
- 整列點擊＝開抽屜，移除行內展開與 chevron；懶載入＋refreshGen 世代重載語意搬進抽屜（SpecRow 的 design D4 邏輯移植）
- 抽屜狀態比照 detailChange 接進 store；已封存頁需要變更與討論兩種目標
- 卡片資訊強化由 Rust 端擴充 listSpecs/listArchived payload（不開新動詞、不加 adapter 層）——收合卡懶載入，前端無內容可算
- 抽屜寬度統一 w-[max(720px,42vw)]＋全螢幕鈕（使用者裁定：預設都一樣寬）
**Ruled out**: RichDetailDrawer 加 readOnly 旗標（change 專屬互動太多，分支地獄）；行內展開與抽屜並存（雙互動語意）；前端預讀全文算卡片資訊（39 份封存＝啟動 39×4 次讀檔）
**Open**: 各卡片最終帶哪些欄位待使用者從提案選單挑選（規格卡：需求數/Purpose 首句與 TBD 待補提示/溯源變更數；封存變更卡：任務徽章配色分級/觸及規格數/createdBy/來源討論 icon；封存討論卡：slug 複製鈕/衍生變更數）；新增議題——專案頁籤「進行中」徽章語意不明，改進方向待議

### Round 2 — assumptions (2026-07-11)

**Focus**: 專案頁籤徽章（進行中變更數）的存廢與語意
**Position**: 徽章保留但改語意為「待收尾數」（方向 B）：
- 現行徽章＝changeStage in-progress 的變更數（tabs.ts:inProgressCount），在 active 分頁與看板「進行中」欄標頭純冗餘
- 徽章唯一不可替代的場景是背景分頁的切換訊號；使用者確認會同時開多個專案，場景成立
- 「進行中」不是行動訊號（agent 在做事、不需要人）；改計「等使用者執行動詞」的卡片：已就緒變更＋已結論未轉出的討論
- 徽章從狀態顯示翻成行動提示，歸零有明確達成感；tooltip 文案同步改
- 背景分頁 stats 快照需擴欄位（store.ts:448 現取 inProgressChanges），小改
**Ruled out**: A 拿掉徽章（多專案場景成立，背景訊號有價值）；C 數字改圓點（「在飛」依然不是行動訊號，資訊量更低）
**Open**: 卡片欄位選單待挑；新增議題——複製鈕改放標題文字正後方、全卡片對齊（含 desktop-ux-polish 在途改動的卡片），與在途變更的協調方式待議

### Round 3 — assumptions (2026-07-11)

**Focus**: 複製鈕位置規則、與在途變更的協調方式、卡片欄位選單定案
**Position**: 複製鈕一律緊跟標題文字後方，規則歸本討論的新變更：
- 標題＋複製鈕包成 flex 群組（標題 min-w-0 truncate、複製鈕 shrink-0、hover 顯現照舊），群組吃 flex-1，meta icons 維持靠右
- 看板討論卡 slug 為 break-all 多行（DiscussionColumn.tsx:93），改行內尾隨（按鈕跟在最後字元後流動）
- 影響：ChangeCard、看板討論卡、規格卡、封存變更卡按新規則搬／做；封存討論卡補複製鈕；變更／討論抽屜標頭已合規免動（參照原型）
- 協調：對 desktop-ux-polish 輕量 ingest，其任務 4.1/4.2 的兩顆新複製鈕（已轉出細列、討論抽屜標題）出生即按新位置；實作順序 desktop-ux-polish 先落地，本變更後動 DiscussionColumn/ChangeCard 避免同檔並行
- 卡片欄位選單全拿（使用者無刪減）：規格卡＝需求數＋Purpose 首句與 TBD 待補提示＋溯源變更數；封存變更卡＝任務徽章配色分級＋觸及規格數＋createdBy＋來源討論 icon；封存討論卡＝slug 複製鈕＋衍生變更數
**Ruled out**: 規則整包 ingest 進 desktop-ux-polish（21 任務的在途變更不宜再長大）；flex 群組套用於多行 slug（按鈕垂直置中於多行旁，不符「文字後方」）
**Open**: 新增議題——看板討論卡、變更卡、已轉出細列三種設計不一致，統一卡片解剖學待議

### Round 4 — assumptions (2026-07-11)

**Focus**: 看板討論卡／變更卡／已轉出細列設計不一致，統一卡片解剖學
**Position**: 定一套三列骨架（識別列／描述列／meta 列），全 app 全卡共用，四項決定全數同意：
- 識別列＝等寬標題＋行內複製鈕＋右端狀態 chip 與頭像；描述列＝一至兩行截斷（可選）；meta 列＝進度條或輪數＋時間
- 標題字體統一等寬：變更名稱與討論 slug 同為 kebab-case 可複製把手，語意相同外觀一致（變更卡由粗體 sans 改 mono）
- 變更卡補描述列：proposal 的 Why 首句一行截斷；changes list payload 擴摘要欄位（與規格/封存卡的 payload 擴充同一條路）
- 討論卡作者收斂為頭像圓點＋tooltip（現為全名 email 直出，DiscussionColumn.tsx:113）；「N 輪」挪進 meta 列
- 已轉出細列維持細列不升級全卡，共用識別元素（slug 等寬＋行內複製＋同款階段 chip）
- 狀態 chip 規則：僅在「所在位置無法表達狀態」時出現（討論欄一欄兩態需要；變更卡所在欄即階段，不加）
- 規格卡／封存卡（本討論重設計）同套解剖學，全 app 卡片一個心智模型
**Ruled out**: 已轉出細列升級為全卡（desktop-ux-polish D3 正把它收進欄底收合列降噪，反方向）；chip 有無隨卡各自決定（不一致的根源）
**Open**: 無——進結論

## Conclusion

**Decision**: 四個議題一次定案——
1. 規格頁與已封存頁廢除行內展開，改與變更頁一致的抽屜（Sheet）呈現：新建唯讀規格抽屜與封存抽屜（重用 Sheet／SectionedDoc／TaskList readOnly／DeltaSpecView／RoundsView/ConclusionView），整列點擊開抽屜、懶載入與 refreshGen 世代重載搬進抽屜、狀態比照 detailChange 接 store、寬度統一 w-[max(720px,42vw)]＋全螢幕鈕。
2. 收合卡片資訊強化，欄位全拿：規格卡＝需求數＋Purpose 首句（TBD 佔位偵測為「Purpose 待補」琥珀提示）＋溯源變更數；封存變更卡＝任務徽章配色分級＋觸及規格數＋createdBy＋來源討論 icon；封存討論卡＝補 slug 複製鈕＋衍生變更數。資料由 Rust 端擴充 listSpecs/listArchived payload（不開新動詞）。
3. 專案頁籤徽章改語意為「待收尾數」＝已就緒變更＋已結論未轉出討論（等使用者執行動詞的卡片數）；背景分頁 stats 快照擴欄位；tooltip 同步改。
4. 統一卡片解剖學（三列骨架：識別列／描述列／meta 列）：標題一律等寬＋複製鈕緊跟標題文字後方（多行 slug 用行內尾隨）、變更卡補 Why 首句描述列（payload 擴欄位）、討論卡作者收斂頭像圓點＋tooltip、「N 輪」入 meta 列、已轉出細列維持細列但共用識別元素、狀態 chip 僅在所在位置無法表達狀態時出現。

**Rationale**: 行內展開與抽屜是兩套閱讀心智模型，統一成抽屜後卡片降級為純資訊卡，資訊強化才有意義；收合卡懶載入使前端無內容可算，卡片欄位必須由 list payload 帶出——一次擴欄位同時餵飽規格卡、封存卡與變更卡描述列。徽章「進行中 N」在 active 分頁與看板欄標頭純冗餘、對背景分頁也非行動訊號；「待收尾 N」把狀態顯示翻成行動提示（使用者確認多專案並用，背景訊號場景成立）。卡片不一致的根源是各自演化無共用骨架；一致性靠共用元素詞彙（等寬把手＋行內複製＋同款 chip），而非把不同層級（全卡 vs 細列）做成同一種卡。

**Rejected alternatives**: RichDetailDrawer 加 readOnly 旗標（change 專屬互動太多，分支地獄）；行內展開與抽屜並存（雙互動語意）；前端預讀全文算卡片資訊（39 份封存＝啟動 39×4 次讀檔）；徽章拿掉（多專案背景訊號有價值）；徽章數字改圓點（「在飛」非行動訊號）；規則整包 ingest 進 desktop-ux-polish（21 任務在途變更不宜再長大）；已轉出細列升級全卡（與 desktop-ux-polish D3 降噪方向相反）。

**Deferred**: 無。

**Capture to**: proposal＋design（建議扇出兩個變更：A「規格/封存頁抽屜化＋卡片資訊強化＋頁籤徽章」可立即動工，與 desktop-ux-polish 無檔案交集；B「看板卡片解剖學統一」動 ChangeCard/DiscussionColumn，排在 desktop-ux-polish 落地後）；另對 desktop-ux-polish 輕量 ingest（任務 4.1/4.2 兩顆新複製鈕出生即置於標題後、細列識別元素同款）；LANGUAGE.md 建議收新詞「待收尾」（定義：等使用者執行動詞的卡片＝已就緒變更＋已結論未轉出討論）。

**Next**: /speclink-propose --from-discussion spec-archive-drawer-ux
