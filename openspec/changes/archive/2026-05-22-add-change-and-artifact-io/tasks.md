## 0. Spec ↔ task traceability checklist

本節列出 spec 中每條 requirement 與 design 中每個 decision heading 的對應 task block；analyzer 透過此節做 substring matching 完成 coverage 追蹤。每一條打勾代表「該 requirement / decision 已有對應 task block 在下方覆蓋」。

### Change-store requirements

- [x] 0.1 Requirement covered: `State.db schema MUST be upgraded to version 2 with a `change` table` — by tasks §1.1, §1.2, §1.3
- [x] 0.2 Requirement covered: `` `speclink new change` SHALL create a change row and scaffold its directory `` — by tasks §3.1, §3.2, §7.1
- [x] 0.3 Requirement covered: `Change name grammar SHALL match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with byte length 1–64` — by tasks §2.1, §2.2, §7.2
- [x] 0.4 Requirement covered: `` `speclink list --changes` SHALL list all changes from state.db `` — by tasks §3.1, §3.2, §7.3
- [x] 0.5 Requirement covered: `` `speclink show change <name>` SHALL emit change metadata and existing artifact filenames `` — by tasks §3.1, §3.2, §7.4
- [x] 0.6 Requirement covered: `` `speclink delete change <name>` SHALL be destructive and require explicit confirmation `` — by tasks §3.1, §3.2, §5.3, §7.5
- [x] 0.7 Requirement covered: `Change state in slice A SHALL be the literal `proposing`` — by tasks §3.2, §7.1
- [x] 0.8 Requirement covered: `Change row Etag (the `version` column) SHALL start at 1 on creation` — by tasks §3.1, §3.2, §7.1

### Artifact-io requirements

- [x] 0.9 Requirement covered: `Artifact kind whitelist SHALL be `proposal`, `design`, `tasks`, `spec`` — by tasks §2.1, §2.2, §4.1, §7.6
- [x] 0.10 Requirement covered: `Artifact filesystem path SHALL be derived from change name and kind` — by tasks §4.1, §4.6, §7.6
- [x] 0.11 Requirement covered: `Capability id grammar SHALL match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with byte length 1–64` — by tasks §2.1, §2.2, §4.1
- [x] 0.12 Requirement covered: `` `speclink artifact read` SHALL return content and an Etag computed from file bytes `` — by tasks §4.2, §4.6, §7.7
- [x] 0.13 Requirement covered: `` `speclink new artifact` SHALL enforce sha256-based optimistic concurrency `` — by tasks §4.3, §4.6, §7.8
- [x] 0.14 Requirement covered: `Artifact writes SHALL use a tempfile-then-rename atomic sequence` — by tasks §4.4, §4.6
- [x] 0.15 Requirement covered: `All artifact operations SHALL require the change row to exist` — by tasks §4.6, §5.2
- [x] 0.16 Requirement covered: `` `speclink list --specs --change <name>` SHALL enumerate spec capabilities from the filesystem `` — by tasks §4.5, §4.6, §7.9
- [x] 0.17 Requirement covered: `Error envelope SHALL preserve the standard shape from slice A onward` — by tasks §6.1, §6.2

### Design decisions

- [x] 0.18 Design decision covered: `Etag = `sha256(file bytes)`，artifact 不進 state.db` — by tasks §4.2, §4.3, §4.6, §7.8
- [x] 0.19 Design decision covered: `State.db v2 migration 只加一張 `change` 表` — by tasks §1.1, §1.2
- [x] 0.20 Design decision covered: `ChangeStore 與 ArtifactStore 拆成兩個 trait` — by tasks §2.1, §2.2
- [x] 0.21 Design decision covered: `Versioned<T> + expected_etag 共用型別` — by tasks §2.1, §2.2
- [x] 0.22 Design decision covered: `Atomic write 採 tempfile-in-same-dir + rename` — by tasks §4.4, §4.6
- [x] 0.23 Design decision covered: `` `spec.list-in-change` 走 filesystem，不查 state.db `` — by tasks §4.5, §7.9
- [x] 0.24 Design decision covered: `Change name 與 capability id 共用 grammar` — by tasks §2.1, §2.2
- [x] 0.25 Design decision covered: `` `delete change` 用 `--confirm-name <name>` 而非 `--force` `` — by tasks §5.3, §7.5
- [x] 0.26a Design decision covered: `沿用 bootstrap 既有 error pattern — 不引入新 error enum` — by tasks §2.2, §2.3, §5.3, §6.1, §6.2
- [x] 0.26b Design decision covered: `LocalProvider 開檔路徑統一 `migrate(2)`` — by tasks §1.3

