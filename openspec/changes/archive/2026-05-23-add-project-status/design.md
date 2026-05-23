## Context

SpecLink 已 ship Phase 1 #1（`add-tool-describe-and-catalogue`），catalogue 與 `speclink describe-tools` CLI 在位；下一步 Phase 1 #2 補 `project.status` 與 `change.show` envelope，讓 dogfooding 啟動瞬間就有可用的「目前狀況一眼看完」與「下一動作建議」。

- `project.status` 的 op 規格已寫在 `doc/protocol/operations.md` §1360，output schema 已定型（§1389）。catalogue entry 已在 `crates/runtime/src/catalogue/mod.rs`，schema 是 stub，本 slice 補齊。
- `change.show` 已在 `crates/runtime/src/change_ops.rs::show_change` 實作，envelope 是 `{change, artifacts[]}`；按 design.md §18.5 backlog #2 指示本 slice 順手擴充。
- LocalProvider 既有 method 已可支援：`LocalChangeStore::list_changes()`（回完整 `ChangeRow`，含 state column）、`get_project_metadata`（回 `instance_id` / `working_dir` / `created_at`）。state.db 已有 `change` 表（v2，含 `all_tasks_done` column 由 task_ops 維護）。
- Phase 2 #1 `add-discuss-ops` 才會引入 discussion store，本 slice 不能假設 discussion 表存在。
- human output 一律走 `cli-human-output` spec 的 YAML pretty-printer，已歸檔。

## Goals / Non-Goals

**Goals**：
- `project.status` op 完整可用：CLI + envelope schema + `current_change` 偵測。
- `change.show` envelope 加 `all_tasks_done: bool` + `next_actions: [string]`，dogfood UX 提升。
- 不引入新 provider trait method，runtime in-memory aggregation。
- 嚴格 TDD，先寫 failing test 再實作。

**Non-Goals**：
- 不做 dashboard table（YAML 即可；後續 dogfood 痛了再開新 slice）。
- 不重算 `all_tasks_done`（讀 change 表 column 即可）。
- 不算 artifact DAG ready 狀態（屬 P1-3 `add-instructions-get`）。
- 不實作 discussions 計數（P2 #1 補；本 slice 永遠回 0）。
- 不新增 error code。
- catalogue 結構：本 slice **擴 Operation struct 加 `outputs_schema: fn() -> Value` 欄位**（B 方案），讓 catalogue 雙向自描述。Non-Goal 是「不重命名既有欄位、不改 `inputs_schema` 函式簽名、不變動既有 37 個 op 的 inputs schema 內容」。
- 不破壞 `show change` 既有 consumer：兩新欄位採 additive 加在 envelope，舊欄位不動。

## Decisions

### Decision: project.status 走 runtime-side aggregation，不新增 provider trait method

**Why**：避免「provider 加 `count_changes_by_state` thin-wrapper」這種 adapter 過薄反模式（design 階段 interface depth check：seam 在 runtime、provider 只負責 row-level read）。`LocalChangeStore::list_changes()` 已回完整 `ChangeRow`，runtime 直接 group-by `state` 計算 6 個 bucket 即可。

**Alternatives**：
- 在 provider 加 `count_changes_by_state() -> HashMap<State, u64>`：rejected — 多一層 wrapper、provider 介面變寬、HttpProvider 同步要實作；目前 list_changes 已 OK，row count 對個人 RD scale 可接受。
- 直接在 cli 層算：rejected — runtime 才是 op layer，CLI 只負責 dispatch。

### Decision: discussions_count 此 slice 永遠回 `{active: 0, converged: 0}`、欄位保留

**Why**：discussion store 屬 P2 #1。omit field 會違反 operations.md §1389 schema `required` 列；現在不寫 0 → P2 ship 時 envelope shape 從「無欄位」變「有欄位」會破 consumer。值固定 0 是 forward-compatible 過渡方案。

**Alternatives**：
- omit 欄位 + 等 P2 補：rejected — 違反 operations.md output schema。
- 提早建 discussion 表 stub：rejected — 屬 P2 工作搶到 P1、scope 失控。

