## MODIFIED Requirements

### Requirement: 先封存子流程的順序提示與收尾提醒

<!-- BEFORE: 封存前提示讀 blockedBy，字面「plan 建議先封存 <blockedBy>」 -->

內嵌 speclink-commit 技能（事實來源 crates/engine/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至工具技能目錄）的「先封存再一起提交」子流程 SHALL 在執行 speclink archive 之前執行 speclink plan --json：目標 change 的 `archiveAfter` 非空時 SHALL 提醒「plan 建議先封存 <archiveAfter 的名稱>；它們封存後，重讀本 change 對同名 requirement 的區塊、對照正式規格重寫（走 ingest）再封存 <name>」；`requirementOverlap` 含 `conflict` 為 true 者時 SHALL 提醒兩邊都新增（或改名成）、或都移除（或改名掉）同名 requirement，要先改掉其中一邊；`blockedBy` 非空時 SHALL 另提「宣告前置 <名稱> 尚未封存」；三者皆為僅建議——SHALL NOT 阻擋、使用者確認後照常封存，SHALL NOT 寫「跑 drift 即可」；三者皆空、目標不在 plan 內或 plan 失敗時 SHALL NOT 提及順序。子流程 SHALL 同時記下 `archiveAfter` 含目標 change 的其他 change。子流程封存成功後 SHALL 敘明三件事：工作區存在 openspec/manual/ 時提醒可跑 manual 技能檢查手冊是否過期（條件僅為目錄存在，僅提醒、SHALL NOT 代跑）；封存前記下的 change 非空時提醒它們先對照正式規格重寫（走 ingest）再封存（僅提醒、SHALL NOT 單獨觸發詢問、SHALL NOT 代跑 ingest）；再執行 speclink plan --json，`next` 非 null 時提「plan 的下一個可開工：<next>，執行 /speclink-apply <next>」，有效 worktree 政策開啟且第 1 波兩個以上可開工時列可並行名單，`next` 為 null 或 plan 失敗時不提；皆僅提醒、SHALL NOT 代跑。

#### Scenario: 子流程封存前的順序提示

- **WHEN** 使用者於 commit 技能選擇「先封存再一起提交」，目標 change 在 plan 中的 archiveAfter 為 ["add-a"]
- **THEN** 技能檔指示在執行 speclink archive 前提醒「plan 建議先封存 add-a；add-a 封存後對照正式規格重寫再封存 <name>」，使用者確認後照常封存

#### Scenario: 無阻擋不提

- **WHEN** 目標 change 的 archiveAfter、blockedBy 皆為空且無 conflict
- **THEN** 技能檔指示不提及順序，直接進入封存

#### Scenario: 子流程封存後的收尾提醒

- **WHEN** 子流程封存成功、工作區存在 openspec/manual/、封存前 add-c 的 archiveAfter 含目標 change、plan 的 next 為 add-b
- **THEN** 技能檔指示提醒手冊可能過期，提醒 add-c 先對照正式規格重寫再封存，並提「下一個可開工：add-b，執行 /speclink-apply add-b」，三者皆僅提醒
