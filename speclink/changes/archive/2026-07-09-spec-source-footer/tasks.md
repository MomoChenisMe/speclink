## 1. 來源解析工具

- [x] 1.1 [Red] 於 packages/ui/src/__tests__/trace.test.ts 寫失敗測試，實現 D1：於前端解析 @trace source，不動 core 與 IPC 與 D2：聚合至 spec 層級去重呈現，非逐 requirement 標註——輸入含單一／多個／重複 source 的 raw markdown 應回傳去重且依首次出現保序的 source 名陣列，無 @trace 或畸形區塊則回空／略過。驗證：`npm test -w packages/ui` 該檔由紅。
- [x] 1.2 [Green] 實作 packages/ui/src/trace.ts 純函式令 1.1 轉綠：以正則抽 @trace 區塊的 source 行、去重保序回傳，不新增 core／IPC。驗證：`npm test -w packages/ui` 相關測試轉綠。

## 2. 規格頁展開檢視來源 footer

- [x] 2.1 [Red] 為「規格頁提供清單、搜尋與展開檢視」的新增行為，於 packages/ui/src/__tests__/specList.test.tsx 寫失敗測試：展開含 source 的 spec 於全文下方顯示來源變更 footer（去重保序＋在地標籤），無帶 source 的 @trace 時 footer 缺席。驗證：`npm test -w packages/ui` 見紅。
- [x] 2.2 [Green] 實現 D3：footer 僅顯示不可點擊且不顯示 updated——SpecList 展開區於 Markdown 之後條件渲染 footer（呼叫 trace.ts，純文字、不可點、不顯示 updated 或 code），packages/ui/src/i18n.tsx 新增 footer 標籤鍵；令 2.1 轉綠。驗證：`npm test -w packages/ui` 全綠。

## 3. 重構與回歸

- [x] 3.1 [Refactor] 檢視 trace.ts 與 SpecList footer 實作，去除重複、對齊既有命名與 Tailwind 樣式，確認未觸及 core／IPC／adapter。驗證：`npm test -w packages/ui` 全綠，且 `npm run build -w apps/desktop` 通過。
