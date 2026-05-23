## Context

`doc/protocol/operations.md` 與 `doc/speclink-design.md` §21.5 已手寫 37 個 operation 的 catalogue metadata（canonical id、category、CLI binding、tool binding、SDK method、HTTP endpoint、MVP flag、destructive flag、idempotency、lock requirement、inputs JSON schema）。目前該 catalogue 純為文件形式，無法從 Rust 程式碼查詢。後續 Phase 1 #3 `instructions.get` 與 #4 skill 部署都需要程式化查 canonical op id 與 inputs schema；本 change 把 catalogue 固化為 Rust single source of truth，並透過 `speclink describe-tools` CLI 對外輸出多格式。

設計依據：
- `doc/speclink-design.md` §16.2（`describe-tools` CLI 規範）
- `doc/speclink-design.md` §21（Operation Catalogue 角色與映射規則）
- `doc/speclink-design.md` §22.2（Layer 1 curated 12 ops）
- `doc/protocol/operations.md` `tool.describe` 完整 op spec

## Goals / Non-Goals

**Goals:**

- 在 `crates/runtime/` 內提供 catalogue source registry — 37 個 op 的 metadata 可從 Rust 程式碼查詢
- 提供 `speclink describe-tools` CLI，支援 3 種 format：`json`（machine 預設）、`text`（human markdown）、`copilot-sdk`（CopilotKit SDK descriptor shape）
- 提供 `--filter` / `--categories` / `--phases` / `--full` 篩選 flag
- 預設輸出 Layer 1 curated 12 ops subset（design.md §22.2），帶 `--full` 才輸出 37 ops 全集
- CI snapshot test 守住 catalogue 與 `doc/protocol/operations.md` 的一致性

**Non-Goals:**

- 不實作 `copilotkit` / `openai` / `langchain` / `mcp` / `claude` format — 標 [deferred]，回 `tool.format_not_supported`
- 不實作 HTTP `GET /api/projects/{id}/tool-catalogue` endpoint — 依賴 HttpProvider，屬 [deferred]
- 不實作 catalogue codegen pipeline — MVP 手動 mirror（design.md §21.1）
- 不實作 catalogue editing CLI — catalogue 是 Rust const，編譯期固定
- 不實作 `instructions.get`（Phase 1 #3）— 本 slice 只提供 catalogue source 給後續 slice 用
- 不實作 skill 部署的 binding 生成（Phase 1 #4）— 同上
- 不修改既有 op 行為 — 純新增

## Decisions

### Catalogue 表示方式採 Rust const struct，非 YAML / JSON 檔

採 `&'static [Operation]` const slice，每個 `Operation` 包含 id、category、cli、tool、sdk、http、mvp、destructive、idempotency、lock、phases、inputs schema、curated flag 等欄位。

**替代方案：**
- **獨立 `catalogue.yaml`**：runtime parse YAML 載入。優點是 non-Rust 工具也可讀；缺點是 missing-file / parse-error runtime 風險、無編譯期欄位檢查、增加 binary 與 runtime overhead。
- **`build.rs` codegen from `operations.md`**：parse markdown 表格生成 Rust。優點是 doc 即 source；缺點是 markdown parser 脆弱、build 階段失敗難 debug、early MVP 投資 too high。

**理由**：const struct 編譯期欄位齊全檢查（漏一個 op 立即 type error）、零 runtime 開銷、無 file dependency；codegen 延後到 MVP 後（design.md §21.1）；CI snapshot test 守 doc 同步。

**Rust 慣用法諮詢（task 1.1 / 1.4）**：
- `rust-skills:m05-type-driven` — 用 `&'static [Operation]` 與 `fn() -> serde_json::Value` 函式指標保留 zero-cost 抽象；type-state / PhantomData 對 catalogue 過度設計，本 slice 不採用。
- `rust-skills:domain-cli` — clap `ValueEnum` 接 `--format` 八個 literal（3 supported + 5 deferred）；`value_delimiter = ','` 解析 comma-separated `--filter` / `--categories` / `--phases`。
- `rust-skills:m15-anti-pattern` + `rust-skills:coding-guidelines`（task 1.4 / 9.4 重構）— enum variant 用 `PascalCase`、struct 欄位 `snake_case`；避免 `pub` over-export；`#[non_exhaustive]` 標記 `DescribeFormat` 以容後續 format 加入而不破 API。

### Catalogue 與 doc 同步靠 CI snapshot test + manual mirror

