## 0. 前提

- [x] 0.1 確認 desktop-discussion-board 已完成歸檔——speclink list --json 的 changes 不含 desktop-discussion-board，且正典 openspec/specs/desktop-app/spec.md 含「討論於看板第 0 欄兩級呈現」等三條討論需求（本刀 MODIFIED delta 的基底）。未歸檔則停止並回報。驗證：兩項檢查皆成立。

## 1. 看板討論欄（design D1 詞彙替換以 LANGUAGE.md 為單一對照；design D2 已轉出細列改衍生樹）

- [x] 1.1 紅：更新 packages/ui/src/__tests__/discussionColumn.test.tsx，涵蓋 spec 需求「討論於看板第 0 欄兩級呈現」的新斷言——concluded 卡按鈕為「轉為變更」「封存」且「促轉」「歸檔」不再出現、卡片輪數文案「N 輪」、群組標題「已轉出變更的討論」、細列首行為討論 topic 且 slug 不出現於看板、子變更以 ├／└ 樹狀前綴逐列列出並帶階段標示（spec Example「chip 階段派生矩陣」值沿用、discussionChipStage 斷言不動）。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 1.2 綠：修改 packages/ui/src/components/DiscussionColumn.tsx——動詞按鈕與 aria-label 換詞、群組標題換名、PromotedRow 改衍生樹（topic 首行、樹狀子項列、名稱過長 truncate 而樹狀前綴不截斷）。驗證：1.1 測試全綠、既有看板測試不破。

## 2. 討論抽屜與封存展開（design D3 抽屜生命週期階梯與預設分頁）

- [x] 2.1 紅：更新 packages/ui/src/__tests__/discussionDrawer.test.tsx 與 archivedList.test.tsx，涵蓋 spec 需求「討論抽屜檢視與 GUI 促轉」（本刀 RENAMED 為「討論抽屜檢視與轉出變更」）與「已封存頁含討論節」——抽屜分頁依序為 結論／討論過程 N／背景／衍生變更、結論區段非空時預設呈現結論內容（spec Scenario「有結論的討論預設開啟結論分頁」）、結論為空時預設背景、標題下方階梯含「討論中」「已結論」「轉出變更」三站且現站可辨、衍生變更分頁按鈕文字未轉出為「轉為變更」／已轉出為「再轉出一個變更」、切分失敗仍整篇單一檢視退回、封存頁討論節展開的區段標題為「背景」「討論過程」「結論」。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 2.2 綠：修改 packages/ui/src/components/DiscussionDrawer.tsx（分頁序與預設分頁邏輯、階梯列、按鈕文字）與 ArchivedList.tsx（討論節區段標題換詞）。驗證：2.1 測試全綠、npm test -w packages/ui 全綠。

## 3. 確認框（design D4 確認框文案使用者語言化）

- [x] 3.1 紅：更新 apps/desktop/src/__tests__/App.test.tsx——轉為變更確認框標題「轉為變更？」、說明含「提案中」與「結論」字樣且不含「from_discussion」「kebab-case」「proposal」「meta」、名稱輸入 label「變更名稱」與說明「英文小寫，字間用 -」、主按鈕「轉為變更」觸發既有 confirmPromote 流；封存討論確認框標題「封存討論？」與使用者語言說明。驗證：npm test -w apps/desktop 出現預期紅燈。
- [x] 3.2 綠：修改 apps/desktop/src/App.tsx 兩個確認框文案與按鈕。驗證：3.1 測試全綠、npm test -w apps/desktop 全綠。

## 4. 整合驗證

- [x] 4.1 全套自動化：npm test -w packages/ui 與 -w apps/desktop 全綠；以 grep 檢查 packages/ui/src 與 apps/desktop/src 的使用者可見文案（JSX 字串與 aria-label）無「促轉」「回合」「脈絡」殘留；git diff 確認 crates/ 零變更（CLI 位元級不變的結構性證據）。驗證：全部成立。
- [x] 4.2 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe，並先 npm run build -w apps/desktop；操作前確認使用者沒在使用螢幕）：三畫面與討論記錄的 mockup 一致——討論欄（轉為變更／封存按鈕、已轉出變更的討論群組、衍生樹細列顯示 topic）、抽屜（階梯現站、分頁序、有結論預設開結論）、轉為變更確認框（使用者語言文案與名稱說明）；舊詞（促轉／已促轉／歸檔按鈕／N 回合／脈絡）不出現於討論 UI。驗證：每畫面有截圖記錄，行為與 specs/desktop-app/spec.md 各 Scenario 一致。
