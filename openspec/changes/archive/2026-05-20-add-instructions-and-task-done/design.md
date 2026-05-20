## Context

Change 2 與 Change 3 已釘住 local provider 的「寫 artifact + 查 status + archive」三條路徑；本 change 補上「告訴 AI 怎麼寫」與「把 task 標完成」兩條。完成後 SpecLink 的 local-only AI workflow 指令集達到自給自足：propose create → instructions → artifact write → status → archive，配合 task done 收尾 apply phase。

設計約束（自前三個 change 繼承，不重述）：
1. Provider trait 保 `Send + Sync + dyn-compatible`、`async_trait`
2. lib crate 用 thiserror、bin crate 用 anyhow
3. JSON envelope、exit code、error code naming 不變
4. Atomic write 策略沿用 .tmp + rename + cleanup
5. metadata.json source of truth、SQLite 為 fast index
6. spec_delta 演算法已在 runtime crate 證明 runtime 可容納 algorithm-only 模組（無 I/O）

## Goals / Non-Goals

**Goals:**

- 新增 `Provider::get_artifact_instructions` 與 `Provider::mark_task_done` trait methods
- 新增 CLI 指令 `speclink instructions <artifact>` 與 `speclink task done <task-id>`
- 釘住 4 種 ArtifactKind 的 hardcoded instructions 內容存放位置與載入方式（`include_str!`）
- 釘住 tasks.md 的 task id 格式（`N.M`）、checkbox 解析規則、原子更新策略
- 釘住 instructions JSON output schema（與 design doc 提到的 `template` / `rules` / `instruction` 對齊）
- 釘住 task done 的 idempotent 行為

**Non-Goals:**

- 不引入 schema 抽象（如 Spectra 的 `spec-driven` schema） — instructions 對 ArtifactKind 一對一
- 不解析 task 的 `[P]` parallel marker — 留給未來 `apply start` 指令
- 不在 task done 寫 timestamp 或 actor 到 tasks.md / metadata.json
- 不引入「task 解析失敗時自動修復」邏輯 — 解析失敗一律 error，由 AI / 人類介入
- 不變更既有 4 條 AI workflow 指令的行為
- 不為 instructions 加入 i18n（locale 欄位回固定值 `"Traditional Chinese (繁體中文)"`，與 `.spectra.yaml` 既有 `locale: tw` 對齊但不引入動態切換）

## Decisions

### Hardcoded instructions 內容以 `include_str!` 編入 runtime binary

`crates/runtime/instructions/{proposal,design,tasks,spec}.md` 四個檔案，內容為 markdown，runtime 透過 `include_str!` 在 compile time 嵌入。

```rust
pub static PROPOSAL_INSTRUCTION: &str = include_str!("../instructions/proposal.md");
pub static DESIGN_INSTRUCTION: &str = include_str!("../instructions/design.md");
pub static TASKS_INSTRUCTION: &str = include_str!("../instructions/tasks.md");
pub static SPEC_INSTRUCTION: &str = include_str!("../instructions/spec.md");
```

**內容範圍**（每個檔案約 30–80 行）：
- `instruction`：純文字段落描述要寫什麼、思考順序、邊界
- `template`：對應 artifact 的 markdown skeleton（與 Spectra 的 schema template 相似）
- `rules`：每個 artifact 的硬性規則（如 spec 必含 `### Requirement:`、tasks 必含 checkbox）
- `dependencies` / `unlocks` / `outputPath`：由 runtime 程式計算（不放 markdown 內）

**理由**：避免引入 schema 設定檔 → 第一版內容隨 CLI 一起釋出；改 instructions 即改程式碼，PR review 同行可見。

**替代方案**：
- **`.speclink/instructions/<kind>.md` 從 filesystem 載入**：拒絕原因，第一版內容需要與 CLI 版本同步演化；外部檔案造成「使用者修改 instructions 但忘了升 CLI」失同步。
- **由 provider 動態回傳（HTTP provider 可遠端管理）**：拒絕原因，local provider 自己沒有 backend 可動態管理；HTTP provider 落地時再讓 trait method 允許 provider 端動態回傳，trait 簽章已支援。
- **用 RON / TOML 結構化檔**：拒絕原因，instruction 與 template 都是長段 markdown，TOML escape 痛苦；純 markdown 最自然。

呼叫 `rust-skills:m11-ecosystem` 確認 `include_str!` 對跨平台 build 無副作用。

### `Provider::get_artifact_instructions` 簽章

