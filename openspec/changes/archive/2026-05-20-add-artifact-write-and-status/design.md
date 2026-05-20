## Context

SpecLink 在 bootstrap change 已釘死 4 crate 結構、Provider async trait（3 個 method）、JSON envelope schema 與 exit code 表。本 change 是第二個 vertical slice：把 `Provider::write_artifact` 從只支援 `Proposal` 擴張到 `Design`、`Tasks`、`Spec` 三類；同時新增 `Provider::get_status` 讓 CLI 能回報 change 進度。

設計約束（不變式，從 bootstrap change 繼承）：
1. `Provider` trait 仍為 `Send + Sync + dyn-compatible`，所有 method async
2. lib crate（provider、provider-local、runtime）用 thiserror，bin crate（cli）用 anyhow
3. JSON envelope schema 不動（`ok` / `data` / `warnings` / `error` / `requestId`）
4. Exit code 表不動（0 / 1 / 2 / 5 / 6 已釘）
5. Error code 命名規則不動（`<capability>.<short_snake_case>`）
6. Local provider 沿用「atomic write via temp file + rename + cleanup on failure」策略
7. metadata.json 是 source of truth、SQLite `in_progress_change` 是 fast index（雙寫）

## Goals / Non-Goals

**Goals:**

- 新增 `Provider::get_status` trait method，回傳 `ChangeStatus`（artifact list + 每個 artifact 的狀態）
- 擴張 `Provider::write_artifact` 支援 `ArtifactKind::{Design, Tasks, Spec}`，sec 須帶 capability 名稱
- 新增 CLI 指令 `speclink artifact write <kind>` 與 `speclink status`
- 釘住 `ChangeStatus` 與 `ArtifactStatus` 的 JSON shape（後續 `instructions` / `analyze` / `validate` 都要相容）
- 釘住 spec artifact 的 `--capability <name>` 路由規則與檔案路徑：`.speclink/changes/<id>/specs/<capability>/spec.md`
- 釘住 design / tasks 的單一檔案位置：`.speclink/changes/<id>/{design.md, tasks.md}`
- 沿用 bootstrap 的測試規格（assert_cmd 整合測試 + insta snapshot + tempfile 隔離）

**Non-Goals:**

- 不釘 archive 階段的 spec sync（delta merge）— 留給 Change 3
- 不釘 lifecycle state 在 `proposed` 之後的所有轉換 — 本 change 只引入 `proposed` 一個 terminal state（不推進）
- 不解析 tasks.md 的 checkbox 進度 — status 的 task-level 進度延後到 Change 4
- 不為 spec artifact 做 delta heading 解析（`## ADDED / MODIFIED / REMOVED` 寫入時當 plain markdown 對待）
- 不引入 `instructions` 指令或 per-artifact guidance 內容
- 不引入 `task done` 指令
- 不在 SQLite 增加新表 — `get_status` 純掃描 filesystem + 讀 metadata.json
- 不變更既有 `create_change` / `write_artifact`（Proposal）/ `get_change` 簽章

## Decisions

### `Provider::get_status` 加入 trait 而非僅 provider-local

`crates/provider/src/lib.rs` 的 `Provider` trait 新增：

```rust
async fn get_status(&self, project_id: &ProjectId, change_id: &ChangeId) -> Result<ChangeStatus, ProviderError>;
```

理由：HTTP provider 將來必有 `GET /v1/projects/{projectId}/changes/{changeId}/status`（已在 design doc 釘住）；trait 第一版就放好對應 method，避免 HTTP provider 落地時破壞性改 trait。

**替代方案**：
- **只在 LocalProvider 加 `get_status` 自己的 inherent method**：拒絕原因，CLI dispatch 會需要做型別 down-cast，違反 `Arc<dyn Provider>` 的抽象。
- **將 status 計算放 runtime，trait 只暴露原始 artifact list**：拒絕原因，HTTP provider 的 status 是 server 端計算（server 端可能根據 schema 加入 dependency-aware 的 status），不應由 client 重算。

### `ChangeStatus` 採固定 typed struct 而非 generic JSON

`crates/provider/src/model.rs` 新增：

```rust
pub struct ChangeStatus {
    pub change_id: ChangeId,
    pub state: State,
    pub artifacts: Vec<ArtifactStatus>,
}

pub struct ArtifactStatus {
    pub id: ArtifactStatusId,    // "proposal" | "design" | "tasks" | format!("spec:{capability}")
    pub kind: ArtifactKind,
    pub path: String,             // POSIX 風格相對於 base
    pub status: ArtifactState,    // Missing | Done
    pub required: bool,           // proposal 與 spec 為 required
    pub dependencies: Vec<String>, // 對應其他 artifact id
}

pub enum ArtifactState { Missing, Done }
```

