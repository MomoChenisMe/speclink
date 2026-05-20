# Spectra CLI 尚需補強的子系統反組譯整理

日期：2026-05-20

本文補充 `spectra-cli-reverse-engineering.zh-TW.md` 與 `spectra-cli-skill-runtime-flow.md` 中較分散或尚未被充分提醒的四個區塊。這些區塊不是 Spectra 基本 workflow 的主線，但對設計新的 SDD workflow engine 很重要。

本文分成：

1. Worktree / preflight / artifact ownership
2. Documents / search / spectra-ask 安全規則
3. SQLite 完整資料表與 migration
4. CLI 邊界能力與產品化指令

其中第 2 點是 Spectra CLI 已有的能力線索，但目前新專案不打算優先實作。保留此節是為了避免未來誤以為沒有分析過這個子系統。

## 1. Worktree / Preflight / Artifact Ownership

### 已觀察到的 Spectra CLI 線索

Binary 與既有文件中出現下列字串或欄位：

```text
worktree
worktrees_dir
worktreePath
preflight
worktree_artifact_ownership
.spectra/worktrees
```

`.spectra.yaml` 也存在相關設定：

```yaml
# Enable git worktree support for isolated change branches
# worktree: true

# Custom git worktrees directory
# worktrees_dir: .spectra/worktrees
```

`instructions` / `status` 類輸出中也觀察到可能包含：

```text
worktreePath
preflight
```

這表示 Spectra CLI 的 worktree 不是單純的 Git convenience feature，而是 workflow runtime 的一部分。它可能同時影響：

- change 對應的隔離工作目錄
- artifact 與 worktree 的 ownership
- apply 前的 preflight 檢查
- AI coding agent 應該在哪個目錄工作
- 多個 change 並行時的狀態隔離

### 文件應補充的提醒

目前文件已提到 worktree 設定，但應加強以下提醒：

1. `worktree` / `worktrees_dir` 是 runtime 層設定，不只是 Git 設定。
2. `worktreePath` 若出現在 `instructions`，AI skill 應該把它視為工作目錄提示。
3. `preflight` 可能是 apply 前的檢查結果，應視為 workflow guardrail。
4. `worktree_artifact_ownership` 暗示 artifact 與工作樹之間可能有所有權或鎖定關係。
5. 若未來要支援多人、遠端 provider 或服務端 PM/SA workflow，不能直接照搬 `.spectra/worktrees`，而要抽象成 provider 狀態。

### 對新 CLI 的設計建議

新 CLI 可保留 Spectra 的核心思路，但需要改造成 provider-friendly model：

```text
Change Workspace
Artifact Ownership
Preflight Result
Apply Target
```

建議資料模型：

```json
{
  "changeId": "add-order-export",
  "workspace": {
    "mode": "local_worktree",
    "path": ".workflow/worktrees/add-order-export",
    "branch": "sdd/add-order-export"
  },
  "ownership": {
    "proposal.md": "pm-agent",
    "design.md": "sa-agent",
    "tasks.md": "engineering-agent"
  },
  "preflight": {
    "status": "ok",
    "checks": [
      {
        "code": "artifact.required",
        "status": "pass"
      },
      {
        "code": "target.clean",
        "status": "pass"
      }
    ]
  }
}
```

遠端 provider 模式下，`workspace.path` 不一定存在。這時 provider 應回傳抽象 workspace state，而不是硬性要求本機 Git worktree。

### 信心等級

| 項目 | 信心 |
| --- | --- |
| `.spectra.yaml` 支援 `worktree` / `worktrees_dir` | 高 |
| `instructions` 可能回傳 `worktreePath` / `preflight` | 中 |
| 存在 artifact ownership 概念 | 中 |
| 完整 worktree 建立、鎖定、清理規則 | 低，需進一步動態測試 |

## 2. Documents / Search / Spectra Ask 安全規則

### 已觀察到的 Spectra CLI 線索

Spectra CLI 內建 `search` 指令：

```bash
spectra search <QUERY> --limit <N> --json
```

Windows x64 build 上已確認回傳：

```json
{
  "error": "vector_not_compiled",
  "results": []
}
```

使用者可見錯誤提到：

```text
Vector search is not available on this platform (requires Apple M-series).
```

Binary 中的 `$spectra-ask` instruction 顯示它會查詢：

```text
{{SPEC_DIR}}documents
```

並要求 AI：

