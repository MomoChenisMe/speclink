# Tasks: add-instructions-and-task-done

每個任務嚴守 TDD（紅→綠→重構）：實作前必先寫對應失敗測試。`[P]` 標記表示該任務可與相鄰 `[P]` 任務並行（不同檔案、無 incomplete 依賴）。

## 1. Hardcoded instructions 內容以 `include_str!` 編入 runtime binary

- [x] 1.1 [P] 新增 `crates/runtime/instructions/proposal.md`，內容含 `## Instruction`、`## Template`、`## Rules` 三段；rules 至少 2 條（如 `proposal.must_include_why`、`proposal.must_list_capabilities`），符合 spec `Hardcoded artifact instructions in runtime` 對非空欄位的要求。**驗證**：`cargo test -p runtime instructions::tests::proposal_instruction_non_empty` 預期紅燈→綠燈。
- [x] 1.2 [P] 新增 `crates/runtime/instructions/design.md`，內容含 `## Instruction`、`## Template`、`## Rules` 三段；rules 至少 2 條。**驗證**：同 1.1 對 design kind。
- [x] 1.3 [P] 新增 `crates/runtime/instructions/tasks.md`，內容含 `## Instruction`、`## Template`、`## Rules` 三段；rules 至少包含「tasks 必含 checkbox」與「task id 必為 N.M 格式」兩條（連結 spec `Tasks.md task id format`）。**驗證**：同 1.1 對 tasks kind。
- [x] 1.4 [P] 新增 `crates/runtime/instructions/spec.md`，內容含 `## Instruction`、`## Template`、`## Rules` 三段；rules 至少包含「使用 SHALL/MUST 而非 should/may」與「Delta heading 限 ADDED/MODIFIED/REMOVED/RENAMED 四種」兩條。**驗證**：同 1.1 對 spec kind。
- [x] 1.5 [P] 在 repo 根目錄新增 `.gitattributes` 規則 `*.md text eol=lf`，避免跨平台 `include_str!` 行尾不一致。**Behavior**：design 章節 ``` Hardcoded instructions 內容以 `include_str!` 編入 runtime binary ``` 的 mitigation。**驗證**：手動檢視 `.gitattributes` + CI 三平台 build 通過。

## 2. `Provider::get_artifact_instructions` 簽章

- [x] 2.1 撰寫 `crates/provider/tests/dyn_provider_compile.rs` 擴充：mock provider 補上 `get_artifact_instructions` 實作，回傳一個假 `ArtifactInstructions`。**Behavior**：spec `` `instructions` command surface `` 與 spec `Hardcoded artifact instructions in runtime` 共同要求的 trait 入口。**驗證**：紅燈。
- [x] 2.2 在 `crates/provider/src/model.rs` 新增 `ArtifactInstructions { artifact_id, kind, output_path, dependencies, unlocks, instruction, template, rules: Vec<InstructionRule>, locale }`、`InstructionRule { id, level: RuleLevel, description }`、`RuleLevel { Error, Warning, Info }`，全部 `serde(rename_all = "camelCase")`，`RuleLevel` 序列化為小寫字串。**Behavior**：spec `` `instructions` JSON output schema `` 對欄位與型別的契約。**驗證**：`model.rs` 單元測試驗證 JSON 欄位順序與名稱（`outputPath` / `instruction` / `template` / `rules` / `locale`）。
- [x] 2.3 在 `crates/provider/src/lib.rs` 的 `Provider` trait 新增 `async fn get_artifact_instructions(&self, project_id: &ProjectId, change_id: &ChangeId, kind: ArtifactKind, capability: Option<&str>) -> Result<ArtifactInstructions, ProviderError>;`。**驗證**：2.1 編譯通過。

## 3. `Provider::mark_task_done` 簽章與 idempotent 行為

