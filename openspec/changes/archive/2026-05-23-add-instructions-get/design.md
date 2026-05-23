## Context

SpecLink 已 ship Phase 1 #1（`add-tool-describe-and-catalogue`）與 #2（`add-project-status`）；catalogue 第 32 條 `instructions.get` 已在 `crates/runtime/src/catalogue/mod.rs` 預留 entry，但 dispatch 與 schema 仍是 stub。本 slice 補上實作，讓 P1-4 skill deploy 後 skill workflow.md 有 prompt source 可引用、dogfooding 可啟動。

- Canonical 設計來源：`doc/speclink-design.md` §11.7（Rules 注入機制流程）+ §18.4（Phase 出貨計畫 P1-3 範圍）+ §7（spec-driven artifact DAG）；`doc/protocol/operations.md` §`instructions.get`（11-field stable output envelope、5 input field、error code）。
- A5 `add-config-rw` 已 ship `ConfigStore::read` → `Versioned<Config>`，回傳 parsed YAML `serde_json::Value`，但 `openspec/specs/config-rw/spec.md` 只明列 `rules.require_artifact_review` / `rules.require_code_review` 兩欄；`context` / `rules.<kind>[]` / `locale` 三欄是否進 config schema 屬 dogfood backlog，本 slice 不擴 A5。
- P1-1 catalogue 第 32 條 entry inputs_schema 與 outputs_schema 已對齊 operations.md（catalogue_doc_sync test 守護）；本 slice 只接 dispatch、不動 schema 結構。
- Phase 2 才補 `discuss.{new,list,show,patch,conclude,delete}`（`add-discuss-ops`）與 `schema.{list,show,fork,delete}`（`add-schema-ops`）；本 slice 不能引入兩者依賴。
- crate 內無任何 `embedded/` 目錄，須立此 convention；`include_str!` 既有 pattern 來自 `crates/runtime/tests/catalogue_doc_sync.rs:9` 與 `crates/provider-local/src/state_db.rs:102`。
- 8 kind 中，proposal/spec/design/tasks 屬 artifact kinds（回 template + output_path）；apply/ingest/archive/commit 屬 workflow phase kinds（template/output_path 為 null，只回 instruction body）。

## Goals / Non-Goals

**Goals:**

- `instructions.get` op 在 runtime + CLI 完整可用，11-field stable envelope 對齊 operations.md §`instructions.get`。
- 8 kind 全覆蓋：proposal / spec / design / tasks / apply / ingest / archive / commit。
- Embedded `spec-driven` schema bundle 透過 `include_str!` 編進 binary；無外部 filesystem 依賴。
- 從 A5 `ConfigStore::read` 拿 `context` / `rules.<kind>[]` / `locale`；欄位不存在或 config 不存在時 fallback null，不阻斷請求。
- `dependencies[]` 從靜態硬表推導（§7 artifact DAG），不查 runtime state。
- 嚴格 TDD：先寫 failing test 再實作；測試覆蓋 8 kind happy path + 2 error code + config fallback + change context 插值。

**Non-Goals:**

- 不實作 `kind=discuss`、`role` 參數、`discussion_id` 參數、`available_roles[]`、`linked_changes_context[]`。Discuss 後端於 Phase 2 `add-discuss-ops`。
- 不引入 user-overridable schema fork、不建立 `.speclink/schemas/`、不暴露 `schema.{list,show,fork,delete}` op。Schema fork 於 Phase 2 `add-schema-ops`。
- 不做 multi-instance spec capability 動態解析：`dependencies[].capability` 永遠為 null。Capability 視角於 Phase 2 `add-spec-canonical-read`。
- 不擴 A5 `ConfigStore` trait；不修改 `openspec/specs/config-rw/spec.md`；不補 config.yaml schema 對 `context` / `rules.<kind>[]` / `locale` 欄位的明文支援（留 dogfood backlog）。
- 不新增 provider trait method；本 slice 不過 provider 邊界（除了 A5 既有 `ConfigStore::read`）。
- 不變動 catalogue 既有 op 結構；不改 `outputs_schema` 函式簽名；不破壞 P1-1 `catalogue_doc_sync` test。
- 不新增 lifecycle state、lock、audit event；不異動 auth / token / secret 路徑。

