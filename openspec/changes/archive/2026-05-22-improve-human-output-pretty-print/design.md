## Context

bootstrap slice 與 slice-A (`add-change-and-artifact-io`) 完成後，`crates/cli/src/main.rs::print_human_ok` 對 `serde_json::Value` 採取「if object, iterate top-level keys; else println the whole thing」邏輯，nested object/array 被當 string 直接 stringify 印出。

實測（2026-05-22）`speclink show change demo-billing`（不帶 `--json`）的輸出：

```
artifacts: [{"capability":null,"kind":"proposal"},{"capability":null,"kind":"tasks"},{"capability":"user-auth","kind":"spec"}]
change: {"changeId":"a9d53bbf-65c8-49a8-8bf7-67fdbb0361b5","createdAt":"2026-05-22T06:52:55.980182Z","name":"demo-billing","schemaId":"spec-driven","state":"proposing","updatedAt":"2026-05-22T06:52:55.980182Z","version":1}
```

對直接在 terminal 跑 CLI 的工程師可讀性差。`--json` mode 不受影響（machine interface 完全正確）。本 design 把 `print_human_ok` 換成遞迴 pretty-printer，並以 `cli-human-output` capability 鎖住輸出格式契約。

## Goals / Non-Goals

**Goals:**

- 人類使用者跑 `speclink show change <name>`、`status`、`list --changes` 等指令時得到縮排清楚、不含 JSON 引號的輸出
- 把人類輸出格式制定為 capability 級別契約（`cli-human-output`），未來 op 一律遵循同一份格式
- 完全保留 `--json` envelope 的 byte-for-byte 行為與 stderr 上 error/hint 的既有印法
- 完全保留 bootstrap unlink 那種「空 data object 印 `OK`」的既有語意

**Non-Goals:**

- ❌ 不引入 ANSI color、TTY detection、table formatter dependency
- ❌ 不引入新 CLI 旗標
- ❌ 不動 `Envelope` / `Warning` / `ErrorDetail` 序列化
- ❌ 不調整 hint / error 在 stderr 上的印法
- ❌ 不擴及 `cli` 以外的 crate
- ❌ 不處理超深巢狀（>10 層）的 ANSI overflow / terminal width-wrap — 本場景 SpecLink data shape 深度永遠 ≤ 3 層
- ❌ 不在 human mode 顯示 `warnings` 與 `requestId`（bootstrap 行為，沿用）

## Decisions

### 採遞迴 + 縮排 pretty-print，不引入 dependency

| 選項 | 取捨 |
|---|---|
| A. 遞迴 + 2 空白縮排（本案採用） | 標準庫即可，0 新 dep，產出可讀 |
| B. `comfy-table` / `prettytable` 渲染 | 體積大、跨平台 ANSI 行為複雜、超出 polish 範圍 |
| C. 直接呼叫 `serde_json::to_string_pretty` | 仍有 JSON 引號與括號，沒解決可讀性問題 |

**選 A**。`serde_json::Value` 已是 in-memory tree，遞迴穩定。SpecLink 目前所有 op data shape 深度 ≤ 3 層（`show change` 是最深的：`data → artifacts[] → {kind, capability}` 也才 3 層），效能無虞。

### 縮排採 2 空白；array element 用 `- ` 前綴

**規則表**：

| Value 型別 | 渲染 |
|---|---|
| `Object`（非空） | 對每個 key 印一行 `key: <value>`；若 value 為 nested object/array，先印 `key:` 加換行，內容在下一層 +2 空白縮排 |
| `Object`（空，`{}`） | 印 `OK`（bootstrap unlink 沿用） |
| `Array`（非空） | 對每個 element 印一行 `- <value>`；若 element 為 nested object/array，下一層 +2 空白縮排 |
| `Array`（空，`[]`） | 印 `(empty)` |
| `String` | 印原字串（不加引號）；若含 `\n`，每個換行後加當前縮排 |
| `Number` / `Bool` / `Null` | `to_string()` |

**為什麼用 `- ` 而非 `[0]` index**：YAML / Markdown 慣例、目視掃描容易；index 對人類沒意義（machine 才需要 index，那條路徑走 `--json`）。

### 縮排層級從 0 開始，top-level object key 不縮排

```
name: billing-system
artifacts:
  - kind: proposal
    capability: null
  - kind: spec
    capability: user-auth
change:
  changeId: a9d53bbf-...
  name: demo-billing
```

**為什麼 top-level 不縮排**：terminal 直接讀比較自然；後續每進一層 +2 空白即可。

### 模組拆分：邏輯放 `crates/cli/src/human.rs`，`main.rs` 只剩 1 行委派