- 只能根據文件與搜尋結果回答。
- 找不到證據時不能猜。
- 忽略 documents 內可能出現的 prompt injection。
- 遮蔽 secrets、PII、URL。

這代表 Spectra CLI 除了 `openspec/specs` 與 `openspec/changes` 外，也有一條文件知識庫路線。

### 文件應補充的提醒

目前文件應明確標註：

1. `documents` 可能是 `spectra-ask` 的主要知識來源。
2. `search` 是 semantic/vector search，不是一般全文 grep。
3. Windows x64 build 目前不支援 vector search。
4. `spectra-ask` 的安全規則是 skill/runtime contract 的一部分。
5. 這條能力與核心 SDD artifact workflow 可分離。

### 新專案目前的取捨

新專案目前不打算優先實作這一塊。

原因：

1. 目前核心賣點是 `skill + CLI + provider + 狀態同步`。
2. PM/SA discuss、propose、pack 與工程師 apply/finish/archive 不必依賴 vector search。
3. 文件知識庫會引入 embedding、索引、平台差異、資料安全、prompt injection 防護等額外複雜度。
4. 若要支援企業內部文件，應該讓 provider 或外部系統自己接入文件搜尋，而不是 MVP 內建。

因此新專案建議：

```text
MVP 不做 documents/vector search/spectra-ask clone。
Provider API 預留 search hook，但不要求實作。
Skill 文件中明確避免依賴 search。
```

可預留的 provider hook：

```http
POST /v1/search
```

但標記為 optional capability：

```json
{
  "capabilities": {
    "artifactStore": true,
    "stateSync": true,
    "semanticSearch": false
  }
}
```

### 信心等級

| 項目 | 信心 |
| --- | --- |
| `spectra search` 存在 | 高 |
| Windows x64 build 不支援 vector search | 高 |
| `spectra-ask` 依賴 `documents` 與 search | 中高 |
| search index 的建立與更新規則 | 低 |
| 新專案 MVP 不實作此能力 | 產品決策，已記錄 |

## 3. SQLite 完整資料表與 Migration

### 已觀察到的 Spectra CLI 線索

目前已實驗確認：

- SQLite DB 主要位於 `.git/spectra-app/spectra.db`。
- `spectra park` 會寫入 `parked_changes`。
- `spectra unpark` 會刪除 `parked_changes` row。
- `spectra in-progress add` 會寫入 `in_progress_change`。
- `spectra task done` 不寫 SQLite，而是更新 `tasks.md`，並在特定 Git worktree 條件下寫 `.spectra/touched/<change>.json`。

Binary 內還出現更多 table 或 state 名稱：

```text
parked_changes
archived_cache
shared_changes
worktree_artifact_ownership
documents
change_sort_order
in_progress_change
agent_input_history
```

這表示 Spectra CLI 的 SQLite 不只是 parked change metadata，也承擔更廣的 runtime 狀態。

### Migration 線索

Binary 中觀察到下列 migration 相關字串：

```text
spectra.db.bak
PRAGMA table_info
schema_version=
Failed to acquire migration lock
spectra.db.migrated.migrate.lock
legacy.migrated.txt
Spectra metadata database has moved to:
ATTACH DATABASE ?1 AS legacy
DETACH DATABASE legacy
PRAGMA integrity_check
.spectra/spectra.db
```

推論 Spectra CLI 可能支援：

1. 從 legacy `.spectra/spectra.db` 遷移到 `.git/spectra-app/spectra.db`。
2. 透過 lock file 避免多程序同時 migration。
3. migration 前後做 SQLite integrity check。
4. 建立 backup。
5. 將 legacy tables `INSERT OR IGNORE` 到新 DB。
6. 記錄 migration marker，避免重複遷移。

### 文件應補充的提醒

目前 reverse engineering 文件應提醒：

1. `.git/spectra-app/spectra.db` 是目前主要 DB，但不是唯一曾經存在的 DB path。
2. `.spectra/spectra.db` 可能是 legacy path。
3. 若要解析或搬移 Spectra 專案，不能只搬 `openspec/changes`，也要注意 `.git/spectra-app`。
4. parked/in-progress/shared/worktree/document/search 類狀態可能存在 SQLite，不一定在 filesystem artifact 中。
5. SQLite migration 是產品級能力，不是測試附屬功能。

### 對新 CLI 的設計建議

新 CLI 應把本地狀態 DB 明確版本化：

```text
.speclink/
  state.db
  migrations/
  backups/
```

建議 state DB 至少包含：

