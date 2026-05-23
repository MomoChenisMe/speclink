<!--
TDD 順序：紅燈（測試）→ 綠燈（最小實作）→ 重構。
每階段結束跑 cargo build / cargo test -p <crate> / cargo fmt --check / cargo clippy -- -D warnings。
Cross-reference 覆蓋 specs/**/spec.md 的 6 個 tool-catalogue Requirement、5 個 describe-tools-cli Requirement，
以及 design.md 的 8 個 Decision 與 Implementation Contract。
-->

## 1. Catalogue source registry skeleton（runtime crate）

- [x] 1.1 觸發 `rust-skills:m05-type-driven` 與 `rust-skills:domain-cli` 取得 catalogue 表示方式的 idiomatic 建議（Decision「Catalogue 表示方式採 Rust const struct，非 YAML / JSON 檔」前置）。**驗證**：在 design.md 對應 Decision 段落加註已諮詢的 skill 與要點摘要（content review）。
- [x] 1.2 在 `crates/runtime/src/catalogue.rs` 撰寫 `Operation` struct 與 `Catalogue` 型別的 **unit test (red)**：`catalogue_all_returns_static_slice`、`catalogue_get_existing_id_returns_some`、`catalogue_get_unknown_id_returns_none`、`catalogue_get_is_case_sensitive`。**驗證**：`cargo test -p speclink-runtime catalogue::` 編譯失敗（type 尚未存在）。
- [x] 1.3 實作 `Operation` struct（含 `id` / `category` / `cli` / `tool_binding` / `sdk_method` / `http_endpoint` / `mvp` / `destructive` / `idempotency` / `lock` / `phases` / `curated` / `description` / `inputs_schema` 全部欄位）+ `Catalogue` 型別 + 空 const slice 讓 1.2 編譯通過（green，Catalogue SHALL provide id-keyed lookup）。**驗證**：`cargo test -p speclink-runtime catalogue::catalogue_get_unknown_id_returns_none` 通過；`catalogue_get_existing_id_returns_some` 仍 fail（slice 空）。
- [x] 1.4 觸發 `rust-skills:m15-anti-pattern` 與 `rust-skills:coding-guidelines` 重構 1.3 的 enum 與 struct 命名。**驗證**：`cargo fmt --check && cargo clippy -p speclink-runtime -- -D warnings` 全綠。

## 2. 37 operation entries 鏡像（Each Operation entry SHALL carry the metadata required by downstream surfaces）

- [x] 2.1 撰寫 unit test (red)：`catalogue_count_is_37`、`catalogue_ids_match_operations_md_set`（id 集合與 spec 列出的 37 個 literal 比對）、`catalogue_destructive_set_is_three`（destructive 集合等於 `{change.delete, discuss.delete, schema.delete}`）、`catalogue_tool_binding_na_only_for_tool_describe`。**驗證**：`cargo test -p speclink-runtime catalogue::catalogue_count_is_37` fail（slice 仍空）。
- [x] 2.2 在 `catalogue.rs` 新增 37 個 `Operation` const entries，逐行對照 `doc/protocol/operations.md` 第 56-94 行的 Index 表填入 `id` / `category` / `cli` / `tool_binding` / `mvp` / `destructive` / `idempotency` / `lock`（green，Catalogue SHALL expose exactly 37 operations as a compile-time const slice）。**驗證**：`cargo test -p speclink-runtime catalogue::catalogue_count_is_37` 通過、其他 2.1 測試全綠。
- [x] 2.3 為 37 個 entry 補 `sdk_method` / `http_endpoint` / `phases` / `description` 欄位，對照 `doc/speclink-design.md` §21.5 與 §4.4 的 phase mapping，使 Each Operation entry SHALL carry the metadata required by downstream surfaces 的契約成立。**驗證**：加 unit test `catalogue_every_op_has_non_empty_basic_fields` 驗證 id/category/cli/sdk_method/description 皆非空；`cargo test -p speclink-runtime catalogue::` 全綠。

## 3. JSON Schema 函式指標（Catalogue SHALL provide JSON Schema for every operation via inputs_schema）