### Decision: current_change 採 actor.host_id 比對；無則 null

**Why**：operations.md §1396 明寫「若有 change 處於 in_progress + actor 為當前 host；否則 null」。

**Fix（dogfood 後修正）**：原 propose 階段把「當前 host」對應到 `link.yaml.instance_id`（project UUID）— 這是語意錯位：`actor.host_id` 由 `state_machine::resolve_host_id()` 寫入、值為 OS hostname；`instance_id` 是 project 級識別碼。兩者永遠不相等，導致 dogfood headline UX 失效（e2e 測試 anomaly #1）。改為「比對當前 host 的 hostname identifier（與 apply.start 寫入 actor.host_id 的同一條 resolution chain）」。

**邏輯**：
1. `list_changes()` 過濾 `state == "in_progress"`
2. 對每個 row 讀 `actor.host_id`（apply_task_ops 寫入 change 表的 `actor_json` column）
3. 若有 row 的 `host_id == state_machine::resolve_host_id()`（即「假如現在跑 apply.start 會寫入的同一個 hostname」） → 填 `current_change`（取 updated_at 最新的）
4. 找不到 → null

**動作**：
- 把 `crates/runtime/src/state_machine.rs::resolve_host_id()` 從 private 改為 `pub`，讓 `project_ops` 可重用同一條 resolution chain。
- `project_ops::project_status` 比對端從 `link.instance_id` 換成 `state_machine::resolve_host_id()`。
- 既有 integration test fixture（`crates/runtime/tests/project_ops.rs`）的 host_id 從 `current_instance_id(working)` 換成 `state_machine::resolve_host_id()`；無需新增 dep。

**Alternatives**：
- 不過濾 host_id，回所有 in_progress：rejected — 違反 operations.md spec、多 host（雖 MVP 為 single-RD）時誤導。
- 改 `apply.start` 寫 `actor.host_id = instance_id`：rejected — 違背 `actor.host_id` 的語意（hostname 不是 project UUID），且 audit log 失去 host traceability。
- 加 CLI `--host-id` flag：rejected — 多一個 flag 解 dogfood 主流程是 UX 反模式。

### Decision: change.show 加 all_tasks_done 從 change 表 column 直接讀

**Why**：`apply-task-ops` slice 已維護 change 表 `all_tasks_done` column（task_ops 寫入），重算等於違反 SSOT。

**Alternatives**：
- show_change 內 parse tasks.md `- [x]` / `- [ ]` count：rejected — 重複邏輯、與 task_ops 計算結果可能不一致。

### Decision: change.show next_actions 採 state-driven 查表 + in_progress 用 1-based sequential checkbox index

**Why**：絕大多數 state 的 next action 純由 state enum 決定；只有 `in_progress + all_tasks_done=false` 需要從 tasks.md 找下一個 task。proposing 狀態的 artifact 缺項過濾用既有 `discover_artifacts` 結果反推。

**Fix（dogfood 後修正）**：原 propose 階段 emit `"task.done <label>"`（如 `"task.done 2.1"`）— 但 `speclink task done` CLI 只收 1-based integer INDEX、不收 label，AI agent 照 next_actions literal 跑會 `invalid digit found in string` retry loop（e2e 測試 anomaly #2）。改為 emit `"task.done <INDEX>"` 用 `parse_checkbox_lines` 算出的 1-based sequential index（與 task.done CLI 同套 index 規則）。

**查表**：
| state | all_tasks_done | next_actions |
|---|---|---|
| `proposing` | — | `["artifact.write proposal", "artifact.write design", "artifact.write tasks"]` 過濾掉已 done 的 kind |
| `reviewing` | — | `["review.approve", "review.reject"]` |
| `ready` | — | `["apply.start"]` |
| `in_progress` | false | `["task.done <INDEX>"]`（`<INDEX>` = `parse_checkbox_lines(tasks.md)` 第一個 `done == false` 的 `.index` 值，與 task_ops 一致）；tasks.md 不存在或無 unchecked → `["task.done"]` |
| `in_progress` | true | `["archive.run"]` |
| `code_reviewing` | — | `["review.approve", "review.reject"]` |
| `archived` | — | `[]` |

