## Context

SpecLink 是 greenfield 專案，`openspec/specs/` 目前完全為空。MVP walking skeleton 的第一片必須先把「能跑出可觀察結果」的最小路徑落地：fresh git repo 內執行 `speclink init && speclink status --json` 回非錯誤 JSON。本 change 提供此最薄端到端，並把後續所有 capability 共用的 disk contract、CLI envelope、crate 邊界、error code namespace 一次釘下。

設計依據 `doc/speclink-design.md` §13.1（Boundary）、§13.6（Working dir 綁定強度）、§13.9（State storage layout）、§14（檔案結構）、§14.1（預設 .gitignore）、§16.1（Project 管理 CLI 流程）、§17.3–§17.5（error/finding code）、§18.1 #1–#15（MVP 必做）、§19.3（LocalProvider）。

當前狀態：尚未有 Rust 程式碼存在；只有設計稿與 spectra 設定。本 change 同時開啟 Cargo workspace 與最早的 4 個 crate（`cli`、`runtime`、`provider`、`provider-local`）。

## Goals / Non-Goals

**Goals:**

- 提供 `speclink init` / `speclink status` / `speclink link` / `speclink unlink` 四個 CLI 表面，皆支援 `--json`
- 落地 two-root storage layout：`.speclink/`（artifact root，git-tracked）與 `.git/speclink/`（state root，不進 git）
- 強制 git working dir，non-git 一律拒絕並回 `project.requires_git`
- 釘下 CLI JSON envelope 與 exit code 對照（後續所有 change 沿用）
- 釘下 4 crate 邊界：`cli` ↔ `runtime` ↔ `provider`（trait）↔ `provider-local`（impl）
- 釘下 state.db 初始 schema 與 migration runner 介面骨架（不含實際 migration 業務內容）
- 釘下 error code namespace 三個（`project.requires_git`、`project.already_initialized`、`project.link_target_not_found`）與 4 個 doctor finding code 字串常量（不含檢查邏輯）

**Non-Goals:**

- 不實作 change CRUD、artifact read/write、apply、review、archive、discuss、schema CLI、skill 部署、HttpProvider、`restore`、`doctor` 完整實作、multi-project listing、worktree marker
- 不引入 `keychain`、`reqwest`、`pack` 等只屬於後續 capability 的 dependency
- 不實作分散式鎖、跨機 sync、CI 整合
- 不為 HttpProvider 預留 trait method 之外的任何 stub 實作（只先抽出 trait shape）

## Decisions

### Two-root storage layout: `.speclink/` artifact root + `.git/speclink/` state root

**選擇**：把人類可讀的 artifact（`link.yaml`、`schemas/`、後續的 `changes/<id>/`）放在 working tree 內的 `.speclink/`；把 mutable runtime state（`state.db`、locks、cache）放在 git common dir 內的 `.git/speclink/`。

**Rationale**：
- Artifact 必須跟 spec 一起進版控，所以放在 working tree。
- State 不該進版控（branch 切換、merge conflict、雲端同步盤 corruption），放 `.git/` 內天然避開所有同步工具與 git index。
- `.git/speclink/` 路徑模仿 spectra cli 既有的 `.git/spectra-app/` 慣例（見 `doc/spectra-reference.md`），用「cross-tool namespace under .git/」這個 idiom。
- Worktree 與 submodule 場景下，`.git/` 本身會被 git 重新路由到 common dir，state 自動跨 worktree 共享，無需自己寫 marker。

**Alternatives 考量**：
- 全部塞 `.speclink/`（artifact + state 同根）：state.db 會被 git diff、被 sync tool corrupt、branch 切換時 state 跟著切。否決。
- 全部塞 `.git/speclink/`（state 與 artifact 同根）：artifact 不進版控，spec 沒法 review。否決。
- State 放 `~/.local/share/speclink/<hash>/`（XDG path）：跨機器後 state 不可攜，且 worktree 共享需自己維護 reverse index。否決。

### State root 路徑統一走 `git rev-parse --git-common-dir`

**選擇**：runtime 解析 state root 時不直接拼 `.git/speclink`，而是呼叫 `git rev-parse --git-common-dir` 取得 git common dir，再附加 `speclink/`。

