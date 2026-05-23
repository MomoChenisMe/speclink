# Tasks — polish-config-error-messages

## 1. Issue 1：新增 `config.edit_mode_required` error code

對應 spec requirement「New error codes SHALL be registered with stable exit codes」MODIFIED delta：把 `config.edit_mode_required` (exit 2) 加進 error code 表（spec scenario「`config edit` without input mode emits `config.edit_mode_required`」與「All six codes appear in CLI error registry」）。

- [x] 1.1 [P] 在 `crates/provider/tests/error_codes.rs` 新增紅燈測試 `provider_error_config_edit_mode_required_has_correct_code` 斷言「New error codes SHALL be registered with stable exit codes」requirement 對新 code 成立：`ProviderError::ConfigEditModeRequired.code() == "config.edit_mode_required"`、`.exit_code()` 之等價值（透過 RuntimeError mapping）為 2、`.retryable() == false`；驗證：`cargo test -p speclink-provider --test error_codes` 紅燈
- [x] 1.2 在 `crates/provider/src/error.rs` 新增 `pub const CONFIG_EDIT_MODE_REQUIRED: &str = "config.edit_mode_required";` 常量、`ProviderError::ConfigEditModeRequired` variant（無欄位、`#[error("`speclink config edit` requires --stdin, --editor <cmd>, or $EDITOR")]`）、`code()` 對應該變體回新常量、`retryable()` 不變（false）；對應 spec requirement「New error codes SHALL be registered with stable exit codes」；驗證：1.1 由紅轉綠
- [x] 1.3 在 `crates/runtime/src/error.rs` 新增 `RuntimeError::ConfigEditModeRequired` variant + `code()` mapping + `exit_code()` mapping（exit 2）+ `retryable()` （false）+ 在現有 `map_provider_error*` 5 處 mapper 增加 `PE::ConfigEditModeRequired => RuntimeError::ConfigEditModeRequired` 對映；驗證：`cargo build --workspace` clean
- [x] 1.4 在 `crates/cli/src/commands/config.rs::run_edit` 把現行的「無 stdin / editor / $EDITOR」fallback 由 `RuntimeError::ConfigKeyNotFound` 改為 `RuntimeError::ConfigEditModeRequired`；驗證：手動跑 `speclink --json config edit`（清 EDITOR env）回 `error.code == "config.edit_mode_required"`
- [x] 1.5 [P] 在 `crates/cli/tests/config_cli.rs` 新增整合測試 `config_edit_without_stdin_or_editor_returns_config_edit_mode_required` 斷言 exit 2、`error.code == "config.edit_mode_required"`、`error.message` 含字面字串「--stdin」與「$EDITOR」；同時把既有的 `config_edit_without_stdin_or_editor_returns_config_key_not_found_with_hint` test 改名 / 更新 expected code 以對應新行為；驗證：`cargo test -p speclink-cli --test config_cli` 全綠

## 2. Issue 2：JSONPath parse 失敗訊息保留 user 原 key

- [x] 2.1 [P] 在 `crates/cli/tests/config_cli.rs` 新增紅燈測試 `config_show_wildcard_key_preserves_user_input_in_message`：跑 `config show --key 'rules.*' --json`、斷言 `error.code == "config.key_not_found"`、`error.message` 含字面字串「rules.*」、`error.message` **不** 含「wildcards not supported」當 key 名（即 message 中「config key `wildcards not supported`」這種片段 SHALL NOT 出現）；驗證：紅燈
- [x] 2.2 修改 `crates/cli/src/commands/config.rs` 兩處 JSONPath parse 路徑（`run_show` 與 `run_set`）：把現行的 `JsonPath::parse(key).map_err(jsonpath_to_runtime_error)` 拆成 (a) 先存 `let raw_key = key.to_string();`、(b) parse 失敗時呼叫新 helper `jsonpath_error_with_user_input(err, &raw_key)`，這個 helper 把 `RuntimeError::ConfigKeyNotFound { key: raw_key.clone() }` 與診斷理由放入一個新欄位（透過 `RuntimeError::ConfigKeyNotFound` Display impl 帶 hint，例如改 `#[error("config key `{key}` not found{hint}")]`、`hint: Option<String>` 走 default 空字串、wildcard 路徑塞 ": wildcards / filters not supported"）；驗證：2.1 由紅轉綠

## 3. Issue 3：`state.etag_mismatch` Display polish

- [x] 3.1 [P] 在 `crates/provider/tests/error_codes.rs` 新增紅燈測試 `state_etag_mismatch_display_does_not_leak_rust_debug_some_wrapper`：建構 `ProviderError::StateEtagMismatch { expected: Some("v1.aaa".into()), actual: "v2.bbb".into() }`、`format!("{e}")` 結果 SHALL NOT contain `Some(`、SHALL contain `v1.aaa` 與 `v2.bbb`；同檔加 `state_etag_mismatch_display_renders_none_as_explicit_marker`：`expected: None` 時 message 不含 `None` 關鍵字、含 `<none>`；驗證：紅燈
- [x] 3.2 修改 `crates/provider/src/error.rs::ProviderError` 的 `StateEtagMismatch` variant：把 `#[error(...)]` attribute 改成自訂 Display 邏輯（或保留 thiserror format、把欄位走 `display_expected` helper return `&'static str` / `String`）；簡單做法：拆 variant 為 `StateEtagMismatch { expected: Option<String>, actual: String }`、改 `#[error("config etag mismatch (expected={}, actual={actual})", expected.as_deref().unwrap_or("<none>"))]` 形式；同步處理 `crates/runtime/src/error.rs` 內 `RuntimeError::StateEtagMismatch` Display；驗證：3.1 由紅轉綠
- [x] 3.3 [P] 在 `crates/cli/tests/config_cli.rs` 新增整合測試 `config_set_wrong_expected_etag_message_does_not_leak_rust_debug`：跑 `config set rules.require_code_review true --expected-etag v99.bogus0000000 --json`、解析 JSON envelope、斷言 `error.message` 含「v99.bogus0000000」、含 envelope 當前 actual etag 字串、不含字面 `Some(`、不含字面 `None`；驗證：3.2 + 3.3 全綠

## 4. Cross-issue regression + 文件更新

- [x] 4.1 跑 `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace`；驗證：三條 CI 命令皆綠
- [x] 4.2 [P] 在 `doc/protocol/operations.md` `config.write` op 的 Errors 表加 `config.edit_mode_required` row（status: `implemented (polish-config-error-messages)`）；對齊 A5 既有 status convention；驗證：`grep "config.edit_mode_required" doc/protocol/operations.md` 至少 1 次
- [x] 4.3 跑 `spectra analyze polish-config-error-messages --json`，遞補/修正所有 Critical / Warning finding；驗證：第二次跑 analyze 後 critical/warning 為 0
- [x] 4.4 跑 `spectra validate polish-config-error-messages` 確認 strict 模式通過；驗證：exit 0
