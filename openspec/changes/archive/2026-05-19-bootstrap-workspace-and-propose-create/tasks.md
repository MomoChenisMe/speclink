## 1. Cargo workspace 採 4 crate 結構並釘死依賴方向

- [x] 1.1 建立 workspace 根目錄 `Cargo.toml`（含 `[workspace] members = ["crates/cli", "crates/runtime", "crates/provider", "crates/provider-local"]`）與 `rust-toolchain.toml`（釘 stable channel + edition 2024）。驗證：`cargo metadata --format-version=1` 列出 4 個 workspace member 且無循環依賴錯誤
- [x] 1.2 建立 4 個 crate 的最小骨架（每個含 `Cargo.toml` 與空的 `src/lib.rs` 或 `src/main.rs`），釘定 cli → runtime + provider + provider-local、runtime → provider、provider-local → provider 的依賴方向。驗證：`cargo build --workspace` 與 `cargo fmt --check` 通過
- [x] 1.3 在 workspace 根目錄加 `.gitignore`（忽略 `target/`、`.speclink/`、`.idea/`、`.vscode/`），並提供 LICENSE 與最小 `README.md` 描述。驗證：`git status` 不出現任何 build artifact，`cargo doc --workspace --no-deps` 完成

## 2. Provider crate — 共用資料模型

- [x] 2.1 撰寫 `crates/provider/src/model.rs` 的 serde round-trip 單元測試，覆蓋 `Project`、`ProjectId`、`ChangeId`、`Change`、`NewChange`、`Artifact`、`NewArtifact`、`State` 各一個 serialize → JSON → deserialize 等價案例。觀察行為：模型可被 serde 序列化為 camelCase JSON 並還原。驗證：`cargo test -p provider model::tests` 預期紅燈（型別尚未存在）
- [x] 2.2 觸發 `rust-skills:m05-type-driven` 取得 newtype + PhantomData 建議，記錄是否套用 `ChangeId` / `ProjectId` 為 newtype 以避免裸 String 跨界
- [x] 2.3 實作 `crates/provider/src/model.rs`，七個型別加上必要的 `AsRef<str>`、`From<String>`、`Display` 介面，並提供 `///` 繁體中文 doc comment。觀察行為：上述測試從紅燈轉為綠燈。驗證：2.1 測試全綠 + `cargo clippy -p provider -- -D warnings` 通過

## 3. Error 架構 lib 用 thiserror、cli 用 anyhow、跨層用點分隔 error code（provider 層）

- [x] 3.1 觸發 `rust-skills:m13-domain-error` 與 `rust-skills:m06-error-handling` 取得 error enum 設計建議，決定 `ProviderError` 各 variant 是否攜帶 context（如 `change_id`）
- [x] 3.2 撰寫 `crates/provider/src/error.rs` 測試：`ProviderError` variant 列舉與 `error_code() -> &'static str` 對應 — `NotAuthenticated → "provider.not_authenticated"`、`Unavailable → "provider.unavailable"`、`ChangeAlreadyExists → "change.already_exists"`、`ChangeNotFound → "change.not_found"`、`Internal → "internal.error"`。觀察行為：每個 variant 對應 Error code naming convention 規定的點分隔字串。驗證：`cargo test -p provider error::tests` 紅燈
- [x] 3.3 實作 `ProviderError` 與 `ResolutionError` enum（用 `thiserror::Error` derive），含 `error_code()` 方法回傳對應點分隔 code。觀察行為：3.2 測試從紅燈轉綠燈。驗證：3.2 綠燈

## 4. Provider trait 使用 async_trait 巨集並要求 Send + Sync + dyn-compatible