```rust
async fn get_artifact_instructions(
    &self,
    project_id: &ProjectId,
    change_id: &ChangeId,
    kind: ArtifactKind,
    capability: Option<&str>,
) -> Result<ArtifactInstructions, ProviderError>;
```

```rust
pub struct ArtifactInstructions {
    pub artifact_id: String,         // "proposal" | "design" | "tasks" | format!("spec:{cap}")
    pub kind: ArtifactKind,
    pub output_path: String,         // POSIX 相對於 base
    pub dependencies: Vec<String>,   // artifact id 字串
    pub unlocks: Vec<String>,        // artifact id 字串
    pub instruction: String,         // 主敘述段
    pub template: String,            // markdown skeleton
    pub rules: Vec<InstructionRule>,
    pub locale: String,              // "Traditional Chinese (繁體中文)"
}

pub struct InstructionRule {
    pub id: String,                  // "proposal.must_include_why"
    pub level: RuleLevel,            // Error | Warning | Info
    pub description: String,
}
```

`capability` 只在 `kind == Spec` 時使用，計算 `output_path` 為 `.speclink/changes/<id>/specs/<cap>/spec.md`；其他 kind 傳 `None`。

**理由**：JSON shape 與 design doc `GET /v1/projects/{projectId}/changes/{changeId}/instructions/{artifactId}` 對齊；HTTP provider 未來落地不需要破壞性改 trait。

**替代方案**：
- **`kind: &str` 而非 `ArtifactKind`**：拒絕原因，typed enum 防呆。
- **`InstructionRule` 第一版只回 `description: String`**：拒絕原因，`id` 與 `level` 屬於穩定契約欄位（spec.md 點分隔 code 命名規則一致）；省略後續加會破壞 JSON shape。

### `Provider::mark_task_done` 簽章與 idempotent 行為

```rust
async fn mark_task_done(
    &self,
    project_id: &ProjectId,
    change_id: &ChangeId,
    task_id: &str,
) -> Result<TaskUpdate, ProviderError>;
```

```rust
pub struct TaskUpdate {
    pub task_id: String,
    pub previous_status: TaskStatus,  // 改動前狀態
    pub current_status: TaskStatus,   // 改動後狀態（恆為 Done）
    pub task_description: String,     // 完整描述（含 [P] marker、不含 checkbox）
}

pub enum TaskStatus { Todo, Done }
```

idempotent 規則：

- 找不到 `N.M` 對應的 checkbox → `ProviderError::TaskNotFound { task_id }`，exit code 2
- 找到但已是 `- [x]` → 回 success，`previous_status = Done`、`current_status = Done`、`task_description` 為當前 description
- 找到且為 `- [ ]` → 改為 `- [x]`，原子寫入 tasks.md；回 success，`previous_status = Todo`、`current_status = Done`

**理由**：AI skill 在 apply 階段可能因 retry / restart 而重複呼叫 `task done`；idempotent 比每次都報「已完成」錯誤友善。

**替代方案**：
- **已完成回 error code `task.already_done`**：拒絕原因，這對 AI skill 處理迴圈造成 false-positive 失敗；idempotent 更乾淨。
- **更新時順帶填 timestamp**：拒絕原因，tasks.md 是 git-tracked 文件，git history 已含時間；多寫 timestamp 造成 diff 噪音。

### tasks.md 的 task id 格式：`N.M`

task id 為 `<section>.<index>` 形式，對應 tasks.md 中的：

```markdown
## N. <Section heading>

- [ ] N.M <Task description>
```

`N` 與 `M` 皆為十進位正整數（無前導 0、無 `-` 後綴）；`N` 為 section 編號（與 `## N.` heading 開頭數字一致）、`M` 為 section 內 task 序號（與 `- [ ] N.M` 開頭數字一致）。

格式不接受 `N.M.P`（三層）；本 change 不支援巢狀任務。

**替代方案**：
- **用全局唯一 task id**（如 UUID）：拒絕原因，tasks.md 是 markdown 文件，UUID 對人類不友善。
- **支援 `N.M.P` 三層**：拒絕原因，巢狀 task 在 Spectra 也很少見，YAGNI。

### tasks.md 原子更新策略

`crates/provider-local/src/storage.rs` 新增 `update_tasks_atomic(base, change_id, new_content)`：

1. 計算 `<change_dir>/tasks.md` 路徑
2. 寫 `tasks.md.tmp`
3. rename `tasks.md.tmp` → `tasks.md`
4. 失敗時 cleanup `.tmp`

