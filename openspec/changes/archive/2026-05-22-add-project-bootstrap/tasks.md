## 1. Workspace bootstrap（建立 Cargo workspace 與 4 crate skeleton）

對應 design 章節：「Crate 邊界：cli ↔ runtime ↔ provider ↔ provider-local」。

- [x] 1.1 建立 workspace root `Cargo.toml`，列出 4 個 member（`crates/cli`、`crates/runtime`、`crates/provider`、`crates/provider-local`），鎖定 Rust edition 2024 與 workspace 共享 dependency。**契約**：`cargo metadata --format-version 1` 列出 4 個 workspace member，順序與 design 中「Crate 邊界：cli ↔ runtime ↔ provider ↔ provider-local」一致。**驗證**：`cargo metadata` 解析後比對 `packages[].name` 集合等於 `{speclink-cli, speclink-runtime, speclink-provider, speclink-provider-local}`。
- [x] 1.2 [P] 為 `crates/provider` 建立 `Cargo.toml` 與空 `lib.rs`，宣告 `async-trait`、`serde`、`thiserror` 三個 dependency。**契約**：crate 可被 workspace build，無 warning。**驗證**：`cargo build -p speclink-provider` exit 0。
- [x] 1.3 [P] 為 `crates/provider-local` 建立 `Cargo.toml` 與空 `lib.rs`，宣告對 `speclink-provider`、`rusqlite`（with feature `bundled`）、`serde_yaml`、`uuid` 的 dependency。**契約**：crate 編譯通過。**驗證**：`cargo build -p speclink-provider-local` exit 0。
- [x] 1.4 [P] 為 `crates/runtime` 建立 `Cargo.toml` 與空 `lib.rs`，宣告對 `speclink-provider`、`speclink-provider-local`、`thiserror`、`uuid`、`tokio` 的 dependency。**契約**：crate 編譯通過。**驗證**：`cargo build -p speclink-runtime` exit 0。
- [x] 1.5 [P] 為 `crates/cli` 建立 `Cargo.toml` 與 `src/main.rs`（暫時空 main），宣告對 `speclink-runtime`、`clap` v4 derive、`anyhow`、`serde_json` 的 dependency。**契約**：binary 可 build。**驗證**：`cargo build -p speclink-cli` exit 0，產出 binary `target/debug/speclink`。
- [x] 1.6 整 workspace 編譯與 lint 全綠。**契約**：`cargo build --workspace`、`cargo fmt --check`、`cargo clippy --workspace -- -D warnings` 全部通過。**驗證**：三個指令 exit 0。

## 2. Provider trait 與共用型別（crates/provider）

對應 design 章節：「Provider trait skeleton 先抽，LocalProvider 為唯一具體實作」。

- [x] 2.1 撰寫 unit test（red）：`LinkYaml` v1 schema serde round-trip、`ProjectStatus` 含 6 個欄位（project_id、provider、artifact_root、state_root、git_head、requires_git）的 serde 對齊；測試確認 trait `ProjectStore` 內 6 個 method 簽名（`init`、`status`、`link`、`unlink`、`get_link`、`save_link`）存在。**契約**：對應 spec requirement「`link.yaml` MUST follow a versioned YAML schema」與「`speclink status` reports project metadata for an initialized project」。**驗證**：`cargo test -p speclink-provider -- --include-ignored` 顯示新測試紅燈（assertion failure 或 unresolved symbol）。
- [x] 2.2 實作（green）`ProjectStore` trait 與 `LinkYaml`、`ProjectInfo`、`ProjectStatus`、`InitOptions`、`ProviderError`（thiserror）型別於 `crates/provider/src/lib.rs` 與 `crates/provider/src/types.rs`。**契約**：6 個 trait method 簽名與 design「Interface / data shape」表一致；`LinkYaml` 含 6 個欄位且 `version` 預設 `1`；`ProviderError` enum 涵蓋 4 個 declared error code（`project.requires_git`、`project.already_initialized`、`project.not_initialized`、`project.link_target_not_found`）。**驗證**：`cargo test -p speclink-provider` 全綠。
- [x] 2.3 重構並通過 `rust-skills:m15-anti-pattern` 與 `rust-skills:coding-guidelines` 檢查；公開 API 加上 doc comment（繁中正文，範例 code 英文）。**契約**：library crate 內無 `unwrap()` / `expect()`、無 `panic!()`、無 `dyn ProjectStore` 多重 box。**驗證**：`cargo clippy -p speclink-provider -- -D warnings -W clippy::pedantic` 通過且 `grep -E '\.unwrap\(\)|\.expect\(' crates/provider/src/**/*.rs` 為空（生產 code 範圍）。

