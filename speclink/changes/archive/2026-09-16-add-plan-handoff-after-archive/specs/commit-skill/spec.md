## ADDED Requirements

### Requirement: 先封存子流程的順序提示與收尾提醒

內嵌 speclink-commit 技能（事實來源 crates/engine/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至工具技能目錄）的「先封存再一起提交」子流程 SHALL 在執行 speclink archive 之前執行 speclink plan --json：目標 change 的 `blockedBy` 非空時 SHALL 提醒「plan 建議先封存 <blockedBy 的名稱>，再封存 <name>」，此為僅建議——SHALL NOT 阻擋、使用者確認後照常封存；`blockedBy` 為空、目標不在 plan 內或 plan 失敗時 SHALL NOT 提及順序。子流程封存成功後 SHALL 敘明兩件事：工作區存在 openspec/manual/ 時提醒可跑 manual 技能檢查手冊是否過期（條件僅為目錄存在，僅提醒、SHALL NOT 代跑）；再執行 speclink plan --json，`next` 非 null 時提「plan 的下一個可開工：<next>，執行 /speclink-apply <next>」，有效 worktree 政策開啟且第 1 波兩個以上可開工時列可並行名單，`next` 為 null 或 plan 失敗時不提；皆僅提醒、SHALL NOT 代跑。

#### Scenario: 子流程封存前的順序提示

- **WHEN** 使用者於 commit 技能選擇「先封存再一起提交」，目標 change 在 plan 中的 blockedBy 為 ["add-a"]
- **THEN** 技能檔指示在執行 speclink archive 前提醒「plan 建議先封存 add-a，再封存 <name>」，使用者確認後照常封存

#### Scenario: 無阻擋不提

- **WHEN** 目標 change 的 blockedBy 為空
- **THEN** 技能檔指示不提及順序，直接進入封存

#### Scenario: 子流程封存後的收尾提醒

- **WHEN** 子流程封存成功、工作區存在 openspec/manual/、plan 的 next 為 add-b
- **THEN** 技能檔指示提醒手冊可能過期，並提「下一個可開工：add-b，執行 /speclink-apply add-b」，兩者皆僅提醒
