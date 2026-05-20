# Tasks: add-local-archive

每個任務嚴守 TDD（紅→綠→重構）：實作前必先寫對應失敗測試。`[P]` 標記表示該任務可與相鄰 `[P]` 任務並行（不同檔案、無 incomplete 依賴）。

## 1. 主 spec 目錄落在 `.speclink/specs/<capability>/spec.md` 而非專案根目錄

- [x] 1.1 在 `crates/provider-local/src/storage.rs` 新增 pub `main_spec_dir(base: &Path) -> PathBuf`、`main_spec_path(base: &Path, capability: &str) -> PathBuf`：路徑硬編 `.speclink/specs/<capability>/spec.md`。**Behavior**：spec `Local provider directory layout` 規定的主 spec 位置。**驗證**：`storage.rs` 單元測試 `main_spec_path_returns_expected` 斷言 path components 為 `.speclink` / `specs` / capability / `spec.md`，並在 Windows path 用 `PathBuf::join` 不出現硬編分隔符。

## 2. `Provider::archive_change` 簽章與回傳型別

- [x] 2.1 撰寫 `crates/provider/tests/dyn_provider_compile.rs` 擴充：mock provider 補上 `archive_change` 實作，回傳一個假 `ArchivedChange`。**Behavior**：spec `Archive rollback safeguards` 與 `archive command surface` 共同要求的 trait 入口。**驗證**：紅燈（trait 與型別尚未定義）。
- [x] 2.2 在 `crates/provider/src/model.rs` 新增 `ArchiveOptions { dry_run: bool, archive_date: chrono::NaiveDate }`、`ArchivedChange { change_id, archive_path, state, archived_at, spec_sync, dry_run }`、`SpecDeltaSummary { capabilities_synced: Vec<CapabilitySyncResult> }`、`CapabilitySyncResult { capability, main_spec_path, added_count, modified_count, removed_count, renamed_count, created_main_spec }`，全部 `serde(rename_all = "camelCase")`。**驗證**：`model.rs` 單元測試 `archived_change_serializes_camelcase` 驗證 JSON 欄位為 `archivePath` / `archivedAt` / `specSync` / `capabilitiesSynced` 等 camelCase。
- [x] 2.3 在 `crates/provider/src/lib.rs` 的 `Provider` trait 新增 `async fn archive_change(&self, project_id: &ProjectId, change_id: &ChangeId, options: ArchiveOptions) -> Result<ArchivedChange, ProviderError>;`。**驗證**：2.1 編譯通過。
- [x] 2.4 [P] 在 `crates/provider/src/error.rs` 新增 `ProviderError` variants：`ChangeNotArchivable { reason: String }`、`SpecDeltaConflict { capability: String, requirement: String, operation: &'static str }`、`SpecDeltaParseError { capability: String, message: String }`；`error_code()` 對應 `archive.change_not_archivable` / `spec.delta_conflict` / `spec.delta_parse_error`。**驗證**：`error.rs` 單元測試擴充。
- [x] 2.5 [P] 在 `crates/provider-local/src/error.rs` 新增對應 `LocalProviderError` variants 與 `error_code()` mapping；新增 `RollbackFailed { tmp_files: Vec<String>, backup_files: Vec<String>, source: Box<LocalProviderError> }` 用於 `Archive rollback safeguards` 步驟 5-7 失敗且 rollback 失敗的情境。**驗證**：`error.rs` 單元測試。

## 3. `ArchiveOptions::archive_date` 由 caller 注入

- [x] 3.1 [P] 在 `crates/cli/src/commands/archive.rs` 入口處取得 `chrono::Local::now().date_naive()` 並組裝 `ArchiveOptions`；不要在 provider-local 內部呼叫時鐘。**Behavior**：design 章節 ``` `ArchiveOptions::archive_date` 由 caller 注入 ``` 的決策。**驗證**：`crates/cli/tests/archive.rs` 整合測試用環境變數 `SPECLINK_TEST_ARCHIVE_DATE=2026-05-19` 覆寫日期（CLI 提供 test-only override hook），驗證 archive 目錄前綴固定。

## 4. Spec delta merge 演算法位置：`crates/runtime/src/spec_delta.rs`