- [x] 3.1 撰寫 `crates/provider/tests/dyn_provider_compile.rs` 擴充：mock provider 補上 `mark_task_done` 實作，回傳一個假 `TaskUpdate`。**Behavior**：spec `` `task done` command surface `` 與 spec `Atomic tasks.md update for task done` 共同要求的 trait 入口。**驗證**：紅燈→綠燈。
- [x] 3.2 [P] 在 `crates/provider/src/model.rs` 新增 `TaskUpdate { task_id, previous_status: TaskStatus, current_status: TaskStatus, task_description }`、`TaskStatus { Todo, Done }`，`TaskStatus` 序列化為 `"todo"` / `"done"` 小寫字串。**Behavior**：spec `` `task done` JSON output schema `` 對欄位的契約。**驗證**：`model.rs` 單元測試 `task_status_serializes_lowercase`。
- [x] 3.3 在 `crates/provider/src/lib.rs` 的 `Provider` trait 新增 `async fn mark_task_done(&self, project_id: &ProjectId, change_id: &ChangeId, task_id: &str) -> Result<TaskUpdate, ProviderError>;`。**驗證**：3.1 編譯通過。
- [x] 3.4 [P] 在 `crates/provider/src/error.rs` 新增 `ProviderError` variants：`ArtifactMissing { artifact_id: String, change_id: ChangeId }`、`TaskInvalidId { task_id: String }`、`TaskNotFound { task_id: String }`、`TasksParseError { message: String }`；`error_code()` 對應 `artifact.missing` / `task.invalid_id` / `task.not_found` / `tasks.parse_error`。**驗證**：`error.rs` 單元測試擴充。
- [x] 3.5 [P] 在 `crates/provider-local/src/error.rs` 新增對應 `LocalProviderError` variants 與 `error_code()` mapping。**驗證**：`error.rs` 單元測試擴充。

## 4. tasks.md 的 task id 格式：`N.M`

- [x] 4.1 撰寫 `crates/runtime/src/tasks_parser.rs` 的 `valid_task_id_pattern` 測試：`is_valid_task_id` helper 接受 `1.1`、`10.3`、`100.50`，拒絕 `1`、`1.1.2`、`01.1`、`1.`、`.1`、空字串、含空白、含字母。**Behavior**：spec `Tasks.md task id format` 規定的格式。**驗證**：紅燈。
- [x] 4.2 在 `crates/runtime/src/tasks_parser.rs` 新增 pub `is_valid_task_id(s: &str) -> bool`：以 `split_once('.')` 與 `str::parse::<u32>()` 驗證，拒絕前導 0、拒絕多於一個點、拒絕負數。**驗證**：4.1 綠燈。

## 5. tasks.md 解析在 `crates/runtime/src/tasks_parser.rs`

- [x] 5.1 撰寫 `tasks_parser.rs` 的 `parse_tasks_happy_path` 測試：輸入 `## 1. Setup\n\n- [ ] 1.1 Install deps\n- [ ] 1.2 Configure env\n\n## 2. Build\n\n- [x] 2.1 Compile` 預期 `ParsedTasks` 含 2 sections、3 task items、line_number 對齊。**Behavior**：spec `Tasks.md task id format` 的 valid task id structure scenario。**驗證**：紅燈。
- [x] 5.2 撰寫 `tasks_parser.rs` 的 parse error 測試：(a) `- [ ] 1.1.1 Subtask`（三層）→ Parse error；(b) `## 1. Setup\n\n- [ ] 2.1 Mismatch`（section number 與 task id 開頭不符）→ Parse error；(c) `- [ ] 1.1 Floating`（缺 `## N.` heading）→ Parse error。**Behavior**：spec `Tasks.md task id format` 的 Three-level rejected 與 Section number mismatch detected scenarios + spec `` `task done` failure mapping `` 的 missing section heading scenario。**驗證**：紅燈。
- [x] 5.3 在 `crates/runtime/src/tasks_parser.rs` 實作 `pub fn parse_tasks(content: &str) -> Result<ParsedTasks, TasksParseError>` 與相關 struct（`ParsedTasks`、`TaskSection { number, heading, tasks }`、`TaskItem { task_id, status: TaskStatus, description, line_number }`）：line-by-line scanner 偵測 `^## (\d+)\. (.+)$` 為 section heading、`^- \[( |x)\] (\d+)\.(\d+) (.+)$` 為 checkbox。**驗證**：5.1、5.2 綠燈。

## 6. tasks.md 原子更新策略