**動作**：
- 移除 `change_ops::parse_first_pending_task_id` 自行 parse 函式（過早抽象、回 label）。
- `compute_next_actions` 在 in_progress + !all_tasks_done 分支改呼叫 `task_ops::parse_checkbox_lines(content)`，取第一個 `!item.done` 的 `item.index`。
- 既有 unit test fixture 改 assert index 數字而非 label。

**Alternatives**：
- next_actions 永遠空、由 UI 自算：rejected — 違反 design.md §18.5 backlog #2 explicit 指示。
- 算 artifact DAG ready 狀態給 next_actions：rejected — 屬 P1-3 `add-instructions-get` 範圍。
- 改 `task done` CLI 同時接受 label：rejected — 多一條輸入路徑、且 label 不保唯一（多個 task 都叫 `1.1` 也合法）；index-only 是 task.done 既有契約、`next_actions` 對齊 CLI 較乾淨。

### Decision: speclink status human output 走 cli-human-output YAML printer

**Why**：`cli-human-output` spec 已歸檔，行為是「envelope.data → indented YAML」。design.md §18.5 已明寫 `status` / `list` 在 human-mode 為 YAML 非 table 屬 spec 行為、非 bug。本 slice 不破例。

**Alternatives**：
- 寫專屬 dashboard table renderer：rejected — 違反 cli-human-output spec、scope 失控。

### Decision: runtime 新建 project_ops 模組；change.show 兩欄位住既有 change_ops

**Why**：interface depth check 通過（seam 清、aggregator 真做 group-by + table lookup、不是 thin-wrapper）。change.show 仍在 change_ops，避免拆得太細。

**模組結構**：
```
crates/runtime/src/
  project_ops.rs        ← NEW（project.status）
  change_ops.rs         ← MOD（show_change 加 2 欄位）
  lib.rs                ← MOD（pub mod project_ops）
```

**Alternatives**：
- 把 project.status 塞 change_ops：rejected — 命名違和、project scope ≠ change scope。
- 抽 project_ops + change_ops 共用 stats crate：rejected — 過度設計、Y Premature abstraction。

### Decision: catalogue schemas 對應升級

**Why**：catalogue 的 `project_status()` 與 `change_show()` 是 schema source；本 slice 加實際 envelope shape 後，schema 必須對齊（避免 `describe-tools --filter project.status` 印 stub）。

**動作**：
- `project_status()` inputs schema：維持 `{additionalProperties:false, properties:{}}`（project.status 無 input arg）。
- `change_show()` inputs schema：維持既有 `{name}`（無新 input arg）。
- 兩個 op 的 **outputs schema** 透過 catalogue 新欄位 `outputs_schema` 暴露 — 見下方獨立 Decision。

### Decision: catalogue Operation struct 擴 `outputs_schema: fn() -> Value` 欄位（B 方案）

**Why**：原 Non-Goal 「不動 catalogue 結構」在 propose 階段定得太緊；spec Requirement 已寫明 `describe-tools` 必須能印 `outputs_schema`，且 SDK 強型別 codegen / catalogue 自描述對人類 UX 有顯著價值。對齊用 propose-after discuss 後決定的 **B 方案**：加 struct 欄位、命名誠實、編譯器強制每 op 都填、可獨立呼叫。

**動作**：
- `crates/runtime/src/catalogue/mod.rs::Operation` struct 加 `pub outputs_schema: fn() -> Value` 欄位（與既有 `inputs_schema` 並列）。
- `crates/runtime/src/catalogue/schemas.rs` 新增 37 個 `<op>_outputs() -> Value` 函式：
  - `project_status_outputs()` → operations.md §1389 完整 schema（七個 required field）。
  - `change_show_outputs()` → 既有 envelope shape（`change` / `artifacts`）+ 新 `all_tasks_done` / `next_actions` 兩欄位。
  - 其餘 35 個 op 給 `empty_object_schema()` stub（之後對應 SDD slice 真做時補完整 outputs schema，跟 inputs 同套迭代節奏）。