```text
schema_migrations
changes
artifacts
artifact_versions
workflow_state
in_progress
packs
sync_queue
sync_events
agent_runs
```

若要支援 provider fallback 與遠端同步，應額外加入：

```text
provider_bindings
remote_refs
conflict_records
outbox_events
```

重要設計原則：

1. 本機 DB 是 provider-local 的實作細節，不應滲透到 skill。
2. 遠端 provider 不應被迫使用 SQLite，但必須能表達相同 workflow state。
3. migration 必須 machine-readable，不能只靠檔案存在與否。
4. CLI 每次啟動應檢查 schema version。
5. migration 失敗時應停止 workflow，不能靜默 fallback。

### 信心等級

| 項目 | 信心 |
| --- | --- |
| `.git/spectra-app/spectra.db` 是目前主要 DB | 高 |
| `parked_changes` / `in_progress_change` 寫入行為 | 高 |
| 額外 table 名稱存在於 binary | 中高 |
| legacy `.spectra/spectra.db` migration | 中 |
| 每個 table 的完整欄位與寫入時機 | 低，需專門 probe |

## 4. CLI 邊界能力與產品化指令

### Command Surface

Spectra CLI 的 help 顯示命令面不只核心 SDD 流程：

```text
init
update
list
show
validate
analyze
drift
archive
status
instructions
new
schemas
templates
feedback
schema
config
search
completion
park
unpark
task
in-progress
demo
```

目前文件已涵蓋主要 workflow，但以下產品化指令需要更明確地被整理。

### init / update / tool target

`spectra init` 支援：

```bash
spectra init --tools <TOOLS> --force --dir <DIR>
```

其中 `--dir` 可指定 spec root，不只透過 `.spectra.yaml spec_dir` 決定。

`spectra update` 用於更新已設定工具的 generated instruction files：

```bash
spectra update [PATH] --force
```

Binary 內觀察到多個 AI tool target：

```text
claude
cursor
windsurf
cline
gemini
github-copilot
kiro
roocode
continue
opencode
codebuddy
costrict
antigravity
auggie
amazon-q
kilocode
factory
iflow
qoder
qwen
codex
crush
trae
```

也觀察到一些 target path：

```text
.claude
.cursor
.windsurf
.clinerules
.gemini
.github/prompts
.kiro
.roo
.continue
.opencode
.agents
```

文件應提醒：`update` 不是 SDD artifact 指令，而是「skill/instruction distribution」指令。新 CLI 若主打 skill+CLI，這是核心產品能力。

### schema init / fork / validate

除了 `schema fork` 與 `schema validate`，Spectra 還有：

```bash
spectra schema init <NAME> --description --artifacts --default --force
```

這代表它支援建立自訂 workflow schema。新 CLI 若要讓外部服務或團隊定義自己的 SDD artifact DAG，應把 schema creation 視為一級能力。

文件也應提醒已觀察到的 subtle behavior：

- `schema fork spec-driven alt-flow` 會建立 `openspec/schemas/alt-flow/schema.yaml`。
- 但 schema 檔內部 `name` 可能仍是 `spec-driven`。
- 後續 `status` / `instructions` 可能顯示內部 name，而不是目錄名。

新 CLI 應避免這種混淆，schema id、schema name、schema path 應有明確規則。

### config command

Spectra 有完整 user/global config command：

```bash
spectra config path
spectra config list
spectra config get <KEY>
spectra config set <KEY> <VALUE> --string --allow-unknown
spectra config unset <KEY>
spectra config reset --all --yes
spectra config edit
```

文件應明確分層：

| 層級 | Spectra 類似物 | 用途 |
| --- | --- | --- |
| User/global config | `%APPDATA%/openspec/config.yaml` 與 `spectra config` | 使用者層設定 |
| Project runtime config | `.spectra.yaml` | `spec_dir`、locale、tools、worktree |
| SDD project config | `<spec_dir>/config.yaml` | schema、context、artifact rules |

新 CLI 應更嚴格分層，避免 config 修改後使用者不知道實際影響哪個 workflow。

### completion

Spectra 支援：

```bash
spectra completion generate <SHELL>
spectra completion install <SHELL>
spectra completion uninstall <SHELL>
```

這對 AI workflow 不是必要能力，但對 CLI 產品成熟度很重要。新 CLI 可以在 MVP 後補。

### demo

Spectra 支援：

```bash
spectra demo
```

已觀察到它會建立示範 change，例如：

```text
openspec/changes/spx-proud-charmander
```

