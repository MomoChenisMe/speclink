## Why

Bootstrap change 已釘死 Provider trait 與 propose create，但目前 SpecLink 只能寫入 proposal 一種 artifact，缺少 design、tasks、spec 三類 artifact 的寫入路徑，也沒有任何指令可以查詢 change 目前的進度狀態。這代表完整的 SDD workflow（從 proposal 到實作前的所有 artifact 都齊備）尚無法在 SpecLink 上跑完。本 change 補完 local provider 上「寫入多 artifact + 觀察 change 狀態」這條 vertical slice，作為後續用 SpecLink dogfood 開發 SpecLink 自身（取代目前依賴 Spectra workflow）的前置條件。

## What Changes

- 在 `crates/cli/` 新增 AI workflow 指令 `speclink artifact write <kind> --change <id> [--capability <name>] --stdin --json`，`<kind>` 可為 `design` / `tasks` / `spec` 三種
- 在 `crates/cli/` 新增 AI workflow 指令 `speclink status --change <id> --json`，輸出該 change 的 artifact DAG（每個 artifact 的 id、path、是否存在、是否標記為 done）
- 在 `crates/provider-local/` 擴張 `LocalProvider::write_artifact` 支援 `ArtifactKind::{Design, Tasks, Spec}`，沿用 bootstrap change 的原子寫入策略（temp file + rename，失敗時 cleanup）
- 在 `crates/provider-local/` 為 spec artifact 加入 `--capability <name>` 路由 — 寫入 `.speclink/changes/<id>/specs/<capability>/spec.md`
- 在 `crates/provider/` 新增 trait method `Provider::get_status(project_id, change_id) -> Result<ChangeStatus, ProviderError>` 與對應 `ChangeStatus` 資料型別（artifact list + per-artifact status + change-level state）
- 在 `crates/provider-local/` 實作 `get_status`，掃描 `.speclink/changes/<id>/` 目錄、讀 `metadata.json`、判定每個 artifact 是否存在
- 在 `crates/runtime/` 新增 `write_artifact(provider, input)` 與 `get_status(provider, input)` 兩個 orchestration 函式
- 沿用 bootstrap change 釘死的 envelope schema、exit code 與 error code naming，不變更 cli-machine-interface
- 新增 2 個 capability spec：`cli-artifact-write`、`cli-status`；修改 `local-provider-storage` 補上多 artifact 的目錄結構與寫入語意

**目標使用者情境**：
- **AI skill 呼叫階段**：skill 在收到 `propose create` 成功後，繼續用 `artifact write design --stdin` / `artifact write tasks --stdin` / `artifact write spec --capability <name> --stdin` 補齊所有 artifact，再用 `status` 確認 change 已就緒
- **工程師本機階段**：可在沒有任何遠端設定下走完寫入流程
- **CI 階段**：暫不涉及

## Non-Goals

- 不實作 archive 指令與 lifecycle state 從 `proposed` 推進的邏輯 — 留給下一個 change (`add-local-archive`)
- 不實作 `instructions` 指令與 per-artifact guidance 內容 — 留給 Change 4 (`add-instructions-and-task-done`)
- 不實作 `task done` 指令與 tasks.md 的 checkbox 解析 — 同上
- 不實作 spec sync（delta merge 到主 `openspec/specs/`）— 屬於 archive 階段，留給下一個 change
- 不實作 `analyze` / `validate` 指令 — 跨 artifact 一致性檢查與 schema 驗證留到 HTTP provider 完成後再做
- 不實作 `discuss start` / `discuss capture` 指令 — discuss 階段主要在 AI 端，CLI 暫不介入
- 不變更 `Provider` trait 既有 method 的簽章（只新增 `get_status` 與 `ArtifactKind::{Design, Tasks, Spec}` 路徑）
- 不為 spec artifact 引入 Delta heading 解析（`## ADDED / MODIFIED / REMOVED`）— 寫入時當作 plain markdown，delta 解析延後到 archive change
- 不支援 `artifact write proposal` — proposal 由 `propose create` 寫入，artifact write 不重複建構同條路徑
- 不引入新 crate；既有 4 crate 已足以容納本 change

## Capabilities

### New Capabilities

- `cli-artifact-write`: `speclink artifact write` 指令的 clap 介面、`<kind>` 子命令、必要與選用旗標（含 `--capability` 用於 spec kind）、stdin 行為、JSON output schema、成功與失敗的 exit code 與 error code
- `cli-status`: `speclink status` 指令的 clap 介面、必要與選用旗標、JSON output schema 中 `ChangeStatus` 的欄位（artifacts array、artifact 的 id/path/status/required/dependencies）、成功與失敗的 exit code 與 error code

### Modified Capabilities

- `local-provider-storage`: 既有 spec 只描述 proposal 的儲存；本 change 補上 design.md、tasks.md、specs/<capability>/spec.md 的目錄結構、寫入順序、原子性保證，以及 metadata.json 中對應的 lifecycle 欄位更新

## Impact

- Affected specs:
  - New: `openspec/specs/cli-artifact-write/spec.md`、`openspec/specs/cli-status/spec.md`
  - Modified: `openspec/specs/local-provider-storage/spec.md`（補多 artifact 寫入語意）
- Affected crates:
  - Modified: `crates/cli/`、`crates/runtime/`、`crates/provider/`、`crates/provider-local/`
  - 不動: `crates/cli/src/commands/propose.rs`（保留 bootstrap 版）
- Affected code:
  - New:
    - crates/cli/src/commands/artifact.rs
    - crates/cli/src/commands/status.rs
    - crates/cli/tests/artifact_write.rs
    - crates/cli/tests/status.rs
    - crates/cli/tests/artifact_write_snapshots.rs
    - crates/cli/tests/status_snapshots.rs
    - crates/runtime/src/artifact.rs
    - crates/runtime/src/status.rs
    - crates/provider-local/tests/multi_artifact_integration.rs
    - openspec/changes/add-artifact-write-and-status/specs/cli-artifact-write/spec.md
    - openspec/changes/add-artifact-write-and-status/specs/cli-status/spec.md
    - openspec/changes/add-artifact-write-and-status/specs/local-provider-storage/spec.md
  - Modified:
    - crates/cli/src/cli.rs（新增 Artifact、Status subcommand）
    - crates/cli/src/commands/mod.rs
    - crates/cli/src/output.rs（新增 ArtifactWriteData、StatusData）
    - crates/cli/src/main.rs（dispatch 新 subcommand）
    - crates/runtime/src/lib.rs（exports）
    - crates/provider/src/lib.rs（新增 get_status method 與 ChangeStatus type）
    - crates/provider/src/model.rs（新增 ChangeStatus、ArtifactStatus 型別）
    - crates/provider-local/src/lib.rs（實作 get_status、擴張 write_artifact）
    - crates/provider-local/src/storage.rs（新增 write_design_atomic / write_tasks_atomic / write_spec_atomic 或統一函式 + ArtifactKind 路由）
  - Removed: 無
- Affected crate dependencies（保證無循環，無變更）:
  - cli → runtime、provider、provider-local（既有）
  - runtime → provider（既有）
  - provider-local → provider（既有）
- 跨 crate 變更必要性論證：trait 加 method（provider crate）與實作（provider-local crate）必須同時改；CLI 新增 subcommand 必經 runtime 編排。三層改動皆為單一 vertical slice 的必要切面，無重構順帶範圍。