- catalogue/mod.rs 37 個 Operation entry 全加 `outputs_schema: schemas::<op>_outputs`（編譯器強制 — 漏一個就 fail）。
- `crates/runtime/src/tool_ops/render.rs::JsonRenderer` 加 `outputs_schema` 鍵在 `parameters` 旁同層；`CopilotSdkRenderer` 不動（AI tool function-call convention inputs-only）。
- 既有 13 個 catalogue unit test 補 `outputs_schema` 非 panic + project_status / change_show outputs 內容斷言。
- 4 個 describe-tools insta snapshot 因 JSON 形狀改變需重 generate；snapshot review 確認改動只在 `outputs_schema` 新欄位。

**Alternatives**：
- A 方案（spec 收掉 outputs 要求）：rejected — 失去 catalogue 雙向自描述價值。
- C 方案（fn 回 `{inputs_schema, outputs_schema}` wrapper）：rejected — 欄位名 `inputs_schema` 卻含 outputs 不誠實、編譯器無法強制每 op 填 outputs、與 Rust idiom 與 `audit: true` 衝突。

**對既有 `inputs_schema` 函式 / Operation entry inputs schema 內容的影響**：零。本 Decision 只加新欄位、新函式、新 JSON key，既有任何 op 的 inputs 一個字不改。

**對 Phase 2/3 後續 slice 影響**：每個 op 真實實作時，要同時補 inputs + outputs schema 兩條。catalogue 雙向自描述成為 phase 慣例。

## Implementation Contract

### Behavior

- `speclink status` 印 envelope `{ok, data, warnings, requestId}`；`data` 對齊 operations.md §1389 七欄位（`provider_type` / `project_id` / `working_dir` / `current_change?` / `changes_count` / `discussions_count` / `schema_active`）。
- `speclink status` 在非 SpecLink 專案目錄（無 link.yaml）→ exit 2，error code `project.not_initialized`。
- `speclink show change <name>` envelope `data` 既有欄位（`change`、`artifacts`）保留不動；新增 `all_tasks_done: bool` + `next_actions: [string]` 兩個欄位、位於 `data` 頂層。
- `speclink status` 與 `speclink show change` 皆 read-only，不寫 audit event、不取 lock。

### Interface / data shape

- Runtime：`crate::project_ops::project_status(working_dir: &Path) -> Result<ProjectStatusData, RuntimeError>` 回 struct，欄位 1:1 對應 envelope schema。
- Runtime：`crate::change_ops::ShowChangeData` 加兩欄位 `all_tasks_done: bool` 與 `next_actions: Vec<String>`，serde 預設 snake_case rename 維持。
- CLI：`Commands::Status { json: bool }`（既有 `--json` 全域 flag 已存在，subcommand level 不重覆）。
- catalogue：`Operation` struct 加 `outputs_schema: fn() -> Value` 欄位；37 個 op 全填 outputs fn ptr（project_status / change_show 真實 schema、其餘 35 stub）。
- catalogue/render：`JsonRenderer` 加 `outputs_schema` 鍵；`CopilotSdkRenderer` 不動。

### Command output（envelope shapes）

`speclink status --json` 成功：
```json
{
  "ok": true,
  "data": {
    "provider_type": "local",
    "project_id": "speclink",
    "working_dir": "/abs/path",
    "current_change": null,
    "changes_count": { "proposing": 0, "reviewing": 0, "ready": 0, "in_progress": 0, "code_reviewing": 0, "archived": 8 },
    "discussions_count": { "active": 0, "converged": 0 },
    "schema_active": "spec-driven"
  },
  "warnings": [],
  "requestId": "<uuid>"
}
```

`speclink show change <name> --json` 成功：
```json
{
  "ok": true,
  "data": {
    "change": { "change_id": "...", "name": "...", "state": "in_progress", "schema_id": "...", "version": 3, "created_at": "...", "updated_at": "..." },
    "artifacts": [{ "kind": "proposal", "capability": null }, ...],
    "all_tasks_done": false,
    "next_actions": ["task.done 2"]
  },
  "warnings": [],
  "requestId": "<uuid>"
}
```

