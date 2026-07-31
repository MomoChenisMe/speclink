## ADDED Requirements

### Requirement: 工具檔生成不寫入 AI 工具的使用者設定檔

工具檔生成（init、update、tools 收斂、工作區補齊 adopt 的所有路徑）SHALL NOT 建立或改寫 AI 工具的使用者設定檔（`.claude/settings.json`）。受管生成物 SHALL 僅限技能檔、指令檔的 SPECLINK marker 區塊、`.speclink.yaml` 與 `.gitignore` 的 `.speclink/` 條目。既有的使用者設定檔 SHALL 視為使用者資料：任何工具同步後 SHALL 位元級不變，清理（prune）SHALL NOT 移除它。

#### Scenario: init 不產生使用者設定檔

- **WHEN** 對全新目錄以 tools=[claude] 執行 init
- **THEN** `.claude/skills/` 技能檔與 CLAUDE.md marker 區塊照常生成，`.claude/settings.json` 不存在

#### Scenario: 既有使用者設定檔在工具同步後位元級不變

- **WHEN** 專案的 `.claude/settings.json` 含使用者自訂內容（如外掛啟用清單），執行 speclink update
- **THEN** 該檔位元級不變，其餘受管檔照常再生

##### Example: 自訂外掛設定不被清空

- **GIVEN** `.claude/settings.json` 內容為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`
- **WHEN** 執行 speclink update
- **THEN** 該檔內容仍為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`，位元級相同

#### Scenario: 工作區補齊不產生使用者設定檔

- **WHEN** 對「已有 openspec/ 但無 .speclink.yaml」的目錄以 tools=[claude] 執行工作區補齊（adopt）
- **THEN** 工作區檔照常補齊（.speclink.yaml、技能檔、marker 區塊、.gitignore 條目），`.claude/settings.json` 不存在
