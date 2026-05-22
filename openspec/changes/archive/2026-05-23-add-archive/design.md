## Context

A3 (`add-state-machine-and-apply`, archived in commit `3933dec`) 把 6-state lifecycle 拉通到 `in_progress + all_tasks_done=true` 那一格就停住：

- `archived` terminal state 沒人能進
- `all_tasks_done=1` flag 沒有 consumer
- `apply.start` / `apply.pause` / `task.done` 的「`archived` state 行為」scenario 在 A3 spec 寫了但無法 e2e 觸發
- Walking-skeleton mode（`require_*_review=false`，A3 硬編）下，user 跑完所有 task 也無法收檔

`doc/speclink-design.md` §6.2 與 `doc/protocol/operations.md:3198` (`archive.run`) 已定義 `archive` op 的契約：spec delta merge + change dir rename + state → archived + audit event。本 slice 用「walking-skeleton 最小可行」原則接通主路徑，把複雜路徑（review approval、schema-aware spec merge、真實 lock、validation hook）顯式留給後續 slice。

**Constraint baseline**：
- A3 既有 6-state lifecycle、`StateMachineStore` trait、`state_transition` audit 表、`change.version` CAS、actor 機制全部沿用
- state.db 目前 v3；本 slice forward-only migrate 到 v4（加 `archived_at` column）
- A1 既有 two-root storage（`.speclink/` artifacts + `.git/speclink/` state）契約沿用
- A2 既有 artifact CRUD + atomic rename helper 沿用
- 走 spectra-cli 既有 `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/` 命名慣例（與 spectra 的 archive 機制不對齊功能、僅復用目錄命名格式）

## Goals / Non-Goals

**Goals:**

1. 接通 `in_progress + all_tasks_done=1 → archived` legal transition，讓 walking-skeleton mode 端到端 SDD cycle 可閉合
2. 實作 `archive.run` op + `speclink archive <change-id>` CLI surface
3. 把 change 內 `specs/<capability>/spec.md` delta merge 進 `.speclink/specs/<capability>/spec.md`（最小可行版：整檔覆蓋）
4. 把 change 目錄 atomic rename 到 `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/`
5. state.db v4 migration：`change` 表加 `archived_at TIMESTAMP NULL`
6. 寫 `state_transition` audit row（reason=`archive_run`）+ JSON envelope `warnings` 帶 `archive.specs_skipped`（若觸發 `--skip-specs`）
7. 新增 2 個 user-facing error code：`change.tasks_incomplete`（exit 2）、`validation.archive_failed`（exit 3，預留，本 slice 路徑不會觸發）

**Non-Goals:**

1. ❌ `code_reviewing + code_approved → archived` 路徑（留 `add-review` slice）
2. ❌ Schema-aware spec delta merge（A2 archived spec.md 是 delta 格式 `## ADDED Requirements` / `## MODIFIED Requirements`，但本 slice 用 dumb 整檔覆蓋；schema-aware delta 解析留 `add-schema-management` slice）
3. ❌ 真實 file lock（per-change exclusive + global short）— 留 `add-locking-and-concurrency` slice；本 slice 用 single SQLite tx 保證 atomicity
4. ❌ `--mark-tasks-complete` flag（emergency forcing；留 doctor slice）
5. ❌ `--no-validate=false` 真實 validation hook（留 analyze slice；本 slice CLI parse flag 但 runtime 路徑硬編 no-op）
6. ❌ `unarchive` / `cancelled` state / archive revert（design §6.2 archived 是 terminal）
7. ❌ Archive 後 `ingest` 反向 transition（design §6.2.1 已明列 ✗ 拒絕）
8. ❌ HttpProvider 對應實作（trait 加 method、但 impl 留 deferred）
9. ❌ External notification / telemetry / GitHub PR auto-link

## Decisions

### Spec delta merge：整檔覆蓋 vs schema-aware diff

