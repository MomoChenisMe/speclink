## 1. Embedded schema bundle scaffolding

實作 Decision: Embedded schema bundle 用 `include_str!` 編進 binary、不走 filesystem lazy load。

- [x] 1.1 [P] 撰寫 `crates/runtime/src/embedded/schemas/spec-driven/schema.yaml`（schema descriptor）：宣告 8 個 kind 的 DAG，後續 hardcoded dependency table matches schema.yaml DAG test 將以此檔為比對基準。完成判準：`yq '.artifacts[].kind' schema.yaml` 列出 8 個 kind，`.artifacts[].dependencies[]` 對齊設計文件 §7 — 對應 Requirement: `instructions.get` SHALL load `template` and `instruction` bodies from an embedded `spec-driven` schema bundle compiled into the binary。
- [x] 1.2 [P] 撰寫 4 個 artifact template markdown：`crates/runtime/src/embedded/schemas/spec-driven/templates/{proposal,spec,design,tasks}.md`，內容對齊 `doc/protocol/operations.md` 對應 op 的 template skeleton。完成判準：4 檔皆為非空 markdown 含對應 `##` 主標（proposal 含 `## Why`、spec 含 `## ADDED Requirements`、design 含 `## Context`、tasks 含 `## 1.` heading）。對應 Requirement: `instructions.get` SHALL load `template` and `instruction` bodies from an embedded `spec-driven` schema bundle compiled into the binary。
- [x] 1.3 [P] 撰寫 8 個 instruction body markdown：`crates/runtime/src/embedded/schemas/spec-driven/instructions/{proposal,spec,design,tasks,apply,ingest,archive,commit}.md`，artifact kinds 4 個內容對齊現行 spec-driven 流程 prompt、workflow phase kinds 4 個對齊 `doc/skill-drafts/` 既有 draft 結構。完成判準：8 檔皆為非空 markdown、每檔首行為 `# Instructions: <kind>`、apply/ingest/archive/commit 內容明確標示「This is a workflow phase, not an artifact-producing step」。
- [x] 1.4 新增 `crates/runtime/src/embedded/mod.rs`，以 `include_str!` 將上述 13 個檔案編進 binary 並暴露 `EMBEDDED_TEMPLATES: &[(&str, &str)]` / `EMBEDDED_INSTRUCTIONS: &[(&str, &str)]` / `EMBEDDED_SCHEMA_YAML: &str` 三個 `pub const`。完成判準：`cargo build -p speclink-runtime` 通過、`cargo test -p speclink-runtime embedded::tests::all_assets_nonempty` 紅燈轉綠燈（含先寫一個 smoke test 確認 13 個 const 皆 `!is_empty()`）。

## 2. Kind enum + static dependency table（runtime layer TDD）

實作 Decision: 單一 `Kind` enum 涵蓋 8 種、用 `is_artifact_kind()` 方法分支，以及 Decision: `dependencies[]` 從靜態硬表推導，不查 runtime state。

- [x] 2.1 撰寫 failing test `crates/runtime/src/instructions_ops.rs::tests::kind_dependency_table_matches_spec`：對 8 kind 逐一 assert `Kind::dependencies()` 回傳的 `&[Dependency]` 長度與 kind 對應規範表（proposal=0、spec=1、design=2、tasks=3、apply=3、ingest=3、archive=2、commit=0）— 對應 Requirement: `instructions.get` SHALL derive `dependencies[]` from a static artifact DAG table。`cargo test` 紅燈確認。
- [x] 2.2 撰寫 `pub enum Kind { Proposal, Spec, Design, Tasks, Apply, Ingest, Archive, Commit }` + `impl Kind { fn from_str(&str) -> Result<Self, UnknownKind>; fn as_str(&self) -> &'static str; fn is_artifact_kind(&self) -> bool; fn template_path(&self) -> Option<&'static str>; fn output_path(&self) -> Option<&'static str>; fn dependencies(&self) -> &'static [Dependency]; }`，落實 Decision: 單一 `Kind` enum 涵蓋 8 種、用 `is_artifact_kind()` 方法分支。完成判準：2.1 test 轉綠燈、`cargo clippy -p speclink-runtime -- -D warnings` 通過。
- [x] 2.3 撰寫 failing test `tests::hardcoded_table_matches_embedded_schema_yaml`：使用 `serde_yaml` 解析 `embedded::EMBEDDED_SCHEMA_YAML`、對 8 kind 逐一比對 schema.yaml 內 dependency 邊集合與 `Kind::dependencies()` 結果。對應 spec scenario「Hardcoded dependency table matches schema.yaml DAG」。`cargo test` 紅燈確認。
- [x] 2.4 調整 2.2 enum 或 schema.yaml 直至 2.3 轉綠燈（兩處重複定義 DAG，本 test 為唯一同步守護）。

