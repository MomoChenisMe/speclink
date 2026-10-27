## MODIFIED Requirements

### Requirement: 入口路由由技能描述承載
<!-- BEFORE: 入口情境聯集不含手冊生成與導覽；本次加入 manual 一項。另將既有文字中的 onboard 對齊為 baseline（技能已由 rename-onboard-to-baseline 改名，正典文字尚未跟上） -->

每個生成為 SKILL.md 的對外技能，其 registry description SHALL 為觸發情境句：先陳述「使用者處於什麼情境時用此技能」，再以一句話說明產出。全體對外技能 description 的觸發情境聯集 SHALL 涵蓋工作流程的所有入口情境：需求模糊要辯論（discuss）、未指定題目的改進掃描（improve）、規劃與提案（propose）、既有專案採用（baseline）、任務實作與恢復（apply）、平行實作（apply-with-worktree）、worktree 合回（worktree-merge）、閒置變更恢復前的漂移檢查（drift）、需求中途變更（ingest）、工藝品質檢查（review）、規格符合檢查（verify)、兩站合跑（quality）、封存（archive）、變更範圍提交（commit）、artifact 一致性檢查（analyze）、安全稽核（audit）、workflow 設定組建（config）、功能溯源（trace）、需要人類操作手冊或想被導覽如何操作系統（manual）。host 載入的技能清單即為唯一入口路由表，引擎 SHALL NOT 於其他載體重複維護入口路由。

#### Scenario: 對外技能描述以觸發情境開場

- **WHEN** 檢視生成的 .claude/skills/speclink-apply/SKILL.md frontmatter 的 description
- **THEN** 句子先陳述觸發情境（任務就緒要實作、或恢復做到一半的變更），再說明技能產出；不是僅有動詞的一句話

#### Scenario: 觸發情境聯集涵蓋全部入口

- **WHEN** 逐一檢視全部對外技能的 description
- **THEN** 上列每個入口情境至少被一個技能的 description 涵蓋（含 manual 的手冊與導覽情境），無入口情境只存在於技能清單以外的載體

### Requirement: 出口交棒由技能結尾承載
<!-- BEFORE: 工具技能清單不含 manual；archive 結尾只帶收尾提交提醒一條。流程鏈清單與邊集表中的 onboard 對齊為 baseline（同上） -->

流程鏈技能（propose、apply、apply-with-worktree、worktree-merge、drift、ingest、review、verify、quality、baseline、discuss、improve）的資產 SHALL 以下一步建議段收尾：依該次執行的結束狀態列出建議的後續技能，且 SHALL 明文為僅建議——SHALL NOT 自動呼叫任何後續技能。archive 為流程終點，其資產 SHALL 於結尾帶一條收尾提交提醒（建議提交封存產生的異動，僅提醒、SHALL NOT 代跑），並 SHALL 於工作區存在 openspec/manual/ 時另帶一條手冊過期檢查提醒（建議跑 manual 技能檢查手冊是否過期，僅提醒、SHALL NOT 代跑）；工具技能（commit、analyze、audit、config、trace、manual）隨叫隨用，SHALL NOT 被要求帶固定出邊。

#### Scenario: apply 完成時的交棒句

- **WHEN** apply 技能把全部非手動任務勾完並輸出完成摘要
- **THEN** 摘要建議品質站（review、verify 或 quality）可跑、或直接封存——直接封存的路徑明示走 archive 技能、或以 commit 技能的「先封存再一起提交」一步到位；若剩手動任務則載明品質站可先跑而封存等手動完成；全程不自動呼叫任何技能

#### Scenario: 品質站落章後的交棒句

- **WHEN** review 或 verify 技能單站蓋章完成
- **THEN** 主 checkout 建議 archive；worktree 內建議先提交蓋章寫入的 meta 異動、再走 worktree-merge；不建議另一站

#### Scenario: 交棒句依狀態分岔

- **WHEN** drift 技能檢出 delta 假設已過期
- **THEN** 建議走 ingest；檢出無漂移時建議回 apply 繼續；兩種情況都只建議、不代跑

#### Scenario: archive 在手冊存在時的提醒句

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段
- **THEN** 除收尾提交提醒外，另含「工作區有 openspec/manual/ 時建議跑 manual 技能檢查手冊是否過期」的敘述，且明文僅提醒、不代跑

##### Example: 交棒句邊集

| 技能 | 結束狀態 | 建議下一步 |
| --- | --- | --- |
| baseline | 初始規格生成完 | 需求清楚→propose；還模糊→discuss |
| propose | artifacts 齊備 | apply；提案中變更 ≥2 時先盤點執行順序（worktree 政策開啟→分可平行／須依序，平行者各開 session 走 apply-with-worktree） |
| apply | 全部勾完 | 品質站或 archive（commit 的先封存再一起提交可一步到位） |
| apply | 需求中途變更 | ingest |
| apply-with-worktree | worktree 內 commit 完 | 品質站（worktree 內）→worktree-merge |
| worktree-merge | 合併清理完 | 回主 checkout archive |
| drift | 假設過期／無漂移 | ingest／apply |
| ingest | artifacts 更新完 | 回 apply |
| review、verify | 落章 | archive（worktree 內→補提交後 worktree-merge） |
| quality | 兩站落章 | archive（worktree 內→worktree-merge） |
| archive | 封存完成 | 提醒提交封存異動；有 openspec/manual/ 時另提醒可跑 manual 檢查過期（皆僅提醒） |
| discuss | 已寫結論且值得開變更 | propose 的 --from-discussion 入口（promote 留給中途轉出） |