- [x] 4.1 觸發 `rust-skills:m05-type-driven` 與 `rust-skills:m07-concurrency`，確認 `async_trait` 巨集與原生 `async fn in trait` 在 `Arc<dyn Provider>` 場景的當前 idiom，並記錄結論
- [x] 4.2 在 `crates/provider/tests/dyn_provider_compile.rs` 撰寫編譯時測試：宣告 `fn accept(p: Arc<dyn Provider>) {}` 函式並用一個內含 `Mutex<HashMap>` 的 mock 結構 `Box<MockProvider>` → `Arc<dyn Provider>` 轉換。觀察行為：trait 物件可跨 thread 傳遞。驗證：紅燈（trait 尚未定義 → 編譯失敗）
- [x] 4.3 實作 `crates/provider/src/lib.rs` 的 `Provider` async trait（`#[async_trait] pub trait Provider: Send + Sync`），含 `create_change`、`write_artifact`、`get_change` 三個 method 與對應 `NewChange`、`NewArtifact` 輸入結構。觀察行為：4.2 編譯通過、`Arc<dyn Provider>` 可被建構與在 async 函式間傳遞。驗證：4.2 綠燈

## 5. 配置檔載入採 dirs 與 toml crate

- [x] 5.1 撰寫 `crates/provider/src/config.rs` 測試覆蓋 TOML parsing：合法 `ProjectConfig` 帶 `provider = "acme"` + `fallback = "local"`、缺欄位使用 default（`fallback = Local`）、不合法 `fallback = "remote"` 回傳錯誤。觀察行為：config 模組可從字串解析出強型別。驗證：`cargo test -p provider config::parse_tests` 紅燈
- [x] 5.2 實作 `ProjectConfig`、`GlobalConfig`、`FallbackPolicy` enum，包含 serde + toml 解析。觀察行為：5.1 測試從紅燈轉綠燈。驗證：5.1 綠燈
- [x] 5.3 撰寫 Configuration file locations and discovery 規格的測試：用 tempfile 建立目錄樹 `<root>/.git/`、`<root>/.speclink/config.toml`、`<root>/sub/sub2/`，從 `<root>/sub/sub2/` 搜尋會停在 `<root>/.speclink/config.toml`；同時測 `SPECLINK_CONFIG_HOME` 環境變數覆寫 global config 位置。觀察行為：search 演算法依規格停在 git root 或 filesystem root。驗證：紅燈
- [x] 5.4 實作 `find_project_config()` 與 `load_global_config()`，含 `SPECLINK_CONFIG_HOME` 覆寫與三平台預設路徑（用 `dirs::config_dir()`）。觀察行為：5.3 測試從紅燈轉綠燈。驗證：5.3 綠燈

## 6. Provider resolution 在 provider crate 內實作並回傳明確 enum

- [x] 6.1 撰寫 `crates/provider/src/resolution.rs` 測試覆蓋 Five-level provider resolution priority：五個層級各至少一個 case（flag 勝 / project config 勝 / global profile 勝 / env var 勝 / 全空回 local），並含 Provider resolution is testable independently of I/O 規格要求 — 測試純函式輸入 `ResolutionInputs`，無 filesystem 或環境呼叫。觀察行為：resolution 為純函式且優先序符合 spec 表。驗證：紅燈
- [x] 6.2 撰寫 Local fallback is always available unless explicitly disabled 與 Resolution result reports the local-fallback reason 測試：fallback=local 且 remote 未認證 → `ResolvedProvider::Local { reason: FallbackFromUnauthenticated }` + 一個 `provider.not_authenticated` warning；fallback=disabled 且 remote 未認證 → `ResolutionError::AuthRequiredNoFallback`；無 config → `Local { reason: NoConfig }`。觀察行為：每個 fallback 情境產生對應 `LocalReason` variant。驗證：紅燈
- [x] 6.3 實作 `resolve()` 純函式、`ResolvedProvider`、`LocalReason`、`ResolutionInputs`、`ResolutionError`。觀察行為：6.1 與 6.2 測試從紅燈轉綠燈。驗證：6.1、6.2 全綠 + `cargo clippy -p provider -- -D warnings` 通過

## 7. SQLite client 採 rusqlite 並包 spawn_blocking

