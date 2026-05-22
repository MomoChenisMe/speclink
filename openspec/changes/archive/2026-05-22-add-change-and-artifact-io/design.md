## Context

`add-project-bootstrap` 完成後 `crates/provider-local` 只有 `LocalProjectStore`，state.db 只有 v1（`_migrations` + `project`），CLI 只支援 `init / status / link / unlink`。整個 SDD workflow 還沒有「change」這個一級實體，artifact 也還無法寫入。

本 change 是 MVP walking skeleton 第二片：補上 change CRUD + artifact I/O 的最小可跑切片，**不**含 state machine 6 狀態、**不**含 schema YAML 驗證、**不**含 locking、**不**含 apply / review / archive / discuss。目的是讓使用者可以手動跑：

```
speclink init
speclink new change foo
speclink new artifact proposal --change foo --stdin < draft.md
speclink artifact read proposal --change foo
speclink list --changes
speclink show change foo
```

為後續所有 slice 提供 change 容器與 artifact 寫入基礎。

## Goals / Non-Goals

**Goals:**

- 在 `state.db` schema 上長出 `change` 表，並讓 migration runner 從 v1 推進到 v2
- 在 `crates/provider` 內定義兩個新 trait（`ChangeStore`、`ArtifactStore`），讓 `cli ↔ runtime ↔ provider` 三層界面保持乾淨
- 把 sha256-based Etag 並發控制建立成可重用 pattern，讓後續 `config.read/write` 等 RMW op 直接沿用
- 提供 7 個 CLI op 的完整 JSON envelope，slice 結束時可實機跑端到端

**Non-Goals:**

- ❌ 不實作 6-state lifecycle transition；`change.state` 在本 slice 永遠是 `proposing`
- ❌ 不解析 schema YAML；`--kind` 只走白名單檢查，schema-driven structural validation 留給 `add-schema-management`
- ❌ 不引入 file-level lock；TOCTOU 視窗留給 `add-locking-and-concurrency`
- ❌ 不在 state.db 開 `artifact` 表；artifact body 完全 filesystem-backed，Etag 即時從 file bytes 算
- ❌ 不做 `artifact.write --overwrite` 盲寫逃生口
- ❌ 不寫 audit log、不寫 touched files index
- ❌ 不偵測「state.db 有 change row 但 filesystem 目錄不見」的 cross-branch artifact missing 情境，留給 doctor / restore slice

## Decisions

### Etag = `sha256(file bytes)`，artifact 不進 state.db

**選項**：

| 選項 | Etag 來源 | 需要 state.db artifact 表？ |
|---|---|---|
| A | `sha256(file_bytes)` | 否 |
| B | state.db 內 monotonic version counter | 是 |
| C | hybrid（counter + sha 雙鍵） | 是 |

**選 A**。理由：

1. **避免 state.db / filesystem 不一致 bug**：個人 RD 模式下使用者用 vim 手動編輯 `proposal.md` 是 first-class scenario。若 Etag 是 state.db counter，這個編輯不會更新 counter；下一次 AI 帶舊 Etag 寫入時誤判 OK → 覆蓋使用者手動修改 → silent data loss。sha256 路徑天然防呆，任何 byte 變動都會反映在 Etag 上。
2. **與 design §13.9 切分對齊**：§13.9 列舉 state.db 內容為「change rows / etag / feedback_tasks / audit / review history / instance_id」，這裡的 etag 是 **change row 上的 version counter**（lifecycle metadata 並發），不是 artifact body 的並發控制。Artifact body 既然完全住在 filesystem，Etag 也該從 filesystem 算。
3. **HttpProvider mapping 對應**：HTTP 端 Etag 慣例本就是 content hash / `If-Match`；slice A 採 sha256 contract，未來 HttpProvider 對應更直接。
4. **state.db schema 最小化**：v2 migration 只加一張 `change` 表，不為了 artifact 多開一張表；migration 成本最低、未來 archive flow 也少一張表要 prune。

**取捨**：每次 `artifact.read / write` 要算一次 sha256。檔案多在 KB～10KB 級，hashing throughput 在現代 CPU 上 > 1 GB/s，實際耗時 < 100µs，可忽略。

### State.db v2 migration 只加一張 `change` 表

v2 migration SQL：

