## Why

Change 2 與 Change 3 完成後，SpecLink 已能在 local provider 上跑完 propose → artifact write → status → archive 一輪 SDD 流程；但 AI skill 在「實作 change」階段仍依賴 Spectra workflow — 缺兩塊：(1) `speclink instructions <artifact>` 讓 skill 知道每個 artifact 該怎麼寫（template、規則、context）、(2) `speclink task done <task-id>` 讓 apply 階段把 tasks.md 的 checkbox 從 `- [ ]` 翻到 `- [x]`。補完這兩條指令後，**SpecLink 的 AI workflow 指令集對 local-only 流程已自足**，可正式取代 Spectra 作為 SpecLink 自身的 SDD 工具。

## What Changes

- 在 `crates/cli/` 新增 AI workflow 指令 `speclink instructions <artifact> --change <id> --json`：`<artifact>` 可為 `proposal` / `design` / `tasks` / `spec`（spec 需附帶 `--capability`）；輸出 instruction 文字、template、rules、locale、依賴 artifact 清單、output path
- 在 `crates/cli/` 新增 AI workflow 指令 `speclink task done <task-id> --change <id> --json`：把 tasks.md 中對應 task 的 checkbox `- [ ]` 改為 `- [x]`；已完成則 idempotent 回 success
- 在 `crates/runtime/` 新增 hardcoded instructions 內容：proposal / design / tasks / spec 四種 kind 各一份 markdown，透過 `include_str!` 編入 binary（本 change 不引入外部 instructions 設定檔）
- 在 `crates/runtime/` 新增 `tasks_parser` module：解析 tasks.md 的 `## N. <heading>` section、`- [ ] N.M <description>` / `- [x] N.M <description>` checkbox，回 typed structure 並支援 update
- 在 `crates/provider/` 的 `Provider` trait 新增 `get_artifact_instructions(project_id, change_id, artifact_kind, capability: Option<&str>) -> Result<ArtifactInstructions, ProviderError>` 與 `mark_task_done(project_id, change_id, task_id) -> Result<TaskUpdate, ProviderError>`
- 在 `crates/provider-local/` 實作上述兩個 method：instructions 從 runtime 取 hardcoded 內容（local provider 不從外部讀取）；mark_task_done 原子更新 tasks.md
- 新增 capability spec：`cli-instructions`、`cli-task-done`；修改 `local-provider-storage`（tasks.md 原子更新規範）

**目標使用者情境**：
- **AI skill 呼叫階段**：`/spectra-propose` 等價 skill 改呼叫 `speclink instructions proposal --change <id>` 取得 template → AI 根據 instruction 產生內容 → 呼叫 `speclink artifact write proposal --stdin`；apply 階段每完成一個 task 就 `speclink task done <task-id>`
- **工程師本機階段**：可直接 `speclink instructions design --change <id>` 看 design 該寫什麼 / `speclink task done 5.2` 標記任務完成
- **CI 階段**：暫不涉及

## Non-Goals

- 不引入外部 instructions 設定檔（YAML / TOML） — 內容硬編於 runtime crate，待第一輪 dogfood 後再評估是否抽出
- 不引入 schema 概念（如 Spectra 的 `spec-driven` schema 名稱） — 第一版 instructions 對應 4 種 ArtifactKind 即可，schema 抽象延後
- 不在 instructions JSON 中回傳 `applyRequires` / `dependencies` 計算結果（Ready / Blocked dependency-aware 狀態） — 屬於 status 領域，本 change 不重複
- 不支援 `task done --undone` 反向取消 task — 視為非 AI workflow 場景
- 不解析 task 之間的相依關係（如 spectra 的 `[P]` 標記）— 解析能力可提供未來 `apply` 指令使用，本 change 不引入
- 不引入 `analyze` / `validate` 指令 — 留待 HTTP provider 完成後再做
- 不引入 `apply start` 指令 — apply 由 AI skill 自行迴圈呼叫 `task done`，無需 CLI 端 apply state machine
- 不更新 metadata.json 的 lifecycle state（task done 不改變 state，仍為 `proposed`）
- 不為 task 加 timestamp 或 actor metadata — tasks.md 只記載 checkbox 狀態，誰做的、何時做留給 git history
- 不變更 propose create / artifact write / status / archive 既有指令的行為
- 不引入新 crate