## 3. State.db migration runner（crates/provider-local）

對應 design 章節：「State.db schema 初始化與 migration runner 介面」。

- [x] 3.1 [P] 撰寫 unit test（red）：`StateDb::open` 在空 path 建立新檔；`StateDb::migrate(1)` 在新 db 建立 `_migrations` 與 `project` 表；idempotent 行為（連續呼叫第二次 no-op）；WAL journal mode 確認。**契約**：對應 spec requirement「`state.db` MUST be initialized at schema version 1 with the prescribed tables」。**驗證**：`cargo test -p speclink-provider-local migration` 顯示 red。
- [x] 3.2 實作（green）`crates/provider-local/src/state_db.rs`，含 `MIGRATIONS: &[&str]` 常量陣列（v1 schema 兩條 SQL：`CREATE TABLE _migrations` 與 `CREATE TABLE project`），`StateDb::open` 開檔並設 `PRAGMA journal_mode=WAL`、`StateDb::migrate(target)` 跑 migration runner 並寫入 `_migrations` row。**契約**：開啟後 `_migrations` 含 version=1 row；schema 欄位與型別精確等於 spec requirement 表格。**驗證**：`cargo test -p speclink-provider-local migration` 全綠，並含一支測試以 raw SQL `pragma_table_info('project')` 確認欄位順序與 NOT NULL 約束。
- [x] 3.3 重構：抽出 migration error 路徑型別、加 doc comment、移除任何 `unwrap()`。**契約**：library crate clippy pedantic 通過；migration 失敗時不留下 partial state.db（測試覆蓋失敗中斷後再 retry 行為）。**驗證**：`cargo clippy -p speclink-provider-local -- -D warnings` 通過。

## 4. LocalProvider link.yaml I/O 與 ProjectStore 實作（crates/provider-local）

對應 design 章節：「Provider trait skeleton 先抽，LocalProvider 為唯一具體實作」、「Two-root storage layout: `.speclink/` artifact root + `.git/speclink/` state root」。

- [x] 4.1 [P] 撰寫 unit test（red）：`link_yaml::write` 寫出含 6 欄位的 YAML、`link_yaml::read` 解析 v1 schema、`LocalProjectStore::get_link`/`save_link` 對應一個 tempdir 內的 `.speclink/link.yaml`。**契約**：對應 spec requirement「Artifact root MUST be located at `.speclink/` in the working tree」與「`link.yaml` MUST follow a versioned YAML schema」。**驗證**：`cargo test -p speclink-provider-local link_yaml` 顯示 red。
- [x] 4.2 實作（green）`crates/provider-local/src/link_yaml.rs`（serde_yaml read/write）與 `crates/provider-local/src/store.rs`（`LocalProjectStore` 實作 `ProjectStore` trait 的所有 6 個 method），其中 `init` 呼叫 `StateDb::migrate(1)` 並插入 project row、`link` 在 state.db 查 project_id 後寫 link.yaml、`unlink` 刪 link.yaml 但不動 state.db。**契約**：所有 spec requirement 中 `speclink unlink` / `speclink link <project_id>` / `speclink link` MUST reject unknown 對應的內部行為通過 unit test。**驗證**：`cargo test -p speclink-provider-local store` 全綠，並含「unlink keeps state.db」與「link to unknown project_id 回 ProviderError::LinkTargetNotFound」兩支測試。
- [x] 4.3 重構：拆出 path helper、確保 LocalProjectStore 無 hidden global state、序列化 YAML 時鎖定 key 順序以利 snapshot 比對。**契約**：clippy pedantic 通過；`cargo insta review` 中 link.yaml 內容快照穩定。**驗證**：`cargo clippy -p speclink-provider-local -- -D warnings` 通過、`cargo insta test -p speclink-provider-local` 通過。

## 5. Runtime path resolution（crates/runtime）

對應 design 章節：「State root 路徑統一走 `git rev-parse --git-common-dir`」。