**Rationale**：
- Worktree 場景：linked worktree 的 `.git` 是檔案而非目錄，內容指向 main worktree 的 git dir。直接拼 `.git/speclink` 在 linked worktree 會出錯。
- Submodule 場景：submodule 的 git dir 在 superproject 的 `.git/modules/<name>/`。
- Custom `GIT_DIR` / `GIT_COMMON_DIR` env：使用者可能用 `git --git-dir` 操作。
- 唯一可靠來源是 git 自己告訴你。

**Alternatives 考量**：
- 自己解析 `.git` 檔案內容（`gitdir: ...`）：等於重寫 git 的 path resolution，會漏 case。否決。
- 用 `libgit2` / `gix` 取代 git CLI：MVP 範圍引入大型 dependency 不划算；後續若需更多 git interaction 再評估。本 change 採 `Command::new("git").args(["rev-parse", "--git-common-dir"])`。

### 強制 git working dir，non-git 拒絕 init

**選擇**：`speclink init` 在 non-git working dir 直接以 exit code 2 + error code `project.requires_git` 拒絕，hint 提示使用者先 `git init`。

**Rationale**：
- 上一個 decision 把 state root 綁在 git common dir。non-git 沒有 `.git/`，state root 無處安放。
- 與其在 non-git 下退化成「state 放 `.speclink/state.db`」（兩條 path resolution 路徑、雙倍測試成本、未來 doctor 邏輯複雜化），不如一律強制 git。
- SpecLink 的設計前提是 spec/artifact 進版控，非 git 用法本來就不在目標情境。
- 強制 git 的決策已在 design doc §13.1 與 §18.2 deferred 中釘下（non-git fallback 列 deferred / 可能 non-goal）。

**Alternatives 考量**：
- Non-git 下 state 退化到 `.speclink/.state/state.db`：see above，否決。
- Init 時自動 `git init`：副作用過大，使用者預期之外。否決，改為清楚 hint。

### `.gitignore` 政策：單行 `.speclink/link.yaml`

**選擇**：`speclink init` 在 working dir 的 `.gitignore`（不存在則建立）追加單行：`.speclink/link.yaml`。其他 `.speclink/` 內容（`schemas/`、後續的 `changes/`）皆進版控。

**Rationale**：
- `link.yaml` 含本機 binding 資訊（machine-local project_id mapping、instance_id），不該共用。
- `schemas/` 是 spec 規範本體，必須進版控。
- `state.db` 在 `.git/speclink/` 不在 working tree，本來就被 git 忽略，不需 `.gitignore` 條目。
- 一行政策最簡單，使用者一眼看懂。

**Alternatives 考量**：
- 整個 `.speclink/` 都 ignore：等於 spec 不進版控，與 SpecLink 定位相反。否決。
- Ignore `.speclink/state*`、`.speclink/cache/`、`.speclink/locks/`：state 本來已在 `.git/`，不需要這些條目；列出反而誤導讀者以為 state 在 working tree。否決。
- 寫到 `.git/info/exclude` 而非 `.gitignore`：使用者看不到，違反「明示優於暗示」。否決。

### Worktree init 沿用主 repo 的 project_id

**選擇**：當 `Bootstrap::init` 偵測到 state root 內已存在 `state.db`（preexisting state），不再透過 staging 階段產生新的 `project_id`，改為從既存 state.db 讀取唯一一筆 `project` row 的 id 作為新 `link.yaml` 的 `project_id`；既存 state.db 維持不動（不 insert、不 rename）。

**Rationale**：
- 在 `git worktree add` 出來的 linked worktree 內跑 init 時，state root 與主 repo 共享（透過 `git rev-parse --git-common-dir`）；既存 `state.db` 已有主 repo 的 project row。
- 若沿用既存做法生新 `project_id`、且 staging 內的 row 因 preexisting 而被丟棄 — 結果是 worktree 的 link.yaml 內 `project_id` 在主 state.db 內找不到對應 row，後續 `speclink link <id>` 會回 `link_target_not_found`。
- 共享 `project_id` 才符合「main + worktrees 屬於同一個 SpecLink project」這個前提（design §13.6 / §13.7）。
- 仍 rotate `instance_id` 與 refresh `created_at`，標示這是同 project 的不同綁定實例。

