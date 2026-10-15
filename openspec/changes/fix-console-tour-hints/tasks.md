## 1. 重現（Red）

- [ ] 1.1 在 `apps/server-web/src/__tests__/tour.test.tsx` 新增回歸測試：讀取 `apps/server-web/src/i18n/messages.ts` 匯出的繁中與英文訊息，對每個 `tour.*.hint` 鍵斷言不含「右側面板」「滑出」「drawer」「slides」（不分大小寫），失敗訊息點名違規的語言與鍵；並斷言 `tour.navUsers.hint` 提到詳情頁（繁中「詳情頁」、英文「detail page」）、`tour.listPrimary.hint` 提到對話框（繁中「對話框」、英文「dialog」），對應「首次進入提供可略過的分步導覽」的新情境。驗證：`npm test -w apps/server-web` 時新測試因現行兩個鍵含「右側面板」與「drawer」而失敗。 <!-- speclink-task:tsk_01M4Z3AHAC4B8XRZHDN0BAHP2R -->

## 2. 修正（Green）

- [ ] 2.1 改寫 `apps/server-web/src/i18n/messages.ts` 中 `tour.navUsers.hint` 與 `tour.listPrimary.hint` 的繁中與英文字串：「使用者」說明點一列進入詳情頁編輯、邀請在置中的對話框完成、列上不放輸入控制項；「建立與編輯」說明每個列表頁只有一個主要動作、按下去開啟置中的對話框、不離開這一頁。繁中用 `openspec/LANGUAGE.md` 的正典詞（詳情頁），英文用簡短主動句。驗證：`npm test -w apps/server-web` 全部通過，含 1.1 的新測試與既有導覽測試。 <!-- speclink-task:tsk_01M4Z3AHACB586P3K7TVNVEHXR -->
- [ ] 2.2 排查同類過時字眼：在 `apps/server-web/src/i18n/messages.ts`、`apps/desktop/src/i18n/messages.ts` 與 `packages/ui/src/i18n.tsx` 的使用者可見字串中搜尋「右側面板」「滑出」「抽屜」「drawer」「slides」（只看字串值，不看 `drawer.*` 這類訊息鍵名），逐一列出命中處並註明修正或不修的理由。驗證：任務記錄列出搜尋寫法與每個命中的處理；目前已知只有本變更的兩個鍵。 <!-- speclink-task:tsk_01M4Z3AHACDVFTCDZSYT47M6RQ -->

## 3. 收尾

- [ ] 3.1 跑 repo 腳本測試，確認詞彙守門與文件測試仍通過：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束。 <!-- speclink-task:tsk_01M4Z3AHACRNEMX51E6DQ8NNM8 -->