### Design Implementation Contract subsections

- [x] 0.26 Subsection covered: `Behavior` — by tasks §7.1 through §7.10
- [x] 0.27 Subsection covered: `Interface / data shape` — by tasks §2.2, §6.2, §7.10
- [x] 0.28 Subsection covered: `Failure modes` — by tasks §6.1, §6.2, §7.2, §7.5, §7.6, §7.7, §7.8
- [x] 0.29 Subsection covered: `Acceptance criteria` — by tasks §8.1, §9.1, §9.2, §9.3, §9.4
- [x] 0.30 Subsection covered: `Scope boundaries` — by proposal Non-Goals and tasks §1 through §9

## 1. State.db v2 migration runner 擴充（crates/provider-local）

對應 design「State.db v2 migration 只加一張 `change` 表」、「LocalProvider 開檔路徑統一 `migrate(2)`」與 spec requirement「State.db schema MUST be upgraded to version 2 with a `change` table」。**Test 寫在 `crates/provider-local/src/state_db.rs::tests` inline mod，沿用 bootstrap 既有 4 支 v1 migration test 同樣的 inline 慣例（provider-local crate 目前沒有 `tests/` 目錄，本 slice 不引入）。**

- [x] 1.1 [P] 撰寫 red 測試於 `crates/provider-local/src/state_db.rs::tests`（inline mod）：`StateDb::migrate(2)` 在 v1 db 上跑完後 `_migrations` 含 `(version=2)` row；`pragma_table_info('change')` 回傳七個欄位且型別與 NOT NULL 約束與 spec example 表一致；migration 在 v2 db 上重跑為 no-op；mid-migration 注入 fault 後 transaction rollback、`change` 表不存在。**契約**：紅燈覆蓋三條 spec requirement（first-time / idempotent / partial rollback）。**驗證**：`cargo test -p speclink-provider-local state_db::tests::migrate_v2` 顯示 unresolved symbol 或 assertion failure。
- [x] 1.2 綠燈：在 `crates/provider-local/src/state_db.rs::MIGRATIONS` 陣列追加 v2 entry，內容為新增 `change` 表的 `CREATE TABLE` SQL；migration runner 仍走原本的 transaction-wrapped path（bootstrap 既有 `unchecked_transaction` 機制不動）。**契約**：1.1 的三支 red 全綠；既有 v1 migration 測試（`crates/provider-local/src/state_db.rs` 內 `mod tests` 既有 4 支）不 regression。**驗證**：`cargo test -p speclink-provider-local` 全綠。
- [x] 1.3 把 `LocalProjectStore::open_state_db()` 內 hardcoded `db.migrate(1)` bump 為 `db.migrate(2)`（位於 `crates/provider-local/src/store.rs`）。**契約**：對應 design decision「LocalProvider 開檔路徑統一 `migrate(2)`」；bootstrap 既有 init / status / link / unlink 四條 CLI flow 跑完後 state.db 應升到 v2、且既有測試（`crates/runtime/tests/bootstrap.rs` 與 `crates/cli/tests/cli.rs`）全部仍綠。**驗證**：`cargo test --workspace` 全綠；在 tempdir 跑 `speclink init` 後查 `_migrations` 表 `MAX(version)` 回 `2`。
- [x] 1.4 [P] 在 `crates/provider-local/src/state_db.rs` 新增四個 `change` 表 helper method：`insert_change_row(change_id, name, state, schema_id, created_at, updated_at)` / `get_change_by_name(name) -> Option<ChangeRow>` / `list_changes() -> Vec<ChangeRow>` / `delete_change_by_name(name) -> bool`；命名沿用既有 `insert_project_row` / `has_project` / `single_project_id` 風格。**契約**：對應 design decision「State.db v2 migration 只加一張 `change` 表」與 spec requirement「`speclink list --changes` SHALL list all changes from state.db」、「`speclink delete change <name>` SHALL be destructive」。**驗證**：紅燈先寫 unit test（`crates/provider-local/src/state_db.rs::tests` 內），綠燈實作；`cargo test -p speclink-provider-local state_db` 全綠。
- [x] 1.5 重構：把 v1 / v2 SQL 抽出為個別常量、加 doc comment 標 schema version 對照表；確認 `MIGRATIONS` array index 對 v 編號的不變式有測試覆蓋。**契約**：clippy pedantic 通過；migration array 仍是 `&[&str]` 固定順序。**驗證**：`cargo clippy -p speclink-provider-local -- -D warnings -W clippy::pedantic` 通過。

