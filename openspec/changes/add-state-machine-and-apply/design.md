## Context

`add-project-bootstrap`（slice A1, archived）建立了 LocalProvider + two-root storage + `project` 表。`add-change-and-artifact-io`（slice A2, awaiting archive）長出 `change` 表 + artifact filesystem I/O，但 `change.state` 永遠 stub 為 `'proposing'`、`change.version` 永遠 1。本 slice 是 walking skeleton 第三片，把 design §6.2 的 6-state transition、design §16.7 的 apply/task CLI、operations.md 的 `apply.start / apply.pause / task.done` 三條 op contract 接通，讓 SDD lifecycle 從「卡 proposing」進化到「能跑完 walking skeleton 主路徑：propose → ready → in_progress → all_tasks_done」。

本 slice 對齊 design.md 既有規範，**不**改寫 §6.2 / §6.3 / §16.7 任何規格；只挑出本 slice 範圍內需要 trade-off 的 6 個決策、寫進 design。Review (§16.8)、Archive (§16.9)、Locking (§12.2)、Schema (§16.11)、Config (§16.0)、Ingest cascade (§6.2.1) 全部明確留給後續 slice。

Stakeholders：

- AI workflow skill（特別是 `spectra-apply`）— 主要 driver，呼 `apply.start` / `task.done` 跑 SDD 第三階
- 個人 RD（solo dev）— walking-skeleton 預設 `require_*_review=false` 直接受益
- 後續 review / archive slice — 本 slice 預留的 transition hook + `all_tasks_done` flag + `state_transition` audit 表是它們的接面

## Goals / Non-Goals

**Goals:**

- 把 design §6.2 6-state transition graph 全部 transition 路徑在 `crates/runtime/src/state_machine.rs` 內實作完整（含 apply 雙向 idempotent + ensure-actor 表）
- 在 `crates/provider` 內新增 `StateMachineStore` trait，把「state mutation + audit insert」綁在同一個 SQLite tx 內，提供 row-level optimistic concurrency（`change.version` compare-and-swap）
- 5 個 CLI op (`apply.start` / `apply.pause` / `task.list` / `task.done` / `task.undo`) JSON envelope 與 operations.md 對齊，可被 `spectra-apply` skill 直接呼叫
- `artifact.write` 寫入後 trigger DAG evaluator → 自動推進 `proposing → reviewing/ready`，walking skeleton 端到端可跑通
- 在 walking-skeleton 4-state mode（`require_*_review=false`）下不依賴 review / archive / locking / schema / config slice，本 slice 獨立可 ship

**Non-Goals:**

- ❌ 不實作 review / archive / locking / schema-management / config-rw / ingest-cascade slice 任何 op（皆有對應 future change）
- ❌ 不引入 `change.advance` 顯式 CLI（auto-trigger 已涵蓋；manual override 留給 doctor slice）
- ❌ 不引入 task HTML comment marker（本 slice 用 1-based 行內 index；marker 與 feedback marker 一起在 review slice 補）
- ❌ 不對 `state_transition` 表做任何 query CLI（只寫不讀，audit query 屬 review slice）
- ❌ 不在 `apply.start` 取真實 lock（trait method 簽章預留 lock token 概念交給 locking slice 之後 retrofit 即可）
- ❌ 不寫 `feedback_tasks` 表（review slice 處理）
- ❌ HttpProvider 對應實作 — `StateMachineStore` trait 在本 slice 加進 `crates/provider`，但只有 `provider-local` 提供 impl

## Decisions

### State.db v3 migration：alter `change` 表 + 新增 `state_transition` 表

**選項**：

| 選項 | `change` 表變動 | audit 儲存 |
|---|---|---|
| A | alter add `actor_json` + `all_tasks_done` 兩欄 | 新表 `state_transition`，每次 transition insert 一 row |
| B | 不動 `change`；actor / all_tasks_done 移到新表 `change_runtime` | 新表 `state_transition`，每次 transition insert 一 row |
| C | 不動 `change`；actor / all_tasks_done / audit 全合進新表 `change_runtime`（沒有獨立 audit 表） | 用 `change_runtime` 多 row 模擬 audit（每次 transition append） |

**選 A**。理由：