```sql
CREATE TABLE change (
  change_id   TEXT PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  state       TEXT NOT NULL,
  schema_id   TEXT NOT NULL,
  version     INTEGER NOT NULL DEFAULT 1,
  created_at  TIMESTAMP NOT NULL,
  updated_at  TIMESTAMP NOT NULL
);
INSERT INTO _migrations (version, applied_at) VALUES (2, CURRENT_TIMESTAMP);
```

**替代方案**：把 `state` 改成 INTEGER + 對照表、或把 `change_id` 改成 INTEGER AUTOINCREMENT。**駁回**：(a) state 用 TEXT 與後續 6-state CLI / JSON 輸出對齊（`"proposing"` / `"reviewing"` ...），不必另做 enum↔string 映射；(b) `change_id` 用 UUID v4 對齊 §13.6 `instance_id` 風格，未來 HttpProvider 同 schema 跨機可移植，AUTOINCREMENT 跨機會撞號。

### ChangeStore 與 ArtifactStore 拆成兩個 trait

**選項**：

| 選項 | trait 數 | 取捨 |
|---|---|---|
| A | 1 個 `ProjectStore` 加方法 | 介面爆肥；slice F archive 與 slice E apply 都要碰同一 trait |
| B | 2 個 trait（`ChangeStore` + `ArtifactStore`） | 介面收斂；可被分別 mock |

**選 B**。`change.*` 操作 SQLite-only，`artifact.*` 操作 filesystem-only，責任邊界天然分明。`provider-local` 內各自實作 `LocalChangeStore` / `LocalArtifactStore`，**沿用 bootstrap「一 trait 一 struct」pattern**（bootstrap `LocalProjectStore` 即此 pattern；不引入 aggregate 型別）。

bootstrap slice 既有的 `ProjectStore` trait 與 `LocalProjectStore` 不動；本 change 新增的兩個 trait 與兩個 struct 並列存在，CLI / runtime 依需要 build 對應 store 實例。

### Versioned<T> + expected_etag 共用型別

在 `crates/provider/src/types.rs` 新增：

```rust
pub struct Etag(pub String);   // "sha256:<hex>"  或 change row counter 序列化字串

pub struct Versioned<T> {
    pub value: T,
    pub etag: Etag,
}

pub enum ExpectedEtag {
    None,                       // 「新建專用」語意
    Some(Etag),                 // 「覆寫需匹配」語意
}
```

`ArtifactStore::write_artifact` 簽名（錯誤型別沿用 bootstrap 既有的單一 `ProviderError` enum，不另開 `ArtifactError`）：

```rust
async fn write_artifact(
    &self,
    change: &str,
    kind: ArtifactKind,
    capability: Option<&str>,
    bytes: &[u8],
    expected: ExpectedEtag,
) -> Result<Versioned<()>, ProviderError>;
```

**替代方案**：`expected_etag: Option<Etag>`，用 `None` 兼表「新建」與「不關心」。**駁回**：兩種語意混用會讓盲寫逃生口悄悄回來；用 `ExpectedEtag` enum 把語意鎖死「不傳 = 新建 only」「傳了 = 覆寫 only」。

### Atomic write 採 tempfile-in-same-dir + rename

**選項**：

| 選項 | 路徑 |
|---|---|
| A | tempfile 與目標檔同 parent dir，`std::fs::rename` 替換 | atomic on POSIX; atomic on Windows via `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` |
| B | tempfile 在 OS temp dir，跨 device `copy + remove` | 非 atomic，可能跨 filesystem 失敗 |
| C | flock + 直接 write | 不 atomic on crash |

**選 A**。`std::fs::rename` 在同 parent dir 上跨平台 atomic（POSIX `rename(2)` 與 Windows `MoveFileExW` 加 `MOVEFILE_REPLACE_EXISTING` flag）。Rust stdlib 在 Windows 上預設就帶該 flag。`tempfile` crate 在 `with_prefix_in(parent_dir)` 用法下保證同 parent。

**Crash policy**：tempfile 採 RAII guard，drop 時 best-effort `unlink`；若程式 crash，下次啟動時可能殘留 `.speclink/changes/foo/.proposal.md.<random>` 之類檔案，**屬於可接受 residue**，由 doctor / housekeeping slice 清理；本 slice 不主動掃。

