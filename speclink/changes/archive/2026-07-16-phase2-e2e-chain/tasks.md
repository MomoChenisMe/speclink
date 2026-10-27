## 1. 劇本骨架與前四環節

- [x] 1.1 建立 crates/speclink-server/tests/phase2_chain.rs 骨架：單一 #[test]、步驟 helper（名稱即步驟名）、失敗時 panic 攜步驟編號/名稱並傾印 server stderr 尾段與 workspace 目錄樹（涵蓋「失敗現場可讀且 CI 必跑」——以開發期人為注入壞斷言驗證訊息格式後移除）。擴充 tests/common 的訂閱者 helper（記錄事件流、可強制斷線、以 Last-Event-ID 重連）。 <!-- speclink-task:tsk_01KXMAC48K2VMAMWKA99AHY43P -->
- [x] 1.2 【紅→綠】步驟 (1)-(3)：全新資料庫啟動取 setup token → HTTP 走 /setup（Admin＋Project/Repo）→ invite 子命令 → 接受頁設密碼 → 登入建 PAT → CLI 以 PAT new change 並寫全部 artifacts。斷言環節共用同一資料庫與帳號、change 清單可見。 <!-- speclink-task:tsk_01KXMAC48K3WP866K8TXBQHKAP -->
- [x] 1.3 【紅→綠】步驟 (4) policy（涵蓋「policy 變化可觀察」情境）：寫入 workflow config 一條可觀察政策差異（無 CLI/wire 寫入面，依 design 決策 2 以第二條 store 連線直寫；差異取 locale）→ CLI instructions 輸出反映 → 改回 → 輸出恢復。 <!-- speclink-task:tsk_01KXMAC48KR74T3V8FJXJPVPCT -->

## 2. 後四環節

- [x] 2.1 【紅→綠】步驟 (5) evidence 三連：task done 攜 touched files → 斷言 evidence 記錄可查（taskId/actor/touchedFiles）且訂閱者收到 task-completed 事件——寫入、事件、證據三面同時成立。 <!-- speclink-task:tsk_01KXMAC48K5ZN6TPVYKQ3M79Z4 -->
- [x] 2.2 【紅→綠】步驟 (6)-(7)：apply 階段動詞後投影完整（正典 specs、delta、manifest 驗證通過）；remote drift（有 checkout）回完整報告（依 server-drift-api 刀，前置已歸檔）。 <!-- speclink-task:tsk_01KXMAC48KVMKN9442JHNV1XQS -->
- [x] 2.3 【紅→綠】步驟 (8) archive：正典 specs 更新含本劇本 delta、change 入 archive、清單如實；全鏈劇本首次端到端全綠（涵蓋「全鏈劇本綠」情境）。 <!-- speclink-task:tsk_01KXMAC48K24VASS5G2G6MDRYZ -->

## 3. event recovery 兩路徑

- [x] 3.1 【紅→綠】續傳路徑（涵蓋「續傳路徑收斂」情境）：訂閱者於步驟 (5) 後強制斷線、錯過 (6)-(8) 事件，以 Last-Event-ID 重連補齊無重複；結尾斷言訂閱者去重後視角與直接查詢正典一致。 <!-- speclink-task:tsk_01KXMAC48KKHHDHE9PRAJJDQZT -->
- [x] 3.2 【紅→綠】reset 路徑（涵蓋「reset 路徑收斂」情境）：劇本第二配置以極小保留筆數執行，斷線期間序號被清理 → 重連收 reset → /sync-state 與查詢全量收斂 → 重新訂閱 → 結尾視角一致。組態值於劇本內顯性宣告並註解對應路徑。 <!-- speclink-task:tsk_01KXMAC48KBYXQDGH04P3ZAWPB -->

## 4. CI 與回歸

- [x] 4.1 確認劇本在 CI 必跑路徑（cargo test -p speclink-server 既有 job 內）；若單測耗時逾 60 秒改為顯性分組並於 ci.yml 列出（不靜默 ignore）。驗收：CI 定義可見其執行。 <!-- speclink-task:tsk_01KXMAC48K5FSZE5RQQJDW0Z9H -->
- [x] 4.2 執行 npm run test:all 確認全 workspace 回歸：本刀純新增測試，凍結零 diff；若劇本揭露產品缺陷，以 #[ignore] 暫標該步附待開 change 名並回報，不順手修產品程式碼。驗收：全數通過（或揭露缺陷時如實回報清單）。 <!-- speclink-task:tsk_01KXMAC48K0MB9YSTXP9MQZCN5 -->
