## Context

`add-artifact-write-and-status` 已將 SpecLink 在 local provider 上的「寫 + 讀」能力補齊；本 change 補上「收尾」— archive 流程是 SDD 週期的終點，必須處理三件事：(1) 把 change 目錄從 active 區搬到 archive 區、(2) 清理 SQLite `in_progress_change` 表、(3) 把 change 內的 delta spec 套用到主 spec 目錄。

設計約束（自 bootstrap 與 Change 2 繼承）：
1. `Provider` trait 保持 `Send + Sync + dyn-compatible`，所有 method async
2. lib crate 用 `thiserror`、bin crate 用 `anyhow`
3. JSON envelope schema、exit code 表、error code 命名規則不變
4. Atomic write 策略沿用（temp + rename + cleanup）
5. metadata.json 是 source of truth、SQLite 是 fast index

主 spec 目錄選址：`.speclink/specs/<capability>/spec.md`，與 `.speclink/changes/` 同層 — 統一收斂在 `.speclink/` 底下，與 design doc 既有的 local provider directory layout 思路一致。專案根目錄不出現 `specs/` 是刻意決定（見下方 Decisions）。

## Goals / Non-Goals

**Goals:**

- 新增 `Provider::archive_change` trait method，回傳 archive 結果與 delta 套用摘要
- 新增 CLI 指令 `speclink archive <change>` 與 `--dry-run` 旗標
- 釘住主 spec 目錄位置：`.speclink/specs/<capability>/spec.md`
- 釘住 archive 目錄結構：`.speclink/changes/archive/YYYY-MM-DD-<id>/`，內部保持原 change 目錄結構
- 釘住 spec delta merge 演算法：ADDED / MODIFIED / REMOVED / RENAMED 四種 heading 的解析與套用規則
- 釘住 archive 流程的失敗復原策略（rollback）
- 釘住 `State::Archived` 與 `metadata.json` 的 `archivedAt` 欄位

**Non-Goals:**

- 不引入 `in_progress` 狀態（已於 proposal Non-Goals 詳述）
- 不引入 archive trace 註記（在 main spec 加 `<!-- @trace ... -->`）— 留給 Change 4 或更晚
- 不引入 archive 反向操作（restore）
- 不引入 cross-spec 一致性檢查
- 不引入 delta schema validator（解析失敗即報錯）
- 不變更既有 propose create / artifact write / status 三條指令的行為
- 不為主 spec 目錄添加任何 metadata 檔（沒有 `.spec_index.json` 或類似結構）

## Decisions

### 主 spec 目錄落在 `.speclink/specs/<capability>/spec.md` 而非專案根目錄

主 spec 統一放 `.speclink/specs/<capability>/spec.md`，與 `.speclink/changes/` 同層。

**理由**：
- 與 design doc 既有 layout 一致（local provider 所有狀態收斂在 `.speclink/`）
- 專案根目錄不放 SpecLink 內部檔案，使用者 repo 結構不被污染
- 未來 HTTP provider 把 main spec 存遠端，本機只有 `.speclink/` 內的 cache copy，不會牽動 root layout

**替代方案**：
- **專案根 `specs/<capability>/spec.md`（OpenSpec 風格）**：拒絕原因，使用者 repo 已有自己的 `specs/` 或其他 doc 目錄，SpecLink 把檔案丟根層會與既有結構衝突。
- **`.speclink/main-specs/` 強調是 archived merge 結果**：拒絕原因，多此一舉的命名；`specs/` 已是 SDD 慣例。
- **可由 `.speclink/config.toml` 設定**：拒絕原因，本 change 不需配置彈性；硬編路徑簡化第一版實作，未來如出現多個團隊需求再評估。

### `Provider::archive_change` 簽章與回傳型別

`crates/provider/src/lib.rs` 新增：

```rust
async fn archive_change(
    &self,
    project_id: &ProjectId,
    change_id: &ChangeId,
    options: ArchiveOptions,
) -> Result<ArchivedChange, ProviderError>;
```