## Decisions

### Decision: Embedded schema bundle 用 `include_str!` 編進 binary、不走 filesystem lazy load

**Why**：MVP 只支援 binary 內建 `spec-driven` schema（§10.1）；schema fork ops 屬 Phase 2。`include_str!` 已是 SpecLink 既有 pattern（`catalogue_doc_sync.rs`、migration SQL），無新依賴、無 runtime IO、binary 自帶 — 不需要 `.speclink/` 任何前置物即可呼叫 `instructions.get`。

**Alternatives:**

- Lazy filesystem load from `.speclink/schemas/spec-driven/`：rejected — 需要先建立 user-overridable schema 目錄結構（Phase 2 `add-schema-ops` 才該做），且 init 流程沒寫此檔；fresh project 第一次跑 skill 就會炸。
- 用 `rust-embed` crate：rejected — 多一條依賴、`include_str!` 對 ~12 個小 markdown 檔已足夠；`rust-skills:m11-ecosystem` 對小型 static asset 建議優先用 std macro。
- 把 instruction body 寫死成 `&'static str` constant in source code：rejected — markdown body 應由 designer / writer 維護，從 `.md` 檔案 `include_str!` 進來比 string literal 易讀、易改、易 diff review。

### Decision: 單一 `Kind` enum 涵蓋 8 種、用 `is_artifact_kind()` 方法分支

**Why**：operations.md §3550 input schema 把 9 kind（含 discuss）放同一 enum；CLI surface `speclink instructions <kind>` 也是單一 positional argument。`Kind` enum 同時用作 dispatch + 序列化標籤，比拆兩個 type（`ArtifactKind` / `PhaseKind`）少一層 mapping。`is_artifact_kind()` / `template_path()` / `output_path()` / `dependencies()` 全部掛在同一 impl block。

**Alternatives:**

- 拆 `ArtifactKind` (proposal/spec/design/tasks) + `PhaseKind` (apply/ingest/archive/commit) 兩個 enum：rejected — input parser 仍要回 union type、序列化 tag 也要統一；double type 增 boilerplate、無對應 invariant 保護。
- 用 `&'static str` 而非 enum：rejected — 違反 `rust-skills:m05-type-driven`（用 type 表達 domain invariant、拒絕 stringly-typed）；非法 kind 應在 parse 階段就 reject 而非 dispatch 才發現。

`kind=discuss` 處理：parse 階段接受（避免 clap 拒絕未來會合法的值），dispatch 階段回 `instructions.unknown_kind` error。

### Decision: `dependencies[]` 從靜態硬表推導，不查 runtime state

**Why**：§7 artifact DAG 是 schema 屬性、不是 change 實例屬性 — proposal/spec/design/tasks 的依賴關係對所有 change 一樣。靜態表存在 `Kind::dependencies() -> &'static [Dependency]`，編譯期常數、零 runtime cost、零 IO。

依賴表：
- `proposal` → `[]`
- `spec` → `[proposal]`
- `design` → `[proposal, spec]`
- `tasks` → `[proposal, spec, design]`
- `apply` → `[proposal, spec, tasks]`
- `ingest` → `[proposal, spec, tasks]`
- `archive` → `[spec, tasks]`
- `commit` → `[]`

`dependencies[].capability` 永遠為 null（§Non-Goals）；`dependencies[].path` 為 schema template 的 `output_path`（如 `proposal.md`），caller 拿 path 後可呼 `artifact.read`。

**Alternatives:**

