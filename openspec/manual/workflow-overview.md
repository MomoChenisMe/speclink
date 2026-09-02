---
title: 工作流總覽：站別與交棒
section: 開始使用
order: 50
keywords: [工作流, 技能, 交棒, 下一步, SDD, 站別, baseline, 手冊]
sources: [skill-routing, user-documentation]
generated: 2026-09-02
---

# 工作流總覽：站別與交棒

Speclink 的工作流由一組 AI 技能串起來。每個技能就是一站：你在某個情境呼叫它，它做完事，結尾告訴你下一站建議去哪。沒有集中式的流程總表，路由就寫在技能自己身上。這一頁把全部的站攤開，並列出交棒的邊。

## 兩件事先分清楚

- 技能是工作流知識，CLI 與 Host 是執行引擎。技能告訴 agent 該做什麼、怎麼判斷；真正改檔案的是 `speclink` 動詞。
- 技能只會「建議」下一步，不會自動呼叫另一個技能。要不要走下一站，由你決定。

## 每一站的入口情境

每個技能的描述都先寫「你在什麼情境用它」。全部入口情境如下：

| 情境 | 技能 |
| --- | --- |
| 需求模糊、要辯論 | `/speclink-discuss` |
| 沒有指定題目的改進掃描 | `/speclink-improve` |
| 規劃與提案 | `/speclink-propose` |
| 既有專案採用 Speclink、還沒有規格 | `/speclink-baseline`（舊稱 onboard） |
| 任務實作、或恢復做到一半的變更 | `/speclink-apply` |
| 幾個變更要平行實作 | `/speclink-apply-with-worktree` |
| worktree 的變更要合回主分支 | `/speclink-worktree-merge` |
| 閒置變更恢復前的漂移檢查 | `/speclink-drift` |
| 需求中途變更 | `/speclink-ingest` |
| 工藝品質檢查 | `/speclink-review` |
| 規格符合檢查 | `/speclink-verify` |
| 兩站合跑 | `/speclink-quality` |
| 封存 | `/speclink-archive` |
| 只提交某個變更的檔案 | `/speclink-commit` |
| 產物一致性檢查 | `/speclink-analyze` |
| 安全稽核 | `/speclink-audit` |
| 組建工作流設定 | `/speclink-config` |
| 功能溯源 | `/speclink-trace` |
| 需要一份人類操作手冊、或想被導覽怎麼操作系統 | `/speclink-manual` |

這些站分三類：

- 必經的生命週期階段：propose → apply → archive。
- 條件式階段：discuss、improve、baseline、drift、ingest、review、verify、quality、worktree 流程。看情況才走。
- 工具技能：commit、analyze、audit、config、trace、manual。隨叫隨用，沒有固定的下一站。audit 是安全檢查，commit 是限定某個變更檔案的 Git 工具；兩者都不是每個變更必經的步驟。

> [!NOTE]
> apply、drift、ingest、analyze、audit 這五個技能的內文行為沒有各自的規格，本手冊只寫路由層面的資訊。其他技能各有一頁。

## 從哪裡開始

- 需求已經明確：直接 `/speclink-propose`。
- 需求還要取捨：先 `/speclink-discuss`。
- 只是想理解問題、沒有待決事項：直接問答就好，不用建立討論記錄。
- 既有專案、還沒有規格：先 `/speclink-baseline` 建立規格基準，見[基準盤點：既有專案採用 Speclink](baseline.md)。
- 要恢復一個閒置的變更：先 `/speclink-drift`，再回 apply。
- 實作途中收到會改產物的新需求：走 `/speclink-ingest`。
- 想先讀一份操作手冊、或被導覽一遍：`/speclink-manual`，見[操作手冊：生成與導覽](manual.md)。

## 交棒邊表

每個流程鏈技能結尾都有「下一步建議」段。建議依該次執行的結束狀態而不同：

| 技能 | 結束狀態 | 建議下一步 |
| --- | --- | --- |
| baseline | 初始規格生成完 | 需求清楚→propose；還模糊→discuss |
| discuss | 已寫結論且值得開變更 | propose 的 `--from-discussion` 入口 |
| propose | 產物齊備 | apply。提案中變更有 2 個以上時，先盤點執行順序 |
| apply | 全部勾完 | 品質關卡（review、verify 或 quality）或 archive。commit 技能的「先封存再一起提交」可以一步到位 |
| apply | 需求中途變更 | ingest |
| apply-with-worktree | worktree 內 commit 完 | 品質關卡（在 worktree 內）→ worktree-merge |
| worktree-merge | 合併清理完 | 回主 checkout 執行 archive |
| drift | 假設過期 | ingest |
| drift | 無漂移 | apply |
| ingest | 產物更新完 | 回 apply |
| review、verify | 落章 | archive。在 worktree 內則先提交蓋章寫入的異動，再 worktree-merge |
| quality | 兩站落章 | archive。在 worktree 內則 worktree-merge |
| archive | 封存完成 | 提醒你提交封存產生的異動；工作區有 `openspec/manual/` 時另提醒可跑 manual 檢查手冊是否過期。都只提醒，不代跑 |

apply 完成時如果還剩手動任務，建議會說品質關卡可以先跑，封存要等手動任務完成。

## 提案時只產必要的產物

propose 只需要完成 apply 所需鏈上的必要產物。design 是選用產物，不符合建立條件時可以跳過。不要預期每個變更固定產出四份產物。

## 品質關卡蓋章之後

review 與 verify 蓋章時，會在同一個寫入裡寫下章欄位並刪除工單檔（review.md 與 verify.md）。所以：

- 已蓋章的變更封存後，封存目錄裡沒有工單檔。
- 只有未結的工單能經 carry 旗標隨封存移動。
- 本地模式的工單文字只留在 git 歷史；remote 模式蓋章後工單文字讀不回來。

蓋章後執行 show 回報「無工單」是預期行為。細節見[品質關卡總覽](quality-stations.md)。

## 各站的頁面

1. [基準盤點：既有專案採用 Speclink](baseline.md)
2. [討論：需求還模糊時](discuss.md)
3. [提案：建立變更與產物](propose.md)
4. [實作：完成任務](apply.md)
5. [續作與需求變更：drift 與 ingest](drift-ingest.md)
6. [品質關卡總覽](quality-stations.md)、[審查站](review.md)、[驗證站](verify.md)
7. [封存](archive.md)
8. [提交單一變更的檔案](commit.md)
9. [平行實作與合回：worktree](worktree.md)
10. [溯源：一個功能怎麼來的](trace.md)
11. [工作流政策與設定](policy-config.md)
12. [產出流程 schema 管理](schemas.md)
13. [操作手冊：生成與導覽](manual.md)

**出處**：`skill-routing`、`user-documentation`