## Capabilities

### New Capabilities

- `cli-instructions`: `speclink instructions` 指令的 clap 介面、`<artifact>` 子命令、`--capability` 旗標（用於 spec kind）、JSON output schema（含 `outputPath` / `dependencies` / `unlocks` / `instruction` / `template` / `rules` / `locale`）、成功與失敗的 exit code 與 error code
- `cli-task-done`: `speclink task done` 指令的 clap 介面、task id 格式、JSON output schema（含被更新前後的 status）、idempotent 行為、成功與失敗的 exit code 與 error code

### Modified Capabilities

- `local-provider-storage`: 既有規格涵蓋 atomic artifact write；本 change 補上「task done 對 tasks.md 的原子更新」— 寫入順序 + 失敗 rollback；以及 tasks.md 的 task id 解析規則（`N.M` 形式對應 `## N.` section 與 `- [ ] N.M` checkbox）

## Impact

- Affected specs:
  - New: `openspec/specs/cli-instructions/spec.md`、`openspec/specs/cli-task-done/spec.md`
  - Modified: `openspec/specs/local-provider-storage/spec.md`（補 tasks.md 原子更新與 task id 解析）
- Affected crates:
  - Modified: `crates/cli/`、`crates/runtime/`、`crates/provider/`、`crates/provider-local/`
- Affected code:
  - New:
    - crates/cli/src/commands/instructions.rs
    - crates/cli/src/commands/task.rs
    - crates/cli/tests/instructions.rs
    - crates/cli/tests/task_done.rs
    - crates/cli/tests/instructions_snapshots.rs
    - crates/cli/tests/task_done_snapshots.rs
    - crates/runtime/src/instructions.rs
    - crates/runtime/src/tasks_parser.rs
    - crates/runtime/instructions/proposal.md
    - crates/runtime/instructions/design.md
    - crates/runtime/instructions/tasks.md
    - crates/runtime/instructions/spec.md
    - crates/provider-local/tests/instructions_integration.rs
    - crates/provider-local/tests/task_done_integration.rs
    - openspec/changes/add-instructions-and-task-done/specs/cli-instructions/spec.md
    - openspec/changes/add-instructions-and-task-done/specs/cli-task-done/spec.md
    - openspec/changes/add-instructions-and-task-done/specs/local-provider-storage/spec.md
  - Modified:
    - crates/cli/src/cli.rs（新增 Instructions、Task subcommand）
    - crates/cli/src/commands/mod.rs
    - crates/cli/src/main.rs
    - crates/cli/src/output.rs（新增 InstructionsData、TaskDoneData）
    - crates/cli/src/exit_code.rs（新增 task / instructions 相關 error code mapping）
    - crates/runtime/src/lib.rs（exports）
    - crates/provider/src/lib.rs（新增 get_artifact_instructions、mark_task_done trait methods + 對應型別）
    - crates/provider/src/model.rs（新增 ArtifactInstructions、TaskUpdate、TaskStatus）
    - crates/provider/src/error.rs（新增 task / instructions 相關 ProviderError variant）
    - crates/provider-local/src/lib.rs（實作兩個新 method）
    - crates/provider-local/src/storage.rs（新增 update_tasks_atomic helper）
    - crates/provider-local/src/error.rs
  - Removed: 無
- Affected crate dependencies（無變更，無循環）:
  - cli → runtime、provider、provider-local（既有）
  - runtime → provider（既有；本 change 不引入新外部 crate）
  - provider-local → provider、runtime（Change 3 已建立）
- 跨 crate 變更必要性論證：trait 加 method（provider）與實作（provider-local）必須同時改；instructions 內容放 runtime（與 spec_delta 同樣是 algorithm/data 層）；CLI 兩條子命令經 runtime 編排。三層改動為單一 vertical slice 切面。
