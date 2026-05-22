## 0. Spec ↔ task traceability checklist

對應 `cli-human-output` spec 中每條 requirement 與 design 中每個 decision heading 的 task 對應；analyzer 透過此節做 substring matching 完成 coverage 追蹤。

### cli-human-output requirements

- [x] 0.1 Requirement covered: `Human-mode CLI output SHALL recursively pretty-print the `data` payload` — by tasks §1.1, §1.2, §1.3, §2.1
- [x] 0.2 Requirement covered: `The renderer SHALL NOT alter `--json` envelope behavior` — by tasks §2.2, §3.1
- [x] 0.3 Requirement covered: `The renderer SHALL leave stderr error and hint output untouched` — by tasks §2.3, §3.2

### Design decisions

- [x] 0.4 Design decision covered: `採遞迴 + 縮排 pretty-print，不引入 dependency` — by tasks §1.1, §1.2
- [x] 0.5 Design decision covered: `縮排採 2 空白；array element 用 `- ` 前綴` — by tasks §1.2, §1.3
- [x] 0.6 Design decision covered: `縮排層級從 0 開始，top-level object key 不縮排` — by tasks §1.2
- [x] 0.7 Design decision covered: `模組拆分：邏輯放 `crates/cli/src/human.rs`，`main.rs` 只剩 1 行委派` — by tasks §1.1, §2.1
- [x] 0.8 Design decision covered: `Snapshot test 以 insta 鎖輸出格式` — by tasks §1.3

### Design Implementation Contract subsections

- [x] 0.9 Subsection covered: `Behavior` — by tasks §1.2, §1.3, §2.1
- [x] 0.10 Subsection covered: `Interface / data shape` — by tasks §1.1, §2.1
- [x] 0.11 Subsection covered: `Failure modes` — by tasks §1.2（pure function 無 panic）
- [x] 0.12 Subsection covered: `Acceptance criteria` — by tasks §3.1, §3.2, §3.3, §3.4
- [x] 0.13 Subsection covered: `Scope boundaries` — by proposal Non-Goals and tasks §1 through §3

## 1. `crates/cli/src/human.rs` 新模組與遞迴 renderer（TDD red-green-refactor）

對應 design「採遞迴 + 縮排 pretty-print」「縮排採 2 空白」「縮排層級從 0 開始」「模組拆分」與 spec requirement「Human-mode CLI output SHALL recursively pretty-print the `data` payload」。

- [x] 1.1 紅燈：新增 `crates/cli/src/human.rs` 模組（暫留空 `pub fn render_human(value: &serde_json::Value) -> String { String::new() }`），在同檔 inline `mod tests` 撰寫 7 支 `#[test]` 對應 spec 7 個 scenario：`empty_object`、`flat_object`、`nested_object`、`array_of_objects`、`empty_array`、`array_of_scalars`、`string_with_newlines`，每支 assertion 比對 spec 中 example 的精確 expected string。**契約**：紅燈覆蓋 spec 全部 7 個 scenario。**驗證**：`cargo test -p speclink-cli human` 顯示 assertion failure（render_human 回空字串）。
- [x] 1.2 [P] 綠燈：實作 `render_human(value: &serde_json::Value) -> String` 與底層 `render_value(value: &serde_json::Value, indent: usize, out: &mut String)`。**規則**：(a) `Object` 非空 → 對每個 (key, value) 印 `key: <scalar>` 或 `key:\n<indented nested>`；(b) `Object` 空 → `OK`；(c) `Array` 非空 → 每元素 `- <scalar>` 或 `-\n<indented>`；(d) `Array` 空 → `(empty)`；(e) `String` → raw（無 JSON 引號），含 `\n` 在每換行後補 current indent；(f) `Number` / `Bool` / `Null` → `to_string()`。縮排 step = 2 空白。**契約**：1.1 紅燈 7 支全綠；對應 spec example 中所有 expected string byte-for-byte 相符。**驗證**：`cargo test -p speclink-cli human` 全綠。
- [x] 1.3 [P] 重構：把 1.2 的 7 支 `assert_eq!` 改為 `insta::assert_snapshot!` 形式，建立 `crates/cli/src/snapshots/speclink_cli__human__tests__*.snap`；確認 snapshot 內容與 spec example 完全一致。**契約**：spec 7 個 example 都對應一個 stable snapshot。**驗證**：`cargo insta test -p speclink-cli human` 全綠；`cargo insta accept --workspace` 後 git diff 顯示 7 個新 `.snap` 檔。

