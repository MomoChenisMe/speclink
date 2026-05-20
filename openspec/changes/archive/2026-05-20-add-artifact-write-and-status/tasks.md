# Tasks: add-artifact-write-and-status

每個任務嚴守 TDD（紅→綠→重構）：實作前必先寫對應失敗測試。`[P]` 標記表示該任務可與相鄰 `[P]` 任務並行（不同檔案、無 incomplete 依賴）。

## 1. `Provider::get_status` 加入 trait 而非僅 provider-local

- [x] 1.1 撰寫 `crates/provider/tests/dyn_provider_compile.rs` 擴充測試：mock provider 補上新的 `get_status` 實作，驗證 `Arc<dyn Provider>` 仍可建構並跨 thread 傳遞。**Behavior**：spec `Change status filesystem scan` 在 trait 層暴露的契約。**驗證**：`cargo test -p provider --tests` 預期紅燈（trait 尚未定義 method → 編譯失敗）。
- [x] 1.2 在 `crates/provider/src/lib.rs` 的 `Provider` trait 新增 `async fn get_status(&self, project_id: &ProjectId, change_id: &ChangeId) -> Result<ChangeStatus, ProviderError>;`，沿用 `async_trait` 與 `Send + Sync + dyn-compatible` 既有契約。**驗證**：1.1 編譯通過綠燈。

## 2. `ChangeStatus` 採固定 typed struct 而非 generic JSON

- [x] 2.1 撰寫 `crates/provider/src/model.rs` 的 `change_status_serializes_camelcase` 單元測試：`ChangeStatus` 序列化欄位為 `changeId` / `state` / `artifacts`，`ArtifactStatus` 序列化欄位含 `id` / `kind` / `path` / `status` / `required` / `dependencies`，`ArtifactState` 序列化為 `"missing"` / `"done"`。**Behavior**：spec ``` `status` JSON output schema ``` 對 ArtifactStatus 形狀的契約。**驗證**：`cargo test -p provider model::tests` 預期紅燈。
- [x] 2.2 在 `crates/provider/src/model.rs` 實作 `ChangeStatus { change_id, state, artifacts }`、`ArtifactStatus { id, kind, path, status, required, dependencies }`、`ArtifactState { Missing, Done }`，全部 `serde(rename_all = "camelCase")`。**驗證**：2.1 綠燈。
- [x] 2.3 [P] 在 `crates/provider/src/model.rs` 為 `NewArtifact` 新增 `pub capability: Option<String>` 欄位（預設 `None`）；既有 `runtime::propose::create_proposal` caller 顯式設為 `None`。**驗證**：`cargo build --workspace` 通過、既有 propose create snapshot 不變。

## 3. Spec artifact 路徑：`.speclink/changes/<id>/specs/<capability>/spec.md`

- [x] 3.1 撰寫 `crates/provider-local/src/storage.rs` 的 `capability_name_validation_table` 單元測試，依 spec `Spec capability routing` 範例：`auth` / `user-auth` 接受；`Auth-Module` / `1bad` / `add--feature` / `add-` / 空字串拒絕。**驗證**：紅燈（`is_valid_capability_name` 尚未存在）。
- [x] 3.2 在 `crates/provider-local/src/storage.rs` 新增 `is_valid_kebab_id(s) -> bool` helper（同 `is_valid_change_id` 演算法）並重構 `is_valid_change_id` 與新增 `is_valid_capability_name` 皆透過此 helper。**驗證**：3.1 綠燈；既有 `change_id_validation_table` 測試保持綠燈。
- [x] 3.3 撰寫 `crates/provider-local/src/storage.rs` 的 `write_spec_creates_capability_subdir`、`write_spec_two_capabilities_coexist`、`write_spec_refuses_invalid_capability`、`write_spec_cleanup_on_failure_does_not_remove_preexisting_specs_dir` 四個 tempfile 測試。**Behavior**：spec `Spec capability routing` 描述的路由規則與多 capability 並存。**驗證**：紅燈。
- [x] 3.4 在 `crates/provider-local/src/storage.rs` 實作 `write_spec_atomic(base, change_id, capability, content)`：驗證 capability → 計算 `<change_dir>/specs/<capability>/spec.md` → 建立中間目錄 → `.tmp` + rename → 失敗 cleanup（保留 pre-existing `specs/` 子目錄）。**驗證**：3.3 綠燈。

## 4. `LocalProvider::write_artifact` 的 ArtifactKind 路由