**Alternatives 考量**：
- 在主 state.db 額外插一筆 row 表示 worktree 是另一個 project：違反「同 working tree = 同 project」前提，且讓多 worktree 場景累積無限多 row。否決。
- 不 init，硬要求使用者跑 `speclink link <main-id>`：對使用者 ergonomics 差，且需要 main 先把 project_id 告知 worktree（無自動 discoverable 入口）。否決。
- 把 state.db row 數 ≠ 1 的情境也涵蓋：MVP 不支援多 project per state.db；本決策只處理 row 數恰為 1 的場景，0 或多筆走 Internal error 並由未來 capability 處理。

### `state_root` 路徑顯示：strip prefix 失敗時走 canonical absolute path

**選擇**：`relative_state_root_display(working_dir, state_root)` 在 `strip_prefix` 失敗（state root 不在 working dir 內，如 linked worktree）時，不再用 `components().map(...).join("/")` 拼，改回傳 `state_root` 的 canonical absolute path 字串（POSIX 單 `/` 前綴或 Windows drive letter）。

**Rationale**：
- 原本用 `components()` join `"/"` 對 absolute path 會把 `Component::RootDir`（POSIX 的 `/`）視為空字串，join 後產出 `//path` 開頭雙斜線。
- 雙斜線會讓 consumer（特別是 JSON-eating tools）困惑，且在某些 POSIX 系統上（如 macOS 的 `//foo` 可能被視為網路 path）行為不一致。
- 退回原始 `Path` display 可確保格式正確，付出的代價是 worktree 場景下回 absolute path（這本來就是合理結果）。

**Alternatives 考量**：
- 強制把所有 state root 顯示為 working-dir-relative：linked worktree 場景下要算 `../../something` 並不直觀；且 worktree 與 main 可能不在同一 parent tree。否決。
- 把雙斜線當 cosmetic 不修：consumer 端的 path normalization 不可靠（POSIX 認 `//foo` 為 implementation-defined），會在未來咬。否決。

### Bootstrap CLI 拆四個 subcommand：init / status / link / unlink

**選擇**：用 clap v4 derive 在 `crates/cli/src/commands/{init,status,link,unlink}.rs` 各一支 subcommand，全部走 `crates/runtime` 的同一個 bootstrap module。

**Rationale**：
- `init` 是首次 setup（artifact root + state root + schema seed + state.db migration + .gitignore）；不可重入（已 init 再 init 回 `project.already_initialized` conflict，除非 `--force`）。
- `status` 是純讀取（zero-side-effect），CI 與 doctor 都要呼叫。
- `link` 是把現有 working dir 綁定到既存 state（用於 clone 後 re-bind、或從 backup 還原），與 init 區隔避免重複觸發 schema seed。
- `unlink` 只移除 binding metadata，不刪 artifact、不刪 state.db；提供安全的「臨時抽離」入口。

**Alternatives 考量**：
- 把 link 合進 `init --bind`：兩個語意混在一支命令，使用者難辨「我要新建還是重新接管」。否決。
- 不要 unlink，逼使用者手動刪 `link.yaml`：違反「對等可逆操作」原則，且 unlink 是 doctor / debug 需要的入口。保留。

### CLI JSON envelope 標準化

**選擇**：所有 SpecLink CLI 命令 `--json` 輸出統一 envelope：

成功：
```json
{ "ok": true, "data": <command-specific>, "warnings": [], "requestId": "<uuid>" }
```

失敗：
```json
{ "ok": false,
  "error": { "code": "project.requires_git",
             "message": "<human>",
             "hint": "<actionable>",
             "retryable": false,
             "retry_after_ms": null },
  "requestId": "<uuid>" }
```

Envelope 序列化由 `crates/cli/src/output.rs` 集中處理，所有 subcommand 回傳 `Result<Value, CliError>`，由 main 統一包裝。

**Rationale**：
- 與 design doc §17.1 對齊；後續每個 change 都直接沿用，不會在每支命令重新發明。
- `retryable` + `retry_after_ms` 雖然 LocalProvider 不會用到，但 envelope 必須先把欄位定下，否則 HttpProvider 上線後要做 breaking change。
- `requestId` 用 UUID v4，所有 log 都帶上，方便跨 process / 跨 agent 追蹤（多 agent 並發場景已在 design §12 釘下）。