1. **actor / all_tasks_done 是 change 的 1:1 屬性，不該拆表**：B/C 把 1:1 屬性拆表會逼每次 query 都 JOIN，且 lifecycle 推進跟 row 同生命週期（change 刪除時這兩欄一定要消），同表反而簡化 cascade delete
2. **`state_transition` 是 1:N 紀錄，必須獨立表**：每個 change 會有多次 transition（apply.start / pause / done / undo / 未來 review reject re-entry），1:N 走獨立表是 textbook 設計；C 用 multi-row 模擬會跟 1:1 屬性混雜，難加 timestamp index
3. **SQLite ALTER TABLE ADD COLUMN 是 cheap operation**：只更 schema 不掃 row；migration 在 §12.7 forward-only 框架下 idempotent，與 A2 v2 migration 同 pattern
4. **與 design §13.9 storage layout 對齊**：§13.9 列舉 state.db 內容包含「audit / review history」，本 slice 的 `state_transition` 就是 audit 的具體 schema

**取捨**：alter table 對 binary downgrade 不友善（跟 A2 同問題，§18.1 #67 forward-only 已接受）。Release notes 需警告「v3 binary 跑過後 v2 binary 開不了」，與 A2 → A1 同邏輯。

### Auto-transition trigger：`artifact.write` hook 而非顯式 `change.advance` CLI

**選項**：

| 選項 | trigger 來源 | DAG 完整性檢查 |
|---|---|---|
| A | `artifact.write` 成功後 evaluator hook，自動 transition | 每次寫檔即時檢查（含初次寫 / 覆寫） |
| B | 顯式 `speclink change advance <id>` CLI，user 主動 trigger | evaluator 跑於 advance op 內 |
| C | A + B 並存：hook 自動跑、advance 作為 manual fallback | 兩處 |

**選 A**。理由：

1. **對齊 design §6.2**：「proposing → reviewing：退出觸發者 = engine auto」明示 transition 是 engine 自動、不是 user 顯式行為
2. **減少 AI skill 認知負擔**：`spectra-apply` skill 不必先 `artifact.write` 再 `change.advance`；hook 自動推進讓 skill 路徑線性
3. **避免「忘記 advance」失敗模式**：B 路徑 user/AI 容易 artifact 寫完忘記 advance，state 卡住 user 還需 doctor 才發現
4. **C 的 advance 在本 slice 沒實際用途**：DAG 不齊時 advance 也只能回 error；DAG 齊時 hook 早就 transition 過了。advance 留到 doctor slice（state 跟 filesystem 不一致時 manual recovery）才有真實場景

**取捨**：hook 隱式 transition 對沒讀過 design §6.2 的工程師可能「不知道發生了什麼」。緩解：每次 transition 在 `apply.start` 之外的場景也回 warning（warnings array 內附 `state_transitioned` warning code）讓 CLI 輸出可見；JSON envelope 的 `warnings` 欄位本就是為此設計。

### Walking-skeleton review-flag 預設值：硬編 `false`，不讀 config

**選項**：

| 選項 | flag 來源 | config slice 依賴 |
|---|---|---|
| A | 硬編 `require_artifact_review=false` + `require_code_review=false` | 無依賴；本 slice 獨立可 ship |
| B | 嘗試讀 `.speclink/config.yaml`，缺失或解析失敗 fallback 預設 false | 需要 YAML parser + path resolver，事實上等同抓半個 `add-config-rw` 進來 |
| C | 硬編 `true / true`，但提供 env var `SPECLINK_SKIP_REVIEW=1` 暫時 bypass | env var 暫時管道；user 體驗差 |

**選 A**。理由：

1. **與 walking-skeleton 原則對齊**：本 slice 目標是「最薄一片接通 lifecycle」，引入 config 讀取等於把 `add-config-rw` 部分提前，違反 slice boundary
2. **個人 RD MVP 場景剛好都是 false**：design §6.3 提到「個人專案 / solo dev 可全關，跑成 4 state」，硬編 false 與多數 MVP 用戶預期一致
3. **未來 config-rw slice 接管時是純加法**：把 `state_machine::ReviewPolicy::walking_skeleton()` 換成 `ReviewPolicy::from_config(&config)`，state_machine 內部表不變
4. **C 的環境變數方案是 anti-pattern**：env var 在 CI / 多 agent 環境下行為難預測，design §16.0 全域規範已禁絕「CLI 行為從 env var 推導」的 path

**取捨**：團隊用戶若預期 `require_*_review=true` 會困惑。緩解：本 slice CHANGELOG 與 release notes 明文「Walking skeleton mode hard-codes review flags to false; config-driven flags arrive in slice `add-config-rw`」；`apply.start` 等 op 在 transition 時的 warning 附 hint「review flags currently hard-coded to false in walking-skeleton mode」。

