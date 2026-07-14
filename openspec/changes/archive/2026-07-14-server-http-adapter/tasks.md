## 1. host 橋接（engine-over-TeamStore）

- [x] 1.1 【紅】針對「engine 動詞經橋接於 TeamStore 上執行」的雙路徑一致寫測試：對含相同 change 內容的 fs workspace 與 in-memory TeamStore scope，分別執行同一查詢動詞（list、status）與同一變更型動詞（task done），比對 typed outcome 結構、領域事件種類與 not_found 失敗情境的錯誤碼。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXDYG45WV3MBPBB58N9BBH4V -->
- [x] 1.2 【綠】實作 crates/speclink-host/src/bridge.rs 的讀取視圖：以 TeamStore snapshot 供應 engine 命令層 store 讀取；1.1 的查詢動詞部分轉綠。 <!-- speclink-task:tsk_01KXDYG45XN2WETV0YQMAZ8NJ8 -->
- [x] 1.3 【綠】實作寫入捕捉與提交：變更型動詞寫入捕捉為 UnitOfWork staged ops，成功後連同領域事件經 commit_with_events 原子提交；revision_conflict 映射保留 expected/actual。1.1 全綠，並補「橋接寫入原子落店」情境測試（task done 後文件、revision、事件同 commit 可見）。 <!-- speclink-task:tsk_01KXDYG45XHH6P3A77KA2FT2M6 -->

## 2. server crate 與啟動組態

- [x] 2.1 建立 crates/speclink-server（axum＋tokio，僅此 crate 引入 async）納入 workspace；實作組態檔載入（store 段 sqlite/memory、projects 段 key/name/repos、tokens 段 token/actor）。【紅→綠】針對「啟動組態 fail closed」寫測試並實作：組態缺失/不可解析/未知 driver 皆啟動失敗、stderr 指出路徑與原因、不綁定連接埠。 <!-- speclink-task:tsk_01KXDYG45XC6G3GGV7K5XFJYSS -->
- [x] 2.2 實作 /healthz 與 /readyz：healthz 回程序存活；readyz 呼叫 store health，不可用回非 2xx。驗收：memory 組態下兩端點 2xx；以不存在的 sqlite 路徑組態驗 readyz 非 2xx。 <!-- speclink-task:tsk_01KXDYG45XFTCDAVYXXSDQD62V -->

## 3. 認證、binding 與錯誤映射

- [x] 3.1 【紅】針對「binding 與認證前置 fail closed」寫路由測試：未知 token 回 401 permission_denied；未註冊 project key 回 404 not_found；X-Speclink-Repo 未註冊回 not_found；雙 repo project 缺標頭回拒絕且 message 指出候選；api version 不相容回拒絕帶版本原因；相容請求的 /binding 回 actor/project/repo/apiVersion/engineVersion/capabilities（polling 宣告、無 push transport）。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXDYG45X73JVSD02FGASP2JG -->
- [x] 3.2 【綠】實作認證與 binding 前置（token→actor、registry 查核、repo 裁決重用 host resolve_binding）與 /binding 路由，3.1 全綠。 <!-- speclink-task:tsk_01KXDYG45XES1Y3W0BZ4NENCAZ -->
- [x] 3.3 實作錯誤映射單點：engine 五碼、store 六類、binding 拒絕映射到 wire 八值 reason 與 HTTP status（設計決策 6 的對照表），message 沿用 engine 現行訊息。以單元測試固定每一格映射。 <!-- speclink-task:tsk_01KXDYG45XK8D8AEK6HPRQTQPW -->

## 4. 查詢與命令路由

- [x] 4.1 【紅→綠】實作 changes 查詢路由（清單、狀態、apply/artifact instructions、artifact 內容、specs、config、whoami、language），回應皆為 protocol DTO 並攜 scope 級 ETag。測試：以 typed client 對測，欄位形狀與 stub 對測一致；對不存在 change 回 404 not_found 三元組。 <!-- speclink-task:tsk_01KXDYG45XMR7PAQR6CX9R145K -->
- [x] 4.2 【紅→綠】實作 changes 命令路由（create change、put artifact 帶 If-Match、task done/undone、claim、archive）：寫入經橋接與 Host commit 原子提交。測試涵蓋「寫入原子提交且 CAS 衝突可辨」兩情境——競寫敗方 409 revision_conflict 帶 expected/actual 且無部分寫入；task done 後 outbox 含 task-completed 事件。 <!-- speclink-task:tsk_01KXDYG45X59HCXVGDNXVGR4G2 -->
- [x] 4.3 【紅→綠】實作 discussions 路由（建立、context、rounds、conclude、promote、archive）經同一橋接與提交路徑。測試：promote 成功時回應含新 change 名，且 outbox 含 discussion-promoted 與 change-created 兩筆事件。 <!-- speclink-task:tsk_01KXDYG45XBGNA44KCFVA3X7WS -->

## 5. 輪詢地基與端到端

- [x] 5.1 【紅→綠】實作 /sync-state 與查詢 ETag 的共用狀態記號（scope 全文件 revision 聚合摘要）：任何成功 commit 後記號必變；If-None-Match 未變回 304、變了回 200 新記號。測試涵蓋「輪詢偵測變更」情境與 store 失聯時查詢回 503 unavailable。 <!-- speclink-task:tsk_01KXDYG45X3KNS70H0Z6Y3A75Z -->
- [x] 5.2 【紅→綠】建立 crates/speclink-server/tests/e2e_cli.rs：啟動 tempdir SQLite 組態的真 server（依賴 sqlite-team-store 刀已落地），以環境變數把真實 CLI binary 指向它，經命令路由播種後重放代表性 remote 動詞（list/status/instructions apply/discuss list）：儲存決定型輸出與 fs 模式逐位元一致、帶本地路徑或投影欄位者剔除該類欄位後內容一致（全情境欄位形狀 parity 由 stub 對測凍結，設計決策 7）；重啟 server 後資料仍完整可查。 <!-- speclink-task:tsk_01KXDYG45XNJMM332ADP6657JB -->
- [x] 5.3 執行 npm run test:all 確認全 workspace 回歸：既有 parity 31 項、color 16 項、twin 8 情境（stub 對測）凍結零 diff。驗收：全數通過。 <!-- speclink-task:tsk_01KXDYG45XJ1V2NRYFVQBN9CWW -->