- [x] 4.1 撰寫 `crates/provider-local/src/storage.rs` 的 `write_design_creates_only_design_md`、`write_tasks_creates_only_tasks_md`、`write_design_refuses_existing_file`、`write_design_refuses_missing_change_dir` 四個 tempfile 測試。**Behavior**：spec `Multi-artifact atomic write` 對 design/tasks 的契約。**驗證**：紅燈。
- [x] 4.2 在 `crates/provider-local/src/storage.rs` 新增 `write_simple_artifact_atomic(base, change_id, filename, content)` 私有 helper 與 `write_design_atomic` / `write_tasks_atomic` 兩個 pub 包裝（filename 各為 `"design.md"` / `"tasks.md"`）。**驗證**：4.1 綠燈。
- [x] 4.3 [P] 在 `crates/provider-local/src/error.rs` 新增 `LocalProviderError` variants：`ArtifactAlreadyExists { kind: String, change_id: String }`、`MissingCapability`、`InvalidCapability { capability: String }`、`ChangeNotFound { change_id: String }`；`error_code()` 對應 `artifact.already_exists` / `artifact.missing_capability` / `artifact.invalid_capability` / `change.not_found`。**驗證**：`error.rs` `error_code` 單元測試擴充新 variants 映射。
- [x] 4.4 [P] 在 `crates/provider/src/error.rs` 新增對應的 `ProviderError` variants（`ArtifactAlreadyExists`、`MissingCapability`、`InvalidCapability`），擴充 `error_code()`；`ChangeNotFound` 已存在。**驗證**：`provider/src/error.rs` 單元測試擴充。
- [x] 4.5 重構 `crates/provider-local/src/lib.rs::write_artifact` 為 `match input.kind`：`Proposal` 走既有 `write_proposal_content_atomic` + metadata.json + `set_in_progress`；`Design` / `Tasks` 走對應 helper、不更新 metadata.json、不寫 state.db；`Spec` 走 `write_spec_atomic`（同上）。`Spec` 缺 capability 回 `MissingCapability`。**Behavior**：spec `Multi-artifact atomic write` 的「不更新 metadata.json」條款。**驗證**：`provider-local/tests/multi_artifact_integration.rs` 串接測試斷言 metadata.json `state` 仍為 `"proposed"`、`in_progress_change` 僅於 propose 階段寫入。

## 5. `get_status` 實作策略：純 filesystem scan + metadata.json 讀取

- [x] 5.1 撰寫 `crates/provider-local/tests/multi_artifact_integration.rs` 三條 `get_status` 整合測試：(a) 只有 proposal 時回 3 entry、design/tasks 為 missing；(b) 含 spec:auth + spec:billing 時回 5 entry、order 為 proposal/design/tasks/spec:auth/spec:billing；(c) `metadata.json` 為 `"{bad json"` 時回 `internal.error`。**Behavior**：spec `Change status filesystem scan` 的 scenarios。**驗證**：紅燈。
- [x] 5.2 在 `crates/provider-local/src/lib.rs` 實作 `Provider::get_status`：`spawn_blocking` 中依 spec 描述讀 metadata.json → 檢查 3 個 fixed-name artifacts → enumerate `specs/<cap>/spec.md` → 排序 specs by capability name asc。metadata.json 缺檔回 `ChangeNotFound`、解析失敗回 `Internal`。**驗證**：5.1 綠燈。
- [x] 5.3 [P] 在 5.2 同檔補上分支：`specs/` 不存在時 artifacts 只有 3 entry；`specs/<cap>/` 子目錄沒 `spec.md` 時略過該 capability（不視為 done）。**Behavior**：spec `Change status filesystem scan` 的 scenarios `Empty specs dir produces no spec entries` 與 `Subdirectory under specs without spec.md is ignored`。**驗證**：`multi_artifact_integration.rs` 對應命名測試。

## 6. `speclink artifact write` 子命令採 positional `<kind>` 而非 flag