理由：CLI JSON output 需要穩定 schema；HTTP provider 的 server 端要回傳同 schema；strongly typed 比 `serde_json::Value` 安全。

`ArtifactState` 第一版只有 `Missing` 與 `Done` 兩態 — `Ready` / `Blocked` 等 dependency-aware 狀態屬於 runtime / instructions 範疇，本 change 不引入。

**替代方案**：
- **`ArtifactState` 第一版加入 `Ready` / `Blocked`**：拒絕原因，dependency resolution 需要 schema 知識（哪個 artifact 依賴哪個），屬於 instructions capability；本 change 不做 instructions。
- **spec artifact id 用 `String` 而非帶 capability 前綴**：拒絕原因，同 change 內可能有多個 capability spec（如 bootstrap 寫了 4 個），id 必須足以區分。

### Spec artifact 路徑：`.speclink/changes/<id>/specs/<capability>/spec.md`

`speclink artifact write spec --change <id> --capability <name> --stdin` 寫入 `.speclink/changes/<id>/specs/<name>/spec.md`。

`<name>` 規則：與 change-id 共用 kebab-case 規則（`^[a-z][a-z0-9-]{0,63}$`、無連續 hyphen、不以 hyphen 結尾），由 `provider-local::storage::is_valid_capability_name` 驗證（與 `is_valid_change_id` 相同實作）。

**替代方案**：
- **將 spec 寫成單一 specs.md 列出所有 capability**：拒絕原因，與 Spectra / OpenSpec 慣例不符；多 capability 在同檔內混淆 diff review。
- **取消 `--capability` 旗標，從 stdin 內容的第一行 heading 推斷**：拒絕原因，stdin 內容應為純 spec 內文，metadata 不該與內容耦合。

### `speclink artifact write` 子命令採 positional `<kind>` 而非 flag

```bash
speclink artifact write design --change <id> --stdin --json
speclink artifact write tasks --change <id> --stdin --json
speclink artifact write spec  --change <id> --capability <name> --stdin --json
```

clap structure：

```rust
pub enum ArtifactCommand {
    Write(ArtifactWriteCommand),
}

pub enum ArtifactWriteCommand {
    Design(ArtifactWriteArgs),
    Tasks(ArtifactWriteArgs),
    Spec(ArtifactWriteSpecArgs),
}
```

`ArtifactWriteSpecArgs` 含 `capability: String`；其餘共用 `ArtifactWriteArgs { change, flags: MachineInterfaceFlags }`。

`--stdin` 為 required（與 bootstrap 的 `propose create` 不同 — `propose create` 禁用 stdin，本指令以 stdin 為主要輸入路徑）。stdin 為空（EOF 立刻）對應 `input.invalid` exit 2。

**替代方案**：
- **`speclink artifact write --kind design --change <id>`**：拒絕原因，clap derive 對 enum subcommand 比 enum flag 更 idiomatic，且 future kind 可加而不破壞既有 CLI。
- **每個 kind 一條 top-level 指令（`speclink design write`、`speclink tasks write`）**：拒絕原因，artifact 是統一概念，CLI 命名應反映；散開 top-level 會稀釋 surface。

