## Why

`add-project-bootstrap` 完成後 LocalProvider 只能 `init / status / link / unlink`，state.db 只有 `project` 一張表，整個 SDD workflow 還沒「亮」起來。下一個 walking skeleton 切片必須讓人可以「建 change → 寫 artifact → 列 / 看 change」端到端跑通，才能在 slice C/D/E 上長出 state machine、apply、review、archive。

本 change 是 MVP walking skeleton 的第二片：bundle `change.* (4 ops)` + `artifact.* (2 ops)` + `spec.list-in-change (1 op)` 共 7 ops，外加 state.db v2 migration（只加 `change` 表）。Artifact body 嚴格走 filesystem，state.db 不存任何 artifact 內容；artifact 並發控制走 `sha256(file bytes)` 路徑、不引入新表。設計依據 doc/speclink-design.md §13.9 / §14 / §16.4 / §16.6 / §19.2 / §19.2.2 / §21.5。

## What Changes

- 新增 CLI 指令（皆屬「AI workflow 階段」，可由 skill 呼叫）：
  - `speclink new change <name>` — 建立 change，寫 state.db `change` row + scaffold `.speclink/changes/<name>/` 目錄；state 一律 stub 為 `proposing`
  - `speclink list --changes` — 列舉所有 change（從 state.db `change` 表查），輸出 change_id、name、state、updated_at
  - `speclink show change <name>` — 顯示單一 change metadata（state.db row）+ 該 change 下既有 artifact 清單（從 filesystem 列）
  - `speclink delete change <name>` — destructive，刪 state.db row + filesystem 目錄；需要 `--confirm-name <name>` 二次確認
  - `speclink new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <sha256>]` — 寫 artifact；新檔 expected_etag 必須為 null、覆寫必須帶現檔 sha256
  - `speclink artifact read <kind> --change <name> [--capability <cap>]` — 讀 artifact；回 content + 即時算的 sha256 etag
  - `speclink list --specs --change <name>` — 列舉某 change 下所有 spec 的 capability id（從 filesystem `specs/` dir listing）
- State.db v2 migration（forward-only，遵守 §12.7 migration flow）：
  - 新增 `change` 表：`change_id TEXT PK / name TEXT UNIQUE NOT NULL / state TEXT NOT NULL / schema_id TEXT NOT NULL / version INTEGER NOT NULL DEFAULT 1 / created_at TIMESTAMP NOT NULL / updated_at TIMESTAMP NOT NULL`
  - `_migrations` 表寫入 version=2 row
  - 不新增 artifact 表（artifact 完全 filesystem-backed）
- Filesystem layout（disk contract，§14）：
  ```
  .speclink/changes/<change-id>/
    proposal.md            # 唯一
    design.md              # 唯一（optional）
    tasks.md               # 唯一
    specs/<capability>/spec.md   # 多份
  ```
- Artifact Etag 規則（依 §19.2.2 optimistic concurrency）：
  - `artifact.read`：`fs::read(path)` → `sha256(bytes)` → 回 `Versioned { content, etag: sha256_hex }`
  - `artifact.write` 新建（檔不存在）：`expected_etag == null` → 寫入並回新 etag；`expected_etag != null` → `artifact.not_found`
  - `artifact.write` 覆寫（檔已存在）：`expected_etag == null` → `artifact.version_conflict`（禁止盲寫）；`expected_etag == sha256(current_bytes)` → atomic rename 寫入並回新 etag；mismatch → `artifact.version_conflict`
  - 寫入採 `tempfile + atomic rename` 序列；TOCTOU 窗口留給 slice B locking 收
- Change row Etag：state.db `change.version` 欄位（monotonic counter），給未來 state transition / metadata 更新時 RMW 用；slice A 只在 create 寫 version=1，不做任何 update
- 新 error codes：`change.not_found`（exit 2）、`change.duplicate_name`（exit 7）、`change.invalid_name`（exit 2）、`artifact.kind_invalid`（exit 2）、`artifact.capability_required`（exit 2）、`artifact.not_found`（exit 2）、`artifact.version_conflict`（exit 7）
- JSON envelope：所有 7 個 op 沿用 bootstrap slice 既有 `{ ok, data, warnings?, requestId }` / `{ ok: false, error: { code, message, hint?, retryable, retry_after_ms? }, requestId }` 形狀；新增的 data shape 由 spec 章節定義

## Non-Goals

- ❌ State machine 6 狀態 transition（`reviewing` / `ready` / `in_progress` / `code_reviewing` / `archived`）— 由後續 `add-state-machine-and-apply` 負責；slice A `change.state` 一律寫 `proposing`、`change.show` 顯示但不轉移
- ❌ Schema YAML 解析與 artifact body 結構驗證 — 由後續 `add-schema-management` 負責；slice A `artifact.write --kind` 只做白名單檢查（`proposal` / `design` / `tasks` / `spec`），不解析 schema 內容
- ❌ Locking / lock 階層 / stale lock 接管 — 由後續 `add-locking-and-concurrency` 負責；slice A 寫入採 atomic rename 防 partial write，但 TOCTOU 窗口不收
- ❌ `apply.start` / `apply.pause` / `task.done` / `instructions.<kind>` — 由後續 slice 負責
- ❌ `review.approve` / `review.reject` / `review.history` / `archive.run` / `discuss.*` — 各自獨立 change
- ❌ `config.read` / `config.write`（`.speclink/config.yaml`）— 由後續 `add-config-rw` 負責
- ❌ HttpProvider 對應實作 — Provider trait method 在本 change 加進 `crates/provider`，但只有 `provider-local` 提供 impl；HttpProvider impl 統一延後
- ❌ `artifact.write --overwrite` 逃生口（盲寫覆蓋）— MVP 範圍外，不入 slice A
- ❌ Change history / audit log / touched files index — 由各自的 slice 負責
- ❌ Cross-branch artifact missing 偵測（`change.artifact_missing` op-level reject）— 留給 doctor / restore slice
- ❌ 跨 platform CI 矩陣 — 本 change 在 bootstrap 既有 CI 矩陣下跑，不新增 platform job