- 從 schema.yaml 解析 DAG：deferred — schema.yaml 仍 embed，但本 slice runtime 不解析它（schema.yaml 留給 Phase 2 `add-schema-ops` 對接）；現在用 hardcode table 與 schema.yaml 在 P1-3 重複定義，由 schema.yaml 文字內容當文檔、Rust hardcode 當 runtime SOT。
- 從 change.artifact 表 query 已建立 artifact：rejected — 違反 §11.7 設計 — instructions 是「該寫什麼」的指引，與「目前寫到哪」獨立。Change context 影響的是是否填入 capability 字段（本 slice 一律 null），不影響 dependency edge 本身。

### Decision: Config 三欄（`context` / `rules.<kind>[]` / `locale`）採 best-effort read + fallback null

**Why**：§11.6 設計原則「malformed = warning + fallback」。A5 `ConfigStore::read` 回 `Versioned<Config>`、本 slice 對 `Config` struct 做 additive 擴充（新增三個 Optional 欄位：`context: Option<String>`、`locale: Option<String>`、`instructions: HashMap<String, Vec<String>>`），由 serde 自動 hydrate；trait method 簽名不變。`hydrate_config_fields` 直接讀三個欄位、`None` / empty → wire-format null；不再走 JSONPath（serde 已處理 type-safe 解析）。`instructions.get` 是 read-only meta op，永遠不該因 config 不完整而 fail；缺欄位讓 skill 看到 null 自然降級。

設計文件 §11.7 寫的路徑是 `rules.<kind>[]`，但 A5 `Rules` struct 是 strict（內含 `require_artifact_review` / `require_code_review` 兩個 boolean），無法混存 per-kind string array。改用頂層 `instructions.<kind>[]` 路徑，避開型別衝突；對應 `Config.instructions: HashMap<String, Vec<String>>` 欄位。

具體 fallback：
- A5 `LocalConfigStore::read_config` 對 config 缺失 / malformed 已內建 fallback 行為：回 `Ok(Versioned { value: Config::default(), .. })` + 透過 `take_warnings` emit `config.malformed_using_defaults`，**不 raise `config.malformed` error**（A5 spec 明寫）。所以 `instructions.get` 看到的 Config 永遠是 well-formed（最差是 `Config::default()`，三欄皆 None / empty）。
- Config 內 `context` 欄缺失 → `Config.context: None` → wire-format null
- Config 內 `instructions.<kind>` 缺 key → `Config.instructions.get(kind)` 回 None → wire-format null（與 empty Vec `[]` 區別保留）
- Config 內 `locale` 欄缺失 → `Config.locale: None` → wire-format null
- A5 已有 `config.malformed_using_defaults` warning：本 op 透過 `take_warnings` 收集後 propagate 到 envelope 的 `warnings[]`

**Alternatives:**

- 強制 `config.yaml` 必須有 `context` / `instructions` / `locale` 三欄：rejected — 違反 §11.6；初始化的 config.yaml 不會有這些，第一次跑 skill 就 fail 不可接受。
- 用 JSONPath subset 讀 `serde_yaml::Value`：rejected — A5 提供 strongly-typed `Config` struct，本 slice additive 擴三個 Optional 欄位即可，無需走 raw YAML 路徑；JSONPath 適合 config show CLI 的 user-driven query，不適合 op-internal 型別已知的讀取。
- 在 instructions.get 同時 silent 吞 `config.malformed`：N/A — A5 已決定 malformed → fallback to defaults + warning（不 raise error），本 op 不需另外處理。

### Decision: Change context 插值範圍限縮在「change exists check」+ output payload meta echo

**Why**：operations.md §3667 semantics step 3 寫「若 `change_id` 提供、套用 change-specific 插值（如 capability list、prior state）」。Capability list 屬 Phase 2 `add-spec-canonical-read`；prior state 對 4 個 artifact kind 沒有等價語意（template / instruction body 都是 schema-static）。本 slice 在提供 `--change <id>` 時只做兩件事：
1. **Existence check**：透過 `ChangeStore::get_change(id)`，不存在回 `change.not_found`（exit 2）
2. **Schema id resolution**：從 change row 讀 `schema_id` 欄（未來 schema fork 用，目前一律 `spec-driven`）

