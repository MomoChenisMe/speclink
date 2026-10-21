## ADDED Requirements

### Requirement: 新需求改到承諾行為時切回 spec-driven

ingest 技能 SHALL 在更新 artifact 之前讀取 `speclink status --change <name> --json` 的 `schemaName`。`schemaName` 為 `no-spec-change`，且帶入的新需求會新增、修改或移除正式規格承諾的行為時，技能 SHALL 先執行 `speclink change schema <name> spec-driven`，成功後在 proposal 補上 Capabilities 段、以 `speclink new artifact spec` 寫出 delta spec，再依新需求更新 design 與 tasks；切換被拒（例如有未結的驗證工單）時，技能 SHALL 回報 stderr 原因並停止，不得繞過切換直接寫 delta。帶入的新需求不改承諾的行為時，技能 SHALL 保持 schema 不變。技能 SHALL NOT 自行把 spec-driven 變更切到 no-spec-change；只有使用者明確要求、且變更已沒有 delta spec 時才執行。

#### Scenario: 不改規格變更帶入改行為的需求

- **WHEN** 對 schema 為 no-spec-change 的變更 tidy 執行 ingest，新需求要求 `speclink list --json` 多一個欄位，而正式規格承諾了該輸出的形狀
- **THEN** 技能先執行 `speclink change schema tidy spec-driven`，再在 proposal 補 Capabilities、寫出對應的 delta spec 並更新 tasks，摘要說明 schema 已切換及原因

#### Scenario: 切換被拒時停止

- **WHEN** 同上情境，但 tidy 有未結的驗證工單，`speclink change schema` 以非零 exit code 拒絕
- **THEN** 技能回報 stderr 的原因與 `speclink verify discard tidy` 的提示並停止，沒有寫入任何 delta spec

#### Scenario: 不改行為的需求維持 schema

- **WHEN** 對 schema 為 no-spec-change 的變更執行 ingest，新需求只是再多搬一個內部函式
- **THEN** 技能不執行 `speclink change schema`，只更新 design 與 tasks
