## Context

SpecLink 是 greenfield 專案，本 change 是第一個 spec — 因此「目前狀態」幾乎是空的：repo 只有 `openspec/`、`doc/`、`CLAUDE.md`、`AGENTS.md`、`README.md`。沒有 `Cargo.toml`、沒有任何 Rust 程式碼。本設計文件釘住四件事，作為後續所有 change 的不變式：

1. Cargo workspace 結構與 crate 間依賴方向（決定後不再隨意動）
2. `Provider` async trait 形狀（`Send + Sync` 從第一版就釘死，避免後續加 HTTP provider 時需要破壞性修改）
3. CLI machine interface 的 JSON 外層 schema 與 exit code 對照（決定後其他 command 直接套用）
4. Error 命名規則（點分隔、capability 前綴）

設計約束來自 `CLAUDE.md` 與 `doc/speclink-provider-api-and-runtime-design.md`：lib crate 用 thiserror、bin crate 用 anyhow；I/O 邊界 async；跨 crate 透過 trait；不引入不必要的抽象層。

## Goals / Non-Goals

**Goals:**

- 建立可執行的最小 vertical slice：`speclink propose create --change <id> --summary <text> --json` 從 CLI 入口走到 local provider 寫檔，回傳穩定 JSON
- 釘死 Cargo workspace 與 4 個 crate 邊界，明確依賴方向，保證無循環
- 釘死 `Provider` trait 的 async fn 簽章、`Send + Sync` 與 dyn-compatibility
- 釘死 JSON 外層 schema（`ok` / `data` / `warnings` / `error` / `requestId`）
- 釘死 MVP exit code 與 error code 命名規則
- 釘死 `.speclink/` 目錄結構與 SQLite `state.db` 第一版 schema

**Non-Goals:**

- 不釘 HTTP provider 的 trait 細節（trait method 簽章釘住即可，HTTP 實作下個 change）
- 不釘 auth / token / keychain 流程
- 不釘其他 CLI 指令的 clap 形狀（讓未來 command 各自決定）
- 不釘 analyzer / validator / pack 的內部資料結構
- 不為 native provider plugin 或 WASM provider 預留介面

## Decisions

### Cargo workspace 採 4 crate 結構並釘死依賴方向

workspace 根目錄含 `Cargo.toml`（`[workspace]` 區塊）、`rust-toolchain.toml`（指定 stable + edition 2024）。MVP 建立 4 個 crate：

```
crates/
  cli/             # binary crate，clap command surface
  runtime/         # library，propose orchestration
  provider/        # library，Provider trait + 共用型別 + resolution 邏輯
  provider-local/  # library，filesystem provider 實作
```

依賴方向（保證無循環）：

```
cli --> runtime --> provider
cli --> provider-local --> provider
```

`cli` 同時依賴 `runtime` 與 `provider-local` — `runtime` 不直接 instantiate `provider-local`，而是接受 `Arc<dyn Provider>`，由 `cli` 在啟動時依 resolution 結果注入。

**替代方案**：

- **3 crate（合併 runtime + cli）**：對單一指令來說 runtime 邏輯很薄，可直接放 cli。拒絕原因：cli 是 binary（anyhow），runtime 是 library（thiserror），錯誤類型不同；分開後第二個 command 加入時不需重構。
- **5 crate（再拆出 `types`）**：把 `Project`/`Change`/`Artifact`/`Pack`/`State` 拆到獨立 types crate。拒絕原因：MVP 只有 provider 用這些型別，YAGNI；未來出現非 provider 共用型別（如 analyzer findings）再拆。

### Provider trait 使用 async_trait 巨集並要求 Send + Sync + dyn-compatible

`crates/provider/src/lib.rs` 定義：

```rust
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn create_change(&self, project_id: &ProjectId, input: NewChange) -> Result<Change, ProviderError>;
    async fn write_artifact(&self, project_id: &ProjectId, change_id: &ChangeId, input: NewArtifact) -> Result<Artifact, ProviderError>;
    async fn get_change(&self, project_id: &ProjectId, change_id: &ChangeId) -> Result<Change, ProviderError>;
}
```

