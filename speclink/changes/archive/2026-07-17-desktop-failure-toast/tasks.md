## 1. Toaster 原語

- [x] 1.1 [Red] 為 D4: toast 採 shadcn registry 的 sonner，拆除其 wrapper 的 next-themes 相依 與 D3: 逾時、關閉鈕與單槽取代語意由 sonner 設定提供 寫失敗測試（`packages/ui/src/__tests__/sonner.test.tsx`，不 mock sonner）：掛載 `Toaster` 後發出一則訊息，訊息可見且帶關閉鈕；以同一 toast id 再發一則，僅後者可見、前者不在畫面上（取代語意，非佇列）；以 vitest 假計時器推進 6000 毫秒後訊息消失；點按關閉鈕立即消失。驗證：`npm test -w packages/ui` 見紅（模組尚不存在）。 <!-- speclink-task:tsk_01KXQPJFFS32WZ6NN9YNX38JR7 -->
- [x] 1.2 [Green] 將 `sonner` 加入 `packages/ui/package.json`，實作 `Toaster`（`packages/ui/src/components/ui/sonner.tsx`，取自 shadcn registry 的 sonner item 並拆除 `next-themes`／`useTheme()`）令 1.1 轉綠：設定 `theme="system"`（對接 `apps/desktop/src/index.css` 既有的 `prefers-color-scheme` 深色模式）、`duration={6000}`、`closeButton`、`visibleToasts={1}`、`closeButtonAriaLabel` 取自 i18n；自 `packages/ui/src/index.ts` 匯出；`toast.close` 鍵加入 `packages/ui/src/i18n.tsx` 的 zh-TW 與 en 兩本字典。可觀察行為：訊息浮現、逾時自動消失、可按鈕提前關閉、滑鼠停留其上時計時暫停。驗證：`npm test -w packages/ui` 全綠（含既有 `i18n.test.tsx` 的兩語言 key 集合相等）。 <!-- speclink-task:tsk_01KXQPJFFSW4Z5YK237WR1RJQZ -->

## 2. store 失敗發訊與成功靜默

- [x] 2.1 [Red] 為「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求的 store 側寫失敗測試（`apps/desktop/src/__tests__/store.test.ts`，以 vitest 對 `sonner` 模組設 mock），實現 D1: store 直接呼叫 sonner 的 toast API，不保留失敗狀態 與 D6: 失敗訊息文字由 desktop i18n 組出，sonner 不涉訊息內容。斷言：刪除成功／封存變更成功／封存討論成功三條路徑執行後未發出任何 toast；刪除失敗／封存變更失敗／封存討論失敗／拖排寫回失敗／開啟專案失敗／初始化專案失敗六條路徑執行後各發出一次 toast，訊息同時包含主詞與 core 錯誤字串且皆帶同一個固定 id。同步移除既有斷言 `verbResult` 的測試（`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/workspace.test.ts`），改斷言 toast 發出。驗證：`npm test -w apps/desktop` 見紅。 <!-- speclink-task:tsk_01KXQPJFFSPQEQ7402JR0CAKGP -->
- [x] 2.2 [Green] 將 `sonner` 加入 `apps/desktop/package.json`；於 `apps/desktop/src/store.ts` 刪除 `verbResult` 欄位（不設替代欄位、不設清除動作），六條失敗路徑改以 sonner 的 error 型 toast 發出「主詞 · i18n 失敗描述 ✗ core 錯誤」並帶模組層級的固定 id 常數，令 2.1 轉綠；三條成功路徑不再發出任何訊息；`apps/desktop/src/i18n/messages.ts` 新增 `store.archiveFailed`、`store.openProjectFailed`、`store.initFailed`，移除 `store.deleted`、`store.discussionArchived`（zh-TW 與 en 同步增刪以維持兩語言 key 集合相等）；開啟專案失敗與初始化失敗補上使用者選定的路徑／目錄為主詞。可觀察行為：封存成功不再出現英文 `archive ✓`。驗證：`npm test -w apps/desktop`（含 `messages.test.ts` 的 key 集合相等斷言）。 <!-- speclink-task:tsk_01KXQPJFFSBX7XJE5NSKY2NT62 -->
- [x] 2.3 [Red] 為 D5: 封存成功時關閉詳情抽屜 寫失敗測試（`apps/desktop/src/__tests__/store.test.ts`）：詳情抽屜選定某 change 時觸發封存且成功，`detailChange` 應為 null；封存失敗時 `detailChange` 維持不變（抽屜留著，使用者才看得到失敗當下的上下文）。驗證：`npm test -w apps/desktop` 見紅。 <!-- speclink-task:tsk_01KXQPJFFSYVK8R5A1KH3CQKS0 -->
- [x] 2.4 [Green] 於 `apps/desktop/src/store.ts` 的 archive 動詞成功分支清除 `detailChange`，令 2.3 轉綠。可觀察行為：自詳情抽屜封存成功後抽屜關閉、卡片自看板消失、側欄「已封存」計數遞增，全程無文字訊息。驗證：`npm test -w apps/desktop`。 <!-- speclink-task:tsk_01KXQPJFFSSHVWEQXMRCY72J8J -->

