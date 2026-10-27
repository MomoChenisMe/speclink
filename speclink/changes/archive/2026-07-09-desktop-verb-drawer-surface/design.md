## Context

RichDetailDrawer 的動作列已有 analyze／validate 按鈕，接 onRunVerb → store.runVerb → 同名 Tauri 動詞（確定性引擎）。但 store.runVerb 以 formatVerbResult 把結果壓成一行字串寫入 verbResult，App 僅在視窗頂列渲染該字串；analyze 的 AnalyzeReport.findings 只被數 length 就丟棄。engine 的 analyzer 已產出 Coverage／Consistency／Ambiguity／Gaps 四維度發現項。另一面，desktop 於討論卡（concluded）與討論抽屜衍生變更分頁提供 promote 動作，但 promote 產出需 LLM 補完的 stub、與 GUI 既有「不提供 conclude／add-round」不一致。承 discuss desktop-sdd-verb-scope 結論。

## Goals / Non-Goals

**Goals:**

- validate／analyze 結果在抽屜內、近動作處人性化呈現；analyze 呈四維度富面板。
- desktop 動詞面收斂為「檢視器＋自足確定性動詞」，撤除產出 stub 的 promote。

**Non-Goals:**

- 不改 core／analyzer／IPC（沿用既有回傳的 AnalyzeReport 與 validate 結果）。
- 不移除視窗頂列狀態列——其保留供看板全域操作（刪除／封存／拖排失敗）。
- 不移除討論抽屜的衍生變更檢視與已轉出分組（維持唯讀、列子變更與跳轉）；僅撤「轉為變更／再轉出」動作。
- 不改變 discuss 的轉出能力本身（CLI／agent 仍可 promote）。

## Decisions

### D1：validate／analyze 結果呈現於詳情抽屜，頂列保留全域操作

change 動詞（validate／analyze）結果改由 store 保留結構化結果、經 props 傳入 RichDetailDrawer 於抽屜內呈現：validate 於動作列近處通過／失敗（失敗附首則錯誤），analyze 呈四維度面板。視窗頂列 verbResult 狀態列保留給看板全域操作（刪除／封存／拖排失敗）。

- 替代：維持結果只在頂列一行——否決：離抽屜動作太遠、使用者實測感覺沒反應，且丟棄 analyze 結構。

### D2：analyze 沿用引擎回傳的 AnalyzeReport 渲染四維度面板，不新增 IPC

新增 AnalyzePanel 元件，讀 store 保留的 AnalyzeReport.findings，依 Coverage／Consistency／Ambiguity／Gaps 分組呈各維度發現數與逐條發現項（嚴重度＋訊息）。資料已隨既有 Tauri analyze 回傳到前端，僅停止 stringify、改保留結構。

- 替代：於 core 另做面板專用彙整——否決：findings 已足夠，重複彙整屬過度設計。

### D3：撤除 desktop promote 動作，衍生變更維持唯讀

移除討論卡（concluded）的「轉為變更」鈕、討論抽屜衍生變更分頁的「轉為變更／再轉出一個變更」鈕，並移除 store 的 promote 動作與確認流程；衍生變更分頁與已轉出分組維持唯讀（列子變更與跳轉）。promote 併入 GUI 不提供的寫入動詞清單。

- 替代：保留 promote——否決：其輸出是 LLM 才能補完的 stub，與 GUI 既有 no-LLM-write 線不一致。

## Implementation Contract

- 行為：詳情抽屜內觸發 validate 於動作列近處呈通過／失敗（失敗顯示首則錯誤）；觸發 analyze 呈 Coverage／Consistency／Ambiguity／Gaps 四維度面板，各維度顯示發現數與逐條發現項（嚴重度＋訊息，對應 speclink analyze 的 --json）。視窗頂列僅呈看板全域操作結果。討論卡（concluded）不再有「轉為變更」鈕；討論抽屜衍生變更分頁無「轉為變更／再轉出一個變更」鈕，僅列子變更現況與跳轉。GUI 不提供 promote。
- 介面／資料形狀：store.runVerb 對 validate／analyze 保留結構化結果（validate: valid 與 errors；analyze: AnalyzeReport 的 findings，含 dimension／severity／message），以 props 傳入 RichDetailDrawer；新增 AnalyzePanel 元件。DiscussionColumn 與 DiscussionDrawer 移除 onPromote 相關 props／按鈕；store 移除 promote 動作與 promoteError。i18n 增／減對應鍵。不新增 Tauri command、不改 --json 契約。
- 失敗模式：validate 失敗於抽屜顯示錯誤訊息；analyze 執行失敗於抽屜顯示錯誤；無 promote 路徑。
- 驗收：packages/ui 測試（analyzePanel 呈四維度與發現項、richDrawer 於抽屜內呈 validate 結果、discussionColumn／discussionDrawer 無 promote 按鈕）；apps/desktop store 測試（runVerb 保留結構化結果、無 promote 動作）。驗證：`npm test -w packages/ui` 與 `npm test -w apps/desktop` 全綠。
- 範圍邊界：in scope＝抽屜內 validate／analyze 結果呈現、AnalyzePanel、store 結構化結果資料流、promote 自 column／drawer／store 撤除、i18n。out of scope＝core／analyzer／IPC、頂列全域操作行為、discuss 的轉出能力本身。

## Risks / Trade-offs

- [與 promoted-discussion-toggle 重疊「討論於看板第 0 欄兩級呈現」] → 本變更僅淨移除 concluded 卡的「轉為變更」動詞，其「討論欄」delta 依現行基線撰寫（不含開關／chip 改動）；promoted-discussion-toggle 另引入 header 開關與 chip 配色。兩者同修此需求，apply 時 SHALL 對後套用者跑 drift 對齊，避免其一的全需求重現覆蓋另一的改動。
- [analyze findings 序列化欄位是否齊備] → 緩解：驗證既有 Tauri analyze command 回傳含 dimension／severity／message；若不足屬既有 IPC 契約另案處理（本變更不改 IPC）。
