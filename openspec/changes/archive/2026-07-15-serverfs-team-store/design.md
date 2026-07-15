## Context

TeamStore 契約與 conformance suite（crates/speclink-store）是唯一驗收面：能力檢查、CAS race、mixed snapshot、partial commit（AfterDocWrites 崩潰後 commit 必須從未發生）、outbox failure（立即失敗且 store 續用）、crash recovery（四故障點全有或全無）、tenant scope。SQLite driver（crates/speclink-store-sqlite）以 SQL transaction 承擔原子性、conformance 全綠，是可複製範本。既有 speclink-fs crate 是本地引擎 Store seam（openspec/ 佈局），與本刀的 server 端 TeamStore driver 是不同層的東西——命名沿藍圖 §13.4 用 speclink-store-fs。server 的 driver 接線在 build_store 單點（StoreConfig enum）。§15.3 對 FS driver 的特別義務：NAS 暫時失聯、檔案鎖失效、mtime 精度不足要文件化並受測。

## Goals / Non-Goals

**Goals:**

- conformance 報告零 failure——與 SQLite 同一套 suite、同一 StoreHarness 介面、同樣的真實重開驗證。
- 原子性不依賴任何資料庫：單一原子發布點（索引檔替換）承擔全有全無。
- mtime 與檔案時間戳完全不參與語意；NAS 失聯是可恢復錯誤不是損毀。

**Non-Goals:**

- 不做多程序/多節點共享寫入（single-node 單寫者；cluster 不宣告）。
- 不做檔案佈局的人類可編輯性承諾：目錄是 driver 私有格式，手改視同損毀（版本守門與 digest 攔截）；要人類可讀用 export bundle。
- 不做壓縮、去重或歷史裁剪（有量測證據再議）。
- 不動 conformance suite 與 TeamStore 契約；suite 不足以表達的 FS 特有邊界以 driver 專屬測試補充，不回頭改共同 suite。
- 不動既有 speclink-fs crate（本地引擎 seam，概念不同層）。

## Decisions

### 決策 1：每 scope 一份索引檔是唯一事實指標

資料目錄佈局：根下 meta 檔（driver 識別與 schema version）、每 scope 一個子目錄——內含 index 檔（JSON：每文件的 current revision 與內容檔參照、下一 revision 序號、outbox 已寫至序號、acked cursor）、documents 目錄（每 revision 一個不可變內容檔，檔名含 revision）、history 目錄（每 revision 一筆記錄檔）、outbox 目錄（每序號一筆記錄檔）。讀取面（snapshot）= 讀一次 index 後按參照讀內容檔——index 是一致性的邊界，mixed snapshot gate 由「單次讀 index」滿足。排序與身分全部出自 index 與檔名內的序號，mtime 不參與任何判斷。

### 決策 2：commit 的原子發布點是 index 檔替換

commit(uow, events)：CAS 檢查（對 index 現值）→ 新內容檔、history 記錄檔、outbox 記錄檔全部寫入並 fsync（此時皆為未被引用的惰性檔案）→ 新 index 寫入暫名檔並 fsync → 原子 rename 蓋過舊 index。四個故障點（AfterDocWrites、AfterHistoryAppend、BeforeOutboxAppend、AfterOutboxAppend）都發生在 rename 之前——崩潰後重開讀到舊 index，commit 從未發生，孤兒檔案（未被任何 index 引用的 revision/outbox 檔）於開啟時掃描清除。這與 conformance 的 partial-commit gate（AfterDocWrites 崩潰後內容必須是舊值）及 crash-recovery gate（全有或全無）語意一致，且與 SQLite driver 的可觀察結果相同。

### 決策 3：advisory 檔案鎖單寫者，殘留鎖以持有者活性判定

資料目錄內鎖檔以 OS advisory lock（flock 語意）取得排他——lock 隨程序死亡自動釋放，天然處理「持有者已死的殘留鎖」；鎖檔本身不寫入任何內容（pid／時間這類持有者資訊會與現實不同步，而 kernel 持有的 lock 才是唯一事實）。鎖不可得（另一程序持有）回 unavailable——不等待、不搶占。NFS 等 advisory lock 語意不可靠的檔案系統：文件明示部署前提（本地碟或支援 flock 的掛載），不嘗試以 lock file 存在性做自製鎖（mtime/存在性鎖正是 §15.3 點名的失效模式）。

### 決策 4：I/O 失敗分類與 NAS 失聯測試

讀寫過程的 I/O 錯誤映射：目錄不存在或權限拒絕 → backend（帶來源描述）；暫時性失敗（如掛載點消失）→ unavailable；meta/index 存在但不可解析或版本較新 → corrupt（帶原因）拒用不寫。NAS 失聯以測試模擬：對資料目錄改權限/移走子目錄使操作失敗，斷言回錯不 panic、既有狀態位元不變、恢復後重開即續用。mtime 竄改測試：對全部檔案 touch 任意時間戳後重開，讀取面與排序不變。

### 決策 5：conformance 掛接與 sqlite 範本同構

tests/conformance.rs 以 StoreHarness 掛接（tempdir 資料目錄）：arm_crash 以 test-only 故障鉤在指定階段放棄操作（不執行 rename），harness 開全新實例重開驗證；arm_outbox_failure 在 outbox 記錄檔寫入時注入錯誤，斷言 commit 整筆失敗且 store 續用（無需重開）。export/import 走契約 Bundle 與 content_digest，與 driver 佈局無關。driver 專屬補充測試只涵蓋 suite 不表達的邊界：鎖競爭、mtime 竄改、NAS 失聯、孤兒清理、版本守門。

## Implementation Contract

- Behavior：以資料目錄路徑開啟 store；同目錄重開可見全部成功 commit；任一故障點崩潰後重開等價於 commit 未發生；第二個程序開同目錄得 unavailable；export bundle 與 SQLite driver 對同語意內容產生相同 digest。
- Interface / data shape：實作 TeamStore trait 全方法，開啟入口形如 open(dir) -> Result；manifest 宣告 single-node 等級與 snapshot/cas/transaction/history/outbox/migration/backup；server 組態 store 段新增 serverfs 變體（path 欄位），未知 driver 拒啟動的既有行為涵蓋拼錯名。
- Failure modes：鎖被他程序持有 → unavailable；目錄權限/不存在 → backend；meta 或 index 損毀、版本較新、非本 driver 目錄 → corrupt 拒用不寫；I/O 中途失敗 → 該 commit 失敗、舊狀態完好。
- Acceptance criteria：cargo test -p speclink-store-fs 全綠且 conformance 報告零 failure；cargo test -p speclink-server 全綠（組態接線）；npm run test:all 全綠且既有凍結零 diff。

## Risks / Trade-offs

- 每 commit 一次 index 全量重寫 → 規格庫量級（文件數百）下 index 是小 JSON，可接受；巨型 scope 的增量 index 留待量測。
- 孤兒檔案清理在開啟時掃描 → 崩潰頻繁的環境有暫時磁碟殘留；清理冪等且不影響正確性。
- flock 在部分網路檔案系統語意不可靠 → 部署文件明示前提；不自製更差的鎖。
- 目錄可被人手動編輯 → 版本守門與 index 參照使亂改大機率成 corrupt 明錯，不靜默吞壞資料。

## Migration Plan

純新增 crate 與組態變體；與 postgres-team-store 刀可平行（共享接點僅 workspace Cargo 清單與 StoreConfig enum，衝突面是兩行級）。上線即「組態選 serverfs」；回退改回 sqlite——資料遷移用既有 export/import bundle（driver 無關）。

## Open Questions

（無）