## 3. instructions_ops::run dispatch（runtime layer TDD）

實作 Implementation Contract 內 Observable behavior、JSON output envelope（success）與 Failure modes 區塊：8 kind happy-path envelope + unknown kind error mapping，落實 Decision: Error code 與 exit code mapping 對齊 §17 既有規範。

- [x] 3.1 撰寫 failing test `tests::run_returns_artifact_kind_envelope`：對 `{proposal, spec, design, tasks}` 各跑一次 `instructions_ops::run(Input { kind, .. })`、assert 回傳 `Output { kind, schema_id: "spec-driven", instruction: non_empty, template: Some(non_empty), output_path: Some("<kind>.md"), dependencies: <expected>, available_roles: None, linked_changes_context: None }`。對應 Requirement: `speclink instructions <kind>` SHALL return an 11-field envelope for supported artifact and workflow phase kinds + scenario「Get proposal instructions (artifact kind)」（覆蓋 Implementation Contract 內 Observable behavior 與 JSON output envelope（success） 兩 subsection）。
- [x] 3.2 撰寫 failing test `tests::run_returns_phase_kind_envelope_with_null_template`：對 `{apply, ingest, archive, commit}` 各跑一次、assert `template: None` + `output_path: None` + `instruction: non_empty` + `dependencies` 長度對應規範表。對應 scenario「Get apply instructions (workflow phase kind)」（同樣覆蓋 Observable behavior subsection 對 phase kind 的契約）。
- [x] 3.3 實作 `pub fn run(input: Input, deps: &Dependencies) -> Result<Output, Error>` 主 dispatch 邏輯：parse kind → fetch instruction body from `EMBEDDED_INSTRUCTIONS` → fetch template (artifact kinds only) from `EMBEDDED_TEMPLATES` → 取 `Kind::dependencies()` 轉 wire-format `Dependency { kind, capability: None, path }`。完成判準：3.1 + 3.2 兩 test 轉綠燈。觸發 `rust-skills:m13-domain-error` 取得 error enum idiomatic 寫法。
- [x] 3.4 撰寫 failing test `tests::run_returns_unknown_kind_error_for_discuss_and_typo`：餵 `"discuss"` + `"random_kind_xyz"` 兩 case、assert 回傳 `Err(Error::UnknownKind { kind })` + error code `instructions.unknown_kind` + hint 列 8 個支援 kind。對應 Requirement: `instructions.get` SHALL reject unknown kinds with `instructions.unknown_kind` and exit 2 + Decision: Error code 與 exit code mapping 對齊 §17 既有規範（覆蓋 Implementation Contract 內 Failure modes subsection）。
- [x] 3.5 在 `Kind::from_str` 對非 8 kind 字串回 `Err(UnknownKind)`、`run` 收到此 error 轉 `Error::UnknownKind` 並 fill hint 字串。`cargo test` 3.4 轉綠燈。

## 4. Config 三欄 hydration with fallback（runtime layer TDD）

實作 Decision: Config 三欄（`context` / `rules.<kind>[]` / `locale`）採 best-effort read + fallback null，落實 Requirement: `instructions.get` SHALL hydrate `context`, `rules`, and `locale` from config with best-effort fallback to null。本群組對應 Implementation Contract 內 Acceptance criteria 第 4–5 條 test。