## 2. `crates/cli/src/main.rs::print_human_ok` 委派 + lib.rs 暴露模組

對應 spec requirement 與 design「模組拆分」「Behavior」「Interface / data shape」。

- [x] 2.1 在 `crates/cli/src/lib.rs` 新增 `pub mod human;`；把 `crates/cli/src/main.rs::print_human_ok` 改寫為單行委派 `println!("{}", speclink_cli::human::render_human(data));`，刪除既有的 for-loop / nested if 分支。**契約**：對應 design「Interface / data shape」中 `print_human_ok` 改為 single-line delegation。**驗證**：`cargo build -p speclink-cli` 成功；`cargo test -p speclink-cli` 全綠。
- [x] 2.2 [P] 紅燈+綠燈：在 `crates/cli/tests/cli.rs` 新增整合測試 `human_mode_show_change_avoids_raw_json_chars`：跑 `speclink show change <name>`（不帶 `--json`）後驗證 stdout 不含 `{"`、`":[`、`"]`、`",`、`":{` 等 JSON 殘留標記（用 `assert!(!stdout.contains("{\""), ...)` 等多支 assertion）；fixture 使用 bootstrap-init + `new change` + 預植 proposal / design / specs/user-auth/spec.md 模擬 slice-A scenario 9 的 3-artifact 結構。**契約**：對應 spec scenario「Array of objects renders each element with `-` bullet」與「Nested object value renders with 2-space indentation」端到端驗證。**驗證**：`cargo test -p speclink-cli human_mode_show_change_avoids_raw_json_chars` 全綠。
- [x] 2.3 [P] 紅燈+綠燈：在 `crates/cli/tests/cli.rs` 新增整合測試 `human_mode_failure_stderr_unchanged`：跑 `speclink show change unknown`（不帶 `--json`、不存在的 change）後驗證 stderr 含 `error[change.not_found]:` 開頭一行與 `hint:` 開頭一行，stdout 為空。**契約**：對應 spec requirement「The renderer SHALL leave stderr error and hint output untouched」。**驗證**：`cargo test -p speclink-cli human_mode_failure_stderr_unchanged` 全綠。

## 3. 既有 snapshot / 既有測試 regression 保護

對應 spec requirement「The renderer SHALL NOT alter `--json` envelope behavior」與 design 中 acceptance criteria 第 5 列「`--json` mode 對 11 個 op byte-for-byte 不變」。

- [x] 3.1 跑 `cargo test --workspace` 全綠：bootstrap-slice `crates/runtime/tests/bootstrap.rs` + slice-A `crates/cli/tests/{change_crud,artifact_io,etag_concurrency,snapshots}.rs` 與 `crates/runtime/tests/{change_ops,artifact_ops}.rs` 一個都不能 regression；特別確認 `crates/cli/tests/snapshots.rs` 10 個 slice-A snapshot 與 `crates/cli/src/snapshots/speclink_cli__output__tests__error_envelope_pretty.snap` 不變。**契約**：對應 design「Acceptance criteria」第 2、3、5 列。**驗證**：`cargo test --workspace` 全綠。
- [x] 3.2 跑 `cargo insta pending --workspace`：本 change 完成後預期只有 7 個 *新* pending snapshot（對應 §1.3），不應出現 既有 snapshot 的 diff。**契約**：對應 spec scenario「Existing JSON snapshots remain stable」。**驗證**：`cargo insta pending --workspace` 列表中只見 `speclink_cli__human__tests__*.snap`，無既有 snapshot 被改。
- [x] 3.3 跑 `cargo fmt --check` 與 `cargo clippy -p speclink-cli -- -D warnings -W clippy::pedantic`。**契約**：對應 design「Acceptance criteria」第 6 列。**驗證**：兩個指令 exit 0。
- [x] 3.4 人工 smoke：在乾淨的 git working tree 跑 `git init && speclink init && speclink new change demo && printf "## Why\n\nbody\n" | speclink new artifact proposal --change demo --stdin && speclink show change demo`（最後一條不加 `--json`），目視確認輸出含 `name: demo`、`artifacts:` 後面跟 2 空白縮排的 `- kind: proposal` 形狀，且不含任何 `{"` / `"]` 字元。**契約**：對應 design「Acceptance criteria」第 1 列端到端工程師體驗。**驗證**：手動跑指令，輸出符合預期。
