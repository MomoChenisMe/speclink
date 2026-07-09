## 1. 動詞結果進抽屜（validate）

- [x] 1.1 [Red] 為「桌面 app 提供動詞操作面」寫失敗測試（apps/desktop store.test 與 packages/ui richDrawer.test），實現 D1：validate／analyze 結果呈現於詳情抽屜，頂列保留全域操作——store.runVerb 對 validate 保留結構化結果、RichDetailDrawer 於動作列近處呈通過／失敗（失敗附首則錯誤），頂列僅呈全域操作。驗證：`npm test -w apps/desktop`、`npm test -w packages/ui` 見紅。
- [x] 1.2 [Green] store.runVerb 對 validate 保留 {valid, errors} 並經 props 傳 RichDetailDrawer 於抽屜內呈現，change 動詞結果不再寫入頂列 verbResult（頂列保留刪除／封存／拖排失敗）。令 1.1 轉綠。驗證：`npm test -w apps/desktop`、`npm test -w packages/ui`。

## 2. analyze 四維度面板

- [x] 2.1 [Red] 寫失敗測試 packages/ui/src/__tests__/analyzePanel.test.tsx，實現 D2：analyze 沿用引擎回傳的 AnalyzeReport 渲染四維度面板，不新增 IPC——依 Coverage／Consistency／Ambiguity／Gaps 呈各維度發現數與逐條發現項（嚴重度＋訊息）。驗證：`npm test -w packages/ui` 見紅。
- [x] 2.2 [Green] 新增 packages/ui/src/components/AnalyzePanel.tsx 讀 store 保留的 AnalyzeReport.findings 分維度渲染，RichDetailDrawer 於 analyze 後在抽屜內呈該面板，使「桌面 app 提供動詞操作面」的 analyze 情境成立。令 2.1 轉綠。驗證：`npm test -w packages/ui`。

## 3. 撤除 promote

- [x] 3.1 [Red] 為「討論抽屜檢視與轉出變更」與「討論於看板第 0 欄兩級呈現」寫失敗測試（discussionColumn.test、discussionDrawer.test），實現 D3：撤除 desktop promote 動作，衍生變更維持唯讀——concluded 討論卡無「轉為變更」鈕、討論抽屜衍生變更分頁無「轉為變更／再轉出」鈕但仍列子變更與跳轉。驗證：`npm test -w packages/ui` 見紅。
- [x] 3.2 [Green] 移除 DiscussionColumn concluded 卡的轉為變更鈕、DiscussionDrawer promote pane 的轉出鈕與 onPromote props，apps/desktop store 移除 promote 動作與 promoteError，i18n 清理相關鍵。令 3.1 轉綠。驗證：`npm test -w packages/ui`、`npm test -w apps/desktop`。

## 4. 重構與回歸

- [x] 4.1 [Refactor] 檢視抽屜結果資料流與 promote 移除：確認未觸及 core／analyzer／IPC、頂列全域操作行為不變、無孤兒 imports／props，並套用 sharp-edges 稽核確認 analyze findings 於邊界反序列化安全。驗證：`npm test -w packages/ui`、`npm test -w apps/desktop` 全綠，且 `npm run build -w apps/desktop` 通過。