- [x] 4.1 撰寫 test stub `MockConfigStore`（in `tests::mocks`），可設定 `read_config()` 回傳 `Versioned<Config>` + `take_warnings()` 回 `Vec<ConfigWarning>`。觸發 `rust-skills:m05-type-driven` 確認 trait double 寫法 idiomatic。
- [x] 4.2 撰寫 failing test `tests::run_config_file_missing_returns_all_null`：用 `MockConfigStore` 模擬 `Config::default()`（A5 對缺檔的 fallback 已寫入此 value）、assert `Output { context: None, rules: None, locale: None, .. }`。對應 spec scenario「Config file does not exist」+ Decision: Config 三欄（`context` / `rules.<kind>[]` / `locale`）採 best-effort read + fallback null。
- [x] 4.3 撰寫 failing test `tests::run_config_partial_keys_each_field_independent`：用 `MockConfigStore` 餵 `Config { locale: Some("Traditional Chinese (繁體中文)"), context: None, instructions: HashMap::new(), .. }`、assert `locale: Some("Traditional Chinese (繁體中文)")` + `context: None` + `rules: None`。對應 scenario「Config exists with partial keys」。
- [x] 4.4 撰寫 failing test `tests::run_instructions_empty_array_vs_null`：覆蓋 spec scenario 表格 — `instructions = {"proposal": vec![]}` → `rules: Some(vec![])`；`instructions = HashMap::new()` → `rules: None`。確認 null vs empty array 區分。
- [x] 4.5 撰寫 failing test `tests::run_config_malformed_forwards_warning`：`MockConfigStore` 對 `read_config` 回 `Ok(Versioned { value: Config::default(), .. })` + `take_warnings` 回 `vec![ConfigWarning { code: "config.malformed_fallback", .. }]`（模擬 A5 fallback 行為）、assert `run` 回 `Ok(...)`、envelope warnings 含 `config.malformed_fallback`。對應 scenario「Config malformed forwards A5 warning」。
- [x] 4.6 實作 `fn hydrate_config_fields(kind: Kind, config_store: &dyn ConfigStore) -> (Option<String>, Option<Vec<String>>, Option<String>, Vec<ConfigWarning>)`：呼 `ConfigStore::read_config()` → 取 `Config.context` / `Config.instructions.get(kind.as_str())` / `Config.locale` 三欄；同時呼 `take_warnings()` 收集 warnings 供 envelope forward。完成判準：4.2–4.5 四 test 轉綠燈。

## 5. Change context existence check（runtime layer TDD）

實作 Requirement: `instructions.get` SHALL verify change existence when `--change <id>` is provided and reject missing changes with `change.not_found`，落實 Decision: Change context 插值範圍限縮在「change exists check」+ output payload meta echo。

- [x] 5.1 撰寫 failing test `tests::run_with_change_id_calls_change_store_and_passes_on_existence`：用 `MockChangeStore`（spy invocations）、`Input { change_id: Some("my-feature"), .. }`、change 存在時 assert `data.schema_id: "spec-driven"` + change_store invocations count == 1。對應 Requirement: `instructions.get` SHALL verify change existence when `--change <id>` is provided and reject missing changes with `change.not_found` + Decision: Change context 插值範圍限縮在「change exists check」+ output payload meta echo。
- [x] 5.2 撰寫 failing test `tests::run_without_change_id_does_not_invoke_change_store`：`Input { change_id: None, .. }`、assert change_store invocations count == 0（驗證 spec scenario「No --change flag does not invoke ChangeStore」）。
- [x] 5.3 撰寫 failing test `tests::run_with_missing_change_id_returns_change_not_found`：change_store 設為「change 不存在」、assert `Err(Error::ChangeNotFound { id })` + error code `change.not_found`。對應 scenario「Change does not exist」（覆蓋 Implementation Contract 內 Failure modes subsection 對 `change.not_found` 的契約）。
- [x] 5.4 實作 `fn verify_change_context(change_id: Option<&str>, change_store: &dyn ChangeStore) -> Result<Option<String>, Error>`：None → 直接回 `Ok(None)`；Some(id) → `change_store.get_change(id)?`、回 schema_id（目前固定 `"spec-driven"`）；not found → `Err(Error::ChangeNotFound)`。完成判準：5.1–5.3 三 test 轉綠燈。

## 6. CLI surface（cli layer TDD）

實作 Decision: CLI subcommand `speclink instructions <kind>` 為 `Commands::Instructions { kind: String, change: Option<String>, json: bool }`，落實 Requirement: `--role` and `--discussion` flags SHALL be accepted by the CLI surface but ignored by the dispatcher。本群組對應 Implementation Contract 內 Acceptance criteria 第 7 條 + Scope boundaries 對 CLI in-scope 項目。

