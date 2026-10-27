## 1. 備份產生與驗證

- [x] 1.1 【紅】針對「備份檔自描述且逐項可驗證」寫測試：backup 子命令輸出的 tar 含 manifest（格式版本、UTC 時間、版本資訊、scope 清單）、逐 scope bundle（出自 TeamStore export，非資料庫檔拷貝）、identity 快照與逐成員 digest；檔內無憑證明文（掃描不含 spk_ 前綴明文與密碼欄位值）。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXG4YK84TMD6GAAD66P505BP -->
- [x] 1.2 【綠】實作 backup 子命令（crates/speclink-server/src/backup.rs）：離線模式對 --config 指向的資料直接執行；identity 快照走 SQLite 線上備份 API（含 WAL 收斂）；tar 組裝與 digest 側檔。1.1 全綠。 <!-- speclink-task:tsk_01KXG4YK847W9BEGAKCXPZ0XEN -->
- [x] 1.3 【紅→綠】verify-backup 子命令：manifest 與逐成員 digest 比對、bundle 結構解析、格式版本檢查；任一位元竄改回非零並指出成員；未知格式版本拒絕。涵蓋「竄改一位元即拒絕」與「未知格式版本拒絕」情境。 <!-- speclink-task:tsk_01KXG4YK843AWB5DBPWQQMT18V -->

## 2. 還原與 validation

- [x] 2.1 【紅】針對「restore 只進空目標且驗證即還原的一部分」寫測試：非空目標（任一 scope 有文件或任一 user 存在）拒絕並輸出摘要且目標位元不變；空目標還原後逐 scope digest/文件數比對、identity 計數與 schema version 對 manifest 比對，全綠回 0；人為製造不符（還原後改一份文件再跑 validation 路徑）得到逐項差異報告與非零 exit code。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXG4YK84TRQWYK6R3BJGYA8H -->
- [x] 2.2 【綠】實作 restore 子命令：完整性驗證（重用 verify-backup）→ identity 快照落位 → 逐 scope import（全新建立模式）→ restore validation 報告。2.1 全綠。 <!-- speclink-task:tsk_01KXG4YK841SJMHSKWMAFDNPA3 -->

## 3. admin 資料操作

- [x] 3.1 【紅→綠】scope export 下載路由與頁面入口：admin 門禁、即時 export 產 bundle 下載、未知 scope 回 404、audit 記 scope-exported。涵蓋「export 下載可還原驗證」情境（下載的 bundle 過結構與 digest 驗證）。 <!-- speclink-task:tsk_01KXG4YK84Y1GBT60GBA8NBPN0 -->
- [x] 3.2 【紅→綠】備份資訊與 migration 觸發：backup/verify 子命令將結果摘要寫入 identity 庫備份記錄表（schema 演進一版，migrate 升級測試沿用既有守門）；admin 頁顯示最近備份資訊；migration 觸發前置 health 檢查、成功記 store-migrated、health 失敗不執行不記錄（涵蓋「health 不過不 migrate」情境）。 <!-- speclink-task:tsk_01KXG4YK84Q6YD8V0KCS604K4M -->

## 4. 災難演練與回歸

- [x] 4.1 【紅→綠】災難演練 e2e（涵蓋「災難演練閉環」）：真 server 播種（setup、成員、PAT、changes 動詞流程、audit）→ backup → 全新 tempdir restore → validation 全綠 → 啟動還原後 server：成員原 PAT 通行、CLI 查詢輸出與備份前逐位元一致、audit 歷史完整、/setup 維持關閉。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXG4YK84TXE19HH1GRZB74VF -->
- [x] 4.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；部署文件補 backup/restore/verify-backup 操作說明與排程範例。驗收：全數通過。 <!-- speclink-task:tsk_01KXG4YK84E8JM08AQ8CJVBJ6N -->
