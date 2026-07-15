# serverfs-team-store Specification

## Purpose

TBD - created by archiving change 'serverfs-team-store'. Update Purpose after archive.

## Requirements

### Requirement: FS driver 通過 TeamStore conformance

檔案系統 driver SHALL 實作 TeamStore 契約全數方法，並以 StoreHarness 掛接契約的 conformance suite 執行至全綠——涵蓋能力檢查、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 與 tenant scope 全部情境。manifest SHALL 宣告 single-node 等級，capability 集合 SHALL 含 snapshot、cas、transaction、history、outbox、migration 與 backup；SHALL NOT 宣告 cluster。export bundle SHALL 使用契約的 content_digest——同語意內容與其他 driver 的 bundle digest 一致。

#### Scenario: conformance suite 全綠

- **WHEN** 以 tempdir 資料目錄建構 FS driver 並執行 conformance suite
- **THEN** 報告零 failure；manifest 檢查確認 single-node 等級與必要 capability 全數存在

#### Scenario: bundle 與 driver 無關

- **WHEN** 對 FS driver 與 SQLite driver 寫入語意相同的文件後各自 export
- **THEN** 兩份 bundle 的逐文件 digest 一致

---

<!-- @trace
source: serverfs-team-store
updated: 2026-07-15
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/serverfs_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-fs/Cargo.toml
  - crates/speclink-store-fs/src/layout.rs
  - crates/speclink-store-fs/src/lib.rs
  - crates/speclink-store-fs/tests/atomic_publish.rs
  - crates/speclink-store-fs/tests/bundle_and_outbox.rs
  - crates/speclink-store-fs/tests/conformance.rs
  - crates/speclink-store-fs/tests/single_writer.rs
  - crates/speclink-store-fs/tests/version_gate.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: 索引原子發布與崩潰復原

commit SHALL 以每 scope 索引檔的原子替換為唯一發布點：新內容檔、history 與 outbox 記錄先行落盤且未被引用前 SHALL 惰性無效；索引替換前的任一故障點崩潰，重開後 SHALL 呈現 commit 從未發生——文件內容、revision、history 與 outbox 一致為舊狀態；未被索引引用的孤兒檔案 SHALL 於開啟時清除。讀取 SHALL 以單次索引讀取為一致性邊界；排序與 revision SHALL 出自索引與記錄序號，SHALL NOT 以檔案 mtime 參與任何語意判斷。

#### Scenario: 四故障點崩潰全無殘留

- **WHEN** 於 AfterDocWrites、AfterHistoryAppend、BeforeOutboxAppend、AfterOutboxAppend 分別注入崩潰（放棄索引替換），以同目錄開全新實例
- **THEN** 四個情境重開後文件內容為舊值、revision 未動、history 與 outbox 無新記錄；孤兒檔案被清除

#### Scenario: mtime 竄改不影響語意

- **WHEN** 對資料目錄內全部檔案改寫任意 mtime 後重開讀取
- **THEN** 文件內容、revision 排序、history 與 outbox 讀取結果與竄改前完全一致

---

<!-- @trace
source: serverfs-team-store
updated: 2026-07-15
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/serverfs_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-fs/Cargo.toml
  - crates/speclink-store-fs/src/layout.rs
  - crates/speclink-store-fs/src/lib.rs
  - crates/speclink-store-fs/tests/atomic_publish.rs
  - crates/speclink-store-fs/tests/bundle_and_outbox.rs
  - crates/speclink-store-fs/tests/conformance.rs
  - crates/speclink-store-fs/tests/single_writer.rs
  - crates/speclink-store-fs/tests/version_gate.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: 檔案鎖單寫者與失聯恢復

driver SHALL 以資料目錄內鎖檔的 OS advisory lock 取得排他寫入；鎖被他程序持有時 SHALL 回 unavailable，SHALL NOT 等待或搶占；持有程序死亡後鎖 SHALL 隨之釋放，新程序可正常接管。I/O 失敗 SHALL 分類回錯：權限拒絕或路徑不存在回 backend、暫時性失敗回 unavailable，SHALL NOT panic 且既有狀態 SHALL 位元不變；失敗排除後重開 SHALL 直接續用。

#### Scenario: 雙程序互斥

- **WHEN** 一個實例持有資料目錄期間，第二個實例嘗試開啟同目錄
- **THEN** 第二個實例得到 unavailable；第一個實例正常結束後，第二次嘗試成功開啟且資料完整

#### Scenario: 失聯期間回錯不損毀

- **WHEN** 使資料目錄的子目錄暫時不可存取後執行 commit，再恢復存取重開
- **THEN** 該 commit 以錯誤失敗且不 panic；恢復後重開的狀態等於失聯前，後續 commit 正常

---

<!-- @trace
source: serverfs-team-store
updated: 2026-07-15
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/serverfs_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-fs/Cargo.toml
  - crates/speclink-store-fs/src/layout.rs
  - crates/speclink-store-fs/src/lib.rs
  - crates/speclink-store-fs/tests/atomic_publish.rs
  - crates/speclink-store-fs/tests/bundle_and_outbox.rs
  - crates/speclink-store-fs/tests/conformance.rs
  - crates/speclink-store-fs/tests/single_writer.rs
  - crates/speclink-store-fs/tests/version_gate.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-store-drivers.zh-TW.md
-->

---
### Requirement: 版本守門與組態接線

資料目錄 SHALL 記錄 driver 識別與 schema version：空目錄初始化為現行版本；version 較新、meta 損毀或非本 driver 的目錄 SHALL 拒用回 corrupt（帶原因）且 SHALL NOT 寫入。server 組態的 store 段 SHALL 接受 serverfs 變體（資料目錄路徑），經 build_store 單點建構；sqlite SHALL 維持預設持久層選項。

#### Scenario: 陌生目錄拒用

- **WHEN** 以指向非本 driver 建立之目錄（含任意檔案）的組態啟動 server
- **THEN** 啟動失敗、stderr 指出原因；目錄內容位元不變

#### Scenario: serverfs 組態可服務

- **WHEN** 以 serverfs 變體組態啟動 server 並執行既有 e2e 動詞流程
- **THEN** 行為與 sqlite 組態一致；重啟後資料完整

<!-- @trace
source: serverfs-team-store
updated: 2026-07-15
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/serverfs_store.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-store-fs/Cargo.toml
  - crates/speclink-store-fs/src/layout.rs
  - crates/speclink-store-fs/src/lib.rs
  - crates/speclink-store-fs/tests/atomic_publish.rs
  - crates/speclink-store-fs/tests/bundle_and_outbox.rs
  - crates/speclink-store-fs/tests/conformance.rs
  - crates/speclink-store-fs/tests/single_writer.rs
  - crates/speclink-store-fs/tests/version_gate.rs
  - docs/platform-architecture.zh-TW.md
  - docs/server-store-drivers.zh-TW.md
-->