- [x] 3.1 撰寫 unit test (red)：`catalogue_schema_is_object_type`（每 op 跑 `(op.inputs_schema)()` 必為 Object 且 `type == "object"`）、`catalogue_schema_is_deterministic`（同 op 連跑兩次回傳 Value 相等）。**驗證**：`cargo test -p speclink-runtime catalogue::catalogue_schema_is_object_type` 編譯失敗（`inputs_schema` 尚未實作）。
- [x] 3.2 為每個 op 抽出 `fn op_<category>_<verb>_schema() -> serde_json::Value`，使用 `serde_json::json!` macro literal 定義（Decision「JSON inputs schema 採 embedded `serde_json::json!` literal」），讓 Catalogue SHALL provide JSON Schema for every operation via inputs_schema 的契約成立。MVP 為 37 個 op 至少給 stub `{"type":"object","properties":{}}`；對 4 個 slice-A 已實作 ops（`change.create` / `change.list` / `change.show` / `change.delete`）填完整 schema 對齊 operations.md。**驗證**：3.1 全部 test 通過。
- [x] 3.3 重構：抽 `crates/runtime/src/catalogue/schemas.rs` sub-module 收所有 schema 函式，`catalogue.rs` 只引用函式指標。**驗證**：`cargo fmt --check && cargo clippy -p speclink-runtime -- -D warnings` 全綠；`cargo test -p speclink-runtime catalogue::` 仍綠。

## 4. Curated subset 標記（Catalogue SHALL mark exactly 12 operations as curated for Layer 1 SDK subset）

- [x] 4.1 撰寫 unit test (red)：`catalogue_curated_count_is_12`、`catalogue_curated_set_matches_design_22_2`（curated id 集合等於 spec 列出的 12 個 literal）。**驗證**：`cargo test -p speclink-runtime catalogue::catalogue_curated_count_is_12` fail。
- [x] 4.2 把 12 個 curated op 的 `curated` 欄位設 `true`（其餘 25 個保持 `false`），對應 design.md §22.2 與 spec 列表，使 Catalogue SHALL mark exactly 12 operations as curated for Layer 1 SDK subset 的契約成立（Decision「預設輸出 Layer 1 curated 12 ops subset，`--full` 切全集」前置）。**驗證**：4.1 兩 test 全綠。

## 5. CI snapshot test：catalogue ↔ doc 同步（Catalogue SHALL stay in sync with doc/protocol/operations.md via a CI snapshot test、Decision「Catalogue 與 doc 同步靠 CI snapshot test + manual mirror」）

- [x] 5.1 在 `crates/runtime/tests/catalogue_doc_sync.rs` 撰寫 integration test (red)：以 `include_str!("../../../doc/protocol/operations.md")` 載入內容、用 regex 或行 split 解析 Index table（第 56-94 行）、比對 row count 與 `Catalogue::all().len()`，使 Catalogue SHALL stay in sync with doc/protocol/operations.md via a CI snapshot test 的契約成立。**驗證**：`cargo test -p speclink-runtime --test catalogue_doc_sync` 先跑通基礎 count 比對（37 == 37）；若 mismatch 則 panic 與顯示 missing/extra ids。
- [x] 5.2 擴充 5.1 比對每 row 的 6 欄（Catalogue ID / Category / CLI / Tool binding / MVP `✓→true` `[deferred]→false` / Destructive `⚠→true` `—→false`）與對應 `Operation` 欄位。**驗證**：`cargo test -p speclink-runtime --test catalogue_doc_sync` 全綠；手動暫時把 catalogue 內 `change.create` 改成 `change.created` 跑 test 驗證會 fail 並印出 missing/extra id（記錄在 PR 描述、改回正確 id）。

## 6. `tool.describe` op handler（runtime crate；Decision「`tool.describe` op handler 放 `runtime`，不放 `cli`」）

