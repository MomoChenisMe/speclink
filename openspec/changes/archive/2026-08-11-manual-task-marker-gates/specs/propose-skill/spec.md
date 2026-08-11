## ADDED Requirements

### Requirement: 手動測試任務的起草標記

propose 技能文字 SHALL 指示 tasks 起草時對人工驗收/手動測試類任務(agent 無法自動執行、需使用者實際操作驗證的任務)加 `[M]` 前綴標記,使寫碼與人工驗收的完成度得以分開判讀;自動化測試與寫碼任務 SHALL NOT 加此標記。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: 起草含人工驗收的 tasks

- **WHEN** 提案含「開啟文件實際操作確認」類的人工驗收項,propose 流程產出 tasks.md
- **THEN** 該任務行帶 `[M]` 前綴,自動化測試與寫碼任務行不帶

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 propose 技能檔含 `[M]` 起草指引,與 golden 對照一致
