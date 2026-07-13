## Context

現況的 `speclink_core::store::Store`（crates/speclink-core/src/store.rs）是刻意同步、object-safe 的文件 seam：約 37 個方法、讀取回 Option／Vec／bool、以 PathBuf 暴露顯示位置，`speclink-fs` 是唯一產品實作、`teststore` 是測試替身。engine-typed-core 之後，命令層已有 typed outcome／error 與 domain events；change-metadata-fail-closed 之後全部設定與 metadata 解析 fail closed。平台架構 §4.3 固定了 TeamStore 的概念契約與三級能力等級，§15.3 列出每個 adapter 必須受測的失敗模型；重構路線圖 §3.1 明定「不在現有每個 method 直接加 expectedRevision，而是分層：Local DocumentStore 保留、TeamStore 另立契約」。本刀是 Phase 1B：交付契約、參考實作與 conformance suite——沒有任何現有流程接線（接線屬順位 4 host 刀與 Phase 2 server 刀）。

## Goals / Non-Goals

**Goals:**

- 一個獨立的 `speclink-store` crate 承載 TeamStore 契約的唯一 Rust 定義：typed error、scope 定址、snapshot、UoW／CAS、immutable history、outbox、manifest／capabilities、export/import。
- conformance suite 以可重用形式輸出，並以 in-memory reference store 證明可執行——路線圖 §4.3 要求它在第一個 SQLite driver 前就能跑。
- 六類 gate 情境（CAS race、mixed snapshot、partial commit、outbox failure、crash recovery、tenant scope）全部有故障注入測試。
- 既有 crates 原始碼零改動；cargo test --workspace 自動涵蓋新 crate。

**Non-Goals:**

- 不實作任何產品 driver（SQLite／Server FS／PostgreSQL 屬 Phase 2）；不動 N-API facade。
- 不接線 speclink-core 的任何 command 到 TeamStore；不動既有 Store trait 與 speclink-fs。
- 不定案 Client Protocol DTO；不做 Host 啟動驗證（本刀只交付可供驗證的 manifest 宣告面）。
- 不做事件訂閱與 transport；outbox 只保證持久化、cursor 重讀與確認。
- 不引入 async runtime 與任何網路相依。

## Decisions

### 決策一：契約落點為獨立 crate speclink-store，與 speclink-core 零相依

新 crate 只含契約型別、trait、錯誤、in-memory reference 與 conformance。不依賴 speclink-core：commit 攜帶的事件以自含的 event record 型別表達（事件名、JSON 載荷、actor、UTC 時間戳），而非 core 的 typed domain event——store 邊界上事件是「待持久化的記錄」，不是領域邏輯。這使未來 speclink-core／speclink-host 可以反向依賴本 crate 而無循環；driver crates（speclink-store-sqlite 等）只需依賴本 crate。

替代方案：(a) 契約放 speclink-core 的新模組——core 與契約互為相依的風險在 Phase 1C 接線時爆發（core 需要 TeamStore 型別、契約需要 core 事件型別），且 driver 被迫拖入整個引擎；(b) 直接改造既有 Store trait——路線圖 §3.1 明文不建議，且會打破輸出凍結與 store-abstraction 正典。取捨：多一個 crate 的維護面，但這正是「storage 解耦的規格驅動引擎」的分界石。

### 決策二：契約維持同步、object-safe

trait 全部同步方法、可作 trait object（與 store.rs 現有立場一致：引擎不帶 async runtime）。單節點 driver（SQLite、Server FS）天然同步；Phase 2 server 的 async 邊界以 spawn_blocking 或同等機制在 Host adapter 層處理；Node async Store bridge 在 N-API facade 層轉接（Phase 4）。

替代方案：async trait（async_trait 或 RPITIT）——把 tokio 生態拖進最底層契約 crate，違反 core 無 async runtime 的既有約束，且 object safety 與版本相容更複雜。真正需要 async driver 時以 contract version 演進另議，不預先設計。

### 決策三：typed error 為封閉的 Store 錯誤集合，與 command 錯誤碼分層

