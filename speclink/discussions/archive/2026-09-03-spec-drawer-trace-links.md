---
topic: 規格抽屜沿 @trace 連到變更與討論
slug: spec-drawer-trace-links
status: promoted
promoted_to: drawer-provenance-links
created: 2026-09-03
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 規格抽屜沿 @trace 連到變更與討論

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者問 manual 技能是否只讀正式規格、加入所屬 change 的討論能否加深手冊的精確度與脈絡。經一輪評估裁定手冊維持只讀規格，題目轉為 desktop 規格抽屜能否沿 @trace 連到變更、再連到討論，把脈絡鏈接起來。目標可驗證，未經磨題階段。相關正典：manual-skill／manual-pages（單一來源契約、sources 綁 capability）、desktop-app（規格抽屜的溯源資訊、detail 抽屜互斥、變更與討論抽屜開啟時底層落回看板）、desktop-manual-page（出處可點開規格抽屜）、trace-verb（speclink trace 演進鏈）、discussion-docs（討論與變更鏈結雙向可查）。進行中變更：無。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-03)

**Focus**: 手冊要不要吃討論；規格抽屜要不要沿 @trace 連到變更與討論
**Position**: 手冊維持只讀規格（使用者裁定）；規格抽屜的溯源 footer 從純文字改為可點、連到既有封存抽屜，即可閉環：
- 引擎已有 `speclink trace <cap> --json`（changes[].archivedDir／fromDiscussion、discussions[].promotedTo、requirements[].source），鏈本身不需新引擎
- SpecDrawer 已渲染溯源 footer（packages/ui/src/components/SpecDrawer.tsx 以 parseTraceSources 從 @trace 抽 source 名，純文字）；ArchivedDrawer 已有來源討論籤可跳討論抽屜；缺的只有「規格→封存變更」這一跳
- name→datedName 走既有 s.archived 清單（開工作區即載入；remoteDataSource 與 tauriDataSource 皆實作 listArchived；199 份封存名稱無重複），零引擎改動、兩模式同行為
- 抽屜互斥表允許「規格→封存」；封存抽屜不觸發底層落回看板，自手冊頁一路跳到討論，底層仍留手冊頁
- 手冊評估數據：已封存討論 120 份／358 輪／約 1.08 MB，規格 81 份／約 1.36 MB；199 份已封存變更中 189 份帶 from_discussion
**Ruled out**: 手冊取材加討論——精確度來自 Scenario 而討論無 Scenario；輪只能往後加、早期立場會被推翻（manual-generation-skill 第 1 輪 HTML、第 3 輪改 Markdown）；讀量近倍增；sources 契約綁 capability 且討論封存後無更新日期。規格抽屜改吃 speclink trace 樹——需新 desktop query，且 server 無 trace 端點、remote 模式做不到，v1 過重
**Open**: 一跳直達討論（footer 同列來源討論）還是兩跳（沿封存變更抽屜）；找不到封存目錄的 source 名如何呈現（建議比照手冊出處：不可點文字）；逐條需求就地標來源變更是否留作 v2

### Round 2 — interview (2026-09-03)