### `spec.list-in-change` 走 filesystem，不查 state.db

**選項**：

| 選項 | 來源 |
|---|---|
| A | `read_dir(.speclink/changes/<name>/specs/)`，過濾含 `spec.md` 的子目錄 |
| B | 在 state.db 加 `spec` 表追蹤 capability |

**選 A**。slice A 不在 state.db 加 artifact 表（見上一個 decision）；filesystem 是 source of truth；`read_dir` 在 capability 數量 < 100 的 MVP scale 下耗時 < 1ms。**替代 B 駁回**：與 §13.9 切分衝突、徒增一張需要 archive / drift / restore 維護的表。

CLI surface 用 `speclink list --specs --change <name>`（沿用 §16.4 動詞前置慣例）。

### Change name 與 capability id 共用 grammar

兩者共用正規表示式 `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` 與 1–64 byte 長度上下限。validation helper 抽在 `crates/provider/src/types.rs::validate_kebab_id`，change-store 與 artifact-io 都呼叫同一份。

**替代方案**：允許 capability id 含 `_`、`.`、`/` 等。**駁回**：spec capability id 會直接出現在 filesystem 路徑 `specs/<capability>/spec.md`，與 change 名也共用 path-segment 範疇；用同一份 grammar 跨平台路徑 安全（避開 Windows reserved name、避開 path traversal）。

### `delete change` 用 `--confirm-name <name>` 而非 `--force`

§16.6 已標明「`--force` 留給 destructive ops、`--overwrite` 留給 rewrite」。change 刪除屬於 destructive；但用 `--force` 等於把責任丟給使用者一個 boolean，AI 也可能誤帶。改要求 `--confirm-name <name>` 與目標 change 名 exact match，AI skill 永遠不主動帶（§19.5 「AI auto --force」威脅對應）。

### 沿用 bootstrap 既有 error pattern — 不引入新 error enum

**選項**：

| 選項 | error 結構 |
|---|---|
| A | 在既有 `ProviderError` 單一 enum 上加 7 個 variant；在既有 `provider::codes` module 加 7 個 `pub const` |
| B | 新增 `ChangeError` / `ArtifactError` 兩個獨立 enum，各自帶 `code()` 方法 |

**選 A**。bootstrap 已建立「one error enum per crate」pattern：`provider/src/error.rs::ProviderError` 涵蓋 4 個 declared variant + `Internal`，`RuntimeError` 透過 `#[from] ProviderError` 統一往上吸收。本 slice 沿用此 pattern：

- `provider::error::ProviderError` 加 7 個 variant：`ChangeNotFound { name }`、`ChangeDuplicateName { name }`、`ChangeInvalidName { name, reason }`、`ArtifactKindInvalid { kind }`、`ArtifactCapabilityRequired`、`ArtifactNotFound { path }`、`ArtifactVersionConflict { expected: Option<Etag>, actual: Etag }`
- `provider::codes` module 加 7 個 const，命名同 spec error code（`CHANGE_NOT_FOUND`、`CHANGE_DUPLICATE_NAME`、…）
- `ProviderError::code()` match arm 擴 7 條對應
- `ProviderError::retryable()` 對 `ArtifactVersionConflict` 回 `true`，其餘維持 `false`

`RuntimeError` 同步加 7 個 variant，並擴 `code()` 與 `exit_code()` 對照 match arm。CLI `output::error_code_to_exit` 同步擴 match arm（兩處對照表並存是 bootstrap 既有結構，不在本 slice 內整併）。

**駁回 B**：兩個獨立 enum 違背 bootstrap 已建立的單一 enum pattern；增加 CLI / runtime 上層 `#[from]` 串接點；mock 測試與既有 `LocalProjectStore` 無法共用 error fixture。trait 拆兩個（`ChangeStore` / `ArtifactStore`）是合理的，但 error 不該跟著拆。

### LocalProvider 開檔路徑統一 `migrate(2)`

**問題**：bootstrap `crates/provider-local/src/store.rs::LocalProjectStore::open_state_db()` 內 hardcode `db.migrate(1)`。若不調整，本 slice 寫入 `change` 表的 SQL 會 fail，因為 `change` 表只在 `MIGRATIONS[1]` (v2) 才存在。

