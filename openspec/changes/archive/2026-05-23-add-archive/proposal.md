## Why

`add-state-machine-and-apply` 完成後 walking skeleton 卡在「`in_progress` + `all_tasks_done=true` flag」這一格 — 6-state lifecycle 的 `archived` 終態目前無人能進；user 即使把所有 task 跑完也無法收檔，整個 SDD cycle 還沒有「閉合」demo。Walking skeleton 第四片必須接通 `in_progress (all_tasks_done) → archived` transition、把 change/specs delta merge 進 `.speclink/specs/`、把 change 目錄搬進 `archive/`，讓 user 在 walking-skeleton mode（`require_*_review=false`）下可端到端跑完 `propose → ready → in_progress → archived` 的 4-state 主路徑。

設計依據 `doc/speclink-design.md` §6.2（archived terminal state + transition graph）、§14（filesystem layout 內 `.speclink/archive/` 區）、§16.9（archive CLI）、`doc/protocol/operations.md` 的 `archive.run` op spec。

## What Changes

- 新增 CLI 指令（屬「AI workflow 階段」，可由 skill 呼叫）：
  - `speclink archive <change-id> [--skip-specs] [--yes] [--no-validate] [--json]` — 把 change 從 `in_progress` 推進 `archived`；spec delta merge 進 capability spec、change 目錄搬到 `archive/<YYYY-MM-DD>-<change-id>/`；對 `archived` 再呼一次 → `state.transition_invalid` ；非互動環境 / `--yes` 跳過 prompt
- State machine 接通（design §6.2）：
  - `in_progress + change.all_tasks_done=1 → archived`（顯式 `archive.run` 驅動，reason=`archive_run`）；其他 state 一律 reject `state.transition_invalid`
  - walking-skeleton mode（`require_code_review=false`，A3 已硬編）下，archive 只看 `state=in_progress` + `all_tasks_done=1` 兩個條件；`require_code_review=true` 的 `code_reviewing + code_approved → archived` 路徑留 `add-review` slice，本 slice 不接
- State.db v4 migration（forward-only，遵守 §12.7 migration flow）：
  - `change` 表 alter：新增 `archived_at TIMESTAMP NULL`（archive 成功時寫；其他 state 為 NULL）
  - `_migrations` 表寫入 version=4 row
  - 不新增表（audit event 走既有 `state_transition` 表 + 走 stdout JSON envelope 帶回，本 slice 不引入專屬 audit log 表）
- Filesystem layout：
  - 成功 archive 時 atomic 把 `.speclink/changes/<change-id>/` rename 到 `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/`；`<YYYY-MM-DD>` 用 archive 時 UTC 日期；同日重名 → `<YYYY-MM-DD>-<change-id>-2` / `-3` ...（沿用 spectra 慣例）
  - `archive/` 目錄沿用 §14 既有設計、本 slice 確保 `speclink init` 之後在 first archive 時自動 create（init 階段不預建空目錄）
- Spec delta merge（walking-skeleton 最小可行版）：
  - 對 `.speclink/changes/<change-id>/specs/<capability>/spec.md` 每份檔案：
    - 若 `.speclink/specs/<capability>/` 不存在 → 整 dir create、檔案 atomic copy
    - 若 `.speclink/specs/<capability>/spec.md` 已存在 → atomic overwrite（不解析 delta、不 diff、不 conflict detect — schema-aware merge 留後續 schema slice）
  - 回傳 `merged_specs: [{ capability, lines_added, lines_removed }]`（lines_added = 新檔案總行數；lines_removed = 舊檔案總行數，若不存在則為 0）
  - `--skip-specs` 旗標跳過 merge、只搬目錄 + transition state、寫 audit `archive.specs_skipped`
- Provider trait 擴充（`crates/provider/src/lib.rs`）：新增 `ArchiveStore` trait（`archive_change(ArchiveRequest)`、`set_archived_at(change_id, archived_at)`）；所有 method 在同一 SQLite tx 內完成 state transition + `state_transition` audit insert + `archived_at` set + 目錄搬遷 + spec merge
- 新 error codes（design §17.4 命名規則）：
  - `change.tasks_incomplete`（exit 2）— `state=in_progress` 但 `all_tasks_done=0`；hint「complete all tasks first or use `--skip-specs` for emergency archive」（emergency 仍要 all_tasks_done=1；不開後門）
  - `validation.archive_failed`（exit 3）— code 預留、本 slice 路徑硬編 no-op；reserve 給 analyze slice 接 `--no-validate=false` 行為
  - `archive.specs_skipped`（audit event，非 error）— `--skip-specs` 時寫入
