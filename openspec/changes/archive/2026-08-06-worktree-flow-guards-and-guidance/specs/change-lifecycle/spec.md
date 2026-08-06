## ADDED Requirements

### Requirement: 封存的 linked worktree 環境守門

封存動詞（單筆與 bulk）SHALL 於任何檔案效果之前判定執行環境：workspace root 的 .git 為檔案（linked worktree 特徵）且 git 回報的當前分支具 speclink/ 前綴時 SHALL 拒絕封存——非零 exit code，stderr 說明封存不得於 linked worktree 內執行、並指路先以 worktree-merge 合回主分支再封存；change 目錄、正典規格與解封存備份目錄 SHALL 維持零變動。.git 為目錄（主 checkout）時本守門 SHALL NOT spawn git 且封存行為不變。git 不可用、指令失敗或分支輸出為空（detached HEAD）時 SHALL 放行（fail-open，沿 worktree discovery 的既有慣例：無 git 的環境不得因此無法封存）；分支無 speclink/ 前綴時 SHALL 放行。

#### Scenario: worktree 內封存被拒且零檔案效果

- **WHEN** 於分支 speclink/some-change 的 linked worktree 內對任一 change 執行封存
- **THEN** exit code 非零；stderr 含 worktree 事實與 worktree-merge 指路；該 change 目錄仍在原位，無正典規格寫入亦無備份目錄產生

#### Scenario: 非 speclink 分支的 worktree 放行

- **WHEN** workspace root 的 .git 為檔案、當前分支為 feature/anything，執行封存
- **THEN** 封存行為與主 checkout 完全相同

#### Scenario: 主 checkout 零額外開銷

- **WHEN** workspace root 的 .git 為目錄，執行封存
- **THEN** 本守門不 spawn git，封存行為與導入前完全相同

#### Scenario: git 不可用時 fail-open

- **WHEN** workspace root 的 .git 為檔案但 git 不可用，執行封存
- **THEN** 封存照常執行