- [x] 6.1 觸發 `rust-skills:domain-cli` 取得 clap derive + positional + reserved flag 寫法 idiomatic 建議。
- [x] 6.2 撰寫 failing test `crates/cli/tests/instructions_cli.rs::instructions_proposal_emits_11_field_envelope`：用 `assert_cmd` 啟動 `speclink instructions proposal --json`（在 tempdir + bootstrap project）、assert exit 0、stdout JSON `data` 包含 11 個 field 且型別對齊 spec 表格。對應 Requirement: `speclink instructions <kind>` SHALL return an 11-field envelope for supported artifact and workflow phase kinds。
- [x] 6.3 撰寫 failing test `instructions_unknown_kind_exits_2`：`speclink instructions discuss --json` + `speclink instructions xyz_typo --json`、assert exit 2、stderr `error.code: "instructions.unknown_kind"` + hint 列 8 個 kind。對應 Requirement: `instructions.get` SHALL reject unknown kinds with `instructions.unknown_kind` and exit 2 + scenario「Reject `discuss` kind」/「Reject arbitrary string」。
- [x] 6.4 撰寫 failing test `instructions_change_not_found_exits_2`：`speclink instructions proposal --change nonexistent --json`、assert exit 2、stderr `error.code: "change.not_found"`、`error.message` 含 `nonexistent`。對應 Requirement: `instructions.get` SHALL verify change existence when `--change <id>` is provided and reject missing changes with `change.not_found`。
- [x] 6.5 撰寫 failing test `instructions_role_and_discussion_accepted_but_ignored`：`speclink instructions proposal --role pm --discussion abc-123 --json`、assert exit 0、`data.available_roles: null` + `data.linked_changes_context: null`、`warnings: []`。對應 Requirement: `--role` and `--discussion` flags SHALL be accepted by the CLI surface but ignored by the dispatcher + scenario「--role is accepted but ignored」。
- [x] 6.6 撰寫 failing test `instructions_help_text_mentions_phase_2`：`speclink instructions --help`、assert stdout 含 `(reserved for Phase 2)` 字串、`--role` 與 `--discussion` 兩處皆有。對應 scenario「--role help text mentions Phase 2」 + Requirement: `--role` and `--discussion` flags SHALL be accepted by the CLI surface but ignored by the dispatcher。
- [x] 6.7 撰寫 failing test `instructions_config_missing_returns_three_null_fields`：tempdir project 無 `.speclink/config.yaml`、`speclink instructions proposal --json`、assert `data.context: null` + `data.rules: null` + `data.locale: null`。對應 Requirement: `instructions.get` SHALL hydrate `context`, `rules`, and `locale` from config with best-effort fallback to null + scenario「Config file does not exist」。
- [x] 6.8 新增 `crates/cli/src/commands/instructions.rs`：clap 結構 `pub struct Instructions { pub kind: String, pub change: Option<String>, pub role: Option<String>, pub discussion: Option<String> }`、`--role` / `--discussion` 的 `help` 字串含 `(reserved for Phase 2, currently ignored)`、handler 呼 `runtime::instructions_ops::run` 並 serialize envelope。對應 Decision: CLI subcommand `speclink instructions <kind>` 為 `Commands::Instructions { kind: String, change: Option<String>, json: bool }`。
- [x] 6.9 在 `crates/cli/src/commands/mod.rs` 註冊 `pub mod instructions`、在 `crates/cli/src/main.rs` 的 `Commands` enum 加 `Instructions(commands::instructions::Instructions)` arm + dispatch。完成判準：6.2–6.7 六 test 轉綠燈、`cargo clippy -p speclink-cli -- -D warnings` 通過。

## 7. Catalogue schema population（替換 P1-1 預留 stub）

本群組落實 Implementation Contract 內 Scope boundaries「In scope: 在 `crates/runtime/src/catalogue/schemas.rs` 把 `instructions_get()` / `instructions_get_outputs()` 兩個 stub 替換成真實 schema」項目。Codebase 無 central op dispatcher — CLI subcommand 直呼 `instructions_ops::run`，與 P1-1 / A5 既有 pattern 一致；本群組只負責讓 catalogue entry 32 的 schema 函式回傳 operations.md 規範的 schema，而非 `empty_object_schema()` stub。