## 2. Provider trait 與共用型別新增（crates/provider）

對應 design「ChangeStore 與 ArtifactStore 拆成兩個 trait」、「Versioned<T> + expected_etag 共用型別」、「Change name 與 capability id 共用 grammar」、「沿用 bootstrap 既有 error pattern — 不引入新 error enum」。

- [x] 2.1 [P] 紅燈（types.rs）：撰寫 serde / signature 測試於 `crates/provider/src/types.rs::tests`（沿用 bootstrap 既有 inline `mod tests` 慣例），覆蓋 `Versioned<T>`、`Etag`（含 `sha256:` prefix 驗證）、`ExpectedEtag` enum 兩變體、`ArtifactKind` 四變體（`Proposal` / `Design` / `Tasks` / `Spec`）；`ChangeRow` 七欄位 serde round-trip；`validate_kebab_id` 對 spec table 中 9 個案例（5 有效 / 4 無效）的結果。**契約**：對應 spec requirement「Change name grammar SHALL match ...」、「Capability id grammar SHALL match ...」、「Artifact kind whitelist SHALL be ...」。**驗證**：`cargo test -p speclink-provider types` 顯示 unresolved symbol。
- [x] 2.2 [P] 紅燈（error.rs / lib.rs）：在 `crates/provider/src/error.rs::tests` 與 `crates/provider/src/lib.rs::tests` 內擴充既有 `provider_error_codes_match_declared_namespace` 測試，覆蓋 7 個新 const（`CHANGE_NOT_FOUND` = `"change.not_found"`、`CHANGE_DUPLICATE_NAME` = `"change.duplicate_name"`、`CHANGE_INVALID_NAME` = `"change.invalid_name"`、`ARTIFACT_KIND_INVALID` = `"artifact.kind_invalid"`、`ARTIFACT_CAPABILITY_REQUIRED` = `"artifact.capability_required"`、`ARTIFACT_NOT_FOUND` = `"artifact.not_found"`、`ARTIFACT_VERSION_CONFLICT` = `"artifact.version_conflict"`）；測試 `ProviderError` 7 個新 variant 的 `code()` 字串與 `retryable()` 行為（只有 `ArtifactVersionConflict` 為 `true`）；新增 `DummyStore` 對 `ChangeStore` + `ArtifactStore` 的 trait shape compile-time check（沿用 bootstrap `ProjectStore` 的 `DummyStore` pattern）。**契約**：對應 design decision「沿用 bootstrap 既有 error pattern」。**驗證**：`cargo test -p speclink-provider` 顯示 red。
- [x] 2.3 綠燈：在 `crates/provider/src/types.rs` 新增 `Etag` newtype（含 `sha256:` prefix invariant 與 `Etag::from_bytes(&[u8])` constructor）、`Versioned<T>` 泛型 struct、`ExpectedEtag` enum、`ArtifactKind` enum、`ChangeRow` struct、`validate_kebab_id(&str) -> Result<(), IdError>` 共用 helper；在 `crates/provider/src/error.rs::codes` module 加 7 個 `pub const`；在 `crates/provider/src/error.rs::ProviderError` enum 加 7 個 variant（沿用 bootstrap 既有 thiserror pattern）並擴 `code()` / `retryable()` match arm；在 `crates/provider/src/lib.rs` 新增 `ChangeStore` 與 `ArtifactStore` trait（簽名與 design 章節一致，trait method 回 `Result<_, ProviderError>`），用 `#[async_trait::async_trait]` + `: Send + Sync` 標記（與 `ProjectStore` 對齊）。**契約**：2.1–2.2 全綠；**不**新增 `ChangeError` / `ArtifactError` enum；`ExpectedEtag` 不可用 `Option<Etag>` 替代（語意鎖）。**驗證**：`cargo test -p speclink-provider` 全綠。
- [x] 2.4 重構：公開 API 加繁中 doc comment（範例 code 保英文），跑 `rust-skills:m05-type-driven` 與 `rust-skills:m13-domain-error` 檢查；確認 library crate 無 `unwrap()` / `expect()` / `panic!()`。**契約**：clippy pedantic 通過。**驗證**：`cargo clippy -p speclink-provider -- -D warnings -W clippy::pedantic` 通過。

## 3. LocalChangeStore 實作（crates/provider-local）

對應 design「Implementation Contract — Provider trait」中 `ChangeStore` 四個 method 與 spec requirement「`speclink new change` SHALL create a change row...」、「`speclink list --changes` SHALL list...」、「`speclink show change` SHALL emit...」、「`speclink delete change` SHALL be destructive...」。