- [x] 4.1 在 `crates/provider-local/Cargo.toml` 新增 `runtime = { path = "../runtime" }` dependency。**Behavior**：design 決策 — spec_delta 在 runtime crate、provider-local 呼叫。**驗證**：`cargo build --workspace` 無循環錯誤。
- [x] 4.2 [P] 撰寫 `crates/runtime/src/spec_delta.rs` 的 ParsedDelta 結構單元測試骨架：`pub struct ParsedDelta { pub added: Vec<RequirementBlock>, pub modified: Vec<RequirementBlock>, pub removed: Vec<RequirementBlock>, pub renamed: Vec<RenamedEntry> }`、`pub struct RequirementBlock { pub name: String, pub content: String }`、`pub struct RenamedEntry { pub from: String, pub to: String, pub content: String }`。**驗證**：`cargo test -p runtime spec_delta::tests::parsed_delta_default_empty` 綠燈。

## 5. Spec delta heading 解析規則

- [x] 5.1 撰寫 `crates/runtime/src/spec_delta.rs` 的 `parse_delta` 紅燈測試：輸入只含 `## ADDED Requirements` 與 2 個 `### Requirement:` 區塊，預期 `added.len() == 2` / `modified.is_empty()`；輸入含未知 heading `## DEPRECATED Requirements` 預期 `SpecDeltaError::Parse`；輸入同 heading 出現兩次預期 `SpecDeltaError::Parse`。**Behavior**：spec `Delta heading recognition` 三個 scenario。**驗證**：紅燈。
- [x] 5.2 在 `crates/runtime/src/spec_delta.rs` 實作 `parse_delta(content: &str) -> Result<ParsedDelta, SpecDeltaError>`：line-by-line scanner 偵測 `## ADDED Requirements` 等四種固定字串、其他 `## ` 開頭一律 Parse error、重複 heading 一律 Parse error；leading whitespace 視為內容。**驗證**：5.1 綠燈。
- [x] 5.3 [P] 撰寫並實作 `RENAMED` 區塊 parsing：每個 `### Requirement:` 內必有 `**FROM:**` 與 `**TO:**`，缺一回 Parse error。**Behavior**：spec `Delta heading recognition` 對 RENAMED 的補充規則（透過 spec `RENAMED requirements rename-only semantics` 規定的 FROM/TO 行）。**驗證**：`spec_delta.rs` 單元測試 `parse_renamed_requires_from_and_to` 紅燈→實作後綠燈。

## 6. Requirement 區塊邊界以下一個 `### Requirement:` 或下一個 `## ` 為界

- [x] 6.1 撰寫 `crates/runtime/src/spec_delta.rs` 的 `parse_delta_requirement_with_nested_scenarios` 與 `parse_delta_requirement_name_with_backticks` 測試：前者驗證 nested `#### Scenario:`、`##### Example:` 內容歸屬於上層 requirement；後者驗證 ``### Requirement: `artifact write` command surface`` 名稱解析為 ``` `artifact write` command surface ```（含 backtick）。**Behavior**：spec `Requirement block delimitation` 兩個 scenario。**驗證**：紅燈。
- [x] 6.2 在 5.2 既有 `parse_delta` 補上 requirement block 邊界邏輯：遇到下一個 `### Requirement:`、下一個 `## `、或 EOF 時結束；名稱以 trim 後完整字串為 key。**驗證**：6.1 綠燈。

## 7. ADDED requirements append semantics（apply 規則）

- [x] 7.1 撰寫 `crates/runtime/src/spec_delta.rs::apply_delta` 紅燈測試：(a) main 為 `None` + delta 含 ADDED → 結果含 ADDED 內容；(b) main 為 `Some(s)` 含 `### Requirement: X` 且 delta ADDED 同名 → `SpecDeltaError::Conflict { operation: "ADDED" }`；(c) main 為 `None` 結果 `created_main_spec = true`、main 為 `Some(_)` 結果 `created_main_spec = false`。**Behavior**：spec `ADDED requirements append semantics` 兩個 scenario。**驗證**：紅燈。
- [x] 7.2 在 `crates/runtime/src/spec_delta.rs` 實作 `apply_delta(main: Option<&str>, delta: &ParsedDelta) -> Result<(String, ApplySummary), SpecDeltaError>` 的 ADDED 分支：append 到 main 末尾；若名稱衝突回 Conflict。`ApplySummary` 含 `added_count` / `modified_count` / `removed_count` / `renamed_count` / `created_main_spec`。**驗證**：7.1 綠燈。

