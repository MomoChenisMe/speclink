## Context

#### 當前狀態

- A1–A4 walking skeleton 已落地：project bootstrap、change/artifact I/O、6-state lifecycle、archive 全綠
- A3 `crates/runtime/src/state_machine.rs` 把 `require_artifact_review` / `require_code_review` 兩個 review flag 硬編為 `false`，跑 walking-skeleton 4-state mode（`proposing → ready → in_progress → archived`）
- A4 archive 流程依賴 A3 的 hardcode flag 決定能否走捷徑進 `archived`（`require_code_review=false` + `all_tasks_done=true`）
- `config.yaml` 由 A1 init 階段生成預設模板，但 **沒有任何 op 可讀寫**；user 只能手動 `vim .speclink/config.yaml`，engine 不感知
- `doc/protocol/operations.md` 對 `config.read` / `config.write` 已有完整 op spec（L1482–1664）；MVP 標記 ✓，僅缺實作

#### 設計來源

- `doc/speclink-design.md`：§11（Config 結構）、§11.6（Malformed 行為）、§12.7（migration flow）、§16.4（config CLI）、§17.6（audit event 表）
- `doc/protocol/operations.md`：`config.read`（L1482）、`config.write`（L1550）
- A3 `state_transition` audit 表為「audit-table-parallel-to-state.db」模式 — A5 沿用同模式對 config 寫入做 audit log

#### 約束

- YAML 必須維持唯一 SOT（user 手動 vim 是合法路徑、design §11 「config 一檔到底」）；state.db 不可成為 config 真值來源
- 並發安全靠 `change.version` CAS（A2/A3）+ SQLite tx 原子性；A5 沿用同範式對 config 做 CAS，不引入 advisory lock（lock 由 `add-locking-and-concurrency` slice 接）
- 跨平台：`mtime_ns` 在 Windows / macOS / Linux 精度不一致（Windows FAT32 = 2s、ext4 = 1ns、APFS = 1ns）— 不可作為 CAS 唯一依據、必須跟 sha256 雙重比對
- Walking-skeleton 原則：malformed config = warning + fallback、不阻斷 engine（design §2 第 11 條）

## Goals / Non-Goals

#### Goals

- 提供 `config.read` / `config.write` 兩個 op + `ConfigStore` trait + state.db v5 migration，覆蓋 walking-skeleton mode 下 config-driven review-flag toggle 路徑
- 偵測 user 手動編輯 config.yaml 的 external-edit 情境，使 `expected_etag` CAS 對「engine 寫入後 user 又 vim 改檔」這條 race 路徑可拒絕、不靜默吃改
- 為 A5 之後的 `add-review` slice 提供 review-flag 真實 toggle 基底，不再硬編
- 把 A3 review-flag hardcode 一次性接通 config，避免在 review slice 偷渡 config-rw 工作

#### Non-Goals

- 不實作 advisory lock（沿用 A3/A4 stub no-op）
- 不實作 role config / `${VAR}` interpolation / schema 嚴格驗證 / `instructions.get` 真實 rule 注入
- 不實作 `config.changed` 對外 telemetry / webhook
- 不為 mode=edit 做 deep diff（`keys_changed = ["__edit__"]` 簡化）
- 不實作 HttpProvider `ConfigStore` impl（trait 加進 `crates/provider`、impl 留 HttpProvider slice）
- 不引入 config.yaml schema 版本演進機制（schema 演進等 `add-schema-management` slice）

## Decisions

### State.db v5 cache vs YAML SOT 取捨

採「YAML = SOT、state.db v5 = cache + CAS + audit」雙層架構。

- 原因 1：design §11 明列 config 一檔到底；user 手動 vim 是合法操作，db 不能成為唯一真值
- 原因 2：cache 必要性 — 高頻 op（state_machine evaluator 每次 transition 都需讀 `rules.require_*_review`）若每次 re-parse YAML，I/O + parse cost 顯著；db 列邊欄位給 fast etag check
- 原因 3：CAS 必要性 — `expected_etag` 機制要求穩定 token；單靠 file mtime 在 FAT32 / NFS 上 race window 過大、單靠 sha256 每次重算 cost
- 替代方案 A（db = SOT、YAML = export）：被否決 — 違反 §11 設計、user 手動編輯路徑被破壞
- 替代方案 B（無 db cache、純檔案 CAS）：被否決 — race window 大、audit log 無持久化位置（design §17.6 audit 表義務）

### Config etag 命名格式對齊 artifact etag

採 `v<version>.<sha256[:12]>` 格式（與 A2 artifact etag 相同 grammar）。

