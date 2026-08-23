## Why

架構藍圖 §9.4 承諾 Remote Store 保存 task completion 回報的 touched-file evidence，但 server 端點 task_done 目前把 CLI 已送達的 touchedFiles 直接丟棄（routes 的 Json(_req)），engine 在無本機 workspace 的 server 上也不落任何 evidence 記錄——每一次遠端 task done，「這個任務動了哪些檔案」就永久流失、事後補不回。crates/speclink-server/tests/it/phase2_chain.rs 以 #[ignore] 紅測試釘著此缺陷。同批修正：Day 12 盤點揭露三份使用者文件對 Desktop 遠端工作區的記載已陳舊（該能力 2026-07-19 即落地），依討論 remote-partial-gaps-priority 結論與本刀併修。

目標使用者是透過 AI 代理跑 SDD 的團隊開發者：在遠端模式下勾任務的人（CLI 或 Desktop 遠端看板），以及之後靠 drift、commit 檔案歸屬與封存 trace 回溯實作證據的人。對應 workflow 的 apply（task done）與 drift／archive 階段。

## What Changes

- Engine 的 TaskDone 命令增帶 Host 注入的 touched-file 候選清單，沿 host-runtime-binding-policy 的「Host 在邊界解析、Engine 全程消費」原則：本地 Host 照舊由 git 狀態探得候選；server Host 改從 wire 上的 TaskDoneRequest.touchedFiles 收下，不再丟棄。候選的歸屬過濾（僅未被先前任務認領的新髒檔）留在 Engine，兩模式同一套。
- evidence 寫入改走 Store seam：本地 supplier 照舊落 change 目錄的 .evidence.json（可觀察行為不變）；TeamStore bridge 把 evidence 作為 change-scoped 文件隨 Unit of Work 原子 commit，並隨 change 的封存／廢棄同生命週期移動。
- task-completed outbox 事件 payload 增列 touchedFiles（additive，事件消費端不需遷移）。
- server 端 evidence 成為可查事實：host drift 的 evidence_summary 在 remote 模式開始有值；phase2_chain.rs 的 #[ignore] 測試轉綠並把斷言指向新的 evidence 查詢面。
- 文件漂移修正（中英兩語同步）：product-status 的 Desktop Remote Workspace 列與 remote task evidence 列、remote-getting-started 第 6 節「登入後開不出遠端看板」敘述、roadmap 遠端協作線，以及 Desktop session 模組頂部「remote 變體無建構路徑」的過期註解。

## Non-Goals

- Desktop 遠端看板勾任務時的本機 git 探測（checkout 綁定下的 touched 收集）不在本刀：desktop 遠端勾任務照舊不送 touchedFiles，沿「無新髒檔不新增記錄」語意呈現，不偽造。
- 不做 evidence 回填工具——已丟失的歷史補不回，不追溯。
- 不做離線佇列或任何「先存後送」語意（既有紅線）。
- Desktop 遠端工作區剩餘小縫（changeCapabilities／changeMeta 不支援、討論 promotedTo 空清單、離線衝突完成度）的盤點與立案屬討論結論的第三步，不在本刀。
- 已棄用的 legacy remote REST v1 旁路不加 evidence 支援。
- 無新增 CLI 子指令與旗標；task done 的 argv 面與人眼輸出不變。

## Capabilities

### New Capabilities

(none) — 規格掃描：最近的 verify-evidence（evidence 語意正典）、teamstore-contract（儲存契約）、server-verb-api（動詞端點面）皆已存在且直接覆蓋本刀範圍，全為修改、無新 capability。

### Modified Capabilities

- `verify-evidence`: evidence 記錄與本機 workspace 解耦——touched 候選由 Host 注入、寫入走 Store seam，store 模式下 evidence 隨 change 文件同生命週期；task-completed 事件攜帶 touchedFiles。
- `teamstore-contract`: 文件模型增 evidence 文件的定址與讀寫，contract 由 conformance suite 釘死（三 driver 同步）。
- `server-verb-api`: task done 端點消費請求 payload 的 touchedFiles，server 端 evidence 可查。

## Impact

- 影響 crate：speclink-core、speclink-fs、speclink-host、speclink-server、speclink-protocol、speclink-remote、speclink-node、speclink-cli、speclink-store（含 conformance suite）、speclink-store-sqlite、speclink-store-fs、speclink-store-postgres；apps/desktop（core 的 complete 呼叫端＋前端註解修正）；文件面在 docs/。evidence 收進 Store seam 後，每個 Store supplier 與每個 `Command::TaskDone` 建構點都在波及面內——這比立案時預估的窄清單廣，實際落點見下方 Affected code。
- 相容性影響：wire contract 不變（TaskDoneRequest 早已攜 touchedFiles，僅 server 開始消費）；task-completed 事件 payload 增欄位為 additive，既有訂閱者無需遷移；task done 的人眼輸出與 --json 皆不變，無回歸對照破壞；本地 .evidence.json 檔案格式與位置不變。
- Affected specs: verify-evidence、teamstore-contract、server-verb-api（皆 MODIFIED）
- Affected code:
  - Modified: crates/speclink-core/src/store.rs、crates/speclink-core/src/tasks.rs、crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/archive.rs、crates/speclink-core/src/drift.rs、crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/teststore.rs、crates/speclink-fs/src/lib.rs、crates/speclink-fs/src/layout.rs、crates/speclink-fs/tests/store_fs.rs、crates/speclink-host/src/bridge.rs、crates/speclink-host/src/commit.rs、crates/speclink-host/src/drift.rs、crates/speclink-host/src/worktree.rs、crates/speclink-host/tests/bridge_dual_path.rs、crates/speclink-protocol/src/query.rs、crates/speclink-remote/src/client.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/src/backup.rs、crates/speclink-server/tests/it/phase2_chain.rs、crates/speclink-node/src/store_bridge.rs、crates/speclink-node/index.d.ts、crates/speclink-cli/src/verbs/progress.rs、crates/speclink-cli/src/verbs/station.rs、crates/speclink-cli/src/verbs/checks.rs、crates/speclink-store/src/types.rs、crates/speclink-store/src/conformance/mod.rs、crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store-fs/src/layout.rs、crates/speclink-store-postgres/src/lib.rs、apps/desktop/core/src/manage.rs、apps/desktop/src/session.ts、docs/product-status.zh-TW.md、docs/product-status.md、docs/remote-getting-started.zh-TW.md、docs/remote-getting-started.md、docs/roadmap.zh-TW.md、docs/roadmap.md
  - New: (none)
  - Removed: (none)
