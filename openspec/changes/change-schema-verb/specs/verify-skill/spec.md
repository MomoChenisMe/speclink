## ADDED Requirements

### Requirement: 不改規格變更改到行為時的修正步驟

verify 技能對 `no-spec-change` 變更回報 `Behavior change in a no-spec-change change: <capability> / <requirement>` 的 WARNING 時，該 finding 的建議 SHALL 依序寫出修正步驟：執行 `speclink verify discard <name>` 放棄這張驗證工單、執行 `speclink change schema <name> spec-driven` 切回 spec-driven、以 `/speclink-ingest <name>` 補上該需求的 delta spec、再重新執行 verify。驗證報告的下一步 SHALL 指向這組步驟，SHALL NOT 建議以 `--accept` 帶過這條 WARNING。

#### Scenario: 行為改變的建議列出修正步驟

- **WHEN** 對 no-spec-change 變更 tidy 執行 verify，凍結 patch 改變了正式規格承諾的一個輸出欄位
- **THEN** 驗證工單的 WARNING 建議依序含 `speclink verify discard tidy`、`speclink change schema tidy spec-driven`、`/speclink-ingest tidy` 與重新 verify，報告沒有建議使用 `--accept`

#### Scenario: 沒有行為改變時不出現修正步驟

- **WHEN** 對 no-spec-change 變更執行 verify，凍結 patch 沒有改到任何承諾的行為
- **THEN** 驗證報告沒有切換 schema 的建議