- [x] 6.1 撰寫 `crates/runtime/src/tasks_parser.rs` 的 `mark_task_done_happy_path` 測試：輸入 `## 1. Setup\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n` + task_id `1.1` 預期 new_content 為 `## 1. Setup\n\n- [x] 1.1 First\n- [ ] 1.2 Second\n`、previous_status = Todo、task_description = `"First"`。**Behavior**：spec `` `task done` atomic update `` 的 File content preservation scenario。**驗證**：紅燈。
- [x] 6.2 撰寫 `tasks_parser.rs` 的 idempotent 測試：對 `- [x] 1.1 Done` 呼叫 `mark_task_done_in_content("1.1")` 預期 `previous_status = Done`、`new_content == content`。**Behavior**：spec `Atomic tasks.md update for task done` 的 Idempotent update scenario。**驗證**：紅燈。
- [x] 6.3 撰寫 `tasks_parser.rs` 的 not_found / invalid_id 測試：`1.99` 找不到 → `TasksUpdateError::NotFound`；`1.1.2` 格式錯 → `TasksUpdateError::InvalidId`。**Behavior**：spec `` `task done` failure mapping `` 的對應分支。**驗證**：紅燈。
- [x] 6.4 [P] 撰寫 `tasks_parser.rs` 的 `[P]` marker 保留測試：對 `- [ ] 2.3 [P] Refactor parser` 呼叫 `mark_task_done_in_content("2.3")` 預期 `task_description = "[P] Refactor parser"`、new_content 含 `- [x] 2.3 [P] Refactor parser`。**Behavior**：spec `` `task done` JSON output schema `` 的 Task description preserves [P] marker scenario。**驗證**：紅燈。
- [x] 6.5 [P] 撰寫 `tasks_parser.rs` 的「多行 description」測試：對 `- [ ] 1.1 First\n  Continuation\n` 呼叫 `mark_task_done_in_content("1.1")` 預期 `task_description = "First"`（只取 checkbox 同行內容）。**Behavior**：spec `` `task done` JSON output schema `` 的 multi-paragraph notes scenario。**驗證**：紅燈。
- [x] 6.6 在 `tasks_parser.rs` 實作 `pub fn mark_task_done_in_content(content: &str, task_id: &str) -> Result<UpdateResult, TasksUpdateError>` 與 `UpdateResult { new_content, previous_status, task_description }`：先呼叫 `is_valid_task_id` → `parse_tasks` 找對應 line → 在原 content 字串上 single-line 替換 `[ ]` 為 `[x]`、其餘 byte 不動。**驗證**：6.1、6.2、6.3、6.4、6.5 綠燈。
- [x] 6.7 在 `crates/provider-local/src/storage.rs` 新增 pub `update_tasks_atomic(base, change_id, new_content)`：計算 tasks.md 路徑、寫 `.tmp`、rename、失敗 cleanup。**Behavior**：spec `Atomic tasks.md update for task done` 的 6 步驟協議。**驗證**：`storage.rs` 單元測試 `update_tasks_atomic_writes_via_tmp_rename` 紅燈→綠燈。
- [x] 6.8 撰寫 `crates/provider-local/tests/task_done_integration.rs` 的 rollback 測試：將 tasks.md 父目錄設 readonly 模擬 rename 失敗，呼叫 `LocalProvider::mark_task_done` 預期回 `LocalProviderError::Io(_)`、tasks.md 原內容未變、無 `.tmp` 殘留。**Behavior**：spec `Atomic tasks.md update for task done` 的 Atomic rollback on rename failure scenario。**驗證**：紅燈→實作 7.3 後綠燈。

## 7. CLI `speclink task done <task-id>` 子命令

