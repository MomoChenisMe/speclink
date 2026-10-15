## 1. 單複數機制（D2）

- [ ] 1.1 Red：在 `packages/ui/src/__tests__/i18n.test.tsx` 為 `tn` 寫測試：英文 n＝1 且有 `<key>.one` 時回單數形、n＝0 與 n＝2 回複數形、沒有 `.one` 時回原鍵、繁中一律回原鍵、`{n}` 已代入；Provider 外的預設 context 也有 `tn`。驗證：`npm test -w packages/ui` 新測試因 `tn` 不存在而失敗。 <!-- speclink-task:tsk_01M4Z3FRR8CRTCMFRKVB132JD3 -->
- [ ] 1.2 Green：在 `packages/ui/src/i18n.tsx` 的 `I18nContextValue`、`I18nProvider` 與預設 context 實作「D2 單複數由 `tn(key, n)` 與 `.one` 鍵處理」的規則。驗證：`npm test -w packages/ui` 的 1.1 測試通過。 <!-- speclink-task:tsk_01M4Z3FRR8C17MKVFT168VF6X8 -->

## 2. 字典與守門（D4）

- [ ] 2.1 Red：依「D4 守門測試與鍵集合測試」修改 `packages/ui/src/__tests__/i18n.test.tsx`、`apps/server-web/src/__tests__/i18n.test.tsx` 與 `apps/desktop/src/__tests__/messages.test.ts`：鍵集合比對排除 `.one` 鍵；每個 `.one` 鍵在同一語言有基礎鍵且只出現在英文；新增守門：英文值符合「`{n}` 之後隔零到兩個單字接以 s 結尾單字」的訊息必須有 `.one`（對應「英文計數文案依數量使用單數或複數」與「後台英文計數文案依數量使用單數或複數」）。驗證：`npm test -w packages/ui` 與 `npm test -w apps/server-web` 的守門測試點名 34 個缺 `.one` 的鍵而失敗。 <!-- speclink-task:tsk_01M4Z3FRR83H4RHKX5DM4KS056 -->
- [ ] 2.2 Green：在 `packages/ui/src/i18n.tsx` 的英文字典為 design Implementation Contract 列的 31 個鍵補 `.one`，在 `apps/server-web/src/i18n/messages.ts` 的英文字典為 3 個鍵補 `.one`；單數形照文法寫（例如「1 user has not activated yet.」「1 requirement has no source change」「The last round still lists 1 open item」）。驗證：`npm test -w packages/ui`、`npm test -w apps/server-web` 與 `npm test -w apps/desktop` 的字典測試全部通過。 <!-- speclink-task:tsk_01M4Z3FRR8XQK809WHJKCS1JFJ -->

## 3. 呼叫端（D3）

- [ ] 3.1 依「D3 呼叫端逐一改用 `tn`」把 34 個鍵的每個呼叫端從 `t(key).replace("{n}", …)` 改成 `tn(key, n)`：先在 `packages/ui/src`、`apps/desktop/src`、`apps/server-web/src` 對每個鍵名逐一搜尋呼叫端並列出清單，改完再搜尋 `.replace("{n}"`，確認剩下的都不是這 34 個鍵；並為規格詳情頁「1 scenario」與總覽「1 user has not activated yet.」各補一條元件測試（英文、n＝1）。驗證：任務記錄附呼叫端清單與剩餘 `.replace("{n}"` 的說明；`npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全部通過。 <!-- speclink-task:tsk_01M4Z3FRR8XMP6183PM313Y94Y -->
- [ ] 3.2 排查同類位置：在三份字典的英文區搜尋含 `{n}` 的全部訊息（寫法：搜尋 `{n}`），逐一標明「名詞計數，已補 `.one`」或「序數、分數或不隨數量變化，不需要」與理由。驗證：任務記錄列出每個含 `{n}` 的訊息與判定。 <!-- speclink-task:tsk_01M4Z3FRR8ZZJRK278Q0BFSG9K -->

## 4. 狀態列對齊（D1、D5）

- [ ] 4.1 Red：新增 `packages/ui/src/__tests__/detailStatusLayout.test.tsx`，依「D5 狀態列以 DOM 結構測試」斷言 `DetailStatus` 根元素帶 `grid-cols-[max-content_minmax(0,1fr)]`、每個 `StatusRow` 帶 `col-span-2` 與 `grid-cols-subgrid`、標籤帶 `min-w-14` 與 `max-w-28` 且沒有 `w-14`。驗證：`npm test -w packages/ui` 新測試失敗。 <!-- speclink-task:tsk_01M4Z3FRR8YP3VV6WWGP1J34DA -->
- [ ] 4.2 Green：依「D1 狀態列以 subgrid 共用標籤欄」修改 `packages/ui/src/components/DetailShell.tsx` 的 `DetailStatus` 與 `packages/ui/src/components/DetailHeader.tsx` 的 `StatusRow`，保留列間細線（「詳情頁狀態列的標籤欄依最長標籤對齊」）。驗證：`npm test -w packages/ui` 與 `npm test -w apps/desktop` 全部通過，含既有四個詳情頁的測試。 <!-- speclink-task:tsk_01M4Z3FRR8X942ZHA5TFRM2WJ0 -->
- [ ] [M] 4.3 在桌面 app 實機切到英文介面，開規格詳情頁：狀態列「Requirements」「Provenance」與數值不重疊、數值左緣對齊、需求顯示「1 scenario」；再開一則只有 1 輪的討論，卡片顯示「1 round」；切回繁中，狀態列版面與修改前相同。 <!-- speclink-task:tsk_01M4Z3FRR8YCQVP0C8JKYT7RHH -->

## 5. 收尾

- [ ] 5.1 跑完整的受影響測試與 repo 腳本測試：`npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全部通過，且 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束。 <!-- speclink-task:tsk_01M4Z3FRR8D9N7EZZBY0Y97FYP -->
