## ADDED Requirements

### Requirement: worktree 環境的技能敘述

內嵌 speclink-archive 技能（事實來源 crates/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 敘明：封存於主 checkout 執行；於 linked worktree（speclink/ 分支）內執行封存會被引擎拒絕，應先以 worktree-merge 技能收尾合回主分支再封存。

#### Scenario: 技能檔含主 checkout 限定敘述

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔全文
- **THEN** 內文含「封存於主 checkout 執行」與「worktree 內封存會被引擎拒絕」的敘述，且指路 worktree-merge 技能
