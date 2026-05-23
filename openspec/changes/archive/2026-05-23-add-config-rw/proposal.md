## Why

A3 `add-state-machine-and-apply` 把 `require_artifact_review` / `require_code_review` 兩個 review flag 硬編為 `false`、A4 `add-archive` 同樣靠這條 walking-skeleton 4-state 路徑收檔；config.yaml 至今沒有任何讀寫 op，整個 SDD engine 無法依使用者意願 toggle review 流程，後續 `add-review` slice 也無法接通 6-state lifecycle。Walking skeleton 第五片必須把 `config.read` / `config.write` 兩個 op、`ConfigStore` trait、state.db v5 migration（config etag cache + config_change audit log）、A3 hardcode 改讀 config 一次補齊，讓使用者可端到端跑「`speclink config set rules.require_code_review true` 後下一次 `archive` 拒絕未過 review 的 change」這條最小可示範路徑，為 `add-review` slice 提供 review-flag toggle 基底。

設計依據 `doc/speclink-design.md` §11（Config 結構）、§11.6（Malformed 行為）、§12.7（migration flow）、§16.4（config CLI）、§17.6（audit event 持久化），以及 `doc/protocol/operations.md` 的 `config.read` / `config.write` op spec。設計討論 conclusion 已於對話確認：YAML 為 SOT、state.db v5 為 etag cache + CAS token + audit log；mode=edit 的 `keys_changed` 採簡化 `["__edit__"]` 不做 deep diff；外部編輯（user 手動 vim config.yaml）以 `state.etag_mismatch` 拒絕並寫 audit；role config 完整解析、`${VAR}` env interpolation、schema 嚴格驗證、`instructions.get` 真實 rule 注入皆推遲到後續 slice。

## What Changes

- 新增 CLI 指令（皆屬「人類設定階段」與「AI workflow 階段」共用，可由 skill 呼叫）：
  - `speclink config show [--key <jsonpath>] [--json]` — 讀 config.yaml，回 `Versioned<Config>` 結構；`--key` 用 JSONPath subset 提取部分（grammar：`a.b.c` / `a[0].b`，與 §11.4 對齊）
  - `speclink config set <key> <value> [--expected-etag <etag>] [--json]` — 對應 `config.write` mode=set；`<value>` 解析順序為 `true/false/null` → integer → float → string raw（不解析 JSON literal、避免 shell 引號歧義）
  - `speclink config edit [--editor <cmd>] [--expected-etag <etag>] [--json]` — 對應 `config.write` mode=edit；launch `$EDITOR`（或 `--editor` 指定）載入當前 config.yaml、保存後讀回完整 content；non-TTY 環境（如 CI / skill 自動化）必須走 `--stdin` mode（見下一行）
  - `speclink config edit --stdin [--expected-etag <etag>] [--json]` — 從 stdin 讀完整 YAML 內容覆寫；給 skill / CI / 自動化使用
- 新 op（依 `doc/protocol/operations.md` L1482–1664 spec）：
  - `config.read` — read-only、idempotent、無 lock；回 `Versioned<Config> = { value, etag }`
  - `config.write` — `oneOf { mode='set', key, value, expected_etag? } | { mode='edit', content, expected_etag? }`；取 `global-short` lock；回更新後的 `Versioned<Config>`
- State.db v5 migration（forward-only，遵守 §12.7 migration flow）：
  - 新表 `config_state`（singleton row）：`id INTEGER PRIMARY KEY CHECK (id = 1) / content_sha256 TEXT NOT NULL / size_bytes INTEGER NOT NULL / mtime_ns INTEGER NOT NULL / version INTEGER NOT NULL DEFAULT 1 / updated_at TIMESTAMP NOT NULL / written_by TEXT NULL`；用途 = config.yaml etag cache + CAS token
  - 新表 `config_change`（audit log）：`change_seq INTEGER PRIMARY KEY AUTOINCREMENT / changed_at TIMESTAMP NOT NULL / mode TEXT NOT NULL CHECK (mode IN ('set', 'edit', 'external_edit')) / keys_changed TEXT NOT NULL / etag_before TEXT NULL / etag_after TEXT NOT NULL / actor_json TEXT NULL / reason TEXT NOT NULL CHECK (reason IN ('config_write', 'config_external_edit'))`
  - `_migrations` 表寫入 version=5 row
  - **不**新增其他表；不動既有 `project` / `change` / `state_transition` 表