```rust
pub struct ArchiveOptions {
    pub dry_run: bool,
    pub archive_date: chrono::NaiveDate, // 由 caller 注入（測試可固定）
}

pub struct ArchivedChange {
    pub change_id: ChangeId,
    pub archive_path: String,            // POSIX 路徑，相對於 base
    pub state: State,                    // 應為 Archived
    pub archived_at: String,             // ISO 8601 UTC
    pub spec_sync: SpecDeltaSummary,     // 套用摘要
    pub dry_run: bool,
}

pub struct SpecDeltaSummary {
    pub capabilities_synced: Vec<CapabilitySyncResult>,
}

pub struct CapabilitySyncResult {
    pub capability: String,
    pub main_spec_path: String,          // POSIX 路徑
    pub added_count: usize,
    pub modified_count: usize,
    pub removed_count: usize,
    pub renamed_count: usize,
    pub created_main_spec: bool,         // 主 spec 是否為本次新建
}
```

**理由**：summary 數字讓 AI skill 與工程師快速驗證套用範圍；`created_main_spec` 區分「全新 capability 首次寫主 spec」與「既有 capability 套 delta」。

**替代方案**：
- **將 spec sync 拆獨立 trait method `apply_spec_deltas`**：拒絕原因，archive 與 spec sync 是同一個原子操作，拆兩個 method 會出現「目錄搬完但 spec 沒同步」的中間狀態，難以 rollback。
- **回傳 `Vec<RequirementChange>` 列出每個 requirement 異動明細**：拒絕原因，AI skill 真正需要的是 summary；明細在 archive 後可直接讀 main spec 看 diff。

### `ArchiveOptions::archive_date` 由 caller 注入

archive 目錄前綴 `YYYY-MM-DD` 由 `archive_change` 接收的 `archive_date` 決定，**不由 provider 內部呼叫 `chrono::Utc::now()`**。

**理由**：測試可固定日期、CI 可重現；CLI 入口呼叫 `chrono::Local::now().date_naive()` 傳入。

**替代方案**：
- **provider 內部自取 today**：拒絕原因，測試需 mock 時間，傳入比 trait 拆分時鐘抽象簡單。

### Spec delta merge 演算法位置：`crates/runtime/src/spec_delta.rs`

merge 純算法（input 為 `&str` of delta + `&str` of existing main spec，output 為 `String` of new main spec），不觸碰 filesystem，放 runtime crate。`provider-local` 在 `archive_change` 中讀檔、呼叫 runtime 函式、寫檔。

`crates/provider-local/Cargo.toml` 新增依賴 `runtime`。

**理由**：
- spec_delta 是純資料處理邏輯，沒有 I/O，放 runtime 可被其他 provider 復用（HTTP provider 未來 archive 時也能用同一份算法）
- 反向（spec_delta 放 provider crate）會讓 provider crate 變胖；runtime 已負責 orchestration，spec 算法歸 runtime 更自然

**替代方案**：
- **spec_delta 放 provider crate**：拒絕原因，provider crate 應只含 trait + 型別 + resolution；算法不屬於這個 layer。
- **spec_delta 放 provider-local 自己**：拒絕原因，HTTP provider 未來要重寫一份。
- **新建 `crates/spec-delta/`**：拒絕原因，演算法量不大（< 300 LoC 預估），YAGNI。

呼叫 `rust-skills:m05-type-driven` 確認單一檔案承擔解析 + 套用的 trade-off vs 拆成 parser + applier 兩個 module。

### Spec delta heading 解析規則

支援 4 種 top-level `## ` heading：

```
## ADDED Requirements
## MODIFIED Requirements
## REMOVED Requirements
## RENAMED Requirements
```

每個 heading 下含若干 `### Requirement: <name>` 區塊，每區塊延伸到下一個 `### Requirement:` 或下一個 `## ` 為止。

對 main spec 套用規則：