不修改 `instruction` / `template` / `dependencies` 內容（這些都是 schema-static），不注入 capability list（Phase 2）。

**Alternatives:**

- `--change` 不做 existence check、純當 metadata echo：rejected — operations.md §3679 已列 `change.not_found` 為 error code，跳過違反 contract。
- 在 instruction body 內插值 `${change_id}` 占位符：rejected — 過早優化、目前 8 個 instruction body 沒人用此占位符；插值若有需要由 caller AI 在 prompt 端處理。

### Decision: CLI subcommand `speclink instructions <kind>` 為 `Commands::Instructions { kind: String, change: Option<String>, json: bool }`

**Why**：與 P1-1 `describe-tools` / A5 `config show` 既有 CLI pattern 對齊 — clap derive、positional + `--change` flag + 全域 `--json` flag。`kind` 用 `String` 而非 `clap::ValueEnum<Kind>` 接：避免 clap 在 surface 上把 8 kind 寫死（未來 schema fork 後 kind 集合可變），改在 runtime parse 階段 reject unknown kind 回 `instructions.unknown_kind`。

**Alternatives:**

- `clap::ValueEnum` derive on `Kind`：rejected — kind 集合屬 active schema 屬性，未來 schema fork 後動態變化；clap-level 寫死將與 §10 schema 抽象設計衝突。
- 拆 `speclink instructions proposal` / `speclink instructions tasks` 等 8 個 subcommand：rejected — surface 變寬、`describe-tools` 也只列一條 op、與 catalogue 1:1 對應。

### Decision: Error code 與 exit code mapping 對齊 §17 既有規範

| Error code | Exit | 觸發條件 |
|---|---|---|
| `instructions.unknown_kind` | 2 | `<kind>` 不屬於當前 schema 支援集合（含 `discuss`、typo 等） |
| `change.not_found` | 2 | 提供 `--change <id>` 但 change row 不存在 |
| `config.malformed` | bubble | A5 既有 — config.yaml 解析失敗時由 ConfigStore 拋出，本 op 不另包 |

不引入新 error code；`role.unknown` / `discussion.not_found` 屬 Phase 2。

**Alternatives:**

- 把 `instructions.unknown_kind` 細分成 `instructions.discuss_not_yet_supported`：rejected — Phase 2 ship discuss 後此 code 就無用；保持單一 unknown_kind 對 forward compat 友善。

## Implementation Contract

### Observable behavior

- 執行 `speclink instructions <kind> [--change <id>] [--json]`，當 kind ∈ {proposal, spec, design, tasks, apply, ingest, archive, commit} 且（提供時）`change` 存在 → exit 0、stdout 為 11-field JSON envelope。
- 當 kind = discuss 或 unknown string → exit 2、stderr 為 `instructions.unknown_kind` 錯誤 envelope。
- 當 `--change <id>` 提供但 change 不存在 → exit 2、stderr 為 `change.not_found` 錯誤 envelope。
- 不提供 `--change` 時，op 仍 success；output 中 `schema_id` 為 active schema（目前固定 `"spec-driven"`），`change` 相關插值不發生。

### CLI surface

```
speclink instructions <kind> [OPTIONS]

Arguments:
  <kind>                    Artifact kind 或 workflow phase kind

Options:
      --change <CHANGE_ID>  套用 change context（existence check + schema_id 解析）
      --json                Machine-readable JSON output（同 global --json）
      --role <ROLE>         (accepted, ignored — 留給 Phase 2)
      --discussion <ID>     (accepted, ignored — 留給 Phase 2)
```

`--role` / `--discussion` clap surface 接受（避免未來破 surface），dispatch 時忽略；說明文字標 "(reserved for Phase 2)"。

### JSON output envelope（success）