- [x] 5.1 [P] 撰寫 unit test（red）覆蓋三條 spec requirement：「State root MUST be located under the git common directory with namespace `speclink/`」、「State root MUST resolve to the main git directory in a linked worktree」、「Path resolution algorithm MUST shell out to `git rev-parse --git-common-dir`」。`paths::resolve_state_root(working_dir)` 在 tempdir + `git init` 場景回 `<tempdir>/.git/speclink/`；在 `git worktree add` 出來的 linked worktree 場景回 main repo 的 `<main>/.git/speclink/`；當 `git` 不在 PATH 時回 `RuntimeError::RequiresGit`。**契約**：三條 requirement 對應的 runtime 行為紅燈。**驗證**：`cargo test -p speclink-runtime paths` 顯示 red。
- [x] 5.2 實作（green）`crates/runtime/src/paths.rs` 與 `crates/runtime/src/git.rs`，後者封裝 `Command::new("git").args(["rev-parse", "--git-common-dir"])`，前者組合 state root 路徑。**契約**：linked worktree 測試（5.1 中第二支）通過；不在 PATH 時走 `RequiresGit` 路徑。**驗證**：`cargo test -p speclink-runtime paths` 全綠，含 `paths_resolve_in_linked_worktree_points_to_main_git_dir` 一支具名測試。
- [x] 5.3 重構：把 git CLI 呼叫包成可 mock 的小 trait（避免測試呼叫真 git），確保 Windows 路徑分隔符經 `PathBuf` 處理。**契約**：library crate 內無 `String` 路徑拼接；Windows / macOS / Linux 三平台路徑均經 `Path::join`。**驗證**：`cargo clippy -p speclink-runtime -- -D warnings` 通過。

## 6. Runtime bootstrap orchestration（crates/runtime）

對應 design 章節：「強制 git working dir，non-git 拒絕 init」、「Init MUST commit artifact and state changes only after every prepare step succeeds」（design 章節「Schema seed 過程被中斷導致 partial init」的 mitigation）、「`.gitignore` 政策：單行 `.speclink/link.yaml`」。

- [x] 6.1 紅燈：`speclink init` initializes a SpecLink project in a git working tree — 撰寫測試 driving `bootstrap::init` 在 tempdir + `git init` 後成功建立 `.speclink/link.yaml`、`.speclink/schemas/`、`.git/speclink/state.db`、`.gitignore`。**契約**：spec requirement「`speclink init` initializes a SpecLink project in a git working tree」對應行為紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_success` 顯示 red。
- [x] 6.2 紅燈：`speclink init` MUST reject non-git working directories — 撰寫測試使 tempdir 不執行 `git init`，呼叫 `bootstrap::init` 必須回 `RuntimeError::RequiresGit` 且不建立任何檔案；此測試同時覆蓋 design「failure modes」中 `project.requires_git` 路徑。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_rejects_non_git` 顯示 red。
- [x] 6.3 紅燈：`speclink init` MUST refuse re-initialization without `--force` — 撰寫測試先呼叫一次 `bootstrap::init` 成功後，第二次呼叫無 `--force` 必須回 `RuntimeError::AlreadyInitialized` 並保留既有 link.yaml mtime。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_conflict_without_force` 顯示 red。
- [x] 6.4 紅燈：`speclink init --force` MUST re-init while preserving `state.db` — 撰寫測試比對 force 前後 state.db 的 SHA-256 與位元組長度應相同，而 link.yaml 的 instance_id 必須變更、created_at 必須更新。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_force_preserves_state_db` 顯示 red。
- [x] 6.5 紅燈：`.gitignore` policy MUST be a single line for `.speclink/link.yaml` — 撰寫三支測試覆蓋 missing / append / idempotent force 三個 scenario，每支驗證 `.gitignore` 內 `.speclink/link.yaml` 行存在且不重複。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime gitignore_policy` 三支測試顯示 red。
- [x] 6.6 紅燈：Init MUST commit artifact and state changes only after every prepare step succeeds — 撰寫測試注入 mid-init 失敗（mock state.db migration 失敗），驗證 working tree 與 state root 不留 partial：`.speclink/link.yaml` 不存在、`.speclink/schemas/` 不存在、state root 內無 state.db。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_partial_failure_cleanup` 顯示 red。
- [x] 6.7 綠燈：實作 `crates/runtime/src/bootstrap.rs` 與 `crates/runtime/src/error.rs`，`bootstrap::init` 採 prepare-then-commit 順序（先在 tempdir 拼 link.yaml、跑 state.db migration、seed schema，全部 OK 後 atomic rename 到目標位置）；non-git 走 `RuntimeError::RequiresGit`；既有 link.yaml 走 `AlreadyInitialized`（除非 `--force`）；`.gitignore` append 採 line-based exact match。**契約**：6.1–6.6 所有測試綠燈。**驗證**：`cargo test -p speclink-runtime bootstrap` 全綠。
- [x] 6.8 重構：抽出 `.gitignore` 操作為 `gitignore::append_if_missing(path, line)` 純函式；prepare-then-commit 用 RAII guard 確保失敗 cleanup 一定執行。**契約**：clippy pedantic 通過；RAII guard 在 Drop 中 best-effort cleanup tempdir。**驗證**：`cargo clippy -p speclink-runtime -- -D warnings` 通過。