**Alternatives 考量**：
- 直接印命令特定 JSON（無 envelope）：每命令格式不同，consumer 寫死大量 case，HttpProvider 對接時必須改所有 consumer。否決。
- 用 JSON-RPC 風格（`jsonrpc`/`result`/`error`/`id`）：欄位命名不直覺，且 SpecLink 不是雙向 RPC。否決。

### State.db schema 初始化與 migration runner 介面

**選擇**：`provider-local` crate 暴露 `StateDb::open(path)` 與 `StateDb::migrate(target_version)`。Migration 用內嵌 SQL 字串陣列 `MIGRATIONS: &[&str]`，順序執行，記錄當前 version 在 `_migrations` table。本 change 只提供 migration runner 與 v1 schema（最小 schema：`project(id, created_at, instance_id, working_dir)`、`_migrations(version, applied_at)`）。

**Rationale**：
- 後續每個 capability change 都會新增 migration（change CRUD 加 `changes` table、artifact io 加 `artifacts` table）。Runner 介面必須在第一份 change 就釘下。
- 用內嵌 SQL `&str` array 而非 `refinery`/`sqlx-migrate` 等 macro-based 工具：dependency 更輕，MVP 不需要 down migration、不需要外部 file 載入。
- v1 schema 只含 `project` 一個 row（singleton）+ `_migrations` book-keeping。後續 capability 可疊加。

**Alternatives 考量**：
- 用 `sqlx-migrate` 從 `migrations/*.sql` 載入：引入 sqlx CLI 工具鏈，過早。否決（MVP 範圍），後續若 migration 數量大或需要 down migration 再評估。
- 不做 migration、直接 CREATE TABLE IF NOT EXISTS：升級舊 state.db 時無法處理 schema 變更，會在第二個 capability change 馬上撞到。否決。

### Provider trait skeleton 先抽，LocalProvider 為唯一具體實作

**選擇**：`crates/provider` 定義 trait `ProjectStore`（method：`init`、`status`、`link`、`unlink`、`get_link`、`save_link`）。`crates/provider-local` 提供 `LocalProjectStore` 實作。`crates/runtime` 對 trait 編程。

**Rationale**：
- HttpProvider 是 MVP 後第一個正式可替換目標。本 change 不寫 HttpProvider，但若 runtime 與 LocalProvider 直接耦合，未來抽 trait 是 breaking change（所有 runtime code 要改）。
- Trait 此時只有一個實作，看似 over-abstraction，但這條 seam 通過「Deletion test」：刪掉 trait 等於把 LocalProvider 邏輯灌進 runtime，violates §19.1 設計約束（Provider 必須可替換）。
- Trait method 僅涵蓋本 change 範圍（init/status/link/unlink），不為未來功能預留 method。後續 change 再用「Modified Capabilities」追加 trait method。

**Alternatives 考量**：
- 不抽 trait，runtime 直接 import `provider-local`：see above，否決。
- 抽 trait 且預留未來所有 method（change CRUD、artifact io、apply、review、archive）：違反「禁止為假設性需求預留介面」rule。否決。

### Crate 邊界：cli ↔ runtime ↔ provider ↔ provider-local

**選擇**：四個 crate，dependency 方向單一：

```
cli → runtime → provider (trait) ← provider-local
      runtime → provider-local (concrete dep for binary wiring at top level)
```

Crate 職責：
- `cli`：clap parsing、JSON envelope serialization、exit code mapping、main entrypoint
- `runtime`：bootstrap orchestration（git check → save link → init state.db → seed schemas → write .gitignore）、path resolution（git-common-dir）、error types
- `provider`：`ProjectStore` trait、shared types（`ProjectInfo`、`LinkYaml`、`InitOptions`、`StatusResponse`）
- `provider-local`：`LocalProjectStore` impl、SQLite open/migration、YAML read/write