### Task id 策略：1-based 行內順序 index，不引入 marker

**選項**：

| 選項 | task id 來源 | 對 tasks.md 改動的耐受度 |
|---|---|---|
| A | 1-based index：每次 task.done 即時 parse tasks.md 排序產生 | tasks.md 任意 reorder / insert 會 break index 穩定性 |
| B | HTML comment marker：第一次 task.list 時自動 inject `<!-- speclink:task id=t-uuid -->`，後續用 uuid 識別 | atomic write、marker 隨 task 漂移，穩定 |
| C | tasks.md 規範改為「task 必帶顯式 id 前綴 `- [ ] [T1] ...`」，user/AI 手工分配 | user/AI 容易撞號或漏號，需 validator |

**選 A**。理由：

1. **walking skeleton 不需要 marker 穩定性**：本 slice 場景是「task 寫好後依序 done」，極少在 done 過程中改 tasks.md；index 漂移風險低
2. **Marker 機制與 feedback marker 一起在 review slice 補**：design §6.2 reject re-entry 機制本就需要 marker（`speclink:feedback id=fb-...`），review slice 同時引入 task marker（`speclink:task id=t-...`）較對稱，避免本 slice 引入後 review slice 又改一次格式
3. **A 是 zero-config**：user/AI 寫 markdown checkbox 即可，無學習成本；B/C 都增加 surface
4. **task.undo 在本 slice 對稱實作**：A 路徑下 undo 用 index 同樣可達；marker 在 undo 場景沒有額外優勢

**取捨**：A 的 index 漂移是已知限制。緩解：tasks.md 改動視為「需重新閱讀 task 編號」的明確契約（spec 內明文寫死「task index 由文件當前 reading 順序決定，user 在 task done 期間應避免重排 tasks.md」）；marker slice 落地後此限制自動解除。

### `StateMachineStore` 拆成獨立 trait（不併入 `ChangeStore`）

**選項**：

| 選項 | trait 拆分 |
|---|---|
| A | 新 `StateMachineStore`：`get_state` / `transition_state` / `set_actor` / `clear_actor` / `set_all_tasks_done`；既有 `ChangeStore` 不動 |
| B | 把 5 個 method 加進既有 `ChangeStore`，單一巨型 trait |
| C | `StateMachineStore` + `ActorStore` + `TaskStateStore` 三 trait 細分 |

**選 A**。理由：

1. **跟 A2 拆分風格一致**：A2 design.md 已決定「`ChangeStore` 與 `ArtifactStore` 拆成兩個 trait」、駁回「單一 `ProjectStore` 加方法」；本 slice 沿用 — `StateMachineStore` 是 lifecycle 行為的單一接面
2. **HttpProvider 對應乾淨**：HTTP 端 `POST /api/.../apply/start` 等 endpoint 對應 `StateMachineStore` 一個 trait，未來實作時 mock / wiremock 隔離容易
3. **C 過度拆分**：actor / all_tasks_done 是 transition 路徑上的副作用，跟 state mutation 同一個 SQLite tx；強拆會破壞「state + side effect 同 tx commit」設計（design §6.2 明文要求 audit event 同 tx）
4. **B 巨型 trait 違反 SRP**：CRUD 與 lifecycle 行為混雜，後續 review slice 加 method 會繼續滾雪球

**取捨**：A 在 `LocalProvider` 結構上需要再開一個 `LocalStateMachineStore` struct（接 `Arc<Mutex<rusqlite::Connection>>`）；管理成本略增但與既有 `LocalChangeStore` / `LocalArtifactStore` 對稱。

### 並發控制：`change.version` CAS + 單 SQLite tx，不接 file lock

**選項**：

| 選項 | 並發控制 |
|---|---|
| A | `change.version` compare-and-swap + state mutation + audit insert 在同一 SQLite tx | 與 A2 sha256 etag 同 family；不依賴 locking slice |
| B | per-change file lock（`.git/speclink/locks/<change-id>.lock`），先取 lock 再 update | 需要 locking slice 的 lock manager；本 slice 等同抓 locking slice 進來 |
| C | A + B 都做（雙保險） | 重複設計，浪費實作 |

**選 A**。理由：