`crates/runtime/tests/catalogue_doc_sync.rs` 比對：
1. `Catalogue::all().len() == 37`（透過解析 `doc/protocol/operations.md` 的 Index 表行數驗證）
2. 每個 op 的 `id` / `category` / `cli` / `tool_binding` / `mvp` / `destructive` 欄位與 Index 表一致

**替代方案：**
- **`build.rs` 自動同步**：build 時 panic if drift。優點是強制同步；缺點是 build 失敗訊息隱晦、CI 已經能 fail。
- **只靠人工 review**：不寫 test。優點是零成本；缺點是 drift 不可避免、後續 Phase 1 #3 / #4 一旦讀到 stale 資料 debug 成本高。

**理由**：snapshot test 是最低成本的 drift detector，本身是 codegen pipeline 的 stepping stone，未來把 test 改成 generator 即可。

### MVP format 支援範圍限 3 種

`json` / `text` / `copilot-sdk` — 對應 `doc/protocol/operations.md` `tool.describe` `format` enum 與 §22.2 已 ship 的 Layer 1 SDK adapter。其他 5 種（`copilotkit` / `openai` / `langchain` / `mcp` / `claude`）寫成 enum variant 但 handler return `tool.format_not_supported`。

**替代方案：**
- **MVP 一次寫齊 8 種 format**：優點是 §16.2 文件範例直接可跑；缺點是 5 個 SDK 規格各自不同、實作量 4x、本 slice scope creep。
- **MVP 只支援 `json`**：優點是最小切片；缺點是 `text` 是 human debug 必需（design.md §16.2）、`copilot-sdk` 是 Phase 1 #4 skill 部署前置（要先驗證 SDK shape 對得上 catalogue 才能 deploy）。

**理由**：3 種 format 覆蓋三類 consumer（machine / human / SDK），其他 format 等真實 adoption signal 再加（§18.2 deferred）。

### 預設輸出 Layer 1 curated 12 ops subset，`--full` 切全集

`Operation` 加 `curated: bool` 欄位；預設 `describe-tools` 輸出 `Catalogue::all().filter(|op| op.curated).collect()`；帶 `--full` 才輸出全集。Curated 12 ops 定義在 catalogue source（design.md §22.2 列出的清單）。

**替代方案：**
- **預設輸出 37 ops 全集**：優點是「給就給全」直觀；缺點是 SDK consumer 預設拿到 37 個 tool descriptor 會觸發 context bloat、AI tool selection accuracy 過 20 就掉（design.md §22.2 引用）。
- **`curated` 由 CLI flag 決定不寫進 catalogue**：優點是 catalogue 純 metadata；缺點是 §22.2 已明文「curated subset 是 catalogue 一部分」，多處 surface（SDK / CLI / 文件）需重複定義。

**理由**：design.md §22.2 已決策、catalogue 是 single source；curated 標記寫進 catalogue 一處同步全 surface。

### Filter 邏輯採 AND（intersection）

`--filter <ops>` / `--categories <c>` / `--phases <p>` / `--full` 同時生效時取交集：先 `--full` or `curated` 取初始集合，再依序套 `filter` / `categories` / `phases` 過濾。

**替代方案：**
- **OR（union）**：優點是可加 ops；缺點是與 `--categories` 同用語意混亂（「discuss category 但不要 discuss.delete」無法表達）。

**理由**：AND 是 CLI filter 的標準語意（`grep` / `find` / `kubectl get -l` 皆然）；用 multiple `--filter` 達成 OR 即可（clap `Vec<String>`）。

### CLI 採 `describe-tools`（hyphenated single command），不拆 subcommand

對齊 design.md §16.2、§21.5 既有命名，clap `Commands::DescribeTools { ... }`。

**替代方案：**
- **`describe tools <op>` subcommand**：優點是可擴成 `describe schemas` / `describe ops` 等；缺點是 `describe-tools` 已寫進 §16.2，change 命名要回頭改 doc；且 describe 動詞與 §16.0 `show <noun>` query verb 衝突。

**理由**：對齊既有 doc，不引入 describe 動詞家族（query verb 由 `show` / `list` 主導，design.md §16.0）。

### JSON inputs schema 採 embedded `serde_json::json!` literal

每個 op 的 `inputs_schema: serde_json::Value` 用 `json!({...})` macro 在 catalogue source 內 inline 定義。

