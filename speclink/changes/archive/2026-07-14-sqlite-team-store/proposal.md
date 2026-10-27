## Why

TeamStore 契約（manifest、typed 讀取、UoW/CAS、history、outbox、export/import、conformance suite）在 teamstore-contract-v2 刀已定案，但目前唯一實作是測試用的 in-memory reference——沒有任何可持久化的 driver，Phase 2 的 reference server 無法對真實資料啟動。平台藍圖（docs/platform-architecture.zh-TW.md §13.1、§14 Phase 2 第 2 項）指定 SQLite 為官方 server 的預設 Store；路線圖（docs/implementation-refactor-roadmap.zh-TW.md §4.3）明訂順序：先讓 SQLite reference implementation 通過 conformance 與 failure tests，之後才以同一套 suite 實作 Server FS 與 PostgreSQL。這一刀就是那個 SQLite reference implementation。

目標使用者：架設 speclink-server 的小型團隊（SQLite 是他們的預設持久層），以及後續 ServerFS/PostgreSQL driver 的實作者（以本刀為對照範本）。

## What Changes

- 新增 `speclink-store-sqlite` crate：以單一 SQLite 資料庫檔實作 TeamStore 契約全數方法——manifest（single-node 等級：snapshot、cas、transaction、history、outbox、migration、backup 能力宣告）、health、migrate、snapshot 讀取、begin_unit_of_work／commit／rollback（CAS 與原子性）、revision history、outbox read/ack、export/import。
- 通過既有 conformance suite 全部情境（含 CAS race、mixed snapshot、partial commit、outbox failure、crash recovery、tenant scope 六類 gate），以 StoreHarness 的 arm_crash 故障注入實作「程序崩潰後重開檔案」的復原驗證——crash 模擬必須真實重開資料庫連線，不得只在記憶體內假裝。
- schema 內建版本標記與 migrate 路徑：開啟未知版本的資料庫 fail closed（回 corrupt 或 backend 錯誤，不靜默升級、不損毀資料）。
- 併發模型為 single-node 單程序：WAL journal 模式、單一寫入者序列化；同檔多程序不在支援範圍（manifest 等級如實宣告）。

## Capabilities

### New Capabilities

- `sqlite-team-store`: SQLite 持久化 TeamStore driver——契約全方法、conformance 全綠、崩潰復原與版本化 schema 的行為保證。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增 crate，不動任何既有 crate 的程式碼與行為；CLI/桌面輸出凍結（parity 31 項、color 16 項、twin 8 情境）不受影響。唯一共用檔變更是 workspace Cargo 清單與 lockfile 納入新 crate 與 rusqlite 依賴。
- Affected specs: `sqlite-team-store`（新增）
- Affected code:
  - New: crates/speclink-store-sqlite/Cargo.toml、crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store-sqlite/src/schema.rs、crates/speclink-store-sqlite/tests/conformance.rs
  - Modified: Cargo.toml、Cargo.lock
  - Removed: 無
