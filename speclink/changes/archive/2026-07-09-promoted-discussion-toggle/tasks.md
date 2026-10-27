## 1. header 開關與 promoted 隱藏

- [x] 1.1 [Red] 於 packages/ui/src/__tests__/discussionColumn.test.tsx 寫失敗測試，實現 D1：header 開關取代欄底預設展開群組——存在 promoted 時 header 呈帶計數的 ↗ 開關且 promoted 預設隱藏、點按切換顯示欄底衍生樹，0 promoted 時開關缺席。驗證：`npm test -w packages/ui` 見紅。
- [x] 1.2 [Green] DiscussionColumn 以元件內 local 狀態實作 header「顯示已轉出」開關（ArrowUpRight＋promoted 計數），預設關閉、關閉零佔位、0 promoted 不渲染開關；packages/ui/src/i18n.tsx 新增開關 aria-label／tooltip。令 1.1 轉綠。驗證：`npm test -w packages/ui`。

## 2. 計數與空狀態

- [x] 2.1 [Red] 為「討論於看板第 0 欄兩級呈現」寫失敗測試，實現 D3：討論欄計數只算 active，promoted 計數移至開關——欄計數徽章只反映 open＋concluded 數，且無 active 但有 promoted 時欄體不顯「尚無討論」。驗證：見紅。
- [x] 2.2 [Green] DiscussionColumn 計數改只算 active、promoted 數量呈於開關，並調整空狀態條件（0 active 且有 promoted 時欄體留白）。令 2.1 轉綠。驗證：`npm test -w packages/ui`。

## 3. chip 階段配色

- [x] 3.1 [Red] 寫失敗測試涵蓋 D2：階段 chip 沿用看板 STAGE_STYLE 配色——提案中／進行中／已就緒 chip 呈對應階段欄的 teal 濃度、已封存中性、已刪除 destructive 加刪除線。驗證：見紅。
- [x] 3.2 [Green] promoted 細列的階段 chip 取用 STAGE_STYLE 的 badge class（及已封存／已刪除樣式），取代現行統一灰底。令 3.1 轉綠。驗證：`npm test -w packages/ui` 全綠。

## 4. 重構與回歸

- [x] 4.1 [Refactor] 檢視 DiscussionColumn 變更，去除重複、對齊命名與樣式，確認未觸及 open／concluded 卡動詞與 core／IPC／adapter。驗證：`npm test -w packages/ui` 全綠，且 `npm run build -w apps/desktop` 通過。