**替代方案：**
- **獨立 schema 檔（`schemas/<op>.json`）由 build.rs `include_str!`**：優點是 schema 可被外部工具直接讀；缺點是 file scattered、編譯期不檢查 JSON 合法性、本 slice 無 external consumer。
- **`jsonschema::JSONSchema` 型別 const**：優點是強型別；缺點是 `jsonschema` crate 的 type 非 const-constructible，需 runtime 初始化。

**理由**：`json!` macro 編譯期驗證 JSON 文法、與 catalogue metadata 共處一檔便於 review；未來真要外部 consumer 取 schema，加一個 `describe-tools --format json --filter <op> --json-schema-only` 的 sub-mode 即可。

### `tool.describe` op handler 放 `runtime`，不放 `cli`

`crates/runtime/src/tool_ops.rs` 提供 `describe_tools(req: DescribeToolsRequest) -> Result<DescribeToolsResponse, ToolError>`；CLI subcommand 純做 clap → request 與 response → envelope 轉換。

**替代方案：**
- **handler 直接在 `cli/commands/describe_tools.rs`**：優點是少一個 module；缺點是後續 SDK / HTTP transport 接同一 op 時 logic 重複；違反 design.md §15 crate 拆分原則（CLI 純命令面、runtime 純 workflow）。

**理由**：對齊既有 ops 拆分（`change_ops` / `artifact_ops` / `config_ops` / `archive_ops` 皆在 runtime），維持 CLI thin layer。

## Implementation Contract

### Behavior

執行 `speclink describe-tools` 後，stdout 印 catalogue 子集（依 flag），exit 0；無副作用、不寫檔、不修改 state.db、不取 lock。

### Interface

**CLI clap subcommand**（新增 `Commands` variant）：

```
DescribeTools {
  format: DescribeFormat,           // --format <json|text|copilot-sdk>，預設 json
  filter: Vec<String>,              // --filter <op-id>,...（catalogue id，如 change.create）
  categories: Vec<String>,          // --categories <cat>,...
  phases: Vec<DescribePhase>,       // --phases <discuss|propose|apply|archive|ingest>,...
  full: bool,                       // --full（停用 curated subset）
}
```

`DescribeFormat` enum：`Json` / `Text` / `CopilotSdk` 為合法值；`Copilotkit` / `Openai` / `Langchain` / `Mcp` / `Claude` 為 enum variant 但 runtime 回 `tool.format_not_supported`。

**Runtime API**：

```
pub struct DescribeToolsRequest {
    pub format: DescribeFormat,
    pub filter: Vec<String>,
    pub categories: Vec<String>,
    pub phases: Vec<DescribePhase>,
    pub full: bool,
}

pub struct DescribeToolsResponse {
    pub format: DescribeFormat,
    pub content: DescribeContent,   // Json(Vec<ToolDescriptor>) | Text(String) | CopilotSdk(Vec<CopilotSdkDescriptor>)
}

pub fn describe_tools(req: DescribeToolsRequest) -> Result<DescribeToolsResponse, ToolOpsError>;
```

**Catalogue 查詢 API**：

```
pub struct Operation {
    pub id: &'static str,                      // 如 "change.create"
    pub category: &'static str,                // 如 "change"
    pub cli: &'static str,                     // 如 "new change <name>"
    pub tool_binding: &'static str,            // 如 "new_change"
    pub sdk_method: &'static str,              // 如 "speclink.changes.create"
    pub http_endpoint: &'static str,           // 如 "POST /api/projects/{id}/changes"
    pub mvp: bool,
    pub destructive: bool,
    pub idempotency: Idempotency,              // enum: NonIdempotent / Idempotent / IdempotentWithVersion
    pub lock: LockRequirement,                 // enum: None / GlobalShort / GlobalExclusive / ChangeExclusive / DiscussExclusive
    pub phases: &'static [Phase],              // enum 集合
    pub curated: bool,                         // §22.2 Layer 1
    pub description: &'static str,             // 一句話
    pub inputs_schema: fn() -> serde_json::Value,   // 回 JSON Schema（用 fn() 因 Value 非 const）
}

pub struct Catalogue;
impl Catalogue {
    pub fn all() -> &'static [Operation];
    pub fn get(id: &str) -> Option<&'static Operation>;
}
```

### JSON envelope shape

成功（exit 0）：

```json
{
  "ok": true,
  "data": {
    "format": "json",
    "content": [
      {
        "id": "change.create",
        "name": "new_change",
        "description": "...",
        "parameters": { "$schema": "...", "type": "object", ... }
      }
    ]
  }
}
```

