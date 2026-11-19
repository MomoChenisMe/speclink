---
title: 分支與落地記錄
section: SDD 工作流
order: 182
keywords: [分支, 落地, landed, change landed, commit, 開工]
sources: [change-landing, "commit-skill#提交後回報落地", "worktree-merge-skill#合併後回報落地"]
generated: 2026-11-19T16:40:41+08:00
---

# 分支與落地記錄

每個變更可以帶兩筆記錄：它在哪個分支上開工，以及它的程式碼最後落在哪個 commit。之後有人問「這個變更的程式碼在哪裡」，就查得到。

> [!NOTE]
> Speclink 只負責記下與讀出這兩筆資料，不依它們做任何判斷，也不檢查分支或 commit 是否真的存在。「落地後才能封存」這類規則不是內建的；需要的團隊由自己的系統（內嵌了引擎的宿主）把關。

## 開工時記下分支

執行 `speclink in-progress add <變更>` 標記開工時，CLI 會把 checkout 目前的分支名稱一起記進這個變更的資料。你不用多做任何事。

- checkout 不在任何分支上（detached HEAD）時不記分支，開工標記照常寫入。
- 資料夾不是 git checkout 時也不記。專案資料夾自己不是 git repo、只是放在別的 repo 底下，同樣算不是 git checkout。
- git 給的分支名稱不合規則時不記，指令不會因此失敗。
- 變更已經開工時，在別的分支上再執行一次，不會改寫已記下的分支。
- 執行 `speclink in-progress remove <變更>` 退回提案中時，分支的記錄會一併移除。

分支名稱的規則是 1 到 255 個字元，不含空白與控制字元。含斜線或中文的名稱（例如 `功能/登入`）可以記。

## 回報程式碼落在哪個 commit

```
speclink change landed <變更> [--commit <sha>] [--branch <分支>]
```

- 不帶 `--commit` 時，CLI 用 checkout 目前的 HEAD。
- 帶 `--branch` 時，覆寫先前記下的分支。
- 記下的內容是落地的 commit、回報的時間與回報的人。匿名時不記回報的人。
- 成功時印出一行 `✓ <變更> landed at <commit 的前 12 個字元>`。加 `--json` 時改印變更名稱、完整的 commit 與分支（有記才出現）。

還有幾件事：

- 任何階段的變更都能回報，提案中的也可以。回報不會改變變更的階段。
- 重複回報會覆寫上一次的記錄。
- 封存時，這些記錄跟著變更一起保留。
- 本機專案與遠端專案的用法與輸出相同。

### 會被拒絕的情況

| 情況 | 結果 |
| --- | --- |
| 不帶 `--commit`，資料夾又不是 git checkout | 非零結束，訊息說明要給 `--commit`，或在 git checkout 裡執行 |
| commit 不是 7 到 64 個十六進位字元 | 非零結束 |
| `--branch` 給的名稱不合上面的規則 | 非零結束 |
| 變更不存在 | 非零結束，訊息說明找不到這個變更 |
| 變更的資料無法解析，或被手改成寫入後讀不回來的形式 | 拒絕 |
| 連到內嵌了引擎的 Server，而它的規則不准這次回報 | 拒絕 |

任何一種情況被拒絕時，變更的資料都不會有任何改動。

## 技能會替你回報

兩支技能在完成自己的工作之後會執行 `speclink change landed`，所以多數時候你不用手動回報。

| 技能 | 什麼時候回報 | 回報什麼 |
| --- | --- | --- |
| `/speclink-commit` | git commit 成功之後 | 提交後的 HEAD |
| `/speclink-worktree-merge` | 變更分支合併進主分支成功之後 | 合併後主分支的 HEAD，分支記為 `speclink/<變更>` |

`/speclink-commit` 在兩種情況不回報：

- 你在 worktree 的變更分支上提交。這時落地由 `/speclink-worktree-merge` 在合併後回報。
- 這次提交先跑了封存。變更已經封存，回報只會得到「找不到變更」的錯誤。

提交失敗、你取消提交，或合併失敗、中止時，技能都不回報。

回報被拒絕時（例如連到的 Server 不支援這個動作），技能把訊息告訴你，然後繼續。它不會取消或重做已經完成的提交，也不會還原合併。

> [!NOTE]
> 本機專案的回報會寫進這個變更的資料檔，所以回報之後工作樹會多一個改動的檔。`/speclink-commit` 會告訴你它留給這個變更的下一次提交。`/speclink-worktree-merge` 會告訴你主 checkout 多了這個改動的檔；它不替你提交，並在交棒時點名這個檔。遠端專案沒有這個檔。

兩支技能的其他步驟見[提交單一變更的檔案](commit.md)與[平行實作與合回：worktree](worktree.md)。

## 怎麼讀回來

執行 `speclink show <變更> --json`。輸出會帶出分支與落地記錄：落地的 commit、回報的時間，以及回報的人（有記才出現）。變更沒有這些記錄時，輸出裡就沒有這兩項。

`speclink show <變更>` 的一般輸出不顯示這兩筆記錄。`speclink list --json` 的欄位也不因它們而改變。

**出處**：`change-landing`、`commit-skill`、`worktree-merge-skill`