**決定**：本 slice 把該行 bump 為 `db.migrate(2)`。**全 LocalProvider 開檔路徑**統一升 v2，包含既有 `init` / `status` / `link` / `unlink` 四條 CLI flow。對使用者 invisible：v1 → v2 migration 為 forward-only、不改既有 `project` 表內容、idempotent。

**替代方案**：為 change/artifact ops 新增獨立 `open_state_db_v2()` 函式，避免動既有 `LocalProjectStore`。**駁回**：兩條開檔路徑 = 兩條 migration 狀態，違反 §12.7 forward-only 規範；既有 `LocalProjectStore` 與新 stores 共用同一個 `state.db` 檔，schema 必須單調對齊。

**Risk**：bootstrap slice 已 release 的 binary 開的舊 db（v1）在 slice A binary 第一次跑時會升 v2，**無法 downgrade**。若 user 之後切回 bootstrap binary，`schema_version() == 2 > MIGRATIONS.len() == 1`，會觸發 `StateDbError::SchemaVersion`。本 slice 不解這個跨版本 downgrade 問題（§18.1 #67 已標 forward-only / no downgrade）；release notes 必須警告。

## Implementation Contract

### Behavior

操作者在 bootstrap-init 過的 speclink 專案內，可以：

1. 用 `speclink new change <name>` 在 `.speclink/changes/<name>/` 建立空目錄、`change` 表新增一列
2. 用 `speclink new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <etag>] --stdin` 寫入 proposal / design / tasks / spec 四種 markdown；新建不帶 etag，覆寫必須帶現檔 sha256
3. 用 `speclink artifact read <kind> --change <name> [--capability <cap>]` 讀回 content + sha256 Etag
4. 用 `speclink list --changes` 列出所有 change（依 updated_at desc）
5. 用 `speclink show change <name>` 看 change metadata + 該 change 下既有 artifact 清單
6. 用 `speclink list --specs --change <name>` 列出該 change 下所有 spec 的 capability id
7. 用 `speclink delete change <name> --confirm-name <name>` 把 change row 與 `.speclink/changes/<name>/` 一起拿掉

### Interface / data shape

#### CLI subcommand 對 Catalogue ID 映射（§21.5）

| Catalogue ID | CLI 形式 | AI skill 可呼叫 |
|---|---|---|
| `change.create` | `speclink new change <name>` | ✓ |
| `change.list` | `speclink list --changes` | ✓ |
| `change.show` | `speclink show change <name>` | ✓ |
| `change.delete` | `speclink delete change <name> --confirm-name <name>` | ✓（destructive，skill 不主動帶）|
| `artifact.write` | `speclink new artifact <kind> --change <name> [--capability <cap>] [--expected-etag <etag>] --stdin` | ✓ |
| `artifact.read` | `speclink artifact read <kind> --change <name> [--capability <cap>]` | ✓ |
| `spec.list-in-change` | `speclink list --specs --change <name>` | ✓ |

#### Provider trait（新增至 `crates/provider/src/lib.rs`，沿用 bootstrap `ProjectStore` 既有 `async-trait` 風格 + 單一 `ProviderError` 錯誤型別）

```rust
#[async_trait::async_trait]
pub trait ChangeStore: Send + Sync {
    async fn create_change(&self, name: &str, schema_id: &str) -> Result<ChangeRow, ProviderError>;
    async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError>;
    async fn get_change(&self, name: &str) -> Result<ChangeRow, ProviderError>;
    async fn delete_change(&self, name: &str) -> Result<(), ProviderError>;
}

#[async_trait::async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn read_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<Versioned<Vec<u8>>, ProviderError>;

    async fn write_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
        bytes: &[u8],
        expected: ExpectedEtag,
    ) -> Result<Versioned<()>, ProviderError>;

    async fn list_spec_capabilities(&self, change: &str) -> Result<Vec<String>, ProviderError>;
}
```

#### Runtime entry struct（新增至 `crates/runtime/src/change_ops.rs` + `crates/runtime/src/artifact_ops.rs`）

沿用 bootstrap `ops.rs::Operations<G: GitProbe>` 的「struct + GitProbe 泛型 + build_store helper」pattern。每個 op runtime 模組對應一個 struct：

```rust
pub struct ChangeOperations<G: GitProbe> { git: G }
pub struct ArtifactOperations<G: GitProbe> { git: G }
```