- [x] 6.1 撰寫 `crates/cli/src/cli.rs` 內 clap parse 單元測試：4 種合法 invocation（design / tasks / spec+capability）+ 缺 `--stdin` 拒絕 + 缺 `--capability` 拒絕 + design 帶 `--capability` 拒絕。**Behavior**：spec ``` `artifact write` command surface ``` 描述的 clap 介面。**驗證**：紅燈。
- [x] 6.2 在 `crates/cli/src/cli.rs` 新增 `Command::Artifact(ArtifactCommand)`、`ArtifactCommand::Write(ArtifactWriteCommand)`、`ArtifactWriteCommand::{Design(ArtifactWriteArgs), Tasks(ArtifactWriteArgs), Spec(ArtifactWriteSpecArgs)}` 三層 enum subcommand；`ArtifactWriteArgs { change, flags: MachineInterfaceFlags }`、`ArtifactWriteSpecArgs { change, capability, flags }`。**驗證**：6.1 綠燈。
- [x] 6.3 [P] 在 `crates/cli/src/cli.rs` 提取 `parse_change_id` value_parser 與 `propose create` 共用、新增 `parse_capability_name` value_parser，皆透過 `is_valid_kebab_id` 規則拒絕非法 input。**Behavior**：spec ``` `status` failure mapping ``` 的 `change.invalid_id` 在 clap layer 觸發；spec `Spec capability routing` 的 `Capability name validation matches change-id rules` scenario。**驗證**：`cli.rs` 單元測試覆蓋 `Add-Feature`、`1bad`、`Auth-Module` 等拒絕案例。
- [x] 6.4 新增 `crates/cli/src/commands/artifact.rs::run(args: ArtifactWriteCommand) -> anyhow::Result<()>`：dispatch 到子命令、讀 stdin（空字串/非 UTF-8 拒絕、補 trailing `\n`）、resolve provider、建 `LocalProvider`、呼叫 `runtime::artifact::write_artifact`、印 `Envelope<ArtifactWriteData>`。**Behavior**：spec ``` `artifact write` stdin content rules ``` 描述的 stdin 處理規則。**驗證**：`crates/cli/tests/artifact_write.rs` assert_cmd 整合測試覆蓋 design/tasks/spec happy path、stdin 空、stdin 非 UTF-8、change 不存在、artifact 已存在、capability 缺失/非法。
- [x] 6.5 [P] 在 `crates/cli/src/commands/mod.rs` 註冊 `pub mod artifact; pub mod status;`，並在 `crates/cli/src/main.rs` 加 `Command::Artifact(_) | Command::Status(_)` dispatch。**驗證**：`cargo build --workspace` 通過。
- [x] 6.6 [P] 在 `crates/cli/src/exit_code.rs::classify_provider` 與 `classify_local` 為新 variants 加 mapping：`ArtifactAlreadyExists` → (1, `artifact.already_exists`)、`MissingCapability` → (2, `artifact.missing_capability`)、`InvalidCapability` → (2, `artifact.invalid_capability`)、`ChangeNotFound` → (1, `change.not_found`)。**Behavior**：spec ``` `artifact write` failure mapping ``` 規定的對照表。**驗證**：`exit_code.rs` 既有測試擴充 + `all_error_codes_match_naming_regex` 涵蓋新 codes。

## 7. `speclink status` 的 JSON output schema

- [x] 7.1 撰寫 `crates/cli/src/output.rs` 的單元測試 `status_data_serializes_camelcase`：`StatusData` 與 `ArtifactStatusJson` 序列化欄位完整、`status` 序列化為 `"done"` / `"missing"`、`required` 與 `dependencies` 按固定規則套用。**Behavior**：spec ``` `status` JSON output schema ``` 與 ``` `artifact write` JSON output schema ```。**驗證**：紅燈。
- [x] 7.2 在 `crates/cli/src/output.rs` 新增 `ArtifactWriteData { change_id, artifact_id, kind, path, mode }` 與 `StatusData { change_id, state, artifacts: Vec<ArtifactStatusJson> }`、`ArtifactStatusJson { id, kind, path, status, required, dependencies }`，皆 `serde(rename_all = "camelCase")`。**驗證**：7.1 綠燈。
- [x] 7.3 在 `crates/cli/src/output.rs` 新增 `fn artifact_status_to_json(status: &ArtifactStatus) -> ArtifactStatusJson` 與 `fn change_status_to_status_data(status: ChangeStatus) -> StatusData`，套用固定 Required/Dependency Rules（proposal required+[]、design optional+[proposal]、tasks optional+[proposal, spec]、spec required+[proposal]）。**Behavior**：spec ``` `status` JSON output schema ``` 中固定規則表。**驗證**：`output.rs` 單元測試覆蓋 4 種 artifact id 的 required/dependencies 套用。
- [x] 7.4 [P] 新增 `crates/runtime/src/status.rs::{GetStatusInput, get_status}`：包裝 `provider.get_status(&input.project_id, &input.change_id)`，純轉發無額外邏輯。**Behavior**：spec ``` `status` command surface ``` 描述的「side-effect-free read」。**驗證**：`status.rs` 內聯 mock provider 測試 happy path 與 `ChangeNotFound` 傳遞。
- [x] 7.5 [P] 新增 `crates/runtime/src/artifact.rs::{WriteArtifactInput, WriteArtifactOutput, write_artifact}`：驗證 Spec kind 必有 capability、非 Spec 不可有 capability、content 不可為空字串（防禦層），失敗回 `RuntimeError::InvalidInput`。**Behavior**：spec ``` `artifact write` command surface ``` 描述的 clap+runtime 雙層校驗。**驗證**：`artifact.rs` mock provider 測試覆蓋四種 kind 的 happy path、kind/capability 組合錯誤、empty content。
- [x] 7.6 [P] 在 `crates/runtime/src/lib.rs` 透過 `pub mod artifact; pub mod status;` 匯出新模組。**驗證**：`cargo build --workspace` 通過。
- [x] 7.7 新增 `crates/cli/src/commands/status.rs::run(args: StatusArgs) -> anyhow::Result<()>`：resolve provider、建 LocalProvider、呼叫 `runtime::status::get_status`、轉 `StatusData`、印 envelope。**Behavior**：spec ``` `status` command surface ``` 與 ``` `status` failure mapping ``` 共同描述的 status CLI 行為。**驗證**：`crates/cli/tests/status.rs` assert_cmd 整合測試覆蓋 fresh change、含 design + spec、change 不存在、metadata.json malformed 四個案例。