1. **與 A2 並發策略對齊**：A2 design.md 已決定「TOCTOU 視窗留給 slice B locking 收」、artifact 寫入靠 sha256 etag；本 slice state mutation 走 `change.version` CAS 是同 family
2. **SQLite tx 已提供必要原子性**：state update + audit insert 必須同 tx commit（design §6.2 明文）；任一失敗 rollback；不需要 OS-level lock
3. **CAS 提供 last-writer-wins safety**：兩個 agent 同時 `apply.start` 同一 change：先 commit 者 version 從 1→2 成功，後 commit 者帶 expected=1 撞 `state.version_conflict`，user 重試（design §12.5）
4. **B 在本 slice 沒實作 lock manager，等同抓 locking slice 進來**：違反 slice boundary 原則

**取捨**：CAS 在「同一 agent 多執行緒 race」場景靠 SQLite 內部 mutex；在「多 process 同時寫」場景靠 SQLite WAL mode 提供 reader-writer 隔離（A1 已啟 WAL mode）。`apply.start` 在 operations.md 標示 `Lock: change-exclusive`，本 slice 該欄位實作層暫時 stub 為 no-op，等 locking slice 接通 lock manager 後 retrofit。

## Implementation Contract

### Observable behavior

- 跑完 `init → new change → write 3 artifact (proposal/spec/tasks) → apply start → task done × N → apply pause` 序列，state 依序 `proposing → ready → in_progress → ready`；無 review CLI 介入
- 同序列在 task 全 done 階段，state 維持 `in_progress` 但 `change.all_tasks_done = 1`；`apply.start` 後續呼叫回 `{ state: "in_progress", actor: <new>, message: null }` 並 reassign actor
- `apply.start` 對 `code_reviewing` / `archived` state 回 `ok: true` + state 描述 + hint message，**非** error（spec 行為對齊 design §6.2 表）

### CLI surface

| Command | Inputs | Success output (data shape) | Error codes |
|---|---|---|---|
| `speclink apply start <change-id> [--actor <id>] [--json]` | change_id required；actor optional（缺則自動推導） | `{ change_id, state, actor: {agent_host, os_user, host_id} \| null, message: string \| null }` | `change.not_found` (2)、`state.transition_invalid` (7)、`state.version_conflict` (7) |
| `speclink apply pause <change-id> [--json]` | change_id required | `{ change_id, state, actor: null, message: string \| null }` | `change.not_found` (2)、`state.transition_invalid` (7)、`state.version_conflict` (7) |
| `speclink task list --change <id> [--json]` | change_id required | `{ tasks: [{ index, done, text }] }` | `change.not_found` (2)、`task.no_tasks_file` (2) |
| `speclink task done <task-index> --change <id> [--json]` | change_id required；task-index 1-based int required | `{ index, done: true, all_tasks_done: bool, state, auto_transitioned: bool }` | `change.not_found` (2)、`task.no_tasks_file` (2)、`task.index_out_of_range` (2)、`state.version_conflict` (7) |
| `speclink task undo <task-index> --change <id> [--json]` | change_id required；task-index 1-based int required | `{ index, done: false, all_tasks_done: false, state, reverted_from: "code_reviewing" \| null }` | `change.not_found` (2)、`task.no_tasks_file` (2)、`task.index_out_of_range` (2)、`state.version_conflict` (7) |

### Provider trait interface

`crates/provider/src/lib.rs` 新增（trait 簽章草案）：

```rust
pub trait StateMachineStore: Send + Sync {
    fn get_change_state(&self, change_id: &ChangeId) -> Result<ChangeStateView, ProviderError>;
    fn transition_state(
        &self,
        change_id: &ChangeId,
        expected_version: u64,
        request: TransitionRequest,
    ) -> Result<ChangeStateView, ProviderError>;
    fn set_actor(
        &self,
        change_id: &ChangeId,
        expected_version: u64,
        actor: Option<Actor>,
    ) -> Result<ChangeStateView, ProviderError>;
    fn set_all_tasks_done(
        &self,
        change_id: &ChangeId,
        expected_version: u64,
        done: bool,
    ) -> Result<ChangeStateView, ProviderError>;
}
```

`TransitionRequest { to_state: ChangeState, actor: Option<Actor>, reason: StateTransitionReason }`；`ChangeStateView { change_id, state, version, actor, all_tasks_done }`。所有 method 內部單一 SQLite tx；失敗自動 rollback；CAS 不一致回 `ProviderError::StateVersionConflict { current_version }`。

