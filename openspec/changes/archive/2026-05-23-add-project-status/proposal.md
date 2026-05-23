## Why

Dogfooding 即將啟動（Phase 1 #4 ship 後切換），但目前 `speclink show change <name>` envelope 只給 `{change, artifacts[]}`，apply skill / 人工 user 都得從 `task list` 推算「下一步該打哪個 op」、「task 是否全完」。Phase 1 #2 把 `project.status`（dashboard 入口）與 `change.show` UX 補完，讓 dogfooding 啟動瞬間就有可用的「目前狀況一眼看完」與「下一動作建議」。

## What Changes

- 新增 op `project.status`（catalogue 既有 entry，從 metadata-only 變 implemented），output schema 完整對齊 `doc/protocol/operations.md` §1360
- 新增 CLI subcommand `speclink status [--json]`，human mode 走既有 cli-human-output YAML pretty-printer、不做 dashboard table
- 擴充 `change.show` envelope，加 `all_tasks_done: bool`（讀 change 表 column）與 `next_actions: [string]`（state → 查表，必要時掃 tasks.md 抓第一個 pending task ID）
- `discussions_count` 此 slice 永遠回 `{active: 0, converged: 0}`，註明屬 P2 #1 `add-discuss-ops` 才實作（schema 仍保留欄位，避免 P2 ship 時 break 既有 consumer）
- `current_change` 規則：state.db 找 `state='in_progress'` 且該 row 的 `actor.host_id` 等於當前 instance_id 的 row；找不到回 null

## Non-Goals

- 不做 dashboard table renderer（YAML pretty-print 即可；dogfood 真痛點再開另一條 slice 處理）
- 不重 parse tasks.md 計算 `all_tasks_done`（task_ops 已維護 change 表 column）
- 不做 artifact DAG ready 計算（屬 P1-3 `add-instructions-get` 範圍）
- 不新增 provider trait method（runtime-side aggregation；避免 thin-wrapper adapter）
- 不實作 discussions 計數（P2 #1 `add-discuss-ops` 才補；本 slice 保留 schema 欄位、值固定 0）
- 不新增 error code（reuse 既有 `project.not_initialized`）
- 不重命名 catalogue 既有欄位 / 不變動既有 37 個 op 的 inputs schema 內容（catalogue 會擴 `outputs_schema` 新欄位 — 屬 additive、見 design Decision「catalogue Operation struct 擴 `outputs_schema: fn() -> Value` 欄位」）

## Capabilities

### New Capabilities

- `project-status`: `project.status` op 的 output schema、CLI binding（`speclink status`）、`current_change` 偵測規則、`discussions_count` 過渡期行為

### Modified Capabilities

- `change-store`: `change.show` envelope 擴充 `all_tasks_done` 與 `next_actions` 兩欄位，含 state-driven next-action 查表規則與 in_progress 掃 tasks.md 取第一個 pending task 的演算法

## Impact

- Affected specs:
  - 新建 `openspec/specs/project-status/spec.md`
  - delta 擴充 `openspec/specs/change-store/spec.md`（加 1 條 Requirement）
- Affected code:
  - New:
    - `crates/runtime/src/project_ops.rs`（project-level aggregator）
    - `crates/cli/src/commands/status.rs`
    - `crates/cli/tests/status.rs`
    - `crates/runtime/tests/project_ops.rs`
  - Modified:
    - `crates/runtime/src/lib.rs`（新增 `pub mod project_ops`）
    - `crates/runtime/src/change_ops.rs`（`ShowChangeData` 加兩欄位、`show_change` 計算邏輯）
    - `crates/runtime/tests/change_ops.rs`（補 `show_change` 兩新欄位 test）
    - `crates/cli/src/main.rs`（新增 `Commands::Status` clap variant + dispatch handler）
    - `crates/cli/tests/change_crud.rs`（既有 `show_change_*` test 加新欄位斷言）
    - `crates/runtime/src/catalogue/schemas.rs`（新增 37 個 `<op>_outputs()` 函式：project_status / change_show 給真實 outputs schema、其餘 35 給 `empty_object_schema()` stub）
    - `crates/runtime/src/catalogue/mod.rs`（`Operation` struct 加 `pub outputs_schema: fn() -> Value` 欄位；37 個 op entry 全填新欄位）
    - `crates/runtime/src/tool_ops/render.rs`（`JsonRenderer` 加 `outputs_schema` 鍵；`CopilotSdkRenderer` 不動）
    - `crates/cli/tests/describe_tools.rs`（4 個 insta snapshot 重生 + 新增 outputs_schema 斷言 test）
  - Removed: 無
- Affected crates: `cli`、`runtime`、`provider-local`（read-only 呼叫既有 method）
- CLI exit codes：`speclink status` 正常 0；`project.not_initialized` exit 2
- Skill 影響：dogfood UX 提升，apply skill 後續可消費 `next_actions` hint；本 slice 不改 skill
