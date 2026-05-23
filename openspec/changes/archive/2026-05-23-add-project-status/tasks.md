# Tasks — add-project-status

嚴格 TDD：每個實作 task 都先寫失敗 test，再寫實作。

## 1. Catalogue Operation struct 擴 outputs_schema 欄位 + 37 個 outputs fn + render 更新（Decision: catalogue schemas 對應升級；Decision: catalogue Operation struct 擴 `outputs_schema: fn() -> Value` 欄位（B 方案）；Requirement: catalogue `project_status` SHALL expose both inputs and outputs schemas via describe-tools）

- [x] 1.1 在 `crates/runtime/src/catalogue/mod.rs` 為 `Operation` struct 加 `pub outputs_schema: fn() -> Value` 欄位（與既有 `inputs_schema` 並列、非 Option）。此步驟會讓既有 37 個 Operation const initializer 編譯 fail — 屬預期紅燈（對應 Requirement: catalogue Operation struct enforces outputs_schema for every entry）。
- [x] 1.2 在 `crates/runtime/src/catalogue/schemas.rs` 新增 `fn empty_object_outputs_schema() -> Value`（reuse 既有 `empty_object_schema` 模式）；新增 37 個 `<op>_outputs() -> Value` 函式（命名規則：對應既有 inputs fn 後綴 `_outputs`），35 個 stub op 直接 `empty_object_outputs_schema()`，`project_status_outputs()` 寫 operations.md §1389 完整 schema（七個 required field），`change_show_outputs()` 寫 envelope shape（含 change/artifacts/all_tasks_done/next_actions）。
- [x] 1.3 在 `crates/runtime/src/catalogue/mod.rs` 為 37 個 `Operation` const entry 全部加 `outputs_schema: schemas::<op>_outputs,` 行；run `cargo build -p speclink-runtime` 確認結構 compile 過（紅綠轉綠）。
- [x] 1.4 在 `crates/runtime/src/catalogue/mod.rs` 既有 unit test 補：(a) `outputs_schema_pointers_all_non_empty` — 對 37 個 op 呼叫 `(op.outputs_schema)()` assert 不 panic、回 valid Value；(b) `project_status_outputs_required_has_six_names` — assert `project.status` outputs schema 的 `required` array 含六 name；(c) `change_show_outputs_properties_has_all_tasks_done_and_next_actions`。
- [x] 1.5 在 `crates/runtime/src/tool_ops/render.rs::JsonRenderer` 加 `outputs_schema` 鍵在 `parameters` 旁同層；確認 `CopilotSdkRenderer` 不動（assert via 既有 `render_copilot_sdk_emits_name_description_parameters_only` test 仍綠）。
- [x] 1.6 [P] 在 `crates/runtime/src/tool_ops/render.rs` 新增 unit test `render_json_includes_outputs_schema_sibling`：對 sample op 呼叫 JsonRenderer，assert 結果 JSON 物件含 `outputs_schema` key 與 `parameters` key 並列。
- [x] 1.7 跑 `cargo test -p speclink-runtime catalogue tool_ops::render` 確認 1.4 + 1.6 全綠。

## 2. Runtime project_ops 新模組（Decision: project.status 走 runtime-side aggregation，不新增 provider trait method；Decision: runtime 新建 project_ops 模組；change.show 兩欄位住既有 change_ops）