- [x] 6.1 觸發 `rust-skills:m13-domain-error` + `rust-skills:m06-error-handling` 取得 `ToolOpsError` enum idiom；在 `crates/runtime/src/error.rs` 新增 `ToolOpsError::FormatNotSupported { format }` / `UnknownOp { id }` / `UnknownCategory { category }`，並把它們對應到 `RuntimeError` 與 exit code 2 對照表（`output::error_code_to_exit` 兩處同步）。**驗證**：`cargo build -p speclink-runtime && cargo build -p speclink-cli` 編譯通過；新增 unit test `runtime_error_tool_format_not_supported_maps_to_exit_2` 驗證對照表。
- [x] 6.2 在 `crates/runtime/src/tool_ops.rs` 撰寫 unit test (red)：`describe_tools_default_returns_curated_12`、`describe_tools_full_returns_37`、`describe_tools_categories_and_filter_intersection`（covering Filter flags SHALL apply as AND intersection）、`describe_tools_empty_intersection_returns_empty`、`describe_tools_phases_discuss_returns_only_discuss_ids`。**驗證**：`cargo test -p speclink-runtime tool_ops::` 編譯失敗。
- [x] 6.3 實作 `DescribeToolsRequest` / `DescribeToolsResponse` / `DescribeFormat` enum / `DescribePhase` enum / `DescribeContent` enum + `pub fn describe_tools(req) -> Result<_, ToolOpsError>`（green，Decision「Filter 邏輯採 AND（intersection）」— 含 AND 篩選邏輯：先依 `full` flag 選 `all()` or `curated`，再依序過濾 filter→categories→phases）。**驗證**：6.2 全部 test 通過。
- [x] 6.4 撰寫 error path unit test (red)：`describe_tools_unknown_op_returns_unknown_op_error`、`describe_tools_unknown_category_returns_unknown_category_error`、`describe_tools_unsupported_format_returns_format_not_supported`（涵蓋 5 種 deferred format 之一如 `mcp`）。**驗證**：對應 test fail（handler 尚未拒絕）。
- [x] 6.5 在 `describe_tools` 內加 pre-validation：先檢查 format 是否在 `{Json, Text, CopilotSdk}` 三個允許值內、filter ids 是否全在 `Catalogue::get()`、categories 是否全有對應 op，三者任一失敗 early return ToolOpsError（green，Unsupported formats SHALL fail fast with `tool.format_not_supported`、Unknown filter values SHALL be rejected with category-specific error codes）。**驗證**：6.4 全部 test 通過。

## 7. 三種 format renderer（MVP format 支援範圍限 3 種）

- [x] 7.1 在 `crates/runtime/src/tool_ops/render.rs` 撰寫 unit test (red)：`render_json_emits_id_name_description_parameters_keys`（每 element keys 為 `{id, name, description, parameters}`）、`render_text_emits_markdown_table_with_header_and_separator`（首行 starts with `|`、含 header row + separator row）、`render_copilot_sdk_emits_name_description_parameters_only`（每 element 只有 `{name, description, parameters}` 三 key）。**驗證**：`cargo test -p speclink-runtime tool_ops::render::` 編譯失敗。
- [x] 7.2 實作三個 render 函式 `render_json` / `render_text` / `render_copilot_sdk`，由 `describe_tools` 依 format 分派（green，`speclink describe-tools` SHALL emit catalogue subsets in three supported formats）。`render_text` 用 fixed column widths 產 markdown table；`render_copilot_sdk` 移除 `id` 欄位、`parameters` 直接放 schema。**驗證**：7.1 全部 test 通過。
- [x] 7.3 重構：把三個 renderer 抽成獨立檔 `tool_ops/render/json.rs` / `text.rs` / `copilot_sdk.rs`，公開 trait `Render` 統一介面以利後續 5 種 deferred format 接續。**驗證**：`cargo fmt --check && cargo clippy -p speclink-runtime -- -D warnings` 全綠；7.1 與 6.2 全部 test 仍綠。

## 8. CLI subcommand（cli crate；Decision「CLI 採 `describe-tools`（hyphenated single command），不拆 subcommand」）

