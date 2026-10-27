## 1. 章節檢視元件：提案與設計章節以中文標籤呈現（design D1 章節切分：頂層標題白名單映射，未知照排；D2 對照表與標籤文案：i18n 收斂、涵蓋三型提案模板與設計模板）

- [x] 1.1 撰寫章節切分與渲染的失敗測試（packages/ui/src/__tests__/components.test.tsx）：含 Why／What Changes／Non-Goals／Capabilities／Impact 的提案文件渲染出「為什麼／變更內容／非目標／能力／影響」標籤且英文標題不直出；含 Context／Decisions／Risks / Trade-offs 的設計文件同理（背景／決策／風險與取捨）；白名單外標題（自訂決策標題）照 prose 排；「Non-Goals (optional)」形式命中「非目標」；無任何白名單章節整篇退回（無標籤區塊）。驗證：npm test -w packages/ui 紅燈。
- [x] 1.2 實作 SectionedDoc 元件（packages/ui/src/components/SectionedDoc.tsx：行掃描切頂層標題、白名單→i18n key 對照表、命中成標籤區塊＋內文 prose、未知併入 prose 段、零命中整篇退回；packages/ui/src/i18n.tsx 補章節標籤 key，zh-TW 中文、en 原文）。驗證：1.1 全數轉綠。

## 2. 接線：變更抽屜與已封存檢視（design D3 接線：RichDetailDrawer 與 ArchivedList 的提案／設計分頁換用 SectionedDoc）

- [x] 2.1 撰寫接線的失敗測試（packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx）：變更抽屜提案分頁呈現「為什麼」標籤且 Why 不直出；已封存變更展開的提案分頁同型斷言；規格分頁色標與討論側標籤既有測試維持全綠（互不相擾）。驗證：npm test -w packages/ui 紅燈。
- [x] 2.2 實作接線：RichDetailDrawer 與 ArchivedList 的提案／設計 TabsContent 換用 SectionedDoc（packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx，empty 文案照舊）——行為即規格「提案與設計章節以中文標籤呈現」。驗證：2.1 全數轉綠。

## 3. 任務群組標題與章節標籤同款式（design D4 任務群組標題款式對齊）

- [x] 3.1 撰寫失敗測試（packages/ui/src/__tests__/taskList.test.tsx）：群組標題元素帶章節標籤同款式 class（小字 semibold uppercase tracking-wider muted）、不再是 text-base font-bold；標題文字照來源；既有勾選與拖曳測試維持全綠。驗證：npm test -w packages/ui 紅燈。
- [x] 3.2 實作 TaskList 群組標題款式（packages/ui/src/components/TaskList.tsx 的群組標題元件，僅動 className、拖曳讓位行為不動）——行為即規格「任務群組標題與章節標籤同款式」。驗證：3.1 轉綠。

## 4. 收尾驗證（design D5 測試策略：jsdom 結構驗證，視覺以真實視窗驗收）

- [x] 4.1 全量迴歸：npm test -w packages/ui 與 npm test -w apps/desktop 全綠；speclink list --json 抽查形狀不變（本刀純前端、CLI 零變更）。
- [x] 4.2 真實視窗驗收：npm run build -w apps/desktop 後 cargo build --release -p speclink-desktop（先關閉執行中 exe），開 release exe 並排核對——變更抽屜提案／設計分頁的中文標籤區塊與討論抽屜結論欄位款式一致、任務分頁群組標題同款、英文模板標題不再出現；操作前確認使用者未在使用螢幕。驗證：截圖人工核對三項皆符合。

## 5. 大標題款式（design D6 標籤款式：粗體大標題、單一常數全面套用）

- [x] 5.1 撰寫大標題款式的失敗測試（packages/ui/src/__tests__/components.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx）：六處標籤（SectionedDoc 章節、TaskList 群組、RoundsView 輪欄位、ConclusionView 結論欄位、DeltaSpecView 色標標頭、ArchivedList 討論區段標題）className 引用同一款式常數且含粗體大標題 class（text-xl font-bold）、不再含 text-xs／uppercase／tracking-wider；色標標頭仍帶各 delta 色彩 class。驗證：npm test -w packages/ui 紅燈。
- [x] 5.2 實作大標題款式：SectionedDoc 匯出共用款式常數，六處替換引用（packages/ui/src/components/SectionedDoc.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/components/ArchivedList.tsx）；Capabilities 次級標籤用略小一級同族款式（text-base font-bold）——行為即規格「標籤為大標題且字級大於內文」。驗證：5.1 全數轉綠且全套件無回歸。
- [x] 5.3 迴歸與真實視窗補驗：npm test -w packages/ui 與 npm test -w apps/desktop 全綠；重建前端與 release exe，開變更抽屜提案分頁與討論抽屜結論分頁截圖核對大標題款式一致、規格分頁色標保色；操作前確認使用者未在使用螢幕。驗證：截圖人工核對兩項皆符合。

## 6. 任務群組標題次級款（design D6 標籤款式的次級層：使用者第二次比對裁定，Spectra 任務清單原尺寸）

- [x] 6.1 改寫任務群組標題款式的失敗測試（packages/ui/src/__tests__/taskList.test.tsx）：群組標題 className 引用次級款常數（text-base font-bold、與 Capabilities 次級標籤同源）、不再含 text-xl；文字照來源。驗證：npm test -w packages/ui 紅燈。
- [x] 6.2 實作任務群組標題次級款（packages/ui/src/components/TaskList.tsx 改引 SUB_LABEL_CLS）——行為即規格「任務群組標題與章節標籤同款式」（次級層定義）。驗證：6.1 轉綠、全套件無回歸；重建前端與 release exe 後開任務分頁截圖核對群組標題與任務文字同級、小於章節主標題（操作前確認使用者未在使用螢幕）。