## 8. MODIFIED requirements replace semantics

- [x] 8.1 撰寫 `apply_delta` 的 MODIFIED 紅燈測試：(a) main 含 `### Requirement: Token rotation`（1 scenario）+ delta MODIFIED `Token rotation`（3 scenarios）→ 結果為 3 scenarios；(b) main 不含 `Missing Req` + delta MODIFIED `Missing Req` → Conflict。**Behavior**：spec `MODIFIED requirements replace semantics` 兩個 scenario。**驗證**：紅燈。
- [x] 8.2 在 `apply_delta` 實作 MODIFIED 分支：以 requirement name 字串完整匹配 + 整段替換邊界（從 `### Requirement: <name>` 到下一 `### Requirement:` / `## ` / EOF）。**驗證**：8.1 綠燈。

## 9. REMOVED requirements delete semantics

- [x] 9.1 撰寫 `apply_delta` 的 REMOVED 紅燈測試：(a) main 含 `A`、`B`、`C` 三個 requirement，delta REMOVED `B` → 結果只剩 `A` 與 `C`、A 與 C 之間不出現多餘空行；(b) delta REMOVED 區塊內含 `**Reason**:` 與 `**Migration**:` 不影響 main spec 文字。**Behavior**：spec `REMOVED requirements delete semantics` 兩個 scenario。**驗證**：紅燈。
- [x] 9.2 在 `apply_delta` 實作 REMOVED 分支：定位區塊邊界、刪除整段含尾隨單一空行（多於 1 行的空白保留）。**驗證**：9.1 綠燈。

## 10. RENAMED requirements rename-only semantics

- [x] 10.1 撰寫 `apply_delta` 的 RENAMED 紅燈測試：(a) main 含 `### Requirement: User login` + delta RENAMED `FROM: User login` `TO: Sign-in` → main 結果為 `### Requirement: Sign-in`，body 保留；(b) delta RENAMED 缺 `**FROM:**` → Parse error。**Behavior**：spec `RENAMED requirements rename-only semantics` 兩個 scenario。**驗證**：紅燈。
- [x] 10.2 在 `apply_delta` 實作 RENAMED 分支：只改 heading 該行的名稱字串，其餘 body 不動。**驗證**：10.1 綠燈。

## 11. Apply order across heading sections

- [x] 11.1 撰寫 `apply_delta` 的 ordering 測試：delta 同時含 RENAMED `FROM: A TO: B` 與 MODIFIED `### Requirement: B`（新內容）→ 最終 main spec 為 `### Requirement: B` 帶 MODIFIED 內容（即先 RENAMED 再 MODIFIED）。**Behavior**：spec `Apply order across heading sections` scenario。**驗證**：紅燈→實作 10.2 後綠燈（apply_delta 內部固定 RENAMED → REMOVED → MODIFIED → ADDED 順序）。

## 12. Apply summary output

- [x] 12.1 撰寫 `apply_delta` summary 測試：delta 含 2 ADDED + 1 MODIFIED + 1 REMOVED + 0 RENAMED → summary 對應 counts，apply 失敗時不回傳 partial summary。**Behavior**：spec `Apply summary output` scenario。**驗證**：紅燈→實作 7.2/8.2/9.2/10.2 後綠燈。

## 13. Archive 流程的原子性與 rollback 策略