**Rationale**：
- 四個 crate 是 design doc §15 拆分的最小子集（11 個 crate 中只啟用 4 個）；其他 crate 留待對應 capability 啟用。
- Runtime 與 provider-local 的具體 wire 必須有一個 layer 做。MVP 走 runtime 直接 instantiate `LocalProjectStore`；HttpProvider 進場時再加 provider resolution layer。
- Cli 不直接 import `provider-local`：保持「換 provider 不動 cli」的設計前提。

**Alternatives 考量**：
- 三 crate（合 provider 與 provider-local）：trait 與 impl 同 crate，未來新增 provider-http 必須改既有 crate `Cargo.toml`，且循環 import 風險上升。否決。
- 五 crate（多加 `types` crate）：MVP 範圍下 shared type 很少（5 個 struct），放 `provider` 已足。否決。

### Doctor finding code 字串常量先註冊，邏輯不實作

**選擇**：本 change 在 `crates/runtime/src/error.rs` 註冊 4 個 finding code 字串常量：
- `doctor.project.requires_git`
- `doctor.state.db_missing`
- `doctor.state.db_corrupted`
- `doctor.state.db_schema_invalid`

僅作為 `const &str` 暴露，無對應 check 函式、無 `doctor` 子命令。

**Rationale**：
- Finding code 是跨 change 的 stable identifier，必須一次釘死避免後續重命名造成 breaking change。
- 不寫 check 邏輯是因為 doctor 系統的 lifecycle（finding registry、auto-fix allowlist、JSON envelope）由獨立 change `add-doctor` 處理。
- `state.db_missing` 標記為 `auto_fixable=true`（在常量旁的註解），引導後續 `add-state-recovery` 對齊。

**Alternatives 考量**：
- 不註冊，等 `add-doctor` 一次處理：未來重命名 finding code 是 breaking。否決。

## Implementation Contract

### Observable behavior

| Command | Stdout（人類）| Stdout（`--json`）| Exit | Side effect |
|---|---|---|---|---|
| `speclink init`（fresh git repo，未 init）| `Initialized SpecLink project at <path>` | `{"ok":true,"data":{"project_id":"<uuid>","artifact_root":".speclink","state_root":".git/speclink"},"warnings":[],"requestId":"<uuid>"}` | 0 | 建立 `.speclink/link.yaml`、`.speclink/schemas/`、`.git/speclink/state.db`、`.gitignore`（追加單行） |
| `speclink init`（non-git working dir）| 錯誤訊息 + hint | `{"ok":false,"error":{"code":"project.requires_git","message":"...","hint":"Run `git init` first","retryable":false,"retry_after_ms":null},"requestId":"<uuid>"}` | 2 | 無 |
| `speclink init`（已 init，未加 `--force`）| 錯誤訊息 | `{"ok":false,"error":{"code":"project.already_initialized",...},"requestId":"<uuid>"}` | 7 | 無 |
| `speclink status`（已 init）| 多行 key: value | `{"ok":true,"data":{"project_id":"<uuid>","provider":"local","artifact_root":".speclink","state_root":".git/speclink","git_head":"<sha>","requires_git":true},...}` | 0 | 無 |
| `speclink status`（未 init）| `Not a SpecLink project` | `{"ok":false,"error":{"code":"project.not_initialized",...},"requestId":"<uuid>"}` | 2 | 無 |
| `speclink link <project_id>`（target state.db 含該 project）| `Linked to <project_id>` | `{"ok":true,"data":{"project_id":"<id>"},...}` | 0 | 建立或覆寫 `.speclink/link.yaml` |
| `speclink link <project_id>`（不存在）| 錯誤訊息 | `{"ok":false,"error":{"code":"project.link_target_not_found",...},"requestId":"<uuid>"}` | 2 | 無 |
| `speclink unlink`（已 init）| `Unlinked` | `{"ok":true,"data":{},...}` | 0 | 移除 `.speclink/link.yaml` |

註：`project.not_initialized` 列入 error code，作為 status / unlink 的失敗回應；但在本 change 中沒有「未初始化拒絕」以外的觸發路徑。

### Interface / data shape

**`.speclink/link.yaml` 格式**：

```yaml
version: 1
project_id: <uuid>
instance_id: <uuid>
provider: local
created_at: <RFC3339>
working_dir_fingerprint: <sha256-of-canonicalized-path>
```

**`.git/speclink/` 內容**：