## 7. Runtime status / link / unlink（crates/runtime）

對應 spec requirement：`speclink status` reports project metadata、`speclink status` reports `project.not_initialized`、`speclink link <project_id>` binds、`speclink link` MUST reject unknown、`speclink unlink` removes binding metadata but preserves state and artifacts。對應 design 章節：「Bootstrap CLI 拆四個 subcommand：init / status / link / unlink」。

- [x] 7.1 紅燈：`speclink status` reports project metadata for an initialized project — 撰寫測試，已 init 專案呼叫 `runtime::status` 回 ProjectStatus 含 project_id、provider、artifact_root、state_root、git_head（任意 sha）、requires_git 共 6 欄位。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime status_returns_metadata` 顯示 red。
- [x] 7.2 紅燈：`speclink status` reports `project.not_initialized` when no project exists — 撰寫測試 tempdir + `git init` 但無 `.speclink/link.yaml`，`runtime::status` 必須回 `RuntimeError::NotInitialized`。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime status_without_init` 顯示 red。
- [x] 7.3 紅燈：`speclink link <project_id>` binds the working directory to an existing project row — 撰寫測試手動插入 state.db row 模擬 backup 還原，`runtime::link("<id>")` 必須寫出 link.yaml 內 project_id 等於該 id，且不修改 state.db。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime link_known_project` 顯示 red。
- [x] 7.4 紅燈：`speclink link` MUST reject unknown `project_id` — 撰寫測試 state.db 不含目標 row，`runtime::link("<unknown>")` 必須回 `RuntimeError::LinkTargetNotFound` 且不建立 link.yaml。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime link_unknown_project` 顯示 red。
- [x] 7.5 紅燈：`speclink unlink` removes binding metadata but preserves state and artifacts — 撰寫測試：已 init 專案呼叫 `runtime::unlink` 後，`.speclink/link.yaml` 不存在；`.speclink/schemas/` 內容、`<state-root>/state.db` 內容 SHA-256 均不變。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime unlink_preserves_state_and_schemas` 顯示 red。
- [x] 7.6 綠燈：實作 `crates/runtime/src/status.rs`、`runtime::link::run`、`runtime::unlink::run`，內部委派給 `ProjectStore` trait。**契約**：7.1–7.5 全綠。**驗證**：`cargo test -p speclink-runtime status_link_unlink` 全綠。
- [x] 7.7 重構：把 git_head 取得封裝進 `git::head_sha`；status JSON serialization 鎖 key 順序。**契約**：clippy 通過。**驗證**：`cargo clippy -p speclink-runtime -- -D warnings` 通過。

## 8. CLI JSON envelope（crates/cli/src/output.rs）

對應 spec requirement：「SpecLink CLI commands emit a stable JSON envelope」、「SpecLink CLI exit codes follow a fixed mapping」。對應 design 章節：「CLI JSON envelope 標準化」。

- [x] 8.1 [P] 撰寫 unit test（red）覆蓋兩條 spec requirement：「SpecLink CLI commands emit a stable JSON envelope」與「SpecLink CLI exit codes follow a fixed mapping」。`output::success(data, warnings)` 序列化出 `{ok:true, data, warnings, requestId:<uuid4>}`；`output::error(code, message, hint, retryable)` 序列化出 `{ok:false, error:{code,message,hint,retryable,retry_after_ms}, requestId:<uuid4>}`；requestId pattern 為 UUID v4；exit code 對照表（`project.requires_git`→2、`project.already_initialized`→7、`project.link_target_not_found`→2、`project.not_initialized`→2）對應 declared error code。**契約**：envelope 兩種形狀通過 JSON Schema 驗證；exit code 表格與 spec 一致。**驗證**：`cargo test -p speclink-cli envelope` red。
- [x] 8.2 實作（green）`crates/cli/src/output.rs`：`Envelope` enum（`Ok { data, warnings }`、`Err { error }`）、`Envelope::write_stdout(self, writer)`、`error_code_to_exit(code) -> i32` 對照函式、UUID v4 requestId 產生器。**契約**：8.1 兩支測試與對照表測試綠燈。**驗證**：`cargo test -p speclink-cli envelope` 全綠。
- [x] 8.3 重構：把 envelope 序列化 snapshot 化（`insta`）；確保 stderr 在 `--json` 模式不含 JSON。**契約**：snapshot 穩定；envelope 不含未來欄位之外的多餘 key。**驗證**：`cargo insta test -p speclink-cli envelope` 通過。

## 9. CLI subcommand wiring 與整合測試（crates/cli）

對應 spec requirement（全部 11 個 `project-bootstrap` requirement）與 spec requirement（全部 `local-storage-layout` 中與 CLI 直接觀察的 requirement）。對應 design 章節：「Bootstrap CLI 拆四個 subcommand：init / status / link / unlink」、「CLI JSON envelope 標準化」。

- [x] 9.1 撰寫整合測試（red）以 `assert_cmd` + `tempfile` 端到端鎖定 design「Implementation Contract」中所列的 observable behavior 與 failure modes：覆蓋 design「Acceptance criteria」表中 15 個測試 case，包含：fresh git init、non-git rejection、re-init conflict、`--force` 保留 state.db、status 欄位、status 未 init、linked worktree 解析、`.gitignore` append 與 idempotent、unlink 保留 state.db / schemas、link known / unknown、envelope success / error shape、state.db v1 schema 直接 SQL 查詢驗證。每支測試名稱對應 design 表中第 1–15 列。**契約**：所有 15 支命名測試 red。**驗證**：`cargo test -p speclink-cli --test '*'` 全部 red。
- [x] 9.2 實作（green）`crates/cli/src/commands/{init,status,link,unlink}.rs` 四個 clap subcommand 與 main wiring；每個 subcommand 接 `--json` flag、呼叫 runtime、把 `RuntimeError` map 到 `Envelope::Err` 並用 `error_code_to_exit` 設 process exit。**契約**：15 支整合測試全綠；非 `--json` 模式輸出 human-readable 並把 trace 走 stderr。**驗證**：`cargo test -p speclink-cli --test '*'` 全部綠燈，且 `speclink init --help` 顯示完整 help 文案。
- [x] 9.3 重構：snapshot 整合測試的 `--json` stdout（`insta`）；確認 stderr 在 `--json` 模式無 JSON 輸出（grep assertion）；觸發 `rust-skills:domain-cli` 與 `rust-skills:m15-anti-pattern` 檢查。**契約**：clippy pedantic 通過；JSON snapshot 與 spec 範例對齊。**驗證**：`cargo clippy -p speclink-cli -- -D warnings` 通過、`cargo insta review` 確認所有 snapshot 對齊預期。

## 10. Doctor finding code 字串常量註冊（crates/runtime/src/error.rs）

對應 design 章節：「Doctor finding code 字串常量先註冊，邏輯不實作」。

- [x] 10.1 [P] 撰寫 unit test：`runtime::error::finding_codes::*` 四個 `const &str` 暴露且值精確等於 `doctor.project.requires_git`、`doctor.state.db_missing`、`doctor.state.db_corrupted`、`doctor.state.db_schema_invalid`。**契約**：常量穩定，不可被後續 change 重命名。**驗證**：`cargo test -p speclink-runtime finding_codes` 綠燈，並以 snapshot（`insta`）固定模組內常量列表。
- [x] 10.2 加 doc comment 明示「reserved code，actual check implemented in `add-doctor` change」；於 `state.db_missing` 旁註明 `auto_fixable=true` 預期。**契約**：rustdoc 渲染後可見保留說明。**驗證**：`cargo doc -p speclink-runtime --no-deps` 無 warning。

## 11. Acceptance、cross-platform 與 release readiness

對應 design「Acceptance criteria」整章與「Risks / Trade-offs」中 Git CLI / linked worktree / SQLite WAL / partial init 對應的測試行。

- [x] 11.1 macOS（本機）跑全 workspace 測試與 lint。**契約**：`cargo test --workspace`、`cargo fmt --check`、`cargo clippy --workspace -- -D warnings` 全綠。**驗證**：三個指令本機 exit 0。
- [x] 11.2 Linux（GitHub Actions stable Rust）跑相同檢查。**契約**：CI matrix 上 Linux 工作項目綠。**驗證**：CI run log 顯示三個指令 exit 0；linked worktree 測試與 `.gitignore` append 測試於 Linux 通過。
- [x] 11.3 Windows（GitHub Actions stable Rust）跑相同檢查。**契約**：CI matrix 上 Windows 工作項目綠；路徑分隔符差異無 regression。**驗證**：CI run log 顯示三個指令 exit 0；測試 `status_after_init_returns_expected_fields` 中 `artifact_root` 仍為 `.speclink`（POSIX 樣式）、`state_root` 仍為 `.git/speclink`。
- [x] 11.4 對 `--json` 輸出建立 `insta` snapshot，覆蓋 init 成功、init non-git 失敗、status 成功、link 失敗共四種代表性 envelope。**契約**：snapshot 內容包含所有 spec requirement「SpecLink CLI commands emit a stable JSON envelope」要求的 key。**驗證**：`cargo insta test --workspace` 通過、snapshot diff 為空。
- [x] 11.5 README 加上 `## Quick start`（fresh git repo → `speclink init` → `speclink status` 三步驟）；更新 `doc/speclink-design.md` §18.1 中本 change 涵蓋的 MVP 項目標記為 done，並對齊 design「Scope boundaries」中 in-scope 項目（init / status / link / unlink）與 out-of-scope 項目（change CRUD / artifact / apply / review / archive / discuss / skill / doctor 完整實作 / restore / HttpProvider）。**契約**：README quick start 步驟可實機跑通；design doc MVP 表反映現況；scope boundaries 章節未被本 change 越界擴充。**驗證**：手動跑 README 步驟在 tempdir 通過；`grep -n 'project-bootstrap' doc/speclink-design.md` 顯示對應行已更新；`grep -n 'Out of scope' openspec/changes/add-project-bootstrap/design.md` 列表內項目未被 7.x / 8.x / 9.x 任務違反。