**Decision**：採整檔覆蓋。`.speclink/changes/<id>/specs/<cap>/spec.md` 整個檔內容 atomic 寫到 `.speclink/specs/<cap>/spec.md`；目標 capability dir 不存在則 create、存在則 overwrite。`lines_added` = 新檔 line count、`lines_removed` = 舊檔 line count（不存在則 0）。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. 整檔覆蓋（本 slice 採用） | 簡單、可預測、無 conflict、不依賴 schema 解析；缺點：A2-style「MODIFIED Requirements」delta 格式進不去（會覆蓋既有 spec body） | 採用 — walking-skeleton 階段；schema slice 補 delta 解析 |
| B. Schema-aware delta merge：parse `## ADDED Requirements` / `## MODIFIED Requirements` / `## REMOVED Requirements`、套 line-level patch | 對 modified capability 才對；但本 slice 還沒 schema parser、要硬塞會把 archive slice 範圍翻倍 | 駁回 — 由 `add-schema-management` slice 接管 |
| C. 兩階段：本 slice 先 dumb merge、後續 slice 補 delta；但 dumb merge 對既有 spec 是「破壞性」覆蓋 | walking-skeleton 階段 `.speclink/specs/<cap>/` 多半是空（A1/A2/A3 已經由 spectra promotion 進來，但 SpecLink 自己的 archive 還沒 run 過任何 change），dumb merge 等於「第一次寫入」；破壞性極低 | 採 A、走時序順序 |

**Risk**: 若有人在 SpecLink 自己跑過幾輪 archive 後跑 schema slice，過去 archive 寫入的 capability spec 已被 dumb 覆蓋；schema slice 必須 idempotent + 可從 git history 重建。本 slice **不**處理此 risk；schema slice 自處理。

### Lock acquisition：stub no-op vs 真 lock

**Decision**：採 stub no-op lock helper。`ArchiveOperations::run` 在進入 archive 流程前 call `acquire_change_exclusive(change_id)` + `acquire_global_short()`，兩個 helper 在本 slice 都返回 `Ok(NoopGuard)`（compile-time stub）；guard drop 時 no-op。所有 atomic guarantee 走「single SQLite transaction wrap DB writes」+「filesystem rename 自身原子」。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. Stub helper（本 slice 採用） | 介面提早立、impl 留後；archive op 的 lock acquire site 跟 lock slice 對齊；single-RD 場景無 race | 採用 |
| B. 真 lock（per-change advisory + global advisory） | walking-skeleton 不需要、且 lock slice 還沒設計具體 SQL/file primitive | 駁回 — 範圍蔓延 |
| C. 不留 lock 介面、lock slice 自己進來插 | 之後 lock slice retrofit 要改 archive_ops.rs；介面對 caller 隱形 | 駁回 — 提早立介面成本低 |

**Risk**: 同機多 agent 同時對同 change 跑 archive → 兩條 path 都進、其中一條 SQLite tx commit 後另一條的 `change.version` CAS 失敗、rollback DB；但目錄 rename 在 commit 之後執行、可能兩條都嘗試 rename → 第二條 rename fail（source 已不存在）+ runtime error。**Mitigation**：rename 在 tx commit 後執行；rename 失敗時把 DB 狀態 best-effort revert（再開一條 tx 把 state 推回 `in_progress` + clear `archived_at`）；revert 失敗 → bubble up `runtime.atomicity_compromised` warning（本 slice 不新加 error code、用 `state.version_conflict` 警示）。

### State guard：兩條件 AND vs 完整 6-state check

**Decision**：walking-skeleton mode 下，state guard 只看 (1) `change.state == 'in_progress'` (2) `change.all_tasks_done == 1`。其他 state（含 `code_reviewing`）一律 reject `state.transition_invalid`；`in_progress` 但 `all_tasks_done=0` reject `change.tasks_incomplete`。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. 兩條件 AND（本 slice 採用） | 簡單、與 walking-skeleton mode 對齊；`code_reviewing` 路徑留 review slice 接 | 採用 |
| B. 預先接通 `code_reviewing + code_approved`（read placeholder column） | A3 沒 `code_approved` column、要先加 column 才能 reject sensibly；範圍蔓延 | 駁回 |
| C. `in_progress` 一個條件、`all_tasks_done` 不查 | 違反 design §6.2「all task done 才可 archive」契約 | 駁回 |