- **ADDED**：每個 `### Requirement: X` 區塊 append 到 main spec 末尾（順序與 delta 一致）；若 X 在 main spec 已存在則回 `SpecDeltaConflict` 錯誤
- **MODIFIED**：根據 `### Requirement: X` 名稱在 main spec 找對應區塊，整段替換為 delta 中的新內容；找不到對應 X 回 `SpecDeltaConflict` 錯誤
- **REMOVED**：根據 `### Requirement: X` 名稱在 main spec 找對應區塊並刪除；找不到 X 回 `SpecDeltaConflict` 錯誤。`### Requirement: X` 後可選用 `**Reason**:` 與 `**Migration**:` 兩個 metadata 欄位，本 change 視為純文件不解析
- **RENAMED**：每個 `### Requirement: X` 區塊內含 `**FROM:** old-name` 與 `**TO:** new-name` 兩行；在 main spec 找 `### Requirement: old-name` 並改為 `### Requirement: new-name`；本 change 不支援同時改名與改內容（用兩個 delta：先 RENAMED 再 MODIFIED）

**理由**：與既有 Spectra / OpenSpec 慣例一致；4 種 heading 涵蓋 90% 場景。

**替代方案**：
- **支援 `## REPLACED Requirements`（完全替換 main spec）**：拒絕原因，已可用全 MODIFIED 覆蓋；REPLACED 引入額外語意。
- **只支援 ADDED + MODIFIED**：拒絕原因，REMOVED 是 spec 演化的合理操作；RENAMED 是常見 refactor 必需。

### Requirement 區塊邊界以下一個 `### Requirement:` 或下一個 `## ` 為界

parser 看到 `### Requirement: X` 後，把直到下一個同層或上層 heading 之前的所有內容視為該 requirement 的完整內容（含 `#### Scenario:`、`##### Example:` 等子 heading）。

content hash：本 change 不對 requirement 區塊算 hash 比對；MODIFIED 完全信任 delta 內容覆蓋 main spec。

**替代方案**：
- **解析每個 `#### Scenario:` 並逐 scenario 套用**：拒絕原因，太細；spec 作者用 `### Requirement:` 為單位思考。

### Archive 流程的原子性與 rollback 策略

archive 操作為 multi-step（目錄 rename + SQLite UPDATE + metadata 寫入 + spec delta 套用），無法用單一 filesystem primitive 達到 atomic。採用 **best-effort + explicit rollback**：

執行順序：

1. 讀 `<change>/specs/<capability>/spec.md` 與 `.speclink/specs/<capability>/spec.md`（既有主 spec，可能不存在）
2. 計算 spec delta merge 結果（純運算，不寫檔）；任一 capability 失敗（conflict / parse error）→ 整個 archive 中止，無 side effect
3. 若 `dry_run = true`，回傳 summary，跳過下方步驟
4. 寫入新 main spec 到 `.speclink/specs/<capability>/spec.md.tmp`（每個 capability 一個 tmp）
5. 更新 `<change>/metadata.json.tmp`（含新 `state = "archived"` 與 `archivedAt`）
6. rename `<change>/metadata.json.tmp` → `<change>/metadata.json`
7. rename 每個 `.tmp` 主 spec → 正式檔
8. rename `<change>/` → `<archive_dir>/YYYY-MM-DD-<id>/`（用 `std::fs::rename`，跨同一 filesystem 為 atomic）
9. SQLite：`DELETE FROM in_progress_change WHERE change_id = ?`

失敗點處理：

- 步驟 1-2 失敗：無 side effect，直接回錯誤
- 步驟 4-5 失敗：刪除所有 `.tmp` 檔，回錯誤；既有 `<change>/` 與 main spec 不變
- 步驟 6 失敗：刪除所有 `.tmp` 檔，回錯誤；既有 metadata.json 仍是 proposed
- 步驟 7 部分成功（rename 主 spec 部分成功部分失敗）：嘗試 rollback 已 rename 的主 spec（從備份還原）；若 rollback 也失敗，回 `Internal` 並警告手動修復。**備份**：在步驟 4 前同時為每個既有主 spec 建立 `.bak` 副本，rollback 時用 `.bak` 還原；archive 全部成功後刪 `.bak`
- 步驟 8 失敗：rollback metadata.json + 主 spec（同上 .bak 機制），回 `Internal`
- 步驟 9 失敗：archive 已搬完，metadata 已 archived，僅 SQLite 殘留 — 視為 idempotent 可重新執行（archive 對已不在 active 區的 change id 應 best-effort 處理 SQLite，找不到 row 不視為錯誤）

