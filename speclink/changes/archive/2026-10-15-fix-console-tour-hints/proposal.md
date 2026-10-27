## Problem

Server 後台的首次導覽有兩步的說明描述了已經不存在的介面：

- 「使用者」那一步說「邀請與編輯都在右側面板完成」（英文：「inviting and editing happen in the right-hand drawer」）。
- 「建立與編輯」那一步說主要動作「會從右側滑出面板」（英文：「it slides a drawer in from the right」）。

現在的後台是：點一列進入該筆的詳情頁編輯，「邀請使用者」與「新增專案」開啟置中的對話框。第一次進後台的管理員會照著導覽找一個不存在的右側面板。2026-10-15 重拍文件截圖時由稽核發現。

## Root Cause

server-web-detail-pages 變更把列表的檢視與建立從右側滑出面板改成詳情頁與置中對話框，但沒有一起更新 `apps/server-web/src/i18n/messages.ts` 裡導覽說明的兩個訊息鍵 `tour.navUsers.hint` 與 `tour.listPrimary.hint`（繁中與英文各一份）。導覽的測試只檢查步驟的啟動、略過與跳過，不檢查說明文字是否符合畫面，所以沒有攔下來。

## Proposed Solution

- 改寫這兩個訊息鍵的繁中與英文說明，並把第二步的標題 `tour.listPrimary.title` 從「建立與編輯」改為「建立」（英文 Create）——編輯已移到詳情頁，這一步指的按鈕只負責建立：
  - 「使用者」：列表是主體；點使用者那一列進入詳情頁編輯（「邀請中」分頁的列不能點），邀請在置中的對話框完成；列上不放輸入控制項。
  - 「建立」：這一頁只有一個主要動作，按下去開啟置中的對話框，不離開這一頁。不寫「每個列表頁」：這一步只出現在帶目標元素的使用者頁與專案頁，憑證頁的主要動作是連到帳號頁。
- 在 `apps/server-web/src/__tests__/tour.test.tsx` 加一條回歸測試：兩種語言的全部導覽說明都不出現「右側面板」「滑出」「抽屜」「drawer」「slides」等已不存在的介面字眼，且每種語言都篩得出七個說明鍵（防止鍵名改了之後比對空轉）。
- 正式規格「首次進入提供可略過的分步導覽」補一個情境：說明描述的互動方式必須與畫面上實際的互動方式一致。

## Non-Goals

- 不改導覽的步驟順序、目標元素、啟動與略過邏輯。
- 不改桌面 app 的任何文案（桌面沒有這段導覽）。
- 不改其他導覽步驟的說明（總覽、專案與儲存庫、憑證、系統、稽核紀錄五步經比對仍與畫面一致）。

## Success Criteria

- 開啟後台導覽，「使用者」與「建立」（原「建立與編輯」）兩步的說明在兩種語言都描述詳情頁與置中對話框，不再提到右側面板。
- `npm test -w apps/server-web` 全部通過，含新的回歸測試；把舊的「右側面板」字串放回訊息檔時，新測試會失敗並點名該訊息鍵。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（詞彙守門會掃 `apps/server-web/src/i18n/messages.ts`）。

## Impact

- Affected specs: `server-web-console`（「首次進入提供可略過的分步導覽」補說明須符合現行介面的情境）
- Affected apps: `apps/server-web`（只改訊息檔與測試）；crate、CLI 與桌面 app 不受影響。沒有 CLI、`--json` 或設定欄位的改動。
- Affected code:
  - Modified:
    - `apps/server-web/src/i18n/messages.ts`
    - `apps/server-web/src/__tests__/tour.test.tsx`
