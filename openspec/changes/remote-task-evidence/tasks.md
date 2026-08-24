## 1. TeamStore contract：change evidence 文件種類

- [x] 1.1 先寫紅測試：conformance suite 增 change evidence 案例——任一官方 driver 的 UoW 寫入後 round-trip 逐位元組一致、tenant scope 隔離、export bundle 自動包含（crates/speclink-store/src/conformance/mod.rs）；此時 in-memory reference 未實作、案例應紅 <!-- speclink-task:tsk_01M0PBMRMGXVCJF5AGQ03ECEKD -->
- [x] 1.2 crates/speclink-store/src/types.rs 增 DocumentId::ChangeEvidence { change }（封閉集合第九種，落實 teamstore-contract 的「文件定址採 Project 與 Repo scope 的邏輯 locator」），crates/speclink-store/src/memory.rs 與 crates/speclink-store/src/uow.rs 落實編碼／解碼與列舉；驗證：cargo test -p speclink-store 全綠（含 1.1 案例轉綠） <!-- speclink-task:tsk_01M0PBMRMGCK83BHW34ZDRES40 -->

## 2. 三個官方 driver 映射

- [x] 2.1 sqlite driver 的 ChangeEvidence locator 映射與快照列舉（crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store-sqlite/src/schema.rs）；驗證：cargo test -p speclink-store-sqlite --test conformance 全綠 <!-- speclink-task:tsk_01M0PBMRMGBTVYP5RK8BFDMGP5 -->
- [x] 2.2 serverfs driver 同步（crates/speclink-store-fs/src/lib.rs、crates/speclink-store-fs/src/layout.rs），檔案佈局遵循既有 change 層級文件慣例、不改鎖語意；驗證：cargo test -p speclink-store-fs --test it 全綠 <!-- speclink-task:tsk_01M0PBMRMG23P80V7QNG3F9D1J -->
- [x] 2.3 postgres driver 同步（crates/speclink-store-postgres/src/lib.rs、crates/speclink-store-postgres/src/schema.rs）；驗證：設 SPECLINK_TEST_POSTGRES_URL 後 cargo test -p speclink-store-postgres --test it 全綠，本機無 PostgreSQL 時記下跳過原因、由 CI 守門 <!-- speclink-task:tsk_01M0PBMRMGS4P6SFNSP561BFYW -->

## 3. Engine：候選注入與 evidence 走 Store seam

- [x] 3.1 先寫紅測試：Command::TaskDone 攜 Host 注入候選時不探本機 workspace、歸屬過濾（僅未被先前任務認領的新髒檔）對注入與探測兩來源同一套；未注入時本地行為凍結——.evidence.json 位置、v2 格式、legacy 回退讀取皆不變（crates/speclink-core/src/command/mod.rs 與 crates/speclink-core/src/tasks.rs 的 #[cfg(test)] 模組） <!-- speclink-task:tsk_01M0PBMRMGAYP9EY9Q8HQT3AYD -->
- [x] 3.2 Command::TaskDone 增 touched_files 選填欄位；tasks 的 complete 候選來源改雙軌（注入優先、否則 workspace 探測）；evidence 讀寫收進 crates/speclink-core/src/store.rs 的 Store seam（落實 verify-evidence 的「store 模式的 evidence 記錄與查詢」），本地 fs supplier 映射 change 目錄 .evidence.json（含 legacy 路徑唯讀回退與清除語意，逐位元組不變）；驗證：cargo test -p speclink-core 全綠、crates/speclink-core/tests/golden 無 diff <!-- speclink-task:tsk_01M0PBMRMGSQ6JZJANEWEP6R1Y -->
- [x] 3.3 CLI 回歸對照：task done 的 argv、人眼輸出與 --json 欄位不變，模式分岔宣告未動；驗證：cargo test -p speclink-cli --test it 全綠（含 remote_verb_parity） <!-- speclink-task:tsk_01M0PBMRMGGK4WJZZRTZ7M6WZQ -->

## 4. Host bridge 與 drift 消費

