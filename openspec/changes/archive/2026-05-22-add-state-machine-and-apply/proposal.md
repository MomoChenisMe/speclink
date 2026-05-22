## Why

`add-change-and-artifact-io` 完成後 `change.state` 永遠寫 `proposing`、`change.version` 永遠是 1，整個 SDD lifecycle 卡在第一階。Walking skeleton 第三片必須把 6-state transition 接通、把 `apply.* / task.*` 5 個 op 落地，讓 user / AI 能端到端跑完 `propose → ready → in_progress → all tasks done` 這條主路徑，為後續 review / archive / locking slice 提供 state machine 基底。

設計依據 `doc/speclink-design.md` §6.2（6-state transition + apply 雙向 idempotent + ensure-actor）、§6.3（review optionality）、§16.7（Apply / Task CLI）、`doc/protocol/operations.md` 的 `apply.start` / `apply.pause` / `task.done` 三條 op spec。

## What Changes

- 新增 CLI 指令（皆屬「AI workflow 階段」，可由 skill 呼叫）：
  - `speclink apply start <change-id> [--actor <id>] [--json]` — ensure actor 進入 apply 階段；對 `ready` transition `in_progress`，對 `in_progress` no-op + reassign actor，對 `code_reviewing` / `archived` 不轉移 + 回 hint message（非 error），對 `proposing` / `reviewing` 回 `state.transition_invalid`
  - `speclink apply pause <change-id> [--json]` — 對 `in_progress` transition `ready` + 清 actor；對 `ready` no-op idempotent；其他 state 回 `state.transition_invalid`
  - `speclink task list --change <id> [--json]` — 列舉 `.speclink/changes/<id>/tasks.md` 內所有 `- [ ]` / `- [x]` checkbox（1-based index），含 done 狀態與顯示文字
  - `speclink task done <task-index> --change <id> [--json]` — 把對應 index 的 `- [ ]` 改為 `- [x]`；idempotent（已 done 再呼叫 no-op）；完成最後一個 task 時依 `require_code_review` flag 觸發 auto-transition（`in_progress → code_reviewing` 或設 `all_tasks_done` flag）
  - `speclink task undo <task-index> --change <id> [--json]` — 把 `- [x]` 改回 `- [ ]`；idempotent；若 change 已在 `code_reviewing` state，先回退到 `in_progress` 再 unmark
- State machine 接通（design §6.2 6-state transition graph）：
  - `proposing → ready`（auto）：`require_artifact_review=false` 時，`artifact.write` 後 evaluator 檢查 DAG（proposal + tasks + ≥1 spec 齊全）→ 自動 transition
  - `proposing → reviewing`（auto）：`require_artifact_review=true` 時，DAG 齊全 → transition reviewing（停在此 state 等 review.approve，本 slice 不實作 review CLI）
  - `ready ⇌ in_progress`（顯式）：`apply.start` / `apply.pause` driver
  - `in_progress → code_reviewing`（auto）：`require_code_review=true` + 所有 task `[x]` → 由 `task.done` engine auto-trigger
  - `in_progress` all-done state（`require_code_review=false`）：state 維持 `in_progress`，但設 `all_tasks_done=true` flag（落在 `change.all_tasks_done` 欄位），等後續 `archive.run` slice 接收
- Walking skeleton 預設：本 slice 硬編 `require_artifact_review=false` + `require_code_review=false`（4-state mode：`proposing → ready → in_progress → all_tasks_done`）。`require_*_review` flag 的 config 讀取由後續 `add-config-rw` slice 接通；config 不存在時用 walking-skeleton 預設值。
- State.db v3 migration（forward-only，遵守 §12.7 migration flow）：
  - `change` 表 alter：新增 `actor_json` TEXT NULL（apply.start 寫入、apply.pause 清空、`ingest` 退回時清空）；新增 `all_tasks_done` INTEGER NOT NULL DEFAULT 0（boolean 0/1，task.done 完成最後一個 + `require_code_review=false` 時設 1）
  - 新表 `state_transition`（audit log）：`transition_id TEXT PK / change_id TEXT NOT NULL REFERENCES change / from_state TEXT NOT NULL / to_state TEXT NOT NULL / actor_json TEXT NULL / transitioned_at TIMESTAMP NOT NULL / reason TEXT NOT NULL`；reason 列舉值 `apply_start` / `apply_pause` / `task_done_auto` / `task_undo_revert` / `artifact_dag_complete`
  - `_migrations` 表寫入 version=3 row