**替代方案**：
- **以 SQLite transaction 包整個流程**：拒絕原因，filesystem 操作不在 SQLite transaction 範圍內，假象的 atomic 反而誤導。
- **不做 rollback，失敗時保留半成品由人修復**：拒絕原因，AI skill 遇半成品難以 self-recover，CLI 應提供至少 best-effort cleanup。

### `--dry-run` 旗標的行為

`--dry-run` 在步驟 1-2 完成後即回傳，輸出 envelope 與正常一致但 `data.dryRun = true`、`data.archivePath` 帶上「將會移到」的預期路徑（但實際目錄不變）、SpecSyncSummary 反映「將會套用」的 add/modify/remove/rename 數量。

**理由**：AI skill 在真正 archive 前可預覽影響範圍；工程師 review 用。

**替代方案**：
- **`--check` 命名而非 `--dry-run`**：拒絕原因，dry-run 是業界慣例（terraform、ansible）。

### CLI `speclink archive <change>` 子命令採 positional argument

```bash
speclink archive add-feature [--dry-run] [--json] [--no-color] [--quiet]
```

`<change>` 為 positional，與 propose / artifact write / status 的 `--change <id>` 不同 — archive 場景下 change id 是主角，positional 更符合 Unix 慣例（`git archive <commit>`、`spectra archive <name>`）。

**替代方案**：
- **`--change <id>`**：拒絕原因，少了與既有 `spectra archive` 的記憶熟悉度。
- **同時支援兩種**：拒絕原因，clap 允許但增加 surface 複雜度，YAGNI。

呼叫 `rust-skills:domain-cli` 確認 positional vs flag 的 idiom。

### Error code 新增清單

| 新 error code | 觸發條件 | exit code |
|---|---|---|
| `archive.change_not_archivable` | change 處於不可 archive 狀態（如已是 archived） | 1 |
| `spec.delta_conflict` | ADDED 已存在 / MODIFIED 找不到 / REMOVED 找不到 / RENAMED FROM 找不到 | 7 |
| `spec.delta_parse_error` | delta heading 格式錯誤 | 2 |

`spec.delta_conflict` 用 exit code 7（conflict）符合既有 exit code 表規格。

**替代方案**：
- **conflict 與 parse error 都用 exit code 1**：拒絕原因，conflict 是「使用者需介入」的特定錯誤，AI skill 處理策略不同；exit code 7 已預留。

## Implementation Contract

**Observable behavior**：

執行 `speclink archive add-feature --json`（假設 `add-feature` 為 active change、含 1 capability spec `user-auth`）：

1. 讀 `.speclink/changes/add-feature/specs/user-auth/spec.md`（delta）
2. 讀 `.speclink/specs/user-auth/spec.md`（若不存在則視為「新建主 spec」）
3. 計算 merge 結果並寫入 `.speclink/specs/user-auth/spec.md`
4. 更新 `.speclink/changes/add-feature/metadata.json` 的 state 為 `"archived"`、加 `archivedAt`
5. 把 `.speclink/changes/add-feature/` 移動到 `.speclink/changes/archive/2026-05-19-add-feature/`
6. SQLite `DELETE FROM in_progress_change WHERE change_id = "add-feature"`
7. stdout 印 JSON：

