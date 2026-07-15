# postgres-team-store Specification

## Purpose

TBD - created by archiving change 'postgres-team-store'. Update Purpose after archive.

## Requirements

### Requirement: PostgreSQL driver 通過 TeamStore conformance

PostgreSQL driver SHALL 實作 TeamStore 契約全數方法，並以 StoreHarness 掛接契約的 conformance suite 對真實 PostgreSQL 實例執行至全綠——涵蓋能力檢查、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery（以放棄連線使 transaction abort 模擬崩潰）與 tenant scope。manifest SHALL 宣告 single-node 等級，capability 集合 SHALL 含 snapshot、cas、transaction、history、outbox、migration 與 backup；SHALL NOT 宣告 cluster。export bundle SHALL 使用契約的 content_digest——同語意內容與其他 driver 的 bundle digest 一致。

#### Scenario: conformance 對真實例全綠

- **WHEN** 對環境變數指向的 PostgreSQL 實例執行 conformance suite
- **THEN** 報告零 failure；四故障點的崩潰模擬後以全新連線驗證 commit 從未發生

#### Scenario: bundle 與 driver 無關

- **WHEN** 對 PostgreSQL driver 與 SQLite driver 寫入語意相同的文件後各自 export
- **THEN** 兩份 bundle 的逐文件 digest 一致

---

<!-- @trace
source: postgres-team-store
updated: 2026-07-15
code:
  - .github/workflows/ci.yml
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/pg/mod.rs
  - crates/speclink-server/tests/postgres_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-postgres/Cargo.toml
  - crates/speclink-store-postgres/src/lib.rs
  - crates/speclink-store-postgres/src/schema.rs
  - crates/speclink-store-postgres/tests/bundle_and_outbox.rs
  - crates/speclink-store-postgres/tests/conformance.rs
  - crates/speclink-store-postgres/tests/infra.rs
  - crates/speclink-store-postgres/tests/resilience.rs
  - crates/speclink-store-postgres/tests/single_writer.rs
  - crates/speclink-store-postgres/tests/support/mod.rs
  - crates/speclink-store-postgres/tests/version_gate.rs
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: transaction 原子性與 advisory lock 單寫者

commit SHALL 在單一 SQL transaction 內完成 CAS 檢查、documents 寫入、history append 與 outbox append，任一步失敗 SHALL 整筆 abort 不留痕跡；CAS 失敗 SHALL 回 revision_conflict 帶 expected 與 actual。同 scope 的寫入 SHALL 以 transaction-scoped advisory lock 序列化（transaction 結束自動釋放，連線死亡即釋放，SHALL NOT 有殘留鎖）；跨 scope 寫入 SHALL NOT 互相阻塞；讀取 SHALL NOT 取鎖且 snapshot 讀取面 SHALL 一致（mixed snapshot gate）。

#### Scenario: 併發競寫恰一勝且無互鎖殘留

- **WHEN** 兩條連線以相同 expected revision 併發 commit 同一文件，敗方連線隨後直接關閉
- **THEN** 恰一方成功；敗方收到 revision_conflict；後續新連線的寫入不被任何殘留鎖阻塞

#### Scenario: 跨 scope 不互擋

- **WHEN** 兩條連線同時對不同 scope 執行 commit
- **THEN** 兩者皆成功，互不等待對方的 lock

---

<!-- @trace
source: postgres-team-store
updated: 2026-07-15
code:
  - .github/workflows/ci.yml
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/pg/mod.rs
  - crates/speclink-server/tests/postgres_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-postgres/Cargo.toml
  - crates/speclink-store-postgres/src/lib.rs
  - crates/speclink-store-postgres/src/schema.rs
  - crates/speclink-store-postgres/tests/bundle_and_outbox.rs
  - crates/speclink-store-postgres/tests/conformance.rs
  - crates/speclink-store-postgres/tests/infra.rs
  - crates/speclink-store-postgres/tests/resilience.rs
  - crates/speclink-store-postgres/tests/single_writer.rs
  - crates/speclink-store-postgres/tests/support/mod.rs
  - crates/speclink-store-postgres/tests/version_gate.rs
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: 連線失敗分類與版本守門

