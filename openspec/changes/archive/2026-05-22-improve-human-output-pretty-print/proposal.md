## Summary

把 `crates/cli/src/main.rs::print_human_ok` 從「flat key-value + nested JSON 直接 stringify」改成「遞迴 + 縮排 pretty-print」，讓人類使用者跑 `--json` 以外的指令時得到可讀的輸出。

## Motivation

`add-change-and-artifact-io` 端到端測試（2026-05-22）發現 `speclink show change <name>` non-JSON mode 對含 nested object/array 的 data 直接印 JSON-stringified 字串：

```
artifacts: [{"capability":null,"kind":"proposal"},{"capability":null,"kind":"tasks"},{"capability":"user-auth","kind":"spec"}]
change: {"changeId":"a9d53bbf-...","createdAt":"2026-05-22T06:52:55.980182Z","name":"demo-billing",...}
```

對直接在 terminal 跑的工程師體驗很差，且後續 slice 加新 op（`review history`、`discuss show` 等）nested 結構只會更深，每加一個 op 都會拖累 UX。

本 change 屬於 `cli` crate 內部的人類輸出層 polish；**不**影響 AI skill 路徑（skill 永遠用 `--json`），**不**影響任何 JSON envelope contract，**不**動既有的 spec scenario。改完後 `--json` mode 的行為（envelope shape、field order、exit code、error code）完全不變，只是 stdout 拿掉 `--json` 旗標時印得漂亮一點。

使用者情境：工程師本機操作（在 terminal 直接跑 `speclink` 看結果），不涉及 AI skill、CI、provider API。

## Proposed Solution

1. 在 `crates/cli/src/human.rs` 新增 `render_human(data: &serde_json::Value) -> String` 與底層 `render_value(value, indent)` 遞迴 helper：
   - `Object`：每個 key 印一行 `key: <value>`；若 value 為 nested object/array，先印 `key:` 加換行，nested 內容下一層縮排（2 個空白）
   - `Array`：每個 element 印一行 `- <value>`；若 element 為 nested object/array，下一層縮排
   - `String`：去掉外層引號（避免 `"foo"` 變成 `"\"foo\""`）；含換行的 string 加縮排換行
   - `Number` / `Bool` / `Null`：用 `to_string()`
   - 空 object `{}` 印 `OK`（沿用 bootstrap 既有「無內容回 OK」語意）
   - 空 array `[]` 印 `(empty)`
2. `crates/cli/src/main.rs::print_human_ok` 改為單行委派 `render_human` 後 println；移除既有 for-loop。
3. 新增 `crates/cli/src/human.rs::tests` inline 單元測試覆蓋：flat object / nested object / array of object / mixed / empty object / empty array / string with newline 七個 case，採 `insta::assert_snapshot!` 鎖輸出格式。

## Non-Goals

- ❌ 不改 `Envelope` / `SuccessBody` / `ErrorBody` / `Warning` / `ErrorDetail` 任何序列化形狀
- ❌ 不改任何 op 的 data shape（`changeId` / `artifactDir` / `etag` ... 等 key 與順序維持不變）
- ❌ 不引入 ANSI color、TTY detection、table formatter dependency（`prettytable` / `comfy-table` 等）
- ❌ 不引入新 CLI 旗標（`--pretty` / `--format`）
- ❌ 不調整 hint / error 的 stderr 輸出（沿用既有 `eprintln!("error[{code}]: ...")` 行為）
- ❌ 不動 `print_human_ok` 對「空 object」回 `OK` 的既有語意（bootstrap unlink 依賴）
- ❌ 不擴及 `cli` crate 以外的程式（runtime / provider / provider-local 完全不動）

## Alternatives Considered

| 方案 | 取捨 | 結論 |
|---|---|---|
| A. 引入 `comfy-table` 之類 dependency 做 table 渲染 | 美觀，但 dependency 體積大、跨平台 ANSI 行為複雜、改動範圍超出 polish | 駁回 |
| B. 加 `--pretty` / `--format text\|json` 旗標切換 | 多一個 surface 要維護，但目前只有兩種 mode（`--json` vs 預設 human），加 flag 過度設計 | 駁回 |
| C. 純遞迴 + 縮排（本 change 採用） | 純標準庫實作、不引入 dep、不擴 CLI surface、與 bootstrap 既有「OK」語意相容 | 採用 |
| D. 把 human mode 完全拿掉，永遠輸出 JSON | 與 bootstrap CLI 設計相違（人類預設 mode 不是 JSON）；違反設計上的「使用者體驗」 | 駁回 |

## Impact

- Affected crates：僅 `cli`，不跨 crate
- Affected specs：新增 `cli-human-output` capability（鎖住 human mode 的輸出格式契約，給未來 op 統一遵循）
- Affected code:
  - Modified: crates/cli/src/main.rs（`print_human_ok` 改為單行委派 `render_human`）
  - Modified: crates/cli/src/lib.rs（`pub mod human;`）
  - New: crates/cli/src/human.rs（`render_human` + `render_value` + inline `mod tests` snapshot tests）
- 對 AI skill 路徑：零影響（skill 一律 `--json`）
- 對 provider API contract：零影響
- 對 lifecycle state machine：零影響
- 對 auth / token / secret：零影響（human mode 從不接觸 secret）
- 對 fallback policy：零影響
- Snapshot 影響：`crates/cli/src/snapshots/` 與 `crates/cli/tests/snapshots/` 既有 `--json` snapshot 一個都不動；本 change 只新增 `human.rs::tests` 的 snapshot
- exit code 與 error code：完全不變

## Capabilities

### New Capabilities

- `cli-human-output`: 規範 `speclink` 在不帶 `--json` 旗標時的人類輸出格式契約（縮排規則、空 object 顯示、巢狀渲染）

### Modified Capabilities

(none)