與 Change 2 的 atomic write 不同的是：本函式假設 `tasks.md` 已存在且為 source content；caller（`mark_task_done` 邏輯）先讀整檔 → 在記憶體中修改 → 呼叫 `update_tasks_atomic` 寫回。

**為何不直接 in-place 寫**：rename 提供 atomicity（同 filesystem 下 atomic 換檔）；若 process crash 在寫 `.tmp` 中途，正式檔仍是舊內容，避免 tasks.md 損毀。

**替代方案**：
- **`std::fs::write` 直接覆寫**：拒絕原因，崩潰時可能留下 partial write，破壞 tasks.md。

### tasks.md 解析在 `crates/runtime/src/tasks_parser.rs`

純 markdown parser，無 I/O，與 spec_delta 同層級。

```rust
pub struct ParsedTasks {
    pub sections: Vec<TaskSection>,
}

pub struct TaskSection {
    pub number: u32,
    pub heading: String,
    pub tasks: Vec<TaskItem>,
}

pub struct TaskItem {
    pub task_id: String,        // "1.1"
    pub status: TaskStatus,
    pub description: String,    // 不含 checkbox、不含 task id 前綴
    pub line_number: usize,     // 原檔行號（1-based），供 update 定位
}

pub fn parse_tasks(content: &str) -> Result<ParsedTasks, TasksParseError>;
pub fn mark_task_done_in_content(content: &str, task_id: &str) -> Result<UpdateResult, TasksUpdateError>;

pub struct UpdateResult {
    pub new_content: String,
    pub previous_status: TaskStatus,
    pub task_description: String,
}
```

`mark_task_done_in_content` 在純字串層級操作 — 找對應行、改 `[ ]` 為 `[x]`、不動其他行。實作上以 line-by-line scanner 偵測 section heading（`^## (\d+)\. `）建立當前 section 編號 → 偵測 checkbox（`^- \[( |x)\] (\d+)\.(\d+) `）匹配 task id。

**替代方案**：
- **用 pulldown-cmark crate 完整 markdown AST**：拒絕原因，正則 + line scan 足以，依賴項可避免。
- **把 parser 放 provider-local**：拒絕原因，演算法可被 HTTP provider 復用，放 runtime 更合適。

### CLI `speclink instructions <artifact>` 子命令採 positional `<artifact>`

```bash
speclink instructions proposal --change <id> [--json]
speclink instructions design   --change <id> [--json]
speclink instructions tasks    --change <id> [--json]
speclink instructions spec     --change <id> --capability <name> [--json]
```

clap structure：

```rust
pub enum InstructionsCommand {
    Proposal(InstructionsArgs),
    Design(InstructionsArgs),
    Tasks(InstructionsArgs),
    Spec(InstructionsSpecArgs),
}
```

`spec` 需 `--capability`（與 artifact write spec 一致 surface）；其他 kind 不可帶 `--capability`，否則 clap 拒絕。

**替代方案**：
- **`speclink instructions --kind design`**：拒絕原因，與 artifact write 採 positional 一致較對稱。

### CLI `speclink task done <task-id>` 子命令

```bash
speclink task done <task-id> --change <id> [--json]
```

`<task-id>` 為 positional。`change` 為 `--change <id>` flag（與 status / artifact write 一致）。clap 子命令層級：

```rust
pub enum TaskCommand {
    Done(TaskDoneArgs),
}

pub struct TaskDoneArgs {
    pub task_id: String,
    pub change: String,
    pub flags: MachineInterfaceFlags,
}
```

未來可加 `Undone` / `List` 等子命令；本 change 只實作 `Done`。

**替代方案**：
- **`speclink task-done`（hyphenated 而非 nested）**：拒絕原因，task 是一個 namespace，未來會有 `task list` 等。

### Error code 新增清單

| 新 error code | 觸發條件 | exit code |
|---|---|---|
| `instructions.unsupported_kind` | （dead branch，clap 已擋 — 但 trait method 防禦） | 2 |
| `task.not_found` | tasks.md 中找不到對應 `N.M` checkbox | 2 |
| `task.invalid_id` | task id 格式不符 `\d+\.\d+` | 2 |
| `tasks.parse_error` | tasks.md 解析失敗（如缺 section heading 但有 checkbox） | 1 |
| `artifact.missing` | tasks.md 不存在（task done 對沒寫過 tasks 的 change 呼叫） | 1 |