## 3. 頂欄狀態列移除與 Toaster 掛載

- [x] 3.1 [Red] 為「桌面 app 提供動詞操作面」改寫後的「視窗頂欄 SHALL NOT 承載任何操作結果訊息面」寫失敗測試（`apps/desktop/src/__tests__/App.test.tsx`）：畫面掛載後 `Toaster` 存在；頂欄不含任何操作結果文字節點。驗證：`npm test -w apps/desktop` 見紅。 <!-- speclink-task:tsk_01KXQPJFFS9HTERNV1YXD249AV -->
- [x] 3.2 [Green] 於 `apps/desktop/src/App.tsx` 移除頂欄的狀態文字渲染，改於根層掛載 `Toaster`，令 3.1 轉綠。可觀察行為：視窗頂欄不再有任何狀態文字。驗證：`npm test -w apps/desktop` 全綠、`npm run build -w apps/desktop` 通過。 <!-- speclink-task:tsk_01KXQPJFFS1SW4C1B6VREXYQ6F -->

## 4. 真實視窗驗證

- [x] 4.1 以 release app 驗證「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求中 jsdom 測不到的堆疊語意，實現 D2: toast 層級由 sonner 提供，不自訂 z-index——單元測試只能驗元件掛載與行為，實際 z 軸堆疊正是本刀的核心修復，且 sonner 與 Radix 的 portal 同掛 `document.body`，須眼見為憑。步驟：`npm run build -w apps/desktop` 後 `cargo build --release -p speclink-desktop` 並啟動（重建前先關閉執行中的 exe，否則 linker 存取被拒）；開啟任一 change 的詳情抽屜 → 對尚未滿足歸檔前置的 change 觸發封存 → 截圖確認 toast 完整浮現於抽屜遮罩之上、文字可讀且含主詞與 core 錯誤 → 確認逾時後自行消失；再截圖確認刪除成功後頂欄無任何文字。操作前先確認使用者未在使用螢幕。 <!-- speclink-task:tsk_01KXQPJFFSFKW97JN3MCR99RVZ -->

## 5. 重構與稽核

- [x] 5.1 [Refactor] 檢視回饋面收斂結果：確認 `verbResult` 已無殘留參照、無孤兒 imports 與 props；確認未觸及任務勾選寫回失敗、專案分頁錯誤態、連線狀態、設定頁儲存回饋四個既有回饋面（design 的 Non-Goals 與 Scope boundaries）；確認 `next-themes` 未被引入、`packages/ui` 與 `apps/desktop` 的 sonner 版本一致；確認未觸及任何 crate、CLI 人眼輸出與 `--json` 契約，回歸對照（parity／color／twin harness）無須重跑；對 store 交給 sonner 的訊息邊界套用 sharp-edges 稽核（以 `speclink instructions --skill audit` 取得清單）確認 core 錯誤為空字串或超長字串時不崩潰、且訊息以文字呈現不注入原始 HTML。驗證：`npm test -w packages/ui`、`npm test -w apps/desktop` 全綠，`npm run build -w apps/desktop` 通過。 <!-- speclink-task:tsk_01KXQPJFFSAYK9N92DQ1PVZWTE -->