`text` format 的 `content` 為 string（markdown）；`copilot-sdk` 為 array of `{ name, description, parameters }` 對應 `defineTool` shape（design.md §22.2 範例）。

### Failure modes

| Error code | 觸發條件 | Exit |
|---|---|---|
| `tool.format_not_supported` | `--format` 值為 `copilotkit` / `openai` / `langchain` / `mcp` / `claude` 之一 | 2 |
| `tool.unknown_op` | `--filter` 含未在 catalogue 的 op id | 2 |
| `tool.unknown_category` | `--categories` 含未在 catalogue 的 category | 2 |

Error envelope 結構對齊既有 ops（`{ ok: false, error: { code, message, hint? } }`）。

### Acceptance criteria

1. **Catalogue 完整性**：`crates/runtime/tests/catalogue_doc_sync.rs` integration test 解析 `doc/protocol/operations.md` Index 表，驗證 `Catalogue::all().len() == 37` 且每行 op id / category / cli / tool / mvp / destructive 與 const 一致
2. **Curated 12 ops**：unit test 驗證 `Catalogue::all().iter().filter(|op| op.curated).count() == 12`，且 12 個 id 集合等於 design.md §22.2 列出的 set
3. **Format 渲染**：`crates/cli/src/snapshots/` 加 3 個 `insta` snapshot — `describe_tools_default_json.snap` / `describe_tools_full_text.snap` / `describe_tools_curated_copilot_sdk.snap`
4. **Filter AND 邏輯**：unit test 覆蓋「`--categories change` + `--filter change.delete` 結果只含 change.delete」、「空交集回 `data.content: []`」
5. **Error**：`assert_cmd` 整合測試 — `describe-tools --format mcp` exit 2 + envelope `error.code = "tool.format_not_supported"`；`--filter no.such.op` exit 2 + `tool.unknown_op`
6. **JSON envelope shape stability**：`describe_tools_envelope.snap` snapshot 守 envelope 結構

### Scope boundaries

**In scope**：
- 37 ops mirror 到 `crates/runtime/src/catalogue.rs`（含 inputs schema、curated flag）
- `tool.describe` op handler（`crates/runtime/src/tool_ops.rs`）
- `speclink describe-tools` CLI subcommand（`crates/cli/src/commands/describe_tools.rs`）
- 3 種 format renderer（`json` / `text` / `copilot-sdk`）
- 5 種 format 拒絕路徑（`tool.format_not_supported`）
- CI snapshot test 守 catalogue ↔ doc 同步

**Out of scope**：
- `instructions.get`（Phase 1 #3）
- Skill 部署（Phase 1 #4）
- 其他 5 種 format 實作
- HTTP `GET /api/projects/{id}/tool-catalogue` endpoint
- Catalogue codegen pipeline
- 修改 `doc/protocol/operations.md` 內容（純鏡像，不改 source）
- 修改既有 op 行為

## Risks / Trade-offs

- **[Catalogue 與 doc drift]** → Mitigation：CI snapshot test (`catalogue_doc_sync.rs`) 在每次 PR 跑；任一欄位不一致 fail
- **[12 curated 集合主觀]** → Mitigation：curated 標記寫死在 catalogue source，依 design.md §22.2 既有清單；Phase 2/3 真實 dogfooding 過程踩到再 adjust（透過後續 change）
- **[`serde_json::json!` macro 在 37 inputs schema 文件量大]** → Mitigation：每個 op 的 schema 抽成獨立 `fn op_<id>_schema() -> Value` 函式、catalogue 引用函式指標；單一檔內仍可單獨檢視
- **[`--filter` 用 op id 含 `.`，shell 不需 quote 但 IDE auto-complete 可能誤判]** → Mitigation：CLI help text 明示範例 `--filter change.create,change.delete`；snapshot test 覆蓋
- **[unsupported format 仍佔 `DescribeFormat` enum variant 顯得髒]** → Mitigation：用 `#[doc(hidden)]` 或註解標 `[deferred]`；好處是 clap auto-complete 仍提示完整集合、消費者一眼看出 future support 計畫
- **[catalogue 是 const，新增 op 要改 Rust 重 compile]** → Accepted trade-off：MVP 階段 catalogue 是 source of truth，新增 op = 新增 SDD slice = 必然有 code change

## Migration Plan

純新增 capability、無 breaking change、無 data migration、無 state.db schema bump。直接 ship；無 rollback 需求（rollback = revert 該 commit）。

## Open Questions

無 — scope 與接口已收斂。