- 例：`v3.a1b2c3d4e5f6`
- `<version>` = `config_state.version`（每次 write 累加）
- `<sha256[:12]>` = `config_state.content_sha256` 前 12 字元（hex）
- 原因：A2 已建立此格式、user 端對 etag 字串有一致心智模型；replace_all_matching version-string 等工具相容；CLI `--json` output 一致

### Config_state singleton 表 via CHECK 約束

採單列表設計：`CREATE TABLE config_state (id INTEGER PRIMARY KEY CHECK (id = 1), ...)`。

- 原因：config.yaml 每 project 唯一、不需要多列；CHECK 比 trigger 簡單；migration v5 prepare 階段 `INSERT INTO config_state (id, ...) VALUES (1, ...)` 直接寫死、後續 op 永遠 UPDATE WHERE id=1
- 替代方案（無 id column、用 `LIMIT 1`）：被否決 — 無 PK、無法防 race insert 第二列

### Config_change audit 表設計沿 A3 state_transition 範式

```sql
CREATE TABLE config_change (
  change_seq    INTEGER PRIMARY KEY AUTOINCREMENT,
  changed_at    TIMESTAMP NOT NULL,
  mode          TEXT NOT NULL CHECK (mode IN ('set', 'edit', 'external_edit')),
  keys_changed  TEXT NOT NULL,  -- JSON array
  etag_before   TEXT NULL,
  etag_after    TEXT NOT NULL,
  actor_json    TEXT NULL,
  reason        TEXT NOT NULL CHECK (reason IN ('config_write', 'config_external_edit'))
);
```

- 與 A3 `state_transition` 表結構平行：autoincrement seq + timestamp + actor + reason 列舉值；未來 audit query CLI 可一條 SQL union 兩表
- `keys_changed` 採 JSON array text、不展平 columns；mode=set 寫 `["rules.require_code_review"]`、mode=edit 寫 `["__edit__"]`、external_edit 寫 `["__external_edit__"]`
- `etag_before` 在 external_edit 情境為 db 紀錄的舊 etag（user 編輯前 engine 認知的 etag）；`etag_after` 為 reconcile 後新 etag

### External-edit detection algorithm

`read_config()` 每次呼叫先 reconcile file vs db：

1. 讀檔 bytes → 計算 sha256 + size
2. 讀 `config_state(id=1)` → 取 `content_sha256` / `size_bytes` / `version`
3. 若兩者一致 → 直接回 `Versioned<Config>{ value, etag=v<version>.<sha[:12]> }`
4. 若 size 或 sha 不一致 → 視為 external_edit：
   a. 開 SQLite tx
   b. UPDATE `config_state` SET content_sha256=新, size_bytes=新, mtime_ns=新, version=version+1, updated_at=now, written_by=NULL WHERE id=1 AND version=expected
   c. INSERT `config_change(mode='external_edit', keys_changed='["__external_edit__"]', etag_before=舊, etag_after=新, actor_json=NULL, reason='config_external_edit')`
   d. tx commit
   e. 回 `Versioned<Config>{ value, etag=新 }` + 在 JSON envelope `warnings` 加 `config.external_edit_detected`
- `read_config()` 不 raise error；reconcile 自動完成、user 端只看到 warning
- `write_config(expected_etag=Some(舊))` 在 reconcile 後若 `expected_etag != 新` → `state.etag_mismatch`（write path 才會 fail）

### Walking-skeleton fallback semantics for malformed config

`read_config()` 對 malformed config 採 warning + defaults，**不** raise error：

- YAML parse fail / required key 缺失 / `rules.require_*_review` 型別不符 → 用 `read_defaults()` 回 `Versioned<Config>{ value=defaults, etag="v0.malformed-fallback" }` + warning `config.malformed_using_defaults`
- `write_config(mode='edit', content=<malformed>)` → 走 strict path、raise `config.malformed` error；read path 與 write path semantics 不同（design §11.6 對齊）
- 原因：read path 必須永遠回 value，否則整個 engine 卡死（state_machine evaluator 每次 transition 都讀 config）；write path 必須 strict、不能讓 user 寫入無效 config

### `keys_changed` 對 mode=edit 採簡化標記

mode=edit 寫入 `keys_changed = ["__edit__"]`，不解析新舊兩份 YAML 做 deep diff。

