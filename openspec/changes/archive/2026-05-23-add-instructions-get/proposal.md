## Why

Phase 1 dogfooding 閉環需要把 skill workflow.md 引用的 instruction 內容變成可程式化查詢的 single source（見 `doc/speclink-design.md` §18.4 P1-3 與 §11.7 Rules 注入機制）。目前每個 SDD artifact kind（proposal / spec / design / tasks）的產生指引、artifact template、context、rules、locale、dependencies 散落於 `doc/protocol/operations.md` 與設計文件章節之間，AI skill 沒有統一的 runtime 入口取得「該寫什麼、寫成什麼結構、依賴哪些前置 artifact」。沒有 `instructions.get` op，P1-4 skill deploy 後 skill body 將無 prompt source 可引用、無法啟動 dogfooding。

## What Changes

- 新增 catalogue op `instructions.get`（meta category、read-only、no lock；catalogue 第 32 條，§21.5 已預留位置）。
- 新增 CLI 子指令 `speclink instructions <kind> [--change <id>] [--json]`，屬於 AI skill 可呼叫範圍（透過 Bash binding spawn）。
- 新增 runtime 模組 `crates/runtime/src/instructions_ops.rs`，內含 kind 派發、template / instruction 載入、dependency 推導、config 欄位 hydrate 邏輯。
- 新增 embedded `spec-driven` schema bundle 於 `crates/runtime/src/embedded/schemas/spec-driven/`（schema.yaml + templates/{proposal,spec,design,tasks}.md + instructions/{proposal,spec,design,tasks,apply,ingest,archive,commit}.md），全部透過 `include_str!` 編進 binary。
- 支援 8 個 kind：`proposal` / `spec` / `design` / `tasks`（artifact kinds，回傳 template + output_path）+ `apply` / `ingest` / `archive` / `commit`（workflow phase kinds，template / output_path 為 null）。
- `context` / `rules.<kind>[]` / `locale` 三欄從 A5 `ConfigStore::read` 取得；欄位不存在時 fallback null（對齊 §11.6 malformed = warning + fallback 原則）。
- `dependencies[]` 從 §7 artifact DAG 硬表推導（如 spec→[proposal]、tasks→[proposal,spec,design]）；`capability` 一律為 null。
- 把 catalogue entry 32（`instructions.get`）的 `inputs_schema` / `outputs_schema` 兩個 stub（目前皆為 `empty_object_schema()`）替換成真實 5-field input + 11-field output schema，與 operations.md §`instructions.get` 對齊。

## Non-Goals

- **不**實作 `kind = discuss`、`role` 參數、`discussion_id` 參數、`available_roles[]` 與 `linked_changes_context[]`。傳 `kind=discuss` 一律回 `instructions.unknown_kind`；傳 `role` / `discussion_id` 一律忽略（不報錯也不作用）。Discuss 後端於 Phase 2 `add-discuss-ops` 補。
- **不**引入 user-overridable schema fork：不建立 `.speclink/schemas/` 目錄、不暴露 `schema.{list,show,fork,delete}` op、不做 schema override resolution chain。Schema fork 於 Phase 2 `add-schema-ops` 補。
- **不**做 multi-instance spec capability 動態解析：`dependencies[].capability` 永遠為 null。Capability 視角於 Phase 2 `add-spec-canonical-read` 補。
- **不**擴 A5 `ConfigStore` trait method 簽名（`read_config` / `write_config` / `read_defaults` / `take_warnings` 四個方法不變）。本 slice 對 A5 surface 唯一變動：`provider::config_store::Config` struct 新增三個 Optional 欄位（`context: Option<String>`、`locale: Option<String>`、`instructions: HashMap<String, Vec<String>>`），全部 `#[serde(default)]`、純 additive、A5 既有 round-trip 行為不變。設計文件 §11.7 寫的 `rules.<kind>[]` 路徑因與 A5 strict `Rules` struct（含 `require_*_review` boolean）schema 衝突，改用 `instructions.<kind>[]` 頂層 key。
- **不**改 catalogue 第 32 條的 metadata（id / category / cli / tool_binding / sdk_method / http_endpoint / mvp / destructive / idempotency / lock / phases / curated / description 全部不動）；只替換 inputs_schema / outputs_schema 兩個 stub 函式內容，函式簽名不變。P1-1 既有 `catalogue_doc_sync` test 仍守 schema 與 operations.md 對齊。

## Capabilities

### New Capabilities

- `instructions-resolver`: SpecLink runtime 的 instruction 解析能力 — 把 active schema 的 template / instruction body / artifact DAG dependency、加上 config.yaml 的 context / rules / locale，組合成 stable 11-field JSON envelope 供 skill 引用。

### Modified Capabilities

(none — catalogue 第 32 條於 P1-1 已預留 entry，本 slice 接上 dispatch 不修改 spec 行為)

## Impact

- Affected specs:
  - New: `openspec/specs/instructions-resolver/spec.md`
- Affected code:
  - New:
    - crates/runtime/src/instructions_ops.rs
    - crates/runtime/src/embedded/mod.rs
    - crates/runtime/src/embedded/schemas/spec-driven/schema.yaml
    - crates/runtime/src/embedded/schemas/spec-driven/templates/proposal.md
    - crates/runtime/src/embedded/schemas/spec-driven/templates/spec.md
    - crates/runtime/src/embedded/schemas/spec-driven/templates/design.md
    - crates/runtime/src/embedded/schemas/spec-driven/templates/tasks.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/proposal.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/spec.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/design.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/tasks.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/apply.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/ingest.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/archive.md
    - crates/runtime/src/embedded/schemas/spec-driven/instructions/commit.md
    - crates/cli/src/commands/instructions.rs
    - crates/runtime/tests/instructions_ops.rs
    - crates/cli/tests/instructions_cli.rs
  - Modified:
    - crates/runtime/src/lib.rs（暴露 `instructions_ops` + `embedded` 模組）
    - crates/runtime/src/catalogue/schemas.rs（替換 `instructions_get()` / `instructions_get_outputs()` 兩個 stub 為真實 schema，函式簽名不動）
    - crates/provider/src/config_store.rs（`Config` struct 純 additive 加 `context` / `locale` / `instructions` 三個 Optional 欄位，trait method 簽名不動）
    - crates/cli/src/commands/mod.rs（註冊 `instructions` subcommand）
    - crates/cli/src/main.rs（clap surface）
  - Removed: (none)

- Affected user-facing surface:
  - **AI skill 可呼叫範圍**（Bash binding）：`speclink instructions <kind> --change <id> --json`
  - **JSON output schema**：11 個 field — `kind` / `schema_id` / `instruction` / `template` / `context` / `rules` / `dependencies` / `output_path` / `locale` / `available_roles`（恆為 null）/ `linked_changes_context`（恆為 null）
  - **Exit code**：0 成功；2 使用者輸入錯誤（unknown kind / change not found）
  - **Error codes**：`instructions.unknown_kind`（含 `kind=discuss`）、`change.not_found`（提供 `--change` 但不存在）
  - **無**新增 lifecycle 狀態、**無**新增 lock、**無**新增 audit event、**無**異動 Provider API contract、**無**異動 auth / token / secret 處理路徑