```json
{
  "ok": true,
  "data": {
    "changeId": "add-feature",
    "archivePath": ".speclink/changes/archive/2026-05-19-add-feature",
    "state": "archived",
    "archivedAt": "2026-05-19T12:34:56Z",
    "dryRun": false,
    "specSync": {
      "capabilitiesSynced": [
        {
          "capability": "user-auth",
          "mainSpecPath": ".speclink/specs/user-auth/spec.md",
          "addedCount": 2,
          "modifiedCount": 0,
          "removedCount": 0,
          "renamedCount": 0,
          "createdMainSpec": true
        }
      ]
    }
  },
  "warnings": [],
  "error": null,
  "requestId": "req_..."
}
```

8. exit code = 0

**Interface（命名，不靠行號）**：

- clap 結構：`Cli::Archive(ArchiveArgs)`、`ArchiveArgs { change: String, dry_run: bool, flags: MachineInterfaceFlags }`
- runtime 入口：`crates/runtime/src/archive.rs::archive(provider: Arc<dyn Provider>, input: ArchiveInput) -> Result<ArchivedChange, RuntimeError>`
- spec delta 核心：
  - `crates/runtime/src/spec_delta.rs::parse_delta(content: &str) -> Result<ParsedDelta, SpecDeltaError>`
  - `crates/runtime/src/spec_delta.rs::apply_delta(main: Option<&str>, delta: &ParsedDelta) -> Result<(String, ApplySummary), SpecDeltaError>`
- provider trait method：`Provider::archive_change(project_id, change_id, options) -> Result<ArchivedChange, ProviderError>`
- output 型別：`crates/cli/src/output.rs::ArchiveData`、`SpecSyncSummaryJson`、`CapabilitySyncResultJson`

**Failure modes**：

| 觸發條件 | error code | exit code |
|---|---|---|
| change 不存在 | `change.not_found` | 1 |
| change 已是 archived（metadata.json state == "archived"） | `archive.change_not_archivable` | 1 |
| 同名 archive 目錄已存在（同日重複 archive） | `archive.change_not_archivable` | 1 |
| spec delta heading 格式錯誤 | `spec.delta_parse_error` | 2 |
| ADDED 的 requirement 已存在於主 spec | `spec.delta_conflict` | 7 |
| MODIFIED / REMOVED / RENAMED 找不到對應 requirement | `spec.delta_conflict` | 7 |
| filesystem 失敗（無寫權限、磁碟滿） | `internal.error` | 1 |
| SQLite 失敗 | `internal.error` | 1 |
| `--dry-run` 模式下 spec delta 衝突 | `spec.delta_conflict` | 7 |

stderr：與既有規則一致。

**Acceptance criteria**：

實作後以下測試通過：

1. `cargo build --workspace` 三平台無 warning
2. `cargo fmt --check` 通過
3. `cargo clippy --workspace -- -D warnings` 通過
4. `cargo test --workspace` 通過，含：
   - `crates/runtime/src/spec_delta.rs` 單元測試：`parse_delta` 涵蓋 4 種 heading + 邊界（heading 之間有空白、嵌套 `#### Scenario:`）；`apply_delta` 涵蓋 main 為 `None`（新建）、ADDED happy / 衝突、MODIFIED happy / 找不到、REMOVED happy / 找不到、RENAMED happy / FROM 找不到
   - `crates/provider/src/model.rs` `State` 序列化新測試：`State::Archived` ↔ `"archived"`
   - `crates/provider-local/tests/archive_integration.rs` end-to-end：propose create → write artifacts → archive → 驗證主 spec 內容、archive 目錄存在、原 active 目錄不存在、SQLite 空、metadata.json state = archived
   - `crates/provider-local/tests/archive_integration.rs` rollback：模擬步驟 7 失敗（mock filesystem 不可寫）後驗證主 spec `.bak` 還原成功、active 目錄仍在
   - `crates/cli/tests/archive.rs` assert_cmd 整合：happy path、change not found、already archived、delta conflict（ADDED 已存在）、dry-run（驗證無 filesystem side effect）
   - insta snapshot：archive success、dry-run success、delta_conflict failure
5. 手動驗證：跑 `propose create demo --summary x` → `artifact write spec --capability auth` → `archive demo --json` → 確認 `.speclink/specs/auth/spec.md` 存在、`jq .specSync` 顯示 1 capability synced
6. JSON output 不含 secret

