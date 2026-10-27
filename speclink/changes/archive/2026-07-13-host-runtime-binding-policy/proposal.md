## Why

平台架構藍圖（docs/platform-architecture.zh-TW.md §4.2）定義 Host 是 Engine 對外的唯一應用層邊界：actor 與 project/repo context、lifecycle 裁決、Unit of Work、組合 Engine 與 Store、發布 domain events。現況沒有這一層：Engine 的規格面流程自行讀取本機事實——new change、archive、demo、in-progress、discuss 在 core 內直接呼叫 git config 取得身分；工作流政策的環境變數層（SPECLINK_TDD 等）與 store 模式解析（SPECLINK_STORE_URL）在 core 內直接讀 process env。這是重構路線圖 §3.3 指名的缺口：正式遠端 Server 無法讀 RD 本機的 git identity 與 env，Engine 必須改為接收 Host 解析好的明確 actor 與 EffectiveWorkflowPolicy。同時藍圖 §4.6／§4.7 要求 Project/Repo binding 與 SpeclinkExecutionContext 必須不可含糊——binding 缺失、無權限或多義時停止，模型與 tool arguments 不得選擇或覆寫 identity（§15.1 P0「Tool 可繞過租戶邊界」「Project/Repo scope 不完整」）；§15.1 P0「PM 到 RD handoff 缺少正式 gate」則要求 drafting→review→ready→applying→verified→archived 的生命週期閘門有單一裁決點。若 Phase 2 的 Server 與 Node Host 各自組裝這些規則，Host 正確性必然分叉（路線圖 §3.7：canonical Rust Host 單一實作）。

目標使用者：Phase 2（reference-server）與 Phase 4（N-API Host facade、In-process Tool）的實作者——他們以本刀的 speclink-host 為唯一應用服務層；以及現有 CLI／Node SDK 使用者——他們的可見行為完全不變（身分仍來自 git config、政策仍吃相同環境變數，只是解析點搬到 host 邊界）。

## What Changes

- 新增 `speclink-host` crate（Cargo workspace 成員）：canonical Rust Host 應用服務層——SpeclinkExecutionContext（actor、project/repo binding、mode、resolved policy）、binding 解析與驗證、policy injection、lifecycle gate 裁決。
- ExecutionContext 固定為 Host 產物：actor 與 EffectiveWorkflowPolicy（含 policy digest）由 Host 在邊界解析一次，Engine 全程只消費、不再自行讀取。模型或呼叫端不得經 command 參數傳入或覆寫 identity。
- Engine 去 env／git：core 內 5 個 git identity 呼叫點（new change、archive、demo、in-progress、discuss）改接收明確 actor；工作流政策的環境變數層與 store 模式的環境變數解析改由 Host 邊界注入（core 保留既有的純函式解析）。command runtime 的 execute 入口改攜 ExecutionContext。
- Project/Repo binding contract 型別與 fail-closed 驗證：ProjectId／RepoId 不可變身分、key 為可讀名稱、binding 缺失或多義時拒絕不得自動選第一個；本地 fs 模式映射到固定的 default project/repo（workspace root 即 binding 來源），行為不變。
- lifecycle gate：drafting→review→ready→applying→verified→archived 狀態機型別與 transition 裁決函式落在 Host（單一裁決點）；本地模式以既有站點映射（未開工、started_at、archived），不改任何本地 CLI 可見行為。
- Host 與 TeamStore 契約的組合證明：以 speclink-store 的 in-memory reference 做整合測試——Host 以 ExecutionContext 開 Unit of Work、commit 攜 event records 落 outbox，證明 §3.7 的 UoW／event commit 職責由 Host 承擔（不接線任何現行 CLI 流程）。
- CLI 與 Node dispatch 改經 Host 組裝（識別與政策解析下沉 host 邊界）：人眼與 --json 輸出、exit code、錯誤訊息逐位元不變。

## Non-Goals

