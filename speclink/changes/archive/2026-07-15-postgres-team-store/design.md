## Context

TeamStore 契約與 conformance suite 是唯一驗收面（同 sqlite/serverfs 刀）；SQLite driver 的四表 schema（documents、history、outbox、meta）與「commit 即單一 transaction」策略是直接範本。工作區目前無任何 async runtime 進 store 層（engine/host/store 全同步；async 只在 speclink-server 的 axum 邊界）。CI 是 GitHub Actions（G0 刀建立的全量測試工作流）。§13.2 規定 PostgreSQL password 等 deployment secret 優先來自環境變數或 secret file；§13.1 規定 PostgreSQL 在 distributed coordination 前不宣稱 cluster mode。

## Goals / Non-Goals

**Goals:**

- conformance 零 failure，行為與 SQLite/ServerFS driver 可觀察等價（同語意內容同 bundle digest）。
- 跨連線單寫者以資料庫原生機制（advisory lock）保證，不自製鎖表。
- 測試對真實 PostgreSQL 執行且不可能靜默假綠：跳過必須顯性、CI 必跑。

**Non-Goals:**

- 不做 cluster mode、多寫者、串流複寫感知（distributed coordination 屬後續 Phase；manifest 如實只宣告 single-node）。
- 不做連線池調校介面（單寫者定位下固定小池；有量測證據再開放參數）。
- 不做 PostgreSQL 版本相容矩陣（支援下限定一個近代主版本並在文件明示；CI 以該版本跑）。
- 不做 LISTEN/NOTIFY 事件推播整合（server 的事件廣播已以 outbox 為唯一事實來源，driver 不越層）。
- 不動 conformance suite 與契約；不動既有 driver。

## Decisions

### 決策 1：同步 postgres client，async 不進 store 層

使用同步的 postgres crate（阻塞 I/O），維持「engine/host/store 無 async runtime」的分層鐵律——server 的 axum handler 本就以 spawn_blocking 呼叫 store，driver 同步正好吻合。TLS 依 postgres crate 的原生支援啟用（連線 URL sslmode 尊重）。

### 決策 2：schema 與 SQLite 同構，commit 單一 transaction

四表同構（documents、history、outbox、meta，各帶 scope 欄位；outbox 序號以 per-scope 單調序列實作）。commit(uow, events) 在單一 SQL transaction 內：取 scope 的 advisory lock → CAS 檢查 → documents upsert → history append → outbox append → COMMIT；任一步失敗整筆 abort，revision_conflict 帶 expected/actual。隔離等級用預設 READ COMMITTED 即可——寫入序列化由 advisory lock 保證，讀取一致性由 snapshot 查詢的單一 repeatable 讀 transaction 保證（mixed snapshot gate）。

### 決策 3：transaction-scoped advisory lock 作單寫者

寫入 transaction 開頭以 pg_advisory_xact_lock（key 由 scope 派生的穩定 64 位雜湊）序列化同 scope 寫入——transaction 結束（commit 或 abort）自動釋放，程序崩潰即連線斷、鎖隨之消失，無殘留鎖問題。跨 scope 寫入互不阻塞。讀取不取鎖。與 §15.3「檔案鎖失效」類比的資料庫故障（連線中斷）由 transaction abort 天然處理。

### 決策 4：崩潰模擬＝該點放棄連線

conformance 的 arm_crash 四故障點以 test-only 故障鉤實作：該點直接丟棄連線（不 COMMIT），server 端 transaction abort——harness 隨後以全新連線建 store 驗證「commit 從未發生」。與 SQLite 刀的「毒化連線＋強制重開」同構；arm_outbox_failure 在 outbox append 語句注入錯誤，斷言整筆失敗且 store 續用。

### 決策 5：測試基建——環境變數指向實例，缺席顯性略過，CI 必跑

conformance 與整合測試讀取 SPECLINK_TEST_POSTGRES_URL：有值即對該實例執行（每測試用獨立 schema 隔離、測後清除）；缺席時測試以顯性 skipped 結束並印出啟用指引（docker run 一行命令），不靜默假綠也不硬紅本地無 PG 的環境。CI 工作流新增 PostgreSQL service container 並設該變數——CI 上此測試集必跑，紅燈即擋。root 的 test:all 不強制本地 PG（維持跨機器可跑），文件註記完整驗證需 PG。

### 決策 6：secret 紀律——URL 可省密碼，環境變數補全

server 組態 store 段的 postgres 變體宣告連線 URL；密碼解析順序：URL 內嵌（允許但啟動時 stderr 警告「建議改用環境變數」）→ SPECLINK_POSTGRES_PASSWORD 環境變數。組態檔不可解析或 URL 形狀不合沿用啟動 fail closed。連線失敗（認證錯、庫不存在）啟動失敗印原因；運行中連線中斷回 unavailable，/readyz 轉紅，恢復後續用。

## Implementation Contract

- Behavior：以 postgres 組態啟動的 server 行為與 sqlite 組態一致（既有 e2e 動詞流程、重啟資料完整）；兩個 server 程序指向同庫時寫入互斥由 advisory lock 序列化（不宣稱 cluster，但不損毀）；連線中斷期間請求得 unavailable、恢復後續用。
- Interface / data shape：實作 TeamStore trait 全方法，開啟入口形如 connect(url) -> Result；manifest 宣告 single-node 等級與 snapshot/cas/transaction/history/outbox/migration/backup；組態 store 段 postgres 變體（url 欄位）；SPECLINK_TEST_POSTGRES_URL 與 SPECLINK_POSTGRES_PASSWORD 環境變數。
- Failure modes：認證失敗/庫不存在 → 啟動失敗（backend 帶原因）；運行中連線中斷 → unavailable、不 panic、恢復續用；meta 版本較新或陌生 schema → corrupt 拒用不寫；CAS 敗 → revision_conflict 帶 expected/actual；測試環境無 PG → 顯性 skipped 附啟用指引。
- Acceptance criteria：SPECLINK_TEST_POSTGRES_URL 就緒時 cargo test -p speclink-store-postgres 全綠且 conformance 零 failure；CI 工作流含 PG service 且該測試集必跑；cargo test -p speclink-server 全綠；npm run test:all 全綠且既有凍結零 diff。

## Risks / Trade-offs

- 測試依賴外部 PG 實例 → 顯性 skip＋CI 必跑的組合守住覆蓋；風險是本地開發者忘開 PG 而漏測，skip 訊息與文件補救。
- advisory lock key 是 scope 派生雜湊，理論碰撞使不相關 scope 互相序列化 → 64 位空間下機率可忽略，且後果只是效能非正確性。
- 同步 client 佔用 spawn_blocking 執行緒 → 與 sqlite driver 同模式，single-node 定位可接受。
- 兩程序同庫不被拒絕（advisory lock 只序列化不獨占）→ 與 manifest 的 single-node 宣告一致性靠文件與部署紀律；正確性（CAS、transaction）不受影響，故不做啟動時獨占鎖——cluster 刀屆時正式處理多節點。

## Migration Plan

純新增 crate 與組態變體；與 serverfs-team-store 刀可平行（共享接點僅 workspace Cargo 清單與 StoreConfig enum）。既有部署自 sqlite 遷 postgres 用 export/import bundle（driver 無關）。回退改組態即可。

## Open Questions

（無）