trait 物件型別固定為 `Arc<dyn Provider>`，由 cli 持有並傳給 runtime。

**替代方案**：

- **原生 `async fn in trait`（Rust 1.75+）**：不需 macro。拒絕原因：原生 `async fn in trait` 在 dyn 場景需要 `trait-variant` 或手寫 `Pin<Box<dyn Future>>`，且 RPIT 在 dyn 限制較多；本專案需要 `Arc<dyn Provider>` 在 runtime 與 cli 間傳遞，`async_trait` 雖有 Box::pin 額外配置但 CLI 不在 hot path，trade-off 划算。
- **trait method 回傳具名 Future 型別**：完全避免 macro。拒絕原因：每個 method 寫一份 type alias 太囉嗦，且未來新增 method 維護成本高。
- **`Provider: Clone` 而非 `Arc<dyn Provider>`**：要求實作型別自身 Clone。拒絕原因：local provider 內含 SQLite connection pool，Clone 成本不對稱；統一 Arc 比較乾淨。

呼叫 `rust-skills:m05-type-driven` 與 `rust-skills:m07-concurrency` 確認 dyn-compatible async trait 的當前 idiom。

### SQLite client 採 rusqlite 並包 spawn_blocking

local provider 的 SQLite 操作集中在 `crates/provider-local/src/state_db.rs`，所有公開 API 為 async，內部用 `tokio::task::spawn_blocking` 包裝同步 `rusqlite::Connection`。

```rust
pub struct StateDb { pool: Arc<Mutex<rusqlite::Connection>> }

impl StateDb {
    pub async fn set_in_progress(&self, change_id: &ChangeId) -> Result<(), StateDbError> {
        let pool = self.pool.clone();
        let id = change_id.clone();
        tokio::task::spawn_blocking(move || { /* INSERT OR REPLACE */ })
            .await
            .map_err(StateDbError::JoinError)?
    }
}
```

第一版只開一張表，schema 由 `crates/provider-local/src/state_db.rs::CREATE_TABLES_SQL` 常數定義：

```sql
CREATE TABLE IF NOT EXISTS in_progress_change (
    change_id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
PRAGMA user_version = 1;
```

migration 策略：每次開 connection 先讀 `PRAGMA user_version`，若 < 當前 CLI 版本則跑對應 migration（MVP 只有 version 1，無 migration）。

**替代方案**：

- **sqlx**：原生 async、compile-time SQL 驗證。拒絕原因：sqlx 帶入 macro、runtime feature 選擇、build 時連 DB 等複雜度，且需要在 Cargo feature flag 選擇 tokio / async-std。MVP 只有一張表、兩個 query，rusqlite + spawn_blocking 更輕。
- **redb / sled**：純 Rust 內嵌 KV。拒絕原因：SQLite 是業界穩定選擇，未來 schema 演化的工具鏈（migration、CLI inspection）成熟。
- **直接寫 JSON 檔當狀態**：在 `.speclink/state.json` 存 `{ "in_progress_change": "..." }`。拒絕原因：未來會有 `parked_changes`（多筆）、`tasks_status` 等表，JSON 檔的併發寫入需要 file lock，比 SQLite 麻煩。

呼叫 `rust-skills:m11-ecosystem` 與 `rust-skills:rust-learner` 確認 rusqlite 當前穩定版本。

### Error 架構 lib 用 thiserror、cli 用 anyhow、跨層用點分隔 error code

每個 lib crate 定義自家 `Error` enum：

- `provider::ProviderError`：`NotAuthenticated`、`ChangeAlreadyExists`、`ChangeNotFound`、`Unavailable`、`Internal`
- `provider_local::LocalProviderError`：`Io`、`Toml`、`StateDb`、`InvalidChangeId`
- `runtime::RuntimeError`：`Provider(ProviderError)`、`InvalidInput`

