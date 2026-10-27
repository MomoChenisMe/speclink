## 1. D1：分析面板兩層結構——結構驗證列＋維度摘要卡＋發現卡

- [x] 1.1 重塑動詞結果型別與 store 動作：VerbDrawerResult 改為 { change, validate?, analyze?, error? }（移除 verb 欄位），store 的分析動作單鍵併發執行 validate 與 analyze 並合併為單一結果——先於 apps/desktop/src/__tests__/store.test.ts 寫失敗測試（點分析後 drawerVerb 同時含 validate 與 analyze 結果；任一動詞失敗時 error 呈現且不靜默），實作至綠。涵蓋「桌面 app 提供動詞操作面」的一鍵雙動詞語意。
- [x] 1.2 重構 AnalyzePanel 為兩層結構：頂部結構驗證列（通過單列／失敗逐條列錯）、四張繁體中文維度摘要卡（覆蓋度、一致性、模糊度、缺漏；零發現呈「無問題」成功語意、非零呈「N 個問題」警示語意）、逐條發現卡（嚴重度徽章＋location＋summary＋recommendation 建議行）——先於 packages/ui/src/__tests__/analyzePanel.test.tsx 寫失敗測試（維度名繁中、零與非零的配色語意、發現卡含來源檔與建議行、驗證失敗逐條呈現），實作至綠。
- [x] 1.3 RichDetailDrawer 動作列移除獨立「驗證」按鈕，僅留分析、封存、刪除——packages/ui/src/__tests__/richDrawer.test.tsx 斷言動作列按鈕集合；清理 i18n 中不再使用的驗證鈕文案。

## 2. D2：動詞結果可關閉——分析鈕切換＋面板關閉鈕＋store 清空動作

- [x] 2.1 store 新增 clearDrawerVerb 動作——apps/desktop/src/__tests__/store.test.ts 斷言清空後 drawerVerb 為 null，且既有「換 change 清空」「關抽屜清空」行為不回歸。
- [x] 2.2 RichDetailDrawer 分析鈕改切換行為（aria-pressed；結果開啟時再點按收合）＋分析面板右上關閉鈕——packages/ui/src/__tests__/richDrawer.test.tsx 斷言「點分析展開 → 再點分析收合 → 再點重新執行並展開」與「點面板關閉鈕收合」。涵蓋「桌面 app 提供動詞操作面」的可收合行為。

## 3. D3：已轉出討論改欄底常駐收合列

- [x] 3.1 DiscussionColumn 移除互斥檢視（showPromoted 狀態、header ↗N 開關、欄標題切換），改為欄底「已轉出 N」常駐收合列（預設收合、點按就地展開細列、active 全卡維持可見、計數徽章恆顯 active 數、無 promoted 時收合列缺席、僅 promoted 時不顯空狀態）——先於 packages/ui/src/__tests__/discussionColumn.test.tsx 改寫為失敗測試再實作至綠。涵蓋「討論於看板第 0 欄兩級呈現」的收合列語意。
- [x] 3.2 清理互斥檢視遺留：移除不再使用的 i18n 鍵（顯示已轉出開關與已轉出欄標題文案）並同步修正引用該行為的既有測試斷言（packages/ui/src/__tests__/kanban.test.tsx 等），npm test -w packages/ui 全綠。

## 4. D4：slug 識別擴至 promoted 細列與討論抽屜標題

- [x] 4.1 PromotedRow 首行改 slug（等寬字型）＋複製 slug 鈕（沿用 copied 回饋模式）、topic 降為次行描述，衍生樹與階段 chip 不動——packages/ui/src/__tests__/discussionColumn.test.tsx 斷言細列首行為 slug、點複製鈕寫入剪貼簿。涵蓋「討論於看板第 0 欄兩級呈現」的細列 slug 錨點。
- [x] 4.2 DiscussionDrawer 標題改 slug＋複製鈕、topic 降為副標——packages/ui/src/__tests__/discussionDrawer.test.tsx 斷言標題呈 slug、副標呈 topic、複製鈕寫入剪貼簿。涵蓋「討論抽屜檢視與轉出變更」的標題錨點。
- [x] 4.3 擴充 openspec/LANGUAGE.md 的 slug 受控例外條目：適用範圍由「僅限 discuss 卡標題與其複製鈕」改為「僅限討論識別錨點（討論全卡標題、已轉出細列首行、討論抽屜標題）與其複製鈕」，註記出處 desktop-ux-polish；以 speclink language show 確認條目可讀。
- [x] 4.4 PromotedRow 複製鈕改行內尾隨（ingest 自討論 spec-archive-drawer-ux 的複製鈕位置規則）：按鈕直接跟在 slug 最後一個字元後流動（break-all 多行時位於末行文字尾），不再以 flex-1 推至列右緣；hover 顯現與 copied 回饋不變；DiscussionDrawer 標題（4.2）已合規、不動——先於 packages/ui/src/__tests__/discussionColumn.test.tsx 加失敗斷言（複製鈕為 slug 文字的行內後隨元素而非右緣定位）再實作至綠，npm test -w packages/ui 全綠。

