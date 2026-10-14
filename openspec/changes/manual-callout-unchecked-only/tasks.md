## 1. 手動提示框只列未勾選的手動任務

- [ ] 1.1 先寫紅燈（spec「任務群組標題與章節標籤同款式」的「數字磚與手動提示框」「部分勾選時提示框只列未勾選的手動任務」）：`packages/ui/src/__tests__/changeDetailPage.test.tsx` 中驗任務文件數字磚與手動提示框的那一組。改：既有「任務文件：數字磚 9／7／1／3 與手動提示框列出 3.2；全部勾選後籤槽為「全部完成」」題的全部勾選段落，加驗數字磚仍含「手動 1」且頁面上沒有 `[data-doc-callout]` 提示框。加一題「部分勾選時提示框只列未勾選的手動任務」：tasks.md 有 4 條帶 `[M]` 的任務 4.3（已勾）、5.8、6.9、7.6（未勾），分在不同群組並各夾一條一般任務；初次渲染時提示框標題為「3 個手動任務」、`[data-manual-task]` 恰三列且序號依序為 5.8、6.9、7.6、不含 4.3，數字磚為「手動 4」；接著以 TaskList 收到的 `onToggle`（`onToggleTask` resolve）依序勾 5.8、6.9、7.6，三條都勾完後提示框缺席、頁面沒有任何其他提示框、數字磚仍為「手動 4」；再取消勾選 6.9，提示框重新出現、標題「1 個手動任務」且只列 6.9。加一題「勾選寫回失敗時提示框跟著回滾」：同一份 tasks.md，`onToggleTask` reject，勾 5.8 後寫回失敗，提示框回到「3 個手動任務」並列出 5.8、6.9、7.6。驗證：1.2 實作前，`npm test -w packages/ui -- changeDetailPage` 中上述斷言失敗，同檔其餘題目照綠。 <!-- speclink-task:tsk_01M4W43E0Y73RTG0095XH0DXP7 -->
- [ ] 1.2 讓 1.1 轉綠：`packages/ui/src/components/changeDocSummary.tsx` 的 `tasksSummary` 讓手動提示框的標題 N 與列表只取未勾選的 `[M]` 任務；沒有未勾選的 `[M]` 任務時提示框為 null（不產生替代提示）；「手動」數字磚維持計所有 `[M]` 任務。每列的序號、標題的等寬籤與「前往」鈕照舊。變更詳情頁與已封存詳情頁共用此函式，一起生效。驗證：`npm test -w packages/ui -- changeDetailPage archivedDetailPage proposalDoc` 全綠。 <!-- speclink-task:tsk_01M4W43E0YT6F0XW6QJHVGF9RE -->

## 2. 收尾驗證

- [ ] [M] 2.1 開發環境實機驗收：以 root `npm run dev:desktop` 開桌面視窗，打開一份有未勾手動任務的進行中變更的任務文件，確認提示框只列未勾的手動任務；點整列勾選其中一條，確認它當下從提示框消失、標題的數字減一、「手動」數字磚不變；把手動任務全部勾完，確認提示框整個消失；最後把勾選恢復原狀。 <!-- speclink-task:tsk_01M4W43E0Y1QB7RWB8DCTHJRHJ -->
- [ ] 2.2 全組守門：確認本變更沒有動到詞彙守門與文件連結掃描面。驗證：`npm test -w packages/ui` 全綠，且 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。 <!-- speclink-task:tsk_01M4W43E0YYE6DRSPNVG2A8HGC -->