### `--no-validate` flag 行為：硬編 no-op vs 拒絕未實作

**Decision**：CLI parse `--no-validate` flag、runtime 路徑硬編「跳過 validation step」（等同 `--no-validate=true` 永遠生效）。本 slice 不暴露 validation step 也不暴露 `validation.archive_failed` error path。flag 保留是為了 forward-compat：analyze slice 接 evaluator 時、CLI surface 不需要改。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. parse + 硬編 no-op（本 slice 採用） | CLI 形狀對 design §16.9 一致、analyze slice 接 evaluator 無 CLI breaking change | 採用 |
| B. 不暴露 flag、analyze slice 加 | 後續 CLI surface 有 breaking change；user / skill 要適配兩次 | 駁回 |
| C. parse + 拒絕「未實作」error | user / skill 看到 `not_implemented` 體驗差 | 駁回 |

### Archive 目錄命名：日期前綴 vs 純 change-id

**Decision**：`.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/`，`<YYYY-MM-DD>` 用 archive 時的 UTC 日期。同日同 change-id 衝突（理論上罕見、e.g. unarchive 之後 re-archive 但本 slice 不接 unarchive） → `-2` / `-3` suffix。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. `<YYYY-MM-DD>-<change-id>`（本 slice 採用） | 與 spectra 慣例對齊（雖兩工具不對齊功能）；按時序自然排序 | 採用 |
| B. `<change-id>` 純名 | 同名 change 重複 archive 會撞名（雖本 slice 不接 unarchive、未來可能加 cancelled state） | 駁回 |
| C. `<ULID>-<change-id>` | 無人看；user 視覺辨識度差 | 駁回 |
| D. `<change-id>-<YYYY-MM-DD>` | 按 change-id 排序、不是時序；user 想看「最近 archive 哪幾個」要 scroll | 駁回 |

### State.db v4 migration：加 column vs 新表

**Decision**：alter `change` 表加 `archived_at TIMESTAMP NULL`，非 archived state 一律 NULL。不新增 archive 專屬表。

**Alternatives considered:**

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. Alter `change` 加 column（本 slice 採用） | 與 `actor_json` / `all_tasks_done` 既有 pattern 一致；query 簡單 | 採用 |
| B. 新表 `archived_change`（`change_id PK / archived_at / merged_specs_json`） | normalize 但要 JOIN；本 slice 沒 query CLI 用不到 | 駁回 |
| C. 用既有 `state_transition` 表的 `transitioned_at`（reason=`archive_run` 那筆） | 可以但要 query 時 JOIN；`archived_at` 是常用屬性、直接落在 `change` row 更省 | 駁回 |

### Audit event：reuse `state_transition` row vs 新表

**Decision**：reuse 既有 `state_transition` 表，本 slice 寫一筆 `reason='archive_run'` row（A3 既有 reason enum 加 `ArchiveRun` variant）。`--skip-specs` 觸發時，warning 走 JSON envelope 的 `warnings` array、code=`archive.specs_skipped`、details=`{ "capabilities_skipped": [...] }`；不寫 audit log 表（本 slice 不引入專屬 audit table、留後續 audit slice）。

## Implementation Contract

### Observable behavior

User 跑 `speclink archive <change-id>` 後：

1. **happy path（walking-skeleton mode）**：
   - State 從 `in_progress` 變 `archived`
   - `change.archived_at` 寫入 UTC ISO-8601 timestamp
   - `.speclink/changes/<change-id>/` 目錄消失
   - `.speclink/changes/archive/<YYYY-MM-DD>-<change-id>/` 目錄出現（內容與原 change dir 1:1）
   - 對 change 內每份 `specs/<capability>/spec.md`：對應 `.speclink/specs/<capability>/spec.md` 整檔被覆蓋（若該 capability dir 不存在則 create）
   - `state_transition` 表多一筆 `(change_id, from='in_progress', to='archived', reason='archive_run', transitioned_at=now)` row
   - exit code 0、JSON envelope 帶 `data: { change_id, state: "archived", merged_specs: [...], archived_at, archive_dir }`、`warnings: []`