- [x] 13.1 撰寫 `crates/provider-local/tests/archive_integration.rs` 的 happy path 整合測試：`propose create demo` → `artifact write spec --capability auth` → `archive demo` → 斷言 `.speclink/changes/demo/` 不存在、`.speclink/changes/archive/YYYY-MM-DD-demo/` 存在、`.speclink/specs/auth/spec.md` 存在內容含 ADDED requirements、metadata.json `state == "archived"` 且含 `archivedAt`、`in_progress_change` 表為空。**Behavior**：spec `Archive rollback safeguards` 的 9 步驟成功路徑。**驗證**：紅燈→實作 13.5 後綠燈。
- [x] 13.2 [P] 撰寫 `archive_integration.rs` 的 already-archived 拒絕測試：對已 archive 的 change 再次 archive 應回 `archive.change_not_archivable`。**Behavior**：spec `` `archive` command surface `` 的 already-archived scenario + spec `Archive directory naming and uniqueness` scenario。**驗證**：紅燈。
- [x] 13.3 [P] 撰寫 `archive_integration.rs` 的 same-day rejected 測試：手動建立 `.speclink/changes/archive/YYYY-MM-DD-demo/` 後執行 archive 應拒絕。**Behavior**：spec `Archive directory naming and uniqueness` scenario。**驗證**：紅燈。
- [x] 13.4 撰寫 `archive_integration.rs` 的 rollback 測試：使用 `tempfile::TempDir` 並把 `.speclink/specs/auth/spec.md` 預先建立 `.bak`、然後讓步驟 7（rename 主 spec）失敗（注：以 `MockFilesystem` 不可行；改以「主 spec 目錄設為 readonly」觸發 `rename` 失敗），驗證 `.bak` 內容仍可被讀回、active dir 仍在原位。**Behavior**：spec `Archive rollback safeguards` 的 `Failed final rename rolls back main spec` scenario。**驗證**：紅燈。
- [x] 13.5 在 `crates/provider-local/src/lib.rs` 實作 `Provider::archive_change`：遵循 spec `Archive rollback safeguards` 9 個步驟順序、`.bak` 備份、失敗 rollback。**Behavior**：spec `Archive rollback safeguards` 完整契約。**驗證**：13.1、13.2、13.3、13.4 綠燈。
- [x] 13.6 [P] 在 `crates/provider-local/src/lib.rs::archive_change` 補 SQLite 步驟 8 的 idempotent 行為：`DELETE` 無 row 不視為失敗。**Behavior**：spec `Archive rollback safeguards` 的 `Idempotent SQLite cleanup` scenario。**驗證**：`archive_integration.rs` 額外測試手動清空 SQLite 再 archive 一個 change。

## 14. `--dry-run` 旗標的行為

- [x] 14.1 撰寫 `crates/cli/tests/archive.rs` 的 dry-run 測試：archive 後驗證 `data.dryRun == true`、`data.archivePath` 為預期路徑（但實際目錄不存在）、`.speclink/specs/` 不存在、active dir 仍存在；用 `--dry-run` + 衝突 delta 仍回 `spec.delta_conflict`。**Behavior**：spec `` `archive` command surface `` 的 dry-run scenario + spec `` `archive` failure mapping `` 的 dry-run conflict scenario。**驗證**：紅燈。
- [x] 14.2 在 `crates/runtime/src/archive.rs` 與 `crates/provider-local/src/lib.rs::archive_change` 實作 `ArchiveOptions::dry_run`：完成 delta merge 運算（步驟 1-2）後返回，不執行步驟 3-9。**驗證**：14.1 綠燈。

## 15. CLI `speclink archive <change>` 子命令採 positional argument

- [x] 15.1 撰寫 `crates/cli/src/cli.rs` 的 clap parse 測試：合法 `speclink archive demo`、`speclink archive demo --dry-run --json`、`speclink archive demo --stdin` 拒絕、`speclink archive Add-Feature` 拒絕（kebab-case 驗證）。**Behavior**：spec `` `archive` command surface `` 的 invocation 形式。**驗證**：紅燈。
- [x] 15.2 在 `crates/cli/src/cli.rs` 新增 `Command::Archive(ArchiveArgs)` 與 `ArchiveArgs { change: String, dry_run: bool, flags: MachineInterfaceFlags }`；`change` 為 positional + value_parser `parse_change_id`（複用 Change 2 提取的 helper）。**驗證**：15.1 綠燈。

## 16. Error code 新增清單

- [x] 16.1 [P] 在 `crates/cli/src/exit_code.rs::classify_provider` 與 `classify_local` 為新 variants 加 mapping：`ChangeNotArchivable` → (1, `archive.change_not_archivable`)、`SpecDeltaConflict` → (7, `spec.delta_conflict`)、`SpecDeltaParseError` → (2, `spec.delta_parse_error`)。**Behavior**：design 章節 `Error code 新增清單` 列舉的 3 個新 codes。**驗證**：`exit_code.rs` 既有測試擴充 + `all_error_codes_match_naming_regex` 涵蓋新 codes。

## 17. Lifecycle state value `archived`