```json
{
  "ok": true,
  "data": {
    "kind": "proposal",
    "schema_id": "spec-driven",
    "instruction": "<markdown body from embedded instructions/<kind>.md>",
    "template": "<markdown skeleton from embedded templates/<kind>.md, or null for phase kinds>",
    "context": "<string from config.yaml#context, or null>",
    "rules": ["<from config.yaml#rules.<kind>[]>", "..."],
    "dependencies": [
      { "kind": "proposal", "capability": null, "path": "proposal.md" }
    ],
    "output_path": "proposal.md",
    "locale": "Traditional Chinese (繁體中文)",
    "available_roles": null,
    "linked_changes_context": null
  },
  "warnings": [],
  "requestId": "<uuid>"
}
```

Phase kinds（apply/ingest/archive/commit）的差異：`template` 為 null、`output_path` 為 null；其他 field 結構相同。
`rules` 為 array（caller 端可 iterate）；config 無對應欄位時為 `[]`，整 config 不存在時為 `null`（與其他兩欄一致）。

### Failure modes

| Code | Exit | Envelope shape |
|---|---|---|
| `instructions.unknown_kind` | 2 | `{ok:false, error:{code:"instructions.unknown_kind", message:"Unknown kind: <kind>", hint:"Supported: proposal, spec, design, tasks, apply, ingest, archive, commit", retryable:false}}` |
| `change.not_found` | 2 | `{ok:false, error:{code:"change.not_found", message:"Change <id> not found", hint:null, retryable:false}}` |
| `config.malformed` | bubble | A5 ConfigStore 既有 envelope，op layer 不改 |

### Acceptance criteria

實作完成的判準（每條對應一個 test）：

1. `crates/runtime/tests/instructions_ops.rs` 8 個 happy-path test（proposal/spec/design/tasks/apply/ingest/archive/commit）— assert exit 0、`data.kind` 對齊、artifact kinds 有 template+output_path、phase kinds 兩者為 null。
2. 同檔 `change_context_existence_check` test — 用 stub ChangeStore，提供存在 change 回 success，不存在 change 回 `change.not_found`。
3. 同檔 `unknown_kind_returns_error` test — 餵 `discuss` / `random_string` 兩 case 都拿到 `instructions.unknown_kind`。
4. 同檔 `config_missing_fallback_to_null` test — 用 stub ConfigStore 回 `config.not_found`，assert `context` / `rules` / `locale` 三欄為 null + `rules` 注意是 `null` 不是 `[]`。
5. 同檔 `config_partial_keys_individual_null` test — config 有 `locale` 沒 `context` 沒 `rules.proposal`，assert 該欄為 null 其他正常。
6. 同檔 `dependency_table_matches_schema_dag` test — 對 8 kind 逐一 assert `dependencies` 長度與 `kind` 對應 §7 DAG。
7. `crates/cli/tests/instructions_cli.rs` 含 `--json` happy path + 兩 error code exit 2 + `--role` / `--discussion` accepted-but-ignored 行為。
8. `crates/runtime/tests/catalogue_doc_sync.rs` 既有 test 仍通過（P1-1 守護的 catalogue ↔ operations.md sync）；catalogue 第 32 條 entry 接 dispatch 後此 test 行為不變。

### Scope boundaries

**In scope:**
- 新增 `crates/runtime/src/instructions_ops.rs` + `crates/runtime/src/embedded/` 目錄樹
- 新增 CLI subcommand handler `crates/cli/src/commands/instructions.rs`
- 在 `crates/runtime/src/lib.rs` 暴露模組
- 在 `crates/runtime/src/catalogue/schemas.rs` 把 `instructions_get()` 與 `instructions_get_outputs()` 兩個 stub（目前皆為 `empty_object_schema()` / `empty_object_outputs_schema()`）替換成真實 5-field input + 11-field output schema；函式簽名不動、catalogue entry 32 metadata 不動。對應 operations.md §`instructions.get` Inputs / Outputs JSON schema。
- 在 `crates/cli/src/commands/mod.rs` 與 `crates/cli/src/main.rs` 註冊 subcommand
- 8 個 instruction body markdown 撰寫（內容對齊 §11.7 設計與 P1-1 spec/proposal/design/tasks instruction 風格；workflow phase kinds 內容對齊 `doc/skill-drafts/` 既有 draft 結構）
- TDD：每條 acceptance criteria 對應一個先寫的 failing test

