## ADDED Requirements

### Requirement: SQLite driver 通過 TeamStore conformance

SQLite driver SHALL 實作 TeamStore 契約全數方法，並以 StoreHarness 掛接契約的 conformance suite 執行至全綠——涵蓋能力檢查、CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 與 tenant scope 全部情境。manifest SHALL 宣告 single-node 等級，capability 集合 SHALL 含 snapshot、cas、transaction、history、outbox、migration 與 backup；SHALL NOT 宣告 cluster。

#### Scenario: conformance suite 全綠

- **WHEN** 以 tempdir 內的資料庫檔建構 SQLite driver 並執行 conformance suite
- **THEN** 報告零 failure；manifest 檢查確認 single-node 等級與必要 capability 全數存在

#### Scenario: 跨租戶 scope 隔離

- **WHEN** 兩個不同 Scope（project／repo 組合）寫入同名文件後各自讀取
- **THEN** 各 scope 只見自己的文件與 revision；任一 scope 的 outbox 與 history 不含另一 scope 的記錄

---
### Requirement: commit 原子性與崩潰復原

commit(uow, events) SHALL 在單一資料庫 transaction 內完成 CAS 檢查、文件寫入、history append 與 outbox append；任何一步失敗 SHALL 使整筆 commit 不留任何可觀察痕跡。程序於 commit 途中任一點崩潰後，以同一檔案路徑重開的 store SHALL 呈現「該筆 commit 從未發生」的狀態——文件、history 與 outbox 三者一致，SHALL NOT 出現只有部分落盤的組合。

#### Scenario: 故障注入後重開無 partial commit

- **WHEN** 於 AfterDocWrites、AfterHistoryAppend、BeforeOutboxAppend、AfterOutboxAppend 四個故障點分別注入崩潰（放棄連線不 commit），再以同一路徑開全新連線
- **THEN** 四個情境重開後讀取一致：該筆 commit 的文件內容、history 記錄與 outbox 事件全數不存在

#### Scenario: CAS 衝突回 revision_conflict

- **WHEN** 兩個 commit 以相同 expected revision 競寫同一文件
- **THEN** 恰一方成功；敗方收到 revision_conflict 錯誤且帶 expected 與 actual revision；敗方 UoW 的其他 op 也未落盤

---
### Requirement: 持久化與重開一致性

同一資料庫檔路徑重開的 store SHALL 呈現先前全部成功 commit 的文件、revision、history 與未 ack 的 outbox 事件；outbox cursor 的 ack 進度 SHALL 持久化。driver SHALL 以 WAL journal 模式開啟；殘留的 WAL/SHM 檔案 SHALL 在重開時由 SQLite 正常回放，SHALL NOT 被當成損壞。

#### Scenario: 重開後狀態完整

- **WHEN** commit 三筆文件變更並 ack 第一筆 outbox 事件後關閉 store，以同一路徑重開
- **THEN** 三筆文件與其 revision/history 完整可讀；自持久化 cursor 續讀 outbox 只得到未 ack 的兩筆事件

---
### Requirement: schema 版本守門 fail closed

資料庫 SHALL 記錄 schema version。開啟時：空資料庫 SHALL 初始化為現行版本；version 低於現行 SHALL 由 health 回報需要 migration、並僅在呼叫 migrate 時升級；version 高於現行、meta 記錄損毀、或檔案存在但非本 driver 的 schema SHALL 回 corrupt 錯誤（帶原因）且 SHALL NOT 寫入任何內容。

#### Scenario: 未知較新版本拒開

- **WHEN** 開啟一個 meta 記錄 schema version 為現行版本加一的資料庫檔
- **THEN** 開啟失敗回 corrupt 錯誤且原因指出版本不相容；檔案內容位元不變

#### Scenario: 非 speclink 資料庫拒開

- **WHEN** 開啟一個存在但由其他應用建立的 SQLite 資料庫檔
- **THEN** 開啟失敗回 corrupt 錯誤；SHALL NOT 初始化 schema 或寫入該檔