- 原因 1：deep diff 需要 YAML AST diff library（如 `serde_yaml` AST + 自寫 walker）— A5 不引入新 dep
- 原因 2：mode=edit 通常代表「大規模重寫」、key-level diff 反而資訊冗餘
- 原因 3：audit query CLI（`speclink audit list / show`）尚未存在；deep diff 結果無消費者
- 替代方案（mode=edit 做 deep diff）：被否決 — YAGNI、會引入新 dep + 顯著實作成本
- 未來若 audit query CLI 真的需要：可在 `add-audit-query` slice 加 `--deep-diff` flag、retroactively 重算

### JSONPath subset grammar 與 CLI value 解析規則

- `--key` / `<key>` grammar：`segment ( '.' segment | '[' index ']' )*`、`segment = [a-zA-Z_][a-zA-Z0-9_-]*`、`index = [0-9]+`；不支援 wildcard / filter / recursive descent
- `<value>` 解析（CLI `config set` 用）：依序嘗試 `true/false/null` literal → integer → float → string raw（如 `1.5` 解析為 float、`abc` 解析為 string、`"1.5"` 仍解析為 string `1.5`）
- 不解析 JSON literal（如 `[1,2,3]` 視為 string「[1,2,3]」）— 避免 shell quote 歧義；若需寫 array / object → 走 `config edit` 全檔覆寫

### A1 init prepare/commit phase 插入 config_state row

A1 `speclink init` 流程於 prepare/commit phase 內、與 project row 同 tx 完成 config_state insert：

1. Init prepare 階段：解析 `.speclink/config.yaml` 預設模板 → 計算 sha256 / size / mtime_ns
2. Init commit 階段（同 SQLite tx）：INSERT project row + INSERT `config_state(id=1, content_sha256=<新建 sha>, size_bytes=<bytes>, mtime_ns=<mtime>, version=1, updated_at=now, written_by=NULL)`
3. Rollback path：若 init 任一步驟失敗 → SQLite tx rollback 連帶清除 config_state row（無需額外清理邏輯）
- 替代方案（lazy insert：第一次 `config.read` 時 insert）：被否決 — race window 大（多 agent 同時 init / read 會打架）；migration v5 上線時舊 project 無 row 也容易混亂

### Lock 沿用 stub no-op

A5 `config.write` 取 `global-short` lock 的呼叫點保留、但底層仍是 A3/A4 已有的 stub no-op；單 SQLite tx + `expected_etag` CAS 提供 atomicity。

- 原因：`add-locking-and-concurrency` slice 將替換 stub 為真實 advisory lock；本 slice 不重複實作
- Risk：multi-agent race window（兩 agent 同時 `config set` 不同 key）— 由 SQLite tx + CAS 收斂，最後寫入勝出、輸家收 `state.etag_mismatch` 重試一次即可

## Implementation Contract

#### 觀察行為

- `speclink config show --json` 回 `{ ok: true, data: { value: <Config>, etag: "v<n>.<sha[:12]>" } }`，exit code 0
- `speclink config show --key rules.require_code_review --json` 回 `{ ok: true, data: { key: "rules.require_code_review", value: false, etag: "v1.abc123def456" } }`
- `speclink config set rules.require_code_review true --json` → exit 0、回 `{ ok: true, data: { value: <patched Config>, etag: "v2.<新 sha>", keys_changed: ["rules.require_code_review"] }, warnings: [] }`
- `speclink config edit --stdin --json` < `<new yaml>` → exit 0、回 `{ ok: true, data: { value: <new Config>, etag: "v<n>.<sha>" } }`
- `speclink config set rules.require_code_review true --expected-etag v1.WRONG --json` → exit 7、回 `{ ok: false, error: { code: "state.etag_mismatch", message: "...", retryable: true } }`
- `speclink config set rules.unknown_key true --json` → exit 2、回 `{ ok: false, error: { code: "config.key_not_found", ... } }`
- `speclink config edit --stdin --json` < `<malformed yaml>` → exit 3、回 `{ ok: false, error: { code: "config.malformed", ... } }`
- User 手動 `vim .speclink/config.yaml` 改了內容 → 下次 `config show` 仍成功回新 value、但 `warnings` 包含 `config.external_edit_detected`、db `config_change` 表多一筆 `mode='external_edit'` row
- State machine evaluator 在 `require_artifact_review: true` 設定下、DAG 齊全時自動進 `reviewing`（不再自動進 `ready`）
- Config.yaml 不存在 → `config.read` 回 defaults + warning `config.malformed_using_defaults`（**不** raise error）
- Config.yaml YAML 解析失敗 → 同上（read path warning + fallback；write path raise `config.malformed`）

#### 介面 shape

`ConfigStore` trait（`crates/provider/src/config_store.rs`）：