`crates/cli/src/main.rs` 用 `anyhow::Result<()>`，並在 `cli/src/exit_code.rs` 提供：

```rust
pub fn classify(err: &anyhow::Error) -> (ExitCode, ErrorCode) { ... }
```

依 anyhow 鏈中的 `ProviderError::NotAuthenticated` 映射到 `(6, "provider.not_authenticated")`、`ProviderError::Unavailable` 到 `(5, "provider.unavailable")`、其餘 lib error 到 `(1, "internal.error")`、clap parse error 到 `(2, "input.invalid")`。

error code 命名規則：`<capability>.<short_snake_case>`，capability 來自 spec 名稱前綴。本 change 涉及：

- `provider.not_authenticated`
- `provider.unavailable`
- `change.already_exists`
- `change.invalid_id`
- `input.invalid`
- `internal.error`

**替代方案**：

- **全專案統一 `thiserror`**：cli 也用 thiserror。拒絕原因：anyhow 在 binary 串接 error context 比 thiserror 簡潔，且 CLAUDE.md 明確規定 cli 用 anyhow。
- **error code 用數字（HTTP status 風格）**：例如 4001。拒絕原因：可讀性差，且 SpecLink 的 error code 數量會多到 4 位數，難記憶。

呼叫 `rust-skills:m13-domain-error` 確認 error code 命名與分類最佳實踐。

### JSON output 採 typed serde 結構並集中在 cli output 模組

`crates/cli/src/output.rs` 定義：

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T: Serialize> {
    pub ok: bool,
    pub data: Option<T>,
    pub warnings: Vec<Warning>,
    pub error: Option<ErrorBody>,
    pub request_id: String,
}

#[derive(Serialize)]
pub struct Warning { pub code: String, pub message: String }

#[derive(Serialize)]
pub struct ErrorBody { pub code: String, pub message: String, pub details: serde_json::Value }
```

每個 command 定義自己的 `Data` 型別。`propose create` 的 data：

```rust
#[derive(Serialize)]
pub struct ProposeCreateData {
    pub change_id: String,
    pub state: String,         // "proposed"
    pub artifact_path: String, // .speclink/changes/<id>/proposal.md
    pub mode: String,          // "local"
}
```

`request_id` 由 cli 啟動時用 `uuid::Uuid::new_v4()` 產生。

**替代方案**：

- **`serde_json::json!` macro inline**：每個 command 自行組 JSON。拒絕原因：失去 type checking，schema drift 風險高，且 snapshot test（insta）需要穩定 key order。
- **JSON Schema 自動產生**：用 `schemars` crate 從 typed struct 產 schema。拒絕原因：MVP 不需要對外發佈 schema 檔，等 HTTP provider 介接時再加。

### Provider resolution 在 provider crate 內實作並回傳明確 enum

`crates/provider/src/resolution.rs`：

```rust
pub enum ResolvedProvider {
    Local { reason: LocalReason },
    Remote { name: String, kind: ProviderKind },
}

pub enum LocalReason {
    NoConfig,                    // 完全沒有 global / project config
    FallbackFromUnavailable,     // 有設定但 provider 不可用 + fallback=local
    FallbackFromUnauthenticated, // 有設定但未登入 + fallback=local
    Explicit,                    // config 明確設 type=local
}

pub struct ResolutionInputs<'a> {
    pub flag_provider: Option<&'a str>,
    pub project_config: Option<&'a ProjectConfig>,
    pub global_config: Option<&'a GlobalConfig>,
    pub env_provider: Option<&'a str>,
}