- [x] 2.1 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：fresh project（zero change rows）→ `project_status()` 回 `changes_count` 六欄全 0；`discussions_count` 等於 `{active:0, converged:0}`；`current_change` 為 None。對應 Requirement: `speclink status` SHALL emit a stable envelope conforming to the project.status output schema。
- [x] 2.2 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：seed 11 rows（1 in_progress + 2 ready + 8 archived）→ `changes_count` 對應 6 欄正確。對應 Requirement: `project.status` SHALL aggregate change counts from state.db grouped by lifecycle state（Decision: project.status 走 runtime-side aggregation，不新增 provider trait method）。
- [x] 2.3 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：seed in_progress + actor.host_id == 當前 instance_id → `current_change.change_id` 等於該 row；state 為 `"in_progress"`。對應 Requirement: `project.status` SHALL populate `current_change` only when an in_progress change matches the current host（Decision: current_change 採 actor.host_id 比對；無則 null happy path）。
- [x] 2.4 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：seed in_progress + actor.host_id ≠ 當前 instance_id → `current_change` 為 None（Decision: current_change 採 actor.host_id 比對；無則 null 反例）。
- [x] 2.5 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：seed 兩個 in_progress（皆當前 host）→ `current_change` 取 updated_at 最新者。
- [x] 2.6 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：working_dir 不在 SpecLink 專案內 → RuntimeError 對應 `project.not_initialized`。對應 Requirement: `speclink status` SHALL reject invocation outside a SpecLink project with `project.not_initialized`。
- [x] 2.7 在 `crates/runtime/tests/project_ops.rs` 寫 failing test：seed 任意 row → 回 `discussions_count == {active:0, converged:0}`。對應 Requirement: `project.status` SHALL freeze `discussions_count` at zero pending discuss-ops implementation（Decision: discussions_count 此 slice 永遠回 `{active: 0, converged: 0}`、欄位保留）。
- [x] 2.8 [P] 新建 `crates/runtime/src/project_ops.rs` module，定義 `pub struct ProjectStatusData` + `pub struct CurrentChangeRef` + `pub struct ChangesCountByState` + `pub struct DiscussionsCountByState`，serde rename snake_case；`crates/runtime/src/lib.rs` 加 `pub mod project_ops`（Decision: runtime 新建 project_ops 模組；change.show 兩欄位住既有 change_ops）。
- [x] 2.9 實作 `pub async fn project_status(working_dir: &Path) -> Result<ProjectStatusData, RuntimeError>`：呼叫 `LocalChangeStore::list_changes()` + `get_project_metadata` + 讀 config.yaml `schema.active`；in-memory group-by `state` 算 changes_count；找 in_progress + host 匹配 row 取最新 updated_at 填 current_change；discussions_count 寫死 `{active:0, converged:0}`（Decision: discussions_count 此 slice 永遠回 `{active: 0, converged: 0}`、欄位保留；Decision: project.status 走 runtime-side aggregation，不新增 provider trait method）。
- [x] 2.10 跑 `cargo test -p speclink-runtime --test project_ops` 確認 2.1–2.7 全綠。

## 3. change_ops::show_change 加 all_tasks_done + next_actions（Decision: change.show 加 all_tasks_done 從 change 表 column 直接讀；Decision: change.show next_actions 採 state-driven 查表 + in_progress 掃 tasks.md 取第一個 pending task；Requirement: `change.show` response envelope SHALL include `all_tasks_done` and `next_actions`）

- [x] 3.1 在 `crates/runtime/tests/change_ops.rs` 既有 `show_change_lists_artifacts_after_seed` 加 assert：response 含 `all_tasks_done` field（值 false，fresh change）與 `next_actions` field。對應 Requirement: `change.show` response envelope SHALL include `all_tasks_done` and `next_actions`。
- [x] 3.2 [P] 新增 `show_change_archived_returns_empty_next_actions`：seed archived（all_tasks_done=true）→ `next_actions == []`。
- [x] 3.3 [P] 新增 `show_change_in_progress_with_pending_tasks_returns_task_done_with_first_id`：seed in_progress + 寫 `tasks.md` 第一行 `- [ ] 2.1 Implement parser` → `next_actions == ["task.done 2.1"]`（Decision: change.show next_actions 採 state-driven 查表 + in_progress 掃 tasks.md 取第一個 pending task happy path）。
- [x] 3.4 [P] 新增 `show_change_in_progress_all_tasks_done_returns_archive_run`：seed in_progress + all_tasks_done=true → `next_actions == ["archive.run"]`。
- [x] 3.5 [P] 新增 `show_change_in_progress_without_tasks_md_returns_bare_task_done`：seed in_progress + 不寫 tasks.md → `next_actions == ["task.done"]`（Decision: change.show next_actions 採 state-driven 查表 + in_progress 掃 tasks.md 取第一個 pending task fallback 邊角）。
- [x] 3.6 [P] 新增 `show_change_proposing_filters_existing_artifacts`：seed proposing + 只寫 proposal.md → `next_actions == ["artifact.write design", "artifact.write tasks"]`。
- [x] 3.7 [P] 新增 `show_change_proposing_with_all_three_artifacts_returns_empty`：seed proposing + 寫齊 3 core artifact → `next_actions == []`。
- [x] 3.8 [P] 新增 `show_change_ready_returns_apply_start` / `show_change_reviewing_returns_review_pair` / `show_change_code_reviewing_returns_review_pair` 三個 test，分別 assert `["apply.start"]` / `["review.approve","review.reject"]` / `["review.approve","review.reject"]`。
- [x] 3.9 修改 `crates/runtime/src/change_ops.rs::ShowChangeData` struct，加 `pub all_tasks_done: bool` + `pub next_actions: Vec<String>`，serde rename snake_case（Decision: runtime 新建 project_ops 模組；change.show 兩欄位住既有 change_ops）。
- [x] 3.10 修改 `crates/runtime/src/change_ops.rs::show_change()`：從 `ChangeRow` 直接取 `all_tasks_done` column（Decision: change.show 加 all_tasks_done 從 change 表 column 直接讀）；呼叫新 helper `compute_next_actions(state, all_tasks_done, change_dir)` 算 next_actions；compute_next_actions 嚴格按 spec 表格查表。
- [x] 3.11 在 `crates/runtime/src/change_ops.rs` 加 `fn parse_first_pending_task_id(tasks_md_path: &Path) -> Option<String>`：讀檔；逐行掃；正則 / split 抽符合 `- [ ] <id> <desc>` 的 id token；找不到回 None；檔案不存在回 None；單元 test 寫在 same file `#[cfg(test)]` block。
- [x] 3.12 跑 `cargo test -p speclink-runtime change_ops` 確認 3.1–3.8 + 3.11 unit test 全綠。

