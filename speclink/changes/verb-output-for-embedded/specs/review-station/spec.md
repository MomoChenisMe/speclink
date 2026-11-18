## MODIFIED Requirements

### Requirement: CLI 清單輸出的相容性釘住
<!-- BEFORE: 任何 `speclink list --json` 的輸出都不得因 metadata 含 reviewed 欄位而改變形狀；沒有 `--detail` 這個旗標 -->

不帶 `--detail` 的 `speclink list --json` 的輸出 SHALL 不因 metadata 含任何 reviewed 欄位而改變形狀：含全套 reviewed 欄位的 change 與不含者 SHALL 序列化出同形（位元級同構的欄位集合）。審查狀態 SHALL 只在使用者明確帶上 `--detail` 時進入清單輸出（見 verb-contract「清單的詳細輸出」）。

#### Scenario: 帶章的 change 不外洩新欄位

- **WHEN** 某 change 的 `.meta.yaml` 含全套 reviewed 欄位時執行 `speclink list --json`
- **THEN** 該 change 的 JSON 項目與無 reviewed 欄位的 change 具有相同的欄位集合

#### Scenario: 帶 --detail 時才出現審查狀態

- **WHEN** 同一個 change 分別以 `speclink list --json` 與 `speclink list --json --detail` 列出
- **THEN** 前者的項目沒有 `review` 鍵，輸出與引入 `--detail` 之前逐位元相同；後者的項目有 `review` 鍵