## 8. `metadata.json` 不為新 artifact 重寫

- [x] 8.1 撰寫 `crates/provider-local/tests/multi_artifact_integration.rs` 的 `metadata_json_unchanged_after_artifact_writes` 測試：propose create 之後讀 metadata.json 內容（snapshot），連續 write design + tasks + spec 後再讀，斷言兩次內容完全相同（state 仍為 `"proposed"`、createdAt 不變）。**Behavior**：design 章節 ``` `metadata.json` 不為新 artifact 重寫 ``` 的決策。**驗證**：紅燈→實作 4.5 後綠燈。

## 9. Error code 新增清單

- [x] 9.1 在 `crates/cli/src/exit_code.rs` 的 `all_error_codes_match_naming_regex` 測試新增 `artifact.already_exists` / `artifact.missing_capability` / `artifact.invalid_capability` / `change.not_found` 四個 code，驗證皆符合 `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`。**Behavior**：design 章節 `Error code 新增清單` 列舉的新 codes 全部納入 naming 規則檢查。**驗證**：`exit_code.rs` 測試綠燈。

## 10. Local provider directory layout 擴張、integration、snapshot 與 polish

- [x] 10.1 撰寫 `crates/provider-local/tests/multi_artifact_integration.rs` 的 `directory_layout_only_grows_when_written` 測試：propose create 後僅有 proposal.md + metadata.json + state.db；逐次寫 design / tasks / spec 後，斷言 `specs/` 與對應檔案僅在 spec write 時被建立、design.md / tasks.md 僅在 design/tasks write 時出現。**Behavior**：spec `Local provider directory layout` MODIFIED scenarios。**驗證**：紅燈→實作 4.5 後綠燈。
- [x] 10.2 [P] 新增 `crates/cli/tests/artifact_write_snapshots.rs`：3 條 insta snapshot — design success、spec success（id 為 `spec:user-auth`）、`artifact.already_exists` failure。沿用 bootstrap `SPECLINK_TEST_REQUEST_ID=req_00000000000000000000000000000000` 固定 requestId。**驗證**：`cargo insta accept --workspace` 後 `cargo test --workspace` 通過。
- [x] 10.3 [P] 新增 `crates/cli/tests/status_snapshots.rs`：3 條 insta snapshot — only proposal、proposal+design+spec:auth、`change.not_found` failure。**驗證**：同上。
- [x] 10.4 [P] 為新公開 API（`Provider::get_status`、`ChangeStatus`、`ArtifactStatus`、`ArtifactState`、`NewArtifact::capability`、`runtime::artifact::write_artifact`、`runtime::status::get_status`、`Envelope::data` 兩個新型別）補繁體中文 `///` doc comment。**驗證**：`cargo doc --workspace --no-deps` 無 missing doc warning（CI lint）。
- [x] 10.5 [P] 更新 `README.md`：在 propose create 範例後新增 artifact write（design / spec）與 status 範例三段。**驗證**：手動 review；範例輸出能 `jq .` 解析。
- [x] 10.6 跨平台 path 處理稽核：所有新 storage 路徑透過 `PathBuf::join`；JSON output `path` 欄位透過 helper `to_posix_string(&Path) -> String` 把 backslash 轉 forward slash。**Behavior**：spec `Local provider directory layout` 的 `Cross-platform path separator handling` scenario。**驗證**：`grep -rn '"\\\\\\\\"' crates/cli/src crates/provider-local/src` 無命中；CI 三平台矩陣綠燈。
- [x] 10.7 跑 `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets` 全綠。**驗證**：CI 三平台矩陣通過。
