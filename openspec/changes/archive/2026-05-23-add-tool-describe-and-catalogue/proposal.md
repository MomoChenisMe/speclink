## Why

Phase 1 dogfooding 路徑（design.md §18.4）的第一條 slice。`instructions.get`（Phase 1 #3）與 skill 部署（Phase 1 #4）都需要查 canonical operation ID 與其輸入 schema，因此必須先把目前散落在 `doc/protocol/operations.md`（37 ops）與 design.md §21.5 的 catalogue 內容固化為 **可程式查詢的 Rust single source of truth**。同時提供 `speclink describe-tools` CLI 把 catalogue 以多種格式輸出，給 SDK adapter、文件生成、人工 debugging 使用（§16.2、§18.1 #18）。

## What Changes

- 新增 **catalogue source registry**（`crates/runtime/src/catalogue.rs`）：以 Rust const struct 鏡像 `doc/protocol/operations.md` 的 37 個 op metadata（canonical id、category、CLI binding、tool binding、SDK method、HTTP endpoint、MVP flag、destructive flag、idempotency、lock requirement、inputs JSON schema、phases）。提供 `Catalogue::all()` / `Catalogue::get(id)` / `Catalogue::filter(...)` 查詢 API。
- 新增 **`tool.describe` op handler**（`crates/runtime/src/tool_ops.rs`）：依 `format` / `filter` / `categories` / `phases` / `full` 參數從 catalogue 篩選並序列化。MVP 支援 3 種 format：`json`（machine 預設）、`text`（human markdown）、`copilot-sdk`（CopilotKit SDK descriptor shape）。其他 format（`copilotkit` / `openai` / `langchain` / `mcp` / `claude`）拒絕並回 `tool.format_not_supported`。
- 新增 **`speclink describe-tools` CLI subcommand**（`crates/cli/src/commands/describe_tools.rs`）：對應 `tool.describe` op，flag `--format` / `--filter` / `--categories` / `--phases` / `--full`；默認回 12 個 curated subset（design.md §22.2）的 json format。
- 新增 **Layer 1 curated subset 定義**：12 個 5 skill phase 真正用到的 op 標記在 catalogue（design.md §22.2）。
- **不引入新 error code 之外的 lifecycle 變更**：`tool.describe` 是純 read-only meta-op，no lock、idempotent、不寫 audit。

## Non-Goals

留 design.md 寫。

## Capabilities

### New Capabilities

- `tool-catalogue`: Rust catalogue source registry — 37 ops 的 metadata single source of truth，可由 runtime 與 cli 查詢
- `describe-tools-cli`: `speclink describe-tools` CLI subcommand 與其 multi-format renderer

### Modified Capabilities

（none — 純新增）

## Impact

- 影響 crate：`runtime`（新增 catalogue + tool_ops module）、`cli`（新增 describe_tools subcommand）
- 目標使用者情境：
  - **AI skill 呼叫階段**：Phase 1 #3 / #4 用 `Catalogue::all()` 取 canonical op id 集合 + inputs schema，供 instructions.get 與 skill 部署
  - **人類設定階段**：開發者用 `speclink describe-tools --format text` 看 catalogue，或 `--format copilot-sdk` 餵 SDK
  - **CI 階段**：用 `--format json` 驗證 catalogue 完整性
- CLI 屬性：`describe-tools` **不屬於** AI skill 主流程，是給 SDK adapter / 文件 / debugging 用的 meta 指令（catalogue 本身才是被 skill 用的）
- JSON output：成功時 envelope `data = { format, content }`；`content` 依 format 為 array of tool descriptors（`json` / `copilot-sdk`）或 markdown string（`text`）
- Exit code：成功 0；`tool.format_not_supported` → 2；`tool.unknown_op` → 2
- Provider API contract：無變更（純 engine 內 catalogue lookup，無 Provider method）
- Lifecycle：無變更
- Auth/token：無變更
- Fallback：無變更（純 read-only meta-op，無 provider 依賴）
- 影響檔案：
  - New: `crates/runtime/src/catalogue.rs`
  - New: `crates/runtime/src/tool_ops.rs`
  - New: `crates/cli/src/commands/describe_tools.rs`
  - New: `openspec/specs/tool-catalogue/spec.md`
  - New: `openspec/specs/describe-tools-cli/spec.md`
  - Modified: `crates/runtime/src/lib.rs`
  - Modified: `crates/cli/src/commands/mod.rs`
  - Modified: `crates/cli/src/main.rs`