認證失敗或資料庫不存在 SHALL 使開啟失敗回 backend（帶原因）；運行中連線中斷 SHALL 回 unavailable 且 SHALL NOT panic，連線恢復後 SHALL 直接續用。meta 表記錄的 schema version 較新、或資料庫含陌生 schema SHALL 拒用回 corrupt 且 SHALL NOT 寫入；舊版本 SHALL 經 migrate 升級且既有資料完整保留。server 組態 store 段 SHALL 接受 postgres 變體（連線 URL）；密碼 SHALL 可由環境變數補全，URL 內嵌密碼時啟動 SHALL 輸出建議改用環境變數的警告。

#### Scenario: 連線中斷回 unavailable 且恢復續用

- **WHEN** 運行中使 PostgreSQL 連線中斷後執行讀取，再恢復連線重試
- **THEN** 中斷期間回 unavailable 不 panic；恢復後同一 store 實例讀寫正常

#### Scenario: 密碼來源紀律

- **WHEN** 分別以「URL 無密碼＋環境變數密碼」與「URL 內嵌密碼」啟動 server
- **THEN** 前者正常啟動無警告；後者正常啟動但 stderr 含建議改用環境變數的警告

---

<!-- @trace
source: postgres-team-store
updated: 2026-07-15
code:
  - .github/workflows/ci.yml
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/pg/mod.rs
  - crates/speclink-server/tests/postgres_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-postgres/Cargo.toml
  - crates/speclink-store-postgres/src/lib.rs
  - crates/speclink-store-postgres/src/schema.rs
  - crates/speclink-store-postgres/tests/bundle_and_outbox.rs
  - crates/speclink-store-postgres/tests/conformance.rs
  - crates/speclink-store-postgres/tests/infra.rs
  - crates/speclink-store-postgres/tests/resilience.rs
  - crates/speclink-store-postgres/tests/single_writer.rs
  - crates/speclink-store-postgres/tests/support/mod.rs
  - crates/speclink-store-postgres/tests/version_gate.rs
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: 測試基建不靜默假綠

conformance 與整合測試 SHALL 以環境變數指向的 PostgreSQL 實例執行；變數缺席時 SHALL 以顯性 skipped 結束並印出啟用指引，SHALL NOT 靜默回報通過。CI 工作流 SHALL 提供 PostgreSQL service 並設定該變數，使本測試集於 CI 必然執行。

#### Scenario: 無實例顯性略過

- **WHEN** 在未設定測試資料庫環境變數的環境執行本 crate 測試
- **THEN** 測試結果顯示 skipped（非 passed），輸出含一行可直接執行的本地啟用指引

#### Scenario: CI 必跑

- **WHEN** 檢視 CI 工作流定義
- **THEN** 含 PostgreSQL service 與環境變數設定，本 crate 測試集在其中執行且失敗會使工作流失敗

<!-- @trace
source: postgres-team-store
updated: 2026-07-15
code:
  - .github/workflows/ci.yml
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/pg/mod.rs
  - crates/speclink-server/tests/postgres_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-postgres/Cargo.toml
  - crates/speclink-store-postgres/src/lib.rs
  - crates/speclink-store-postgres/src/schema.rs
  - crates/speclink-store-postgres/tests/bundle_and_outbox.rs
  - crates/speclink-store-postgres/tests/conformance.rs
  - crates/speclink-store-postgres/tests/infra.rs
  - crates/speclink-store-postgres/tests/resilience.rs
  - crates/speclink-store-postgres/tests/single_writer.rs
  - crates/speclink-store-postgres/tests/support/mod.rs
  - crates/speclink-store-postgres/tests/version_gate.rs
  - docs/server-store-drivers.zh-TW.md
-->