## 4. CLI speclink status subcommand（Decision: speclink status human output 走 cli-human-output YAML printer；Requirement: `speclink status` human-mode output SHALL reuse the cli-human-output YAML pretty-printer）

- [x] 4.1 在 `crates/cli/tests/status.rs` 寫 failing integration test `status_in_empty_dir_exits_2_with_project_not_initialized`：tempdir 無 .speclink/；跑 `speclink status --json` → exit 2、envelope `error.code == "project.not_initialized"`。對應 Requirement: `speclink status` SHALL reject invocation outside a SpecLink project with `project.not_initialized`。
- [x] 4.2 在 `crates/cli/tests/status.rs` 寫 failing test `status_in_speclink_project_emits_envelope_with_seven_fields`：用 fixture project；跑 `speclink status --json` → exit 0、envelope.data 含七個 required field。對應 Implementation Contract sub-heading: Behavior（speclink status 印 envelope）以及 Implementation Contract sub-heading: Command output（envelope shapes）第一範例。
- [x] 4.3 [P] 在 `crates/cli/tests/status.rs` 寫 failing test `status_read_only_does_not_mutate_state`：跑 `speclink status --json` 兩次；assert state.db mtime 不變。
- [x] 4.4 [P] 在 `crates/cli/tests/status.rs` 寫 failing test `status_human_mode_emits_yaml_not_ansi`：跑 `speclink status`（無 --json）→ stdout 是 YAML、無 ANSI escape `\x1b[`、無 box-drawing char。對應 Requirement: `speclink status` human-mode output SHALL reuse the cli-human-output YAML pretty-printer（Decision: speclink status human output 走 cli-human-output YAML printer）。
- [x] 4.5 在 `crates/cli/src/commands/status.rs` 新建 handler `pub async fn run(json: bool) -> Result<()>`：呼叫 `runtime::project_ops::project_status(working_dir)`；走既有 `output::write_envelope` 與 cli-human-output YAML printer 路徑。
- [x] 4.6 在 `crates/cli/src/main.rs` 加 `Commands::Status` clap variant；main fn match dispatch 呼叫 4.5 handler。
- [x] 4.7 跑 `cargo test -p speclink-cli --test status` 確認 4.1–4.4 全綠。

## 5. CLI show change envelope 更新（Implementation Contract sub-heading: Command output（envelope shapes）第二範例）

- [x] 5.1 在 `crates/cli/tests/change_crud.rs` 既有 `show_change_with_artifacts` test 補 assert：envelope.data 含 `all_tasks_done` 與 `next_actions`（Implementation Contract sub-heading: Command output（envelope shapes））。
- [x] 5.2 [P] 既有 `show_change_empty_has_empty_artifacts_array` test 補 assert：data.all_tasks_done == false；data.next_actions == ["artifact.write proposal", "artifact.write design", "artifact.write tasks"]（fresh proposing 無檔）。
- [x] 5.3 跑 `cargo test -p speclink-cli show_change` 確認 5.1+5.2 綠。

## 6. 整合測試與 snapshot 對齊（Implementation Contract sub-heading: Acceptance criteria；Implementation Contract sub-heading: Behavior）