- [x] 5.1 新增 BoardSearchBar 元件：搜尋圖示、輸入非空時的清除鈕（清空後保持聚焦）與即時命中數、Cmd+F（macOS）／Ctrl+F（其他平台）聚焦快捷鍵——先於 packages/ui/src/__tests__/kanban.test.tsx（或新增 boardSearchBar.test.tsx）寫失敗測試再實作至綠，KanbanBoard 以該元件取代裸 Input。涵蓋「看板搜尋過濾卡片」的搜尋列呈現。
- [x] 5.2 篩選 chips（建立者、建立時間近 7 天／近 30 天／更早、來源討論）：過濾規則以純函式收斂於 packages/ui/src/search.ts（單元測試：各維度命中、AND 交集、單獨清除還原），chips UI 於 BoardSearchBar 呈現（未啟用中性、啟用 teal 實心帶所選值與清除 ×）。
- [x] 5.3 desktop-core 變更清單補 created 日期欄位（取 .openspec.yaml 的 created）：apps/desktop/core/src/query.rs 疊加欄位＋cargo test -p speclink-desktop-core 斷言 payload 含 created；packages/ui/src/adapter.ts 的 ChangeItem 增 created 欄位。

## 6. D6：全文搜尋走桌面 core 單一查詢命令

- [x] 6.1 新增 apps/desktop/core/src/search.rs：search_workspace 遍歷 active 變更 artifacts（提案、設計、任務、delta 規格）與 active 討論記錄，不分大小寫子字串比對，回傳 [{ kind, id, artifact, snippet }]（每卡首個命中、snippet 為命中前後約 30 字元裁切）——cargo test -p speclink-desktop-core 單元測試以 testfixture 建立變更與討論，斷言命中、未命中、大小寫不敏感、snippet 含命中原文。
- [x] 6.2 IPC 與 adapter 接線：src-tauri 新增 search_workspace 命令單行委派、packages/ui/src/adapter.ts 的 DataSource 介面增 searchWorkspace 方法、apps/desktop/src/adapter/tauriDataSource.ts 實作 invoke 呼叫——apps/desktop/src/__tests__/store.test.ts 以假 dataSource 斷言方法簽名與回傳形狀。
- [x] 6.3 前端全文查詢接線：query 非空時 200ms 去抖觸發、latest-wins 序號防交錯、全文命中併入可見集合（欄位命中 OR 全文命中，再與篩選 AND）、IPC 失敗靜默退回欄位比對——store 或 App 測試斷言去抖後併入與失敗退回。涵蓋「看板搜尋過濾卡片」的全文比對層。

## 7. D7：模糊比對限名稱層、命中高亮與 snippet

- [x] 7.1 packages/ui/src/search.ts 新增 subsequence 模糊比對純函式並限定套用於變更卡名稱與討論卡 slug——單元測試：etc 命中 engine-typed-core、dta 命中 desktop-acp-agent、摘要與主題不套用模糊層。涵蓋「看板搜尋過濾卡片」的名稱層模糊比對。
- [x] 7.2 命中高亮與 snippet 呈現：欄位子字串命中於卡片高亮命中原文（mark 樣式）、僅模糊命中不高亮、全文命中卡片於卡身呈 snippet 行（artifact 名＋裁切前後文＋命中高亮）——packages/ui/src/__tests__/kanban.test.tsx 斷言三種呈現。

## 8. D8：封存落點浮層化

- [x] 8.1 ArchiveDropZone 改絕對定位浮層（看板欄列容器 relative、落點 absolute 疊於右緣、不參與 flex），浮現條件收斂為「拖曳中且 active 卡為變更卡」——packages/ui/src/__tests__/kanban.test.tsx 斷言拖曳討論卡時不渲染落點、拖曳變更卡時渲染且落點不在欄列 flex 流內；useDroppable id 與封存確認流程不變。涵蓋「拖曳封存落點以浮層呈現」。
- [x] 8.2 真實視窗手動驗證（依 CLAUDE.md GUI 備忘，jsdom 測不出拖曳互動）：拖曳變更卡時四欄寬度零變動、拖至浮層放開觸發封存確認、拖討論卡無落點、Cmd+F 聚焦搜尋、分析面板開合與關閉鈕——驗證結果記錄於 commit 訊息或工作紀錄。

## 9. 回歸與收尾

- [x] 9.1 全量回歸：npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-desktop-core 全綠，npm run build -w apps/desktop 建置成功；確認未動引擎 CLI 輸出（cargo test -p speclink-cli 全綠）。