- [x] 3.1 [P] 紅燈：在 tempdir + `git init` + bootstrap-init 環境（test 放在 `crates/provider-local/src/change_store.rs::tests` 內，沿用既有 `store.rs` inline test 慣例），撰寫測試：`create_change("foo", "spec-driven")` 寫入 row 後 `get_change("foo")` 回欄位完整 `ChangeRow`、`version=1`、`state="proposing"`；第二次 `create_change("foo", _)` 回 `ProviderError::ChangeDuplicateName { name: "foo" }`；`list_changes` 在 0 row / 3 row 場景下回正確排序（updated_at desc）；`delete_change` 刪 row 後 `get_change` 回 `ProviderError::ChangeNotFound`、且 `.speclink/changes/foo/` 目錄不存在。**契約**：紅燈對應四條 spec requirement。**驗證**：`cargo test -p speclink-provider-local change_store` 顯示 red。
- [x] 3.2 綠燈：實作 `crates/provider-local/src/change_store.rs`，提供 `LocalChangeStore` struct（沿用 bootstrap `LocalProjectStore` 「working_dir + state_root 兩個 PathBuf 欄位 + `new()` constructor」pattern），對 `ChangeStore` trait 的具體實作；trait method 走 §1.4 新增的 `StateDb` helper（不在這層暴露 `rusqlite::Connection`）；`create_change` 在 SQL 與 `fs::create_dir_all(.speclink/changes/<name>/)` 之間採 transactional pattern（先 `BEGIN IMMEDIATE`、SQL insert、再建目錄、commit；任一步失敗就 rollback + 嘗試 `remove_dir`）；`list_changes` 走 `SELECT ... ORDER BY updated_at DESC`；`delete_change` 反向（先 `DELETE`、再 `remove_dir_all`、commit）；用 `speclink_runtime::ARTIFACT_ROOT` 常量、不重新宣告。**契約**：3.1 全綠；mid-write fault 注入時 row 與目錄一致；error 走 `ProviderError`、不引入新 enum。**驗證**：`cargo test -p speclink-provider-local change_store` 全綠。
- [x] 3.3 重構：把 path helper（`changes_dir(state_root)`、`change_dir(state_root, name)`）抽出到 `crates/provider-local/src/paths.rs`；確認 Windows 路徑分隔符經 `PathBuf::join`；跑 `rust-skills:m02-resource` 確認 `rusqlite::Connection` 生命週期。**契約**：clippy pedantic 通過；無字串路徑拼接。**驗證**：`cargo clippy -p speclink-provider-local -- -D warnings -W clippy::pedantic` 通過。

## 4. LocalArtifactStore 實作（crates/provider-local）

對應 design「Etag = sha256(file bytes)」、「Atomic write 採 tempfile-in-same-dir + rename」、「`spec.list-in-change` 走 filesystem」與 spec requirement「Artifact filesystem path SHALL be derived from change name and kind」、「`speclink artifact read` SHALL return content and an Etag...」、「`speclink new artifact` SHALL enforce sha256-based optimistic concurrency」、「Artifact writes SHALL use a tempfile-then-rename atomic sequence」。

