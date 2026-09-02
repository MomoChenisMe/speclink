---
title: 封存
section: SDD 工作流
order: 170
keywords: [封存, archive, 正典規格, 守門, Purpose, 證據]
sources: [archive-skill, archive-merge, change-lifecycle, verify-evidence, spec-validation]
generated: 2026-09-02
---

# 封存

封存是流程的終點：把變更裡的 delta 規格併進正典規格，然後把變更目錄搬進封存區。技能是 `/speclink-archive`，指令是 `speclink archive <變更名>`。封存後，正典規格就是現況的唯一真相。

## 在哪裡執行

封存在主 checkout 執行。在 linked worktree（分支名以 `speclink/` 開頭）裡執行封存，引擎會拒絕，並指路先用 worktree-merge 技能合回主分支再封存。見 [平行實作與合回](worktree.md)。

## 封存前的守門

封存會依序過幾道守門。任何一道擋下，都是零檔案效果：正典不動、沒有新的快照、變更目錄不搬。

### 1. metadata 完整

變更的 .openspec.yaml 存在但解析不了時，封存拒絕並指出檔案位置與原因。

### 2. 不在 linked worktree 內

工作區的 .git 是檔案（linked worktree 的特徵），且目前分支以 `speclink/` 開頭時拒絕。stderr 說明封存不得在 linked worktree 內執行，並指路 worktree-merge。git 不可用、分支名取不到、或分支不以 `speclink/` 開頭時放行。

### 3. 任務完成度

任務總數大於零、完成數小於總數時拒絕。stderr 列出完成數／總數，並給兩條出路：完成任務後再封存，或帶 `--mark-tasks-complete`。帶這個旗標時，引擎先把 tasks.md 全部勾選再封存。

| 任務總數 | 完成數 | 帶 `--mark-tasks-complete` | 結果 |
| --- | --- | --- | --- |
| 3 | 1 | 否 | 拒絕 |
| 3 | 1 | 是 | 先全勾再封存 |
| 3 | 3 | 否 | 照常封存 |
| 0 | 0 | 否 | 照常封存 |

這道守門在引擎本體生效，CLI、桌面 app、server 通道一體適用。桌面上對任務未完成的變更觸發封存，會收到引擎的拒絕訊息，變更留在看板。

### 4. 沒有未結的工單

變更目錄還有 review.md 或 verify.md 時拒絕，並列出處置：完成蓋章、放棄該站、或帶 `--carry-review`／`--carry-verify` 把工單一起搬進封存區。見 [品質關卡總覽](quality-stations.md)。

### 5. 章沒有失效

變更有審查章或驗證章，且章判為過期時拒絕。stderr 點名過期的站別與原因（內容錨列出第一個不符的檔案；任務錨說明計數），並指路重跑該站技能後再封存。兩章都過期時並列點名。

| 章的狀態 | 蓋章後的變動 | 結果 |
| --- | --- | --- |
| 審查章齊備 | 審查面某檔內容改變 | 拒絕，點名審查站 |
| 兩章齊備 | 補勾 `[M]` 任務 | 放行 |
| 兩章齊備 | 新增一個任務 | 拒絕，任務錨破 |
| 無章 | 任意 | 放行 |
| 章欄位不全 | 任意 | 放行 |

任務未完成與章失效同時發生時，先報任務完成度的訊息。帶 `--mark-tasks-complete` 時，章失效的判定在全勾之前做：被擋下時 tasks.md 一個字都不動，沒手測的 `[M]` 任務不會被代勾。某一站的工單開立中時，那一站的章不入失效判定，由未結工單守門處理。remote 通道沒有工作樹可讀，只判任務錨、不判內容錨。

### 6. delta 能合併進正典

引擎先讀完全部 capability 的 delta 與正典，做完全部驗證、產生合併計畫，全部通過才開始寫。以下任一情形拒絕：

1. ADDED 的需求名已經存在於正典。
2. MODIFIED、REMOVED、RENAMED 的來源需求名不存在於正典。
3. 同一個需求名出現在同一份 delta 的多個操作區段（含 RENAMED 的 FROM／TO 與其他區段互撞）。
4. RENAMED 的目標名已經存在於正典。
5. MODIFIED 區塊漏掉了正典既有的 scenario，又沒有附刪除聲明。
6. 正典還沒有的 capability 出現 ADDED 以外的操作。
7. 正典還沒有的 capability，delta 的 Purpose 不合格：缺 `## Purpose` 區段、內容為空、或 trim 後不足 50 個字元。

