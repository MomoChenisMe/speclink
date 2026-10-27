## Context

TeamStore 契約由 speclink-store crate 定義：TeamStore trait（manifest、health、migrate、snapshot、begin_unit_of_work、commit、rollback、revision history、outbox read/ack、export/import）、typed 錯誤六類（not_found、permission_denied、revision_conflict、unavailable、corrupt、backend）、以及 conformance suite（crates/speclink-store/src/conformance/mod.rs 的 run 函式，經 StoreHarness trait 驅動，含 arm_crash 故障注入四點：AfterDocWrites、AfterHistoryAppend、BeforeOutboxAppend、AfterOutboxAppend）。目前唯一實作是 in-memory reference（crates/speclink-store/src/memory.rs），僅供測試。平台藍圖 §13.1 指定 SQLite 為官方 server 預設持久層；路線圖 §4.3 要求 SQLite 先過 conformance，再以同套 suite 做 ServerFS/PostgreSQL。

## Goals / Non-Goals

**Goals:**

- 一個以單一 SQLite 資料庫檔持久化的 TeamStore driver，契約全方法實作、conformance suite 全綠。
- 崩潰復原是真實的：故障注入後重開資料庫檔案（新連線），驗證不留 partial commit、outbox 與文件同進退。
- schema 版本化：資料庫記錄 schema version，開啟未知版本 fail closed。

**Non-Goals:**

- 不做 HTTP server、auth、Project/Repo registry（server-http-adapter 刀）。
- 不做 ServerFS 與 PostgreSQL driver（後續刀以同套 conformance 複製本刀範本）。
- 不做多程序併發寫入支援——manifest 如實宣告 single-node；cluster 能力不宣告。
- 不接線任何現行 CLI/桌面流程；本刀交付的是 library crate 與其測試。
- 不做 backup 排程與 restore validation 工具（reference-server 後續子刀）；export/import 契約方法本身在範圍內。

## Decisions

### 決策 1：獨立 crate，依賴 rusqlite（bundled）

driver 落在新 crate crates/speclink-store-sqlite，只依賴 speclink-store（契約與 conformance）與 rusqlite。rusqlite 啟用 bundled feature，SQLite 以原始碼靜態連結——server 發布目標是單一 binary 與 Docker image（藍圖 §13.1），不能依賴系統 libsqlite3 的版本差異。speclink-store 維持零依賴 core 的分層；本 crate 同樣不依賴 speclink-core。

### 決策 2：schema——文件、歷史、outbox、meta 四張表

單一資料庫檔內：documents（scope＋doc id 為主鍵，content、revision、digest）、history（revision records，append-only）、outbox（單調遞增 cursor 主鍵，event records）、meta（schema_version、store 識別）。DocumentId 與 Scope 序列化為穩定字串鍵（沿用 DocRef 的既有表示），JSON 欄位存放 event payload 與 bundle metadata——契約的 content_digest（SHA-256）由 speclink-store 提供，不在 SQL 層重算。

### 決策 3：WAL 模式＋單一寫入連線，commit 是一個 SQL transaction

開啟時設 journal_mode=WAL 與 foreign_keys=ON。commit(uow, events) 在單一 SQL transaction 內完成：逐 op 檢查 expected revision（CAS）、寫 documents、append history、append outbox——任何一步失敗整筆 rollback，revision 檢查失敗回 revision_conflict（帶 expected/actual）。SQLite transaction 的原子性即是 partial-commit gate 的實作基礎；不自行實作 write-ahead 或雙檔案協定。

### 決策 4：崩潰注入以「毒化連線＋強制重開」實作

conformance 的 arm_crash 要求在四個故障點模擬崩潰。測試 harness 的做法：driver 內建 test-only 故障鉤（以 #[cfg] 或 harness 專用建構子注入），觸發時在該點直接放棄連線（不 commit、程序內丟棄 Connection），harness 隨即以同一路徑開全新連線重建 store 再驗證狀態。因為所有寫入都在單一 SQL transaction 內，四個故障點的可觀察結果一致：整筆 commit 不存在——這正是契約要的「無 partial commit」。故障鉤不進 release 介面。

### 決策 5：schema version 開檔守門

meta 表記錄 schema_version（本刀為 1）。開啟資料庫時：無 meta 表且資料庫為空 → 初始化 schema；version 低於現行且有 migrate 路徑 → 僅在呼叫 migrate 時升級（health 回報 needs-migration）；version 高於現行或 meta 損毀 → 回 corrupt（帶原因），絕不寫入。與 change-metadata-fail-closed 同一原則：存在但不可解讀的持久化狀態不得靜默視為預設。

### 決策 6：conformance 是唯一驗收面

不為 driver 另寫一套行為測試敘事：tests/conformance.rs 以 StoreHarness 掛接 SQLite driver（tempdir 資料庫檔）執行 speclink-store 的 run，報告全綠即驗收。driver 專屬的少量補充測試只涵蓋 conformance 無法表達的邊界：未知 schema version 開檔、WAL 檔案存在時的重開、同程序兩個 store 實例指向同檔的拒絕或序列化行為。

## Implementation Contract

- Behavior：以檔案路徑建構 SqliteTeamStore；同一路徑重開可見先前 commit 的全部文件、歷史與 outbox；崩潰（任一故障點）後重開狀態等價於該筆 commit 從未發生。
- Interface / data shape：實作 speclink-store 的 TeamStore trait 全方法，無額外公開語意；建構子形如 open(path) -> Result<SqliteTeamStore, StoreError>；manifest 宣告 single-node 等級與 snapshot、cas、transaction、history、outbox、migration、backup 能力。
- Failure modes：路徑不可寫或 SQLite I/O 錯誤 → backend（帶來源描述）；資料庫檔存在但非 speclink schema 或 version 不可解讀 → corrupt（帶原因）；CAS 失敗 → revision_conflict（expected/actual）；檔案暫時鎖住 → unavailable。全部沿用 speclink-store 的封閉錯誤集合，不新增錯誤類別。
- Acceptance criteria：cargo test -p speclink-store-sqlite 全綠，其中 conformance 報告 0 failure（含 crash recovery 四故障點與 tenant scope）；cargo build --workspace 不影響其他 crate。

## Risks / Trade-offs

- rusqlite bundled 增加編譯時間與 binary 體積 → 換取發布免依賴系統 SQLite，符合單 binary 目標。
- 單一寫入連線序列化所有寫入 → single-node 定位下可接受；吞吐瓶頸留待 PostgreSQL driver 解決，不在 SQLite 層做連線池複雜化。
- test-only 故障鉤存在於 driver 原始碼 → 以 cfg 隔離、不出現在公開 API；比起在 SQL 層攔截 VFS 簡單一個數量級，且驗證目標（transaction 原子性）相同。

## Migration Plan

純新增 crate，無既有資料要遷移。資料庫 schema version 1 是起點；未來 schema 演進走 meta 表版本與 migrate 方法。上線與回退都只是「用或不用這個 crate」。

## Open Questions

（無）
