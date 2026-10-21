## ADDED Requirements

### Requirement: 不改規格變更的行為檢查

verify 技能 SHALL 在 `speclink status --change <name> --json` 的 `schemaName` 為 `no-spec-change` 時，以「行為檢查」取代以 delta spec 為準的規格涵蓋與需求對應：從 proposal Why 列出的正式規格出發，再以凍結 patch 裡的具體名稱（指令、旗標、`--json` 欄位、檔名、畫面文案）搜尋涵蓋這些程式區域的正式規格需求，逐一比對 patch 是否改變了需求承諾的可觀察行為。patch 改變了任一承諾的行為時，技能 SHALL 回報 WARNING `Behavior change in a no-spec-change change: <capability> / <requirement>`，建議改用 spec-driven 並為該需求補 delta spec；這條 WARNING 依既有規則屬必修、擋乾淨蓋章。沒有發現行為改變時，驗證報告 SHALL 列出比對過的正式規格。任務完成度與設計遵循的檢查照常執行。`schemaName` 不是 `no-spec-change` 的變更 SHALL 維持現行驗證流程。

#### Scenario: 改到承諾的輸出時回報 WARNING

- **WHEN** 對 no-spec-change 變更執行 verify，凍結 patch 改變了 `speclink list --json` 的一個欄位名稱，而正式規格 cli 相關需求承諾了該欄位
- **THEN** 驗證工單的回合含一條 WARNING `Behavior change in a no-spec-change change: <capability> / <requirement>`，建議改用 spec-driven 並補 delta；`speclink verify stamp` 在未帶 `--accept` 時拒絕蓋章

#### Scenario: 純重構通過並列出比對過的規格

- **WHEN** 對 no-spec-change 變更執行 verify，凍結 patch 只搬移函式與改名內部變數，任務全數完成
- **THEN** 驗證工單沒有行為改變的 finding，報告列出比對過的正式規格名稱

#### Scenario: spec-driven 變更維持現行流程

- **WHEN** 對 schema 為 spec-driven 的變更執行 verify
- **THEN** 技能照現行流程依 delta spec 檢查規格涵蓋、需求對應與情境涵蓋，不執行行為檢查