```rust
pub trait ConfigStore {
    fn read_config(&self) -> Result<Versioned<Config>, ProviderError>;
    fn write_config(&self, request: WriteConfigRequest) -> Result<Versioned<Config>, ProviderError>;
    fn read_defaults(&self) -> Config;
}

pub enum WriteConfigRequest {
    Set { key: JsonPath, value: ConfigValue, expected_etag: Option<Etag>, actor: Option<ActorJson> },
    Edit { content: String, expected_etag: Option<Etag>, actor: Option<ActorJson> },
}

pub struct Config {
    pub rules: Rules,
    pub roles: serde_yaml::Value,  // opaque map；A5 不解析、roles slice 才接
    // 其他欄位…
}

pub struct Rules {
    pub require_artifact_review: bool,  // default false
    pub require_code_review: bool,      // default false
}
```

`Provider` trait 加入口（`crates/provider/src/lib.rs`）：

```rust
pub trait Provider {
    fn config_store(&self) -> &dyn ConfigStore;
    // 既有 method 不動
}
```

#### 命令輸出

- 所有 `config.*` op JSON envelope 沿用 A1–A4 shape
- Human-output renderer（A4 `cli-human-output` slice 已落地）負責把 `value` map pretty-print；A5 不額外加 renderer 規則
- `config show --key <path>` 在 human mode 印 `<key> = <value> (etag=<etag>)`、JSON mode 回 `{ key, value, etag }`

#### 失敗模式

- `config.not_found`（exit 2）— config.yaml 檔案不存在、read path 走 fallback 不抛此 error；僅 `write_config(mode='set')` 在 file 真的不存在時抛（罕見、init 之後不該發生）
- `config.malformed`（exit 3）— write path content 解析失敗 / schema 不符
- `config.key_not_found`（exit 2）— mode=set 的 `key` JSONPath 不存在於現 config
- `state.etag_mismatch`（exit 7）— `expected_etag` 不符 / external edit 後 user 沒重讀就寫
- `provider.connection_failed`（exit 5）— HttpProvider 用、LocalProvider 路徑不抛
- 所有 error 沿用 A1+ 既有 envelope shape：`{ ok: false, error: { code, message, hint?, retryable, retry_after_ms? }, requestId }`

#### 接受條件

- `crates/provider-local/tests/config_store.rs` 5 條核心測試全綠：(a) happy read（fresh state.db、yaml 存在）、(b) write set CAS（expected_etag 正確 → success、不正確 → state.etag_mismatch）、(c) write edit CAS（同上、content stdin path）、(d) external_edit detection（直接修改檔案 bytes、下次 read 寫一筆 audit row + warning）、(e) malformed fallback（YAML 故意寫壞、read 回 defaults + warning）
- `crates/cli/tests/config_cli.rs` 整合測試覆蓋 `config show` / `config set` / `config edit --stdin` 三條路徑 + 5 個 error code + exit code 對照表
- `crates/runtime` state machine evaluator 整合測試：`config set rules.require_artifact_review true` 後、artifact.write 後不再進 `ready` 而進 `reviewing`；`config set rules.require_code_review true` 後、所有 task done 後不再自動進 `archived` candidate 而進 `code_reviewing`（本 slice 不接 review.approve、停在此 state 即可）
- A1 `speclink init` 整合測試：init 後直接查 `state.db.config_state` 應有 id=1 row、`version=1`、sha 與 config.yaml 對得上；rollback path 測試（init 中途 fail）後 `config_state` 不應有 row

#### Scope 邊界

In scope：
- `crates/provider`：新 `ConfigStore` trait + 相關型別
- `crates/provider-local`：`LocalConfigStore` impl + v5 migration SQL + project bootstrap 插入 config_state
- `crates/runtime`：state_machine evaluator 讀 config 取 review flags
- `crates/cli`：`config show` / `config set` / `config edit` subcommand

Out of scope：
- Advisory lock 真實實作
- Role config 解析
- `${VAR}` env interpolation
- Schema 嚴格驗證（A5 只跑 syntactic YAML parse + 兩個 review flag 型別檢查）
- `instructions.get` 真實 rule 注入
- `config.changed` 對外 telemetry
- Deep diff for mode=edit
- HttpProvider `ConfigStore` impl
- `speclink audit list / show` CLI
- `speclink config delete <key>` / `config unset <key>`

## Risks / Trade-offs