- JSON envelope：沿用 bootstrap / A2 / A3 既有 `{ ok, data, warnings?, requestId }` shape；data shape `{ change_id, state: "archived", merged_specs, archived_at, archive_dir }` 由 spec 章節定義
- Lock：設計要求 `change-exclusive + global-short`（design §12.2）；本 slice 用 stub no-op lock（lock impl 仍在 `add-locking-and-concurrency` slice）；single-RD 走單一 SQLite tx atomicity 保證 state transition + audit + 檔案搬遷 + spec merge 原子性

使用者情境：本 slice 全部 op 都屬於「AI workflow 階段」（可由 `spectra-archive` skill 呼叫）；無人類設定階段 CLI；無 CI 專屬 op。

## Non-Goals

- ❌ `code_reviewing + code_approved → archived` 路徑 — 由後續 `add-review` slice 負責；本 slice 不接 code review approval flow
- ❌ `--mark-tasks-complete` flag — emergency 用、會偽造 task completion 狀態；留 doctor slice（auto-fix 路徑）負責
- ❌ `--no-validate=false` 真實 validation — 依賴 analyze slice；本 slice 把 `--no-validate` flag 接進 CLI parse 但 runtime 路徑硬編「跳過 validation」，等 analyze slice 補 evaluator
- ❌ Schema-aware spec delta merge / conflict detection / 合理的 diff — 由後續 `add-schema-management` slice 負責；本 slice 採整檔覆蓋
- ❌ 真實 lock acquisition（per-change file lock + global advisory lock）— 由後續 `add-locking-and-concurrency` slice 負責；本 slice 只走 SQLite tx 原子性，TOCTOU 窗口暫時留著
- ❌ `archive` 撤銷 / `unarchive` 指令 — `archived` 是 terminal state（design §6.2）；需「放棄但留歷史」未來可加 `cancelled` state，不是本 slice 範圍
- ❌ Archive 後 `ingest` 反向 transition — design §6.2.1 已明列 `archived → 任何` 一律拒絕；本 slice 維持此規範、不開後門
- ❌ Audit query CLI（`speclink audit list / show`）— 本 slice 只寫 `state_transition` row + JSON envelope 帶 `change.archived` warning，查詢 CLI 留待後續 slice
- ❌ `change.code_review_pending` error code — 設計列為 archive op 可能 error，但屬 review 路徑；本 slice 不暴露、留 `add-review` slice
- ❌ HttpProvider 對應實作 — Provider trait method 在本 slice 加進 `crates/provider`，但只有 `provider-local` 提供 impl
- ❌ Tracking、telemetry、external notification（Slack / GitHub Issues 等）— 不在 MVP 範圍

## Capabilities

### New Capabilities

- `archive-runner`: `archive.run` op + CLI 介面、JSON envelope shape、state guard（`in_progress` + `all_tasks_done=1`）、spec delta merge 算法（walking-skeleton 整檔覆蓋版）、change 目錄搬遷規則（`archive/<YYYY-MM-DD>-<id>/` 含同日重名 suffix）、state.db v4 migration、`--skip-specs` / `--yes` / `--no-validate` flag 行為、新增 3 個 error / audit code 與對應 exit code

### Modified Capabilities

- `state-machine`: 解除「`archived` 終態目前不可達」限制；新增「`in_progress` + `all_tasks_done=1` → `archived`」legal transition（reason=`archive_run`）；`archived` terminal state 對所有 `apply.*` / `task.*` op 行為由「scenario 描述但無法觸發」改為「scenario 描述且必須可被 e2e 驗證」（A3 既有 scenario 文字不動，但因 A4 可達 `archived`，scenario 從 unreachable 變 reachable）

## Impact

- Affected specs:
  - New capability spec for `archive-runner`
  - Modified capability spec for `state-machine`