2. **`--skip-specs` path**：
   - 同 happy path、但 `.speclink/specs/<capability>/spec.md` 不被覆蓋
   - JSON envelope `warnings` 加一筆 `{ code: "archive.specs_skipped", message, details: { capabilities_skipped: [...] } }`
   - audit `state_transition` row reason 仍是 `archive_run`（不另開 `archive_run_skipped`）

3. **error paths**：
   - State 不在 `in_progress` → `state.transition_invalid`、exit 7、無檔案 / DB 變動
   - State 在 `in_progress` 但 `all_tasks_done=0` → `change.tasks_incomplete`、exit 2、無檔案 / DB 變動
   - Change 不存在 → `change.not_found`、exit 2
   - Filesystem rename 失敗（罕見、e.g. cross-device、permission）→ 嘗試 best-effort revert DB；revert 也失敗 → bubble up；exit 1（runtime general error）

### CLI surface

```
speclink archive <change-id> [--skip-specs] [--yes] [--no-validate] [--json]
```

- `<change-id>` positional、required
- `--skip-specs`：跳過 spec delta merge、僅搬目錄 + transition state（emergency 用）
- `--yes`：跳過互動 confirm prompt（CLI 預設不 prompt — 因為本 slice 屬 AI workflow op、非 destructive；flag 保留以對齊 design §16.9 catalogue、不引入新 prompt 行為）
- `--no-validate`：parse 但 runtime no-op；analyze slice 接管
- `--json`：machine-readable envelope（與 A1/A2/A3 一致）

### JSON envelope shape

Success（`data` shape）：

```json
{
  "change_id": "demo-billing",
  "state": "archived",
  "merged_specs": [
    { "capability": "user-auth", "lines_added": 142, "lines_removed": 0 },
    { "capability": "audit-log", "lines_added": 87, "lines_removed": 64 }
  ],
  "archived_at": "2026-05-22T18:00:00Z",
  "archive_dir": ".speclink/changes/archive/2026-05-22-demo-billing"
}
```

`--skip-specs` 觸發時 `merged_specs: []`、`warnings` 帶 `archive.specs_skipped`。

Error envelope 沿用 A3 既有 `{ ok: false, error: { code, message, hint, retryable, retry_after_ms }, requestId }` shape。新增 2 個 `(code, retryable)` 對：
- `change.tasks_incomplete` → retryable=`no`、hint="complete all tasks first with `speclink task done <i> --change <id>`"
- `validation.archive_failed` → retryable=`no`、hint="run `speclink validate <id>` first"（本 slice 不觸發）

### Provider trait surface

新增 `crates/provider/src/lib.rs::ArchiveStore` trait：

```rust
pub trait ArchiveStore: Send + Sync {
    fn archive_change(&self, req: ArchiveRequest) -> Result<ArchiveResult, ProviderError>;
}
```

`ArchiveRequest` / `ArchiveResult` 在 `crates/provider/src/types.rs` 新增、shape 對齊上面 JSON envelope。`StateTransitionReason::ArchiveRun` 加進 A3 既有 enum。

`LocalArchiveStore` 在 `crates/provider-local/src/archive_store.rs` impl：用 single SQLite tx wrap 「`change.state` update + `change.archived_at` set + `state_transition` insert」；tx commit 後 atomic rename change dir；rename 成功後 atomic write 各 capability spec.md（每份用 tempfile + rename，與 A2 既有 helper 一致）；任一階段失敗按 above「filesystem rename 失敗」regress。

### Acceptance criteria