- **[Risk] User 在 engine 跑到一半手動 vim config.yaml、剛好踩到 state machine evaluator 讀 config 的瞬間** → Mitigation：reconcile 走 SQLite tx + CAS、最差情況是 evaluator 讀到舊 value 跑完一次 transition、下一次 transition 再讀到新 value；不會產生 db corruption 或 race condition
- **[Risk] Multi-agent 同時 `config set` 不同 key、兩邊都讀到 v1、各自寫 v2** → Mitigation：CAS 保證最後寫入勝出、輸家收 `state.etag_mismatch`、CLI hint 提示 retry；單 RD 場景幾乎不發生、multi-agent 真實 lock 由 `add-locking-and-concurrency` slice 替換 stub
- **[Risk] Migration v5 在舊 db（v4）跑時、舊 project 沒有 config_state row** → Mitigation：v5 migration 內含 `INSERT OR IGNORE INTO config_state (id, ...) VALUES (1, <讀目前 config.yaml 算 sha>, ..., 1, now, NULL)` 一條補列邏輯；對 fresh init project 也安全（A1 init 已先插一列、v5 migration 的 INSERT 不衝突）
- **[Risk] Windows FAT32 mtime_ns 精度 2s、`mtime_ns` 對不上但 sha 一致** → Mitigation：reconcile 演算法以 sha256 為主、`mtime_ns` 只做 fast-path skip（如果 mtime 一致則信任 cache）；CAS 真實判斷靠 sha
- **[Risk] `read_defaults()` 在 walking-skeleton 階段回死值（`require_*_review=false`），未來 default 演進需要改碼** → Mitigation：把 defaults 從程式碼 const 改為 `crates/provider-local/src/migrations/v5_config_tables.sql` 旁邊的 `default_config.yaml` 檔、build script 編進 binary；下次 default 演進改 yaml 不改碼。本 slice 先放 const、`add-schema-management` slice 再搬
- **[Risk] external_edit detection 把「git 切換 branch」也誤判** → Mitigation：git checkout 後 mtime 會變、但 sha 也會變；engine 視之為 external_edit、寫一筆 audit row + warning；行為符合預期（user 切 branch 確實改了 config）；不視為 bug
- **[Trade-off] mode=edit `keys_changed = ["__edit__"]` 失去 key-level audit 顆粒度** → 接受：YAGNI、audit query CLI 還沒有；未來真需要時可 retroactively 重算 deep diff

## Migration Plan

#### 部署順序

1. `crates/provider/src/config_store.rs` 加 trait + 型別（純編譯期變更、不動 runtime）
2. `crates/provider-local/src/migrations/v5_config_tables.sql` + 註冊進 `state_db.rs` 既有 migration runner（v5）
3. `crates/provider-local/src/config_store.rs` impl `LocalConfigStore`
4. `crates/provider-local/src/store.rs` 修 init prepare/commit phase
5. `crates/runtime/src/state_machine.rs` 改讀 config（feature flag 不需要、A3 hardcode 直接替換）
6. `crates/cli/src/commands/config.rs` 加 subcommand
7. 整合測試 + analyze + validate

#### Rollback 策略

- Migration v5 為 forward-only（design §12.7）— 不提供 downgrade
- 若 A5 上線後發現嚴重 bug：
  - 個人 RD 場景：刪 `.git/speclink/state.db` + `speclink restore --from-artifacts`（A1 已落地）重建 db、可選擇 pin 在 A4 version 的 binary
  - 不提供「降到 v4 schema」自動工具；design §12.7 明列 no downgrade
- `restore --from-artifacts` 需要支援 v5 schema、A5 task 內含此補丁

#### State.db 兼容性

- 舊 db（v4）跑 v5 migration → 自動補 `config_state` row（讀當下 config.yaml 算 sha 進 INSERT）+ 建 `config_change` 表（空）；無破壞性
- 新 init project（A1 加裝 v5 migration） → 走 fresh path、config_state row 在 init prepare/commit phase 內 insert、不依賴 migration 補列邏輯
- A1 init 已有的「fresh state.db」path 要把 v5 schema 包進 `migrate(5)` 統一升版

#### `doc/speclink-design.md` 更新

- §1.1 walking skeleton 表加 A5 row（內容由 tasks.md 第 1 條負責落地）
- §11.x 對 etag 公式 / external_edit semantics / state.db v5 兩表 schema 做 cross-reference
- §18.1 對 `config.read` / `config.write` 兩條 MVP item 加「✅ Implemented (A5)」標記
- 操作面更新由 `archive.run` slice 後正式進入 archive 流程（A5 自己也會被 archive、design 表會再加 A6 row）

## Open Questions

- 無 — discuss conclusion 已收斂三個關鍵點（external_edit 偵測要做、A4 已 archive、`keys_changed` 簡化）；後續若 implementation 中發現 edge case，走 `/spectra-ingest` 把新決策補回 design.md