- [x] 7.1 撰寫 `crates/cli/src/cli.rs` 的 clap parse 測試：合法 `speclink task done 1.1 --change demo`、合法 `speclink task done 10.3 --change demo --json`、`speclink task done 1.1 --stdin` 拒絕、`speclink task done 1.1.2 --change demo` 在 clap 層通過（task_id 解析延後到 runtime）、`speclink task done` 缺 task id 拒絕。**Behavior**：spec `` `task done` command surface `` 的 invocation 形式。**驗證**：紅燈。
- [x] 7.2 在 `crates/cli/src/cli.rs` 新增 `Command::Task(TaskCommand)` 與 `TaskCommand::Done(TaskDoneArgs)`、`TaskDoneArgs { task_id: String, change: String, flags: MachineInterfaceFlags }`；`task_id` 為 positional + value_parser `parse_task_id`（呼叫 `is_valid_task_id` 在 clap 層擋三層 id）。**驗證**：7.1 綠燈。
- [x] 7.3 撰寫 `crates/provider-local/tests/task_done_integration.rs` 的 happy / idempotent / not_found / artifact_missing 整合測試：(a) `- [ ] 1.1` 寫入後 mark_task_done → `- [x] 1.1`；(b) 對已 done 的 task id 再次呼叫 → success、tasks.md 不變；(c) 對 `1.99` 不存在的 id → `ProviderError::TaskNotFound`；(d) 對沒寫過 tasks.md 的 change 呼叫 → `ProviderError::ArtifactMissing { artifact_id: "tasks", .. }`。**Behavior**：spec `` `task done` command surface `` 與 spec `` `task done` failure mapping `` 共同描述的 4 個 scenario。**驗證**：紅燈。
- [x] 7.4 在 `crates/provider-local/src/lib.rs` 實作 `Provider::mark_task_done`：透過 `spawn_blocking` 讀 tasks.md → 呼叫 `runtime::tasks_parser::mark_task_done_in_content` → 若 `previous_status == Todo` 呼叫 `storage::update_tasks_atomic`、若 `Done` 直接回 `TaskUpdate`；錯誤 map 為 `LocalProviderError` 再轉 `ProviderError`。**Behavior**：spec `Atomic tasks.md update for task done` 的「idempotent 時略過 .tmp 步驟」條款。**驗證**：7.3 綠燈。
- [x] 7.5 新增 `crates/runtime/src/instructions.rs::{InstructionsInput, get_instructions}`：取 `provider.get_artifact_instructions()` 純轉發。**Behavior**：spec `Hardcoded artifact instructions in runtime` 對 runtime 的轉發契約。**驗證**：`instructions.rs` mock provider 測試 happy path 與 ChangeNotFound 傳遞。
- [x] 7.6 [P] 新增 `crates/runtime/src/task.rs::{MarkTaskDoneInput, mark_task_done}`：取 `provider.mark_task_done()` 純轉發，先在 runtime 層用 `is_valid_task_id` 防禦（雖然 clap 已擋）。**Behavior**：spec `` `task done` command surface `` 的 task id 格式契約。**驗證**：`task.rs` mock provider 測試。
- [x] 7.7 [P] 在 `crates/runtime/src/lib.rs` 透過 `pub mod instructions; pub mod task; pub mod tasks_parser;` 暴露新模組。**驗證**：`cargo build --workspace` 通過。
- [x] 7.8 新增 `crates/cli/src/commands/task.rs::run(args: TaskCommand) -> anyhow::Result<()>`：dispatch Done → resolve provider → 建 LocalProvider → 呼叫 `runtime::task::mark_task_done` → 印 `Envelope<TaskDoneData>`。**Behavior**：spec `` `task done` command surface `` 的整體 CLI 流程。**驗證**：`crates/cli/tests/task_done.rs` assert_cmd 整合測試覆蓋 happy / idempotent / not_found / invalid_id / artifact_missing。

## 8. CLI `speclink instructions <artifact>` 子命令採 positional `<artifact>`

- [x] 8.1 撰寫 `crates/cli/src/cli.rs` 的 clap parse 測試：4 種合法 invocation（proposal / design / tasks / spec+capability）、缺 `--change` 拒絕、design 帶 `--capability` 拒絕、spec 缺 `--capability` 拒絕、`--stdin` 拒絕。**Behavior**：spec `` `instructions` command surface `` 的 invocation 形式。**驗證**：紅燈。
- [x] 8.2 在 `crates/cli/src/cli.rs` 新增 `Command::Instructions(InstructionsCommand)`、`InstructionsCommand::{Proposal(InstructionsArgs), Design(InstructionsArgs), Tasks(InstructionsArgs), Spec(InstructionsSpecArgs)}`、`InstructionsArgs { change, flags }`、`InstructionsSpecArgs { change, capability, flags }`。**驗證**：8.1 綠燈。
- [x] 8.3 新增 `crates/cli/src/commands/instructions.rs::run(args: InstructionsCommand) -> anyhow::Result<()>`：dispatch 到 4 種 kind → resolve provider → 建 LocalProvider → 呼叫 `runtime::instructions::get_instructions` → 印 `Envelope<InstructionsData>`。**Behavior**：spec `` `instructions` command surface `` 的 4 種 happy path 與 spec `` `instructions` failure mapping `` 規定的 5 個失敗分支（change 不存在、change id 非法、spec 缺 capability、非 spec 帶 capability、capability 名稱非法）。**驗證**：`crates/cli/tests/instructions.rs` assert_cmd 整合測試覆蓋 4 種 happy path 與 spec `` `instructions` failure mapping `` 的全部 5 個失敗分支。
- [x] 8.4 撰寫 `crates/provider-local/tests/instructions_integration.rs` 的 4-kind 整合測試：對 design / spec 兩個 kind 呼叫 `LocalProvider::get_artifact_instructions`，斷言回傳 `ArtifactInstructions` 含非空 instruction / template / rules、`output_path` 為預期 POSIX 路徑、spec kind 的 `artifact_id == "spec:auth"`、locale = `"Traditional Chinese (繁體中文)"`。**Behavior**：spec `Hardcoded artifact instructions in runtime` 的 each kind produces non-empty scenario。**驗證**：紅燈。
- [x] 8.5 在 `crates/provider-local/src/lib.rs` 實作 `Provider::get_artifact_instructions`：根據 kind 從 runtime 取得對應 `include_str!` 內容、parse 出 instruction/template/rules（runtime 提供 `compose_local_instructions` helper）、組裝 `ArtifactInstructions`、計算 `dependencies` / `unlocks` / `output_path` 依固定規則。**驗證**：8.4 綠燈。
- [x] 8.6 在 `crates/runtime/src/instructions.rs` 新增 `pub fn compose_local_instructions(kind: ArtifactKind, change_id: &ChangeId, capability: Option<&str>) -> Result<ArtifactInstructions, InstructionsError>`：負責讀 `include_str!` 內容、parse `## Instruction` / `## Template` / `## Rules` 三段、根據 kind 套用固定 dependencies/unlocks。**Behavior**：spec `Hardcoded artifact instructions in runtime` 的 runtime 內部結構（spec 允許 internal format 變動，本任務釘 MVP 版本）。**驗證**：`instructions.rs` 單元測試 4 種 kind 的 compose 結果非空。

