## 1. crate 骨架與測試基建

- [x] 1.1 建立 crates/speclink-store-postgres（依賴 speclink-store 與同步 postgres crate）納入 workspace；定義四表 schema SQL（documents、history、outbox、meta，含 scope 欄位與 per-scope outbox 序列）與 schema version 常數。驗收：cargo build -p speclink-store-postgres 通過。 <!-- speclink-task:tsk_01KXJ3YMYHQ6KVPC1PDAS225W0 -->
- [x] 1.2 【紅→綠】測試基建（涵蓋「測試基建不靜默假綠」的略過面）：測試 helper 讀 SPECLink 測試資料庫環境變數——有值即連線並以獨立 schema 隔離、測後清除；缺席以顯性 skipped 結束並印一行 docker 啟用指引。驗收：無變數環境顯示 skipped 非 passed。 <!-- speclink-task:tsk_01KXJ3YMYHSWQZX9MNKW7VDP5G -->
- [x] 1.3 【紅→綠】版本守門與連線分類（涵蓋「連線失敗分類與版本守門」的守門面）:空庫初始化為現行版本；version 較新或陌生 schema 拒用回 corrupt 不寫入；舊版 migrate 升級資料完整；認證失敗/庫不存在回 backend 帶原因。 <!-- speclink-task:tsk_01KXJ3YMYHWA721FQBVM2ZPH9Q -->

## 2. commit 與單寫者

- [x] 2.1 【紅】針對「transaction 原子性與 advisory lock 單寫者」寫測試：CAS 併發競寫恰一勝、敗方 revision_conflict 帶 expected/actual、敗方連線關閉後無殘留鎖；跨 scope 併發 commit 不互擋；snapshot 讀取面一致（讀取期間的併發寫入不撕裂）。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXJ3YMYHAZ52MRZQP2HR1B0N -->
- [x] 2.2 【綠】實作 snapshot 讀取與 commit：單一 SQL transaction 內 pg_advisory_xact_lock（scope 派生 64 位 key）→ CAS → documents/history/outbox 寫入 → COMMIT；讀取不取鎖。2.1 全綠。 <!-- speclink-task:tsk_01KXJ3YMYH18WYYKV3N0X23AAX -->
- [x] 2.3 【紅→綠】連線中斷韌性：運行中斷線回 unavailable 不 panic、恢復後同一實例續用（涵蓋「連線中斷回 unavailable 且恢復續用」情境）。 <!-- speclink-task:tsk_01KXJ3YMYHY69GBR600FYGS4F3 -->

## 3. conformance 全綠

- [x] 3.1 實作 history 查詢、outbox read/ack 與 export/import（契約 Bundle 與 content_digest）。驗收：與 SQLite driver 對同語意內容的 bundle digest 一致（「bundle 與 driver 無關」情境）。 <!-- speclink-task:tsk_01KXJ3YMYHHEM3RP21ESC9DW32 -->
- [x] 3.2 【紅→綠】tests/conformance.rs 以 StoreHarness 掛接真實例（獨立 schema、arm_crash 以該點放棄連線使 transaction abort、arm_outbox_failure 注入 append 語句錯誤），執行 conformance run 修至零 failure——含能力檢查（single-node、必要 capabilities、無 cluster）與 tenant scope。 <!-- speclink-task:tsk_01KXJ3YMYHMGRYD5H4WK74E97H -->

## 4. server 接線、CI 與回歸

- [x] 4.1 【紅→綠】server 組態 store 段新增 postgres 變體（url）：密碼可由環境變數補全、URL 內嵌密碼啟動警告（涵蓋「密碼來源紀律」情境）；build_store 單點接線；以 postgres 組態跑既有 server e2e 動詞流程，行為與 sqlite 組態一致、重啟資料完整。 <!-- speclink-task:tsk_01KXJ3YMYHPM235YN7EVPJFWGQ -->
- [x] 4.2 CI 工作流新增 PostgreSQL service container 與測試環境變數，使本 crate 測試集於 CI 必跑且失敗擋工作流（涵蓋「CI 必跑」情境）。驗收：CI 定義含 service 與變數，工作流跑過一次全綠。 <!-- speclink-task:tsk_01KXJ3YMYHS4RWR3HCKAVN2XV8 -->
- [x] 4.3 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；文件註記完整驗證需 PG 與支援的最低主版本。驗收：全數通過。 <!-- speclink-task:tsk_01KXJ3YMYH91371WVHNHCVQ59S -->
