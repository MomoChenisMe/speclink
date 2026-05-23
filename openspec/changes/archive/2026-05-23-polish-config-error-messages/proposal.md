## Problem

A5 (`add-config-rw`) 落地後 QA 發現 3 個 config 路徑的 error message / error code 不潔淨：

1. **`config edit` 無 `--stdin` / `--editor` / `$EDITOR`** 時回 `config.key_not_found`，message 把 hint 塞進 `key` 欄位讀起來像「config key `<edit-mode>: pass --stdin or set $EDITOR` not found」。語意上 user 沒給 key、是 mode 不足，硬塞 `config.key_not_found` 會誤導 AI / 人類 caller。
2. **`config show --key rules.*`** 拒絕 wildcard 時 error.code 為 `config.key_not_found`（正確）、但 message body 把診斷字串「wildcards not supported」當 key interpolate 進去，輸出「config key `wildcards not supported` not found」讀起來像 key 命名怪。
3. **`state.etag_mismatch`** 的 `Display` impl 直接用 `{:?}` Debug format，輸出「config etag mismatch (expected=Some("v99.deadbeef0000"), actual=v2.4bf38086a3c7)」洩漏 Rust `Some(...)` wrapper，AI consumer 要清掉這層才能用。

3 條都是 message / code 呈現問題，不影響功能（A1–A5 E2E 41 case 全綠），但會被下游 AI agent 的 error renderer 卡到。

## Root Cause

1. `crates/cli/src/commands/config.rs::run_edit` 在 stdin / editor 兩條 path 都缺時直接 reuse `RuntimeError::ConfigKeyNotFound` 變體當 carrier，沒有對應的「mode required」error variant。
2. `crates/cli/src/commands/config.rs::jsonpath_to_runtime_error` 把 `JsonPathParseError::UnsupportedWildcard` / `UnsupportedSyntax` 直接塞進 `RuntimeError::ConfigKeyNotFound { key: <diagnostic-string> }`，造成 message template「config key `{key}` not found」吞下診斷字串。
3. `crates/provider/src/error.rs` 與 `crates/runtime/src/error.rs` 兩處 `StateEtagMismatch` variant 的 `#[error(...)]` attribute 用 `expected={expected:?}` 走 Debug format、`Option` 包了一層 `Some(...)`；應改 Display + 對 None 印 `<none>` / 對 Some 印純值。

## Proposed Solution

- **(1) 新增 error code `config.edit_mode_required`**（exit 2）+ `ProviderError::ConfigEditModeRequired` / `RuntimeError::ConfigEditModeRequired` 兩 variant；CLI `run_edit` 改 raise 此 variant、message 形如「`config edit` requires --stdin or --editor / $EDITOR」。
- **(2) JSONPath parse 失敗時保留原始 `<key>` 字串於 message**，把診斷理由（wildcard / filter / bad-segment）走 message 而非 key field；error code 仍維持 `config.key_not_found`（spec 不變）。具體：CLI layer 在 `jsonpath_to_runtime_error` 把 `JsonPathParseError` 各 variant map 成 `RuntimeError::ConfigKeyNotFound { key: <原 user 輸入> }` + 在 message 額外帶 hint，而非把 reason 塞進 key。
- **(3) `StateEtagMismatch` Display polish**：兩 crate 的 `#[error(...)]` attribute 改用 Display 模式 + 對 `Option<String>` 加 helper format（`Some(v)` → `v`、`None` → `<none>`），輸出形如「config etag mismatch (expected=v99.deadbeef0000, actual=v2.4bf38086a3c7)」或「expected=<none>」。

3 條都附對應 cargo test 驗 stdout JSON envelope 的 `error.code` / `error.message` 字串。

## Non-Goals

- 不擴展 JSONPath grammar（wildcard / filter / recursive-descent 仍拒絕）。
- 不重新編列其他 slice 的 error message hygiene（限 config / state.etag_mismatch 三條）。
- 不動 spec-level scenario 描述（A5 已 archive-ready，僅做最小 spec delta 新增 1 條 error code）。
- 不引入 i18n 框架（message 仍英文 hard-coded）。

## Success Criteria

- `speclink --json config edit`（無 stdin / editor / $EDITOR）exit 2、`error.code == "config.edit_mode_required"`、message 提示「pass --stdin or set $EDITOR」。
- `speclink --json config show --key 'rules.*'` exit 2、`error.code == "config.key_not_found"`、`error.message` 含「rules.*」（user 原始輸入）而非「wildcards not supported」當 key 名。
- `speclink --json config set rules.require_code_review true --expected-etag v99.bogus` exit 7、`error.code == "state.etag_mismatch"`、`error.message` 不含字面 `Some(` 或 `)`、形如「config etag mismatch (expected=v99.bogus, actual=v<n>.<sha>)」。
- A5 既有 41 個 walking-skeleton QA case 不退步、`cargo test --workspace` 全綠、`cargo clippy --workspace -- -D warnings` clean、`cargo fmt --check` clean。

## Impact

- Affected specs: `config-rw`（modify 既有「New error codes SHALL be registered with stable exit codes」requirement 新增 `config.edit_mode_required` row）
- Affected code:
  - Modified:
    - crates/provider/src/error.rs
    - crates/runtime/src/error.rs
    - crates/cli/src/commands/config.rs
    - crates/cli/src/main.rs（exit code mapping 若需）
    - crates/provider/tests/error_codes.rs
    - crates/cli/tests/config_cli.rs
  - New: （無）
  - Removed: （無）