兩個 struct 各自實作 `pub async fn` 方法（`create_change` / `list_changes` / `show_change` / `delete_change` ／`read_artifact` / `write_artifact` / `list_spec_capabilities`）對應 CLI subcommand entry；內部 build `LocalChangeStore` / `LocalArtifactStore` 並委派。**不**擴 bootstrap 既有 `Operations<G>` struct（避免單一入口長期肥大化）。

#### JSON envelope shape

success 與 error envelope 沿用 bootstrap slice 既有 `output::Envelope`；新增 `data` 形狀詳見 `specs/change-store/spec.md` 與 `specs/artifact-io/spec.md` 內 example 區塊。Etag 字串格式統一為 `sha256:<hex_lowercase>`，hex 部分為 64 個 lowercase hex char。

### Failure modes

| Error code | Exit | 觸發場景 | retryable |
|---|---|---|---|
| `change.not_found` | 2 | `show change` / `delete change` / 任何 artifact op 對不存在的 change | false |
| `change.duplicate_name` | 7 | `new change` 名稱已存在 | false |
| `change.invalid_name` | 2 | `new change` 名稱不符 grammar / 長度；或 `delete change` 沒帶 / 帶錯 `--confirm-name` | false |
| `artifact.kind_invalid` | 2 | `--kind` 不在白名單；或 `--capability` 不符 grammar | false |
| `artifact.capability_required` | 2 | `--kind spec` 缺 `--capability` | false |
| `artifact.not_found` | 2 | `artifact.read` 對應檔案不存在；或 `artifact.write` 新建路徑卻帶了 non-null `--expected-etag` | false |
| `artifact.version_conflict` | 7 | `artifact.write` 覆寫 etag 不符；或覆寫缺 `--expected-etag` | true |

mid-write atomic-rename 失敗（disk full / permission）對外回 `runtime.io`（沿用 bootstrap 既有），不另開新 code。

### Acceptance criteria

實作完成 SHALL 透過以下方式驗證：

| # | 驗證項目 | 驗證手段 |
|---|---|---|
| 1 | state.db 從 v1 升 v2 後 `change` 表 schema 符合 spec table | `pragma_table_info('change')` snapshot test |
| 2 | `speclink new change foo` 後 `.speclink/changes/foo/` 存在且為空 | `assert_cmd` + `tempfile` 整合測 |
| 3 | 重複 `new change foo` 第二次回 `change.duplicate_name`、exit 7 | 同上 |
| 4 | name `Foo` / `foo_bar` / 65-byte 都回 `change.invalid_name` | parametrized unit test |
| 5 | `new artifact proposal --change foo --stdin` 新建後可 `artifact read` 回相同 bytes + 對應 sha256 | round-trip 整合測 |
| 6 | 覆寫帶正確 `--expected-etag` 成功，帶錯誤值回 `artifact.version_conflict` exit 7 | concurrency 整合測 |
| 7 | 覆寫不帶 `--expected-etag` 直接回 `artifact.version_conflict` | 同上 |
| 8 | `--kind spec` 缺 `--capability` 回 `artifact.capability_required` | 整合測 |
| 9 | tempfile 寫到一半 crash（注入 fault）→ 目標檔不變、tempfile 不殘留 | RAII guard 單元測 |
| 10 | `list --specs --change foo` 對含與不含 `spec.md` 的子目錄都正確 | filesystem 整合測 |
| 11 | `delete change foo --confirm-name foo` 後 row 與目錄都消失；不帶或帶錯 `--confirm-name` 不動 | 整合測 |
| 12 | `--json` envelope 對 7 個 op 各取一個 snapshot | `insta` snapshot |
| 13 | Etag 對同 bytes 跨平台一致（Windows / macOS / Linux） | CI 矩陣跑 |

### Scope boundaries