**Focus**: A 案定案後，溯源連結的 UI 要對齊哪個既有樣式
**Position**: 溯源從內文底部的純文字 footer 搬進抽屜標頭的出身列，重用共用的 SourceChipRow：
- 對齊對象：變更詳情抽屜與已封存抽屜的標頭都是「標題列（名稱＋複製鈕）＋出身列（『來自』＋首籤＋『+N』浮層）」；SourceChipRow（packages/ui/src/components/SourceDiscussionChip.tsx）是兩者共用的單一實作，規格抽屜直接重用
- 規格抽屜標頭改兩層：標題列（capability 名＋複製名稱鈕，與規格卡複製鈕同款）、出身列（「來自」＋變更籤＋「+N」）；無狀態列與動作列——正典唯讀，與已封存抽屜「無進度條與動詞動作列」同理
- 首籤＝出身＝最早封存的變更（建立此 capability 的那個），其餘依封存日期收進 +N 浮層；浮層項副標放封存日期（ArchivedItem.date）
- 點籤：host 以 s.archived 把變更名對應到 datedName 後 openArchived({kind:"change", datedName})；找不到封存目錄的名稱以不可點的灰籤呈現（SourceLinkItem 補 disabled 旗標，比照手冊「不存在的出處不可點」）
- 內文底部 footer 移除；i18n 的 specs.sourceChanges／specs.sourceSep 隨之孤兒化，一併清掉
- 正典：desktop-app「桌面 app 呈現 change 與 spec 的清單與內容」只寫「溯源資訊」未指定位置，改標頭不衝突，需 MODIFIED delta 載明籤與跳轉
**Ruled out**: 保留 footer 只加連結——與另外兩個抽屜的出身列樣式不一致，正是使用者指出的問題；以「溯源」作出身列標籤——出身列文法是「來自」，同一關係詞跨三個抽屜一致，「溯源」留給頁面與指令名
**Open**: 首籤取最早還是最新的變更；複製名稱鈕要不要一併補上；逐條需求就地標來源變更留作 v2

### Round 3 — interview (2026-09-03)

**Focus**: 已封存討論抽屜補上衍生變更（promoted_to）的呈現與跳轉
**Position**: 比照已封存變更抽屜的「來自」列，在已封存討論抽屜的出身列加「衍生」＋變更籤＋「+N」浮層，重用 SourceChipRow：
- 資料已在手上：已封存討論清單項帶 promotedTo（ArchivedList.tsx 以其長度顯示衍生變更數徽章），host 比照 archivedSourceDiscussions（App.tsx）自清單派生，抽屜零新查詢
- 點籤：子變更在封存清單→openArchived({kind:"change", datedName})；仍活躍→openDetail（底層依規落回看板）；兩處皆無（已刪除）→不可點灰籤；三態判定重用 discussionChipStage
- 浮層副標：已封存者放封存日期，活躍者放看板階段詞
- 對稱性：已封存變更抽屜「來自」討論籤 ↔ 已封存討論抽屜「衍生」變更籤，與規格抽屜「來自」變更籤同一元件、同一互動；三個唯讀抽屜的標頭文法一致
- 正典：「已封存頁含討論節」規定區段標題僅背景／討論過程／結論——加在標頭不動區段清單，不衝突；「已封存項目以抽屜檢視」的出身列需 MODIFIED delta 載明衍生列
- 上一輪三題（首籤最早封存、標籤「來自」、補複製鈕）使用者未改，視為定案
**Ruled out**: 比照活討論抽屜加第四區段「衍生變更」（列各子變更現況＋開啟卡片鈕）——已封存討論的子變更幾乎全為已封存，「現況」欄無資訊量；且與正典區段清單字面衝突；標頭籤列更輕
**Open**: 標籤字「衍生」或「衍生變更」；收案前全盤檢視

### Round 4 — assumptions (2026-09-03)

**Focus**: 收案前的全盤檢視——找漏洞而非復述
**Position**: 結構成立，「衍生」標籤定案，五處補強後收案：
- 規格抽屜要拿到封存日期與 datedName 才能排序與跳轉：比照 DiscussionDrawer 的 archivedChanges prop，host 傳入 s.archived（開工作區即全量載入，remoteDataSource.listArchived 無分頁），抽屜內解析與排序，零新查詢
- 籤的三態統一一套機制：SourceLinkItem 補 disabled；已封存→openArchived、活躍→openDetail（衍生列限定；規格的 @trace 來源必為已封存）、皆無→不可點灰籤，浮層副標統一「無封存記錄」；已封存變更抽屜既有「來自」列已由文件載入器 live 優先／封存後備處理，無同類缺口
- 既有測試要改：packages/ui/src/__tests__/specDrawer.test.tsx 四處斷言 footer 字面「來源變更：」須改為標頭籤；archivedDrawer.test.tsx 補衍生列三態；App 測試補接線
- 正典 delta 兩條 MODIFIED：desktop-app「桌面 app 呈現 change 與 spec 的清單與內容」（溯源資訊改為標頭標題列＋出身列籤、點籤跳轉、無封存記錄不可點、內文底部不再有溯源行）與「已封存項目以抽屜檢視」（封存討論抽屜出身列加「衍生」籤列與三態跳轉）；scenario 名保留不改（改名＝引擎眼中的未宣告刪除）
- 詞彙：LANGUAGE.md「衍生變更」詞條補用法註記——出身列標籤縮寫為「衍生」，與「來自」「同源」同為兩字；不新增詞條
**Ruled out**: 為同名多份封存目錄訂解析規則——目前 199 份無重名、正典沉默，不預作；規格抽屜出身列加「最近更新」日期——未被要求
**Open**: 無——全部進結論