- [x] 4.1 bridge 把 Store seam 的 evidence 寫入 staged 為 ChangeEvidence 操作、與 tasks.md 勾選及 task-completed 事件同一 UoW 原子 commit；封存經 seam 讀 evidence 後以 ArchivedChange 相對名 .evidence.json 落檔、discard 隨 change 文件一併刪除；drift 的 evidence_summary 改經 seam 讀取（crates/speclink-host/src/bridge.rs、crates/speclink-host/src/drift.rs）；驗證：cargo test -p speclink-host 全綠 <!-- speclink-task:tsk_01M0PBMRMG0SSETM2C2V7CSB7W -->

## 5. Server 端點與事件面

- [x] 5.1 先轉紅測試為現行斷言：去掉 phase2_chain.rs 中 task_done_with_touched_files_leaves_queryable_evidence_on_the_server 的 #[ignore]，斷言指向 outbox 的 task-completed payload 攜 touchedFiles 與新 evidence 端點可讀回（crates/speclink-server/tests/it/phase2_chain.rs） <!-- speclink-task:tsk_01M0PBMRMG395TBQWB1J0W7VM0 -->
- [x] 5.2 routes 的 task_done 消費請求 payload 的 touchedFiles 填入 Command（不再 Json(_req) 丟棄）；DomainEvent::TaskCompleted 增 touchedFiles（additive、無候選不偽造）；新增 GET /changes/{name}/evidence 唯讀端點——viewer 以上、回應 camelCase（#[serde(rename)]）、記錄缺席回空集合非 not_found——落實 server-verb-api 的「task done 消費 touchedFiles 且 evidence 有唯讀端點」（crates/speclink-server/src/routes.rs、crates/speclink-core/src/command/mod.rs 的事件組裝）；驗證：cargo test -p speclink-server --test it 全綠（含 5.1 轉綠），並斷言 payload 欄位存在、camelCase 命名與型別 <!-- speclink-task:tsk_01M0PBMRMGA26R12SB2V4Q7VB6 -->

## 6. 文件漂移修正（中英同步）

- [x] 6.1 product-status 兩列改判：Desktop Remote Workspace 依實況重寫（chooser 建構路徑與 skip／folder 兩模式已於 2026-07-19 落地）、Remote task evidence 於本刀落地後改 Available，查核日期更新（docs/product-status.zh-TW.md、docs/product-status.md） <!-- speclink-task:tsk_01M0PBMRMGCAW3B988MMBSA8BN -->
- [x] 6.2 remote-getting-started 第 6 節改寫為已可用的遠端開啟流程——skip（免 checkout）與 folder（綁 checkout）兩模式與其邊界（docs/remote-getting-started.zh-TW.md、docs/remote-getting-started.md） <!-- speclink-task:tsk_01M0PBMRMGBT44GBSPXZYX7ZWZ -->
- [x] 6.3 roadmap 遠端協作線改寫「目前到哪」與「可觀察的下一步」（evidence 缺口以本刀閉合、桌面看板已可指向遠端），並把 apps/desktop/src/session.ts 頂部「remote 變體無建構路徑」過期註解改為指向現行 chooser／remote_open 建構路徑（docs/roadmap.zh-TW.md、docs/roadmap.md、apps/desktop/src/session.ts） <!-- speclink-task:tsk_01M0PBMRMG8WW10B8Y5EGY3N1Y -->

## 7. 收尾

- [x] 7.1 橫跨多面收尾：補跑 npm run test:all 一次全綠；git status 盤點差異僅含本 change Impact 所列檔案、docs 中英六檔皆動過、無孤兒殘留 <!-- speclink-task:tsk_01M0PBMRMGFYDD31KT0SCTB21D -->
- [x] 7.2 品質站補救落地：drift 回應增列 evidence 欄位（server-drift-api「規格面 drift 端點且工作區面不進 wire」的 store 面輸入集，delta scenario「store 保存的 evidence 隨回應下行」）、TaskDoneRequest 增選填 headCommit（verify-evidence「headCommit 由 wire 攜入」）、worktree overlay 的 evidence 寫入直通主 store、Injected 候選的 wire 輸入過濾；驗證：cargo test --workspace 全綠（含 drift_api 與 phase2_chain 的新斷言） <!-- speclink-task:tsk_01M0T2B66JYS3VDSN0Q5QA6FN7 -->