拒絕時一次列出全部違規，每條寫明 capability、操作、需求名與原因，並附補救路線：

- 第 1 到 6 類是「過期」：先跑 drift，再用 ingest 更新 delta。見 [續作與需求變更](drift-ingest.md)。
- 第 7 類是「Purpose」：補寫 `## Purpose` 區段，並用 `speclink validate` 取得完整指引。

這道守門沒有旁路旗標。`--no-validate` 只略過文件驗證，不解鎖合併守門；`--skip-specs` 是整段跳過規格套用，維持既有語意。

> [!TIP]
> MODIFIED 整塊取代正典需求。正典需求原有的每個 scenario 名稱都要出現在 delta 裡；真的要刪掉某個 scenario，就在 MODIFIED 區塊內加一行 REMOVED-SCENARIO 註解明示放棄，一行一個。漏掉又沒聲明，封存會逐條點名遺失的 scenario 名稱。聲明註解寫進正典前會被剝除。

## 新 capability 的 Purpose

delta 新開一個正典還沒有的 capability 時，delta 檔頂部要有一段 `## Purpose`，一兩句、50 個字元以上（以字元計，中文一個字算一個）。封存時這段內容會複製成新正典規格的 Purpose。既有 capability 的正典 Purpose 不會被 delta 改動；既有 capability 的 delta 帶 Purpose 也不會構成拒絕理由。

這條規則在三處共用同一個門檻：

- **變更驗證**：`speclink validate <變更名>` 對新開 capability 缺合格 Purpose 時報 error，訊息附 `## Purpose` 的範例骨架。同時，新開的 capability 名稱與既有名稱相近時報 warning 並列出近似名，提醒你可能該用既有的名字；這個 warning 不影響驗證結果。
- **封存守門**：上面的第 7 類。
- **正典規格驗證**：`speclink validate --specs` 逐份驗證正典規格。缺 `## Purpose` 或內容為空報 error；不足 50 字元只在 `--strict` 時報 warning；內容仍是封存佔位文字時報 warning。`--all` 同時驗變更與規格。`--specs` 不能和變更名一起給，錯誤訊息會指路單獨 `--specs` 或 `--all`。

## 寫入的順序

全部驗證通過後，引擎依序做三件事：先把所有受影響正典的封存前備份寫進快照目錄，再把合併結果寫回正典，最後把變更目錄搬進封存區。寫到一半 I/O 失敗，可以用已落地的快照與 git 恢復。

## 封存後會留下什麼

- **正典規格裡的 trace 區塊**：每條 ADDED 或 MODIFIED 的需求後面，引擎一律注入一個 trace 區塊，只有兩欄：來源變更名與封存日期。不含檔案清單。
- **變更的三站欄位**：封存目錄裡的 .openspec.yaml 同時保留建立、開工、封存三站的時間與人；開工欄位不會被剝掉。
- **任務證據**：變更目錄的 .evidence.json 跟著搬進封存區。
- **蓋章的變更不含工單檔**：蓋章時工單已刪除。只有帶 `--carry-*` 搬走的未結工單會出現在封存目錄裡。

### 零證據提示

封存一個沒有任何任務證據記錄的變更時，stderr 會出現恰好一行提示，點名變更名、說明沒有任務證據記錄。這不擋封存，也不影響 exit code。看到提示時，確認這個變更是不是漏走了 apply 流程；純規格或純文件的變更本來就零證據，屬正常。有任何一筆證據時，一個字都不印。

## 批次封存

`speclink archive --all` 或一次給多個變更名時，引擎先做就緒預檢，跳過未就緒的變更並回報原因；delta 過期的理由寫的是「archive 將拒絕」。新開 capability 缺 `## Purpose` 的變更，預檢會點名該 capability。linked worktree 守門同樣擋批次。章失效的變更會中止整批並點名，不會靜默跳過。

## 封存之後

封存完成後，技能提醒你用一般的 git 提交收尾這次封存產生的異動：delta 併入正典規格、變更目錄搬進封存區。這只是提醒，技能不會代跑提交。commit 技能的「挑選變更檔案」流程不適用於封存之後。

**出處**：`archive-skill`、`archive-merge`、`change-lifecycle`、`verify-evidence`、`spec-validation`