錯誤型別 StoreError 為封閉 enum：not_found、permission_denied、revision_conflict（帶 expected 與 actual revision）、unavailable、corrupt（帶原因）、backend（帶來源描述）——穩定錯誤碼字串隨型別提供。所有讀取回 Result；「存在性查詢」以 Ok 內的 Option 表達不存在、以 Err 表達故障，兩者不再混同。此集合是 store 層語彙，不與命令層五碼（invalid_argv 等）合併——Host（順位 4 刀）負責兩層映射。

替代方案：沿用 anyhow 字串錯誤——conformance 無法斷言錯誤分類，driver 間語意必然漂移，被拒；直接復用命令層錯誤碼——store 層沒有 argv 概念、命令層沒有 revision_conflict，兩層語彙不同構，硬併徒增誤映射。

### 決策四：定址採 Project／Repo scope 加邏輯 document locator

文件身分為三元組：ProjectId、RepoId、DocumentId。DocumentId 是封閉 enum，涵蓋現有領域文件種類：change metadata、change artifact（change 名＋artifact 相對名）、canonical spec（capability 名）、live／archived discussion（slug）、workflow config、archived change 文件。契約不出現 PathBuf；顯示位置由 store 以字串提供（僅供 UI 呈現，不作身分）。本地單機情境映射到固定的 default project／repo。

替代方案：沿用字串路徑作身分——跨媒介（DB row、物件儲存）沒有路徑，且路徑正規化差異（分隔線、大小寫）會變成隱性身分分裂，被拒；開放字串 kind——失去封閉 enum 的窮舉檢查，driver 對未知 kind 的行為不可測，被拒。

### 決策五：Unit of Work 為唯一寫入路徑，commit 是唯一原子點

寫入流程固定為：begin unit of work（攜 command 識別與 actor）→ 對 UoW 暫存寫入／刪除（每筆攜 expected revision；新建攜「不得已存在」語意）→ commit（帶 event records）或 rollback。commit 的原子保證：全部文件寫入、project revision 遞增、每文件 immutable history 追加（actor、UTC 時間戳、內容 digest、來源 command、刪除以 tombstone 表達）、outbox 追加——全部生效或全部不生效。任一 expected revision 不符即整體以 revision_conflict 拒絕（回衝突文件、expected 與 actual）。snapshot(scope) 回傳帶單一 revision 的一致視圖；讀 snapshot 不受並行 commit 影響（讀到的是固定時點）。rollback 與版本回退一律以新 revision 表達，不覆寫歷史。

替代方案：逐 method 加 expectedRevision（路線圖 §3.1 明文不建議——archive 這種多文件操作仍無原子性）；樂觀寫入加事後補償——partial commit 正是 P0 要消滅的失敗模式，被拒。

### 決策六：manifest 宣告 capabilities 與能力等級，conformance 按宣告分級執行

manifest 回傳 contract version、driver 識別、capability 集合（snapshot、cas、transaction、history、outbox、migration、backup、cluster）與宣告的能力等級（local-single-writer／single-node／cluster）。conformance suite 讀 manifest 決定執行組：宣告 single-node 以上者六類 gate 情境全跑，缺任一必要 capability 或測試失敗即整體判不通過——「宣告了就必須做到」，不存在部分通過的 Team mode。Host 啟動驗證（拒絕能力不足的 Team mode）屬順位 4 刀，本刀交付宣告面與判定邏輯。

替代方案：能力以文件約定不進型別——回到「名義上通過型別，實際語意不同」的 P0 缺口，被拒。

### 決策七：conformance suite 與 in-memory reference 同 crate 交付，故障注入為第一級設計

conformance 以可重用函式庫形式放在 crates/speclink-store/src/conformance/（對外可用，未來 driver 與自訂 Store 以相同入口執行；本 crate 的測試以 in-memory reference 呼叫同一入口）。in-memory reference 是契約的最小正確實作（測試基建、非產品 driver），內建故障注入點：commit 各階段（文件寫入後、history 追加後、outbox 追加前後）可注入崩潰，重啟語意以「重建 store 後不變式仍成立」模擬。六類 gate 情境對應：CAS race（兩 UoW 競寫同文件，恰一成功、失敗方拿到衝突詳情）、mixed snapshot（讀方在寫方 commit 前後各取 snapshot，各自內部一致）、partial commit（注入中途崩潰後無半套狀態外洩）、outbox failure（outbox 追加失敗時整個 commit 不生效）、crash recovery（崩潰重啟後 revision／history／outbox 三者一致）、tenant scope（跨 project／repo 讀寫被 not_found 或 permission_denied 隔離，絕不串資料）。