### `speclink status` 的 JSON output schema

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusData {
    pub change_id: String,
    pub state: String,           // "proposed"
    pub artifacts: Vec<ArtifactStatusJson>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactStatusJson {
    pub id: String,              // "proposal" | "design" | "tasks" | "spec:<capability>"
    pub kind: String,            // "proposal" | "design" | "tasks" | "spec"
    pub path: String,            // POSIX 路徑
    pub status: String,          // "missing" | "done"
    pub required: bool,
    pub dependencies: Vec<String>,
}
```

`required` 規則（第一版固定）：`proposal` 為 true，`spec` 為 true（至少要有一個 capability spec），`design` / `tasks` 為 false。實際 `applyRequires` 邏輯由 instructions 指令在 Change 4 引入。

`dependencies` 規則（第一版固定）：
- `proposal`: `[]`
- `design`: `["proposal"]`
- `tasks`: `["proposal", "spec"]`
- `spec:<capability>`: `["proposal"]`

**替代方案**：
- **狀態用 enum 字串如 `ready` / `blocked`**：拒絕原因，dependency-aware status 屬 instructions；本版只給檔案在不在的事實。
- **包含 metadata.json 完整內容**：拒絕原因，metadata 屬 lifecycle，與 artifact DAG 分開；如要 metadata，用 `get_change`。

### `LocalProvider::write_artifact` 的 ArtifactKind 路由

`crates/provider-local/src/lib.rs::write_artifact` 改為：

```rust
async fn write_artifact(...) -> Result<Artifact, ProviderError> {
    let kind = input.kind;
    let content = input.content;
    let change_id = change_id.clone();
    let base = self.base_path.clone();
    // capability 由 caller 透過 NewArtifact 額外欄位帶入（見下）
    let capability = input.capability.clone();

    tokio::task::spawn_blocking(move || -> Result<PathBuf, LocalProviderError> {
        match kind {
            ArtifactKind::Proposal => write_proposal_content_atomic(&base, &change_id, &content),
            ArtifactKind::Design   => write_design_atomic(&base, &change_id, &content),
            ArtifactKind::Tasks    => write_tasks_atomic(&base, &change_id, &content),
            ArtifactKind::Spec     => {
                let cap = capability.ok_or(LocalProviderError::MissingCapability)?;
                write_spec_atomic(&base, &change_id, &cap, &content)
            }
        }
    }).await??;
    // ... 更新 state.db（若是 first-time write 才插入；後續 artifact write 不再 set_in_progress）
}
```

`NewArtifact` 新增 optional `capability: Option<String>` 欄位。當 `kind == Spec` 必填，否則必須為 `None`（trait 層校驗：傳遞錯誤組合回 `ProviderError::Internal { message: "capability required for spec kind" }`，CLI 在 clap layer 已先擋）。

**替代方案**：
- **將 `Spec` 變體本身帶 capability：`ArtifactKind::Spec(String)`**：拒絕原因，破壞 `ArtifactKind` 既有 PartialEq / 序列化簡潔性；JSON 序列化會變 tagged enum。
- **CLI 把 capability 編入 `content` 第一行**：拒絕原因，content/metadata 耦合，違反原子寫入清潔分離。

呼叫 `rust-skills:m05-type-driven` 確認 newtype 與 trait method 新增的最佳實踐。

### `metadata.json` 不為新 artifact 重寫

新 artifact 寫入時不更新 metadata.json 內容（其欄位仍為 `{changeId, state: "proposed", createdAt, createdBy}`）。

理由：lifecycle state 第一版只在 `propose create` 時寫入；後續 design / tasks / spec 寫入不改變 lifecycle state（仍為 `proposed`）。Archive change 才需要更新 state。

但 metadata.json 必須存在 — `write_artifact` 在 change dir 不存在時應該回 `ChangeNotFound`（既有錯誤），不主動建立。

**替代方案**：
- **每次寫 artifact 都更新 metadata.json 的 `lastModifiedAt`**：拒絕原因，YAGNI；無人讀此欄位。

### `get_status` 實作策略：純 filesystem scan + metadata.json 讀取

`LocalProvider::get_status` 流程：

1. 讀 `.speclink/changes/<id>/metadata.json` — 若不存在回 `ChangeNotFound`
2. 從 metadata.json 取出 `state`、`changeId`
3. 掃描 `<change_dir>/`：
   - `proposal.md` 存在 → ArtifactStatus { id: "proposal", kind: Proposal, status: Done }
   - 否則 → Missing
   - 同樣處理 `design.md`、`tasks.md`
4. 掃描 `<change_dir>/specs/`：每個子目錄含 `spec.md` 視為一個 spec artifact，id = `spec:<dirname>`

不查 SQLite — `in_progress_change` 只回答「目前正在做哪個 change」，不答「某個 change 內 artifact 完成度」。

**替代方案**：
- **將 artifact 完成度寫入 SQLite 新表**：拒絕原因，filesystem scan 在單 change 內 O(1)（檔案數 < 20），不需 index；SQLite 雙寫一致性風險不值得。

呼叫 `rust-skills:m11-ecosystem` 確認 std::fs 掃描 vs walkdir crate trade-off（本 change 不引入 walkdir，深度固定 ≤ 2）。

### Error code 新增清單

| 新 error code | 觸發條件 | exit code |
|---|---|---|
| `artifact.invalid_kind` | clap 層處理（不應命中，dead branch） | 2 |
| `artifact.missing_capability` | `artifact write spec` 缺 `--capability` | 2 |
| `artifact.invalid_capability` | capability 名稱不符 kebab-case | 2 |
| `artifact.already_exists` | 對應檔案已存在（不允許覆寫；後續 change 可開放） | 1 |
| `change.not_found` | `artifact write` / `status` 對不存在的 change | 1 |

`change.not_found` 已在 bootstrap 的 `ProviderError::ChangeNotFound` 釘住但未被任何指令觸發，本 change 首次觸發。

**替代方案**：
- **`artifact write` 直接覆寫既有檔案**：拒絕原因，AI skill 可能誤覆寫 — 預設拒絕較安全；後續可加 `--force`。
- **`change.not_found` 重用 `change.invalid_id`**：拒絕原因，「不存在」與「id 非法」是兩種錯誤，AI skill 處理路徑不同。

## Implementation Contract

**Observable behavior**：

執行 `echo "design content" | speclink artifact write design --change add-feature --stdin --json` 後（假設 `.speclink/changes/add-feature/` 已由 `propose create` 建立）：

1. `.speclink/changes/add-feature/design.md` 寫入內容 `design content\n`（trailing newline 補齊）
2. stdout 印一行 JSON：

```json
{"ok":true,"data":{"changeId":"add-feature","artifactId":"design","kind":"design","path":".speclink/changes/add-feature/design.md","mode":"local"},"warnings":[],"error":null,"requestId":"req_..."}
```

3. process exit code = 0

執行 `echo "spec content" | speclink artifact write spec --change add-feature --capability user-auth --stdin --json` 後：

1. `.speclink/changes/add-feature/specs/user-auth/spec.md` 寫入
2. stdout JSON 中 `artifactId = "spec:user-auth"`、`kind = "spec"`、`path` 為對應路徑
3. exit code = 0

執行 `speclink status --change add-feature --json` 後：

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "state": "proposed",
    "artifacts": [
      {"id":"proposal","kind":"proposal","path":".speclink/changes/add-feature/proposal.md","status":"done","required":true,"dependencies":[]},
      {"id":"design","kind":"design","path":".speclink/changes/add-feature/design.md","status":"done","required":false,"dependencies":["proposal"]},
      {"id":"tasks","kind":"tasks","path":".speclink/changes/add-feature/tasks.md","status":"missing","required":false,"dependencies":["proposal","spec"]},
      {"id":"spec:user-auth","kind":"spec","path":".speclink/changes/add-feature/specs/user-auth/spec.md","status":"done","required":true,"dependencies":["proposal"]}
    ]
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

artifact 順序固定為 `proposal`、`design`、`tasks`、然後 specs（按 capability name 字典序）。

**Interface（命名，不靠行號）**：

- clap 結構：
  - `Cli::Artifact(ArtifactCommand)`
  - `ArtifactCommand::Write(ArtifactWriteCommand)`
  - `ArtifactWriteCommand::{Design(ArtifactWriteArgs), Tasks(ArtifactWriteArgs), Spec(ArtifactWriteSpecArgs)}`
  - `Cli::Status(StatusArgs)`
- runtime 入口：
  - `crates/runtime/src/artifact.rs::write_artifact(provider: Arc<dyn Provider>, input: WriteArtifactInput) -> Result<WriteArtifactOutput, RuntimeError>`
  - `crates/runtime/src/status.rs::get_status(provider: Arc<dyn Provider>, input: GetStatusInput) -> Result<ChangeStatus, RuntimeError>`
- provider trait method：
  - `Provider::get_status(project_id, change_id) -> Result<ChangeStatus, ProviderError>`
  - `Provider::write_artifact` 既有簽章不變，但 `NewArtifact` 結構新增 `capability: Option<String>`
- output 型別：
  - `crates/cli/src/output.rs::ArtifactWriteData`
  - `crates/cli/src/output.rs::StatusData`
- 共用型別：
  - `crates/provider/src/model.rs::{ChangeStatus, ArtifactStatus, ArtifactState}`

**Failure modes**：

| 觸發條件 | error code | exit code | warning |
|---|---|---|---|
| change 不存在 | `change.not_found` | 1 | 無 |
| spec kind 缺 `--capability` | `artifact.missing_capability` | 2 | 無 |
| capability 名稱非法 | `artifact.invalid_capability` | 2 | 無 |
| 同 artifact 已存在 | `artifact.already_exists` | 1 | 無 |
| `--stdin` 為空（EOF immediately） | `input.invalid` | 2 | 無 |
| stdin 讀取 IO 失敗 | `internal.error` | 1 | 無 |
| filesystem 寫入失敗 | `internal.error` | 1 | 無 |
| `status` 對不存在 change | `change.not_found` | 1 | 無 |
| design / tasks kind 帶 `--capability` | `input.invalid` | 2 | 無 |

stderr：與 bootstrap 既有規則一致。

**Acceptance criteria**：

實作後以下測試通過：

1. `cargo build --workspace` 三平台無 warning（CI 矩陣）
2. `cargo fmt --check` 通過
3. `cargo clippy --workspace -- -D warnings` 通過
4. `cargo test --workspace` 通過，含：
   - `crates/provider-local/src/storage.rs` 新增 design / tasks / spec 寫入的 tempfile 測試（含原子性與既存檔案不覆寫）
   - `crates/provider-local/tests/multi_artifact_integration.rs` 一條 change 跑完 4 個 artifact 寫入後 `get_status` 結果正確
   - `crates/provider/src/lib.rs` 的 `dyn Provider` compile test 包含新 `get_status` method
   - `crates/runtime/src/artifact.rs` mock provider 測試：每個 kind 一條 happy path、capability 校驗、stdin 空字串
   - `crates/runtime/src/status.rs` mock provider 測試：missing change、partial complete、all complete
   - `crates/cli/tests/artifact_write.rs` 用 assert_cmd 跑 4 種 invocation（design、tasks、spec、含 capability 校驗）
   - `crates/cli/tests/status.rs` 用 assert_cmd 跑 happy path 與 change 不存在
   - insta snapshot 鎖定 artifact write 成功與失敗、status 各狀態的 JSON output
5. 手動驗證：在空目錄序列執行
   - `speclink propose create --change demo --summary "test" --json`
   - `echo "design body" | speclink artifact write design --change demo --stdin --json`
   - `echo "spec body" | speclink artifact write spec --change demo --capability foo --stdin --json`
   - `speclink status --change demo --json`
   檢查：四個檔案存在、status JSON artifacts 列出 proposal/design/spec:foo 為 done，tasks 為 missing
6. JSON output 不含任何 secret 字串

**Scope boundaries**：

- **In scope**：本 design 涵蓋的 trait 變更、CLI 子命令、storage 函式、3 個 spec（2 新 1 改）
- **Out of scope**：archive、instructions、task done、analyze、validate、pack、unpack、discuss、HTTP provider、auth 流程、Provider unavailable fallback warning 在新指令的呈現（warning 機制已存在，新指令自動繼承）

## Risks / Trade-offs

- **[`get_status` filesystem scan 對大 change 慢]** Mitigation：本 MVP 單 change 內 artifact 數量 < 20，掃描成本 < 1ms；若未來需要 millisecond-critical status，再評估在 SQLite 增加 artifact 索引表。
- **[`NewArtifact::capability` 對 trait 的破壞性新增]** Mitigation：`Option<String>` 為新增欄位且預設 `None`，對既有 caller（runtime 寫 proposal）無影響；trait method 簽章不變，只新增 method `get_status` — HTTP provider 在下下個 change 落地時需同步實作。
- **[spec artifact id `"spec:<capability>"` 在 JSON 中含冒號]** Mitigation：JSON 字串對冒號無限制；CLI snapshot 已驗證解析。AI skill 解析 id 時用 `split_once(':')` 區分 kind 與 capability。
- **[`--stdin` 在 Windows 上的編碼處理]** Mitigation：CLI 統一以 `BufReader::new(io::stdin())` + UTF-8 解析；非 UTF-8 輸入回 `input.invalid` exit 2。
- **[既有 `propose create` 是否需要兼容變更]** Mitigation：bootstrap 既有 spec 與測試完整保留；本 change 只擴張 LocalProvider 行為，不改 propose create 路徑。
- **[`required: bool` 第一版固定值缺彈性]** Mitigation：本 change 不引入 instructions / schema，required 規則先 hardcode；待 Change 4 加 instructions 時改為由 schema 驅動。

## Migration Plan

N/A — 本 change 純擴張，無資料 migration。既有由 bootstrap change 建立的 `.speclink/changes/<id>/` 結構（只有 proposal.md + metadata.json）對新 `status` 指令仍合法，只是 design / tasks / spec 列為 missing。

## Open Questions

- **`artifact write` 對既存檔案是否提供 `--force` overwrite？** 本 change 暫不提供（預設拒絕）；後續 Change 4 在引入 `instructions` 時可一併加 `--force`，因為 AI 修正 artifact 是合理流程。
- **`status` 是否需要 `--include-archived` 模式？** 本 change 不引入 archive，故不需要；Change 3 會增加 `--include-archived` 旗標。
- **spec artifact id 用 `spec:<capability>` 與用 `<capability>` 哪個對 skill 友善？** 暫定 `spec:<capability>` 以與 proposal/design/tasks 同層級保持 namespace 清楚；若 Change 4 的 instructions 指令發現過於囉嗦再改。