## Conclusion

**Decision**: 手冊維持只讀正式規格；脈絡鏈改由 desktop 三個唯讀抽屜的標頭籤列接起來——規格抽屜加「來自」變更籤、已封存討論抽屜加「衍生」變更籤，與既有「來自」討論籤同元件同互動，鏈的每一跳皆可點。
- 規格抽屜標頭改兩層：標題列（capability 名＋複製名稱鈕，與規格卡同款）、出身列（「來自」＋首籤＋「+N」浮層）。首籤為最早封存的變更（出身），其餘依封存日期收進浮層，浮層項副標為封存日期。點籤開啟該封存變更抽屜（互斥規則下規格抽屜關閉、底層頁面不動）。內文底部的「來源變更：」灰字行移除，其 i18n 詞條一併清除。
- 已封存討論抽屜出身列加「衍生」＋變更籤＋「+N」浮層，資料自封存討論清單項的 promotedTo 派生，零新查詢。三態：子變更已封存→開其封存抽屜；仍活躍→開其詳情抽屜（底層依正典落回看板）；皆無→不可點灰籤。
- 三態共用一套機制：SourceLinkItem 補 disabled 旗標，無對應記錄的籤副標統一「無封存記錄」。規格抽屜以 host 傳入的封存清單解析變更名→datedName／日期。
- 不做：逐條需求就地標來源變更（v2）；規格抽屜內嵌 speclink trace 樹；活討論抽屜式的第四區段。
**Rationale**: 引擎已有整條鏈（speclink trace）且三跳中兩跳的 UI 已存在，補最後一跳並統一標頭文法即可閉環；把脈絡放在既有抽屜一次點擊之外，手冊讀者（操作者）與規格讀者（開發者／PO）各取所需，手冊的單一來源契約不動。
**Rejected alternatives**: 手冊取材加討論——精確度來自 Scenario 而討論無 Scenario、輪含被推翻立場、讀量近倍增、sources 契約綁 capability；規格抽屜改吃 speclink trace 樹——需新 desktop query 且 server 無 trace 端點、remote 做不到；保留 footer 只加連結——與另外兩個抽屜標頭文法不一致；已封存討論加第四區段「衍生變更」——子變更幾乎全為已封存、「現況」欄無資訊量且與正典區段清單字面衝突；出身列標籤用「溯源」「衍生變更」——出身列文法為兩字關係詞，「溯源」留給頁面與指令名。
**Deferred**: 逐條需求旁標「來自哪個變更」（trace JSON 的 requirements[].source 已可支撐）；同名多份封存目錄的解析規則（目前無重名、正典沉默）；規格抽屜「最近更新」日期；若真需要手冊脈絡，缺口補在規格 Purpose 而非手冊來源。
**Capture to**: proposal（單一變更，建議名 drawer-provenance-links）、spec（desktop-app 兩條 MODIFIED：「桌面 app 呈現 change 與 spec 的清單與內容」「已封存項目以抽屜檢視」）、LANGUAGE.md（「衍生變更」詞條補出身列標籤「衍生」的用法註記）
**Next**: /speclink-propose --from-discussion spec-drawer-trace-links