## 12. Worktree init 與 state_root 顯示修正（acceptance feedback）

對應 design 章節：「Worktree init 沿用主 repo 的 project_id」、「`state_root` 路徑顯示：strip prefix 失敗時走 canonical absolute path」。對應新增 spec requirement：「`speclink init` inside a linked worktree MUST share the main repo's `project_id`」、「`state_root` field in command output MUST be a clean path with no leading double slash」。

- [x] 12.1 紅燈：`speclink init` inside a linked worktree MUST share the main repo's `project_id` — 撰寫測試在 main 內 init 後 `git worktree add` 建 worktree，在 worktree 內呼叫 `Bootstrap::init`，驗證 `<wt>/.speclink/link.yaml` 內 `project_id` 等於 main 的 project_id；驗證 main `<git-common-dir>/speclink/state.db` 內 `project` row 仍恰為 1 筆；驗證 wt 的 `instance_id` 與 main 不同。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime bootstrap_init_in_linked_worktree_shares_project_id` 顯示 red。
- [x] 12.2 紅燈：`state_root` field in command output MUST be a clean path with no leading double slash — 撰寫測試在 worktree 內呼叫 `Operations::status` 與 `Bootstrap::init`，驗證回傳的 `state_root` 字串不以 `//` 開頭。**契約**：對應 spec requirement 紅燈。**驗證**：`cargo test -p speclink-runtime state_root_display_has_no_leading_double_slash` 顯示 red。
- [x] 12.3 綠燈：在 `Bootstrap::init` 偵測 `preexisting_state_db = true` 且 worktree 無既存 link.yaml 時，從 state.db 讀取唯一 project row 的 id 沿用為 `project_id`，不再 staging insert；同時把 `relative_state_root_display` 對 strip_prefix 失敗 case 改回傳 canonical absolute path（不再用 `components().join("/")` 拼）。**契約**：12.1、12.2 全綠；既有 8 支 bootstrap test 與 5 支 ops test 仍綠。**驗證**：`cargo test --workspace --all-targets` 全綠。
- [x] 12.4 重構：把 `relative_state_root_display` 從 `bootstrap.rs` 與 `ops.rs` 兩處複製貼上的版本抽出為 `paths::display_state_root(working_dir, state_root)` 單一實作；確保 worktree 共享行為的單元測試直接針對該 helper。**契約**：兩處 callsite 不再有重複實作；clippy clean。**驗證**：`cargo clippy --workspace --all-targets -- -D warnings` 通過、`grep -n "fn relative_state_root_display" crates/runtime/src` 只顯示一處（在 `paths.rs`）。