替代方案：conformance 只作本 crate 內部測試——自訂 Store 與未來 driver 無法復用同一基準，違反藍圖「共用 conformance suite」要求，被拒。

### 決策八：export/import 為 versioned bundle，round-trip 屬 conformance

export(scope) 回傳帶格式版本、scope、project revision 與逐文件 digest 的 bundle；import 驗證版本與 digest、以指定模式（全新建立或覆蓋既有）套用，結果回報逐文件結果。conformance 含 round-trip 情境：export 後 import 到全新 store，逐文件內容與 history 起點一致。備份排程、增量與跨版本遷移工具不在本刀。

## Implementation Contract

- **行為**：本刀交付後，工作區多一個 `speclink-store` crate；cargo test -p speclink-store 執行契約單元測試與 conformance suite（以 in-memory reference 為受測體）且全綠；cargo test --workspace 與 npm run test:all 全綠且既有測試零變動。任何現行 CLI 指令、Node dispatch、桌面行為與輸出完全不變。
- **介面／資料形狀**：TeamStore trait（同步、object-safe）：manifest()、health()、migrate(target_version)、snapshot(scope)、begin_unit_of_work(command_ctx)、commit(uow, event_records)、rollback(uow)、export(scope)、import(bundle, mode)、outbox 讀取（自 cursor 起）與確認。StoreError 封閉 enum 六類含穩定錯誤碼字串；DocRef 三元組（ProjectId、RepoId、DocumentId 封閉 enum）；manifest 含 contract version、capabilities、能力等級。命名遵循 Rust 慣例：型別 PascalCase、方法 snake_case。
- **失敗模式**：CAS 不符回 revision_conflict 帶 expected／actual；commit 任一階段失敗即整體不生效（無半套狀態）；跨 scope 存取以 not_found 或 permission_denied 隔離；conformance 對「宣告能力但測試失敗」判整體不通過。
- **驗收**：conformance 六類 gate 情境各至少一個故障注入測試，全綠；in-memory reference 通過完整 suite；speclink-core 等既有 crates 的 git diff 為空（Cargo.toml／Cargo.lock 的 workspace 成員追加除外）；parity／color／twin 對照不需重跑（無行為變更），root npm run test:all 全綠。
- **範圍邊界**：in scope——契約型別、trait、錯誤、in-memory reference、conformance suite、export/import 型別與 round-trip；out of scope——產品 driver、Host 接線、Client Protocol、事件 transport、N-API facade、speclink-core 任何改動。

## Risks / Trade-offs

- [契約先行、無真實 driver 驗證，形狀可能不合 SQLite/PostgreSQL 實作] → conformance suite 本身就是可執行的契約消費者；Phase 2 第一個 driver 若揭露形狀問題，以 contract version 演進處理，不回頭改語意；in-memory reference 刻意保持最小、避免把記憶體實作的便利假設寫進契約。
- [同步契約與未來 async server 的阻抗] → 決策二明定 Host adapter 層以 spawn_blocking 轉接；契約 crate 零 async 相依讓轉接成本可控。
- [event record 與 core domain event 形狀日後分叉] → record 只承載序列化事件（名稱＋JSON 載荷＋actor＋時間戳），不重複定義事件語意；Phase 1C 接線時由 host 做單向映射，正典事件種類仍以 command-runtime 規格為準。
- [conformance 的崩潰模擬（in-memory）與真實檔案系統／DB 崩潰語意有落差] → 故障注入點設在契約規定的 commit 階段邊界而非實作細節；Phase 2 driver 另有各自媒介的 crash fixtures（藍圖 §15.3），本刀不宣稱涵蓋媒介層。
- [新 crate 讓 workspace 編譯時間增加] → 契約 crate 零外部重相依（serde、chrono 級別），增量成本可忽略。

## Migration Plan

純新增，無遷移：不動任何既有資料格式與行為。回滾即移除 workspace 成員。後續刀的採用路徑：順位 4 host 刀開始消費 ExecutionContext 與本契約；Phase 2 driver 實作本契約並跑同一 conformance。

## Open Questions

（無）
