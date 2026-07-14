# host-runtime Specification

## Purpose

TBD - created by archiving change 'host-runtime-binding-policy'. Update Purpose after archive.

## Requirements

### Requirement: ExecutionContext 由 Host 解析且不可覆寫

SpeclinkExecutionContext（actor、project、repo、mode、resolved workflow policy 含 digest）SHALL 由 Host 在進入點解析一次；Engine 的 command 輸入 SHALL NOT 含 actor 或 policy 欄位，呼叫端與模型 SHALL NOT 能經 command 參數傳入或覆寫 identity。本地 fs 模式的 actor SHALL 沿用現行 git config 身分語意：無 git 或未設 user.name 時為無章匿名，行為與現行一致。

#### Scenario: command 無從攜帶 identity

- **WHEN** 檢視命令層的 Command 輸入型別並嘗試以任意 command 參數影響蓋章身分
- **THEN** Command 封閉 enum 不存在 actor 或 policy 欄位；蓋章內容只隨 ExecutionContext 的 actor 改變

#### Scenario: 本地 actor 語意不變

- **WHEN** 在設有 git user.name 與 user.email 的 workspace 執行 new change，與移除 git 身分後再執行一次
- **THEN** 前者的 created_by 章與現行版本逐位元一致；後者沿用現行無章行為


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Engine 規格面不讀 process env 與 git identity

speclink-core 的非測試碼 SHALL NOT 直接讀取 process 環境變數或執行 git 身分查詢：工作流政策的環境變數層與 store 模式解析 SHALL 以 Host 注入的查找值運作，git 身分 SHALL 以 ExecutionContext 的 actor 供給。政策與模式的解析效果 SHALL 與現行完全一致（僅讀取位置改變）。

#### Scenario: 注入值決定政策而非 process env

- **WHEN** 以注入的環境覆寫集合（含 SPECLINK_TDD）呼叫政策解析，同時 process env 設定相反的值
- **THEN** 解析結果只反映注入集合；process env 的相反值無效果

#### Scenario: core 無殘留直讀

- **WHEN** 對 speclink-core 非測試碼執行 process env 讀取與 git config 呼叫的靜態盤點
- **THEN** 盤點結果為零命中（測試模組與 host crate 除外）


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Project 與 Repo binding 驗證 fail closed

binding 解析 SHALL 在缺失、無權限或存在多個候選時以帶原因的錯誤拒絕，SHALL NOT 自動選擇第一個候選。本地 fs 模式 SHALL 以 workspace root 映射固定 default project 與 repo，無需任何新設定且行為不變。ProjectId 與 RepoId SHALL 為不可變身分，key 僅為可讀名稱。

#### Scenario: 多義 binding 拒絕

- **WHEN** binding 解析面對兩個同時合格的候選 repo
- **THEN** 回帶「多個候選」原因的拒絕錯誤並列出候選，不自動選擇

#### Scenario: 本地 default binding 零設定

- **WHEN** 於無任何 remote 設定的本地 workspace 建立 ExecutionContext
- **THEN** binding 解析成功映射 default project/repo；全部現行本地動詞行為不變


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: lifecycle gate 是單一裁決點

生命週期 SHALL 以封閉的六站狀態機表達：drafting、review、ready、applying、verified、archived；站間 transition SHALL 經 Host 的單一裁決函式判定，非法 transition SHALL 回帶原因的拒絕。本地模式 SHALL 提供唯讀站點推導（未開工＝drafting、已標記開工＝applying、已封存＝archived），且本刀 SHALL NOT 以 gate 裁決改變任何現行動詞的行為。

#### Scenario: 非法 transition 拒絕

- **WHEN** 對處於 drafting 的變更請求直接 transition 到 verified
- **THEN** 裁決函式回拒絕並指出缺少的中間站；合法路徑（drafting→review→ready→applying→verified→archived）逐步請求則全數允許

#### Scenario: 本地站點唯讀推導

- **WHEN** 對「未開工」「已標記開工」「已封存」三種本地變更狀態執行站點推導
- **THEN** 分別得到 drafting、applying、archived；推導不寫入任何檔案


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Host 承擔 TeamStore 的 UoW 與 event commit

Host SHALL 提供對 TeamStore 契約的 commit 組合路徑：以 ExecutionContext 開 unit of work、將領域事件單向映射為 event records、經 commit 原子落入文件與 outbox；TeamStore 的 revision_conflict SHALL 原樣傳遞為 Host 錯誤。本刀的 commit 路徑 SHALL NOT 接線任何現行 CLI 流程。

#### Scenario: commit 後 outbox 含對應事件

- **WHEN** Host 以 ExecutionContext 對 in-memory reference store 開 UoW 寫入一份文件並帶一筆領域事件 commit
- **THEN** commit 成功後自 cursor 0 重讀 outbox 得到恰一筆對應 event record（含 actor 與事件名）；文件與事件同 commit 可見

#### Scenario: CAS 衝突原樣傳遞

- **WHEN** 兩個 Host commit 以相同 expected revision 競寫同一文件
- **THEN** 敗方收到的 Host 錯誤保留 revision_conflict 分類與 expected/actual 詳情


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: 組裝點遷移輸出凍結

CLI 與 Node dispatch 改經 Host 組裝後，全部現行動詞的人眼輸出、--json 輸出、exit code 與錯誤訊息 SHALL 與遷移前逐位元一致；政策環境變數與 git 身分的可觀測效果 SHALL 不變。

#### Scenario: baseline 對照逐位元一致

- **WHEN** 對同一樣本 workspace 於遷移前後執行覆蓋表動詞（人眼與 --json 兩形式，含設定 SPECLINK_TDD 與 git 身分的情境）
- **THEN** stdout、stderr 與 exit code 逐位元一致；parity、color 與 twin 回歸對照全綠

<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: engine 動詞經橋接於 TeamStore 上執行

Host SHALL 提供 engine-over-TeamStore 執行橋接：以 TeamStore snapshot 供應 engine 命令層的讀取視圖、把變更型動詞的寫入捕捉為 UnitOfWork staged ops、成功時連同領域事件經 Host 的 commit 組合路徑原子提交。同一動詞對語意相同的內容分別經本地 fs seam 與經橋接執行，typed outcome、錯誤分類與領域事件 SHALL 一致；TeamStore 的 revision_conflict SHALL 映射為命令層錯誤且保留 expected/actual 詳情。橋接 SHALL NOT 分叉 engine 命令層的動詞語意，發現的檔案系統暗依賴 SHALL 修在橋接視圖。

#### Scenario: 雙路徑 outcome 一致

- **WHEN** 對含相同 change 內容的 fs workspace 與 TeamStore scope 分別執行同一查詢動詞與同一變更型動詞
- **THEN** 兩路徑的 typed outcome 結構相等、變更型動詞回報相同種類的領域事件；失敗情境（如 not_found）的錯誤碼相同

#### Scenario: 橋接寫入原子落店

- **WHEN** 經橋接執行 task done 成功
- **THEN** 任務勾選後的文件內容、revision 遞增與 task-completed 事件記錄在同一 commit 內可見；commit 前 store 無任何中間狀態

<!-- @trace
source: server-http-adapter
updated: 2026-07-14
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/health.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
-->