- [x] 4.1 [P] 紅燈（path resolution）：撰寫 unit test 覆蓋 spec table 四個 kind 的 path mapping，含 `kind=spec` 必要 `capability` 與其他 kind 忽略 `capability` 並回 `artifact.capability_ignored` warning 的行為；capability id 走 `validate_kebab_id` 拒絕 `User_Auth`。**契約**：對應 spec requirement「Artifact filesystem path SHALL be derived...」與「`--capability` ignored for non-spec kinds」。**驗證**：`cargo test -p speclink-provider-local artifact_path_resolution` red。
- [x] 4.2 [P] 紅燈（sha256 etag round-trip）：撰寫 unit test：tempdir 上 write_artifact(content=B0, expected=None) → read 回 (content=B0, etag=sha256(B0))；write_artifact(content=B1, expected=Some(etag_b0)) → read 回 (content=B1, etag=sha256(B1))。**契約**：對應 spec example「concurrency matrix」中第 1、4 列。**驗證**：`cargo test -p speclink-provider-local artifact_etag_roundtrip` red。
- [x] 4.3 [P] 紅燈（etag 並發矩陣完整覆蓋）：對 spec example「concurrency matrix」5 行各寫一支具名測試（`write_new_without_etag_ok` / `write_new_with_etag_not_found` / `write_existing_without_etag_conflict` / `write_existing_matching_etag_ok` / `write_existing_mismatching_etag_conflict`），每支驗證對應 `ProviderError` 變體與檔案內容是否變動。**契約**：對應 spec requirement「`speclink new artifact` SHALL enforce sha256-based optimistic concurrency」全覆蓋。**驗證**：`cargo test -p speclink-provider-local artifact_concurrency_matrix` red。
- [x] 4.4 [P] 紅燈（atomic rename + crash residue）：撰寫 unit test 注入「rename 前 panic」場景，驗證目標檔不存在或仍是 pre-write 版本、且 tempfile 不殘留在 parent dir；另一支測試 `kind=spec --capability user-auth` 寫到不存在的 `specs/user-auth/` 時 helper 會 `create_dir_all`。**契約**：對應 spec requirement「Artifact writes SHALL use a tempfile-then-rename atomic sequence」中兩個 scenario。**驗證**：`cargo test -p speclink-provider-local artifact_atomic_write` red。
- [x] 4.5 [P] 紅燈（list_spec_capabilities）：tempdir 模擬 `specs/rate-limiting/spec.md` + `specs/user-auth/spec.md` + `specs/incomplete/`（無 spec.md），`list_spec_capabilities("foo")` 回 `["rate-limiting", "user-auth"]`；空目錄回空 vec；不存在的 change 回 `ProviderError::ChangeNotFound`。**契約**：對應 spec requirement「`speclink list --specs --change <name>` SHALL enumerate spec capabilities from the filesystem」三個 scenario。**驗證**：`cargo test -p speclink-provider-local list_spec_capabilities` red。
- [x] 4.6 綠燈：實作 `crates/provider-local/src/artifact_store.rs`，含 `resolve_path`（kind + capability → PathBuf）、`compute_etag(bytes)`（回 `Etag("sha256:..." )`）、`atomic_write(path, bytes)`（用 `tempfile::NamedTempFile::new_in(parent)` + `persist(target)`）、`LocalArtifactStore` 對 `ArtifactStore` trait 的完整實作；artifact op 前先呼叫 `ChangeStore::get_change` 做 existence check。**契約**：4.1–4.5 全綠；library crate 無 `unwrap` / `expect` / `panic`。**驗證**：`cargo test -p speclink-provider-local artifact` 全綠。
- [x] 4.7 重構：把 sha256 hashing helper 抽到 `provider-local/src/hash.rs`；確認 `tempfile` crate version 與 workspace 對齊；跑 `rust-skills:m15-anti-pattern` 與 `rust-skills:coding-guidelines` 檢查。**契約**：clippy pedantic 通過；hashing helper 不洩漏部分 bytes 到 log。**驗證**：`cargo clippy -p speclink-provider-local -- -D warnings -W clippy::pedantic` 通過。

## 5. Runtime change_ops / artifact_ops 模組（crates/runtime）

對應 design「Runtime entry struct（新增至 `crates/runtime/src/change_ops.rs` + `crates/runtime/src/artifact_ops.rs`）」與 spec requirement 中 CLI 觀察的所有外部行為。**命名沿用 bootstrap 既有 `ops.rs::Operations<G>` 慣例；不擴 `Operations<G>` 本身。**