**In scope**:
- `crates/provider`：新增 `ChangeStore` + `ArtifactStore` trait（trait-level error 型別沿用既有 `ProviderError`）、`Versioned<T>` + `Etag` + `ExpectedEtag` + `ArtifactKind` + `ChangeRow` 共用型別、`validate_kebab_id` helper；`ProviderError` 加 7 個 variant；`provider::codes` module 加 7 個 `pub const` error code
- `crates/provider-local`：state.db `MIGRATIONS` 陣列加 v2 entry；`LocalProjectStore::open_state_db()` 內 `migrate(1)` bump 為 `migrate(2)`；`LocalChangeStore` + `LocalArtifactStore` 各自獨立 struct（沿用 bootstrap「一 trait 一 struct」pattern，不引入 aggregate）；atomic rename helper
- `crates/runtime`：新增 `change_ops.rs` (`ChangeOperations<G>`) + `artifact_ops.rs` (`ArtifactOperations<G>`)，沿用 bootstrap `ops.rs::Operations<G>` 命名慣例；`RuntimeError` 加 7 個 variant；`code()` 與 `exit_code()` match arm 擴充
- `crates/cli`：7 個 subcommand module（`new_change` / `new_artifact` / `list_changes` / `list_specs` / `show_change` / `delete_change` / `artifact_read`）；`commands/mod.rs` pub mod 列表更新；`main.rs::Commands` enum 加 variant；`output::error_code_to_exit` match arm 擴充；`hint_for` 對 7 個新 code 加 hint
- 測試位置沿用 bootstrap 慣例：`crates/cli/tests/` + `crates/runtime/tests/` + `crates/provider-local/tests/`；不放在 workspace root 的 `tests/`
- 至少涵蓋 acceptance criteria 表中 13 項

**Out of scope**:
- 任何 `change.state` 改變的 CLI op（slice E）
- schema YAML 讀取或 artifact 結構驗證（slice D）
- 任何 lock 檔操作（slice B）
- audit log / touched files / review history（後續 slice）
- HttpProvider impl（永遠延後）
- `artifact.write --overwrite` 盲寫 flag
- 把 `output::error_code_to_exit` 與 `RuntimeError::exit_code()` 兩處對照表整併（bootstrap 既有 duplication，本 slice 不重構，只擴 match arm）

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| TOCTOU 視窗：兩個 process 同時讀取 → 各自 sha 算出 → 各自比對 → 各自寫入；後寫者覆蓋前寫者 | slice B locking 會用 per-change file lock 收掉；MVP 個人 RD 場景 race 機率極低，可接受 |
| atomic rename 跨 filesystem 失敗（極少數情況：`.speclink/` 在 mount point 上）| tempfile 強制建在 `.speclink/changes/<name>/` 內、與目標檔同 parent；不跨 mount |
| sha256 hash 對大檔的 latency（> 1 MB 的 spec） | 不顯著（GB/s 級）；若未來真有 > 100 MB 的 artifact，再評估 mmap + 切片 hash |
| Windows file lock 干擾 rename（防毒軟體鎖檔） | 沿用 bootstrap slice 既有 `runtime.io` retry pattern；本 slice 不額外處理 |
| AI skill 可能誤帶 `--confirm-name <錯誤名>` 嘗試刪除 | exact match 規則 + skill draft 永不主動帶（§19.5 對應）|
| state.db schema_id 欄位寫死 `'spec-driven'`（slice A 沒 schema CLI） | 暫接受；slice D 加 schema CLI 時 backfill / accept legacy 'spec-driven' 為合法值 |
| `change` row 的 `version` 欄位 slice A 永遠是 1（未真正運作） | 不是 bug — 欄位存在是為了 slice E state transition 用，slice A 沒有 RMW 機會；spec 已明示 |
| migration v2 與並發 init / migration race | bootstrap slice 既有 migration runner 已採 `BEGIN IMMEDIATE`；本 slice 沿用，並在整合測加一支 race test |
| bootstrap binary downgrade：使用者跑過 slice A binary 後 state.db 已 v2，回頭跑 bootstrap binary 會在 `StateDb::migrate(1)` 撞 `target_version < current` 或 `MIGRATIONS.len() < found` | 設計接受此限制（§18.1 #67 forward-only / no downgrade）；release notes 必須說明「升級後不可回退」；slice B 之後增加 `doctor.state.db_schema_invalid` finding 引導 user 升回新 binary |
| `LocalProjectStore::open_state_db()` 從 `migrate(1)` bump 到 `migrate(2)` 影響 bootstrap slice 既有所有 flow（init / status / link / unlink）| migration 為 forward-only、idempotent、不改既有 `project` 表資料；新增 `crates/runtime/tests/bootstrap.rs` 內既有測試在 v2 下重跑必須仍綠 |