**Out of scope:**
- 不改 `crates/provider/` trait 介面
- 不改 `crates/provider-local/`（ConfigStore / ChangeStore 行為不變）
- 不擴 `state.db` migration（不新增 v6）
- 不改 catalogue `Operation` struct 欄位或 `outputs_schema` 函式簽名；不動 catalogue entry 32 的 metadata（id / category / cli / tool_binding / sdk_method / http_endpoint / mvp / destructive / idempotency / lock / phases / curated / description）
- 不引入 central op dispatch table：codebase 既有設計是 CLI subcommand 直呼 runtime 模組（如 `commands::config` → `ConfigOperations`），本 slice 沿用此 pattern，不引入 `match op_id` 的 router 反 pattern
- 不寫 user-facing schema fork CLI / doc
- 不引入 i18n 變更（locale 只 echo，不影響 instruction body 內容）
- 不寫 skill workflow.md（P1-4 範疇）

## Risks / Trade-offs

- **Risk**：embedded `templates/` 與 `instructions/` markdown 內容若與 `doc/protocol/operations.md` / `doc/speclink-design.md` 既有指引漂移，AI 看到的會是 stale prompt。  
  **Mitigation**：本 slice 完成後在 `crates/runtime/tests/instructions_ops.rs` 加 smoke test `embedded_bodies_nonempty` + `embedded_artifact_template_contains_expected_section`（如 proposal 含 "## Why"、tasks 含 "- [ ]"）；長期由 P1-4 dogfood 把漂移踩出來。

- **Risk**：硬表 `Kind::dependencies()` 與 embed 的 `schema.yaml` 重複定義 DAG。  
  **Mitigation**：本 slice 內加 test `hardcode_table_matches_schema_yaml`（解析 schema.yaml、逐 kind 比對 dependencies），雖然 runtime 不讀 schema.yaml 但 test 讀、確保兩處同步。Phase 2 `add-schema-ops` ship 後此 hardcode 表轉為 schema.yaml 解析結果的 cache，test 變 redundant 可刪。

- **Risk**：`--role` / `--discussion` clap surface accept-but-ignore 行為可能誤導 user 以為已生效。  
  **Mitigation**：help text 明標 "(reserved for Phase 2, currently ignored)"；envelope 內 `available_roles` / `linked_changes_context` 恆 null 也是 signal。Phase 2 ship 後此 surface 變實作。

- **Trade-off**：config 三欄不擴 A5 spec → AI skill 看到 config 不存在的 `context` 欄會永遠 null。  
  **Why accept**：§11.6 fallback null 是設計原則；dogfood 真的痛了再開獨立 `polish-config-instructions-schema` slice 補。

- **Risk**：`Kind` enum 在 surface 上不限定，clap 把任何 string 都接過來 → bad UX（typo 要跑到 runtime 才看到 error）。  
  **Mitigation**：error envelope `hint` 列完整支援 kind 清單，user 一看就知道哪個拼錯；長期由 schema fork ops 對應的 dynamic completion 解決。

## Open Questions

- 8 個 instruction body 撰寫時，是否抄 spectra 既有 `.claude/skills/spectra-*/SKILL.md` 結構？建議：apply/ingest/archive/commit 4 個 phase instruction body 對齊 `doc/skill-drafts/` 既有 draft（若 dir 內有對應檔）；artifact kinds 4 個對齊 `doc/protocol/operations.md` 各 op 的 instruction 段落。撰寫期間如發現必須等 Phase 2 才能確定的內容（如 role-aware discuss prompt），inline 留 `(Phase 2 will refine)` 註記不阻塞。