- Affected code:
  - Modified: crates/provider/src/lib.rs（新增 `ArchiveStore` trait；保留既有 `ProjectStore` / `ChangeStore` / `ArtifactStore` / `StateMachineStore`）
  - Modified: crates/provider/src/types.rs（新增 `ArchiveRequest` / `ArchiveResult` / `MergedSpec` struct、`StateTransitionReason::ArchiveRun` enum variant）
  - Modified: crates/provider/src/error.rs（在既有 `ProviderError` 加 2 個新 variant `ChangeTasksIncomplete` / `ValidationArchiveFailed`；在既有 `codes` module 加 2 個新 `pub const` error code + 1 個 audit event code `ARCHIVE_SPECS_SKIPPED`）
  - New: crates/provider-local/src/archive_store.rs（`LocalArchiveStore` 對 `ArchiveStore` trait 的具體 impl，含 single-tx state update + audit insert + 目錄搬遷 + spec merge）
  - Modified: crates/provider-local/src/state_db.rs（`MIGRATIONS` 陣列追加 v4 entry：alter `change` 加 `archived_at TIMESTAMP NULL`；新增 helper method `set_archived_at`）
  - Modified: crates/provider-local/src/store.rs（`LocalProjectStore::open_state_db()` 內 `db.migrate(3)` bump 為 `db.migrate(4)`）
  - Modified: crates/provider-local/src/lib.rs（pub use 新增 `LocalArchiveStore`）
  - New: crates/runtime/src/archive_ops.rs（`ArchiveOperations<G>` runtime entry 含 `run`，命名沿用 A2 / A3 `ChangeOperations<G>` / `ApplyOperations<G>` 慣例；含 spec merge helper + change dir rename helper）
  - Modified: crates/runtime/src/state_machine.rs（transition 表新增 `(InProgress, Archived, ArchiveRun)`；新增 `archived` terminal state 對所有 apply / task op 的 reject helper）
  - Modified: crates/runtime/src/error.rs（`RuntimeError` 加 2 個新 variant；`code()` 與 `exit_code()` match arm 擴充 2 個新 code）
  - Modified: crates/runtime/src/lib.rs（pub mod / pub use 新增 `archive_ops`）
  - New: crates/cli/src/commands/archive.rs（`speclink archive <change-id>` 命令實作；含 `--skip-specs` / `--yes` / `--no-validate` / `--json` flag）
  - Modified: crates/cli/src/commands/mod.rs（pub mod 列出新 archive module）
  - Modified: crates/cli/src/main.rs（`Commands` enum 加 `Archive` variant；match arm dispatch；`hint_for` 擴充 2 個新 code）
  - Modified: crates/cli/src/output.rs（`error_code_to_exit` match arm 擴充 2 個新 code）
  - New: crates/cli/tests/archive_walking_skeleton.rs（walking-skeleton 全路徑端到端：init → new change → write 3 artifacts → apply start → task done × N → archive → 驗證 `.speclink/specs/`、`.speclink/changes/archive/` 結果）
  - New: crates/cli/tests/archive_state_guards.rs（archive 對 `proposing` / `reviewing` / `ready` / `code_reviewing` / `archived` 五個非法 state 一律 reject、對 `in_progress + all_tasks_done=0` reject `change.tasks_incomplete`）
  - New: crates/cli/tests/archive_skip_specs.rs（`--skip-specs` 路徑：跳過 spec merge、僅搬目錄 + transition + audit `archive.specs_skipped`）
  - New: crates/runtime/tests/archive_ops.rs（runtime archive.run 單元測 + spec merge 邏輯測 + dir rename 邏輯測 + 同日重名 suffix 測）
  - New: crates/provider-local/tests/migration_v4.rs（v4 migration 加 `archived_at` 欄位 + 雙向 idempotent）
  - Modified: doc/speclink-design.md（slice naming table §1.1 補 A4 entry：`add-archive`；`§18.1 MVP` 章 walking-skeleton 段補 A4 出貨紀錄 — 但等本 change archive 時再 sync，本 slice 提案先列影響）
- 重用既有公開常量：`speclink_runtime::ARTIFACT_ROOT`（不重宣告）；`speclink_provider::codes::*` 既有 19 個 `project.*` + `change.*` + `artifact.*` + `state.*` + `task.*` code 保留不動
- Affected crates: `cli`、`runtime`、`provider`、`provider-local`
- Affected design refs: doc/speclink-design.md §1.1 (slice naming table、本 slice 落地 A4), §6.2 (6-state lifecycle、archived terminal state), §12.2 (lock 階層、本 slice 用 stub), §12.7 (schema migration flow), §13.9 (state storage layout), §14 (filesystem layout、archive 目錄), §16.9 (Archive CLI), §17.4 (error code naming), §17.6 (audit event 寫入時機保證); doc/protocol/operations.md `archive.run`
- Prerequisite: 本 change apply 前 user 必須先 archive `add-state-machine-and-apply`（已完成於 commit 3933dec），使 `openspec/specs/state-machine/spec.md` 出現在 working tree（Modified Capability 才能成立）