- ConfigStore trait 在 `crates/provider/src/config_store.rs`：
  - `fn read_config(&self) -> Result<Versioned<Config>>` — 同時 reconcile file vs db etag（若不一致、寫一筆 `config_change(reason='config_external_edit', mode='external_edit')` audit row、bump version、回新 etag）
  - `fn write_config(&self, request: WriteConfigRequest) -> Result<Versioned<Config>>` — mode=set / edit；內部完成 CAS check + file write + db update + audit insert，全在同一 SQLite tx
  - `fn read_defaults(&self) -> Config` — 回 walking-skeleton 預設值（`require_artifact_review=false` / `require_code_review=false` 等）；config.yaml 不存在或 malformed 時 fallback 用
- A3 hardcode 接通 config（呼應 §11.6「Malformed = warning + fallback」）：
  - `crates/runtime` state machine evaluator 不再硬編 review flags；改呼叫 `Provider::read_config()` 取得 `value.rules.require_artifact_review` / `value.rules.require_code_review`
  - Config 不存在 → 用 `read_defaults()`（4-state walking-skeleton mode）
  - Config 存在但 malformed（YAML 解析失敗 / required key 缺失 / 型別不符） → 用 `read_defaults()` + emit JSON envelope warning `config.malformed_using_defaults`（**不** raise error、不阻斷 state transition；只是 walking-skeleton 行為）
- 新 error codes（design §17.4 命名規則）：
  - `config.not_found`（exit 2）— `config.read` 找不到 config.yaml；A1 init 之後罕見、僅作 defensive guard
  - `config.malformed`（exit 3）— `config.write` mode=edit 收到的 content 解析失敗、或 mode=set 套用後 schema 不過；read path 不抛此 error（read path 走 warning + fallback）
  - `config.key_not_found`（exit 2）— mode=set 的 `key` JSONPath 不存在
  - `state.etag_mismatch`（exit 7）— `expected_etag` 與當前 etag 不符（沿用 A2 既有 code、不新增）
- JSON envelope：沿用 A1–A4 既有 `{ ok, data, warnings?, requestId }` / `{ ok: false, error: { code, message, hint?, retryable, retry_after_ms? }, requestId }` shape；新 data shape：
  - `config.read` data = `{ value: Config, etag: string }`；`--key` 模式 data = `{ key: string, value: <leaf>, etag: string }`
  - `config.write` data = `{ value: Config, etag: string }`；mode=set 額外帶 `keys_changed: [<key>]`
- A1 `project-bootstrap` `speclink init` 流程擴充：init 成功後在同一 prepare/commit phase 內 insert `config_state(id=1, content_sha256=<新建 config.yaml 的 sha256>, size_bytes=<bytes>, mtime_ns=<新檔 mtime>, version=1, updated_at=now, written_by=NULL)`；init 失敗 rollback 時連帶清除 config_state row
- A2 `artifact-io` etag 命名格式不受影響（沿用 `v<version>.<sha256[:12]>`）；A5 對 config etag 採同樣 grammar：`v<version>.<sha256[:12]>`
- Lock：依 op spec 取 `global-short` lock；A5 沿用 A3/A4 stub no-op lock 路徑（真實 advisory lock 由 `add-locking-and-concurrency` slice 接），單 SQLite tx 保證原子性

使用者情境：本 slice CLI 跨「人類設定階段」（user 互動 `config set` / `config edit`）與「AI workflow 階段」（skill 透過 `config.read` 取 rules）；無 CI 專屬 op；無工程師本機 apply 階段 op。

## Non-Goals