## Capabilities

### New Capabilities

- `change-store`: state.db `change` 表 schema、change CRUD（`change.create` / `change.list` / `change.show` / `change.delete`）、change name validation 規則、change 與 filesystem dir 的雙向綁定、change-store 並發控制（`change.version` etag）
- `artifact-io`: filesystem-backed artifact 讀寫（`artifact.read` / `artifact.write` / `spec.list-in-change`）、artifact kind 白名單、sha256-based etag 並發控制、atomic write 順序、artifact path 推導規則（含 `kind=spec` 的 `--capability` 規範）

### Modified Capabilities

(none — 本 change 在 disk layout 上沿用既有 two-root contract，沒有任何既有 capability 的 requirement 被改寫)

## Impact

- Affected specs:
  - New capability spec for change-store
  - New capability spec for artifact-io
- Affected code（路徑與命名沿用 bootstrap 既有結構：error 走單一 `ProviderError` enum、runtime 走 `ops`-style 命名、tests 落在各 crate 自己的 `tests/` 目錄）：
  - Modified: crates/provider/src/lib.rs（新增 `ChangeStore` + `ArtifactStore` trait 宣告；保留既有 `ProjectStore`）
  - Modified: crates/provider/src/types.rs（新增 `Etag` newtype、`Versioned<T>`、`ExpectedEtag` enum、`ArtifactKind` enum、`ChangeRow` struct、`validate_kebab_id` helper）
  - Modified: crates/provider/src/error.rs（在既有 `ProviderError` 單一 enum 上加 7 個新 variant；在既有 `codes` module 加 7 個新 `pub const` error code）
  - New: crates/provider-local/src/change_store.rs（`LocalChangeStore` 對 `ChangeStore` trait 的具體 impl）
  - New: crates/provider-local/src/artifact_store.rs（`LocalArtifactStore` 對 `ArtifactStore` trait 的具體 impl，含 sha256 + atomic rename helper）
  - Modified: crates/provider-local/src/state_db.rs（`MIGRATIONS` 陣列追加 v2 entry；新增 `insert_change_row` / `get_change_by_name` / `list_changes` / `delete_change_by_name` 等 helper method）
  - Modified: crates/provider-local/src/store.rs（`LocalProjectStore::open_state_db()` 內 hardcoded `db.migrate(1)` bump 為 `db.migrate(2)`，確保 v2 schema 對所有 LocalProvider 開檔路徑都套用）
  - Modified: crates/provider-local/src/lib.rs（pub use 新增 `LocalChangeStore` 與 `LocalArtifactStore`）
  - New: crates/runtime/src/change_ops.rs（`ChangeOperations<G>` struct，runtime 層 change CRUD entry，命名沿用 bootstrap `ops.rs::Operations<G>` 慣例）
  - New: crates/runtime/src/artifact_ops.rs（`ArtifactOperations<G>` struct，runtime 層 artifact I/O entry）
  - Modified: crates/runtime/src/error.rs（`RuntimeError` 加 7 個新 variant；`code()` match arm 擴充；`exit_code()` match arm 擴充覆蓋 7 個新 code 的 exit 對照）
  - Modified: crates/runtime/src/lib.rs（pub mod / pub use 新增兩個 ops 模組）
  - New: crates/cli/src/commands/new_change.rs
  - New: crates/cli/src/commands/new_artifact.rs
  - New: crates/cli/src/commands/list_changes.rs（`speclink list --changes` 對應）
  - New: crates/cli/src/commands/list_specs.rs（`speclink list --specs --change <name>` 對應）
  - New: crates/cli/src/commands/show_change.rs（`speclink show change <name>` 對應）
  - New: crates/cli/src/commands/delete_change.rs（`speclink delete change <name>` 對應）
  - New: crates/cli/src/commands/artifact_read.rs
  - Modified: crates/cli/src/commands/mod.rs（pub mod 列出 7 個新 module）
  - Modified: crates/cli/src/main.rs（`Commands` enum 加 7 個新 variant；match arm dispatch；`hint_for` 擴充 7 個新 code）
  - Modified: crates/cli/src/output.rs（`error_code_to_exit` match arm 擴充 7 個新 code）
  - New: crates/cli/tests/change_crud.rs（`speclink new/list/show/delete change` 端到端整合測，沿用 bootstrap `crates/cli/tests/cli.rs` 樣板）
  - New: crates/cli/tests/artifact_io.rs（`speclink new artifact` / `artifact read` / `list --specs` 整合測）
  - New: crates/cli/tests/etag_concurrency.rs（artifact.write etag matrix 5 行整合測）
  - New: crates/runtime/tests/change_ops.rs（runtime change CRUD 測）
  - New: crates/runtime/tests/artifact_ops.rs（runtime artifact I/O + etag matrix 測）
- 重用既有公開常量：`speclink_runtime::ARTIFACT_ROOT`（不重宣告）；`speclink_provider::codes::*` 既有 4 個 `project.*` code 保留不動
- Affected crates: `cli`、`runtime`、`provider`、`provider-local`
- Affected design refs: doc/speclink-design.md §13.9 (state storage layout)、§14 (檔案結構)、§16.4 (change CRUD)、§16.6 (artifact 寫入)、§19.2 (Provider trait)、§19.2.2 (Versioned<T> + expected_etag)、§21.5 (catalogue 對 CLI/SDK/HTTP mapping)