- [x] 7.1 觸發 `rust-skills:m07-concurrency`（確認 spawn_blocking 與 `tokio::sync::Mutex` 互動的當前 idiom）與 `rust-skills:m11-ecosystem` + `rust-skills:rust-learner`（鎖定 rusqlite 當前穩定版本與 features）
- [x] 7.2 撰寫 `crates/provider-local/src/state_db.rs` 單元測試（tempfile）驗證 SQLite state database schema version 1：新 DB 開啟自動建立 schema 且 `PRAGMA user_version` 回 1、`in_progress_change semantics` 之 `INSERT OR REPLACE` 覆蓋舊 row、開啟 `user_version=2` 的 DB 回傳 `internal.error`。觀察行為：DB 初始化、版本檢查、INSERT OR REPLACE 行為符合 spec。驗證：`cargo test -p provider-local state_db::tests` 紅燈
- [x] 7.3 實作 `StateDb` 結構，內含 `Arc<tokio::sync::Mutex<rusqlite::Connection>>`，公開 async API（`open`、`set_in_progress`、`get_in_progress`），內部用 `tokio::task::spawn_blocking` 包裝同步操作，連線時 `PRAGMA journal_mode = WAL`。觀察行為：7.2 測試從紅燈轉綠燈。驗證：7.2 綠燈
- [x] 7.4 撰寫 Concurrent access safety 測試：在同一 tempfile DB 上連續執行兩次 `propose create` 對等的 set_in_progress 呼叫，確認 DB 不損毀且最後 row 為第二次值；同時確認 `PRAGMA journal_mode` 回傳 `wal`。觀察行為：WAL 模式啟用且序列寫入結果可預期。驗證：紅燈 → 實作後綠燈

## 8. Lifecycle state 採 metadata.json 與 SQLite 雙重儲存

- [x] 8.1 撰寫 `crates/provider-local/src/storage.rs` 測試覆蓋 Local provider directory layout：在 tempfile root 寫 proposal 後，`.speclink/changes/demo/proposal.md` 與 `.speclink/changes/demo/metadata.json` 存在、`.speclink/state.db` 存在、其他 spec 列出的可選子目錄（`design.md`、`tasks.md`、`specs/`、`archive/`、`packs/`、`cache/`）不被建立。觀察行為：local provider 只建立本 change 需要的檔案。驗證：紅燈
- [x] 8.2 撰寫 Atomic artifact write with metadata pairing 測試：注入寫入失敗（例如 metadata.json 寫入時拋錯）後，`.speclink/changes/<id>/` 整個目錄被 cleanup 移除；且確認沒有殘留 `.tmp` 檔。觀察行為：失敗的 propose create 不留下半成品狀態。驗證：紅燈
- [x] 8.3 撰寫 Change-id validation 測試，覆蓋 spec 表全部 7 個 case：`add-order-export` 接受、`a` 接受、`Add-Feature` 拒絕（uppercase）、`1add` 拒絕（leading digit）、`add--feature` 拒絕（連續 hyphen）、`add-` 拒絕（trailing hyphen）、空字串拒絕。觀察行為：validator 對 regex `^[a-z][a-z0-9-]{0,63}$` + 連續 hyphen + trailing hyphen 規則正確判定。驗證：紅燈
- [x] 8.4 實作 `write_proposal_atomic`（temp file → rename 順序：proposal.md.tmp → proposal.md、metadata.json.tmp → metadata.json）、`is_valid_change_id`、`cleanup_change_dir`，全部用 `std::path::PathBuf` 操作。觀察行為：8.1–8.3 測試從紅燈轉綠燈、metadata.json 含 `state = "proposed"` 與 ISO 8601 UTC `createdAt`。驗證：8.1、8.2、8.3 全綠

## 9. Local provider — Provider trait 實作