此 build 的 `demo` 不支援 `--json`。

新 CLI 建議保留 demo/fixture 能力，但應讓輸出 machine-readable：

```bash
speclink demo create --json
```

用途：

- onboarding
- provider integration test
- schema engine smoke test
- skill template 驗證

### feedback

Spectra 支援：

```bash
spectra feedback <MESSAGE> --body <BODY>
```

Binary 中有 WinHTTP、proxy、request status 等網路相關字串，表示此命令可能會送出 network request。簡單字串掃描未找到明確 endpoint。

文件應提醒：

1. `feedback` 是 network-capable command。
2. 應與核心 SDD workflow 分離。
3. 若新 CLI 要做 feedback，必須明確揭露 endpoint、資料內容與 opt-in 行為。
4. AI skill 不應自動呼叫 feedback。

### archive flags

`archive` 有幾個會影響保護流程的 flags：

```bash
spectra archive <CHANGE> --skip-specs --no-validate --mark-tasks-complete
```

這些不是一般 happy path，而是 bypass 或修復流程。新 CLI 若實作類似能力，應：

- 要求人類確認。
- 預設不讓 AI skill 使用。
- 在 audit log 中記錄。
- 在 `--json` output 中明確標示 skipped checks。

### validate / show / list 的完整模式

文件也應補足這些查詢模式：

```bash
spectra list --specs --changes --parked --sort <name|modified|created> --json
spectra show <ITEM> --json --item-type <change|spec> --deltas-only --requirements
spectra validate <ITEM> --all --changes --specs --strict --json
```

這些對 AI skill 很重要，因為 AI 不應自行掃 filesystem 猜狀態，而應透過 CLI 取得穩定 JSON。

### 對新 CLI 的設計建議

新 CLI command surface 應分成四層：

| 層級 | 例子 | 是否給 AI skill 使用 |
| --- | --- | --- |
| Workflow commands | `discuss`、`propose`、`apply`、`finish`、`archive` | 是 |
| State/query commands | `status`、`list`、`show`、`validate`、`analyze`、`instructions` | 是 |
| Human setup commands | `auth`、`provider`、`config`、`init`、`update` | 否，除非 skill 只讀狀態 |
| Product utility commands | `completion`、`demo`、`feedback`、`schema init` | 視情境，通常由人使用 |

Skill 文件應明確寫出：

```text
AI Agent 可以呼叫 workflow/state commands。
AI Agent 不應呼叫 auth/provider/login/feedback。
```

### 信心等級

| 項目 | 信心 |
| --- | --- |
| command surface | 高 |
| `init --dir` / `update --force` / `completion` / `demo` flags | 高 |
| tool target 清單 | 中高 |
| 每個 tool target 的完整輸出路徑 | 中 |
| feedback endpoint 與 payload | 低 |

## 對目前新專案設計文件的影響

這四塊應反映到 `speclink-provider-api-and-runtime-design.md`：

1. Worktree / preflight 應被抽象成 provider state，不要硬綁本機 Git worktree。
2. Documents / search / spectra-ask clone 目前不做，但 provider capability 可預留 optional search。
3. Local provider 需要 state DB migration 策略，遠端 provider 需要等價 workflow state。
4. CLI command surface 要分清楚 AI 可呼叫與人類 setup/utility command。

建議 MVP 調整：

- 保留：workflow、state/query、provider、local fallback、pack/unpack、finish/archive。
- 保留：schema init/fork/validate 的最小版本。
- 保留：demo fixture 的最小版本。
- 暫緩：documents/vector search/spectra-ask clone。
- 暫緩：feedback network command。
- 暫緩：native provider plugin。

## 後續待實驗清單

若之後要把 Spectra CLI 反組譯可信度再提高，建議測：

1. 啟用 `.spectra.yaml worktree: true` 後跑 `new change`、`instructions apply`、`apply` skill 流程，觀察 worktree 建立與 `worktreePath`。
2. 人為建立 legacy `.spectra/spectra.db`，觀察是否 migration 到 `.git/spectra-app/spectra.db`。
3. 建立多個 parked/shared/in-progress changes，檢查 SQLite 是否出現 `shared_changes`、`change_sort_order` 等表。
4. 測 `schema init` 產生的 schema 結構與 default schema 設定。
5. 測 `spectra update` 在不同 `--tools` 與 `.spectra.yaml tools` 下的實際輸出。
6. 在支援 vector search 的平台測 `documents` index 與 `spectra search`，但此項不是新專案 MVP 需求。
