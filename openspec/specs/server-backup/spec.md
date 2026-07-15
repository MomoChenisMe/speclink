# server-backup Specification

## Purpose

TBD - created by archiving change 'server-backup-restore'. Update Purpose after archive.

## Requirements

### Requirement: 備份檔自描述且逐項可驗證

backup 子命令 SHALL 產生單一備份檔，內含：manifest（備份格式版本、UTC 建立時間、engine/API 版本、store manifest、identity schema version、scope 清單）、每個 registry scope 的 export bundle（經 TeamStore export 契約產生，SHALL NOT 直接拷貝 store 資料庫檔）、identity 資料庫時點一致快照，與覆蓋每個成員的 digest。備份 SHALL NOT 含任何憑證明文。備份 SHALL 於無寫入的條件下執行（server 未運行或部署層維護窗口），SHALL NOT 宣稱寫入中快照的一致性。

#### Scenario: 備份內容完備

- **WHEN** 對含兩個 scope、若干使用者與 audit 記錄的 server 執行 backup 子命令
- **THEN** 備份檔含 manifest、兩個 scope 的 bundle 與 identity 快照，逐成員 digest 齊全；檔內不存在任何 PAT/密碼/token 明文

---

<!-- @trace
source: server-backup-restore
updated: 2026-07-15
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/tests/admin_data.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/backup_restore.rs
  - crates/speclink-server/tests/identity.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-backup.zh-TW.md
-->

---
### Requirement: verify-backup 攔截竄改與未知格式

verify-backup 子命令 SHALL 只讀備份檔完成驗證：manifest 與逐成員 digest 比對、bundle 結構可解析、備份格式版本已知——全數通過回 0；任一成員 digest 不符、結構不可解析或格式版本未知 SHALL 回非零 exit code 並指出不符成員。restore SHALL 以同一驗證為第一步。

#### Scenario: 竄改一位元即拒絕

- **WHEN** 修改備份檔內任一成員的一個位元後執行 verify-backup 與 restore
- **THEN** 兩者皆非零 exit code 且指出 digest 不符的成員；restore 未寫入目標任何內容

#### Scenario: 未知格式版本拒絕

- **WHEN** 對 manifest 宣告未知備份格式版本的備份檔執行 restore
- **THEN** 非零 exit code 且原因指出版本不相容；目標不被寫入

---

<!-- @trace
source: server-backup-restore
updated: 2026-07-15
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/tests/admin_data.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/backup_restore.rs
  - crates/speclink-server/tests/identity.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-backup.zh-TW.md
-->

---
### Requirement: restore 只進空目標且驗證即還原的一部分

restore 子命令 SHALL 要求目標 store 與 identity 皆空，非空 SHALL 拒絕並輸出既有內容摘要，SHALL NOT 提供覆蓋既有資料的旗標。還原 SHALL 依序：備份檔完整性驗證、identity 快照落位、逐 scope import、restore validation——逐 scope 重讀比對 bundle digest 與文件數、identity 的使用者/registry/audit 計數與 schema version 對 manifest 比對。驗證 SHALL 輸出逐項報告；任一不符 SHALL 回非零 exit code、明示差異項並明示目標不可投產。

#### Scenario: 災難演練閉環

- **WHEN** 對播種完整的 server（changes、討論、成員、PAT、audit）備份後，於全新空目標 restore 並啟動 server
- **THEN** restore validation 全綠；成員以備份前的 PAT 照常通行；CLI 查詢輸出與備份前一致；audit 歷史完整在列

#### Scenario: 非空目標拒絕

- **WHEN** 對已含任一 scope 文件或任一使用者的目標執行 restore
- **THEN** 非零 exit code；輸出既有內容摘要；目標內容位元不變

---

<!-- @trace
source: server-backup-restore
updated: 2026-07-15
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/tests/admin_data.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/backup_restore.rs
  - crates/speclink-server/tests/identity.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-backup.zh-TW.md
-->

---
### Requirement: admin 資料操作入 audit

admin 面 SHALL 提供：scope export 下載（經 TeamStore export 即時產生 bundle）、備份資訊檢視（最近備份與驗證的 manifest 摘要與結果）、store migration 觸發（前置 health 檢查通過才執行，失敗回明確錯誤且 SHALL NOT 自動重試）。三者 SHALL 沿用 admin 門禁並各記 audit（動作種類：scope-exported、store-migrated、backup-recorded）；未知 scope 的 export 下載 SHALL 回 404。

#### Scenario: export 下載可還原驗證

- **WHEN** admin 下載某 scope 的 export bundle 並對其執行結構與 digest 驗證
- **THEN** bundle 可解析且 digest 與 store 內容一致；audit 含一筆 scope-exported 記錄

#### Scenario: health 不過不 migrate

- **WHEN** store health 失敗時觸發 migration
- **THEN** migration 未執行；回應明示 health 失敗原因；audit 無 store-migrated 記錄

<!-- @trace
source: server-backup-restore
updated: 2026-07-15
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/tests/admin_data.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/backup_restore.rs
  - crates/speclink-server/tests/identity.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-backup.zh-TW.md
-->