- [x] 9.1 撰寫 `crates/provider-local/tests/local_provider_integration.rs` 用 tempfile 驗證 `LocalProvider::create_change` + `LocalProvider::write_artifact` 串接後 `proposal.md` 內容為 `## Why\n\n<summary>\n`、`metadata.json` 為合法 JSON 且 `state = "proposed"`、SQLite 表 `in_progress_change` 有對應 row。觀察行為：trait 實作完整串接 storage + state_db。驗證：紅燈
- [x] 9.2 實作 `crates/provider-local/src/lib.rs` 的 `LocalProvider` 結構（持有 `StateDb`、`base_path: PathBuf`），impl `Provider` trait。觀察行為：9.1 測試從紅燈轉綠燈。驗證：9.1 綠燈

## 10. Runtime crate — propose orchestration

- [x] 10.1 撰寫 `crates/runtime/src/propose.rs` 測試使用 in-memory mock provider，驗證 `create_proposal` 呼叫順序（先 `create_change` 後 `write_artifact`，順序失敗時回傳對應 `RuntimeError`）、空 summary 與超長 summary（>200 char）皆回 `RuntimeError::InvalidInput`。觀察行為：runtime 入口函式正確 orchestrate provider 呼叫順序與輸入驗證。驗證：紅燈
- [x] 10.2 實作 `create_proposal(provider: Arc<dyn Provider>, input: CreateProposalInput) -> Result<CreateProposalOutput, RuntimeError>` 與 `RuntimeError` enum（`thiserror`，含 `Provider(ProviderError)`、`InvalidInput { reason: String }`）。觀察行為：10.1 測試從紅燈轉綠燈。驗證：10.1 綠燈

## 11. JSON output 採 typed serde 結構並集中在 cli output 模組

- [x] 11.1 撰寫 `crates/cli/src/output.rs` 測試覆蓋 Stable JSON envelope for `--json` output：success envelope（`ok=true`、`error=null`、`requestId` 符合 `^req_[0-9a-f]{32}$`）、failure envelope（`ok=false`、`data=null`、`error` 含 `code`/`message`/`details`）、JSON output schema for propose create（`data` 含 `changeId`/`state`/`artifactPath`/`mode` 四欄位且為 camelCase）、Warning 結構正確、`SPECLINK_TEST_REQUEST_ID` 環境變數覆寫 requestId。觀察行為：envelope 序列化結果完全符合 spec 規範。驗證：紅燈
- [x] 11.2 實作 `Envelope<T>`、`Warning`、`ErrorBody`、`ProposeCreateData` 結構（`serde::Serialize` + `rename_all = "camelCase"`），與 `request_id()` 函式。觀察行為：11.1 測試從紅燈轉綠燈。驗證：11.1 綠燈

## 12. Error 架構 lib 用 thiserror、cli 用 anyhow、跨層用點分隔 error code（cli 層 + Stable exit-code table）

- [x] 12.1 觸發 `rust-skills:m13-domain-error` 確認 anyhow 鏈中辨識特定 lib error variant 的 idiom（`Error::downcast_ref::<ProviderError>()`）
- [x] 12.2 撰寫 `crates/cli/src/exit_code.rs` 測試 `classify(err: &anyhow::Error)`，覆蓋 Stable exit-code table 與 Failure mapping 表所有條目：`ProviderError::NotAuthenticated → (6, "provider.not_authenticated")`、`ProviderError::Unavailable → (5, "provider.unavailable")`、`ProviderError::ChangeAlreadyExists → (1, "change.already_exists")`、`RuntimeError::InvalidInput → (2, "input.invalid")`、`LocalProviderError::InvalidChangeId → (2, "change.invalid_id")`、其餘 → (1, "internal.error")。觀察行為：classify 為 deterministic、輸入相同則 (exit, code) 相同。驗證：紅燈
- [x] 12.3 實作 `classify()` 與 `ExitCode` newtype，含 `ErrorCode` newtype 與 `as_str()` 方法。觀察行為：12.2 測試從紅燈轉綠燈、Error code naming convention 規定的 regex `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$` 對所有實作中出現的 code 通過。驗證：12.2 綠燈