- [x] 17.1 撰寫 `crates/provider/src/model.rs` 的 `state_archived_round_trip` 測試：`State::Archived` 序列化為 `"archived"`、反序列化還原。**Behavior**：spec `Lifecycle state value `archived`` scenario。**驗證**：紅燈。
- [x] 17.2 在 `crates/provider/src/model.rs::State` enum 新增 variant `Archived`，沿用 `rename_all = "lowercase"`。既有 `Draft`、`Proposed` 不變。**驗證**：17.1 綠燈、既有 propose create snapshot 不變（archive 階段才會出現 archived state）。

## 18. `archivedAt` metadata field

- [x] 18.1 撰寫 `crates/provider-local/tests/archive_integration.rs` 的 `metadata_after_archive_contains_archivedAt` 測試：archive 完成後讀 metadata.json，斷言 `state == "archived"`、`archivedAt` 為有效 ISO 8601 UTC、`createdAt` / `createdBy` / `changeId` 保留原值。**Behavior**：spec `` `archivedAt` metadata field `` scenario。**驗證**：紅燈→實作 13.5 後綠燈。

## 19. Integration、snapshot 與 polish

- [x] 19.1 [P] 撰寫 `crates/cli/tests/archive_snapshots.rs`：3 條 insta snapshot — archive success（含 spec sync summary）、dry-run success、`spec.delta_conflict` failure。固定 `SPECLINK_TEST_REQUEST_ID` 與 `SPECLINK_TEST_ARCHIVE_DATE`。**驗證**：`cargo insta accept --workspace` 後 `cargo test --workspace` 通過。
- [x] 19.2 [P] 在 `crates/cli/src/output.rs` 新增 `ArchiveData { change_id, archive_path, state, archived_at, dry_run, spec_sync: SpecSyncSummaryJson }`、`SpecSyncSummaryJson { capabilities_synced }`、`CapabilitySyncResultJson { capability, main_spec_path, added_count, modified_count, removed_count, renamed_count, created_main_spec }`，皆 `serde(rename_all = "camelCase")`。**Behavior**：spec `` `archive` JSON output schema ``。**驗證**：`output.rs` 單元測試 JSON 序列化。
- [x] 19.3 [P] 新增 `crates/runtime/src/archive.rs::{ArchiveInput, archive}`：取 `ArchiveOptions` + `provider.archive_change()` 純轉發；不在 runtime 做額外校驗（policy 在 provider）。**Behavior**：spec `` `archive` command surface `` 的「runtime 把 archive_date 傳到 provider」。**驗證**：`archive.rs` mock provider 測試。
- [x] 19.4 [P] 新增 `crates/cli/src/commands/archive.rs::run(args: ArchiveArgs) -> anyhow::Result<()>`：取 `chrono::Local::now().date_naive()`（或 `SPECLINK_TEST_ARCHIVE_DATE` 覆寫）→ resolve provider → 建 LocalProvider → 呼叫 `runtime::archive::archive` → 印 `Envelope<ArchiveData>`。**Behavior**：spec `` `archive` command surface `` 的 happy / dry-run / failure scenarios。**驗證**：`crates/cli/tests/archive.rs` assert_cmd 整合測試覆蓋全部 scenarios。
- [x] 19.5 [P] 在 `crates/cli/src/commands/mod.rs` 註冊 `pub mod archive;`、`main.rs` 加 `Command::Archive(_)` dispatch。**驗證**：`cargo build --workspace` 通過。
- [x] 19.6 [P] 為新公開 API（`Provider::archive_change`、`ArchiveOptions`、`ArchivedChange`、`SpecDeltaSummary`、`CapabilitySyncResult`、`State::Archived`、`runtime::archive::archive`、`runtime::spec_delta::parse_delta`、`apply_delta`、`Envelope::data` 新型別）補繁體中文 `///` doc comment。**驗證**：`cargo doc --workspace --no-deps` 無 missing doc warning。
- [x] 19.7 [P] 更新 `README.md`：在 status 範例後新增 archive（success + dry-run + delta conflict）三段範例。**驗證**：手動 review。
- [x] 19.8 跨平台 path 處理稽核：archive directory rename 在同 filesystem 內、`mainSpecPath` JSON 欄位走 `to_posix_string` helper（沿用 Change 2 新增）。**Behavior**：spec `Local provider directory layout` 的 `Cross-platform path separator handling` scenario。**驗證**：CI 三平台矩陣綠燈。
- [x] 19.9 跑 `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets` 全綠。**驗證**：CI 通過。
