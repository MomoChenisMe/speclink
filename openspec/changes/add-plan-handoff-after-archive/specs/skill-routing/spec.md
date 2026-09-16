## MODIFIED Requirements

### Requirement: 出口交棒由技能結尾承載

流程鏈技能（propose、apply、apply-with-worktree、worktree-merge、drift、ingest、review、verify、quality、baseline、discuss、improve）的資產 SHALL 以下一步建議段收尾：依該次執行的結束狀態列出建議的後續技能，且 SHALL 明文為僅建議——SHALL NOT 自動呼叫任何後續技能。archive 為流程終點，其資產 SHALL 於結尾帶一條收尾提交提醒（建議提交封存產生的異動，僅提醒、SHALL NOT 代跑），SHALL 於工作區存在 openspec/manual/ 時另帶一條手冊過期檢查提醒（建議跑 manual 技能檢查手冊是否過期，僅提醒、SHALL NOT 代跑），並 SHALL 帶一條「下一個可開工」提示（以 plan 的 next 建議 apply 哪個 change、worktree 政策開啟時列第 1 波可並行名單；next 為 null 或 plan 失敗時不提；僅提醒、SHALL NOT 代跑）；工具技能（commit、analyze、audit、config、trace、manual）隨叫隨用，SHALL NOT 被要求帶固定出邊——唯 commit 的「先封存再一起提交」子流程既是封存入口，SHALL 帶與 archive 同一組封存前順序提示與封存後提醒（見 commit-skill 能力）。

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

#### Scenario: archive 與 commit 子流程的下一個可開工提示

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔結尾段與 speclink-commit 技能檔的先封存子流程
- **THEN** 兩者皆含「封存後以 plan 的 next 提示下一個可開工的 change」的敘述，且明文僅提醒、不代跑

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
| archive | 封存完成 | 提醒提交封存異動；有 openspec/manual/ 時另提醒可跑 manual 檢查過期；plan 的 next 非 null 時提「下一個可開工」（皆僅提醒） |
| commit（先封存子流程） | 封存並提交完 | 有 openspec/manual/ 時提醒可跑 manual；plan 的 next 非 null 時提「下一個可開工」（皆僅提醒） |
| discuss | 已寫結論且值得開變更 | propose 的 --from-discussion 入口（promote 留給中途轉出） |