## 13. 異步 runtime 採 tokio multi-thread 並在 main 內初始化

- [x] 13.1 觸發 `rust-skills:domain-cli` 取得 clap derive + tokio main + 共用 flag 結構（如 `MachineInterfaceFlags`）的當前 idiom
- [x] 13.2 撰寫 `crates/cli/src/cli.rs` 測試確認 clap parse 行為：`propose create --change demo --summary "x" --json --no-color --quiet` 正確解析、Machine-interface flag semantics 涵蓋 `--json`/`--no-color`/`--quiet`/`--stdin` 四旗標、`--summary ""` 在 clap 層被拒、無 subcommand 顯示 help、Cross-platform stdout line ending 規格（stdout 寫入用 `writeln!` 在 Windows 也是 `\n`）。觀察行為：clap 結構與 spec 一致。驗證：紅燈
- [x] 13.3 實作 `crates/cli/src/cli.rs` 的 `Cli`、`ProposeArgs::Create(ProposeCreateArgs)`、共用 `MachineInterfaceFlags { json, no_color, quiet, stdin }`。觀察行為：13.2 測試從紅燈轉綠燈、`speclink --help` 列出 propose 子指令樹。驗證：13.2 綠燈
- [x] 13.4 撰寫 `crates/cli/src/main.rs` 含 `#[tokio::main(flavor = "multi_thread")]`、tracing subscriber 初始化、ExitCode 回傳、stdout 強制 LF（不寫 `\r\n`）。觀察行為：CLI binary 可被執行且支援 async 命令處理。驗證：`cargo run -p cli -- --help` 顯示完整 subcommand 樹

## 14. `propose create` command surface — end-to-end 整合測試

- [x] 14.1 撰寫 `crates/cli/tests/propose_create.rs` 用 `assert_cmd` 覆蓋 `propose create` command surface：成功流程（exit 0、stdout 一行 JSON）、change.invalid_id（exit 2）、空 summary（exit 2）、超長 summary 201 char（exit 2）。觀察行為：command 對輸入規格回應對應 exit code。驗證：紅燈
- [x] 14.2 撰寫 Failure mapping 與 Warning emission on remote-to-local fallback 整合測試：tempfile project config `provider = "acme"` + 無 auth + `fallback = "local"` → exit 0 帶 `provider.not_authenticated` warning；同條件 + `fallback = "disabled"` → exit 6；project config 含已存在的 change id → exit 1 with `change.already_exists`。觀察行為：fallback 行為與失敗映射符合 spec 表。驗證：紅燈
- [x] 14.3 撰寫 Successful proposal creation produces a defined side effect 整合測試：成功執行後依序檢查 `.speclink/changes/demo/proposal.md` 內容、`.speclink/changes/demo/metadata.json` 含 Lifecycle transition `draft` to `proposed` 後的 `state = "proposed"`、SQLite `in_progress_change` 含對應 row。觀察行為：副作用順序與 spec 一致。驗證：紅燈
- [x] 14.4 撰寫 Secret-free output guarantee 整合測試：grep stdout JSON、`proposal.md`、`metadata.json`、`.speclink/state.db` binary，確認皆不含 `token`、`access_token`、`refresh_token`、`api_key`、`password`、`Bearer ` 字串。觀察行為：產出絕無 secret leak。驗證：紅燈
- [x] 14.5 實作 `crates/cli/src/commands/propose.rs::run` 函式 wiring：載入 config → `resolve()` → 建立 `LocalProvider` → 呼叫 `runtime::create_proposal` → 組 envelope → 寫 stdout → 回 ExitCode。觀察行為：14.1–14.4 測試從紅燈轉綠燈。驗證：14.1、14.2、14.3、14.4 全綠

## 15. Secret redaction in tracing output