- [x] 6.1 在 `crates/cli/tests/status.rs` 加 insta snapshot `snapshot_status_envelope`：fixture project、redact requestId/working_dir/created_at；assert envelope shape 對齊 Implementation Contract sub-heading: Command output（envelope shapes）第一個 envelope 範例（Implementation Contract sub-heading: Acceptance criteria 第 2 條）。
- [x] 6.2 [P] 更新 `crates/cli/tests/describe_tools.rs` 既有 4 個 snapshot：跑 `INSTA_UPDATE=always cargo test -p speclink-cli describe_tools` 後 `cargo insta review` — 預期差異是 default_json / full_text / envelope 三個 snapshot 全部多 `outputs_schema` sibling key（curated_copilot_sdk **不該**多 — copilot-sdk renderer 不印 outputs）。任何超出此範圍的 diff 視為 regression。
- [x] 6.3 [P] 在 `crates/cli/tests/describe_tools.rs` 加 `describe_tools_filter_project_status_emits_outputs_schema_sibling`：assert 回應的單一 entry 有 top-level `outputs_schema` key（與 `parameters` sibling，非 nested），且 `outputs_schema.required` 包含 `["provider_type", "project_id", "working_dir", "changes_count", "discussions_count", "schema_active"]` 六 name（Requirement: catalogue `project_status` SHALL expose both inputs and outputs schemas via describe-tools — Scenario: describe-tools json output emits both parameters and outputs_schema for project.status）。
- [x] 6.3a [P] 在 `crates/cli/tests/describe_tools.rs` 加 `describe_tools_copilot_sdk_format_omits_outputs_schema`：跑 `speclink describe-tools --filter project.status --full --format copilot-sdk`，assert 回應的單一 entry 不含 `outputs_schema` key，仍含 `name` / `description` / `parameters`（Requirement: catalogue `project_status` SHALL expose both inputs and outputs schemas via describe-tools — Scenario: describe-tools copilot-sdk format omits outputs_schema）。
- [x] 6.4 跑 `cargo test --workspace` 全綠（Implementation Contract sub-heading: Acceptance criteria 第 1 條）。

## 7. Implementation Contract sub-heading 簽收與文件同步（Behavior / Interface / data shape / Command output（envelope shapes）/ Failure modes / Acceptance criteria / Scope boundaries）

- [x] 7.1 對齊 Implementation Contract sub-heading: Interface / data shape：cross-check 新 `ProjectStatusData` 與 operations.md §1389 output schema 欄位 1:1 對應；cross-check 新 `ShowChangeData` field 命名為 `all_tasks_done` / `next_actions`。
- [x] 7.2 對齊 Implementation Contract sub-heading: Failure modes：跑 `speclink status` 在 empty tempdir → assert exit 2 + `project.not_initialized`；跑 `speclink show change <unknown>` → exit 2 + `change.not_found`（既有不退化）。
- [x] 7.3 對齊 Implementation Contract sub-heading: Scope boundaries：grep `crates/runtime/src/project_ops.rs` 與 `crates/runtime/src/change_ops.rs`，確認沒有 discussion 表查詢、沒有 artifact DAG ready 計算、沒有新 provider trait method。
- [x] 7.4 對齊 Implementation Contract sub-heading: Acceptance criteria 第 4 條：跑 `cargo run --bin speclink -- show change add-tool-describe-and-catalogue --json` → assert `data.all_tasks_done == true` 且 `data.next_actions == []`（archived change e2e 驗證）。
- [x] 7.5 對齊 Implementation Contract sub-heading: Behavior（speclink status 印 envelope + show change 加兩欄位 + 兩 op 皆 read-only）：跑完 6.1 snapshot 後人眼 review YAML / JSON 兩條輸出路徑 stdout 內容與 Behavior 條列吻合。
- [x] 7.6 [P] 更新 `doc/protocol/operations.md` §`change.show` Outputs schema：加 `all_tasks_done` 與 `next_actions` 兩 field 在 properties 區段；同步 §Index 表（若標 implemented 狀態）。
- [x] 7.7 [P] 更新 `doc/speclink-design.md` §18.4 P1-2 行尾標 `(implemented 2026-05-23)`；§18.5 backlog #2 從 backlog 移到「已隨 add-project-status ship」清單。
- [x] 7.8 跑 final `spectra analyze add-project-status --json`，0 Critical / 0 Warning（Implementation Contract sub-heading: Acceptance criteria 第 5 條 — next_actions 7 state lookup matrix 整體簽收）。

## 8. Dogfood anomaly 修正（Decision: current_change 採 actor.host_id 比對；無則 null；Decision: change.show next_actions 採 state-driven 查表 + in_progress 用 1-based sequential checkbox index）