- [x] 5.1 [P] 紅燈：在 `crates/runtime/tests/change_ops.rs` 撰寫測試 driving `ChangeOperations<G>::create_change("foo")` / `list_changes()` / `show_change("foo")` / `delete_change("foo", confirm_name="foo")`，覆蓋成功與每一個 error 路徑（`change.not_found` / `change.duplicate_name` / `change.invalid_name` 三種觸發場景，包含 `delete` 缺 / 錯 confirm 兩支）。**契約**：對應 `specs/change-store/spec.md` 全部 scenarios。**驗證**：`cargo test -p speclink-runtime --test change_ops` red。
- [x] 5.2 [P] 紅燈：在 `crates/runtime/tests/artifact_ops.rs` 撰寫測試 driving `ArtifactOperations<G>::read_artifact` / `write_artifact` / `list_spec_capabilities`，覆蓋 spec example「concurrency matrix」5 行 + `kind=spec` 必要 `capability` + 非 spec kind 帶 capability 的 warning 傳遞 + 對不存在的 change 回 `change.not_found`。**契約**：對應 `specs/artifact-io/spec.md` 全部 scenarios。**驗證**：`cargo test -p speclink-runtime --test artifact_ops` red。
- [x] 5.3 綠燈：實作 `crates/runtime/src/change_ops.rs::ChangeOperations<G: GitProbe>` 與 `crates/runtime/src/artifact_ops.rs::ArtifactOperations<G: GitProbe>`（沿用 bootstrap `ops.rs::Operations<G>` 的 `new(git)` + `build_store(working_dir)` + `state_root(working_dir)` pattern；state_root 解析可直接重用 `speclink_runtime::resolve_state_root::<G>(&self.git, working_dir)` 既有 pub fn，免得重新發明）；`ChangeOperations::create_change` 在呼叫 `LocalChangeStore::create_change` 前先跑 `validate_kebab_id`；`ChangeOperations::delete_change` 校對 `confirm_name == name`；artifact 模組透過 `LocalChangeStore::get_change` 做 existence check 後再呼叫 `LocalArtifactStore`；error 走 `ProviderError → RuntimeError` 透過既有 `RuntimeError::Provider(#[from] ProviderError)` 自動轉換（bootstrap 既有 pattern）即可，**不**呼叫 bootstrap `ops.rs` 內的私有 `map_provider_error`；若需要把 ProviderError 變體一對一展開成 RuntimeError 對應變體，在每個 ops 模組內定義自己的 private `map_provider_error_xxx` 私有 helper（沿用 ops.rs 對等私有 fn pattern）；`runtime::lib.rs` 加 `pub mod change_ops; pub mod artifact_ops;` 與 `pub use change_ops::ChangeOperations; pub use artifact_ops::ArtifactOperations;`。**契約**：5.1–5.2 全綠；`RuntimeError` 新增 7 個變體並對 `ProviderError` 7 個新 variant exhaustive match。**驗證**：`cargo test -p speclink-runtime` 全綠。
- [x] 5.4 重構：確認每個 ops 模組內的 private `map_provider_error_*` 對 `ProviderError` 全部 variant（4 既有 + 7 新增 + `Internal`）exhaustive match；rustc 對 `#[non_exhaustive]` 或新增 variant 漏接會 warning，務必對齊；跑 `rust-skills:m13-domain-error`。**契約**：clippy pedantic 通過；rustc exhaustive match warning 0。**驗證**：`cargo clippy -p speclink-runtime -- -D warnings -W clippy::pedantic` 通過。

## 6. Exit code 對照表雙寫擴充（crates/runtime + crates/cli）

對應 design「JSON envelope shape」與 spec requirement「Error envelope SHALL preserve the standard shape from slice A onward」。**bootstrap 既有 `RuntimeError::exit_code()` 與 `output::error_code_to_exit` 兩處對照表並存，本 slice 不重構整併，只各自擴 match arm。**

- [x] 6.1 紅燈（runtime side）：在 `crates/runtime/src/error.rs::tests::exit_code_mapping_matches_spec_table`（既有測試）擴充 7 個 assertion，覆蓋新 error code 對 exit 表的對照（`change.not_found`→2 / `change.duplicate_name`→7 / `change.invalid_name`→2 / `artifact.kind_invalid`→2 / `artifact.capability_required`→2 / `artifact.not_found`→2 / `artifact.version_conflict`→7）。**契約**：對應 spec requirement「Error envelope SHALL preserve the standard shape from slice A onward」與 design failure modes 表。**驗證**：`cargo test -p speclink-runtime exit_code_mapping` red。
- [x] 6.2 綠燈（runtime side）：在 `crates/runtime/src/error.rs::RuntimeError::exit_code()` 既有 match 上加 7 個新 arm，對齊 6.1 assertion；同時擴 `code()` match arm 涵蓋 7 個新 variant。**契約**：6.1 全綠；bootstrap 既有 4 個 exit code 對照不 regression。**驗證**：`cargo test -p speclink-runtime` 全綠。
- [x] 6.3 紅燈（cli side）：在 `crates/cli/src/output.rs::tests::exit_code_mapping_matches_spec_table`（既有測試）擴充 7 個 assertion，內容與 6.1 對照表完全一致（兩處重複是 bootstrap 既有 duplication，slice A 不整併、只擴）。**契約**：兩處對照表 invariant 保持一致。**驗證**：`cargo test -p speclink-cli exit_code_mapping` red。
- [x] 6.4 綠燈（cli side）：在 `crates/cli/src/output.rs::error_code_to_exit` 既有 match 上加 7 個 entry；新增 7 個 success `data` shape struct（如 `NewChangeData` / `ListChangesData` / 等，全部 `#[serde(rename_all = "camelCase")]` 與 bootstrap 既有 envelope 序列化慣例對齊）；擴 `crates/cli/src/main.rs::hint_for` 對 7 個新 code 加對應 hint 文字（如 `change.duplicate_name` → "Pick a different name or delete the existing change first."）。**契約**：6.3 全綠；既有 bootstrap envelope 測試不 regression；`cargo test --workspace` 全綠。**驗證**：`cargo test --workspace` 全綠。