pub fn resolve(inputs: ResolutionInputs<'_>) -> Result<(ResolvedProvider, Vec<Warning>), ResolutionError>;
```

五層優先序的程式碼路徑與 spec `provider-resolution` 的 WHEN/THEN 場景一對一對應。MVP 因為沒有 HTTP provider，遇到 `Remote` 分支會把它降級成 `Local { reason: FallbackFromUnauthenticated }` 並發 `provider.not_authenticated` warning。若 fallback=`disabled`，回傳 `ResolutionError::AuthRequiredNoFallback` 對應 exit code 6。

**替代方案**：

- **resolution 放在 runtime**：拒絕原因：runtime 依賴 provider，resolution 邏輯只用 provider 與 config 型別，放 provider crate 不會引入循環，反而避免 runtime 與 cli 同時做 resolution。
- **resolution 放在 cli**：拒絕原因：未來 HTTP provider 的 server 端可能也需要解析 client 端發來的 resolution context；留在 lib crate 更乾淨。

### 配置檔載入採 dirs 與 toml crate

`crates/provider/src/config.rs`（同 crate）：

- global config 路徑：`dirs::config_dir().join("speclink").join("config.toml")`；Windows `%APPDATA%\speclink\config.toml`、Linux `~/.config/speclink/config.toml`、macOS `~/Library/Application Support/speclink/config.toml`
- project config 路徑：當前 CWD 向上找 `.speclink/config.toml`（找到 git root 或 filesystem root 為止）
- 環境變數覆蓋：`SPECLINK_CONFIG_HOME` 覆蓋 global config 目錄
- 兩個檔案都可選；缺檔回傳 `Ok(None)`，不視為錯誤
- 解析失敗（無效 TOML）視為使用者輸入錯誤對應 exit code 2

**替代方案**：

- **`config` crate**：支援多層 config 與 env override。拒絕原因：本專案的 config 層級邏輯需要明確控制（5 層 resolution），用 `config` crate 反而難 debug；自行用 toml + dirs 兩個簡單 crate 更直接。
- **`directories` crate**：與 dirs 功能類似但 API 不同。`dirs` 較輕（10 KB）且 API 簡潔，選 dirs。

### 異步 runtime 採 tokio multi-thread 並在 main 內初始化

`crates/cli/src/main.rs` 入口：

```rust
#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode { ... }
```

lib crate（provider、provider-local、runtime）保持 tokio runtime 無關 — 不在 lib 內 spawn task，所有 async 由 caller 推動。`tokio::task::spawn_blocking` 是允許的，因為它只要求有 tokio runtime 在跑，不要求特定 flavor。

**替代方案**：

- **current_thread flavor**：單執行緒省記憶體。拒絕原因：spawn_blocking 在 current_thread 仍可用，但未來 HTTP provider 並發請求會受限。
- **`smol` 或 `async-std`**：拒絕原因：tokio 是業界主流，第三方 crate 相容性最好（reqwest、sqlx 等）。

### Lifecycle state 採 metadata.json 與 SQLite 雙重儲存

每個 change 在 `.speclink/changes/<change-id>/metadata.json` 寫入完整 lifecycle 資訊：

```json
{
  "changeId": "...",
  "state": "proposed",
  "createdAt": "2026-05-19T...",
  "createdBy": { "type": "agent", "name": "claude" }
}
```

同時 SQLite `in_progress_change` 表記錄目前 active change id（單筆）。

**為何雙重儲存**：metadata.json 是 source of truth（人可讀、可手動編輯、可進 git）；SQLite 是 fast lookup index（未來 `status` 指令一次查多個 change 用）。本 change 暫時用不到 SQLite 的查詢效能優勢，但建立雙寫慣例避免後續改架構。

**替代方案**：

- **只用 metadata.json**：拒絕原因：未來掃描 100 個 change 找 in_progress 太慢。
- **只用 SQLite**：拒絕原因：metadata 進不了 git diff，PM 或工程師無法手動 review change 狀態。

## Implementation Contract

**Observable behavior**：

執行 `speclink propose create --change add-order-export --summary "新增訂單匯出流程" --json` 後：

1. 若 `.speclink/changes/add-order-export/` 已存在，process exit code 為 1，stdout 印錯誤 JSON，code = `change.already_exists`
2. 否則建立目錄結構：
   - `.speclink/config.toml`（若不存在）寫入 default content（`mode = "local"`）
   - `.speclink/state.db`（若不存在）建立 SQLite 並跑 schema v1
   - `.speclink/changes/add-order-export/proposal.md` 寫入 `## Why\n\n新增訂單匯出流程\n`
   - `.speclink/changes/add-order-export/metadata.json` 寫入 lifecycle metadata
   - SQLite `in_progress_change` 插入 `change_id = "add-order-export"`
3. stdout 印一行 JSON：

```json
{"ok":true,"data":{"changeId":"add-order-export","state":"proposed","artifactPath":".speclink/changes/add-order-export/proposal.md","mode":"local"},"warnings":[],"error":null,"requestId":"req_..."}
```

4. process exit code = 0

**Interface（命名，不靠行號）**：

- clap 結構：`crates/cli/src/cli.rs::Cli` 為頂層 enum，包含 subcommand `Propose(ProposeArgs)`、`ProposeArgs::Create(ProposeCreateArgs)`
- runtime 入口：`crates/runtime/src/propose.rs::create_proposal(provider: Arc<dyn Provider>, input: CreateProposalInput) -> Result<CreateProposalOutput, RuntimeError>`
- provider trait method：`Provider::create_change`、`Provider::write_artifact`（簽章如 Decisions 區所列）
- output 型別：`crates/cli/src/output.rs::Envelope<ProposeCreateData>`

**Failure modes**：

| 觸發條件 | error code | exit code | warning 是否伴隨 |
|---|---|---|---|
| change id 已存在 | `change.already_exists` | 1 | 無 |
| change id 含非法字元（非 kebab-case） | `change.invalid_id` | 2 | 無 |
| `--summary` 為空字串 | `input.invalid` | 2 | 無 |
| global config 設 type=http 但無 auth + fallback=local | （正常流程 exit 0） | 0 | `provider.not_authenticated` |
| global config 設 type=http 但無 auth + fallback=disabled | `provider.not_authenticated` | 6 | 無 |
| filesystem 寫入失敗（權限、磁碟滿） | `internal.error` | 1 | 無 |
| TOML 解析失敗 | `input.invalid` | 2 | 無 |
| 任何未分類錯誤 | `internal.error` | 1 | 無 |

stderr：僅在 `--quiet` 未指定時印人類可讀的 progress（tracing INFO 等級）；`--json` 模式下 stderr 不影響 stdout JSON。

**Acceptance criteria**：

實作後以下測試通過：

1. `cargo build --workspace` 在 Windows / macOS / Linux 三平台無 warning（CI 矩陣）
2. `cargo fmt --check` 通過
3. `cargo clippy --workspace -- -D warnings` 通過
4. `cargo test --workspace` 通過，含：
   - `crates/provider/src/resolution.rs` 五層優先序的單元測試（每層至少一個 case）
   - `crates/provider-local/src/state_db.rs` SQLite schema 建立、insert、query 的單元測試
   - `crates/provider-local/src/storage.rs` proposal.md 與 metadata.json 寫入的 tempfile 測試
   - `crates/cli/tests/propose_create.rs` 用 assert_cmd 跑 `speclink propose create` 並驗證 stdout JSON、exit code、檔案系統副作用（tempfile 隔離）
   - insta snapshot 鎖定 `propose create` 成功與失敗的 JSON output（含 request_id mask）
5. 手動驗證：在空目錄執行 `speclink propose create --change demo --summary "test" --json` 後檢查：
   - `.speclink/changes/demo/proposal.md` 存在且內容正確
   - `.speclink/state.db` 存在且 `SELECT change_id FROM in_progress_change` 回傳 `demo`
   - stdout 可用 `jq .` 解析無誤
   - exit code 為 0
6. JSON output 不含任何 token 或 secret 字串（lint 規則：grep 任何 known secret pattern 應為零命中）

**Scope boundaries**：

- **In scope**：上述四個 spec 涵蓋的行為、四個 crate 的初始實作、CI workflow 設定（`.github/workflows/ci.yml`）
- **Out of scope**：HTTP provider、auth 流程、`provider add` / `auth login` / `project bind` 等人類設定指令、analyzer / validator / pack 任何功能、其他 AI workflow 指令、跨機器 pack/unpack、Web UI、shell completion、`crates/types/`、`crates/provider-http/`、`crates/auth/`、`crates/analyzer/`、`crates/validator/`、`crates/pack/`、`crates/skill-templates/`

## Risks / Trade-offs

- **[SQLite schema 未來 migration 複雜度]** Mitigation：第一版用 `PRAGMA user_version` 標記 schema version；每次 connection open 時讀 version，若 < CLI 預期版本則跑對應 migration script（MVP 只有 version 1 無需 migration）。後續加表時新增 `migrate_v1_to_v2()` 函式，CI 加 migration 測試確保升級不破壞既有資料。
- **[rusqlite 同步 API 加 spawn_blocking 額外配置]** Mitigation：CLI 場景非 hot path（每個指令呼叫一次 DB），spawn_blocking 配置成本（微秒級）相對 SQLite IO 可忽略。若未來 daemon mode 或大量並發出現瓶頸再評估 sqlx。
- **[async_trait 巨集的 Box::pin 配置開銷]** Mitigation：同上，CLI 非 hot path 可忽略；若未來 trait 進入伺服器端高頻路徑，再考慮拆 trait 或改用 enum dispatch。
- **[Cargo workspace 一次建立 4 crate 導致首次 build 較慢]** Mitigation：MVP 每個 crate 程式碼量極小（< 500 LoC），首次 build 預估 < 60 秒；CI 啟用 `actions/cache` 快取 `target/` 目錄。
- **[`dirs` crate 在非標準 Windows 安裝環境（如 portable mode）找不到 config_dir]** Mitigation：提供 `SPECLINK_CONFIG_HOME` 環境變數覆蓋，作為硬性 fallback。
- **[edition 2024 與第三方 crate 相容性風險]** Mitigation：在 rust-toolchain.toml 釘住 stable channel 但允許 edition 2024（Rust 1.85+ 支援）；若任何 crate 不相容則暫時用 edition 2021，待 ecosystem 跟進。實作前透過 `rust-skills:rust-learner` 確認 edition 2024 採用狀況。
- **[JSON output 的 request_id 在測試中不穩定]** Mitigation：insta snapshot 用 `[request_id]` 樣式遮罩；測試用環境變數 `SPECLINK_TEST_REQUEST_ID` 覆寫為固定值。
- **[metadata.json 與 SQLite 雙寫一致性]** Mitigation：MVP 用簡單順序寫入（先 metadata，再 SQLite）；若 SQLite 寫入失敗，metadata 已存在會導致下次掃描看到 orphan change。對應補救：在 `propose create` 失敗路徑加 cleanup（刪除 metadata 目錄）。長期 fix 是在 SQLite 內也存完整 metadata 鏡像，由背景 reconciler 對齊，但 MVP 範圍不做。

## Migration Plan

N/A — greenfield。本 change 後第一次 commit 即為初始 workspace。

## Open Questions

- **rust-toolchain.toml 是釘 stable 還是釘特定版本（如 1.85.0）？** 傾向釘 channel = stable，CI 用 matrix 確保 stable 與當期 stable 都通過。實作前用 `rust-skills:rust-learner` 確認當前 stable 是否支援 edition 2024。
- **CI 是否在本 change 內納入？** 建議納入（GitHub Actions matrix Windows/macOS/Linux × stable），因為跨平台是 CLAUDE.md 強制需求；若 CI 設定本身有複雜度，可拆獨立 follow-up change。本 change 暫定納入 CI 基礎設定。