E2E test against `~/Documents/GitHub/test-speclink-sdd` 抓出兩個 anomaly：(#1) `current_change` 用 `instance_id` 比對 hostname 永不命中 → dogfood headline UX 失效；(#2) `next_actions` emit label（如 `"task.done 2.1"`）但 `task done` CLI 要 1-based INDEX → AI agent retry loop。本 Group 修正並補測，趁 slice 沒 archive 一次做對。

- [x] 8.1 在 `crates/runtime/src/state_machine.rs` 把 `fn resolve_host_id() -> String` 改為 `pub fn`（讓 `project_ops` 可重用同一條 hostname resolution chain）。對應 Decision: current_change 採 actor.host_id 比對；無則 null。
- [x] 8.2 在 `crates/runtime/src/project_ops.rs::project_status` 把 `actor.host_id == link.instance_id` 比對改成 `actor.host_id == speclink_runtime::state_machine::resolve_host_id()`；移除對 `link.instance_id` 的依賴；對應 Requirement: `project.status` SHALL populate `current_change` only when an in_progress change matches the current host（含新 Scenario「in_progress change owned by the CLI invocation's host SHALL appear in current_change after a real apply.start」）。
- [x] 8.3 在 `crates/runtime/tests/project_ops.rs` 把 host_id fixture（test 2.3 / 2.4 / 2.5）從 `current_instance_id(working)` 改成 `state_machine::resolve_host_id()`；刪除 `current_instance_id` helper 與相關 link.yaml parse 邏輯（不再需要）。對應 Decision: current_change 採 actor.host_id 比對；無則 null。
- [x] 8.4 在 `crates/runtime/src/change_ops.rs` 移除 `fn parse_first_pending_task_id()` 與相關 unit test（過早抽象、回 label）。對應 Decision: change.show next_actions 採 state-driven 查表 + in_progress 用 1-based sequential checkbox index。
- [x] 8.5 在 `crates/runtime/src/change_ops.rs::compute_next_actions` 在 `in_progress + !all_tasks_done` 分支改呼叫 `task_ops::parse_checkbox_lines(content)`，取第一個 `!item.done` 的 `item.index` 作為 INDEX；tasks.md 不存在或無 unchecked → `["task.done"]` 保留。對應 Requirement: `change.show` response envelope SHALL include `all_tasks_done` and `next_actions`（新 Scenario「next_actions emits index that the task.done CLI actually accepts」）。
- [x] 8.6 [P] 更新 `crates/runtime/src/change_ops.rs` 既有 unit test：`compute_next_actions_in_progress_with_pending_id_suggests_task_done_with_id` 改為驗 index（fixture「`- [x] 1 a\n- [ ] 2.5 next\n`」→ assert `["task.done 2"]` 而非 `["task.done 2.5"]`）。
- [x] 8.7 [P] 更新 `crates/runtime/tests/change_ops.rs` integration test `show_change_in_progress_with_pending_tasks_returns_task_done_with_first_id`：tasks.md fixture 改用「先 `- [x]` 已完成、再 `- [ ]` 第一個 pending」格式，assert next_actions 為 `["task.done <index>"]` 而非 label；同時 rename 變數 `2.1` → `4`（與 spec scenario「first unchecked line is `- [ ] 2.1 Implement parser` while three earlier `- [x]` lines have already been checked off」對齊）。
- [x] 8.8 跑 `cargo test --workspace` 全綠（含 catalogue / render / change_ops / project_ops 全條 e2e）。
- [x] 8.9 重 build release binary（`cargo build --release`）後跑 e2e against `~/Documents/GitHub/test-speclink-sdd`：(a) wipe `.speclink/` + `.git/speclink/` 重 init；(b) `speclink new change wip → artifact write 三檔 + spec → status 進 ready → apply start → task list → speclink status --json` assert `data.current_change.change_id == "wip"`；(c) `speclink show change wip --json` assert `data.next_actions[0]` matches regex `^task\.done [0-9]+$`、執行該指令 succeed。對應 Implementation Contract sub-heading: Acceptance criteria 整體 dogfood 簽收。
- [x] 8.10 [P] 更新 `doc/speclink-design.md` §18.5 加 entry：「2026-05-23 `add-project-status` 在歸檔前 dogfood 修正 2 個 anomaly — current_change 比對與 next_actions 索引型別」。
- [x] 8.11 跑 final `spectra analyze add-project-status --json`，0 Critical / 0 Warning。
