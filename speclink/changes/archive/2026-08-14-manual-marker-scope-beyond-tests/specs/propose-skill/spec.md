## ADDED Requirements

### Requirement: 手動任務的起草標記

propose 技能文字 SHALL 指示 tasks 起草時對手動任務(agent 無法代行、需使用者親手操作的任務——人工驗收、建立外部服務帳號、放置金鑰等,不限於測試)加 `[M]` 前綴標記,使寫碼與人工操作的完成度得以分開判讀;agent 做得到的任務(寫碼與自動化測試)SHALL NOT 加此標記。指引 SHALL 以正誤例並列(對比對)呈現:正例(`[M]` 緊接 checkbox、編號在後)與誤例(編號在前)並列,附後果說明(引擎不認、任務被算成寫碼任務而卡住完成度)與「checkbox 後恰一個空格」規則,SHALL NOT 僅給正例。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: 起草含人工操作的 tasks

- **WHEN** 提案含「開啟文件實際操作確認」類的人工驗收項與「至外部服務建立帳號」類的人工前置項,propose 流程產出 tasks.md
- **THEN** 該等任務行皆帶 `[M]` 前綴且緊接 checkbox(編號在標記之後),agent 可自行執行的寫碼與自動化測試行不帶

#### Scenario: 對比對指引呈現

- **WHEN** 閱讀 propose 技能檔的 `[M]` 起草指引
- **THEN** 正例與誤例並列可見,並載明誤寫後果與 checkbox 後恰一個空格的規則

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 propose 技能檔含 `[M]` 對比對起草指引,與 golden 對照一致

## REMOVED Requirements

### Requirement: 手動測試任務的起草標記

**Reason**: 需求名與定義文(「人工驗收/手動測試類任務…需使用者實際操作驗證」)綁死測試與驗證語意,與放寬後的 `[M]` 語意直接矛盾,隨改名重寫。
**Migration**: 由本 delta 新增的「手動任務的起草標記」承接——對比對呈現、位置規則與模板生成規範不變,僅標記語意的定義放寬。
