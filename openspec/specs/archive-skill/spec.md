# archive-skill Specification

## Purpose

/speclink-archive 技能的敘述內容：封存時 trace 與 evidence 如何運作的說明，以及在 linked worktree 環境下封存會被引擎拒絕、必須回主 checkout 執行的指示。本 capability 保證技能文字與引擎的實際行為一致，使用者不會照著技能跑出當場被擋下的動作。

## Requirements

### Requirement: trace 與 evidence 的技能敘述

內嵌 speclink-archive 技能（事實來源 crates/engine/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 敘明：@trace 僅含 source 與 updated、於 ADDED／MODIFIED 物化時一律注入、不含檔案清單。技能檔 SHALL NOT 要求 bulk 封存前工作樹整潔，SHALL NOT 指示刪除 evidence 記錄，SHALL NOT 敘述任何 evidence 守門、拒絕情形或放行旗標。技能檔 SHALL 敘明零證據提示行的意義：封存無任何任務證據記錄的 change 時 stderr 會出現一行提示，見到提示應確認該 change 是否漏走 apply 流程（純規格或文件變更的零證據屬正常）。渲染產物內容由 speclink-core 的 render_golden 測試保護，golden 快照更新屬刻意變更。

#### Scenario: 技能檔無刪除步驟、整潔要求與守門敘述殘留

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔全文
- **THEN** 不含刪除 evidence／touched 記錄的指示，不含 bulk 封存前工作樹必須整潔的要求，亦不含任何 evidence 守門拒絕或 waive-evidence 放行旗標的敘述

#### Scenario: trace 與提示敘述到位

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔對 @trace 與零證據提示的敘述
- **THEN** @trace 敘述為 source 與 updated 兩欄一律注入、無檔案清單；零證據提示段敘明提示出現的條件與應對（確認是否漏走 apply；純規格變更屬正常）

<!-- @trace
source: evidence-home-and-trace-slim
updated: 2026-08-04
-->

---
### Requirement: worktree 環境的技能敘述

內嵌 speclink-archive 技能（事實來源 crates/engine/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 敘明：封存於主 checkout 執行；於 linked worktree（speclink/ 分支）內執行封存會被引擎拒絕，應先以 worktree-merge 技能收尾合回主分支再封存。

#### Scenario: 技能檔含主 checkout 限定敘述

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔全文
- **THEN** 內文含「封存於主 checkout 執行」與「worktree 內封存會被引擎拒絕」的敘述，且指路 worktree-merge 技能

<!-- @trace
source: worktree-flow-guards-and-guidance
updated: 2026-08-06
-->

---
### Requirement: 封存完成後的收尾提交提醒

內嵌 speclink-archive 技能（事實來源 crates/engine/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 於結尾敘明：封存完成後提醒使用者以一般提交收尾本次封存產生的異動（delta 併入正式規格、變更目錄搬移至 archive）——commit 技能的變更選檔流程不適用於封存後，SHALL NOT 導向之。此提醒 SHALL 涵蓋所有進入封存的路徑（apply 直達、review、verify、quality、worktree-merge 之後），且 SHALL 明文為僅提醒——SHALL NOT 代跑提交。技能檔 SHALL 另敘明：工作區存在 openspec/manual/ 時，提醒使用者可跑 manual 技能檢查手冊是否因本次封存而過期——條件僅為該目錄存在（不判斷本次封存動到哪些規格），且 SHALL 明文為僅提醒、SHALL NOT 代跑 manual 技能。技能檔 SHALL 再敘明「下一個可開工」提示：封存完成後執行 speclink plan --json，`next` 非 null 時提一句「plan 的下一個可開工：<next>，執行 /speclink-apply <next>」；有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層）開啟且第 1 波（waves[0].changes）中 blockedBy 為空且未開工的 change 有兩個以上時，SHALL 列出該名單為可並行（各開一個 session 走 apply-with-worktree）；`next` 為 null 或 plan 失敗（依賴成環）時 SHALL NOT 提及順序。此提示 SHALL 明文為僅提醒、SHALL NOT 代跑 apply。

#### Scenario: 技能檔結尾含提交提醒

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段
- **THEN** 內文含封存完成後提醒提交的指示，且明文僅提醒、不代跑提交

#### Scenario: 手冊存在時的過期檢查提醒

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段對手冊的敘述
- **THEN** 內文含「工作區有 openspec/manual/ 時建議跑 manual 技能檢查手冊是否過期」，條件為目錄存在，且明文僅提醒、不代跑

#### Scenario: 封存後提示下一個可開工

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段對 plan 的敘述
- **THEN** 內文含執行 speclink plan --json、next 非 null 時提「下一個可開工：<next>」、worktree 政策開啟且第 1 波兩個以上可開工時列可並行名單、next 為 null 或 plan 失敗時不提的指示，且明文僅提醒、不代跑

##### Example: 提示的分岔

| plan 結果 | worktree 政策 | 提示 |
| --- | --- | --- |
| next = add-a，第 1 波可開工只有 add-a | 開或關 | 「下一個可開工：add-a，執行 /speclink-apply add-a」 |
| next = add-a，第 1 波可開工為 add-a、add-b | 開 | 上句加「add-a、add-b 可並行，各開 session 走 apply-with-worktree」 |
| next = add-a，第 1 波可開工為 add-a、add-b | 關 | 只提 add-a |
| next = null | 開或關 | 不提順序 |
| plan 回成環錯誤 | 開或關 | 不提順序 |


<!-- @trace
source: add-plan-handoff-after-archive
updated: 2026-09-16T16:37:51+08:00
-->

---
### Requirement: 未指名時的候選清單依 plan 順序

內嵌 speclink-archive 技能 SHALL 於未指名 change 時以 speclink plan --json 的 `changes` 陣列列出候選（依配置順序），每個候選 SHALL 標出其 `blockedBy`（空時標「無阻擋」）；SHALL 仍由使用者選擇、SHALL NOT 自動選。plan 失敗（依賴成環）時 SHALL 退回 speclink list --json 列候選。其他技能（commit、review、verify、drift、analyze）的候選清單 SHALL 維持 list --json，不受本需求影響。

#### Scenario: 候選依 plan 順序並標阻擋

- **WHEN** 未指名執行 speclink-archive 技能，plan 的 changes 依序為 add-a（blockedBy 空）、add-b（blockedBy ["add-a"]）
- **THEN** 候選清單依 add-a、add-b 順序呈現，add-a 標「無阻擋」、add-b 標「等 add-a」，並由使用者選擇

#### Scenario: plan 失敗退回 list

- **WHEN** 未指名執行 speclink-archive 技能且 plan 回依賴成環錯誤
- **THEN** 候選清單改以 speclink list --json 的順序呈現，不標阻擋

<!-- @trace
source: add-plan-handoff-after-archive
updated: 2026-09-16T16:37:51+08:00
-->