### Auto-transition contract（`artifact.write` hook）

A2 既有 `ArtifactOperations::write` 在「成功 atomic rename 之後 / 回 OK 之前」插入一段 evaluator：

1. 讀 change current state；若 `state ∈ {ready, in_progress, code_reviewing, archived}`，evaluator 跳過（轉移結束）
2. 列 `.speclink/changes/<name>/` 內 `proposal.md` + `tasks.md` + `specs/*/spec.md` 至少一份是否齊全
3. 齊全：依 walking-skeleton hard-coded `require_artifact_review=false` → call `StateMachineStore::transition_state` 從 `proposing` 推到 `ready`；warnings array 追加 `state_transitioned` warning code
4. 不齊全：no-op，state 維持 `proposing`，不寫 warning

Evaluator 對 `proposing` 以外的 state 是 no-op — 即使 user 在 `in_progress` state 改 proposal，也不會倒退 transition。

### `task.done` auto-trigger contract

`TaskOperations::done(change_id, index)` 流程（單 SQLite tx 對齊 design §6.2）：

1. 讀 tasks.md → parse checkbox list → 找對應 index；index 超出範圍回 `task.index_out_of_range`
2. 若該 task 已 `[x]` → idempotent no-op；不寫檔；仍回最新 `{ all_tasks_done, state }` 給呼叫者
3. 否則 atomic rename 寫回 tasks.md（line 改 `[ ]` → `[x]`）
4. 重新 parse；判斷「所有 task `[x]`」？是 → 進 5；否 → 回 `{ done: true, all_tasks_done: false, state: <unchanged>, auto_transitioned: false }`
5. 走 walking-skeleton hard-coded `require_code_review=false` 分支：call `StateMachineStore::set_all_tasks_done(change_id, expected_version, true)`；state 維持 `in_progress`；audit reason `task_done_auto`；回 `{ done: true, all_tasks_done: true, state: "in_progress", auto_transitioned: false }`
6. （`require_code_review=true` 分支實作完整但 walking-skeleton 路徑不會觸發；保留為 review slice 取消 hard-code 後自動生效）：call `transition_state(in_progress → code_reviewing, reason=task_done_auto)`；回 `{ auto_transitioned: true, state: "code_reviewing", all_tasks_done: true }`

`task.undo(change_id, index)` 流程：若該 task 已 `[ ]` → idempotent no-op；否則寫回 `[ ]`；若 change state == `code_reviewing` → 先 `transition_state(code_reviewing → in_progress, reason=task_undo_revert)` 再清 `all_tasks_done`；其他 state 只清 `all_tasks_done` flag。

### Walking-skeleton 端到端 acceptance

`crates/cli/tests/state_machine_e2e.rs` 必含一條端到端整合測，依序：

1. `speclink init`
2. `speclink new change wse-demo`
3. `speclink new artifact proposal --change wse-demo --stdin < ...` → assert state 仍 `proposing`（DAG 不齊）
4. `speclink new artifact spec auth --change wse-demo --stdin < ...` → assert state 仍 `proposing`（缺 tasks）
5. `speclink new artifact tasks --change wse-demo --stdin < ...` → assert state == `ready`、warning array 含 `state_transitioned`
6. `speclink apply start wse-demo` → assert state == `in_progress`、`actor != null`
7. `speclink task list --change wse-demo --json` → assert tasks array 長度 == tasks.md `- [ ]` 行數
8. `speclink task done 1 --change wse-demo` → assert `all_tasks_done: false`
9. （重複 done 直到最後一個 task）→ assert 最後一次 `all_tasks_done: true`、`auto_transitioned: false`、state == `in_progress`
10. `speclink apply pause wse-demo` → assert state == `ready`、actor cleared
11. （再次 `apply start` 驗證 idempotent + actor reassign）→ assert state == `in_progress`、actor != null

### Scope boundaries

**In scope**：5 個 CLI op + state machine module + StateMachineStore trait + state.db v3 migration + artifact.write hook + actor 推導 helper + tasks.md parser + 5 個 error code + walking-skeleton e2e 測。

