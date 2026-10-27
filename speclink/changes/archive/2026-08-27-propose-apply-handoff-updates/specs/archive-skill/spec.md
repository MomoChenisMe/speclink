## ADDED Requirements

### Requirement: 封存完成後的收尾提交提醒

內嵌 speclink-archive 技能（事實來源 crates/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 於結尾敘明：封存完成後提醒使用者以一般提交收尾本次封存產生的異動（delta 併入正典規格、變更目錄搬移至 archive）——commit 技能的變更選檔流程不適用於封存後，SHALL NOT 導向之。此提醒 SHALL 涵蓋所有進入封存的路徑（apply 直達、review、verify、quality、worktree-merge 之後），且 SHALL 明文為僅提醒——SHALL NOT 代跑提交。

#### Scenario: 技能檔結尾含提交提醒

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段
- **THEN** 內文含封存完成後提醒提交的指示，且明文僅提醒、不代跑提交