- [x] 8.1 觸發 `rust-skills:domain-cli` 取得 clap subcommand + multi-value parse idiom 建議。**驗證**：在 tasks 上方註解或 commit message 記錄諮詢摘要（content review）。
- [x] 8.2 在 `crates/cli/src/commands/describe_tools.rs` 撰寫 integration test (red) 使用 `assert_cmd` + `insta`：`describe_tools_default_invocation_emits_curated_12_json`、`describe_tools_full_flag_emits_37_json`、`describe_tools_format_text_emits_markdown_table`、`describe_tools_format_copilot_sdk_emits_define_tool_descriptors`、`describe_tools_categories_change_filter_change_delete_returns_one`、`describe_tools_empty_intersection_returns_empty_array`、`describe_tools_phases_discuss_returns_only_discuss_ids`、`describe_tools_runs_outside_speclink_project`（驗證 `describe-tools` SHALL be read-only and require no project context；tempfile 跑無 `.speclink/` 無 `.git/` 環境）、`describe_tools_filesystem_untouched`（跑前後 dir listing 相等）。**驗證**：`cargo test -p speclink-cli --test describe_tools` 編譯失敗。
- [x] 8.3 新增 `Commands::DescribeTools { format, filter, categories, phases, full }` clap variant 至 `crates/cli/src/main.rs`，registry 至 `crates/cli/src/commands/mod.rs` 並實作 `describe_tools.rs` handler：clap → `DescribeToolsRequest` → 呼叫 `runtime::describe_tools(req)` → `DescribeContent` → envelope（green，CLI 採 `describe-tools` 命名）。clap `--filter` / `--categories` / `--phases` 用 `value_delimiter = ','` 解析 comma list；`--format` 用 `ValueEnum` derive 接受 8 種 literal（含 5 個 deferred 留給 runtime 拒絕）。**驗證**：8.2 全部 test 通過。
- [x] 8.4 撰寫 error path integration test (red)：`describe_tools_format_mcp_exits_2_with_format_not_supported`、`describe_tools_format_banana_rejected_by_clap`、`describe_tools_filter_unknown_op_exits_2_with_tool_unknown_op`、`describe_tools_categories_bogus_exits_2_with_tool_unknown_category`。**驗證**：8.3 完成後測試應已被 6.5 + 8.3 的 error 對映覆蓋；若 fail 則修 envelope error code mapping 直到通過。
- [x] 8.5 撰寫 snapshot test 並存 fixture：`crates/cli/src/snapshots/describe_tools_default_json.snap`、`describe_tools_full_text.snap`、`describe_tools_curated_copilot_sdk.snap`、`describe_tools_envelope.snap`。**驗證**：`cargo insta review` 確認 4 個 snapshot accept；後續 envelope shape 變更可由 PR diff 立即觀察。

## 9. 整合驗證與文件同步

- [x] 9.1 跑 workspace-wide 驗證：`cargo build --workspace && cargo test --workspace && cargo fmt --check && cargo clippy --workspace -- -D warnings`。**驗證**：全綠；若 fail 修到綠為止。
- [x] 9.2 [P] 在 `README.md` 加 `speclink describe-tools` 使用範例段落（json / text / copilot-sdk 三 format、`--full` / `--filter` / `--categories` / `--phases` 四 flag），列出 5 種 deferred format 與 exit code。**驗證**：content review；範例指令在 README 章節可直接 copy 跑通。
- [x] 9.3 [P] 在 `doc/protocol/operations.md` `tool.describe` 章節（line 4144 起）的 `Inputs.format.enum` 由 3 種擴成 8 種 literal、`Errors` 表加 `tool.unknown_category` 一列，使其與 spec / catalogue 對齊。**驗證**：catalogue_doc_sync test 仍綠（修 doc 不影響 Index 表）；`grep -F "tool.unknown_category" doc/protocol/operations.md` 命中至少一處。
- [x] 9.4 觸發 `rust-skills:m15-anti-pattern` 對 `crates/runtime/src/catalogue.rs` / `tool_ops.rs` / `tool_ops/render/` 與 `crates/cli/src/commands/describe_tools.rs` 做最終 anti-pattern review。**驗證**：依建議調整後 `cargo clippy --workspace -- -D warnings` 仍綠、`cargo test --workspace` 仍綠；commit message 列已套用的 idiomatic 調整。
- [x] 9.5 對照 design.md「Implementation Contract」章節，逐項簽收 6 個 sub-heading 的契約：（1）**Behavior**：手動跑 `speclink describe-tools` 無副作用、exit 0；（2）**Interface**：clap `Commands::DescribeTools` variant 與 runtime `DescribeToolsRequest` / `DescribeToolsResponse` 已同型；（3）**JSON envelope shape**：snapshot `describe_tools_envelope.snap` 結構穩定；（4）**Failure modes**：3 個錯誤碼（`tool.format_not_supported` / `tool.unknown_op` / `tool.unknown_category`）皆對映 exit 2；（5）**Acceptance criteria**：spec 列出的 6 條（catalogue 完整性、curated 12、format snapshot、filter AND、error、envelope shape stability）全綠；（6）**Scope boundaries**：本 slice 未碰 Phase 1 #3 `instructions.get` / #4 skill 部署 / 其他 5 種 format / HTTP endpoint / codegen / `operations.md` Index 表內容。**驗證**：在 PR 描述列出 6 項各自的證據（test name 或 commit hash）；任一項未達成回頭補對應 task。