```
.git/speclink/
  state.db           # SQLite WAL mode
  state.db-wal
  state.db-shm
  locks/             # 預留空目錄（後續 change 使用）
```

**`state.db` v1 schema**：

```sql
CREATE TABLE _migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
CREATE TABLE project (
  id TEXT PRIMARY KEY,             -- project_id (uuid)
  instance_id TEXT NOT NULL,        -- instance_id (uuid, rotates on relocate)
  working_dir TEXT NOT NULL,        -- canonicalized path
  created_at TEXT NOT NULL          -- RFC3339
);
```

**Provider trait shape**（`crates/provider/src/lib.rs`）：

```rust
#[async_trait::async_trait]
pub trait ProjectStore: Send + Sync {
    async fn init(&self, opts: InitOptions) -> Result<ProjectInfo, ProviderError>;
    async fn status(&self) -> Result<ProjectStatus, ProviderError>;
    async fn link(&self, project_id: &str) -> Result<ProjectInfo, ProviderError>;
    async fn unlink(&self) -> Result<(), ProviderError>;
    async fn get_link(&self) -> Result<Option<LinkYaml>, ProviderError>;
    async fn save_link(&self, link: &LinkYaml) -> Result<(), ProviderError>;
}
```

### Failure modes

| Code | Exit | Retryable | Triggered when |
|---|---|---|---|
| `project.requires_git` | 2 | false | working dir 非 git repo（`git rev-parse --is-inside-work-tree` 失敗）|
| `project.already_initialized` | 7 | false | `.speclink/link.yaml` 已存在且未加 `--force` |
| `project.not_initialized` | 2 | false | status/unlink/link 時 `.speclink/link.yaml` 不存在 |
| `project.link_target_not_found` | 2 | false | `link <id>` 時 state.db 內無對應 project row |
| Underlying IO/SQLite error | 1 | varies | message 帶 underlying cause；不洩露絕對路徑外的內部細節 |

所有錯誤都走 envelope；stderr 在 non-`--json` 模式才輸出 human-readable 訊息。

### Acceptance criteria

每項都對應 `tests/cli/*.rs` 內一支 integration test（使用 `assert_cmd` + `tempfile`，TDD 先紅燈）：

1. `init_in_fresh_git_repo_writes_two_roots`：tempdir + `git init` + `speclink init` → `.speclink/link.yaml` 存在 + `.git/speclink/state.db` 存在 + `.gitignore` 含 `.speclink/link.yaml` 一行 + exit 0
2. `init_in_non_git_dir_rejects_with_requires_git`：tempdir 不 `git init` + `speclink init --json` → exit 2 + JSON 含 `error.code == "project.requires_git"`
3. `init_when_already_initialized_returns_conflict`：先 init，再 init → exit 7 + `project.already_initialized`
4. `init_with_force_overwrites_link_yaml`：先 init，再 `init --force` → exit 0，`link.yaml` 內 `created_at` 與 `instance_id` 更新
5. `status_after_init_returns_expected_fields`：init 後 `status --json` → JSON `data` 含 project_id、provider="local"、artifact_root=".speclink"、state_root=".git/speclink"、git_head（任意 sha）
6. `status_without_init_returns_not_initialized`：tempdir + `git init` + `status --json` → exit 2 + `project.not_initialized`
7. `status_in_linked_worktree_resolves_state_root_to_main_git_dir`：在 main repo init 後，`git worktree add ../wt`，cd 進 worktree，`status --json` 回的 `state_root` 解析到 main repo 的 `.git/speclink/`（而非 worktree 本地 `.git`）
8. `gitignore_appends_link_yaml_when_file_exists`：tempdir 有既有 `.gitignore` 含 `node_modules` → init 後 `.gitignore` 含原內容 + 新行 `.speclink/link.yaml`，不重複
9. `gitignore_idempotent_on_reinit_force`：force 再 init 不會出現第二個 `.speclink/link.yaml` 行
10. `unlink_removes_link_yaml_but_keeps_state_db`：init 後 unlink → `.speclink/link.yaml` 消失、`.git/speclink/state.db` 仍在
11. `link_to_known_project_writes_link_yaml`：手動寫一個 state.db row 模擬 backup 還原情境 + `link <id>` → `.speclink/link.yaml` 內 project_id 對齊
12. `link_to_unknown_project_returns_not_found`：`link <unknown-id>` → exit 2 + `project.link_target_not_found`
13. `json_envelope_shape_success`：對成功 case 的 `--json` 輸出做 JSON schema 驗證（ok=true、data 為 object、warnings 為 array、requestId 為 UUID）
14. `json_envelope_shape_error`：對失敗 case 的 `--json` 輸出做 JSON schema 驗證（ok=false、error.code 非空、error.retryable 為 bool）
15. `state_db_migration_v1_creates_expected_tables`：init 後直接打開 state.db → 確認 `_migrations` 表有 version=1 row、`project` 表存在且含 1 row

