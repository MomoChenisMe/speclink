## 1. crate 骨架與佈局

- [x] 1.1 建立 crates/speclink-store-fs（依賴 speclink-store 與檔案鎖 crate）納入 workspace；定義資料目錄佈局（meta 檔、每 scope 的 index/documents/history/outbox）與 schema version 常數。驗收：cargo build -p speclink-store-fs 通過。 <!-- speclink-task:tsk_01KXJ3SZRPGC5JH97E33QTHJXW -->
- [x] 1.2 【紅→綠】版本守門（涵蓋「版本守門與組態接線」的拒用面）：空目錄初始化；version 較新、meta 損毀、非本 driver 目錄拒用回 corrupt 且位元不變。 <!-- speclink-task:tsk_01KXJ3SZRPEQZQQ68WM4MTNXCR -->

## 2. 讀取面與原子 commit

- [x] 2.1 【紅】針對「索引原子發布與崩潰復原」寫測試：CAS 競寫恰一方成功、敗方 revision_conflict 帶 expected/actual；四故障點崩潰重開全無殘留且孤兒檔案被清除；mtime 竄改後讀取語意不變；單次索引讀取為 mixed-snapshot 一致性邊界。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXJ3SZRPK3P1620VKVQYQFQ0 -->
- [x] 2.2 【綠】實作 snapshot 讀取與 commit：內容/history/outbox 檔惰性落盤＋fsync → 新 index 暫名寫入 → 原子 rename 發布；開啟時孤兒掃描清除；排序與 revision 全出自序號。2.1 全綠。 <!-- speclink-task:tsk_01KXJ3SZRP3EGTMZYYG19H107V -->
- [x] 2.3 【紅→綠】檔案鎖單寫者（涵蓋「檔案鎖單寫者與失聯恢復」）：advisory lock 排他、他程序持有回 unavailable 不等待、持有者死亡後可接管（雙程序測試）；I/O 失敗分類（權限 backend、暫時失聯 unavailable）、失聯期間 commit 失敗不損毀、恢復重開續用。 <!-- speclink-task:tsk_01KXJ3SZRPVAMYD557FRDQKZB8 -->

## 3. conformance 全綠

- [x] 3.1 實作 history 查詢、outbox read/ack（cursor 持久化於 index）與 export/import（契約 Bundle 與 content_digest）。驗收：與 SQLite driver 對同語意內容的 bundle digest 一致（「bundle 與 driver 無關」情境）。 <!-- speclink-task:tsk_01KXJ3SZRPNYNX0A3NXYER7S3B -->
- [x] 3.2 【紅→綠】tests/conformance.rs 以 StoreHarness 掛接（tempdir、arm_crash 故障鉤於指定階段放棄索引替換、arm_outbox_failure 注入記錄檔寫入錯誤），執行 speclink-store conformance run 修至零 failure——含能力檢查（single-node、必要 capabilities、無 cluster）與 tenant scope。 <!-- speclink-task:tsk_01KXJ3SZRPQD0QV1K159TNN3G9 -->

## 4. server 接線與回歸

- [x] 4.1 【紅→綠】server 組態 store 段新增 serverfs 變體（path），build_store 單點接線；以 serverfs 組態跑既有 server e2e 動詞流程，行為與 sqlite 組態一致、重啟資料完整（涵蓋「serverfs 組態可服務」情境）。 <!-- speclink-task:tsk_01KXJ3SZRPV09P7TPDFNAC3QVH -->
- [x] 4.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；部署文件補 serverfs 選項與檔案系統前提（需支援 flock 語意）。驗收：全數通過。 <!-- speclink-task:tsk_01KXJ3SZRPA6GNAGBCQJNDSQ5H -->
