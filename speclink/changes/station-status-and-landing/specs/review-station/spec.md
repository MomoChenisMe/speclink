## ADDED Requirements

### Requirement: 蓋章記錄程式碼的 commit

審查蓋章 SHALL 接受由呼叫端提供的 commit，並在寫入章欄位的同一次寫入裡把它記進 metadata 的 `reviewed_commit`。CLI 的 `speclink review stamp` SHALL 以 checkout 目前的 HEAD 提供它（本機與 remote 模式皆然）；讀不到 HEAD 時 SHALL 不提供，蓋章照常進行。commit SHALL 是 7 到 64 個十六進位字元，否則蓋章 SHALL 以非零 exit code 拒絕且不寫入。引擎 SHALL NOT 讀取 git，SHALL NOT 檢查 commit 是否存在，SHALL NOT 以它判斷章是否有效：章的凍結判定仍然只看任務錨與內容指紋錨。清除章的每一條既有路徑 SHALL 一併清除 `reviewed_commit`。沒有提供 commit 時 SHALL NOT 寫入這個鍵。`speclink show <change> --json` SHALL 增列選填的 `review` 物件（`state`，以及蓋章後的 `stampedAt`、`rounds`、`tasksTotal` 與有值才出現的 `stampedBy`、`stampedWith`、`commit`）；`speclink list --json` 的形狀 SHALL 維持不變。蓋章的 stdout 與 exit code SHALL 維持既有的位元級輸出。

#### Scenario: 蓋章記下 HEAD

- **WHEN** checkout 的 HEAD 是某個 commit，對符合蓋章條件的 demo 執行 speclink review stamp demo
- **THEN** metadata 含 `reviewed_commit`，值是該 commit；stdout 與新增這條需求前相同

#### Scenario: 不是 git checkout 時照常蓋章

- **WHEN** 在不是 git checkout 的資料夾對符合條件的 demo 蓋章
- **THEN** 蓋章成功，metadata 沒有 `reviewed_commit`

#### Scenario: 清章時一併清除

- **WHEN** 已蓋章並記有 `reviewed_commit` 的 demo 經既有的路徑失去這個關卡的章
- **THEN** metadata 不再有 `reviewed_commit`

#### Scenario: commit 不影響凍結判定

- **WHEN** 蓋章後 checkout 前進到新的 commit，但 scope 內的檔案內容與任務數都沒有變
- **THEN** 這個關卡的章仍然判定為有效

#### Scenario: show --json 帶出關卡狀態

- **WHEN** 對已蓋章的 demo 執行 speclink show demo --json
- **THEN** 輸出的 review 物件含 state `stamped`、stampedAt、rounds、tasksTotal 與 commit

#### Scenario: list --json 不變

- **WHEN** 比對蓋章前後 speclink list --json 的輸出形狀
- **THEN** 欄位集合相同