### Failure modes

- `speclink status` 在非 SpecLink 目錄 → `project.not_initialized` → exit 2。
- `speclink status` 在 link.yaml malformed → `link.malformed` → exit 2（既有 error）。
- `speclink show change <unknown>` → `change.not_found` → exit 2（既有，不動）。
- 任何 internal panic 或 state.db 損壞 → bubble up 既有 `RuntimeError::Internal` / state.db 相關 variant → exit 1。
- 本 slice 不新增 error variant、不新增 exit code mapping。

### Acceptance criteria

- 全部 cargo test 綠（含新增 unit test + integration test + insta snapshot）。
- `speclink describe-tools --filter project.status --json` 印 catalogue entry 含完整 inputs/outputs schema（不再是 stub）。
- `speclink status` 在 SpecLink 專案根目錄印 envelope 含正確 counts；在非 SpecLink 目錄 exit 2 + `project.not_initialized`。
- `speclink show change add-tool-describe-and-catalogue` envelope 含 `all_tasks_done: true` 與 `next_actions: []`（此 change 已 archived）。
- `speclink show change <in_progress 的 change>` 在 `all_tasks_done=false` 時 `next_actions` 為 `["task.done <id>"]`，`id` 為 tasks.md 第一個 `- [ ]` 行的 task 編號。
- Integration test 涵蓋 6 個 state × 2 個 all_tasks_done 邊角共 7 個有意義組合的 next_actions 對應。
- catalogue doc sync test 仍綠（operations.md §Index 表不需動）。

### Scope boundaries

**In scope**：
- `project.status` op + `speclink status` CLI（含 `--json` 與 human YAML 兩路徑）。
- `change.show` envelope `all_tasks_done` + `next_actions` 兩欄位。
- catalogue Operation struct 擴 `outputs_schema` 欄位、37 個 op 全填 outputs fn ptr（B 方案）；JsonRenderer 加 outputs key。
- Runtime 新模組 `project_ops`、既有 `change_ops::show_change` 擴充。

**Out of scope**：
- discussion store / discussion 計數實作（P2 #1）。
- artifact DAG ready 狀態計算（P1-3）。
- Dashboard table renderer。
- 新 provider trait method。
- HttpProvider 對接（P-future）。
- Audit event 寫入（read-only op 不需）。
- Lock 機制（read-only op 不取 lock）。

## Risks / Trade-offs

- [tasks.md parser 與既有 task_ops 重複] → Mitigation: 若 task_ops 已抽 `parse_first_pending_task_id` 函式則 import；否則本 slice 寫小 parser，並在 implementation tasks 顯式註明這項決策避免後續 refactor 偷偷重複。

- [discussions_count 永遠 0 可能誤導使用者] → Mitigation: spec 與 design 都明文「P2 deferred」；YAML 輸出可考慮在欄位後加 `# pending P2` 註解，但本 slice 不做（避免破 YAML schema），純文件揭露即可。

- [current_change instance_id 比對失敗的邊角] → Mitigation: instance_id rotation 規則已在 design.md §13.6 明寫；若該 change 是其他 host 寫的（少見、single-RD MVP），current_change 為 null 並無功能影響。

- [catalogue schema 升級可能破 describe-tools insta snapshot] → Mitigation: implementation 階段 explicit `INSTA_UPDATE=always` 後 review snapshot diff，確認只變動 project.status / change.show entry。

## Migration Plan

- state.db schema 不動、CLI subcommand 新增、`show change` envelope 兩欄位純 additive。
- catalogue Operation struct **加新欄位**（B 方案）— 對 catalogue crate 的外部 caller 屬 breaking（pattern-match 需更新）；目前 catalogue 僅 cli/runtime 自用、無外部 consumer，影響 contained。
- 既有 `speclink show change` consumer 自動拿到兩新欄位；不需要 forced version bump。
- 部署：merge → next release。

## Open Questions

無 — 範圍與行為已在 propose discuss 階段確認。
