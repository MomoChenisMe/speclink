## 1. server 端點

- [x] 1.1 【紅】針對「一致快照端點」與「change 縮小與 flow 透傳」寫路由測試：單一 store snapshot 一致性（取快照後併發寫入不影響本次回應、兩次快照 id 不同）、snapshot id 與 scope 狀態記號同源（commit 後必變）、policy revision 為 config 文件 revision（無 config 時缺席）、逐文件契約 digest、If-None-Match 未變 304／變更後 200、change 縮小文件集完備（含全部正典 specs、該 change 的 delta specs、config、LANGUAGE，不含他 change）、未知 change 404、flow 透傳不影響文件集、未認證/非成員 401/403、store 失聯 503。文件集完備測試會 seed `DocumentId::Language` 種子（決策 6），故本任務隱含依賴 store 契約新增 Language 變體。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXHXBJ87JF3X4NN1M2A7VNCD -->
- [x] 1.2 【綠】先補 store 契約：`DocumentId` 新增 `Language` 變體（決策 6）、sqlite `lg` 編解碼、host bridge `read_language` 讀 `DocumentId::Language`、封閉集窮盡匹配與 closed-set 測試同步。再實作 context snapshot 端點（crates/speclink-server/src/context.rs，POST、body 為 ContextSnapshotRequest）：取一次 TeamStore snapshot（snapshot 定 id、export 枚舉、逐 doc 同 snapshot 回讀取得 content+revision）讀出全部文件、id 取 scope 狀態記號、digest 用契約 content_digest、DocumentId→openspec 路徑映射。1.1 全綠。 <!-- speclink-task:tsk_01KXHXBJ87R3DXPND475ZZ170B -->

## 2. typed client 方法

- [x] 2.1 【紅→綠】typed client 新增 context snapshot 方法（涵蓋「typed client 的 context snapshot 方法」）：輸入 request 與選填既知 snapshot id（If-None-Match）、輸出「未變/新快照」二值、走既有請求骨架與錯誤翻譯。以 stub server 測試二值輸出與 503 翻譯。 <!-- speclink-task:tsk_01KXHXBJ87EGT2M70YGWBHY5JJ -->

## 3. 供應者汰換與免重寫

- [x] 3.1 【紅→綠】remote 動詞流程的投影供應者汰換（涵蓋「遠端投影以 Context API 為來源」）：VerbContextProvider 的逐 artifact 拼裝改為單一 context snapshot 呼叫（帶 manifest 現值 id）；「未變」時 refresh 免重寫（投影檔案不變動）；API 失敗維持響亮警告、動詞照常、既有投影標 stale。以 stub server 驗證三情境（新快照重建、未變免重寫、失敗標 stale 不阻斷）。 <!-- speclink-task:tsk_01KXHXBJ87ZQYQDAA03FRBZPWQ -->
- [x] 3.2 確認投影機制對完整內容集的既有測試面直接通過：materialize 完整佈局、verify_projection digest fail-closed、stale/refresh、gitignore 保證——供應者升級後 cargo test -p speclink-host 全綠、無需修改既有投影測試期望。 <!-- speclink-task:tsk_01KXHXBJ87614TKXGR2439R5DP -->

## 4. 端到端與回歸

- [x] 4.1 【紅→綠】e2e：對真 server（SQLite）建 change 與正典 specs 後，remote 模式執行 apply 階段動詞——投影含正典 specs、delta specs、artifacts、config、LANGUAGE 與 INDEX，manifest snapshot id 為 server 識別且 verify 通過；contextFiles 每個值為投影下存在的路徑；重複執行同動詞投影不重寫；另一 client 寫入後再執行投影更新。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXHXBJ87ERS8BM13P47EYS12 -->
- [x] 4.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；remote 動詞的人眼與 --json 輸出不變；apply/verify 技能內容零變更（無三處同步）。驗收：全數通過。 <!-- speclink-task:tsk_01KXHXBJ87QVX1Y8A2NB14HY1M -->