- Provider trait 擴充（`crates/provider/src/lib.rs`）：新增 `StateMachineStore` trait（`get_change_state` / `transition_state(change_id, expected_version, to_state, actor, reason)` / `set_actor` / `clear_actor` / `set_all_tasks_done`），所有 method 在同一 SQLite tx 內完成 `change` row update + `state_transition` insert，使用 `change.version` 作 compare-and-swap（不一致回 `state.version_conflict` error）
- Actor 推導（`apply.start` 在 `--actor` 省略時自動推導）：`agent_host = env SPECLINK_AGENT_HOST else "cli"`、`os_user = env USER (Unix) / USERNAME (Windows)`、`host_id = sys hostname`；推導結果存進 `change.actor_json` 並出現在 `apply.start` 回傳的 `actor` 欄位
- Tasks.md parsing 規格：使用 markdown checkbox regex `^(\s*)- \[( |x)\] (.+)$`（行首允許縮排但 task index 仍依文件出現順序、不依縮排層級）；非 checkbox 行（標題、純文字、巢狀清單）忽略；同一 task 行允許保留行尾 HTML comment（為未來 feedback marker 預留）
- 新 error codes（design §17.4 命名規則）：`state.transition_invalid`（exit 7）、`state.version_conflict`（exit 7）、`task.index_out_of_range`（exit 2）、`task.no_tasks_file`（exit 2）、`change.dag_incomplete`（exit 2，僅 `change advance` fallback 用；本 slice 不暴露 CLI，留作 doctor finding hint）
- JSON envelope：所有 5 個新 op 沿用 bootstrap / A2 既有 `{ ok, data, warnings?, requestId }` / `{ ok: false, error: { code, message, hint?, retryable, retry_after_ms? }, requestId }` 形狀；新增 data shape（`apply.start` / `apply.pause` / `task.list` / `task.done` / `task.undo`）由各 spec 章節定義
- Audit event：每次 state transition 在同 tx 內 insert 一筆 `state_transition` row；本 slice 不對外暴露 audit query CLI（`review history` 等留給 review slice）

使用者情境：本 slice 全部 op 都屬於「AI workflow 階段」（可由 `spectra-apply` skill 呼叫）；無人類設定階段 CLI；無 CI 專屬 op。

## Non-Goals