- [x] 7.1 撰寫 failing test `crates/runtime/src/catalogue/schemas.rs::tests::instructions_get_inputs_schema_matches_operations_md`（或在 `crates/runtime/tests/catalogue_schemas_instructions.rs` 整合測試）：assert `schemas::instructions_get()` 回傳 JSON Schema `properties` 包含 4 個 key（`kind` enum 含 9 值、`change_id` / `role` / `discussion_id` 為 nullable string）、`required: ["kind"]`、`additionalProperties: false`。對應 operations.md §`instructions.get` Inputs schema。`cargo test` 紅燈確認。
- [x] 7.2 撰寫 failing test `tests::instructions_get_outputs_schema_matches_operations_md`：assert `schemas::instructions_get_outputs()` 回傳 JSON Schema `properties` 包含 11 個 key（kind / schema_id / instruction / template / context / rules / dependencies / output_path / locale / available_roles / linked_changes_context）、`required: ["kind", "instruction"]`、`rules.items.type: "string"`、`dependencies.items` 為 object 含 kind/capability/path。對應 operations.md §`instructions.get` Outputs schema。`cargo test` 紅燈確認。
- [x] 7.3 把 `schemas::instructions_get()` 與 `schemas::instructions_get_outputs()` 兩個 stub 函式 body 替換成真實 schema（保留 `pub fn ... -> Value` 函式簽名、不動 catalogue entry 32 metadata）。7.1 + 7.2 轉綠燈，並執行 `cargo test -p speclink-runtime --test catalogue_doc_sync` 確認 P1-1 既有 sync test 仍通過（catalogue ↔ operations.md 對齊未被破壞）。

## 8. Smoke / consistency tests

本群組對應 Implementation Contract 內 Acceptance criteria 第 1 + 6 + 8 條 test，守護 Decision: Embedded schema bundle 用 `include_str!` 編進 binary、不走 filesystem lazy load 的 Risk Mitigation 段落。

- [x] 8.1 撰寫 test `tests::embedded_artifact_template_contains_expected_section`：對 4 個 artifact kind 逐一 assert template 含預期主標（proposal→`## Why`、spec→`## ADDED Requirements`、design→`## Context`、tasks→`## 1.`）。對應 Decision: Embedded schema bundle 用 `include_str!` 編進 binary、不走 filesystem lazy load 的 Risk Mitigation。
- [x] 8.2 撰寫 test `tests::embedded_instruction_bodies_nonempty_for_all_8_kinds`：對 8 kind 逐一 assert instruction body `!is_empty()` 且 byte length > 100。對應 spec scenario「Instructions are non-empty for all 8 supported kinds」+ Requirement: `instructions.get` SHALL load `template` and `instruction` bodies from an embedded `spec-driven` schema bundle compiled into the binary。
- [x] 8.3 撰寫 test `tests::output_envelope_matches_operations_md_schema`：對 8 kind 各取一次 output，用 `jsonschema` crate 比對 operations.md §`instructions.get` 出的 outputs JSON schema（從 catalogue entry 32 的 `outputs_schema()` fn 取，與 P1-1 對齊機制相同）。確認 envelope 11-field shape 不漂移。

## 9. Refactor + idiomatic pass

本群組對應 Implementation Contract 內 Scope boundaries 收口檢查：確認所有 in-scope 完成、無 out-of-scope 越界（例如不應該動 `crates/provider/` trait、state.db schema、catalogue Operation struct）。

- [x] 9.1 觸發 `rust-skills:m15-anti-pattern` 對 `crates/runtime/src/instructions_ops.rs` 全檔審查、修正 anti-pattern 並維持 8 個 instruction body + 11-field envelope 行為不變。完成判準：`cargo test --workspace` 全綠、`cargo clippy --workspace -- -D warnings` 無警告。
- [x] 9.2 觸發 `rust-skills:coding-guidelines` 對 `crates/cli/src/commands/instructions.rs` + `crates/runtime/src/embedded/mod.rs` 進行 idiomatic 檢查、確認 doc comment（`///`）覆蓋 public API、無 `unwrap()` / `expect()`（embedded module 例外允許 `include_str!` 配 `&'static str` const）。
- [x] 9.3 執行最終驗證套組：`cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`，全綠後本 slice 實作完成。同時 diff `crates/provider/` / `crates/provider-local/migrations/` / `crates/runtime/src/catalogue/` 確認 Scope boundaries 內 out-of-scope 項目皆未被異動。