**Scope boundaries**：

- **In scope**：本 design 涵蓋的 trait 變更、CLI 子命令、3 個 spec、spec_delta module、archive 流程
- **Out of scope**：HTTP provider 的 archive_change 實作（trait 簽章已釘住但實作延後）、archive 反向、restore、archive bundle 跨機器、analyze cross-spec、spec schema validator

## Risks / Trade-offs

- **[archive 流程的 multi-step 原子性風險]** Mitigation：詳列步驟與 rollback 策略；`.bak` 備份在 archive 完成後刪除；整合測試模擬失敗點驗證 rollback。即便最壞情況（rollback 失敗），main spec `.bak` 與 active 目錄保留可供人工修復。
- **[spec delta merge 找不到對應 requirement 的失敗使用者體驗]** Mitigation：error message 含「找不到的 requirement name」與「main spec 中已存在的 requirement 列表」前 5 個；AI skill 可以根據錯誤訊息判斷是否需要回頭修 delta 或 main spec。
- **[`std::fs::rename` 跨 filesystem 不 atomic]** Mitigation：本 change 假設 `.speclink/` 內所有 rename 操作在同一 filesystem（`.speclink/changes/` 與 `.speclink/changes/archive/` 同根）。若使用者把 `.speclink/changes/archive/` 透過 symlink 跨 mount point，rename 會回 `EXDEV` — 此情境視為非預期使用，由 `internal.error` 反映。
- **[delta parser 採手寫 vs 引入 markdown crate]** Mitigation：本版採手寫 line-by-line scanner（< 200 LoC 預估），避免引入 pulldown-cmark 等較重 dependency；範圍小、語法固定（4 種 heading + `### Requirement:`），手寫 maintenance 成本可控。
- **[`State::Archived` 對既有 JSON 序列化的相容性]** Mitigation：既有 `propose create` snapshot 不會出現 `archived`（propose 階段 state 仍為 `proposed`）；新加 variant 為 enum 末尾，無 wire format 衝擊。AI skill 解析 JSON 應 forward-compatible，看到未知 state 不應 panic — 本 change 不主動處理舊版 AI skill 相容性。
- **[`archived_at` 時區固定 UTC，本機時區資訊遺失]** Mitigation：CLI 印出時亦為 UTC；若使用者要本機時區可在 client 端格式化。維持簡單 — 不支援 timezone 選擇旗標。
- **[provider-local 新增依賴 runtime 帶來的循環風險]** Mitigation：runtime 既有未依賴 provider-local；provider-local → runtime → provider 為單向。Cargo workspace 編譯時驗證無循環。

## Migration Plan

N/A — 本 change 新增功能，不變更既有資料。從 bootstrap 開始的 SpecLink 部署在升到本 change 後：

- `.speclink/changes/<id>/metadata.json` 既有檔案無 `archivedAt` 欄位 — 視為 archive 前狀態，本 change 無 backfill 需求
- `.speclink/specs/` 目錄首次出現由 archive 觸發；既有部署無此目錄
- archive 對 bootstrap 或 Change 2 期間建立的 change 直接適用（無需資料轉換）

## Open Questions

- **`--dry-run` 是否該對 SQLite 也 read-only？** 本 change 設定為「`dry_run=true` 直接於步驟 3 後返回，SQLite 完全不碰」。若未來 archive 增加更多 side effect（例如 finish report 送出），需重新評估 dry-run 邊界。
- **同日多次 archive 同一 change id 是否允許？** 本 change 設定為「同名 archive 目錄已存在則拒絕」對應 `archive.change_not_archivable`。若使用者真的要重新 archive（修正 delta 後），目前需手動移走舊 archive；考慮未來加 `--overwrite-archive`。
- **`### Requirement:` 名稱中含 backtick（如 ``` `propose create` command surface ```）是否會破壞 parser？** 本 change parser 把整行 heading（去前綴 `### Requirement: `）視為純字串 key，backtick 不影響匹配。整合測試需驗證此情境。