- ❌ `review.approve` / `review.reject` / `review.history` / `review.status` — 由後續 `add-review` slice 負責；本 slice state machine 在 `reviewing` / `code_reviewing` state 停住等 reviewer driver
- ❌ `archive.run` — 由後續 `add-archive` slice 負責；本 slice 用 `all_tasks_done` flag 把「ready to archive」訊號交給後續 slice
- ❌ `feedback_tasks` 表 / synthetic feedback task marker / re-entry 機制 — 與 review slice 綁定，一起做
- ❌ `change.advance` 顯式 CLI fallback — 本 slice 全靠 `artifact.write` hook auto-transition；DAG 不齊就停 `proposing`，user 須補齊 artifact 才推進（manual override CLI 留待 doctor slice）
- ❌ Schema YAML 解析 / artifact body 結構驗證 — 由後續 `add-schema-management` slice 負責；本 slice DAG 完整性檢查只看「proposal.md + tasks.md + specs/*/spec.md 至少一份」是否存在，不解析 schema
- ❌ Per-change file lock / stale lock 接管 / lock 階層 — 由後續 `add-locking-and-concurrency` slice 負責；本 slice 所有 state transition 走 `change.version` optimistic concurrency + 單 SQLite tx 保證原子性，TOCTOU 窗口暫時留著
- ❌ `config.read` / `config.write` — 由後續 `add-config-rw` slice 負責；本 slice `require_*_review` flag 硬編為 `false`（walking-skeleton 4-state mode）
- ❌ `ingest` 反向 transition 的 cascade cleanup（design §6.2.1）— 由 `add-ingest-revert` slice 負責；本 slice state machine 只支援前向 transition
- ❌ Task marker（HTML comment 內嵌 task id）— 本 slice 用 1-based 行內順序 index；當 tasks.md 改動會破壞 index 穩定性，user 須在 task 全 done 後才 edit。Marker 機制與 feedback marker 一起在 review slice 補
- ❌ `task list --json` 以外的輸出格式（plain text 友善排版）— pretty-print 由 `improve-human-output-pretty-print` change（slice 平行進行）負責
- ❌ Audit query CLI（`speclink audit list / show`）— state_transition 表本 slice 只寫不讀，查詢 CLI 留待後續 slice
- ❌ HttpProvider 對應實作 — Provider trait method 在本 slice 加進 `crates/provider`，但只有 `provider-local` 提供 impl

## Capabilities

### New Capabilities

- `state-machine`: 6-state lifecycle transition rules、`change.state` 合法值列舉、`proposing → reviewing/ready` 由 `artifact.write` hook 觸發的 DAG 完整性 evaluator、`in_progress → code_reviewing` auto-trigger 條件、`change.actor_json` 管理規則、`state.version` compare-and-swap 並發控制、`state_transition` audit 表 schema、state.db v3 migration、walking-skeleton 預設 `require_*_review=false` 行為
- `apply-task-ops`: `apply.start` / `apply.pause` / `task.list` / `task.done` / `task.undo` 5 個 CLI op、JSON envelope shape、actor 自動推導規則、tasks.md checkbox parsing 規格、task index 與 `all_tasks_done` flag 連動行為、新增 5 個 error code 與對應 exit code

### Modified Capabilities

- `change-store`: 解除「`state` 永遠是 `proposing`」與「`version` 永遠是 1、row 永不 mutate」限制；改寫為「`state` 由 `state-machine` capability 規範合法值集合與 transition、`version` 是 monotonic counter 由 `state-machine` 的 RMW 路徑更新」；新增 `actor_json` 與 `all_tasks_done` 欄位的存取契約

## Impact

- Affected specs:
  - New capability spec for `state-machine`
  - New capability spec for `apply-task-ops`
  - Modified capability spec for `change-store`（需先 archive `add-change-and-artifact-io` 才能 modify）
- Affected code:
  - Modified: crates/provider/src/lib.rs（新增 `StateMachineStore` trait；保留既有 `ProjectStore` / `ChangeStore` / `ArtifactStore`）
  - Modified: crates/provider/src/types.rs（新增 `ChangeState` enum 6 variant、`Actor` struct、`StateTransitionReason` enum、`TransitionRequest` struct）
  - Modified: crates/provider/src/error.rs（在既有 `ProviderError` 加 5 個新 variant；在既有 `codes` module 加 5 個新 `pub const` error code）
  - New: crates/provider-local/src/state_machine_store.rs（`LocalStateMachineStore` 對 `StateMachineStore` trait 的具體 impl，含 single-tx state update + audit insert）
  - Modified: crates/provider-local/src/state_db.rs（`MIGRATIONS` 陣列追加 v3 entry：alter `change` 加 `actor_json` + `all_tasks_done`、create `state_transition` 表；新增 helper method `insert_state_transition` / `update_change_state_cas` / `set_actor` / `clear_actor` / `set_all_tasks_done`）
  - Modified: crates/provider-local/src/store.rs（`LocalProjectStore::open_state_db()` 內 `db.migrate(2)` bump 為 `db.migrate(3)`）
  - Modified: crates/provider-local/src/change_store.rs（A2 既有 `LocalChangeStore::create` 把 hardcoded `state='proposing'` 改成讀 `ChangeState::Proposing` enum；其餘行為不動）
  - Modified: crates/provider-local/src/lib.rs（pub use 新增 `LocalStateMachineStore`）
  - New: crates/runtime/src/state_machine.rs（state transition evaluator + 合法 transition 表 + auto-trigger 條件 + actor 推導 helper）
  - New: crates/runtime/src/apply_ops.rs（`ApplyOperations<G>` runtime entry 含 `start` / `pause`，命名沿用 A2 `ChangeOperations<G>` 慣例）
  - New: crates/runtime/src/task_ops.rs（`TaskOperations<G>` runtime entry 含 `list` / `done` / `undo` + tasks.md parser）
  - Modified: crates/runtime/src/artifact_ops.rs（A2 既有 `ArtifactOperations::write` 在成功寫檔後 call state_machine evaluator 觸發 auto-transition；read / list 路徑不動）
  - Modified: crates/runtime/src/error.rs（`RuntimeError` 加 5 個新 variant；`code()` 與 `exit_code()` match arm 擴充 5 個新 code）
  - Modified: crates/runtime/src/lib.rs（pub mod / pub use 新增 `state_machine` / `apply_ops` / `task_ops`）
  - New: crates/cli/src/commands/apply_start.rs
  - New: crates/cli/src/commands/apply_pause.rs
  - New: crates/cli/src/commands/task_list.rs
  - New: crates/cli/src/commands/task_done.rs
  - New: crates/cli/src/commands/task_undo.rs
  - Modified: crates/cli/src/commands/mod.rs（pub mod 列出 5 個新 module）
  - Modified: crates/cli/src/main.rs（`Commands` enum 加 `Apply` / `Task` 兩個父 subcommand variant；match arm dispatch；`hint_for` 擴充 5 個新 code）
  - Modified: crates/cli/src/output.rs（`error_code_to_exit` match arm 擴充 5 個新 code）
  - New: crates/cli/tests/apply_lifecycle.rs（apply start/pause 雙向 idempotent + ensure-actor 端到端整合測）
  - New: crates/cli/tests/task_workflow.rs（task list/done/undo + all_tasks_done flag 整合測）
  - New: crates/cli/tests/state_machine_e2e.rs（artifact.write → auto-transition → apply.start → task.done × N → all_tasks_done 完整 walking skeleton 端到端整合測）
  - New: crates/runtime/tests/state_machine.rs（transition table + DAG evaluator unit 測）
  - New: crates/runtime/tests/apply_ops.rs（runtime apply.start/pause + actor 推導測）
  - New: crates/runtime/tests/task_ops.rs（runtime task.list/done/undo + tasks.md parser 測 + index 邊界測）
  - Modified: doc/speclink-design.md（補一節「Walking skeleton slice naming」凍住 A1/A2/A3 命名表，與本 slice 提案一起進入；slice 命名表落在 §1 定位之後、§2 設計原則之前）
- 重用既有公開常量：`speclink_runtime::ARTIFACT_ROOT`（不重宣告）；`speclink_provider::codes::*` 既有 11 個 `project.*` + `change.*` + `artifact.*` code 保留不動
- Affected crates: `cli`、`runtime`、`provider`、`provider-local`
- Affected design refs: doc/speclink-design.md §6.2 (6-state lifecycle), §6.3 (review optionality), §12.4 (操作 vs lock 對照, lock 暫不接通), §12.7 (schema migration flow), §13.9 (state storage layout), §16.7 (Apply / Task CLI), §17.4 (error code naming), §17.6 (audit event 寫入時機保證); doc/protocol/operations.md `apply.start` / `apply.pause` / `task.done`
- Prerequisite: 本 change apply 前 user 必須先 archive `add-change-and-artifact-io`，使 `openspec/specs/change-store/spec.md` 出現在 working tree（Modified Capability 才能成立）