- 不做 task stable ID、task-done evidence、VerifyBundle 與 stale evidence（順位 5 stable-task-and-evidence）；lifecycle gate 的 approval 綁定 revision 與 approval 失效規則同屬順位 5 之後。
- 不拆分 spec drift 與 code/git drift（順位 6 drift-client-server-split）——drift 對 git 的讀取本刀不動。
- 不定案 Command/Query/Context/Event Protocol 與 Client SDK、不做 binding handshake 的 HTTP 端點（順位 7 protocol-client-context 與 Phase 2）。
- 不實作 Server、認證授權的真實後端（authorization hook 只留介面位，本地模式恆允許）。
- 不把 policyRevision 與 digest 加進 instructions 或任何現有輸出（藍圖 §4.8 的該要求屬遠端 instructions 路徑，順位 7 起接線；本刀只讓 EffectiveWorkflowPolicy 型別攜帶 digest）。
- 不動 speclink-fs 與既有 Store seam；不動 Context Projection 與 skills。
- 不改任何現行指令的輸出與行為：parity／color／twin 與 baseline 對照必須逐位元全綠。

## Capabilities

### New Capabilities

- `host-runtime`: canonical Rust Host 應用服務層的契約——ExecutionContext 的組成與不可覆寫性、Engine 不讀 process env 與 git identity 的邊界約束、Project/Repo binding 的 fail-closed 驗證、lifecycle gate 狀態機與單一裁決點、Host 對 TeamStore 的 UoW／event commit 組合職責。

### Modified Capabilities

（無）——既有正典的行為需求全部不變；本刀是邊界搬遷與新 Host 層，全部現行輸出凍結。

## Impact

- 影響的 crate：新增 `speclink-host`（依賴 speclink-core 與 speclink-store）；`speclink-core`（execute 攜 ExecutionContext、identity 呼叫點改收 actor、env 解析改注入）；`speclink-cli` 與 `speclink-node`（組裝點改經 host，行為不變）；`speclink-remote` 與 `apps/desktop/core`（global_config_dir 與 git_identity 上移 host、archive/promote 簽名改收 actor 的消費端跟進）；根 Cargo workspace 設定隨成員追加而動。
- 相容性影響：人眼與 --json 輸出、exit code、錯誤訊息逐位元不變；parity 31 項／color 16 項／twin 8 情境必須全綠，遷移前後以 baseline exe 對照。身分來源（git config user.name/email）與政策環境變數（SPECLINK_TDD 等）的效果完全不變，僅讀取位置由 core 內部搬到 host 邊界。
- Affected specs: `host-runtime`（新增）。
- Affected code:
  - New: crates/speclink-host/Cargo.toml、crates/speclink-host/src/lib.rs、crates/speclink-host/src/context.rs、crates/speclink-host/src/binding.rs、crates/speclink-host/src/policy.rs、crates/speclink-host/src/gate.rs、crates/speclink-host/src/commit.rs、crates/speclink-core/tests/no_process_env.rs（零命中靜態盤點）
  - Modified: Cargo.toml、Cargo.lock、crates/speclink-core/src/command/mod.rs（含 in-progress 的 actor 佈線 run_in_progress_add——inprogress.rs 本身已是注入形、未變動）、crates/speclink-core/src/newcmd.rs、crates/speclink-core/src/archive.rs、crates/speclink-core/src/demo.rs、crates/speclink-core/src/discuss.rs、crates/speclink-core/src/util.rs、crates/speclink-core/src/config.rs、crates/speclink-core/src/workspace.rs、crates/speclink-core/src/instructions.rs（env 層改注入）、crates/speclink-core/src/schema.rs（user schemas dir 改注入）、crates/speclink-cli/Cargo.toml、crates/speclink-cli/src/commands.rs、crates/speclink-cli/src/remote_commands.rs（mode 解析改經 host）、crates/speclink-node/Cargo.toml、crates/speclink-node/src/lib.rs、crates/speclink-remote/Cargo.toml、crates/speclink-remote/src/auth.rs（global_config_dir 改取自 host）、apps/desktop/core/Cargo.toml、apps/desktop/core/src/manage.rs（cached_git_identity 委派 host）、apps/desktop/core/src/discussions.rs、apps/desktop/core/src/verbs.rs（promote/archive 改收 actor）、apps/desktop/core/src/query.rs、apps/desktop/core/src/settings.rs（schema 解析傳入 host user dir）
  - Removed: 無