## 7. CLI subcommand wiring（crates/cli/src/commands/）

對應 design「CLI subcommand 對 Catalogue ID 映射」表 7 列。每支整合測試以 `assert_cmd` + `tempfile` 在 bootstrap-init 後的目錄上跑。

- [x] 7.1 [P] 紅燈：`crates/cli/tests/change_crud.rs::new_change_success` — `speclink new change foo` 後 `.speclink/changes/foo/` 存在、`change` 表有對應 row、`--json` envelope 含 `data.changeId` / `name` / `state="proposing"` / `version=1` / `schemaId="spec-driven"`。**契約**：對應 `change-store` spec 中 `speclink new change` SHALL create... 的 success scenario。**驗證**：`cargo test -p speclink-cli new_change_success` red。
- [x] 7.2 [P] 紅燈：`new_change_duplicate_name` — 同名第二次回 exit 7、error code `change.duplicate_name`、`data` 為 null；`new_change_invalid_name` — 對 spec table 中 4 個無效案例（`Foo` / `foo_bar` / `-foo` / 65-byte）各一支具名測試，每支驗證 exit 2 + `change.invalid_name`。**契約**：對應 spec requirement「Duplicate change name rejected」+「Invalid change name rejected」+「Boundary length names」。**驗證**：紅燈。
- [x] 7.3 [P] 紅燈：`crates/cli/tests/change_crud.rs::list_changes_empty` 與 `list_changes_sorted_by_updated_at_desc` — 0 row 場景與 3 row 場景驗證 envelope `data.changes` 排序。**契約**：對應 spec requirement「Empty change table」+「Multiple changes ordered by updated_at descending」。**驗證**：紅燈。
- [x] 7.4 [P] 紅燈：`crates/cli/tests/change_crud.rs::show_change_with_artifacts` / `show_change_empty` / `show_change_not_found` 三支具名測試，分別對應 spec 三個 scenario；`show_change_with_artifacts` 用 fixture 預植 `proposal.md` + `design.md` + `specs/user-auth/spec.md`、驗證 `data.artifacts` 陣列形狀。**契約**：對應 `speclink show change` SHALL emit... 三個 scenario。**驗證**：紅燈。
- [x] 7.5 [P] 紅燈：`crates/cli/tests/change_crud.rs::delete_change_success` / `delete_change_missing_confirm` / `delete_change_mismatched_confirm` / `delete_change_not_found` 四支測試，分別驗證 exit code / error code / 副作用（row 與目錄）。**契約**：對應 spec 四個 destructive scenario。**驗證**：紅燈。
- [x] 7.6 [P] 紅燈：`crates/cli/tests/artifact_io.rs::new_artifact_kind_invalid` / `new_artifact_capability_required` / `new_artifact_capability_ignored_for_proposal` 三支具名測試，後一支驗證 envelope `warnings` 陣列含 `{code:"artifact.capability_ignored", ...}`。**契約**：對應 spec 三個 scenario。**驗證**：紅燈。
- [x] 7.7 [P] 紅燈：`crates/cli/tests/artifact_io.rs::artifact_read_success` / `artifact_read_not_found` / `artifact_read_change_not_found` — 三支具名測試覆蓋讀取路徑與 error precedence（change 不存在優先於 artifact 不存在）。**契約**：對應 spec 三個 scenario。**驗證**：紅燈。
- [x] 7.8 [P] 紅燈：`crates/cli/tests/etag_concurrency.rs` 對 spec example「concurrency matrix」5 行各寫一支具名整合測試（`write_new_without_etag` / `write_new_with_etag_rejected` / `write_existing_without_etag_conflict` / `write_existing_matching_etag` / `write_existing_mismatching_etag`），驗證 stdin 從 `--stdin` 進入、`--expected-etag` 旗標被正確解析、exit code 與 error code 對應、檔案 byte 內容是否變動。**契約**：對應 spec requirement「`speclink new artifact` SHALL enforce sha256-based optimistic concurrency」example matrix。**驗證**：紅燈。
- [x] 7.9 [P] 紅燈：`crates/cli/tests/artifact_io.rs::list_specs_empty` / `list_specs_sorted` / `list_specs_ignores_incomplete` 三支測試對應 spec 三個 scenario。**契約**：對應 spec requirement「`speclink list --specs --change <name>` SHALL enumerate...」。**驗證**：紅燈。
- [x] 7.10 綠燈：在 `crates/cli/src/commands/` 新增 7 支 module 檔，沿用 bootstrap 既有「一 subcommand 一 file」+ 每檔 `pub async fn run(...) -> Result<serde_json::Value, RuntimeError>` 慣例：`new_change.rs` / `new_artifact.rs` / `list_changes.rs` / `list_specs.rs` / `show_change.rs` / `delete_change.rs` / `artifact_read.rs`；在 `crates/cli/src/commands/mod.rs` 加 7 個 `pub mod` 宣告；在 `crates/cli/src/main.rs::Commands` enum 加 7 個新 variant（`NewChange` / `NewArtifact` / `List { --changes / --specs / --change <name> }` / `ShowChange` / `DeleteChange` / `ArtifactRead`）並擴 `main()` 的 match arm；clap derive 結構覆蓋所有旗標；保持 `RuntimeError` map 到 `Envelope::Err` 流程不變（bootstrap 既有 main.rs flow 不改）。**契約**：7.1–7.9 全綠；`speclink --help` 顯示 7 個新 subcommand 形式；bootstrap 既有 `crates/cli/tests/cli.rs` 不 regression。**驗證**：`cargo test --workspace` 全綠且 `assert_cmd::Command::new("speclink").arg("--help")` snapshot 通過。
- [x] 7.11 重構：跑 `rust-skills:domain-cli`（clap derive 排版 / subcommand 命名 / help 文案）+ `rust-skills:m15-anti-pattern`；確認 `--json` 模式下 stderr 不含 JSON、stdout 為單一 JSON object 結尾換行。**契約**：clippy pedantic 通過；snapshot 穩定。**驗證**：`cargo clippy -p speclink-cli -- -D warnings -W clippy::pedantic` 通過。