**Out of scope**：review CLI、archive CLI、locking、schema YAML 解析、config 讀取、ingest revert、feedback marker、task marker、audit query CLI、HttpProvider impl。任何「需要動到 review/archive/locking/schema/config/ingest 的 sub-system」的工作必須拒收，留給對應 future change。

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `add-change-and-artifact-io` archive 未先完成 → `change-store` spec modify 路徑找不到既有 spec → `spectra validate` fail | Proposal `## Impact` 明文 prerequisite；本 slice apply skill 第一步檢查 `openspec/specs/change-store/spec.md` 存在性，缺失即 abort 並提示 user 跑 `/spectra-archive add-change-and-artifact-io` |
| `change.version` CAS 在多 agent 場景頻繁衝突，user 看到大量 `state.version_conflict` | 本 slice 透過 idempotency 設計大幅吸收衝突（apply.start / pause / task.done / undo 在「目標 state == 當前 state」時 no-op，不寫 row）；真實衝突場景僅限「兩 agent 同時 apply.start」這類 race，user 重試一次即解 |
| tasks.md 在 task.done 進行中被 user 手動編輯（reorder/insert）→ 後續 done 操作打到錯 task | spec 內明文「task.done 期間禁止改 tasks.md」契約；marker slice 上線後自動解除 |
| walking-skeleton hard-coded `require_*_review=false` 在團隊環境不符預期，user 不知道為什麼 transition 自動跳過 review | warnings array 在每次 auto-transition 附 hint warning code；release notes 明文 |
| v3 migration 跑過後 v2 binary 開不了 → 已 install slice A2 binary 的 user 升級後不可回退 | 對齊 §18.1 #67 forward-only 政策；release notes 明文 |
| `apply.start` 在 operations.md 標 `Lock: change-exclusive` 但本 slice 沒實作 lock manager → TOCTOU 視窗存在 | trait 簽章預留 future lock token 參數位（暫時用空 placeholder）；locking slice 接通時純加法擴充；本 slice 用 CAS + SQLite tx 保證 single-process 路徑原子性 |
| `state_transition` 表長期累積 row 無 prune → state.db 膨脹 | 本 slice 接受此限制；future archive slice 在 change archive 時可選擇性 cascade 清理 transition row（待 archive spec 決定）|
| Actor 推導 `os_user` 在 Windows 拿 `USERNAME` 與 Unix 拿 `USER` 路徑差異 → 跨平台行為不一致 | runtime helper 用 `whoami` crate（cross-platform）取代 raw env var 讀取，避免 OS-specific branching |

## Migration Plan

1. **Pre-apply check**：apply skill 開工前驗證 `openspec/specs/change-store/spec.md` 存在；不存在則 abort + 提示 archive A2
2. **State.db v3 migration**：`db.migrate(3)` 在 `LocalProjectStore::open_state_db()` 內首次執行；idempotent；任何已 install 此 binary 的 working dir 開檔即升級
3. **Forward-only**：升級後不可回退至 v2 binary；release notes 明文「After applying slice A3, the slice A2 binary will refuse to open the upgraded state.db with `state.db.schema_invalid`」
4. **Rollback strategy**：本 slice **不**支援 binary downgrade；若 user 必須回到 A2 binary，需手動刪 `.git/speclink/state.db` 後重跑 `speclink init`（會遺失 change row + actor + audit；user 須在 archive A2 後或備份 artifact 後操作）
5. **Walking-skeleton flag 切換時機**：當 `add-config-rw` slice 落地後，把 `state_machine::ReviewPolicy::walking_skeleton()` 改為 `ReviewPolicy::from_config(&config)`；transitions table 不變，僅 source 不同

## Open Questions

- **Q1**：`state_transition` 表是否需要 `request_id` 欄位以對齊 JSON envelope 的 `requestId`？
  - 暫定不加（本 slice 不暴露 audit query CLI，request_id 無 reader）；review slice 引入 `review history` CLI 時若需要追蹤可加 v4 migration
- **Q2**：`apply.start` 在 `code_reviewing` state 回 `message: "Already in code review; nothing to apply."`，但 walking-skeleton 4-state mode 永遠不會走到 `code_reviewing`。這條訊息要硬編還是預留 i18n hook？
  - 暫定硬編英文（與 design §11.8 i18n MVP scope 對齊：MVP 嚴格英文）；review slice 上線後此訊息會被實際 user 看到，i18n 設計屆時再決定
- **Q3**：DAG evaluator 在 `proposing → ready` 自動 transition 後，若 user 再次 `artifact.write` 一份 `design.md`（DAG 仍齊全），evaluator 應 no-op 還是 re-trigger？
  - 暫定 no-op（state 已非 `proposing`，evaluator 第一步直接 skip）；spec 內明文此行為