- ❌ Role config 完整解析與 schema 驗證 — 推遲到 `add-discuss` slice（discuss 才會用到 role 機制；A5 把 roles 視為 opaque map、保留欄位但不解析）
- ❌ `${VAR}` env interpolation 嚴格規範 — 推遲到 `add-http-provider` slice（HttpProvider 才需要 token interpolation；A5 純 LocalProvider 不解析 env var、interpolation 字元視為 literal）
- ❌ `instructions.get` 真實 rule 注入 — 推遲到 `add-instructions` slice；A5 只保證 config 可讀寫，`spectra instructions <artifact>` 既有路徑（A1 已部署）暫不改動
- ❌ Schema 嚴格驗證 — 推遲到 `add-schema-management` slice；A5 mode=edit 只跑 syntactic YAML parse + 已知 leaf path 型別檢查（`rules.require_*_review` 必為 bool；`project.id` 必為 string；其他欄位接受 opaque value）
- ❌ Deep diff for mode=edit — A5 採簡化 `keys_changed = ["__edit__"]`；deep diff 等 audit query CLI（`speclink audit list / show`）真正需要 query keys diff 時再加
- ❌ 真實 advisory lock acquisition — 沿用 A3/A4 stub no-op；真實 lock 由 `add-locking-and-concurrency` slice 接
- ❌ `speclink audit list / show` CLI — A5 只寫 `config_change` 表、不查；audit query CLI 留後續 slice
- ❌ `speclink config delete <key>` / `config unset <key>` — `key` 移除語意 MVP 內無需求（user 可走 `config edit` 全檔覆寫）；未來真有需求再加
- ❌ `config.changed` audit event 對外 telemetry / Slack / webhook — 純本機 audit row；對外 hook 不在 MVP
- ❌ HttpProvider `ConfigStore` impl — Trait 加進 `crates/provider`、`crates/provider-local` 提供唯一 impl；HttpProvider 路徑留 `add-http-provider` slice
- ❌ Config 版本 migration（YAML schema 演進） — A5 config.yaml schema 為 v1 固定；未來 schema 演進靠 `add-schema-management` slice 的 schema version + migration runner，A5 不接

## Capabilities

### New Capabilities

- `config-rw`: `config.read` / `config.write` 兩個 op + ConfigStore trait + config etag cache + config_change audit log；YAML 為 SOT、state.db v5 為 cache + CAS + audit；mode=set / edit 雙寫入路徑；external_edit detection；walking-skeleton fallback semantics（malformed config = warning + defaults）

### Modified Capabilities

- `local-storage-layout`: state.db schema 升至 v5；新增 `config_state` singleton 與 `config_change` audit 兩張表；`_migrations` 記錄 v5
- `state-machine`: `require_artifact_review` / `require_code_review` 兩個 review flag 從硬編改為 `Provider::read_config()` 取得；config 不存在或 malformed → fallback `read_defaults()` + warning
- `project-bootstrap`: `speclink init` 流程於同一 prepare/commit phase 內預先 insert `config_state(id=1, version=1, sha256=<新建 config.yaml sha>)` row；init rollback 連帶清除該 row

## Impact

- Affected specs:
  - 新建：`openspec/specs/config-rw/spec.md`
  - 修改：`openspec/specs/local-storage-layout/spec.md`、`openspec/specs/state-machine/spec.md`、`openspec/specs/project-bootstrap/spec.md`
- Affected code:
  - New:
    - `crates/provider/src/config_store.rs`（`ConfigStore` trait + `Config` / `WriteConfigRequest` / `ConfigPatchKey` 型別）
    - `crates/provider-local/src/config_store.rs`（`LocalConfigStore` 實作 + YAML parse + file CAS + audit insert）
    - `crates/provider-local/tests/config_store.rs`（紅燈先寫的 5 條測試：happy read / write set CAS / write edit CAS / external_edit detection / malformed fallback）
    - `crates/cli/src/commands/config.rs`（`config show` / `config set` / `config edit` 三條 subcommand + stdin path）
    - `crates/cli/tests/config_cli.rs`（CLI 整合測試：JSON envelope shape、exit code、stdin path）
    - `crates/provider-local/src/migrations/v5_config_tables.sql`（migration SQL）
  - Modified:
    - `crates/provider-local/src/state_db.rs`（註冊 v5 migration + checksum；既有 migration runner 住處）
    - `crates/runtime/src/state_machine.rs`（review flag 改讀 config + warning emit）
    - `crates/provider-local/src/store.rs`（init prepare/commit phase 插入 config_state row；既有 LocalStore 住處）
    - `crates/cli/src/commands/mod.rs`（註冊 `config` subcommand group）
    - `crates/provider/src/lib.rs`（`Provider` trait 加 `config_store` 入口）
    - `doc/speclink-design.md`（§1.1 walking skeleton 表加 A5 row；§11.x 對 etag 公式 / external_edit semantics / state.db v5 兩表 schema 做 cross-reference）
    - `doc/protocol/operations.md`（`config.read` / `config.write` 兩 op 把「reserved (add-config-rw slice)」改 `implemented (A5)`）
  - Removed: 無