## 8. JSON envelope snapshot（crates/cli/tests/）

對應 design「Acceptance criteria」第 12 列「`--json` envelope 對 7 個 op 各取一個 snapshot」。沿用 bootstrap 既有 `crates/cli/src/output.rs::tests::error_envelope_json_snapshot_with_fixed_request_id` 既有 `insta` 用法。

- [x] 8.1 撰寫 `crates/cli/tests/snapshots.rs`，對 7 個 op 各取一個 success envelope snapshot + 至少一個 error envelope snapshot（用 `insta::assert_snapshot!`，redact `requestId` 與 `createdAt` 為固定字串）。**契約**：snapshot 對 envelope 所有 key 完整覆蓋；redact pattern 不影響 contract assertion。**驗證**：`cargo insta test -p speclink-cli` 通過、`cargo insta review` 確認 diff 為空。

## 9. Cross-platform 與 release readiness

對應 design「Acceptance criteria」第 13 列「Etag 對同 bytes 跨平台一致」與 bootstrap slice 既有 CI 矩陣。

- [x] 9.1 macOS（本機）跑 `cargo test --workspace`、`cargo fmt --check`、`cargo clippy --workspace -- -D warnings` 全綠。**契約**：本機驗證所有新增 test。**驗證**：三個指令 exit 0。
- [x] 9.2 Linux + Windows CI（沿用 bootstrap matrix）跑相同檢查；於 `crates/cli/tests/etag_concurrency.rs` 加一支跨平台 fixture，寫入 byte `b"hello\n"` 並 assert etag 字串等於 `sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03`（POSIX/Windows 一致）。**契約**：CI matrix 上 Linux + Windows 工作項目綠。**驗證**：CI run log 顯示三個指令 exit 0；etag 跨平台 assertion 通過。
- [x] 9.3 README 追加 `## Walking skeleton 2 — change & artifact` 章節，列出 7 個 op 與一段 fresh init → create change → write proposal → read proposal → list changes 的 demo 範例。**契約**：README 步驟在 tempdir 可實機跑通。**驗證**：人工跑 README 步驟，最後一步 `artifact read proposal` 印出寫入的 markdown 與 sha256 etag。
- [x] 9.4 更新 `doc/speclink-design.md` §18.1 MVP 表中對應 op 標記為 done（`change.create` / `change.list` / `change.show` / `change.delete` / `artifact.read` / `artifact.write` / `spec.list-in-change`）；同步 `doc/protocol/operations.md` 中相應 7 個 op 的「MVP 狀態」欄位。**契約**：design 文件與 operations.md 與本 slice 實際完成的 op 對齊；不越界標記其他 slice 的 op。**驗證**：`grep -n 'change.create' doc/speclink-design.md doc/protocol/operations.md` 確認狀態欄位更新。