| 行為 | 驗證手段 |
|---|---|
| `archive in_progress + all_tasks_done` happy path | `crates/cli/tests/archive_walking_skeleton.rs::archive_happy_path_walking_skeleton` |
| 5 個非法 state 都被 reject | `crates/cli/tests/archive_state_guards.rs::archive_rejects_<state>` × 5 |
| `in_progress + all_tasks_done=0` reject `change.tasks_incomplete` | `crates/cli/tests/archive_state_guards.rs::archive_rejects_in_progress_without_all_tasks_done` |
| `--skip-specs` 路徑 | `crates/cli/tests/archive_skip_specs.rs::skip_specs_writes_audit_warning_and_no_spec_files` |
| state.db v4 migration idempotent | `crates/provider-local/tests/migration_v4.rs::migration_v4_idempotent` |
| Spec merge：new capability dir | `crates/runtime/tests/archive_ops.rs::spec_merge_creates_new_capability_dir` |
| Spec merge：existing capability dir 整檔覆蓋 | `crates/runtime/tests/archive_ops.rs::spec_merge_overwrites_existing_spec_md` |
| Archive dir 同日重名 → suffix | `crates/runtime/tests/archive_ops.rs::archive_dir_same_day_collision_appends_suffix` |
| A3 既有「archived state on apply.start」scenario 從 unreachable 變 reachable | `crates/cli/tests/archive_walking_skeleton.rs::apply_start_on_archived_returns_hint`（e2e：先 archive 再 apply start） |

### Scope boundaries

**In scope**：
- archive.run op（含 `--skip-specs` / `--yes` / `--no-validate` flag、後二者 no-op）
- `ArchiveStore` trait + `LocalArchiveStore` impl
- state.db v4 migration（`archived_at` column）
- `ArchiveOperations<G>` runtime entry + spec merge helper + dir rename helper
- `crates/cli/src/commands/archive.rs`
- 2 個新 error code + 1 個 audit event code
- 9 個新 / 修改的 integration / unit test 檔
- A3 既有 unreachable scenario 的 e2e 啟動驗證

**Out of scope**：
- Review path（`code_reviewing → archived`）
- Schema-aware spec merge
- 真實 lock primitive impl
- `--mark-tasks-complete` flag 行為
- `unarchive` / `cancelled` state
- HttpProvider impl

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Dumb 整檔覆蓋會破壞既有 capability spec.md 內容 | Walking-skeleton 階段 `.speclink/specs/<cap>/` 多半是 SpecLink 自身首次寫入（之前都是 spectra 工具寫的）；未來 schema slice 接管 delta merge 後、user 才會看到「merge 而非覆蓋」；本 slice 在 CLI help / propose 文件明寫此限制 |
| Single SQLite tx commit 後 filesystem rename 失敗 → state 與檔案不一致 | tx commit 後 rename；rename 失敗 best-effort revert DB；revert 失敗 bubble up + exit 1；user 走 doctor / restore CLI（後續 slice）排查；本 slice 文件警告此 corner case |
| Stub lock 在多 agent 同時 archive 同 change 下 race | A3 `change.version` CAS 在 tx 內保護 DB 一致；race 第二位會在 SQLite tx commit 階段 fail；filesystem rename 第二位會在 source 不存在時 fail；不會雙寫 |
| `--no-validate` flag 為 no-op 但 skill / user 可能誤以為已 validate | help text 明寫「本 slice 不執行 validation；analyze slice 接通後生效」；release notes 同步 |
| Cross-device rename（罕見、e.g. `.speclink/` mount 在不同 device） | `fs::rename` 失敗時 fallback 不做（不引入 copy + delete fallback）；error 給 user；本 slice 不處理 cross-device、留 doctor slice |
| Archive 後 user 想反悔 → 無 unarchive | design §6.2 明文 archived terminal；user 走 `change.delete` 開新 change；本 slice 不開後門 |
| Single-RD walking-skeleton 對 archive 過程被 Ctrl-C 中斷 | SQLite tx 自動 rollback；若已 commit 但 rename 未開始 → 下次 archive 同 change 會 reject `state.transition_invalid`（已 archived state）；user 手動 inspect `.git/speclink/state.db` 與 `.speclink/changes/<id>/` 不一致；doctor slice 補 finding `state.archive_inconsistent` |
