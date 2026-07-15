## Why

藍圖 §13.1 把 PostgreSQL 列為官方 server 的第三個持久層選項（既有資料庫基礎設施的團隊；distributed coordination 完成前不宣稱 cluster），§14 Phase 2 第 2 項與 roadmap §5 gate 要求三 driver 全過共同 conformance suite——SQLite 與（平行刀中的）ServerFS 之外的最後一塊。SQLite reference implementation 已全綠、契約與 conformance 定案，roadmap §4.3 的複製條件成立。PostgreSQL 的特有課題是兩個：真實資料庫的測試基建（不能只在記憶體裡假裝）與跨連線的單寫者語意（advisory lock）——都必須在這把刀內定案受測。

目標使用者：已有 PostgreSQL 維運基礎的團隊（統一資料庫棧、既有備份與監控直接套用）；後續 cluster 模式的地基（本刀不宣稱、不預建）。

## What Changes

- 新增 `speclink-store-postgres` crate（藍圖 §13.4 的正典交付名）：以 PostgreSQL 資料庫實作 TeamStore 契約全數方法，schema 與 SQLite 範本同構（documents、history、outbox、meta 四表，加 scope 欄位），commit 在單一 SQL transaction 內完成 CAS、寫入與雙 append——原子性由資料庫 transaction 承擔。
- 單寫者語意：每 scope 以 PostgreSQL advisory lock 序列化寫入（transaction-scoped，隨 commit/abort 自動釋放）；manifest 宣告 single-node 等級，SHALL NOT 宣告 cluster。
- 通過同一套 conformance suite（含 arm_crash 四故障點——以「該點放棄連線使 transaction abort」模擬崩潰，harness 開全新連線驗證）。
- 測試基建：conformance 與整合測試以環境變數指向的 PostgreSQL 實例執行（本地用容器、CI 起 service container）；環境變數缺席時測試明確標記略過並印出啟用方式——不靜默假綠，CI 必跑。
- 連線與 secret：server 組態 store 段新增 postgres 變體（連線 URL）；密碼等 deployment secret 依 §13.2 優先來自環境變數（組態 URL 可省略密碼、由環境補全），組態檔含明文密碼時啟動警告。
- schema 版本守門沿用範本：meta 表記 version，較新拒用、舊版經 migrate 升級、陌生 schema 拒用不寫。

## Capabilities

### New Capabilities

- `postgres-team-store`: PostgreSQL TeamStore driver——conformance 全綠、SQL transaction 原子性、advisory lock 單寫者、secret 來源紀律、測試基建不靜默假綠的行為保證。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增 crate 與組態選項；既有 driver、server 路由與 CLI 行為零變更；parity 31 項、color 16 項、twin 8 情境凍結不動。備份格式與 driver 無關，本 driver 自動被既有 backup/restore 涵蓋。與 serverfs-team-store 刀無共享檔案（除 workspace Cargo 清單與 StoreConfig enum 兩行級接點），可平行實作。CI 設定新增 PostgreSQL service container。
- Affected specs: `postgres-team-store`（新增）
- Affected code:
  - New: crates/speclink-store-postgres/Cargo.toml、crates/speclink-store-postgres/src/lib.rs、crates/speclink-store-postgres/src/schema.rs、crates/speclink-store-postgres/tests/conformance.rs
  - Modified: Cargo.toml、Cargo.lock、crates/speclink-server/src/config.rs、crates/speclink-server/src/lib.rs、crates/speclink-server/Cargo.toml、.github/workflows 下的 CI 設定檔
  - Removed: 無