- [x] 15.1 撰寫 Secret redaction in tracing output 規格對應的測試（用 `tracing_subscriber::fmt::TestWriter`），確認 `tracing::info!(token = "abc123", "doing work")` 經 redaction layer 後 captured output 含 `[REDACTED]` 而非 `abc123`，同時對 `Bearer abc123` 的 `Display` 值也替換。觀察行為：tracing event 中的 secret 欄位被 redact。驗證：紅燈
- [x] 15.2 實作 Secret redaction in tracing output 的 layer（`impl Layer for SecretRedactionLayer`），在 `crates/cli/src/main.rs` 的 subscriber 註冊鏈中插入。觀察行為：15.1 測試從紅燈轉綠燈、執行 propose create 時 stderr 不出現 secret 字串。驗證：15.1 綠燈

## 16. insta snapshot 鎖定 JSON output

- [x] 16.1 撰寫 `crates/cli/tests/propose_create_snapshots.rs` 用 insta + `SPECLINK_TEST_REQUEST_ID=req_00000000000000000000000000000000` 鎖定三種 JSON output snapshot：success（含 `data` 完整內容）、`change.already_exists` failure、`provider.not_authenticated` warning。觀察行為：JSON envelope 行為被 snapshot 鎖定，後續變更需顯式 `cargo insta accept`。驗證：`cargo insta accept` 後 `cargo test -p cli --test propose_create_snapshots` 通過

## 17. Cross-platform CI

- [x] 17.1 建立 `.github/workflows/ci.yml`：matrix `os: [ubuntu-latest, macos-latest, windows-latest]` × `toolchain: [stable]`，jobs 含 `cargo fmt --check`、`cargo clippy --workspace -- -D warnings`、`cargo test --workspace`，啟用 `Swatinem/rust-cache` 或 `actions/cache` 對 `~/.cargo/registry`、`~/.cargo/git`、`target/` 快取。觀察行為：每個 push 觸發三平台 CI 全綠。驗證：推 dev branch 後 GitHub Actions 三平台全綠

## 18. 重構與 idiomatic 檢查

- [x] 18.1 觸發 `rust-skills:m15-anti-pattern` 與 `rust-skills:coding-guidelines`，對 cli / runtime / provider / provider-local 四個 crate 各跑一次 anti-pattern review，整理建議清單為 inline comment
- [x] 18.2 套用建議：移除冗餘 `clone()`、修正 ownership pattern、補 `///` 繁體中文 doc comment 至所有公開 API、確認 lib crate 不含 `unwrap()` 或 `expect()`（除明確不變式註解）。觀察行為：clippy 全部通過、`cargo doc --workspace --no-deps -- -D missing_docs` 通過（公開 API 全有 doc）。驗證：`grep -rn "\.unwrap()\|\.expect(" crates/{provider,provider-local,runtime}/src/` 命中 0 或只含 `// SAFETY:` 註解；clippy 與 doc 兩個 cargo 指令通過

## 19. 手動驗收

- [x] 19.1 在乾淨臨時目錄手動執行 `speclink propose create --change demo --summary "test summary" --json`，依序確認：stdout 一行 JSON 可被 `jq .` 解析、`.speclink/changes/demo/proposal.md` 內容為 `## Why\n\ntest summary\n`、`.speclink/changes/demo/metadata.json` 為合法 JSON 且 `state = "proposed"`、`.speclink/state.db` SQLite 查詢 `SELECT change_id FROM in_progress_change;` 回傳 `demo`、process exit code 為 0。觀察行為：MVP vertical slice 在開發者機器上端到端可用。驗證：手動 checklist 全部勾選 + 在 PR 描述附上 stdout JSON 樣本與 SQLite 查詢輸出
- [x] 19.2 重複 19.1 但在已存在 `.speclink/changes/demo/` 的目錄執行，確認 exit code 1 + JSON 含 `error.code = "change.already_exists"` + 既有目錄內容未被覆寫。觀察行為：second-run protection 生效。驗證：手動 diff 既有目錄前後內容無差異
