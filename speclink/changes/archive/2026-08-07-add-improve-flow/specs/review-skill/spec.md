## MODIFIED Requirements

### Requirement: 審查技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-review` 技能檔至 claude 與 codex 兩工具的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 將生成之 CLAUDE.md／AGENTS.md 的 workflow 行改為含模型發起入口、品質關卡與並行品質站的版本（`discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive`），並於技能使用清單加入審查站的觸發時機（實作完成、封存之前、由使用者判斷是否執行）。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** claude 與 codex 的技能目錄各出現 speclink-review 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行更新

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 `(quality? | review? ∥ verify?)` 且技能清單含審查站條目