### Scope boundaries

**In scope**：
- `speclink init / status / link / unlink` 四個 subcommand 的完整實作（含 `--json`、`--force` 旗標、help 文案）
- `crates/cli`、`crates/runtime`、`crates/provider`、`crates/provider-local` 四個 crate 的 skeleton + 完整實作上述行為所需的程式碼
- `Cargo.toml` workspace root + 上述四個 crate 的 `Cargo.toml`
- state.db v1 schema + migration runner（runner 設計支援後續 migration，但本 change 只跑 v1）
- `ProjectStore` trait 完整定義（六個 method）+ `LocalProjectStore` 完整實作
- 15 個 integration test
- Error code 三個 + finding code 四個 字串常量註冊

**Out of scope**：
- 任何 change CRUD / artifact / apply / review / archive / discuss / skill / pack / schema CLI 相關行為
- `crates/cli/src/main.rs` 內除四個 subcommand 外的任何其他 subcommand（即便 stub 也不放）
- `doctor`、`restore`、`drift`、`analyze`、`validate` 子命令（finding code 字串常量除外）
- HttpProvider、auth、keychain
- Worktree 自動 marker、跨 working dir 同步
- Performance optimization、async runtime tuning
- CI matrix 設定（GitHub Actions workflow）
- Cross-platform path normalization 之外的平台差異處理

## Risks / Trade-offs

- [Git CLI 不存在或版本過舊] → init 時先 `git --version`，無法 spawn 則回 `project.requires_git` + hint「Install git first」。最低支援 git 2.5（worktree feature 自 2.5 起）。
- [Linked worktree 內 `.git` 是檔案而非目錄] → 必須走 `git rev-parse --git-common-dir` 而非自己拼路徑，已在 decision 中固定。Test #7 覆蓋。
- [SQLite WAL mode 在 NFS / 雲端同步盤 corrupt] → state.db 在 `.git/` 內，正常情境不會被 sync tool 處理。若使用者刻意把整個 `.git/` 放 Dropbox，後續 doctor 會回 `doctor.state.db_corrupted`。本 change 不寫 doctor 但先註冊 finding code。
- [Schema seed 過程被中斷導致 partial init] → 採「prepare-then-commit」順序：先在 tempdir 拼好 `link.yaml` 內容、state.db 跑完 migration，最後一次性把 `link.yaml` rename 進 `.speclink/`；若 rename 前任何步驟失敗則清掉 tempdir，working tree 不留半成品。
- [`--force` 行為過寬，意外覆寫 link.yaml] → 設計上 `init --force` 只覆寫 `link.yaml`、`instance_id` 重生，不刪 state.db；test #4 與 #15 同時覆蓋確保 state 保留。
- [JSON envelope 欄位過早凍結] → `requestId`、`retryable`、`retry_after_ms` 即使 LocalProvider 不用也保留。若後續發現欄位設計錯誤，因尚未發 release 可以改；發 1.0 後屬於 breaking change。
- [Trait skeleton 過早抽象] → 已在 decision 中過 deletion test，且 trait method 數量收斂到本 change 範圍（六個）。每多一個 method 都對應 spec 內一個 requirement，不會無償膨脹。
- [Doctor finding code 註冊但無實作，consumer 誤判已支援] → 常量旁加 doc comment 明示「reserved code，actual check implemented in add-doctor change」。後續 `add-doctor` 啟用前，`speclink doctor` 子命令不存在，consumer 無從觸發。
