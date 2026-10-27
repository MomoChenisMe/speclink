## 1. crate 骨架與 schema

- [x] 1.1 建立 crates/speclink-store-sqlite（依賴 speclink-store 與 rusqlite bundled feature），納入 workspace Cargo 清單；定義 documents／history／outbox／meta 四張表的 schema SQL 與 schema version 常數（版本 1）。驗收：cargo build -p speclink-store-sqlite 通過，cargo build --workspace 其他 crate 不受影響。 <!-- speclink-task:tsk_01KXDY7CAAG1NBC7VRBNJV67X4 -->
- [x] 1.2 【紅】針對「schema 版本守門 fail closed」寫測試：空資料庫初始化為版本 1；version 為現行加一的資料庫拒開回 corrupt 且檔案位元不變；非 speclink 建立的 SQLite 檔拒開回 corrupt 且不寫入。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXDY7CAAT8HWN6GYB673HW6R -->
- [x] 1.3 【綠】實作 open(path) 開檔守門與 meta 表初始化，1.2 測試轉綠；health 對 version 低於現行回報需要 migration，migrate 執行升級（版本 1 為 no-op 路徑但守門邏輯完整）。 <!-- speclink-task:tsk_01KXDY7CAA702HMRKPXDD67YMP -->

## 2. 讀取與 UoW commit

- [x] 2.1 【紅】針對「commit 原子性與崩潰復原」的 CAS 情境寫測試：兩個 commit 以相同 expected revision 競寫同一文件，恰一方成功、敗方收到帶 expected/actual 的 revision_conflict 且其 UoW 其他 op 未落盤。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXDY7CAASY36882VTPEQZNF9 -->
- [x] 2.2 【綠】實作 snapshot 讀取（文件、revision、digest）與 begin_unit_of_work／commit／rollback：commit 在單一 SQL transaction 內完成 CAS 檢查、documents 寫入、history append、outbox append；2.1 測試轉綠。 <!-- speclink-task:tsk_01KXDY7CAA2FRZHD8VR09CQQRF -->
- [x] 2.3 【紅→綠】針對「持久化與重開一致性」寫測試並實作：commit 三筆文件並 ack 第一筆 outbox 事件後關閉，同路徑重開可完整讀回文件/revision/history，且自持久化 cursor 續讀只得未 ack 事件；WAL 殘留檔重開正常回放。實作 outbox read/ack 的 cursor 持久化與 WAL journal 模式開啟。 <!-- speclink-task:tsk_01KXDY7CAAG8M5AEYZ99PCN2S9 -->

## 3. export/import 與 conformance 全綠

- [x] 3.1 實作 revision history 查詢與 export/import（Bundle 往返，digest 沿用 speclink-store 的 content_digest；ImportMode 與 ImportReport 語意照契約）。驗收：conformance suite 中 history 與 bundle 相關情境通過。 <!-- speclink-task:tsk_01KXDY7CAABCTNW7NT5816Z47K -->
- [x] 3.2 【紅】建立 tests/conformance.rs：以 StoreHarness 掛接 SQLite driver（tempdir 資料庫檔），arm_crash 以 test-only 故障鉤實作「該點放棄連線、harness 開全新連線重建」。執行 speclink-store conformance run。驗收：harness 可執行，此時 crash recovery 或其餘情境可能未全綠。 <!-- speclink-task:tsk_01KXDY7CAA8ZNPNA811BQGWY7W -->
- [x] 3.3 【綠】修至 conformance 報告零 failure——涵蓋能力檢查（single-node 等級、snapshot/cas/transaction/history/outbox/migration/backup 宣告、無 cluster）、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 四故障點與 tenant scope（兩 scope 同名文件互不可見、outbox/history 不串租戶）。 <!-- speclink-task:tsk_01KXDY7CAAAMQTRACXAXF74ATV -->
- [x] 3.4 補充 driver 專屬邊界測試：同程序兩個 store 實例指向同一檔案時的行為（拒絕或序列化，擇一實作並以測試固定）；路徑不可寫回 backend 錯誤。驗收：cargo test -p speclink-store-sqlite 全綠。 <!-- speclink-task:tsk_01KXDY7CAA86B5AX8MQSQ3EX42 -->

## 4. 收尾

- [x] 4.1 執行 npm run test:all 確認全 workspace 回歸無破壞（本刀為純新增 crate，既有 parity/color/twin 凍結不應有任何 diff）。驗收：全數通過。 <!-- speclink-task:tsk_01KXDY7CAAAW7HN41B9VXTF7AS -->