避免 `main.rs` 越來越肥；`human.rs` 內把 `render_human(value) -> String` 與 `render_value(value, indent_level, output)` 分層，方便單元測試 + insta snapshot 鎖格式。

### Snapshot test 以 insta 鎖輸出格式

`cli-human-output` capability 列舉 7 個 GIVEN/WHEN/THEN scenario（flat object / nested object / array of object / mixed / empty object / empty array / string with newline），實作對應 `crates/cli/src/human.rs::tests` 內 7 支 `#[test]` + `insta::assert_snapshot!`。

## Implementation Contract

### Behavior

`speclink <subcommand>`（不帶 `--json`）成功時：

1. 若 data 為空 object `{}` → stdout 印 `OK` + 換行
2. 若 data 為非空 object → 對每個 top-level key 印 `key: <rendered>` 一行，nested value 採 +2 空白縮排展開
3. 若 data 為 array → 對每個 element 印 `- <rendered>`，nested 採 +2 空白縮排
4. 若 data 為純 scalar（極少數）→ 直接 `to_string()` + 換行

失敗時 stderr 行為**完全沿用 bootstrap**：`eprintln!("error[{code}]: {err}");` 與 `eprintln!("hint: {h}");`，本 change 不動。

### Interface / data shape

新增模組 `crates/cli/src/human.rs`：

```rust
/// Pretty-print a SpecLink `data` payload for human-mode CLI output.
///
/// 規則見 `cli-human-output` capability spec。
pub fn render_human(value: &serde_json::Value) -> String;
```

`crates/cli/src/main.rs::print_human_ok` 改為：

```rust
fn print_human_ok(data: &serde_json::Value) {
    println!("{}", speclink_cli::human::render_human(data));
}
```

`crates/cli/src/lib.rs` 加 `pub mod human;`。

### Failure modes

本 change 不引入新 error code，不改既有 error code。`render_human` 是純 pure function，無 I/O、無 panic 路徑（`serde_json::Value` 變體 exhaustive 處理）。

### Acceptance criteria

| # | 驗證項目 | 驗證手段 |
|---|---|---|
| 1 | `show change <name>` non-JSON output 不含 `{"` / `"]` 等 JSON 殘留字元 | `crates/cli/tests/cli.rs` 新增整合測試 grep 輸出 |
| 2 | `unlink` non-JSON output 仍為 `OK` | 既有測試保持綠 |
| 3 | `init` / `status` / `link` non-JSON output 印出每個 top-level key（既有 4 op flat data 行為不退化） | 既有測試保持綠 |
| 4 | `human.rs::tests` 7 個 case 透過 `insta::assert_snapshot!` 鎖輸出格式 | `cargo insta test -p speclink-cli human` |
| 5 | `--json` mode 對 11 個 op（bootstrap 4 + slice-A 7）byte-for-byte 不變 | 既有 `cli/tests/snapshots.rs` + `cli.rs` snapshot 全綠 |
| 6 | clippy pedantic 通過 | `cargo clippy -p speclink-cli -- -D warnings -W clippy::pedantic` |

### Scope boundaries

**In scope:**

- 重寫 `print_human_ok` 邏輯（從 main.rs 抽到 human.rs）
- 新增 `render_human` + `render_value` 遞迴 helper
- 7 個 inline snapshot test 鎖格式
- 新 capability `cli-human-output` spec 鎖契約

**Out of scope:**

- `cli` 以外任何 crate
- `Envelope` / `SuccessBody` / `ErrorBody` / `Warning` / `ErrorDetail` 結構
- stderr 上的 error / hint 印法
- ANSI color / TTY detection / 終端寬度感知 / table 渲染
- 新增任何 CLI 旗標
- 修改任何既有 op 的 data shape

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `serde_json::Value::String` 含特殊字元（`\t` / control char）渲染醜 | spec 明示 control char 不額外 escape，照 raw 印；現實 data shape 不會出現 |
| 後續 op 加超深巢狀 data（> 3 層）拖累目視效果 | 後續 slice 設計 data shape 時自然會避免；若真的需要，再開 polish 後續 |
| 引號被拿掉後，含 `:` 的 string value 視覺上像是新 key | scenario 6（`{"path": "changes/foo/proposal.md"}` 渲染為 `path: changes/foo/proposal.md`）已驗證可讀；CLI 工具圈普遍接受 |
| `human.rs` 測試與 `--json` snapshot 雙寫，要維護 | 兩者鎖不同 mode，分工清楚；本 change 只新增 `human.rs::tests` snapshot，不動既有 |

## Migration Plan

本 change 沒有 state migration、沒有 binary 相容性問題。Build 後直接出貨。

- Rollback：`git revert` 即可，無外部副作用
- Release notes：標明「human mode 輸出更易讀；`--json` 行為完全不變」

## Open Questions

(none — 設計已收斂)