`task.not_found` 與 `task.invalid_id` 對應 exit code 2（user input error）；`tasks.parse_error` 與 `artifact.missing` 為 1（一般錯誤）。

**替代方案**：
- **`artifact.missing` 用既有 `change.not_found`**：拒絕原因，change 存在但 tasks.md 不存在是不同情境；分開 code 讓 AI skill 處理更準確。

## Implementation Contract

**Observable behavior**：

**Instructions**：執行 `speclink instructions design --change demo --json`（假設 demo 為 active change）：

1. CLI 載入 runtime 內 hardcoded design instruction
2. stdout 印一行 JSON：

```json
{
  "ok": true,
  "data": {
    "artifactId": "design",
    "kind": "design",
    "outputPath": ".speclink/changes/demo/design.md",
    "dependencies": ["proposal"],
    "unlocks": ["tasks"],
    "instruction": "Create the design document that explains HOW to implement the change. ...",
    "template": "## Context\n\n...",
    "rules": [
      {"id": "design.must_include_context", "level": "error", "description": "Design must include Context section."}
    ],
    "locale": "Traditional Chinese (繁體中文)"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

3. exit code = 0

**Task done**（idempotent）：執行 `speclink task done 5.2 --change demo --json`（假設 tasks.md 含 `- [ ] 5.2 Wire up SSO endpoint`）：

1. 讀 `.speclink/changes/demo/tasks.md`
2. 在記憶體中將 `- [ ] 5.2 ...` 改為 `- [x] 5.2 ...`
3. 原子寫回 tasks.md
4. stdout 印 JSON：

```json
{
  "ok": true,
  "data": {
    "changeId": "demo",
    "taskId": "5.2",
    "previousStatus": "todo",
    "currentStatus": "done",
    "taskDescription": "Wire up SSO endpoint"
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

5. exit code = 0

再次執行同 `task done 5.2`：`previousStatus = "done"`、`currentStatus = "done"`、exit code 0。

**Interface（命名）**：

- clap 結構：
  - `Cli::Instructions(InstructionsCommand)`、4 種 subcommand
  - `Cli::Task(TaskCommand)`、`TaskCommand::Done(TaskDoneArgs)`
- runtime 入口：
  - `crates/runtime/src/instructions.rs::get_instructions(provider, input) -> Result<ArtifactInstructions, RuntimeError>`
  - `crates/runtime/src/instructions.rs::compose_local_instructions(kind, change_id, capability) -> ArtifactInstructions`（pure helper，供 local provider 使用）
  - `crates/runtime/src/tasks_parser.rs::{parse_tasks, mark_task_done_in_content}`
- provider trait method：兩個新 method
- output 型別：`crates/cli/src/output.rs::InstructionsData`、`TaskDoneData`

**Failure modes**：

| 觸發條件 | error code | exit code |
|---|---|---|
| change 不存在 | `change.not_found` | 1 |
| spec kind 缺 `--capability` | `artifact.missing_capability` | 2 |
| capability 名稱非法 | `artifact.invalid_capability` | 2 |
| design / tasks / proposal 帶 `--capability` | `input.invalid` | 2 |
| task id 格式不符 | `task.invalid_id` | 2 |
| task id 不存在於 tasks.md | `task.not_found` | 2 |
| tasks.md 不存在 | `artifact.missing` | 1 |
| tasks.md 解析失敗 | `tasks.parse_error` | 1 |
| filesystem 失敗 | `internal.error` | 1 |

stderr：與既有規則一致。

**Acceptance criteria**：

實作後測試通過：

1. `cargo build --workspace` 三平台無 warning
2. `cargo fmt --check`、`cargo clippy --workspace -- -D warnings`、`cargo test --workspace` 全綠
3. 含測試：
   - `crates/runtime/src/instructions.rs` 單元測試：4 種 kind 的 `compose_local_instructions` 回傳結果非空 instruction / template / 至少 1 條 rule、`output_path` 正確、spec kind 含 capability 反映在路徑與 artifact id
   - `crates/runtime/src/tasks_parser.rs` 單元測試：`parse_tasks` 涵蓋 happy path、missing section heading + checkbox 報 error、`mark_task_done_in_content` happy / idempotent / not_found / invalid_id
   - `crates/provider-local/tests/instructions_integration.rs` end-to-end：`get_artifact_instructions(Design, None)` 回傳預期 instruction string；`get_artifact_instructions(Spec, Some("auth"))` 回傳含 `spec:auth` artifact_id
   - `crates/provider-local/tests/task_done_integration.rs` end-to-end：propose create → artifact write tasks（內含 `- [ ] 1.1 Write tests`） → mark_task_done("1.1") → 驗證 tasks.md 已更新為 `- [x]`；再次呼叫同 task id idempotent
   - `crates/cli/tests/instructions.rs` 4 種 invocation + clap 拒絕路徑
   - `crates/cli/tests/task_done.rs` happy / idempotent / not_found / invalid_id / missing tasks.md
   - insta snapshot：4 種 instructions（design / spec）+ task done success + already done + not_found
4. 手動驗證：完整跑 propose create → instructions proposal → artifact write proposal → instructions design → artifact write design → instructions tasks → artifact write tasks → task done 1.1 → status → archive 一輪
5. JSON output 不含 secret

**Scope boundaries**：

- **In scope**：本 design 涵蓋的 trait 變更、CLI 子命令、3 個 spec、instructions 4 份 markdown 檔、tasks_parser module
- **Out of scope**：apply state machine、`[P]` parallel marker 解析、analyze、validate、HTTP provider 的兩個 method 實作（簽章已釘，實作後續）、auth、pack、unpack、finish

## Risks / Trade-offs

- **[Hardcoded instructions 內容偏離真實 dogfood 需求]** Mitigation：本 change 完成後即在下一個 change（自身或某個演示 change）dogfood 驗證 instructions 內容；發現偏差可隨時用 follow-up change 修 instruction 檔內容（影響面只在 markdown 字串）。
- **[`include_str!` 對 cross-platform line ending]** Mitigation：所有 instruction .md 檔 commit 時統一 LF；`.gitattributes` 已（或需）設定 `*.md text eol=lf`。本 change 加 `.gitattributes` 規則。
- **[tasks.md parser 對 unusual markdown 不寬容]** Mitigation：規範要求嚴格（`^## N\. `、`^- \[[ x]\] N\.M `）— 與 propose 階段 spectra-propose skill 產生的 tasks.md 對齊；若使用者手寫 task 偏離格式回 `tasks.parse_error`，error message 含被 reject 的 line。
- **[idempotent task done 對 AI skill retry 友善但對人類可能誤導]** Mitigation：JSON output 透過 `previousStatus` 與 `currentStatus` 兩欄位區分「剛改 vs 早已完成」；人類在 stdout 也可看出（雖然 stderr human-readable 輸出本 change 不做特別 highlight）。
- **[instructions JSON 體積較大（含 template + instruction 全文）]** Mitigation：MVP 內容估計每個 instruction < 8 KB；stdout 單行 JSON 雖長但對 AI skill 可處理。
- **[Provider trait 兩個新 method 對 HTTP provider 將來實作的負擔]** Mitigation：HTTP provider 已可走 design doc `GET .../instructions/{artifactId}` 與（未來定義）`PATCH .../tasks/{taskId}` 兩條 endpoint；trait 簽章已預留 capability 與 task_id 參數。

## Migration Plan

N/A — 本 change 純新增。對 bootstrap / Change 2 / Change 3 期間建立的 change（含 tasks.md）直接適用：

- `mark_task_done` 對既有 tasks.md 無 schema 限制（只要符合 `N.M` 格式）
- `get_artifact_instructions` 對任何 change 都回硬編內容，與 change 既有檔案無依賴

## Open Questions

- **是否支援同時標多個 task done（如 `speclink task done 1.1 1.2 2.3`）？** 本 change 限定一次一個 task id；批次操作可寫 shell loop 處理。若 dogfood 後發現 AI 頻繁需要批次，再加 `--many` 或位置參數變 vec。
- **instructions 內容是否需要包含「Why this artifact exists」的高層說明？** 本 change 把 instruction 段定位為「How to write it」；高層介紹由 propose 階段的 AI agent 自然帶出，不重複放 instructions。
- **`speclink task done` 是否該觸發 metadata.json state 從 `proposed` → `in_progress`？** 本 change 不做此轉換（Non-Goals 第 9 條）。若未來引入 `in_progress` state，第一個 task done 是合理觸發點。
- **`.gitattributes` 加 `*.md text eol=lf` 是否會影響使用者既有 markdown？** 本 change 只規範 repo 內檔案；對 SpecLink CLI 行為無影響。若使用者 repo 不需強制 LF，可移除此設定。本 change 暫定加入。