## 9. Error code 新增清單

- [x] 9.1 [P] 在 `crates/cli/src/exit_code.rs::classify_provider` 與 `classify_local` 為新 variants 加 mapping：`ArtifactMissing` → (1, `artifact.missing`)、`TaskInvalidId` → (2, `task.invalid_id`)、`TaskNotFound` → (2, `task.not_found`)、`TasksParseError` → (1, `tasks.parse_error`)。**Behavior**：design 章節 `Error code 新增清單` 列舉的新 codes。**驗證**：`exit_code.rs` 既有測試擴充 + `all_error_codes_match_naming_regex` 涵蓋新 codes。

## 10. Integration、snapshot、output 型別與 polish

- [x] 10.1 [P] 在 `crates/cli/src/output.rs` 新增 `InstructionsData { artifact_id, kind, output_path, dependencies, unlocks, instruction, template, rules: Vec<InstructionRuleJson>, locale }`、`InstructionRuleJson { id, level, description }`，皆 `serde(rename_all = "camelCase")`。**Behavior**：spec `` `instructions` JSON output schema ``。**驗證**：`output.rs` 單元測試 JSON 序列化欄位。
- [x] 10.2 [P] 在 `crates/cli/src/output.rs` 新增 `TaskDoneData { change_id, task_id, previous_status, current_status, task_description }`，`serde(rename_all = "camelCase")`、`previous_status` / `current_status` 序列化為 `"todo"` / `"done"`。**Behavior**：spec `` `task done` JSON output schema ``。**驗證**：`output.rs` 單元測試。
- [x] 10.3 [P] 在 `crates/cli/src/commands/mod.rs` 註冊 `pub mod instructions; pub mod task;`、`main.rs` 加 `Command::Instructions(_) | Command::Task(_)` dispatch。**驗證**：`cargo build --workspace` 通過。
- [x] 10.4 [P] 撰寫 `crates/cli/tests/instructions_snapshots.rs`：4 條 insta snapshot — design success、tasks success、spec success（含 `spec:user-auth`）、change_not_found failure。固定 `SPECLINK_TEST_REQUEST_ID`。**驗證**：`cargo insta accept --workspace` 後 `cargo test --workspace` 通過。
- [x] 10.5 [P] 撰寫 `crates/cli/tests/task_done_snapshots.rs`：3 條 insta snapshot — task done success（todo → done）、already done idempotent、task.not_found failure。**驗證**：同上。
- [x] 10.6 [P] 為新公開 API（`Provider::get_artifact_instructions`、`Provider::mark_task_done`、`ArtifactInstructions`、`InstructionRule`、`RuleLevel`、`TaskUpdate`、`TaskStatus`、`runtime::instructions::get_instructions`、`compose_local_instructions`、`runtime::task::mark_task_done`、`runtime::tasks_parser::{parse_tasks, mark_task_done_in_content, is_valid_task_id}`、`Envelope::data` 兩個新型別）補繁體中文 `///` doc comment。**驗證**：`cargo doc --workspace --no-deps` 無 missing doc warning。
- [x] 10.7 [P] 更新 `README.md`：在 archive 範例後新增 instructions（design 與 spec）與 task done 範例三段。**驗證**：手動 review。
- [x] 10.8 跨平台 path 處理稽核：`outputPath` JSON 欄位走 `to_posix_string` helper；tasks.md 與 instructions 都不出現硬編 path 分隔符。**Behavior**：spec `Local provider directory layout` 的 Cross-platform path separator handling scenario（沿用既有規則）。**驗證**：CI 三平台矩陣綠燈。
- [x] 10.9 跑 `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets` 全綠。**驗證**：CI 通過。
